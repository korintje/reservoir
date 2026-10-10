#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
if ! command -v python3 >/dev/null; then
  echo "Ubuntu Serverのpython3が必要です。" >&2
  exit 1
fi
exec python3 "$SCRIPT_DIR/deploy/install.py" "$@"
