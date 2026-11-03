#!/usr/bin/env python3
"""Open the current native world independently of this terminal/MCP client."""
import argparse
import json
import plistlib
import subprocess
import sys
import time
from pathlib import Path
from verify_lab import Client, ROOT


def native_entry():
    binary = ROOT/'target/release'/('ignis.exe' if sys.platform=='win32' else 'ignis')
    if not binary.is_file():
        raise RuntimeError('Build target/release/ignis before opening the demo')
    if sys.platform != 'darwin':
        return binary
    # Separate GUI identity for gameplay and editor; both point to the ONE
    # current executable. No copied binary, historical build or alternate cache.
    contents = Path.home()/'Applications/Ignis Play.app/Contents'
    info = contents/'Info.plist'
    bundle_id = 'dev.ignisengine.play'
    if info.exists() and plistlib.loads(info.read_bytes()).get('CFBundleIdentifier') != bundle_id:
        raise RuntimeError('An unrelated Ignis Play.app already exists; use --binary explicitly')
    entry = contents/'MacOS/ignis'
    entry.parent.mkdir(parents=True, exist_ok=True)
    if entry.is_symlink():
        if entry.readlink() != binary:
            entry.unlink()
    elif entry.exists():
        raise RuntimeError('Refusing to replace a non-symlink executable in Ignis Play.app')
    if not entry.is_symlink():
        entry.symlink_to(binary)
    info.write_bytes(plistlib.dumps(dict(CFBundleDisplayName='Ignis Play',CFBundleExecutable='ignis',
        CFBundleIdentifier=bundle_id,CFBundleName='Ignis Play',CFBundlePackageType='APPL',
        CFBundleShortVersionString='0.1.0',CFBundleVersion='1',NSHighResolutionCapable=True)))
    return entry


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scene', default='artifacts/expanded-world/world.scene.json')
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--lod', help='Optional verified IGL1 sidecar, relative to project root')
    parser.add_argument('--paused', action='store_true')
    args = parser.parse_args()
    binary = args.binary or native_entry()
    out = ROOT/'artifacts/world-walk'
    out.mkdir(parents=True, exist_ok=True)
    stamp = binary.stat()
    binary_stamp = [stamp.st_ino, stamp.st_size, stamp.st_mtime_ns]
    launch_path = out/'launch.json'
    if launch_path.exists() and sys.platform != 'win32':
        previous = json.loads(launch_path.read_text())
        probe = subprocess.run(['ps', '-p', str(previous['pid']), '-o', 'command='],
                               capture_output=True, text=True, check=False)
        if probe.returncode == 0 and str(binary) in probe.stdout and str(ROOT) in probe.stdout and '--scene ' + args.scene in probe.stdout:
            if previous.get('binary_stamp') != binary_stamp:
                raise RuntimeError('The previous demo still runs an older build; close its window before reopening')
            print(json.dumps(dict(pid=previous['pid'], already_running=True, scene=args.scene)))
            return
    c = Client(binary, window=True, log_path=out/'native.log', detached=True,
               startup_args=('--scene', args.scene, '--continuous', '--third-person', 'player'))
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                    clientInfo=dict(name='world-walk', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('set_viewport_performance', settings=dict(enabled=False, vsync=False))
        view = dict(actor='player', yaw_degrees=0, pitch_degrees=-10,
                    eye_height=1.65, fov_degrees=75, mode='third_person',
                    third_person=dict(distance_meters=4, shoulder_offset=.4))
        c.call('set_player_view', view=view)
        if args.lod:
            c.call('load_mesh_lods', path=args.lod)
            c.call('set_mesh_lod_settings', settings=dict(enabled=True, error_px=.25, color_error=.002))
        if not args.paused:
            c.call('play')
        c.call('set_viewport_performance', settings=dict(enabled=True, vsync=False))
        performance = None
        for _ in range(8):
            time.sleep(.3)
            performance = c.call('inspect_viewport_performance')
            if performance['sample']['intervals'] >= 8 and performance['sample']['seconds'] >= .9:
                break
        sample_complete = performance['sample']['seconds'] >= .9 and performance['sample']['intervals'] >= 8
        result = dict(binary_stamp=binary_stamp, sample_complete=sample_complete, performance=performance, pid=c.process.pid, scene=args.scene, view=view, playing=not args.paused,
                      controls='WASD / Shift / click+mouse / Tab / wheel / F3 / P')
        (out/'launch.json').write_text(json.dumps(result, indent=2))
        print(json.dumps(dict(pid=c.process.pid,scene=args.scene,playing=not args.paused,
                              fps=performance['sample']['fps'] if sample_complete else None,viewport=performance['sample']['viewport'],
                              details=str(out/'launch.json'))))
    except BaseException:
        c.close()
        raise
    else:
        # The native window explicitly survives MCP stdin EOF. A separate process
        # session also keeps terminal/client teardown from owning its lifetime.
        c.process.stdin.close()
        c.process.stdout.close()
        c.log.close()


if __name__ == '__main__':
    main()
