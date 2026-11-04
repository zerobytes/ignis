#!/usr/bin/env python3
"""Open the current editable living specimen in its own native editor window."""
import argparse
import json
import os
from pathlib import Path

from verify_lab import Client, ROOT


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    p.add_argument('--from-birth', action='store_true', help='Start the test specimen at birth for the age slider')
    args = p.parse_args()
    out = ROOT / 'artifacts/living-plants'
    scene = out / 'living.scene.json'
    if not scene.is_file():
        p.error('Run scripts/demo_living_plant.py first')
    removed = {k: os.environ.pop(k) for k in ['IGNIS_HORIZON_FIELD', 'IGNIS_HORIZON_PROXIES'] if k in os.environ}
    try:
        c = Client(args.binary.resolve(), window=True, detached=True, log_path=out / 'window.log',
                   startup_args=('--editor', '--scene', str(scene.relative_to(ROOT))))
    finally:
        os.environ.update(removed)
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='living-plant-editor', version='1')))
        c.request('notifications/initialized', notification=True)
        if args.from_birth:
            c.call('botany', operation=dict(kind='load', path='assets/botany/living-specimen.ignis-botany', publish=True))
        c.call('botany', operation=dict(kind='open', open=True, prefix='living'))
        plant = c.call('botany', operation=dict(kind='living', prefix='living', action=dict(kind='inspect')))
        state = c.call('inspect')
        result = dict(pid=c.process.pid, build_id=state['build_id'], scene=str(scene), day=plant['plant']['production']['plant']['time'])
        (out / 'window.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))
    except BaseException:
        c.close()
        raise
    else:
        c.process.stdin.close()
        c.process.stdout.close()
        c.log.close()


if __name__ == '__main__':
    main()
