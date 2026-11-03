#!/usr/bin/env python3
"""Open the verified emerald scene with its matching experimental grass field."""
import argparse
import hashlib
import json
import os
from pathlib import Path
from verify_lab import Client, ROOT

OUT = ROOT / 'artifacts/emerald-meadow'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    args = parser.parse_args()
    bundle = json.loads((OUT / 'bundle.json').read_text())
    for key in ['scene', 'field']:
        path = OUT / bundle[key]
        if hashlib.sha256(path.read_bytes()).hexdigest() != bundle[key + '_sha256']:
            parser.error(f'{key} changed; rebuild/validate the bundle first')
    old_field = os.environ.get('IGNIS_HORIZON_FIELD')
    old_proxy = os.environ.pop('IGNIS_HORIZON_PROXIES', None)
    os.environ['IGNIS_HORIZON_FIELD'] = str(OUT / bundle['field'])
    try:
        client = Client(args.binary.resolve(), window=True, detached=True,
            log_path=OUT / 'walk.log', startup_args=('--scene', str((OUT / bundle['scene']).relative_to(ROOT)),
                '--continuous', '--first-person', 'player'))
    finally:
        if old_field is None:
            os.environ.pop('IGNIS_HORIZON_FIELD', None)
        else:
            os.environ['IGNIS_HORIZON_FIELD'] = old_field
        if old_proxy is not None:
            os.environ['IGNIS_HORIZON_PROXIES'] = old_proxy
    try:
        init = client.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
            clientInfo=dict(name='emerald-meadow', version='1')))
        client.request('notifications/initialized', notification=True)
        client.call('set_viewport_performance', settings=dict(enabled=True, vsync=False))
        client.call('play')
        result = dict(pid=client.process.pid, scene=str(OUT / bundle['scene']),
            field=str(OUT / bundle['field']), binary=str(args.binary.resolve()),
            controls='Click captures mouse; WASD; Shift run; Esc releases mouse; F3 FPS',
            experimental=True)
        (OUT / 'window.json').write_text(json.dumps(result, indent=2))
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
