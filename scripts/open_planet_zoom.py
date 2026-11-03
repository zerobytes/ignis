#!/usr/bin/env python3
"""Open the current procedural planet with MCP/native surface-to-globe controls."""
import argparse
import hashlib
import json
import subprocess
from open_world_walk import native_entry
from open_earth_walk import init
from verify_lab import Client, ROOT
from verify_planet_zoom import SCENE, CONFIG, OUT

def launch(distance):
    binary=native_entry()
    if not (ROOT/SCENE).is_file() or not (ROOT/CONFIG).is_file():
        from open_planet_walk import prepare
        prepare(binary)
    stamp=binary.stat()
    binary_stamp=[stamp.st_ino,stamp.st_size,stamp.st_mtime_ns]
    launch_path=ROOT/'artifacts/earth-walk/launch.json'
    if launch_path.exists():
        old=json.loads(launch_path.read_text())
        probe=subprocess.run(['ps','-p',str(old['pid']),'-o','command='],capture_output=True,text=True,check=False)
        if probe.returncode==0 and str(binary) in probe.stdout:
            if old.get('planet_zoom') and old.get('binary_stamp')==binary_stamp:
                print(json.dumps({'pid':old['pid'],'already_running':True}));return
            raise RuntimeError('Previous human demo is still running; close its window before replacing it')
    OUT.mkdir(parents=True,exist_ok=True)
    c=Client(binary,window=True,detached=True,log_path=OUT/'native.log',startup_args=(
        '--scene',SCENE,'--continuous','--planet-runtime',CONFIG,'--planet-view-distance',str(distance)))
    try:
        init(c)
        c.call('play')
        c.call('set_viewport_performance',settings={'enabled':True,'vsync':True})
        report={'pid':c.process.pid,'planet_zoom':True,'scene':SCENE,'planet_runtime':CONFIG,'binary_stamp':binary_stamp,
                'scene_sha256':hashlib.sha256((ROOT/SCENE).read_bytes()).hexdigest(),
                'config_sha256':hashlib.sha256((ROOT/CONFIG).read_bytes()).hexdigest(),
                'world':c.call('inspect_planet_runtime'),'view':c.call('inspect_planet_view'),
                'controls':'wheel zoom, G globe/surface, F surface, Tab first/third, click+mouse look, WASD, Shift run, Esc release, P pause'}
        launch_path.write_text(json.dumps(report,indent=2))
        print(json.dumps({'opened':True,'pid':c.process.pid,'planet_zoom':True}))
    except BaseException:
        c.close();raise
    else:
        c.process.stdin.close();c.process.stdout.close();c.log.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--distance',type=float,default=0.)
    launch(p.parse_args().distance)
