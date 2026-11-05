#!/usr/bin/env python3
"""Open the living sequoia volume study in the native editor."""
import argparse
import json
from verify_lab import Client, ROOT

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--textured', action='store_true')
    parser.add_argument('--shadows', action='store_true', help='Keep projected shadows; narrow valleys currently show shadow aliasing')
    args = parser.parse_args()
    out = ROOT/'artifacts/sequoia-volume'
    scene = 'tree.scene.json' if args.textured else 'neutral.scene.json'
    c = Client(ROOT/'target/release/ignis', window=True, detached=True,
        log_path=out/'window.log', startup_args=('--editor', '--scene', f'artifacts/sequoia-volume/{scene}'))
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='sequoia-volume-editor', version='1')))
        c.request('notifications/initialized', notification=True)
        if not args.shadows:
            light=json.loads((out/scene).read_text())['light'];light.pop('shadow',None)
            c.call('set_light',light=light)
        c.call('botany', operation=dict(kind='open', open=True, prefix='sequoia_volume'))
        c.call('set_camera', camera=dict(position=[28,28,44],target=[0,36,0],fov_degrees=42))
        c.call('capture_editor',path='artifacts/sequoia-volume/window.png',width=1700,height=1200)
        result=dict(pid=c.process.pid,build=c.call('inspect')['build_id'],textured=args.textured)
        (out/'window.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps(result))
    except BaseException:
        c.close()
        raise
    else:
        c.process.stdin.close();c.process.stdout.close();c.log.close()

if __name__=='__main__':main()
