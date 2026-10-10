import json
from pathlib import Path
from contextlib import closing
import sqlite3
import tempfile
import unittest
from unittest.mock import patch
import repair
from repair import plan_repair, repair_database

class RepairTests(unittest.TestCase):
    def rows(self, intervals):
        return [dict(id=i+1,resource_id=r,start=s,end=e,user_id=1,passhash='hash') for i,(r,s,e) in enumerate(intervals)]
    def remaining(self, intervals, action='reverse'):
        rows = self.rows(intervals)
        result = {r['id']:r.copy() for r in rows}
        for change in plan_repair(rows, action):
            if change['values']: result[change['id']].update(change['values'])
            else: del result[change['id']]
        return [(r['id'],r['start'],r['end']) for r in result.values()]
    def test_reverse_and_zero(self):
        self.assertEqual(self.remaining([(1,200,100),(1,250,250),(1,300,400)]),[(1,100,200),(3,300,400)])
        self.assertEqual(self.remaining([(1,200,100)],'delete'),[])
    def test_duplicate_prefers_larger_id(self):
        self.assertEqual(self.remaining([(1,100,200)]*3),[(3,100,200)])
    def test_partial_chain_and_adjacency(self):
        self.assertEqual(self.remaining([(1,100,200),(1,150,300),(1,250,400),(1,400,500)]),[(1,100,200),(2,200,300),(3,300,400),(4,400,500)])
    def test_containment_including_shared_boundary(self):
        self.assertEqual(self.remaining([(1,120,180),(1,100,200),(1,100,190),(1,110,200)]),[(2,100,200)])
    def test_containment_before_partial_and_separate_resources(self):
        self.assertEqual(self.remaining([(1,100,200),(1,150,300),(1,180,250),(2,100,200)]),[(1,100,200),(2,200,300),(4,100,200)])
    def database(self,path):
        with closing(sqlite3.connect(path)) as conn, conn:
            conn.executescript("""
            CREATE TABLE users(id INTEGER PRIMARY KEY); INSERT INTO users VALUES(1);
            CREATE TABLE resources(id INTEGER PRIMARY KEY); INSERT INTO resources VALUES(1);
            CREATE TABLE reservations(id INTEGER PRIMARY KEY,resource_id INTEGER,user_id INTEGER,start INTEGER,end INTEGER,passhash TEXT);
            INSERT INTO reservations VALUES(1,1,1,200,100,'keep'),(2,1,1,250,250,'delete'),(3,1,1,150,300,'partial');
            CREATE TRIGGER protect_history BEFORE UPDATE ON reservations BEGIN SELECT RAISE(ABORT,'finished'); END;
            CREATE TRIGGER protect_delete BEFORE DELETE ON reservations BEGIN SELECT RAISE(ABORT,'finished'); END;
            """)
    def test_backup_audit_triggers_and_idempotency(self):
        with tempfile.TemporaryDirectory() as folder:
            db=Path(folder)/'source.db'; self.database(db)
            result=repair_database(db,Path(folder)/'backups','reverse',lambda changes:True)
            self.assertEqual(result['changes'],3)
            with closing(sqlite3.connect(result['backup'])) as conn:
                self.assertEqual(conn.execute('SELECT start,end FROM reservations WHERE id=1').fetchone(),(200,100))
                self.assertEqual(conn.execute('SELECT COUNT(*) FROM reservations').fetchone()[0],3)
            with closing(sqlite3.connect(db)) as conn, conn:
                self.assertEqual(conn.execute('SELECT id,start,end FROM reservations ORDER BY id').fetchall(),[(1,100,200),(3,200,300)])
                audit=conn.execute('SELECT original_json FROM reservation_repair_history ORDER BY id').fetchall()
                self.assertEqual(json.loads(audit[1][0])['passhash'],'delete')
                with self.assertRaises(sqlite3.IntegrityError): conn.execute('DELETE FROM reservations WHERE id=1')
            self.assertEqual(repair_database(db,folder,'reverse',lambda changes:True),{'changes':0,'backup':None})
    def test_cancel_does_not_write(self):
        with tempfile.TemporaryDirectory() as folder:
            db=Path(folder)/'source.db'; self.database(db)
            self.assertEqual(repair_database(db,folder,'reverse',lambda changes:False),{'changes':0,'backup':None})
            self.assertEqual(list(Path(folder).glob('reservoir-before*')),[])
    def test_migrated_booking_version_increases(self):
        with tempfile.TemporaryDirectory() as folder:
            db=Path(folder)/'source.db'; self.database(db)
            with closing(sqlite3.connect(db)) as conn, conn:
                conn.execute('ALTER TABLE reservations ADD COLUMN version INTEGER NOT NULL DEFAULT 1')
            repair_database(db,folder,'reverse',lambda changes:True)
            with closing(sqlite3.connect(db)) as conn:
                self.assertEqual(conn.execute('SELECT version FROM reservations ORDER BY id').fetchall(),[(2,),(2,)])
    def test_unresolved_reference_rolls_back_rows_and_triggers(self):
        with tempfile.TemporaryDirectory() as folder:
            db=Path(folder)/'source.db'; self.database(db)
            with closing(sqlite3.connect(db)) as conn, conn: conn.execute('DELETE FROM users')
            with self.assertRaises(RuntimeError): repair_database(db,folder,'reverse',lambda changes:True)
            with closing(sqlite3.connect(db)) as conn, conn:
                self.assertEqual(conn.execute('SELECT start,end FROM reservations WHERE id=1').fetchone(),(200,100))
                self.assertEqual(conn.execute('SELECT COUNT(*) FROM reservations').fetchone()[0],3)
                with self.assertRaises(sqlite3.IntegrityError): conn.execute('DELETE FROM reservations WHERE id=2')
    def test_sql_failure_rolls_back(self):
        with tempfile.TemporaryDirectory() as folder:
            db=Path(folder)/'source.db'; self.database(db)
            with closing(sqlite3.connect(db)) as conn, conn:
                conn.execute("CREATE TABLE reservation_repair_history(id INTEGER PRIMARY KEY,reservation_id INTEGER,action TEXT CHECK(action='never'),original_json TEXT,repaired_at INTEGER)")
            with self.assertRaises(sqlite3.IntegrityError): repair_database(db,folder,'reverse',lambda changes:True)
            with closing(sqlite3.connect(db)) as conn, conn:
                self.assertEqual(conn.execute('SELECT start,end FROM reservations WHERE id=1').fetchone(),(200,100))
                with self.assertRaises(sqlite3.IntegrityError): conn.execute('DELETE FROM reservations WHERE id=2')
    def test_interactive_script_stops_service_and_confirms_repair(self):
        with tempfile.TemporaryDirectory() as folder:
            db=Path(folder)/'source.db'; self.database(db)
            with patch('sys.argv',['repair.py','--db',str(db),'--backup-dir',folder]), patch('repair.os.geteuid',return_value=0,create=True), patch('builtins.input',side_effect=['2','REPAIR']), patch('builtins.print'), patch('repair.subprocess.check_output',return_value='loaded\n'), patch('repair.subprocess.run') as run:
                repair.main()
                run.assert_called_once_with(['systemctl','stop','reservoir'],check=True)
            with closing(sqlite3.connect(db)) as conn:
                self.assertEqual(conn.execute('SELECT start,end FROM reservations WHERE id=1').fetchone(),(100,200))
    def test_interactive_initial_cancel_leaves_service_untouched(self):
        with patch('sys.argv',['repair.py','--backup-dir','/tmp/backups']), patch('repair.os.geteuid',return_value=0,create=True), patch('builtins.input',return_value=''), patch('builtins.print'), patch('repair.subprocess.check_output') as show:
            repair.main()
            show.assert_not_called()

if __name__=='__main__': unittest.main()
