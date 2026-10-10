#!/usr/bin/env python3
"""Ubuntu Server wizard. No OS changes are made by --plan."""
from __future__ import annotations
import argparse
import getpass
from contextlib import contextmanager
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import platform
import pwd
import re
import shutil
import sqlite3
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.request
from common import *

SOURCE = Path(__file__).resolve().parent.parent
STAGE = "前提条件"

def prompt(label, default=""):
    value = input(f"{label}" + (f" [{default}]" if default != "" else "") + ": ").strip()
    return value or str(default)

def choice(label, values, default):
    while True:
        value = prompt(label + " (" + "/".join(values) + ")", default)
        if value in values:
            return value
        print("選択肢から入力してください。")

def secret(label, minimum=8):
    while True:
        value = getpass.getpass(label + ": ")
        if len(value) < minimum or len(value) > 256:
            print(f"{minimum}文字以上256文字以下で入力してください。")
            continue
        if value == getpass.getpass("もう一度入力: "):
            return value
        print("入力が一致しません。")

def detected_address():
    try:
        routes = json.loads(subprocess.check_output(["ip", "-j", "route", "get", "1.1.1.1"]))
        return routes[0].get("prefsrc", "127.0.0.1")
    except Exception:
        return "127.0.0.1"

def deployment_paths():
    return [STATE, ETC / "reservoir.toml", ETC / "basic.htpasswd", SITE, LIMITS,
             ETC / "tls/server.crt", ETC / "tls/server.key",
             Path("/etc/systemd/system/reservoir.service"),
             Path("/etc/systemd/system/reservoir-backup.service"),
             Path("/etc/systemd/system/reservoir-backup.timer")]

@contextmanager
def deployment_rollback(previous):
    """Restore application configuration after failure; installed OS packages are kept."""
    paths = deployment_paths()
    snapshot = {p: (p.read_bytes(), p.stat().st_mode & 0o777, p.stat().st_uid, p.stat().st_gid)
                if p.is_file() else None for p in paths}
    try:
        yield
    except (Exception, KeyboardInterrupt):
        for path, saved in snapshot.items():
            if saved:
                data, mode, uid, gid = saved
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
                path.chmod(mode)
                os.chown(path, uid, gid)
            else:
                path.unlink(missing_ok=True)
        if previous:
            try:
                firewall(previous)
                run(["timedatectl", "set-timezone", previous["timezone"]])
                run(["nginx", "-t"])
                run(["systemctl", "reload", "nginx"])
                run(["systemctl", "daemon-reload"])
                run(["systemctl", "restart", "reservoir", "reservoir-backup.timer"])
            except Exception as rollback_error:
                print("設定の復帰後のサービス起動を確認してください:", rollback_error, file=sys.stderr)
        else:
            subprocess.run(["systemctl", "stop", "reservoir"], check=False)
            enabled = Path("/etc/nginx/sites-enabled/reservoir")
            if enabled.is_symlink():
                enabled.unlink()
            subprocess.run(["nginx", "-t"], check=False)
            subprocess.run(["systemctl", "reload", "nginx"], check=False)
        raise

def wizard(existing=None):
    old = existing or {}
    mode = choice("公開範囲", ["lan", "internet"], old.get("mode", "lan"))
    host = prompt("アクセス用ホスト名またはIPv4アドレス", old.get("hostname", detected_address() if mode == "lan" else ""))
    default_network = old.get("lan_networks", ["192.168.1.0/24"])
    networks = prompt("LAN許可ネットワーク（カンマ区切り、実際のLANに合わせて変更）", ",".join(default_network))
    tls_options = ["selfsigned", "existing"] if mode == "lan" else ["existing", "acme"]
    tls_default = old.get("tls", "selfsigned" if mode == "lan" else "acme")
    if mode == "lan" and tls_default == "acme":
        tls_default = "existing"
    if tls_default not in tls_options:
        tls_default = tls_options[0]
    tls = choice("HTTPS方式", tls_options, tls_default)
    config = {
        "mode": mode, "hostname": host,
        "lan_networks": [str(ipaddress.ip_network(n.strip(), strict=False)) for n in networks.split(",") if n.strip()],
        "tls": tls, "basic_auth": choice("共通Basic認証", ["on", "off"], "on" if old.get("basic_auth", True) else "off") == "on",
        "ssh_port": int(prompt("現在のSSHポート（通信を維持するため許可）", old.get("ssh_port", 22))),
        "timezone": choice("OSタイムゾーン", ["Asia/Tokyo", "UTC"], old.get("timezone", "Asia/Tokyo")),
        "backup_dir": prompt("バックアップ保存先（絶対パス）", old.get("backup_dir", "/var/backups/reservoir")),
        "backup_time": prompt("毎日のバックアップ時刻", old.get("backup_time", "03:00")),
        "backup_days": int(prompt("バックアップ保持日数", old.get("backup_days", 30))),
        "lab_name": prompt("研究室の表示名", old.get("lab_name", "研究室")),
        "admin_hash": old.get("admin_hash", ""),
    }
    if tls == "existing":
        config["certificate"] = prompt("証明書チェーンの絶対パス", old.get("certificate", "/etc/ssl/reservoir/fullchain.pem"))
        config["certificate_key"] = prompt("秘密鍵の絶対パス", old.get("certificate_key", "/etc/ssl/reservoir/privkey.pem"))
    elif tls == "acme":
        config["certificate"] = f"/etc/letsencrypt/live/{host}/fullchain.pem"
        config["certificate_key"] = f"/etc/letsencrypt/live/{host}/privkey.pem"
        config["email"] = prompt("証明書管理者のメールアドレス", old.get("email", ""))
        if not re.fullmatch(r"[^\s@]+@[^\s@]+\.[^\s@]+", config["email"]):
            raise ValueError("証明書管理者のメールアドレスが不正です")
    else:
        config["certificate"] = "/etc/reservoir/tls/server.crt"
        config["certificate_key"] = "/etc/reservoir/tls/server.key"
    validate(config)
    passwords = {}
    if not old or choice("管理者パスワードを変更", ["yes", "no"], "no") == "yes":
        passwords["admin"] = secret("管理者パスワード")
    if config["basic_auth"] and (not (ETC / "basic.htpasswd").exists() or choice("共通Basic認証の認証情報を変更", ["yes", "no"], "no") == "yes"):
        config["basic_user"] = prompt("共通Basic認証の利用者名", old.get("basic_user", "lab"))
        if not re.fullmatch(r"[A-Za-z0-9_-]{1,64}", config["basic_user"]):
            raise ValueError("Basic認証の利用者名が不正です")
        passwords["basic"] = secret("共通Basic認証のパスワード")
    elif old.get("basic_user"):
        config["basic_user"] = old["basic_user"]
    return config, passwords

def ensure_ubuntu():
    release = dict(line.split("=", 1) for line in Path("/etc/os-release").read_text().splitlines() if "=" in line)
    if release.get("ID", "").strip('"') != "ubuntu":
        raise RuntimeError("Ubuntu Serverが必要です")
    if platform.machine() not in ("aarch64", "arm64", "x86_64"):
        raise RuntimeError("ARM64または検証用x86_64環境が必要です")
    if int(release.get("VERSION_ID", "0").strip('"').split(".")[0]) < 24:
        raise RuntimeError("Ubuntu Server 24.04以降が必要です")
    print("検出OS:", release.get("PRETTY_NAME", "").strip('"'), platform.machine())
    if shutil.disk_usage(SOURCE).free < 4 * 1024**3:
        raise RuntimeError("ソースビルドには4GB以上の空き容量を確保してください")

def create_user(name, home):
    try:
        pwd.getpwnam(name)
    except KeyError:
        run(["useradd", "--system", "--user-group", "--home-dir", home, "--shell", "/usr/sbin/nologin", name])
    Path(home).mkdir(parents=True, exist_ok=True)
    record = pwd.getpwnam(name)
    os.chown(home, record.pw_uid, record.pw_gid)

def builder(args, home, **kwargs):
    env = dict(os.environ, HOME=str(home), CARGO_HOME=str(home / ".cargo"), RUSTUP_HOME=str(home / ".rustup"))
    env["PATH"] = str(home / ".cargo/bin") + ":" + str(home / "node/bin") + ":/usr/local/bin:/usr/bin:/bin"
    return run(["runuser", "-u", "reservoir-build", "--", "env"] + [f"{k}={env[k]}" for k in ("HOME", "CARGO_HOME", "RUSTUP_HOME", "PATH")] + list(args), **kwargs)

def download(url, target):
    with urllib.request.urlopen(url, timeout=90) as response, open(target, "wb") as output:
        shutil.copyfileobj(response, output)

def build_release():
    home = Path("/var/cache/reservoir-build")
    create_user("reservoir-build", home)
    record = pwd.getpwnam("reservoir-build")
    work = home / "source"
    work.mkdir(exist_ok=True)
    os.chown(work, record.pw_uid, record.pw_gid)
    for section in ("backend", "frontend"):
        target = work / section
        if target.exists():
            # This is a dedicated build cache, never a data or user source directory.
            if target.resolve().parent != work.resolve():
                raise RuntimeError("Invalid build cache path")
            shutil.rmtree(target)
        shutil.copytree(SOURCE / section, target, ignore=shutil.ignore_patterns("node_modules", "target", "dist", "build", "db", "reservoir.toml", ".env*", ".test", "test-results", "playwright-report"))
    for directory, dirs, files in os.walk(work):
        os.chown(directory, record.pw_uid, record.pw_gid)
        for file in files:
            os.chown(Path(directory) / file, record.pw_uid, record.pw_gid)
    if not (home / ".cargo/bin/rustup").exists():
        setup = home / "rustup-init.sh"
        download("https://sh.rustup.rs", setup)
        os.chown(setup, record.pw_uid, record.pw_gid)
        builder(["sh", str(setup), "-y", "--profile", "minimal", "--default-toolchain", "1.98.1"], home)
    else:
        builder([str(home / ".cargo/bin/rustup"), "toolchain", "install", "1.98.1", "--profile", "minimal"], home)
    if not (home / "node/bin/node").exists():
        with urllib.request.urlopen("https://nodejs.org/dist/index.json", timeout=90) as response:
            releases = json.load(response)
        version = next(item["version"] for item in releases if item["version"].startswith("v24.") and item.get("lts"))
        arch = "arm64" if platform.machine() in ("aarch64", "arm64") else "x64"
        filename = f"node-{version}-linux-{arch}.tar.xz"
        archive = home / filename
        download(f"https://nodejs.org/dist/{version}/{filename}", archive)
        with urllib.request.urlopen(f"https://nodejs.org/dist/{version}/SHASUMS256.txt", timeout=90) as response:
            hashes = dict((line.split()[1], line.split()[0]) for line in response.read().decode().splitlines())
        if hashlib.sha256(archive.read_bytes()).hexdigest() != hashes[filename]:
            raise RuntimeError("Node.js checksum mismatch")
        with tarfile.open(archive) as bundle:
            bundle.extractall(home, filter="data")
        (home / f"node-{version}-linux-{arch}").rename(home / "node")
    builder(["cargo", "+1.98.1", "build", "--release", "--locked", "-j", "1"], home, cwd=work / "backend")
    builder(["npm", "ci"], home, cwd=work / "frontend")
    builder(["npm", "run", "build"], home, cwd=work / "frontend")
    release = ROOT / "releases" / time.strftime("%Y%m%d-%H%M%S")
    release.mkdir(parents=True)
    shutil.copy2(work / "backend/target/release/reservoir", release / "reservoir")
    shutil.copytree(work / "frontend/dist", release / "frontend")
    for directory, dirs, files in os.walk(release):
        os.chmod(directory, 0o755)
        for file in files:
            os.chmod(Path(directory) / file, 0o755 if file == "reservoir" else 0o644)
    return release

def credentials(config, passwords, binary):
    if "admin" in passwords:
        config["admin_hash"] = subprocess.check_output([str(binary), "hash-password"], input=(passwords["admin"] + "\n").encode()).decode().strip()
    if "basic" in passwords:
        output = subprocess.check_output(["htpasswd", "-niB", config["basic_user"]], input=(passwords["basic"] + "\n").encode()).decode()
        atomic_write(ETC / "basic.htpasswd", output, 0o640)
        import grp
        os.chown(ETC / "basic.htpasswd", 0, grp.getgrnam("www-data").gr_gid)
    passwords.clear()

def setup_tls(config):
    if config["tls"] == "selfsigned":
        path = ETC / "tls"
        path.mkdir(exist_ok=True)
        # Generate again after a name change; keep files for an unchanged name.
        previous = load_state() if STATE.exists() else {}
        if not Path(config["certificate"]).exists() or previous.get("hostname") != config["hostname"]:
            host = config["hostname"]
            try:
                ipaddress.ip_address(host)
                san = "IP:" + host
            except ValueError:
                san = "DNS:" + host
            run(["openssl", "req", "-x509", "-newkey", "rsa:3072", "-sha256", "-days", "825", "-nodes", "-keyout", config["certificate_key"], "-out", config["certificate"], "-subj", "/CN=" + host, "-addext", "subjectAltName=" + san], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            os.chmod(config["certificate_key"], 0o600)
    elif config["tls"] == "acme":
        Path("/var/lib/reservoir-acme").mkdir(exist_ok=True)
        install_nginx(config, bootstrap=True)
        run(["certbot", "certonly", "--webroot", "-w", "/var/lib/reservoir-acme", "--non-interactive", "--agree-tos", "--email", config["email"], "-d", config["hostname"], "--keep-until-expiring"])
        atomic_write(Path("/etc/letsencrypt/renewal-hooks/deploy/reservoir-nginx"), "#!/bin/sh\nnginx -t && systemctl reload nginx\n", 0o755)
    if not Path(config["certificate"]).is_file() or not Path(config["certificate_key"]).is_file():
        raise RuntimeError("証明書と秘密鍵が見つかりません")

def firewall(config, previous=None):
    # Remove only installer-tagged web rules; numbered deletion preserves other applications.
    output = subprocess.check_output(["ufw", "status", "numbered"], env=dict(os.environ, LC_ALL="C")).decode()
    numbers = [int(match.group(1)) for line in output.splitlines()
               if (match := re.match(r"^\[\s*(\d+)\].*# reservoir-(?:web|lan)\s*$", line))]
    for number in sorted(numbers, reverse=True):
        run(["ufw", "--force", "delete", str(number)])
    run(["ufw", "allow", str(config["ssh_port"]) + "/tcp", "comment", "reservoir-ssh"])
    if config["mode"] == "internet":
        for port in ("80", "443"):
            run(["ufw", "allow", port + "/tcp", "comment", "reservoir-web"])
    else:
        for network in config["lan_networks"]:
            for port in ("80", "443"):
                run(["ufw", "allow", "from", network, "to", "any", "port", port, "proto", "tcp", "comment", "reservoir-lan"])
    run(["ufw", "--force", "enable"])

def install_units(config):
    for name in ("reservoir.service", "reservoir-backup.service"):
        atomic_write(Path("/etc/systemd/system") / name, (SOURCE / "deploy" / name).read_text(), 0o644)
    timer = (SOURCE / "deploy/reservoir-backup.timer").read_text().replace("03:00:00", config["backup_time"] + ":00")
    atomic_write(Path("/etc/systemd/system/reservoir-backup.timer"), timer, 0o644)
    lib = Path("/usr/local/lib/reservoir")
    lib.mkdir(parents=True, exist_ok=True)
    for name in ("common.py", "install.py", "reservoirctl.py", "repair.py"):
        shutil.copy2(SOURCE / "deploy" / name, lib / name)
        os.chmod(lib / name, 0o644)
    atomic_write(Path("/usr/local/bin/reservoirctl"), "#!/bin/sh\nexec python3 /usr/local/lib/reservoir/reservoirctl.py \"$@\"\n", 0o755)
    run(["systemctl", "daemon-reload"])

def network_wizard():
    if choice("最後に固定IPを設定しますか（現在のSSH接続が切れる場合があります）", ["yes", "no"], "no") != "yes":
        return
    names = [p.name for p in Path("/sys/class/net").iterdir() if p.name != "lo"]
    device = prompt("ネットワークインターフェース " + ",".join(names), names[0] if names else "")
    if device not in names:
        raise ValueError("インターフェースが見つかりません")
    address = str(ipaddress.ip_interface(prompt("IPv4アドレス/プレフィックス")))
    gateway = str(ipaddress.IPv4Address(prompt("ゲートウェイ")))
    dns = [str(ipaddress.ip_address(v.strip())) for v in prompt("DNS（カンマ区切り）").split(",")]
    network = {"network": {"version": 2, "ethernets": {device: {"dhcp4": False, "dhcp6": False, "addresses": [address], "routes": [{"to": "default", "via": gateway}], "nameservers": {"addresses": dns}}}}}
    target = Path("/etc/netplan/99-reservoir.yaml")
    old = target.read_text() if target.exists() else None
    atomic_write(target, json.dumps(network, indent=2) + "\n")
    try:
        run(["netplan", "generate"])
        print("Netplanが一時適用します。接続を確認し、プロンプトで確定してください。確定しない場合は戻ります。")
        run(["netplan", "try", "--timeout", "120"])
    except Exception:
        if old is None:
            target.unlink(missing_ok=True)
        else:
            atomic_write(target, old)
        run(["netplan", "generate"])
        raise

def verify():
    for attempt in range(30):
        try:
            with urllib.request.urlopen("http://127.0.0.1:8080/api/health", timeout=3) as response:
                if json.load(response)["status"] == "ok":
                    return
        except Exception:
            time.sleep(1)
    raise RuntimeError("APIが起動しません。journalctl -u reservoir を確認してください")

def main():
    global STAGE
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", action="store_true", help="導入内容だけ表示し、OSを変更しない")
    parser.add_argument("--import-db", type=Path, help="旧DBのコピーを初回導入時に移行")
    parser.add_argument("--auto-assume", action="store_true", help="バックアップ後、不正期間を反転し同時刻予約を削除する")
    args = parser.parse_args()
    if args.plan:
        print("Ubuntu確認 → 公開範囲・認証・HTTPS・バックアップ入力 → OS依存・時刻同期 → 専用ユーザーでビルド → DB移行 → Nginx/systemd → バックアップ・動作検査 → 任意の固定IP設定")
        return
    if os.geteuid() != 0:
        parser.error("sudo bash install.sh で実行してください")
    ensure_ubuntu()
    if shutil.which("cloud-init"):
        run(["cloud-init", "status", "--wait"])
    previous = load_state() if STATE.exists() else None
    if args.import_db and (DB.exists() or not args.import_db.is_file()):
        parser.error("--import-dbは初回導入時の既存DBコピーにのみ使用できます")
    config, passwords = wizard(previous)
    assume = args.auto_assume or choice("不整合予約をAuto-assumeする（開始終了を反転・同時刻は削除）", ["yes", "no"], "no") == "yes"
    print("予約のAuto-assume:", "ON" if assume else "OFF")
    print("\n設定概要:", json.dumps({k:v for k,v in config.items() if k not in ("admin_hash",)}, ensure_ascii=False, indent=2))
    if choice("この設定でインストールしますか", ["yes", "no"], "no") != "yes":
        return
    with deployment_rollback(previous):
        STAGE = "OS依存の導入"
        run(["apt-get", "update"])
        run(["apt-get", "install", "-y", "nginx", "sqlite3", "openssl", "python3", "python3-venv", "curl", "ca-certificates", "build-essential", "pkg-config", "apache2-utils", "ufw", "certbot", "unattended-upgrades", "xz-utils"])
        run(["timedatectl", "set-timezone", config["timezone"]])
        run(["timedatectl", "set-ntp", "true"])
        atomic_write(Path("/etc/apt/apt.conf.d/20auto-upgrades"), 'APT::Periodic::Update-Package-Lists "1";\nAPT::Periodic::Unattended-Upgrade "1";\n', 0o644)
        create_user("reservoir", "/var/lib/reservoir")
        ETC.mkdir(mode=0o755, exist_ok=True)
        STAGE = "専用ユーザーによるビルド"
        release = build_release()
        credentials(config, passwords, release / "reservoir")
        STAGE = "HTTPSと公開範囲"
        firewall(config, previous)
        setup_tls(config)
        STAGE = "DBバックアップとリリース切替"
        old_release = (ROOT / "current").resolve() if (ROOT / "current").exists() else None
        backup_path = None
        subprocess.run(["systemctl", "stop", "reservoir"], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if DB.exists():
            from reservoirctl import backup
            backup_path = backup(config)
        elif args.import_db:
            with sqlite3.connect(f"file:{args.import_db.resolve()}?mode=ro", uri=True) as source, sqlite3.connect(DB) as dest:
                source.backup(dest)
        save_state(config)
        temp_link = ROOT / "current.new"
        temp_link.unlink(missing_ok=True)
        temp_link.symlink_to(release)
        os.replace(temp_link, ROOT / "current")
        import grp
        record = pwd.getpwnam("reservoir")
        if DB.exists():
            os.chown(DB, record.pw_uid, record.pw_gid)
            os.chmod(DB, 0o600)
        try:
            if assume and DB.exists():
                STAGE = "予約のAuto-assume"
                from repair import auto_assume
                result = auto_assume(DB, config["backup_dir"])
                print("Auto-assume結果:", json.dumps(result, ensure_ascii=False))
            STAGE = "APIの起動と動作確認"
            install_units(config)
            run(["systemctl", "enable", "--now", "reservoir"])
            verify()
            if not previous:
                with sqlite3.connect(DB) as conn:
                    conn.execute("UPDATE app_settings SET lab_name=? WHERE id=1", (config["lab_name"],))
            install_nginx(config)
            run(["systemctl", "enable", "--now", "reservoir-backup.timer"])
            from reservoirctl import backup
            backup(config)
        except Exception:
            subprocess.run(["systemctl", "stop", "reservoir"], check=False)
            if backup_path and old_release and previous:
                with sqlite3.connect(backup_path) as source, sqlite3.connect(DB) as dest:
                    source.backup(dest)
                save_state(previous)
                (ROOT / "current").unlink()
                (ROOT / "current").symlink_to(old_release)
                run(["systemctl", "start", "reservoir"])
            raise
    print("\n導入完了: https://" + config["hostname"])
    print("定期バックアップと復元: sudo reservoirctl status / backup / restore")
    print("LAN・Basic認証の切替: sudo reservoirctl configure")
    print("インターネット公開時はDNS・ルーター・大学のネットワーク側でも到達性を確認してください。")
    if config["tls"] == "selfsigned":
        print("自己署名証明書です。利用端末に証明書の信頼設定が必要です:", config["certificate"])
    STAGE = "任意の固定IP設定"
    network_wizard()

if __name__ == "__main__":
    try:
        main()
    except (Exception, KeyboardInterrupt) as exc:
        print(f"\n停止した段階: {STAGE}\n{exc}\n既存DBを初期化せずに再実行できます。", file=sys.stderr)
        sys.exit(1)
