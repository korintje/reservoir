"""Interactive, backed-up booking repair while the API is stopped."""
from pathlib import Path
from contextlib import closing
import argparse
import json
import os
import sqlite3
import subprocess
import time


def plan_repair(rows, reversed_action):
    if reversed_action not in ('delete', 'reverse'):
        raise ValueError('Invalid reversal action')
    bookings = {row['id']: dict(row) for row in rows}
    changes = []
    def change(row, action, **values):
        changes.append({'id': row['id'], 'action': action, 'original': dict(row), 'values': values})
        if values:
            row.update(values)
        else:
            del bookings[row['id']]
    for row in sorted(list(bookings.values()), key=lambda r: r['id']):
        if row['start'] == row['end']:
            change(row, 'delete_zero_duration')
        elif row['start'] > row['end']:
            if reversed_action == 'delete':
                change(row, 'delete_reversed_interval')
            else:
                change(row, 'reverse_interval', start=row['end'], end=row['start'])
    # Containment takes priority over partial overlap; ties prefer the larger ID.
    kept = []
    for row in sorted(bookings.values(), key=lambda r: (r['resource_id'], -(r['end']-r['start']), -r['id'])):
        container = next((other for other in kept if other['resource_id'] == row['resource_id'] and other['start'] <= row['start'] and row['end'] <= other['end']), None)
        if container:
            action = 'delete_duplicate' if (container['start'],container['end']) == (row['start'],row['end']) else 'delete_contained'
            change(row, action)
        else:
            kept.append(row)
    previous = None
    for row in sorted(bookings.values(), key=lambda r: (r['resource_id'],r['start'],r['id'])):
        if previous and previous['resource_id'] == row['resource_id'] and row['start'] < previous['end']:
            change(row, 'trim_partial_overlap', start=previous['end'])
        previous = row
    return changes


def repair_database(database, backup_directory, reversed_action, confirm):
    database = Path(database)
    if not database.is_file():
        raise ValueError('DBファイルが見つかりません')
    with closing(sqlite3.connect(database)) as conn:
        conn.row_factory = sqlite3.Row
        conn.execute('BEGIN IMMEDIATE')
        try:
            rows = conn.execute('SELECT * FROM reservations ORDER BY id').fetchall()
            changes = plan_repair(rows, reversed_action)
            if not changes:
                invalid = conn.execute('SELECT COUNT(*) FROM reservations r WHERE r.start < -62135596800 OR r.end > 253402300799 OR NOT EXISTS(SELECT 1 FROM users WHERE id=r.user_id) OR NOT EXISTS(SELECT 1 FROM resources WHERE id=r.resource_id)').fetchone()[0]
                if invalid:
                    raise RuntimeError(f'自動修復対象外の不整合{invalid}件。日時範囲・利用者・設備を確認してください。')
                conn.rollback()
                return {'changes':0, 'backup':None}
            if not confirm(changes):
                conn.rollback()
                return {'changes':0, 'backup':None}
            folder = Path(backup_directory)
            folder.mkdir(parents=True, exist_ok=True, mode=0o700)
            backup = folder / f'reservoir-before-repair-{time.time_ns()}.db'
            backup.touch(mode=0o600, exist_ok=False)
            # Backup through a separate reader while the write lock prevents changes.
            with closing(sqlite3.connect(database)) as source, closing(sqlite3.connect(backup)) as snapshot:
                source.backup(snapshot)
                if snapshot.execute('PRAGMA integrity_check').fetchone()[0] != 'ok':
                    raise RuntimeError('修復前バックアップ検査に失敗しました')
            triggers = conn.execute("SELECT name,sql FROM sqlite_master WHERE type='trigger' AND tbl_name='reservations'").fetchall()
            for trigger in triggers:
                name = trigger['name'].replace('"','""')
                conn.execute(f'DROP TRIGGER "{name}"')
            conn.execute('CREATE TABLE IF NOT EXISTS reservation_repair_history (id INTEGER PRIMARY KEY, reservation_id INTEGER NOT NULL, action TEXT NOT NULL, original_json TEXT NOT NULL, repaired_at INTEGER NOT NULL)')
            for change in changes:
                conn.execute('INSERT INTO reservation_repair_history(reservation_id,action,original_json,repaired_at) VALUES(?,?,?,?)', (change['id'],change['action'],json.dumps(change['original'],ensure_ascii=False),int(time.time())))
                if change['values']:
                    row = change['original'] | change['values']
                    version = ',version=version+1' if 'version' in row else ''
                    conn.execute(f'UPDATE reservations SET start=?,end=?{version} WHERE id=?', (row['start'],row['end'],row['id']))
                else:
                    conn.execute('DELETE FROM reservations WHERE id=?', (change['id'],))
            invalid = conn.execute('SELECT COUNT(*) FROM reservations r WHERE r.start>=r.end OR r.start < -62135596800 OR r.end > 253402300799 OR NOT EXISTS(SELECT 1 FROM users WHERE id=r.user_id) OR NOT EXISTS(SELECT 1 FROM resources WHERE id=r.resource_id)').fetchone()[0]
            overlaps = conn.execute('SELECT COUNT(*) FROM reservations a JOIN reservations b ON a.id<b.id AND a.resource_id=b.resource_id AND a.start<b.end AND b.start<a.end').fetchone()[0]
            if invalid or overlaps:
                raise RuntimeError(f'未解決の不整合{invalid}件・重複{overlaps}組。修復を取り消しました。日時範囲・利用者・設備を確認してください。バックアップ: {backup}')
            for trigger in triggers:
                conn.execute(trigger['sql'])
            conn.commit()
            return {'changes':len(changes), 'backup':str(backup)}
        except BaseException:
            conn.rollback()
            raise


def main():
    parser = argparse.ArgumentParser(description='Reservoir DB対話式修復')
    parser.add_argument('--db',type=Path,default=Path('/var/lib/reservoir/reservoir.db'))
    parser.add_argument('--backup-dir',type=Path)
    args = parser.parse_args()
    if os.geteuid() != 0:
        parser.error('sudo bash repair-db.sh で実行してください')
    from common import STATE, load_state
    backup_dir = args.backup_dir or (Path(load_state()['backup_dir']) if STATE.exists() else Path('/var/backups/reservoir'))
    print('開始 > 終了の予約: (1) 削除 / (2) 開始・終了を反転')
    while True:
        selection = input('選択 [1/2、空欄で中止]: ').strip()
        if not selection:
            return
        if selection in ('1','2'):
            break
    def confirm(changes):
        labels = {'delete_zero_duration':'同時刻を削除','delete_reversed_interval':'逆転期間を削除','reverse_interval':'開始・終了を反転','delete_duplicate':'完全重複を削除（大きいIDを優先）','delete_contained':'内包された短い予約を削除','trim_partial_overlap':'部分重複の開始を調整'}
        for change in changes:
            row = change['original']
            print(f"ID={row['id']} 設備={row['resource_id']} {row['start']} → {row['end']}: {labels[change['action']]} {change['values'] or ''}")
        return input(f'{len(changes)}件の処理をバックアップ後に実行します。REPAIR と入力して確定: ').strip() == 'REPAIR'
    state = subprocess.check_output(['systemctl','show','reservoir','--property=LoadState','--value'],text=True).strip()
    if state != 'not-found':
        subprocess.run(['systemctl','stop','reservoir'],check=True)
    result = repair_database(args.db,backup_dir,'delete' if selection == '1' else 'reverse',confirm)
    print('結果:',json.dumps(result,ensure_ascii=False))
    print('サービスは停止状態です。導入失敗後は sudo bash install.sh を再実行してください。導入済みなら sudo systemctl start reservoir で起動してください。')


if __name__ == '__main__':
    try:
        main()
    except (Exception,KeyboardInterrupt) as exc:
        print('修復を中止しました:',exc)
        raise SystemExit(1)
