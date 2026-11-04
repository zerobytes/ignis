#!/usr/bin/env python3
"""Real MCP light/color, formats and white-image regression; no native window."""
import argparse
import copy
import hashlib
import json
from verify_lab import Client, ROOT
from open_earth_walk import init
from png_pixels import rgba8

OUT = ROOT / 'artifacts/directional-light-color'
PREFIX = 'artifacts/directional-light-color'
MATERIALS = {
    'diffuse': {},
    'dielectric': {'reflectance': {'roughness': .5, 'f0': .04}},
    'metal': {'reflectance': {'roughness': .5, 'f0': .04, 'metallic': 1}},
    'glass': {'surface': {'version': 1, 'mode': 'blend', 'glass': {
        'ior': 1.5, 'roughness': .5, 'thickness_m': .004,
        'attenuation_color': [1,1,1], 'attenuation_distance_m': 1}}},
    'emissive': {'surface': {'version': 1, 'emissive': [.2, .1, .3]}},
    'transmission': {'transmission': .7},
}


def run(baseline=False):
    OUT.mkdir(parents=True, exist_ok=True)
    c = Client(ROOT / 'target/release/ignis', log_path=OUT / ('before.log' if baseline else 'verify.log'))
    results = []
    try:
        init(c)
        build = c.call('inspect')['build_id']
        if baseline:
            c.call('generate', recipe={'width': 12, 'depth': 12, 'seed': 113, 'obstacles': 0})
            template = c.call('inspect')['scene']
            template['recipe'] = None
            for e in template['entities']:
                e['visible'] = False
            template['camera'] = {'position': [0, 1, 6], 'target': [0, 1, 0], 'fov_degrees': 45}
            template['light'].update(direction=[0, 0, -1], ambient=.23, environment=None,
                                     atmosphere=None, shadow=None, direct_intensity=1)
            template['light'].pop('direct_color', None)
            for name, material in MATERIALS.items():
                scene = copy.deepcopy(template)
                scene.setdefault('meshes', {})
                scene['meshes']['assets'] = [{'id': 'sample', 'shading': 'smooth', 'material': material,
                    'geometry': {'positions': [[-2,-1,0],[2,-1,0],[2,3,0],[-2,3,0]],
                                 'triangles': [[0,1,2],[0,2,3]]}}]
                scene['meshes']['instances'] = [{'id': 'sample', 'asset': 'sample', 'position': [0,0,0],
                    'rotation': [0,0,0,1], 'scale': 1, 'color': [.25,.25,.25]}]
                (OUT / f'{name}.scene.json').write_text(json.dumps(scene))
        else:
            schema = json.dumps(c.request('tools/list')['result'])
            assert 'direct_color' in schema
        for name in MATERIALS:
            c.call('load_scene', path=f'{PREFIX}/{name}.scene.json')
            light = c.call('inspect')['scene']['light']
            for intensity in [1, .7, 0, 4]:
                light.update(direct_intensity=intensity)
                if not baseline:
                    light['direct_color'] = [1,1,1]
                c.call('set_light', light=light)
                image_name = f'{name}-{intensity}'
                side = 'before' if baseline else 'after'
                path = f'{PREFIX}/{side}-{image_name}.png'
                c.call('capture', path=path, width=256, height=256)
                if not baseline:
                    assert rgba8((ROOT / path).read_bytes()) == rgba8((OUT / f'before-{image_name}.png').read_bytes()), image_name
                results.append({'fixture': name, 'intensity': intensity,
                                'sha256': hashlib.sha256((ROOT / path).read_bytes()).hexdigest()})
        if not baseline:
            c.call('load_scene', path=f'{PREFIX}/diffuse.scene.json')
            scene = c.call('inspect')['scene']
            light = scene['light']; light.update(direct_color=[1,.8,.5], direct_intensity=2)
            c.call('set_light', light=light)
            published = c.call('inspect')['scene']
            assert published['version'] >= 120
            assert max(abs(a-b) for a,b in zip(published['light']['direct_color'], [1,.8,.5])) < 1e-6
            for color in [[-1,1,1], [1,2,1], [1,1,-.01]]:
                bad = copy.deepcopy(light); bad['direct_color'] = color
                before = c.call('inspect')
                c.call('set_light', light=bad, expected_error=True)
                assert c.call('inspect') == before
            c.call('save_scene', path=f'{PREFIX}/warm.scene.json')
            c.call('save_replay', path=f'{PREFIX}/warm.replay.json')
            c.call('save_checkpoint', path=f'{PREFIX}/warm.ignis-save')
            c.call('save_portable_save', path=f'{PREFIX}/warm.ignis-portable')
            replay = c.call('run_replay', path=f'{PREFIX}/warm.replay.json')
            assert replay['matched'], replay
            for op, path in [('load_scene','warm.scene.json'),
                             ('load_checkpoint','warm.ignis-save'), ('load_portable_save','warm.ignis-portable')]:
                different = copy.deepcopy(light)
                different.update(direct_color=[0,0,1], direct_intensity=.2)
                c.call('set_light', light=different)
                assert c.call('inspect')['scene']['light'] != published['light']
                c.call(op, path=f'{PREFIX}/{path}')
                assert c.call('inspect')['scene']['light'] == published['light'], op
            c.call('capture', path=f'{PREFIX}/warm.png', width=256, height=256)
        report = {'build_id': build, 'native_window': False, 'baseline': baseline,
                  'white_pairs': len(results), 'results': results, 'passed': True}
        if not baseline:
            report['baseline_build_id'] = json.loads((OUT / 'before.json').read_text())['build_id']
        (OUT / ('before.json' if baseline else 'verification.json')).write_text(json.dumps(report, indent=2))
        print(json.dumps({'build': build, 'baseline': baseline, 'white_pairs': len(results), 'passed': True}), flush=True)
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--baseline', action='store_true', help='Capture the unmodified shader before rebuilding')
    run(p.parse_args().baseline)
