#!/usr/bin/env python3
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import pwd
import shutil
import sqlite3
import subprocess
import sys
import time
from common import *

def backup(config):
    directory = Path(config["backup_dir"])
    if not DB.is_file():
        raise RuntimeError("データベースが見つかりません")
    directory.mkdir(parents=True, exist_ok=True, mode=0o700)
    destination = directory / ("reservoir-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S") + "-" + str(time.time_ns())[-6:] + ".db")
    try:
        with sqlite3.connect(DB) as source, sqlite3.connect(destination) as target:
            source.backup(target)
            if target.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
                raise RuntimeError("バックアップの整合性検査に失敗しました")
        destination.chmod(0o600)
        status = {"status": "ok", "completed_at": datetime.now(timezone.utc).isoformat(), "file": str(destination)}
        atomic_write(Path("/var/lib/reservoir/backup-status.json"), json.dumps(status) + "\n", 0o644)
        for old in directory.glob("reservoir-*.db"):
            if old.is_file() and not old.is_symlink() and time.time() - old.stat().st_mtime > config["backup_days"] * 86400:
                old.unlink()
        print("バックアップ:", destination)
        return destination
    except Exception:
        destination.unlink(missing_ok=True)
        atomic_write(Path("/var/lib/reservoir/backup-status.json"), json.dumps({"status":"failed","completed_at":datetime.now(timezone.utc).isoformat()})+"\n", 0o644)
        raise

def restore(config, source):
    source = source.resolve()
    if source == DB.resolve() or not source.is_file():
        raise ValueError("復元元のDBファイルを指定してください")
    with sqlite3.connect(f"file:{source}?mode=ro", uri=True) as conn:
        if conn.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
            raise ValueError("復元元の整合性検査に失敗しました")
        required = {"users","resources","reservations"}
        names = {row[0] for row in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")}
        if not required.issubset(names):
            raise ValueError("Reservoirのバックアップではありません")
    if input("現在の予約を復元元で置き換えます。RESTORE と入力して確定: ").strip() != "RESTORE":
        return
    run(["systemctl","stop","reservoir"])
    try:
        saved = backup(config)
    except Exception:
        run(["systemctl","start","reservoir"])
        raise
    try:
        with sqlite3.connect(f"file:{source}?mode=ro", uri=True) as old, sqlite3.connect(DB) as target:
            old.backup(target)
        record = pwd.getpwnam("reservoir")
        os.chown(DB, record.pw_uid, record.pw_gid)
        run(["systemctl","start","reservoir"])
        from install import verify
        verify()
    except Exception:
        subprocess.run(["systemctl","stop","reservoir"],check=False)
        with sqlite3.connect(saved) as old, sqlite3.connect(DB) as target:
            old.backup(target)
        run(["systemctl","start","reservoir"])
        raise
    print("復元しました。すべての端末で予約を再読み込みしてください。")

def configure(config):
    from install import wizard, credentials, setup_tls, firewall, install_units, verify
    new, passwords = wizard(config, allow_path_change=False)
    print("新しい公開範囲:",new["mode"],"Basic認証:","ON" if new["basic_auth"] else "OFF")
    if input("設定を反映しますか [yes/no]: ").strip() != "yes":
        return
    # Back up every file this operation may replace.
    rollback = Path("/var/backups/reservoir-config") / str(time.time_ns())
    rollback.mkdir(parents=True, mode=0o700)
    paths = [STATE, ETC/"reservoir.toml", ETC/"basic.htpasswd", SITE, LIMITS,
             Path("/etc/systemd/system/reservoir-backup.timer"), ETC/"tls/server.crt", ETC/"tls/server.key"]
    existing = {str(path): path.exists() for path in paths}
    for index, path in enumerate(paths):
        if path.exists():
            shutil.copy2(path, rollback/str(index))
    try:
        credentials(new, passwords, ROOT/"current/reservoir")
        firewall(new, config)
        setup_tls(new)
        install_nginx(new)
        save_state(new)
        # The deployed copy of install_units needs the installed templates.
        timer = "[Unit]\nDescription=Daily Reservoir backup\n[Timer]\nOnCalendar=*-*-* "+new["backup_time"]+":00\nPersistent=true\nRandomizedDelaySec=300\n[Install]\nWantedBy=timers.target\n"
        atomic_write(Path("/etc/systemd/system/reservoir-backup.timer"), timer, 0o644)
        run(["timedatectl","set-timezone",new["timezone"]])
        run(["systemctl","daemon-reload"])
        run(["systemctl","restart","reservoir","reservoir-backup.timer"])
        verify()
    except Exception:
        for index, path in enumerate(paths):
            if existing[str(path)]:
                shutil.copy2(rollback/str(index),path)
            else:
                path.unlink(missing_ok=True)
        firewall(config, new)
        run(["timedatectl","set-timezone",config["timezone"]])
        run(["nginx","-t"])
        run(["systemctl","reload","nginx"])
        run(["systemctl","daemon-reload"])
        run(["systemctl","restart","reservoir","reservoir-backup.timer"])
        raise
    print("設定を反映しました:", "https://"+new["hostname"]+new.get("base_path", "/"))

def main():
    parser=argparse.ArgumentParser(description="Reservoir運用コマンド")
    parser.add_argument("command",choices=["status","backup","restore","configure"])
    parser.add_argument("file",type=Path,nargs="?")
    args=parser.parse_args()
    if os.geteuid()!=0:
        parser.error("sudo reservoirctl で実行してください")
    config=load_state()
    if args.command=="backup":
        backup(config)
    elif args.command=="restore":
        if not args.file: parser.error("復元元のファイルが必要です")
        restore(config,args.file)
    elif args.command=="configure":
        configure(config)
    else:
        subprocess.run(["systemctl","--no-pager","status","reservoir","reservoir-backup.timer"],check=False)
        status=Path("/var/lib/reservoir/backup-status.json")
        print("最新バックアップ:",status.read_text() if status.exists() else "未実行")
        print("公開範囲:",config["mode"],"Basic認証:",config["basic_auth"])
        print("アクセスURL:","https://"+config["hostname"]+config.get("base_path", "/"))
        print("IP直打ち拒否:","ON" if config.get("reject_ip_access",False) else "OFF")

if __name__=="__main__":
    try:
        main()
    except (Exception,KeyboardInterrupt) as exc:
        print("処理に失敗しました:",exc,file=sys.stderr)
        sys.exit(1)
