#!/usr/bin/env python3
"""Open the verified Earth runtime with a detached local MCP controller.

The controller keeps the native MCP transport alive independently of the terminal
and exits when the user closes the native window.
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
from open_world_walk import native_entry
from open_earth_walk import init
from prepare_earth_surface import OUT, SCENE, CONFIG
from verify_lab import ROOT, Client


def host():
    report = json.loads((OUT / 'launch.json').read_text())
    report['controller_pid'] = os.getpid()
    (OUT / 'launch.json').write_text(json.dumps(report, indent=2))
    entry = native_entry()
    c = Client(entry, window=True, log_path=OUT / 'native.log', startup_args=(
        '--scene', SCENE, '--continuous', '--planet-runtime', CONFIG,
        '--third-person', 'player', '--planet-view-distance', '4'))
    try:
        init(c)
        c.call('play')
        c.call('set_viewport_performance', settings={'enabled': True, 'vsync': True})
        report = json.loads((OUT / 'launch.json').read_text())
        report.update(pid=c.process.pid, playing=True, runtime=c.call('inspect_planet_runtime'),
                      view=c.call('inspect_planet_view'), native_build=c.call('inspect')['build_id'])
        (OUT / 'launch.json').write_text(json.dumps(report, indent=2))
        print(json.dumps({'native_ready': True, 'pid': c.process.pid}), flush=True)
        # Window closure owns shutdown; neither stdin EOF nor terminal ownership
        # can terminate the MCP transport while the user is playing.
        c.process.wait()
    finally:
        c.close()


def launch():
    report_path = OUT / 'verification.json'
    if not report_path.is_file() or not (ROOT / SCENE).is_file() or not (ROOT / CONFIG).is_file():
        raise RuntimeError('Run python3 scripts/prepare_earth_surface.py first')
    report = json.loads(report_path.read_text())
    if not report['automated_passed'] or report['diameter_meters'] != 12_742_000:
        raise RuntimeError('The current Earth runtime has not passed MCP validation')
    entry = native_entry()
    stamp = entry.stat()
    fingerprint = [stamp.st_ino, stamp.st_size, stamp.st_mtime_ns]
    scene_hash = hashlib.sha256((ROOT / SCENE).read_bytes()).hexdigest()
    config_hash = hashlib.sha256((ROOT / CONFIG).read_bytes()).hexdigest()
    if report.get('scene_sha256') != scene_hash or report.get('config_sha256') != config_hash:
        raise RuntimeError('Earth authoring files changed since MCP validation; prepare them again')
    launch_path = OUT / 'launch.json'
    if launch_path.exists():
        old = json.loads(launch_path.read_text())
        process = subprocess.run(['ps', '-p', str(old.get('pid') or old.get('controller_pid')), '-o', 'command='], capture_output=True, text=True)
        if old.get('pid') is None and process.returncode == 0 and 'open_earth_surface.py' in process.stdout:
            print(json.dumps({'controller_pid': old['controller_pid'], 'starting': True})); return
        if process.returncode == 0 and SCENE in process.stdout and str(ROOT) in process.stdout:
            if old['binary_stamp'] == fingerprint and old['scene_sha256'] == scene_hash and old['config_sha256'] == config_hash:
                print(json.dumps({'pid': old['pid'], 'already_running': True})); return
            raise RuntimeError('A previous Earth preview is still open with another source/build; close that window first')
    launch_path.write_text(json.dumps(dict(pid=None, scene=SCENE, config=CONFIG,
        binary_stamp=fingerprint, scene_sha256=scene_hash, config_sha256=config_hash,
        mcp_verification_build=report['build_id'], playing=False,
        controls='P play/pause; WASD; Shift600m/s; click+mouse; Tab first/third; wheel zoom; G globe/surface; Esc release'), indent=2))
    with (OUT / 'controller.log').open('w') as log:
        process = subprocess.Popen([sys.executable, str(ROOT / 'scripts/open_earth_surface.py'), '--host'],
            stdin=subprocess.DEVNULL, stdout=log, stderr=log, start_new_session=True)
    print(json.dumps({'controller_pid': process.pid, 'scene': SCENE, 'starting': True}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    host() if args.host else launch()
