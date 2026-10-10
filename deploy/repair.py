"""Explicit, backed-up repair of legacy booking intervals while the API is stopped."""
from pathlib import Path
import json
import sqlite3
import time

def auto_assume(database, backup_directory):
    database = Path(database)
    if not database.is_file():
        return {"reversed": 0, "deleted": 0, "backup": None}
    with sqlite3.connect(database) as conn:
        conn.row_factory = sqlite3.Row
        if not conn.execute("SELECT 1 FROM sqlite_master WHERE type='table' AND name='reservations'").fetchone():
            return {"reversed": 0, "deleted": 0, "backup": None}
        rows = conn.execute("SELECT * FROM reservations WHERE start >= end ORDER BY id").fetchall()
        if not rows:
            return {"reversed": 0, "deleted": 0, "backup": None}
        folder = Path(backup_directory)
        folder.mkdir(parents=True, exist_ok=True, mode=0o700)
        backup = folder / f"reservoir-before-auto-assume-{time.time_ns()}.db"
        # Create with restrictive permissions before writing sensitive data.
        backup.touch(mode=0o600, exist_ok=False)
        with sqlite3.connect(backup) as snapshot:
            conn.backup(snapshot)
            if snapshot.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
                raise RuntimeError("Auto-assume前のバックアップ検査に失敗しました")
        conn.execute("BEGIN IMMEDIATE")
        try:
            rows = conn.execute("SELECT * FROM reservations WHERE start >= end ORDER BY id").fetchall()
            triggers = conn.execute("SELECT name, sql FROM sqlite_master WHERE type='trigger' AND tbl_name='reservations'").fetchall()
            # Maintenance only: historical bookings are otherwise immutable.
            for trigger in triggers:
                name = trigger['name'].replace('"', '""')
                conn.execute(f'DROP TRIGGER "{name}"')
            conn.execute("CREATE TABLE IF NOT EXISTS reservation_repair_history (id INTEGER PRIMARY KEY, reservation_id INTEGER NOT NULL, action TEXT NOT NULL, original_json TEXT NOT NULL, repaired_at INTEGER NOT NULL)")
            reversed_count = deleted_count = 0
            for row in rows:
                equal = row['start'] == row['end']
                conn.execute("INSERT INTO reservation_repair_history(reservation_id,action,original_json,repaired_at) VALUES(?,?,?,?)", (row['id'], 'delete_zero_duration' if equal else 'reverse_interval', json.dumps(dict(row), ensure_ascii=False), int(time.time())))
                if equal:
                    conn.execute("DELETE FROM reservations WHERE id=?", (row['id'],))
                    deleted_count += 1
                else:
                    conn.execute("UPDATE reservations SET start=?,end=? WHERE id=?", (row['end'],row['start'],row['id']))
                    reversed_count += 1
            for trigger in triggers:
                conn.execute(trigger['sql'])
            conn.commit()
        except BaseException:
            conn.rollback()
            raise
    return {"reversed": reversed_count, "deleted": deleted_count, "backup": str(backup)}
