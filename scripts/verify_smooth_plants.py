#!/usr/bin/env python3
"""Compare the shared tree branch with linear/PCHIP sections through native MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window=False):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-smooth-plants-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        assert 'pchip' in json.dumps(c.request('tools/list')['result'])
        c.call('generate', recipe={'seed': 20261026, 'width': 20, 'depth': 20, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        for e in entities:
            e['visible'] = False
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        camera = {'position': [2.8, 0.9, 4.8], 'target': [0.65, -0.10, 0.15], 'fov_degrees': 36}
        c.call('set_camera', camera=camera)
        c.call('set_light', light={'direction': [-0.6, -0.8, -0.7], 'ambient': 0.35})
        a = json.loads((ROOT/'examples/native-tree-branch.plant.json').read_text())
        for station in a['sections']['stations']:
            station.setdefault('offset', [0, 0])
        id = a['id']

        def put(d, operation='edit_plant', **kwargs):
            return c.call(operation, edit={'kind': 'put', 'definition': d, 'detail_policy': 'physical', 'animation_translation_policy': 'physical'}, **kwargs)

        def asset():
            return c.call('inspect_mesh', asset=id)['asset']

        def mesh(kind, **kwargs):
            return c.call('edit_mesh', edit={'kind': kind, 'asset': id, **kwargs})

        def capture(name):
            c.call('capture', path='artifacts/smooth-'+name+'.png', width=1000, height=1000)

        put(a)
        mesh('set_shading', shading='smooth')
        mesh('set_material', material={'pattern': {'kind': 'cellular', 'frequency': [24, 4, 24], 'low': [0.62, 0.60, 0.55], 'high': [1.10, 1.05, 0.97], 'bump': 0.0014, 'warp': 1.1, 'octaves': 3, 'seed': 137, 'contrast': 1.2}, 'transmission': 0})
        base = asset()
        mesh('set_material_coordinates', coordinates=base['geometry']['positions'])
        point = max((p for p in base['geometry']['positions'] if -1.3 < p[1] < -0.9), key=lambda p: p[2])
        mesh('sculpt', stroke={'kind': 'grab', 'centers': [point], 'radius': 0.08, 'strength': 0.0015, 'direction': [0, 0, 1]})
        s = a['sections']['stations']
        height = a['profile']['height']
        root = [s[0]['offset'][0], -height/2, s[0]['offset'][1]]
        # Off-station bind origin probes the smooth volume map, not only fixed knots.
        tip = [(s[1]['offset'][0]+s[2]['offset'][0])/2,
               ((s[1]['at']+s[2]['at'])/2-0.5)*height,
               (s[1]['offset'][1]+s[2]['offset'][1])/2]
        rest = [x-y for x, y in zip(tip, root)]
        bones = [{'name': 'base', 'parent': None, 'translation': root, 'rotation': [0, 0, 0, 1]},
                 {'name': 'branch', 'parent': 0, 'translation': rest, 'rotation': [0, 0, 0, 1]}]
        weights = []
        for x, y, z in asset()['geometry']['positions']:
            t = max(0, min(1, y/height+0.5))
            weights.append({'joints': [0, 1, 0, 0], 'weights': [1-t, t, 0, 0]})
        mesh('set_rig', rig={'bones': bones, 'weights': weights})
        angle = math.radians(6)/2
        mesh('put_clip', clip={'id': 'bend', 'duration_ticks': 120, 'tracks': [{'bone': 1, 'keys': [
            {'tick': t, 'translation': rest, 'rotation': [0, 0, math.sin(v), math.cos(v)]}
            for t, v in [(0, 0), (60, angle), (120, 0)]]}]})
        instance = {'id': 'branch-instance', 'asset': id, 'position': [0, 0, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.25, 0.18, 0.11], 'playback': {'clip': 'bend', 'offset_ticks': 0, 'looping': True}}
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        original = asset()
        source = c.call('inspect_plant', id=id)['plant']['source']
        b = copy.deepcopy(a)
        b['sections']['stations'][1]['offset'][0] += 0.18
        b['sections']['stations'][1]['offset'][1] -= 0.10
        smooth_a = copy.deepcopy(a); smooth_a['sections']['interpolation'] = 'pchip'
        smooth_b = copy.deepcopy(b); smooth_b['sections']['interpolation'] = 'pchip'
        cases = [('linear-A', a), ('pchip-A', smooth_a), ('linear-B', b), ('pchip-B', smooth_b)]
        evaluated = {}
        for name, drawing in cases:
            before = c.call('inspect')
            put(drawing, 'preview_plant')
            assert c.call('inspect') == before
            put(drawing)
            value = asset(); evaluated[name] = value
            assert c.call('inspect_plant', id=id)['plant']['source'] == source
            assert value['geometry']['triangles'] == original['geometry']['triangles']
            assert value['geometry']['material_coordinates'] == original['geometry']['material_coordinates']
            assert value['rig']['weights'] == original['rig']['weights']
            assert value['shading'] == original['shading'] and value['material'] == original['material']
            assert value['clips'][0]['tracks'][0]['keys'][0]['translation'] == value['rig']['bones'][1]['translation']
            assert value['clips'][0]['tracks'][0]['keys'][1]['rotation'] == original['clips'][0]['tracks'][0]['keys'][1]['rotation']
            # The six insertion centers are checked using the untouched base
            # sampling. Sculpted surface centers need not be planar or symmetric.
            empty = copy.deepcopy(drawing); empty['id'] = 'base-probe'
            empty['profile']['id'] = 'base-probe-profile'; empty['sections']['id'] = 'base-probe-sections'
            preview = put(empty, 'preview_plant')
            g = next(x for x in preview['assets'] if x['id'] == empty['id'])['geometry']
            for station in drawing['sections']['stations'][2:]:
                y = (station['at']-0.5)*height
                ring = [p for p in g['positions'] if abs(p[1]-y) < 0.000002]
                # Caps are excluded: all six insertion rings here are material
                # cross sections, and symmetric cap grids keep the same center.
                assert ring
                center = [sum(p[k] for p in ring)/len(ring) for k in range(3)]
                assert math.dist(center, [station['offset'][0], y, station['offset'][1]]) < 0.000002
            capture(name)
        assert evaluated['linear-A'] == original
        for name in ['A', 'B']:
            assert evaluated['linear-'+name]['geometry']['positions'] != evaluated['pchip-'+name]['geometry']['positions']
            assert (ROOT/f'artifacts/smooth-linear-{name}.png').read_bytes() != (ROOT/f'artifacts/smooth-pchip-{name}.png').read_bytes()
        checks.append('Four fixed-camera cases from the same branch preserve canonical sculpt/material/weights/TR and six section centers with pure preview')

        for _ in range(3):
            for name, drawing in cases:
                put(drawing); assert asset() == evaluated[name]
        c.call('undo'); assert asset() == evaluated['linear-B']
        c.call('redo'); assert asset() == evaluated['pchip-B']
        assert c.call('inspect_plant', id=id)['plant']['source'] == source
        assert evaluated['pchip-B']['rig']['bones'] != evaluated['linear-B']['rig']['bones']
        checks.append('Linear/PCHIP and shape A-B-A toggles adapt off-station bind origin and motion without drift or rebasing; undo/redo exact')

        dense_cases = [('linear-B-dense', b), ('pchip-A-dense', smooth_a), ('pchip-B-dense', smooth_b)]
        dense_result = None
        for name, drawing in dense_cases:
            drawing = copy.deepcopy(drawing); drawing['segments'] = [4, 6, 4]
            put(drawing)
            assert c.call('inspect_plant', id=id)['plant']['source'] == source
            dense_result = asset()
            topology = c.call('inspect_mesh_topology', asset=id)['topology']
            assert topology['boundary_edges'] == topology['non_manifold_edges'] == 0
            assert not topology['isolated_vertices']
            capture(name)
        assert len(dense_result['geometry']['positions']) == 706
        assert len(dense_result['geometry']['triangles']) == 1408
        checks.append('Matched dense Linear/PCHIP comparisons use explicit706v1408t budget; connected topology and source stay valid')

        before = c.call('inspect')
        for kind in ['mode', 'budget']:
            bad = copy.deepcopy(smooth_b)
            if kind == 'mode': bad['sections']['interpolation'] = 'unbounded_spline'
            else: bad['segments'] = [63, 63, 63]
            put(bad, expected_error=True)
            assert c.call('inspect') == before
        put(smooth_b, expected_revision=before['revision']-1, expected_error=True)
        assert c.call('inspect') == before
        c.call('save_scene', path='examples/native-smooth-branch.scene.json')
        saved = json.loads((ROOT/'examples/native-smooth-branch.scene.json').read_text())
        assert saved['version'] == SCENE_VERSION
        saved['version'] = 56
        (ROOT/'examples/old-smooth-branch.scene.json').write_text(json.dumps(saved))
        c.call('load_scene', path='examples/old-smooth-branch.scene.json', expected_error=True)
        assert c.call('inspect') == before
        checks.append('Unknown interpolation, budget, stale revision and scene56 fail atomically')

        c.call('step', ticks=60)
        pose = c.call('inspect_mesh_pose', instance=instance['id'])['pose']
        assert pose['positions'] != dense_result['geometry']['positions']
        capture('animated'); capture('repeat')
        assert (ROOT/'artifacts/smooth-animated.png').read_bytes() == (ROOT/'artifacts/smooth-repeat.png').read_bytes()
        c.call('save_replay', path='artifacts/smooth-branch.replay.json')
        assert c.call('run_replay', path='artifacts/smooth-branch.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/smooth-branch.replay.json', path='artifacts/smooth-replay.png', width=1000, height=1000)
        assert (ROOT/'artifacts/smooth-animated.png').read_bytes() == (ROOT/'artifacts/smooth-replay.png').read_bytes()
        c.call('load_scene', path='examples/native-smooth-branch.scene.json')
        assert asset() == dense_result
        assert c.call('inspect_plant', id=id)['plant']['source'] == source
        checks.append('Native rig deformation and saved source/assets reproduce; live/repeat/replay GPU pixels identical')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window,
                  'source_scene': '48aa4d4', 'branch': id, 'coarse_vertices': 258, 'coarse_triangles': 512,
                  'dense_vertices': 706, 'dense_triangles': 1408}
        (ROOT/'artifacts/smooth-plants-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
