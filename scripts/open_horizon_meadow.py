#!/usr/bin/env python3
"""Open the current native 1km field laboratory, preserving other windows."""
import argparse
import json
import os
from pathlib import Path
import subprocess
from verify_lab import Client

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'artifacts/horizon-meadow'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    args = parser.parse_args()
    field = OUT / 'ground.ignis-field'
    scene = OUT / 'world.scene.json'
    if not field.is_file() or not scene.is_file():
        parser.error('Compile the laboratory with the horizon_meadow example first')
    previous = os.environ.get('IGNIS_HORIZON_FIELD')
    os.environ['IGNIS_HORIZON_FIELD'] = str(field)
    try:
        client = Client(args.binary.resolve(), window=True, detached=True, log_path=OUT/'walk.log',
            startup_args=('--scene', str(scene.relative_to(ROOT)), '--continuous', '--first-person', 'player'))
    finally:
        if previous is None:
            os.environ.pop('IGNIS_HORIZON_FIELD', None)
        else:
            os.environ['IGNIS_HORIZON_FIELD'] = previous
    try:
        client.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='horizon-meadow', version='1')))
        client.request('notifications/initialized', notification=True)
        client.call('set_viewport_performance', settings=dict(enabled=True, vsync=False))
        client.call('play')
        result = dict(pid=client.process.pid, binary=str(args.binary.resolve()), scene=str(scene), field=str(field),
            controls='Click to capture mouse; WASD; Shift run; Esc release/close; F3 FPS')
        (OUT/'window.json').write_text(json.dumps(result, indent=2))
        print(json.dumps(result))
    except BaseException:
        client.close()
        raise
    else:
        client.process.stdin.close()
        client.process.stdout.close()
        client.log.close()


if __name__ == '__main__':
    main()
