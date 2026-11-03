#!/usr/bin/env python3
"""Same authored landscape: unpackaged source versus compiled selection, no blur."""
import copy
import json
import math
from png_pixels import rgba8
from verify_lab import Client, ROOT
from verify_materials import f32

OUT = ROOT / 'artifacts/render-comparison'
PREFIX = 'artifacts/render-comparison/'


def compare():
    OUT.mkdir(parents=True, exist_ok=True)
    source = json.loads((ROOT / 'artifacts/atmosphere-landscape/verification.json').read_text())
    cases = [dict(id=c['id'], label=c['label'], camera=c['camera'], light=copy.deepcopy(c['light']))
             for c in source['cases']]
    motion = []
    for i in range(12):
        t = i / 11 * 2 * math.pi
        light = copy.deepcopy(next(c['light'] for c in cases if c['id'] == 'haze'))
        light['direction'] = [.65 * math.cos(t), -1, -.25 + .3 * math.sin(t)]
        motion.append(dict(id=f'motion-{i}', camera=f32(dict(
            position=[400 * math.sin(t), 1200 + 150 * math.sin(t), 7000 + 2000 * math.cos(t)],
            target=[0, 1300, 0], fov_degrees=55)), light=f32(light)))
    client = Client(ROOT / 'target/release/ignis')
    references = {}
    worst = -1
    try:
        client.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                       clientInfo=dict(name='render-comparison', version='1')))
        client.request('notifications/initialized', notification=True)
        for selected in [False, True]:
            client.call('load_scene', path='artifacts/atmosphere-landscape/landscape.scene.json')
            if not selected:
                client.call('edit_scene', edits=[dict(op='clear_distant_meshes')])
                assert not client.call('inspect')['scene'].get('distant_meshes', [])
            else:
                client.call('save_scene', path=PREFIX + 'landscape.scene.json')
            for case in cases + motion:
                light = copy.deepcopy(case['light'])
                light['atmosphere'].pop('edge_softness', None)
                light['atmosphere']['reduce_details'] = selected
                client.call('set_light', light=light)
                client.call('set_camera', camera=case['camera'])
                suffix = 'selected' if selected else 'full'
                name = 'motion' if case in motion else case['id']
                path = PREFIX + name + '-' + suffix + '.png'
                adapter = client.call('capture', path=path, width=1728, height=1117)['adapter']
                assert adapter['atmospheric_edges']['passes'] == 0
                assert adapter['atmospheric_edges']['image_bytes'] == 0
                png = (ROOT / path).read_bytes()
                case[suffix] = adapter['distant_selection']
                if not selected:
                    assert case[suffix]['compiled_bytes'] == 0
                    assert case[suffix]['substituted_instances'] == 0
                    references[case['id']] = png
                else:
                    full = references.pop(case['id'])
                    if full == png:
                        case.update(max_rgb8=0, mae_rgb8=0)
                    else:
                        a, b = rgba8(full), rgba8(png)
                        assert a[:2] == b[:2] and len(a[2]) == len(b[2])
                        differences = [abs(x-y) for i, (x, y) in enumerate(zip(a[2], b[2])) if i % 4 != 3]
                        case.update(max_rgb8=max(differences), mae_rgb8=sum(differences)/len(differences))
                    assert case['max_rgb8'] <= 3, case
                    if case in motion and case['max_rgb8'] > worst:
                        worst = case['max_rgb8']
                        (OUT / 'motion-worst-full.png').write_bytes(full)
                        (OUT / 'motion-worst-selected.png').write_bytes(png)
                    print(json.dumps(dict(case=case['id'], max_rgb8=case['max_rgb8'],
                                          substitutions=case[suffix]['substituted_instances'])), flush=True)
        report = dict(build_id=client.call('inspect')['build_id'], passed=True, width=1728, height=1117,
                      cases=cases, motion=motion, chains=source['chains'],
                      reference='No compiled packages; full source geometry/materials; same authored atmosphere; blur absent.')
        (OUT / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        (OUT / 'comparison.md').write_text(
            '# Referência completa × otimização nativa\n\nMesma cena, câmera, iluminação e aparência atmosférica. Blur desligado nos dois caminhos.\n\n'
            + '\n\n'.join(f'## {label}\n\n![{label}]({OUT / filename})' for label, filename in [
                ('Sem representações compiladas', 'haze-full.png'), ('Seleção otimizada', 'haze-selected.png')]) + '\n')
    finally:
        client.close()


if __name__ == '__main__':
    compare()
