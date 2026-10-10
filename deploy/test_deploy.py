import copy
import unittest
from unittest.mock import patch
from pathlib import Path
import tempfile
import sqlite3
from common import nginx, app_config, validate

CONFIG = {
    "mode":"lan","hostname":"reservoir.local","tls":"selfsigned","basic_auth":True,
    "lan_networks":["192.168.1.0/24","fd00::/64"],"ssh_port":22,"backup_days":30,
    "backup_time":"03:00","timezone":"Asia/Tokyo","certificate":"/etc/reservoir/tls/server.crt",
    "certificate_key":"/etc/reservoir/tls/server.key","backup_dir":"/var/backups/reservoir",
    "lab_name":"研究室","admin_hash":"$argon2id$test",
}
class DeploymentTests(unittest.TestCase):
    def test_modes_and_basic_switch(self):
        for mode in ("lan","internet"):
            for basic in (True,False):
                c=copy.deepcopy(CONFIG)
                c.update(mode=mode,basic_auth=basic)
                if mode=="internet":
                    c.update(hostname="reservoir.example.org",tls="existing")
                text=nginx(c)
                self.assertIn("listen [::]:443 ssl",text)
                self.assertIn("proxy_pass http://127.0.0.1:8080;",text)
                self.assertEqual("deny all;" in text,mode=="lan")
                self.assertEqual('auth_basic "Reservoir";' in text,basic)
                self.assertIn("auth_basic off;",text if not basic else nginx(c,True))
                self.assertIn("secure_cookies = true",app_config(c))
    def test_injection_and_invalid_network_rejected(self):
        for key,value in (("hostname","example.org; return 200;"),("lan_networks",["all; allow all"]),("certificate","/x;include /etc/passwd"),("backup_time","99:99"),("backup_dir","/")):
            c=copy.deepcopy(CONFIG);c[key]=value
            with self.assertRaises(ValueError): validate(c)
    def test_public_self_signed_rejected(self):
        c=copy.deepcopy(CONFIG);c.update(mode="internet",hostname="example.org")
        with self.assertRaises(ValueError):validate(c)
        c=copy.deepcopy(CONFIG);c.update(tls="acme")
        with self.assertRaises(ValueError):validate(c)
    def test_bootstrap_exposes_only_acme_and_redirect(self):
        text=nginx(CONFIG,True)
        self.assertIn("/.well-known/acme-challenge/",text)
        self.assertNotIn("proxy_pass",text)
        self.assertNotIn("listen 443",text)
    def test_failed_update_restores_existing_configuration(self):
        import install
        with tempfile.TemporaryDirectory() as directory:
            existing = Path(directory)/"existing"
            added = Path(directory)/"added"
            existing.write_text("old password and settings")
            existing.chmod(0o640)
            with patch.object(install,"deployment_paths",return_value=[existing,added]), patch.object(install,"firewall") as firewall, patch.object(install,"run") as run:
                with self.assertRaises(RuntimeError):
                    with install.deployment_rollback(CONFIG):
                        existing.write_text("new settings")
                        existing.chmod(0o600)
                        added.write_text("new")
                        raise RuntimeError("nginx test failed")
                self.assertEqual(existing.read_text(),"old password and settings")
                self.assertEqual(existing.stat().st_mode & 0o777,0o640)
                self.assertFalse(added.exists())
                firewall.assert_called_once_with(CONFIG)
                self.assertIn((["systemctl","restart","reservoir","reservoir-backup.timer"],),[c.args for c in run.call_args_list])
    def test_firewall_only_removes_own_web_rules(self):
        import install
        output=b"[ 1] 22/tcp ALLOW IN Anywhere # reservoir-ssh\n[ 2] 443/tcp ALLOW IN Anywhere # another-application\n[ 3] 80/tcp ALLOW IN Anywhere # reservoir-web\n[ 4] 443/tcp ALLOW IN Anywhere # reservoir-lan\n"
        with patch.object(install.subprocess,"check_output",return_value=output), patch.object(install,"run") as run:
            install.firewall(CONFIG)
            deletions=[c.args[0] for c in run.call_args_list if "delete" in c.args[0]]
            self.assertEqual(deletions,[["ufw","--force","delete","4"],["ufw","--force","delete","3"]])
    def test_failed_pre_restore_backup_restarts_service(self):
        import reservoirctl
        with tempfile.TemporaryDirectory() as directory:
            source=Path(directory)/"backup.db"
            with sqlite3.connect(source) as conn:
                for table in ("users","resources","reservations"):
                    conn.execute("CREATE TABLE "+table+" (id INTEGER)")
            with patch("builtins.input",return_value="RESTORE"), patch.object(reservoirctl,"backup",side_effect=RuntimeError("disk full")), patch.object(reservoirctl,"run") as run:
                with self.assertRaises(RuntimeError): reservoirctl.restore(CONFIG,source)
                self.assertEqual([c.args[0] for c in run.call_args_list],[["systemctl","stop","reservoir"],["systemctl","start","reservoir"]])
if __name__=="__main__":unittest.main()
