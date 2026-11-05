#!/usr/bin/env python3
"""Open the living giant sequoia specimen in the native editor, with its age slider."""
import argparse
import json
from verify_lab import Client, ROOT

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--years', type=int, default=1000)
    parser.add_argument('--bark-closeup', action='store_true', help='Open on the trunk material detail')
    args = parser.parse_args()
    if args.years < 0: parser.error('--years must be nonnegative; native time budgets apply')
    out = ROOT / 'artifacts/sequoia-realistic'
    c = Client(ROOT/'target/release/ignis', window=True, detached=True,
        log_path=out/'window.log', startup_args=('--editor', '--scene', 'artifacts/sequoia-realistic/sequoia.scene.json'))
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='sequoia-editor', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('botany', operation=dict(kind='load', path='assets/botany/sequoia-realistic.ignis-botany', publish=True))
        c.call('botany', operation=dict(kind='living', prefix='sequoia_real', action=dict(kind='advance', time=args.years*365)))
        for part,name in [('wood','bark'),('leaves','foliage')]:
            c.call('material',operation=dict(kind='load',asset=f'sequoia_real_{part}',path=f'assets/materials/sequoia/{name}.ignis-material'))
        c.call('botany', operation=dict(kind='open', open=True, prefix='sequoia_real'))
        if args.bark_closeup:
            c.call('set_camera',camera=dict(position=[11,5,15],target=[0,6,0],fov_degrees=44))
        result = dict(bark_closeup=args.bark_closeup,pid=c.process.pid, age_years=args.years, build_id=c.call('inspect')['build_id'])
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
