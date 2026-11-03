#!/usr/bin/env python3
"""Publish terrain/landforms through the current renderer; preserve approved ridges."""
import copy
import hashlib
import json
import subprocess
from dataclasses import asdict

from mountain_recipe import RidgeShape, ridge
from verify_lab import Client, ROOT

OUT = ROOT / 'artifacts/terrain-render-integration'
PREFIX = 'artifacts/terrain-render-integration/'
APPROVED = {
    'range-a': '96b8c27f0bc4f463c498e976c9c9150dc33d13cadc2567c004b41d9c7d338f7e',
    'range-b': 'fde911452dfb1a882a22ba09367efa73be595ddbb9827a5678beb93255a0bcc9',
}


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def verify():
    OUT.mkdir(parents=True, exist_ok=True)
    commits = ['2b561a2', 'de05154', '3b01500', 'd5ca9fc', '91f2503']
    for commit in commits:
        subprocess.run(['git', 'merge-base', '--is-ancestor', commit, 'main'], cwd=ROOT, check=True)
    for name, phase in [('range-a', .3), ('range-b', 1.7)]:
        assert digest(ridge(name, phase)) == APPROVED[name], name
    c = Client(ROOT / 'target/release/ignis')
    cases = []
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                    clientInfo=dict(name='terrain-render-integration', version='1')))
        c.request('notifications/initialized', notification=True)
        # Original visual composition is preserved. Variants change only the
        # build recipe, not the renderer or a semantic asset-specific path.
        source = json.loads((ROOT / 'artifacts/landscape-live/landscape.scene.json').read_text())
        variants = [('approved', RidgeShape()),
                    ('rounded', RidgeShape(height_meters=600, slope_exponent=2.3, folds_meters=12)),
                    ('sharp', RidgeShape(height_meters=1100, crest_wander=.22,
                                         slope_exponent=.7, folds_meters=70))]
        for label, shape in variants:
            scene = copy.deepcopy(source)
            if label != 'approved':
                for i, asset in enumerate(scene['meshes']['assets']):
                    if asset['id'] in APPROVED:
                        scene['meshes']['assets'][i] = ridge(asset['id'], .3 if asset['id'] == 'range-a' else 1.7,
                                                            shape=shape)
                scene['distant_meshes'] = []
            # Files below are derived artifacts. No write to the human's scene.
            path = PREFIX + label + '.scene.json'
            (ROOT / path).write_text(json.dumps(scene, separators=(',', ':')))
            c.call('load_scene', path=path)
            c.call('edit_scene', edits=[dict(op='compile_distant_meshes')])
            c.call('save_scene', path=path)
            cases.append(dict(id=label, path=path, recipe=asdict(shape)))
        for label, input_path in [('terrain', 'artifacts/terrain-tools.scene.json'),
                                  ('landforms', 'artifacts/landforms.scene.json')]:
            scene = json.loads((ROOT / input_path).read_text())
            originals = scene['meshes']['instances'][:]
            # Offscreen duplicates also exercise shared static meshes and
            # conservative camera rejection for native generated assets.
            for i, original in enumerate(originals):
                other = copy.deepcopy(original)
                other['id'] = f'integration-copy-{i}'
                other['position'][0] += 4000
                scene['meshes']['instances'].append(other)
            scene['light']['atmosphere'] = copy.deepcopy(source['light']['atmosphere'])
            scene['light']['atmosphere'].pop('edge_softness', None)
            scene['light']['atmosphere']['layers'].update(detail_start=80, detail_end=300)
            path = PREFIX + label + '.scene.json'
            (ROOT / path).write_text(json.dumps(scene, separators=(',', ':')))
            c.call('load_scene', path=path)
            c.call('edit_scene', edits=[dict(op='compile_distant_meshes')])
            c.call('save_scene', path=path)
            cases.append(dict(id=label, path=path, native_source=input_path))
        report = dict(build_id=c.call('inspect')['build_id'], commits_in_main=commits,
                      approved_hashes=APPROVED, cases=cases,
                      scope=dict(visual_integration=True, gameplay_collision=False,
                                 streaming=False, continuous_procedural_world=False))
        (OUT / 'manifest.json').write_text(json.dumps(report, indent=2))
        print(json.dumps(dict(prepared=True, cases=len(cases), build_id=report['build_id'])))
    finally:
        c.close()


if __name__ == '__main__':
    verify()
