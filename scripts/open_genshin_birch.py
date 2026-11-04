#!/usr/bin/env python3
"""Open the prepared birch review in a separate native window."""
import argparse
import json
import os
from pathlib import Path

from verify_lab import Client, ROOT


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    args = p.parse_args()
    out = ROOT / 'artifacts/genshin-birch'
    scene = out / 'gallery.scene.json'
    if not scene.is_file():
        p.error('Run scripts/genshin_birch.py and scripts/demo_genshin_birch.py first')
    removed = {k: os.environ.pop(k) for k in ['IGNIS_HORIZON_FIELD', 'IGNIS_HORIZON_PROXIES'] if k in os.environ}
    try:
        c = Client(args.binary.resolve(), window=True, detached=True,
                   log_path=out / 'window.log', startup_args=('--scene', str(scene.relative_to(ROOT))))
    finally:
        os.environ.update(removed)
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='birch-review-window', version='1')))
        c.request('notifications/initialized', notification=True)
        state = c.call('inspect')
        result = dict(pid=c.process.pid, build_id=state['build_id'], scene=str(scene),
                      binary=str(args.binary.resolve()), editable=True)
        (out / 'window.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))
    except BaseException:
        c.close()
        raise
    else:
        # The native editor remains open after this launcher's transport closes.
        c.process.stdin.close()
        c.process.stdout.close()
        c.log.close()


if __name__ == '__main__':
    main()
