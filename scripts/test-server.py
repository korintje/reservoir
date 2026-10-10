"""Disposable backend for browser tests. Never opens the bundled or production DB."""
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parent.parent
binary = ROOT / "backend/target/debug" / ("reservoir.exe" if __import__("os").name == "nt" else "reservoir")
if not binary.exists():
    raise SystemExit("Run cargo build in backend/ first.")
with tempfile.TemporaryDirectory(prefix="reservoir-e2e-") as directory:
    folder = Path(directory)
    hashed = subprocess.check_output([str(binary), "hash-password"], input=b"e2e-admin-password\n").decode().strip()
    config = folder / "reservoir.toml"
    config.write_text(
        'web_url = "127.0.0.1:8080"\n'
        + "db_path = " + json.dumps(str(folder / "reservoir.db")) + "\n"
        + "admin_password_hash = " + json.dumps(hashed) + "\n"
        + 'secure_cookies = false\nbasic_auth_enabled = false\npublic_mode = "lan"\n',
        encoding="utf-8",
    )
    import os
    environment = dict(os.environ, RESERVOIR_CONFIG=str(config))
    process = subprocess.Popen([str(binary)], env=environment, cwd=folder)
    try:
        process.wait()
    except KeyboardInterrupt:
        process.terminate()
        process.wait()
