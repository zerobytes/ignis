#!/usr/bin/env python3
"""Open a wind scene with its own GUI identity and the one current executable."""
import argparse
import plistlib
import subprocess
import sys
from pathlib import Path
from verify_lab import ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scene', default='examples/wind-vortex.scene.json')
    args = parser.parse_args()
    binary = ROOT / 'target/release/ignis'
    if not binary.is_file() or not (ROOT / args.scene).is_file():
        raise RuntimeError('Build the current executable and generate the wind scene first')
    options = ['--root', str(ROOT), '--scene', args.scene, '--window', '--continuous']
    if sys.platform != 'darwin':
        subprocess.Popen([str(binary), *options], start_new_session=True)
        return
    app = Path.home() / 'Applications/Ignis Wind.app'
    contents = app / 'Contents'
    info = contents / 'Info.plist'
    identifier = 'dev.ignisengine.wind'
    if info.exists() and plistlib.loads(info.read_bytes()).get('CFBundleIdentifier') != identifier:
        raise RuntimeError('Refusing to replace an unrelated Ignis Wind.app')
    entry = contents / 'MacOS/ignis-wind'
    entry.parent.mkdir(parents=True, exist_ok=True)
    if entry.exists() and not entry.is_symlink():
        raise RuntimeError('Refusing to replace a non-symlink wind executable')
    if entry.is_symlink() and entry.readlink() != binary:
        entry.unlink()
    if not entry.is_symlink():
        entry.symlink_to(binary)
    info.write_bytes(plistlib.dumps({'CFBundleIdentifier': identifier,
        'CFBundleDisplayName': 'Ignis Wind', 'CFBundleName': 'Ignis Wind',
        'CFBundleExecutable': 'ignis-wind', 'CFBundlePackageType': 'APPL',
        'CFBundleShortVersionString': '0.1.0', 'CFBundleVersion': '1',
        'NSHighResolutionCapable': True}))
    subprocess.run(['open', '-n', str(app), '--args', *options], check=True)


if __name__ == '__main__':
    main()
