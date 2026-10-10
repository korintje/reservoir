"""Run the API and Vite with a persistent, isolated local database."""
from pathlib import Path
import argparse
import getpass
import json
import os
import shutil
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--reset-password", action="store_true")
    args = parser.parse_args()
    binary = ROOT / "backend/target/debug" / ("reservoir.exe" if os.name == "nt" else "reservoir")
    if not binary.is_file() or not (ROOT / "frontend/node_modules").is_dir():
        raise SystemExit("backendでcargo build --locked、frontendでnpm ciを先に実行してください。")
    folder = ROOT / ".local"
    folder.mkdir(exist_ok=True)
    config = folder / "reservoir.toml"
    if not config.exists() or args.reset_password:
        while True:
            password = getpass.getpass("開発用管理者パスワード（8文字以上）: ")
            if 8 <= len(password) <= 256 and password == getpass.getpass("もう一度入力: "):
                break
            print("長さまたは入力の一致を確認してください。")
        hashed = subprocess.check_output([str(binary), "hash-password"], input=(password+"\n").encode()).decode().strip()
        config.write_text(
            'web_url = "127.0.0.1:8080"\n'
            + 'db_path = ' + json.dumps(str(folder / "reservoir.db")) + '\n'
            + 'admin_password_hash = ' + json.dumps(hashed) + '\n'
            + 'secure_cookies = false\nbasic_auth_enabled = false\npublic_mode = "lan"\n',
            encoding="utf-8",
        )
        config.chmod(0o600)
    api = subprocess.Popen([str(binary)], env=dict(os.environ, RESERVOIR_CONFIG=str(config)), cwd=folder, start_new_session=os.name != "nt")
    npm = shutil.which("npm.cmd" if os.name == "nt" else "npm")
    command = (["cmd", "/c", npm] if os.name == "nt" else [npm]) + ["run", "dev", "--", "--port", "5173", "--strictPort"]
    frontend = None
    try:
        frontend = subprocess.Popen(command, cwd=ROOT / "frontend", start_new_session=os.name != "nt")
        print("開発画面: http://127.0.0.1:5173  終了: Ctrl+C", flush=True)
        while api.poll() is None and frontend.poll() is None:
            time.sleep(0.5)
        raise SystemExit("開発サーバーが停止しました。上のログを確認してください。")
    except KeyboardInterrupt:
        pass
    finally:
        for process in (frontend, api):
            if process and process.poll() is None:
                if os.name == "nt":
                    subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                else:
                    os.killpg(process.pid, signal.SIGTERM)
                process.wait()

if __name__ == "__main__":
    main()
