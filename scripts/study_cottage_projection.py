#!/usr/bin/env python3
"""Camera-only cottage experiment using native landmark projection and offscreen captures."""
import hashlib
import json
import os
from pathlib import Path

P = Path(__file__).resolve().parents[1]
R = P / 'artifacts/cottage-camera-study'
SOURCE = P / 'artifacts/cottage-normals/examples/cottage-spring.scene.json'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    for name in ['examples', 'artifacts']:
        (R / name).mkdir(parents=True, exist_ok=True)
    before_hash = digest(SOURCE)
    (R / 'examples/input.scene.json').write_bytes(SOURCE.read_bytes())
    os.environ['IGNIS_VERIFY_ROOT'] = str(R)
    from verify_lab import Client
    recipe = json.loads((P / 'examples/elven-cottage.recipe.json').read_text())
    m, p = recipe['main'], recipe['porch']
    # Authorial construction anchors; these are not automatically matched image features.
    landmarks = [dict(id='ridge-west', world=[m['left'], m['ridge'], m['ridge_z']]),
                 dict(id='ridge-east', world=[m['right'], m['ridge'], m['ridge_z']])]
    for i in range(p['bays'] + 1):
        x = p['left'] + i * (p['right'] - p['left']) / p['bays']
        landmarks += [dict(id=f'porch-capital-{i}', world=[x, 3.55, p['front']]),
                      dict(id=f'porch-foot-{i}', world=[x, .86, p['front']])]
    cameras = {name: dict(position=pos, target=[0, 2.65, .5], fov_degrees=34)
               for name, pos in [('baseline', [11, 8.7, 16]), ('lower', [11, 5.2, 16]),
                                 ('low', [11, 3.5, 16]), ('lower-front', [8, 5.2, 18])]}
    c = Client(P / 'target/release/ignis')
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                  clientInfo=dict(name='cottage-projection-study', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('load_scene', path='examples/input.scene.json')
        initial = c.call('inspect')
        reports = {}
        for name, camera in cameras.items():
            state = c.call('inspect')
            report = c.call('project_landmarks', width=1536, height=1024,
                            camera=camera, landmarks=landmarks)
            assert c.call('inspect') == state
            assert report['in_viewport'] == len(landmarks)
            assert report['compared'] == 0 and report['rmse_pixels'] is None
            a, b = [v['pixel'] for v in report['samples'][:2]]
            report['ridge_delta_pixels'] = [b[0] - a[0], b[1] - a[1]]
            c.call('set_camera', camera=camera)
            c.call('capture', path=f'artifacts/{name}.png', width=1536, height=1024)
            assert c.call('inspect')['scene']['meshes'] == initial['scene']['meshes']
            reports[name] = report
        assert reports['baseline']['ridge_delta_pixels'][1] > 0
        assert reports['low']['ridge_delta_pixels'][1] < 0
        # Isolate broad existing structure through public edits, without remeshing or
        # treating this old hypothesis as a corrected reconstruction.
        state = c.call('inspect')
        keep = {'masonry', 'carved-stone', 'roof-backing', 'oak', 'window-infill', 'porch-column'}
        meshes = state['scene']['meshes']
        edits = [dict(op='mesh', edit=dict(kind='remove_instance', id=i['id']))
                 for i in meshes['instances'] if i['asset'] not in keep]
        edits += [dict(op='mesh', edit=dict(kind='remove_asset', id=a['id']))
                  for a in meshes['assets'] if a['id'] not in keep]
        edits += [dict(op='mesh', edit=dict(kind='set_material', asset=a['id'], material={}))
                  for a in meshes['assets'] if a['id'] in keep]
        edits += [dict(op='mesh', edit=dict(kind='put_instance', instance={**i, 'color': [.64, .64, .64]}))
                  for i in meshes['instances'] if i['asset'] in keep]
        c.call('edit_scene', edits=edits)
        mass = c.call('inspect')['scene']['meshes']
        assert {a['id'] for a in mass['assets']} == keep
        for a in mass['assets']:
            original = next(old for old in meshes['assets'] if old['id'] == a['id'])
            assert a['geometry'] == original['geometry']
        mass_views = {name: dict(position=pos, target=[0, 2.65, .5], fov_degrees=fov)
                      for name, pos, fov in [('hero', [11, 3.5, 16], 34),
                                             ('plan', [0, 20, 1], 40),
                                             ('front', [0, 3, 20], 34),
                                             ('side', [20, 3, 0], 34),
                                             ('rear', [0, 3, -20], 34)]}
        for name, camera in mass_views.items():
            c.call('set_camera', camera=camera)
            c.call('capture', path=f'artifacts/mass-{name}.png', width=1536, height=1024)
            if name == 'hero':
                c.call('save_scene', path='examples/mass.scene.json')
        assert digest(SOURCE) == before_hash
        output = dict(passed=True, build_id=initial['build_id'], source=str(SOURCE.relative_to(P)),
                      source_sha256=before_hash, width=1536, height=1024,
                      reference_sha256=digest(P / 'docs/references/image-to-3d/elven-house.png'),
                      landmarks=landmarks, reports=reports, mass_views=mass_views,
                      mass_retained_assets=sorted(keep), mass_edits=edits,
                      limitations=['Camera comparison uses unchanged geometry/materials; mass isolation then removes details and neutralizes materials.',
                                   'Isolated mass is the existing structural hypothesis, not a corrected or accepted reference reconstruction.',
                                   'Anchors are authorial points, not inferred image correspondences.',
                                   'No camera calibration, occlusion proof, similarity score or artistic acceptance.'])
        (R / 'report.json').write_text(json.dumps(output, indent=2) + '\n')
        print(json.dumps(dict(passed=True, cameras=len(cameras), landmarks=len(landmarks),
                             build_id=initial['build_id'], ridge_delta_y={k: v['ridge_delta_pixels'][1] for k, v in reports.items()})))
    finally:
        c.close()


if __name__ == '__main__':
    main()
