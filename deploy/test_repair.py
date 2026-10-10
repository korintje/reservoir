import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
from repair import auto_assume

class RepairTests(unittest.TestCase):
    def database(self, path):
        with sqlite3.connect(path) as conn:
            conn.executescript("""
                CREATE TABLE reservations(id INTEGER PRIMARY KEY,start INTEGER,end INTEGER,passhash TEXT);
                INSERT INTO reservations VALUES(1,200,100,'keep-hash'),(2,250,250,'deleted-hash'),(3,300,400,'valid-hash');
                CREATE TRIGGER protect_history BEFORE UPDATE ON reservations BEGIN SELECT RAISE(ABORT,'finished_reservation'); END;
                CREATE TRIGGER protect_delete BEFORE DELETE ON reservations BEGIN SELECT RAISE(ABORT,'finished_reservation'); END;
            """)
    def test_repair_backups_audits_preserves_triggers_and_is_idempotent(self):
        with tempfile.TemporaryDirectory() as folder:
            db = Path(folder)/'source.db'
            self.database(db)
            result = auto_assume(db,Path(folder)/'backups')
            self.assertEqual((result['reversed'],result['deleted']),(1,1))
            with sqlite3.connect(result['backup']) as original:
                self.assertEqual(original.execute('SELECT start,end FROM reservations WHERE id=1').fetchone(),(200,100))
                self.assertEqual(original.execute('SELECT COUNT(*) FROM reservations').fetchone()[0],3)
            with sqlite3.connect(db) as conn:
                self.assertEqual(conn.execute('SELECT * FROM reservations ORDER BY id').fetchall(),[(1,100,200,'keep-hash'),(3,300,400,'valid-hash')])
                audit=conn.execute('SELECT action,original_json FROM reservation_repair_history ORDER BY id').fetchall()
                self.assertEqual([a[0] for a in audit],['reverse_interval','delete_zero_duration'])
                self.assertEqual(json.loads(audit[1][1])['passhash'],'deleted-hash')
                with self.assertRaises(sqlite3.IntegrityError):conn.execute('UPDATE reservations SET start=0 WHERE id=1')
            self.assertEqual(auto_assume(db,Path(folder)/'backups'),{'reversed':0,'deleted':0,'backup':None})
    def test_failed_repair_restores_original_rows_and_triggers(self):
        with tempfile.TemporaryDirectory() as folder:
            db=Path(folder)/'source.db'
            self.database(db)
            with sqlite3.connect(db) as conn:
                conn.execute("CREATE TABLE reservation_repair_history(id INTEGER PRIMARY KEY,reservation_id INTEGER,action TEXT CHECK(action='never'),original_json TEXT,repaired_at INTEGER)")
            with self.assertRaises(sqlite3.IntegrityError):auto_assume(db,Path(folder)/'backups')
            with sqlite3.connect(db) as conn:
                self.assertEqual(conn.execute('SELECT start,end FROM reservations WHERE id=1').fetchone(),(200,100))
                self.assertEqual(conn.execute('SELECT COUNT(*) FROM reservations').fetchone()[0],3)
                with self.assertRaises(sqlite3.IntegrityError):conn.execute('DELETE FROM reservations WHERE id=2')

if __name__=='__main__':unittest.main()
