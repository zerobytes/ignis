#!/usr/bin/env python3
"""Native seeded broadleaf growth review. No Python tree geometry generation."""
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT
from demo_groundcover import fit

OUT = ROOT / 'artifacts/living-tree'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    c = Client(args.binary.resolve(), log_path=OUT / 'runtime.log', startup_args=('--scene', 'artifacts/living-plants/empty.scene.json'))
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='living-tree', version='1')))
        c.request('notifications/initialized', notification=True)
        definition = json.loads((ROOT / 'examples/living-tree.recipe.json').read_text())
        c.call('botany', operation=dict(kind='create_living', definition=definition, born=0))
        c.call('botany', operation=dict(kind='save', prefix='living', path='assets/botany/living-tree.ignis-botany'))
        c.call('set_camera', camera=dict(position=[10, 7, 14], target=[0, 3.1, 0], fov_degrees=40))
        c.call('set_light', light=dict(direction=[.5, -.82, -.35], ambient=.3, direct_intensity=1.1,
            environment=dict(sky=[.46, .61, .78], horizon=[.59, .65, .62], ground=[.16, .19, .08], axis=[0, 1, 0], intensity=.35),
            shadow=dict(center=[0, 3, 0], extent=10, resolution=2048, bias=.0005)))
        stages = []
        for years in [0, 1, 2, 5, 10, 12, 20]:
            c.call('botany', operation=dict(kind='living', prefix='living', action=dict(kind='advance', time=years * 365)))
            c.call('capture', path=f'artifacts/living-tree/age-{years}.png', width=1600, height=1200)
            r = c.call('botany', operation=dict(kind='inspect'))['recipes'][0]
            leaves=[o for o in r['source']['organs'] if o['shape']['kind']=='blade']
            stages.append(dict(years=years, organs=len(r['source']['organs']),leaves=len(leaves)))
            assert leaves, f'Missing seedling foliage at {years} years'
            if years in [0,1,2]:
                points=[p for o in r['source']['organs'] for p in o['path']]
                lo=[min(p[i] for p in points)-0.15 for i in range(3)]
                hi=[max(p[i] for p in points)+0.15 for i in range(3)]
                fit(c,lo,hi,OUT/f'young-{years}-close.png',width=1200,height=1100)
                c.call('set_camera', camera=dict(position=[10,7,14],target=[0,3.1,0],fov_degrees=40))
        c.call('botany', operation=dict(kind='open', open=True, prefix='living'))
        c.call('save_scene', path='artifacts/living-tree/tree.scene.json')
        c.call('capture_editor', path='artifacts/living-tree/editor.png', width=1600, height=1100)
        c.call('load_scene', path='artifacts/living-tree/tree.scene.json')
        c.call('capture', path='artifacts/living-tree/roundtrip.png', width=1600, height=1200)
        assert (OUT/'roundtrip.png').read_bytes() == (OUT/'age-20.png').read_bytes()
        result = dict(stages=stages, roundtrip_png_exact=True, build_id=c.call('inspect')['build_id'])
        (OUT/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps(result))
    finally:
        c.close()

if __name__ == '__main__':
    main()
