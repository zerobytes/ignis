"""Serve the isolated Three.js study; its UI saves evidence only below captures/."""
import argparse
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import re


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('root',type=Path);p.add_argument('--port',type=int,default=54226)
    args=p.parse_args();root=args.root.resolve()
    class Handler(SimpleHTTPRequestHandler):
        def __init__(self,*a,**kw):super().__init__(*a,directory=str(root),**kw)
        def do_POST(self):
            host=f'127.0.0.1:{self.server.server_port}'
            if self.headers.get('Host')!=host or self.headers.get('Origin')!=f'http://{host}':
                self.send_error(403);return
            name=self.path.removeprefix('/capture/')
            if not self.path.startswith('/capture/') or not re.fullmatch(r'[a-zA-Z0-9_-]+\.(?:png|json)',name):
                self.send_error(400);return
            length=int(self.headers.get('Content-Length','0'))
            if not 0<length<=16*1024*1024:self.send_error(413);return
            body=self.rfile.read(length)
            if name.endswith('.png') and not body.startswith(b'\x89PNG\r\n\x1a\n'):
                self.send_error(400);return
            out=root/'captures';out.mkdir(exist_ok=True)
            temporary=out/(name+'.tmp');temporary.write_bytes(body);temporary.replace(out/name)
            self.send_response(200);self.end_headers();self.wfile.write(b'saved')
    server=ThreadingHTTPServer(('127.0.0.1',args.port),Handler)
    print(f'http://127.0.0.1:{server.server_port}/',flush=True);server.serve_forever()


if __name__=='__main__':main()
