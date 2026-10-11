"""Check generated nginx syntax and real HTTPS routing without root privileges."""
import base64
import copy
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from http.client import RemoteDisconnected
from pathlib import Path
import ssl
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request
from common import nginx
from test_deploy import CONFIG

class Backend(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Set-Cookie','reservoir_admin=test; Path=/api; HttpOnly; Secure')
        self.end_headers()
        self.wfile.write(self.path.encode())
    do_POST = do_GET
    def log_message(self, *args):
        pass

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args):
        return None

binary = Path(sys.argv[1]).resolve()
backend = ThreadingHTTPServer(('127.0.0.1',0),Backend)
threading.Thread(target=backend.serve_forever,daemon=True).start()
try:
    with tempfile.TemporaryDirectory(prefix='reservoir-nginx-') as temporary:
        root = Path(temporary)
        root.chmod(0o755)  # nginx workers may use a different UID in root-run CI.
        cert,key = root/'cert.pem',root/'key.pem'
        subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-subj','/CN=localhost','-keyout',str(key),'-out',str(cert)],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        htpasswd = root/'basic.htpasswd'
        hashed = subprocess.check_output(['openssl','passwd','-apr1','test-password'],text=True).strip()
        htpasswd.write_text('lab:'+hashed+'\n')
        opener = urllib.request.build_opener(NoRedirect(),urllib.request.HTTPSHandler(context=ssl._create_unverified_context()))
        def request(path, basic=False, authenticated=True, data=None, host=None, scheme='https'):
            headers = {'Authorization':'Basic '+base64.b64encode(b'lab:test-password').decode()} if basic and authenticated else {}
            headers['Host'] = host or config['hostname']
            port = 18443 if scheme == 'https' else 18080
            req = urllib.request.Request(f'{scheme}://127.0.0.1:{port}'+path,headers=headers,data=data)
            try:
                return opener.open(req,timeout=3)
            except urllib.error.HTTPError as exc:
                return exc
        for base in ('/','/reservation/','/lab/reservation/'):
            web = root/'frontend'
            directory = web/base.strip('/')
            (directory/'assets').mkdir(parents=True,exist_ok=True)
            (directory/'index.html').write_text('Reservoir index')
            (directory/'assets/test.js').write_text('Reservoir asset')
            for mode in ('lan','internet'):
                for basic in (True,False):
                    config=copy.deepcopy(CONFIG)
                    config.update(base_path=base,mode=mode,basic_auth=basic,certificate=str(cert),certificate_key=str(key),tls='existing',reject_ip_access=True)
                    if mode=='internet': config['hostname']='reservoir.example.org'
                    generated=nginx(config).replace('listen 80;', 'listen 18080;').replace('listen [::]:80;', 'listen [::]:18080;').replace('listen 443 ssl;','listen 18443 ssl;').replace('listen [::]:443 ssl;','listen [::]:18443 ssl;')
                    generated=generated.replace('/opt/reservoir/current/frontend',str(web)).replace('/etc/reservoir/basic.htpasswd',str(htpasswd)).replace('127.0.0.1:8080',f'127.0.0.1:{backend.server_port}')
                    file=root/'nginx.conf'
                    file.write_text(f'pid {root}/nginx.pid;\nerror_log stderr;\nevents {{}}\nhttp {{\naccess_log off;\n'
                        'limit_req_zone $binary_remote_addr zone=reservoir_api:10m rate=10r/s;\n'
                        'limit_req_zone $binary_remote_addr zone=reservoir_login:10m rate=5r/m;\n'
                        +f'client_body_temp_path {root}/body; proxy_temp_path {root}/proxy; fastcgi_temp_path {root}/fastcgi; uwsgi_temp_path {root}/uwsgi; scgi_temp_path {root}/scgi;\n'+generated+'\n}\n')
                    args=[str(binary),'-p',str(root)+'/', '-c',str(file)]
                    subprocess.run(args+['-t'],check=True)
                    process=subprocess.Popen(args+['-g','daemon off;'])
                    try:
                        for attempt in range(50):
                            try:
                                with request(base,basic) as response:
                                    assert response.status==200 and response.read()==b'Reservoir index'
                                break
                            except urllib.error.URLError:
                                if process.poll() is not None: raise RuntimeError('nginx exited')
                                time.sleep(.1)
                        else: raise RuntimeError('nginx did not start')
                        with request(base+'settings',basic) as response:
                            assert response.status==200 and response.read()==b'Reservoir index'
                        with request(base+'assets/test.js',basic) as response:
                            assert response.status==200 and response.read()==b'Reservoir asset'
                        with request(base+'assets/missing.js',basic) as response:
                            assert response.status==404
                        with request(base+'api/health?check=1',basic) as response:
                            assert response.status==200 and response.read()==b'/api/health?check=1'
                            assert 'Path='+base+'api' in response.headers['Set-Cookie']
                        with request(base+'api/admin/login',basic,data=b'{}') as response:
                            assert response.status==200 and response.read()==b'/api/admin/login'
                            assert 'Path='+base+'api' in response.headers['Set-Cookie']
                        if basic:
                            with request(base+'api/health',basic,authenticated=False) as response:
                                assert response.status==401
                        if base!='/':
                            with request(base.rstrip('/')+'?user=1',basic) as response:
                                assert response.status==308 and response.headers['Location'].endswith(base+'?user=1')
                            with request('/api/health',basic) as response:
                                assert response.status==404
                        for scheme in ('http','https'):
                            for ip_host in ('127.0.0.1','[::1]'):
                                try:
                                    with request(base,basic,host=ip_host,scheme=scheme):
                                        raise AssertionError('IP access must close without an HTTP response')
                                except RemoteDisconnected:
                                    pass
                        print('Validated HTTPS:',base,mode,'Basic',basic)
                    finally:
                        process.terminate()
                        process.wait(timeout=5)
finally:
    backend.shutdown()
    backend.server_close()
