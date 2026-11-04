#!/usr/bin/env python3
"""Open the current native 1km field laboratory, preserving other windows."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import hashlib
from verify_lab import Client

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'artifacts/horizon-meadow'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    parser.add_argument('--mode', choices=['proxy','coords','indexed','objects','source'], default='proxy')
    parser.add_argument('--capture',action='store_true',help='Capture the real player camera through MCP')
    parser.add_argument('--measure-only', action='store_true', help='Measure and close only this diagnostic window')
    parser.add_argument('--object-pixels',type=float,default=128.)
    parser.add_argument('--object-shadows',action='store_true',help='Experimental object-atlas shadows; compare before accepting')
    args = parser.parse_args()
    field = OUT / 'ground.ignis-field'
    scene = OUT / 'world.scene.json'
    if not field.is_file() or not scene.is_file():
        parser.error('Compile the laboratory with the horizon_meadow example first')
    proxy = OUT / 'mesh.ignis-proxies'
    if args.mode != 'source' and not proxy.is_file():
        parser.error('Compile mesh proxies before opening the dense laboratory')
    window = OUT / 'window.json'
    if window.exists():
        prior = json.loads(window.read_text())
        command = subprocess.run(['ps','-p',str(prior['pid']),'-o','command='],capture_output=True,text=True).stdout
        if str(scene.relative_to(ROOT)) in command:
            parser.error('This laboratory window is already open; preserve it or close it before reopening')
    values = {'IGNIS_HORIZON_FIELD': str(field), 'IGNIS_HORIZON_PROXIES': str(proxy) if args.mode != 'source' else None,
        'IGNIS_HORIZON_COORDS': 'indexed' if args.mode=='objects' else args.mode if args.mode in ('coords','indexed') else None,
        'IGNIS_HORIZON_OBJECTS':str(OUT/'objects.ignis-impostors') if args.mode=='objects' else None,
        'IGNIS_HORIZON_OBJECT_PIXELS':str(args.object_pixels) if args.mode=='objects' else None,
        'IGNIS_HORIZON_OBJECT_SHADOWS':'1' if args.mode=='objects' and args.object_shadows else None}
    previous = {key:os.environ.get(key) for key in values}
    for key,value in values.items():
        if value is None: os.environ.pop(key,None)
        else: os.environ[key]=value
    try:
        client = Client(args.binary.resolve(), window=True, detached=not args.measure_only, log_path=OUT/'walk.log',
            startup_args=('--scene', str(scene.relative_to(ROOT)), '--continuous', '--first-person', 'player'))
    finally:
        for key,value in previous.items():
            if value is None: os.environ.pop(key,None)
            else: os.environ[key]=value
    (OUT/'launch-start.json').write_text(json.dumps(dict(pid=client.process.pid,mode=args.mode)))
    try:
        client.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='horizon-meadow', version='1')))
        client.request('notifications/initialized', notification=True)
        view=dict(actor='player',yaw_degrees=0.,pitch_degrees=-3.,eye_height=1.65,fov_degrees=65.,mode='first_person')
        client.call('set_player_view',view=view)
        client.call('set_viewport_performance', settings=dict(enabled=True, vsync=False))
        client.call('play')
        # Record only real visible-window samples. Do not steal focus repeatedly
        # from the user's other applications to manufacture a benchmark.
        samples=[]
        deadline=time.monotonic()+90
        valid=0
        while time.monotonic()<deadline:
            time.sleep(.55)
            sample=client.call('inspect_viewport_performance')
            samples.append(sample)
            if sample['sample']['fps'] is not None and sample['age_ms'] is not None and sample['age_ms']<2000:
                valid+=1
            if valid>=8: break
        result = dict(pid=client.process.pid, binary=str(args.binary.resolve()), scene=str(scene), field=str(field),
            mode=args.mode,object_pixels=args.object_pixels,object_shadows=args.object_shadows,presentation_samples=samples,sample_complete=valid>=8,activation="native window; no scripted foreground forcing",view=client.call('inspect_player_view'),
            scene_sha256=hashlib.sha256(scene.read_bytes()).hexdigest(),binary_sha256=hashlib.sha256(args.binary.read_bytes()).hexdigest(),
            controls='Click to capture mouse; WASD; Shift run; Esc release/close; F3 FPS')
        if args.capture:
            client.call('capture',path='artifacts/horizon-meadow/current-player.png',width=1100,height=760)
        (OUT/'window.json').write_text(json.dumps(result, indent=2))
        print(json.dumps(dict(pid=client.process.pid,mode=args.mode,performance=samples[-1],details=str(window))))
    except BaseException:
        client.close()
        raise
    else:
        if args.measure_only:
            client.close()
            return
        client.process.stdin.close()
        client.process.stdout.close()
        client.log.close()


if __name__ == '__main__':
    main()
