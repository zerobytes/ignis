#!/usr/bin/env python3
"""Open the living broadleaf specimen in the native editor, with its age slider."""
import argparse
import json
from verify_lab import Client, ROOT

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--years', type=int, default=12)
    args = parser.parse_args()
    if not 0 <= args.years <= 100: parser.error('--years must be between0 and100')
    out = ROOT / 'artifacts/living-tree'
    c = Client(ROOT/'target/release/ignis', window=True, detached=True,
        log_path=out/'window.log', startup_args=('--editor', '--scene', 'artifacts/living-tree/tree.scene.json'))
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='living-tree-editor', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('botany', operation=dict(kind='load', path='assets/botany/living-tree.ignis-botany', publish=True))
        c.call('botany', operation=dict(kind='living', prefix='living', action=dict(kind='advance', time=args.years*365)))
        c.call('botany', operation=dict(kind='open', open=True, prefix='living'))
        result = dict(pid=c.process.pid, age_years=args.years, build_id=c.call('inspect')['build_id'])
        (out/'window.json').write_text(json.dumps(result, indent=2)+'\n')
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
