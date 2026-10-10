"""Validate generated configs with an existing, non-root nginx binary."""
import copy
from pathlib import Path
import subprocess
import sys
import tempfile
from common import nginx
from test_deploy import CONFIG

binary = Path(sys.argv[1]).resolve()
with tempfile.TemporaryDirectory(prefix="reservoir-nginx-") as temporary:
    root = Path(temporary)
    cert, key = root/"cert.pem", root/"key.pem"
    subprocess.run(["openssl","req","-x509","-newkey","rsa:2048","-nodes","-days","1","-subj","/CN=localhost","-keyout",str(key),"-out",str(cert)],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    for mode in ("lan","internet"):
        for basic in (True,False):
            config=copy.deepcopy(CONFIG)
            config.update(mode=mode,basic_auth=basic,certificate=str(cert),certificate_key=str(key),tls="existing")
            if mode=="internet":config["hostname"]="reservoir.example.org"
            file=root/"nginx.conf"
            file.write_text(
                f"pid {root}/nginx.pid;\nerror_log stderr;\nevents {{}}\nhttp {{\naccess_log off;\n"
                "limit_req_zone $binary_remote_addr zone=reservoir_api:10m rate=10r/s;\n"
                "limit_req_zone $binary_remote_addr zone=reservoir_login:10m rate=5r/m;\n"
                +f"client_body_temp_path {root}/body; proxy_temp_path {root}/proxy; fastcgi_temp_path {root}/fastcgi; uwsgi_temp_path {root}/uwsgi; scgi_temp_path {root}/scgi;\n"
                +nginx(config).replace("listen 80;", "listen 18080;").replace("listen [::]:80;", "listen [::]:18080;").replace("listen 443 ssl;", "listen 18443 ssl;").replace("listen [::]:443 ssl;", "listen [::]:18443 ssl;")+"\n}\n"
            )
            subprocess.run([str(binary),"-t","-p",str(root)+"/","-c",str(file)],check=True)
            print("Validated:",mode,"Basic",basic)
