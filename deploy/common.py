"""Shared, independently testable deployment configuration."""
from __future__ import annotations
import ipaddress
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ETC = Path("/etc/reservoir")
STATE = ETC / "deployment.json"
SITE = Path("/etc/nginx/sites-available/reservoir")
LIMITS = Path("/etc/nginx/conf.d/reservoir-limits.conf")
DB = Path("/var/lib/reservoir/reservoir.db")
ROOT = Path("/opt/reservoir")

def run(args, **kwargs):
    return subprocess.run([str(v) for v in args], check=True, **kwargs)

def atomic_write(path: Path, content: str, mode=0o600):
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix="." + path.name, dir=path.parent)
    try:
        with os.fdopen(fd, "w") as stream:
            stream.write(content)
        os.chmod(name, mode)
        os.replace(name, path)
    finally:
        if os.path.exists(name):
            os.unlink(name)

def validate(config):
    base = config.get("base_path", "/")
    if base != "/" and not re.fullmatch(r"/(?:[A-Za-z0-9_-]+/)+", base):
        raise ValueError("公開パスは / または /reservation/ の形式で指定してください")
    if config["mode"] not in ("lan", "internet"):
        raise ValueError("公開モードが不正です")
    host = config["hostname"]
    if config.get("reject_ip_access", False):
        try:
            ipaddress.ip_address(host)
        except ValueError:
            pass
        else:
            raise ValueError("IP直打ちを拒否する場合はアクセス先にホスト名を指定してください")
    if not re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9.-]{0,252}", host) or ".." in host:
        raise ValueError("ホスト名またはIPv4アドレスを入力してください")
    if config["mode"] == "internet" and ("." not in host or host.endswith(".local")):
        raise ValueError("インターネット公開では公開ドメイン名を指定してください")
    if config["mode"] == "internet" and config["tls"] == "selfsigned":
        raise ValueError("公開ドメインにはACMEまたは既存の証明書を指定してください")
    if config["tls"] not in ("selfsigned", "existing", "acme"):
        raise ValueError("HTTPS方式が不正です")
    if config["mode"] == "lan" and config["tls"] == "acme":
        raise ValueError("LAN限定では自己署名または既存証明書を指定してください。HTTP検証によるACME取得・更新はインターネット公開時に使用できます")
    for network in config["lan_networks"]:
        ipaddress.ip_network(network, strict=False)
    if config["mode"] == "lan" and not config["lan_networks"]:
        raise ValueError("LANの許可ネットワークを指定してください")
    if not 1 <= int(config["ssh_port"]) <= 65535:
        raise ValueError("SSHポートが不正です")
    if not 1 <= int(config["backup_days"]) <= 3650:
        raise ValueError("バックアップ保持日数が不正です")
    if not re.fullmatch(r"(?:[01][0-9]|2[0-3]):[0-5][0-9]", config["backup_time"]):
        raise ValueError("時刻はHH:MM形式で入力してください")
    if config["timezone"] not in ("Asia/Tokyo", "UTC"):
        raise ValueError("サーバーのタイムゾーンを選択してください")
    for name in ("certificate", "certificate_key"):
        if not re.fullmatch(r"/[a-zA-Z0-9_./-]+", config[name]):
            raise ValueError("証明書のパスが不正です")
    if not Path(config["backup_dir"]).is_absolute() or Path(config["backup_dir"]).resolve() == Path("/"):
        raise ValueError("バックアップ先はルート以外の絶対パスにしてください")
    if len(config["lab_name"]) > 100 or not config["lab_name"].strip():
        raise ValueError("研究室名を確認してください")

def nginx(config, bootstrap=False):
    validate(config)
    host = config["hostname"]
    base = config.get("base_path", "/")
    redirect = f'location = {base.rstrip("/")} {{ return 308 {base}$is_args$args; }}' if base != "/" else ""
    outside = 'location / { return 404; }' if base != "/" else ""
    auth = 'auth_basic "Reservoir";\n    auth_basic_user_file /etc/reservoir/basic.htpasswd;' if config["basic_auth"] else "auth_basic off;"
    acl = ""
    if config["mode"] == "lan":
        networks = ["127.0.0.1/32", "::1/128"] + config["lan_networks"]
        acl = "\n".join("allow " + str(ipaddress.ip_network(n, strict=False)) + ";" for n in networks) + "\ndeny all;"
    challenge = """location ^~ /.well-known/acme-challenge/ {
        auth_basic off;
        allow all;
        root /var/lib/reservoir-acme;
    }"""
    public_acl = acl if config["mode"] == "lan" else ""
    # Numeric Host names are caught even when Ubuntu's default site remains enabled.
    # Do not claim default_server: other hosted applications may already own it.
    ip_names = '"~^[0-9.]+$" "~^\\[?[0-9A-Fa-f]*:[0-9A-Fa-f:.]*\\]?$"'
    deny_http = f"server {{ listen 80; listen [::]:80; server_name {ip_names}; return 444; }}\n" if config.get("reject_ip_access", False) else ""
    deny_https = f"""server {{
    listen 443 ssl;
    listen [::]:443 ssl;
    server_name {ip_names};
    ssl_certificate {config["certificate"]};
    ssl_certificate_key {config["certificate_key"]};
    ssl_protocols TLSv1.2 TLSv1.3;
    return 444;
}}
""" if config.get("reject_ip_access", False) else ""
    http = f"""server {{
    listen 80;
    listen [::]:80;
    server_name {host};
    {public_acl}
    {challenge}
    location / {{ return 301 https://$host$request_uri; }}
}}
"""
    if bootstrap:
        return deny_http + http
    return deny_http + deny_https + http + f"""server {{
    listen 443 ssl;
    listen [::]:443 ssl;
    server_name {host};
    ssl_certificate {config["certificate"]};
    ssl_certificate_key {config["certificate_key"]};
    ssl_protocols TLSv1.2 TLSv1.3;
    server_tokens off;
    root /opt/reservoir/current/frontend;
    index index.html;
    {acl}
    {auth}
    client_max_body_size 32k;
    add_header X-Content-Type-Options nosniff always;
    add_header Referrer-Policy same-origin always;
    add_header X-Frame-Options DENY always;
    {redirect}
    {outside}
    location = {base}api/admin/login {{
        limit_req zone=reservoir_login burst=5 nodelay;
        limit_req_status 429;
        proxy_pass http://127.0.0.1:8080/api/admin/login;
        proxy_cookie_path /api {base}api;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-For $remote_addr;
    }}
    location {base}api/ {{
        limit_req zone=reservoir_api burst=30 nodelay;
        limit_req_status 429;
        proxy_pass http://127.0.0.1:8080/api/;
        proxy_cookie_path /api {base}api;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-For $remote_addr;
        proxy_read_timeout 120s;
        add_header Cache-Control no-store always;
    }}
    location {base}assets/ {{ try_files $uri =404; expires 1y; }}
    location {base} {{ try_files $uri $uri/ {base}index.html; add_header Cache-Control no-cache always; }}
}}
"""

def app_config(config):
    # JSON strings are also valid TOML basic strings; credentials are never shell arguments.
    return "\n".join([
        'web_url = "127.0.0.1:8080"',
        'db_path = "/var/lib/reservoir/reservoir.db"',
        "admin_password_hash = " + json.dumps(config["admin_hash"]),
        "secure_cookies = true",
        "basic_auth_enabled = " + str(config["basic_auth"]).lower(),
        "public_mode = " + json.dumps(config["mode"]),
    ]) + "\n"

def install_nginx(config, bootstrap=False):
    previous = SITE.read_text() if SITE.exists() else None
    atomic_write(LIMITS, 'limit_req_zone $binary_remote_addr zone=reservoir_api:10m rate=10r/s;\nlimit_req_zone $binary_remote_addr zone=reservoir_login:10m rate=5r/m;\n', 0o644)
    atomic_write(SITE, nginx(config, bootstrap), 0o644)
    link = Path("/etc/nginx/sites-enabled/reservoir")
    if not link.exists():
        link.symlink_to(SITE)
    try:
        run(["nginx", "-t"])
        run(["systemctl", "reload", "nginx"])
    except Exception:
        if previous is None:
            SITE.unlink(missing_ok=True)
            link.unlink(missing_ok=True)
        else:
            atomic_write(SITE, previous, 0o644)
        raise

def save_state(config):
    validate(config)
    atomic_write(STATE, json.dumps(config, ensure_ascii=False, indent=2) + "\n")
    atomic_write(ETC / "reservoir.toml", app_config(config), 0o640)
    import grp
    os.chown(ETC / "reservoir.toml", 0, grp.getgrnam("reservoir").gr_gid)

def load_state():
    return json.loads(STATE.read_text())
