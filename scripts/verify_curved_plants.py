#!/usr/bin/env python3
"""Native editable trunk, curvature changes and canonical preservation via MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
import math
from pathlib import Path
from tree_plant_recipe import center, definition, radius
from verify_lab import Client, ROOT


def verify(binary, window=False):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-curved-plants-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result'])
        assert 'ellipse' in schema and 'offset' in schema
        c.call('generate', recipe={'seed': 20261026, 'width': 20, 'depth': 20, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        for entity in entities:
            entity['visible'] = False
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': entity} for entity in entities])
        c.call('set_camera', camera={'position': [4.1, 3.6, 8], 'target': [0, 2.4, 0], 'fov_degrees': 37})
        c.call('set_light', light={'direction': [-0.6, -0.8, -0.7], 'ambient': 0.35})
        d = definition()

        def put(drawing, op='edit_plant', **kwargs):
            return c.call(op, edit={'kind': 'put', 'definition': drawing, 'detail_policy': 'physical', 'animation_translation_policy': 'physical'}, **kwargs)

        def edit(kind, **kwargs):
            return c.call('edit_mesh', edit={'kind': kind, 'asset': d['id'], **kwargs})

        put(d)
        edit('set_shading', shading='smooth')
        edit('set_material', material={'pattern': {'kind': 'cellular', 'frequency': [24, 4, 24], 'low': [0.62, 0.60, 0.55], 'high': [1.10, 1.05, 0.97], 'bump': 0.0014, 'warp': 1.1, 'octaves': 3, 'seed': 137, 'contrast': 1.2}, 'transmission': 0})
        base = c.call('inspect_mesh', asset=d['id'])['asset']
        # An explicit rest-space material channel, independent of later curvature.
        edit('set_material_coordinates', coordinates=base['geometry']['positions'])
        t = 0.24
        p = center(t); p[2] += radius(t)
        edit('sculpt', stroke={'kind': 'grab', 'centers': [p], 'radius': 0.20, 'strength': 0.009, 'direction': [0, 0, -1]})
        asset = c.call('inspect_mesh', asset=d['id'])['asset']
        roots = [center(t) for t in [0.08, 0.44, 0.86]]
        bones = [{'name': f'trunk-{i}', 'parent': None if i == 0 else i-1,
                  'translation': p if i == 0 else [a-b for a, b in zip(p, roots[i-1])],
                  'rotation': [0, 0, 0, 1]} for i, p in enumerate(roots)]
        weights = []
        for x, y, z in asset['geometry']['positions']:
            t = max(0, min(2, (y/4.8+0.5)*2))
            low = min(1, int(t)); f = t-low
            weights.append({'joints': [low, low+1, 0, 0], 'weights': [1-f, f, 0, 0]})
        edit('set_rig', rig={'bones': bones, 'weights': weights})
        angle = math.radians(6)/2
        edit('put_clip', clip={'id': 'bend', 'duration_ticks': 120, 'tracks': [{'bone': 1, 'keys': [
            {'tick': tick, 'translation': bones[1]['translation'], 'rotation': [0, 0, math.sin(a), math.cos(a)]}
            for tick, a in [(0, 0), (60, angle), (120, 0)]]}]})
        instance = {'id': 'tree-trunk-instance', 'asset': d['id'], 'position': [0, 2.4, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.20, 0.15, 0.10], 'playback': {'clip': 'bend', 'offset_ticks': 0, 'looping': True}}
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        topology = c.call('inspect_mesh_topology', asset=d['id'])['topology']
        assert topology['boundary_edges'] == topology['non_manifold_edges'] == 0
        assert not topology['isolated_vertices']
        original = c.call('inspect_mesh', asset=d['id'])['asset']
        source = c.call('inspect_plant', id=d['id'])['plant']['source']
        assert c.call('inspect_plant', id=d['id'])['report']['max_detail_displacement'] > 0.002
        c.call('capture', path='artifacts/curved-trunk-before.png', width=1000, height=1000)
        c.call('save_scene', path='examples/native-tree-trunk.scene.json')
        (ROOT/'examples/native-tree-trunk.plant.json').write_text(json.dumps(d, indent=2)+'\n')
        checks.append('Shared-tree axis/sections generate a closed native elliptical trunk with sculpt, bark coordinates and rig')

        changed = copy.deepcopy(d)
        for station in changed['sections']['stations']:
            t = station['at']
            station['offset'][0] += 0.7*math.sin(t*math.pi)*t
            station['offset'][1] -= 0.5*math.sin(t*math.pi)
            station['depth'] *= 1-0.18*math.sin(t*math.pi)
        before = c.call('inspect')
        put(changed, 'preview_plant')
        assert c.call('inspect') == before
        put(changed)
        result = c.call('inspect_mesh', asset=d['id'])['asset']
        assert c.call('inspect_plant', id=d['id'])['plant']['source'] == source
        assert result['rig']['weights'] == original['rig']['weights']
        assert result['geometry']['material_coordinates'] == original['geometry']['material_coordinates']
        assert result['shading'] == original['shading'] and result['material'] == original['material']
        assert result['rig']['bones'] != original['rig']['bones']
        assert result['clips'][0]['tracks'][0]['keys'][1]['rotation'] == original['clips'][0]['tracks'][0]['keys'][1]['rotation']
        assert result['clips'][0]['tracks'][0]['keys'][0]['translation'] == result['rig']['bones'][1]['translation']
        c.call('capture', path='artifacts/curved-trunk-after.png', width=1000, height=1000)
        for _ in range(3):
            put(d); assert c.call('inspect_mesh', asset=d['id'])['asset'] == original
            put(changed); assert c.call('inspect_mesh', asset=d['id'])['asset'] == result
        c.call('undo'); assert c.call('inspect_mesh', asset=d['id'])['asset'] == original
        c.call('redo'); assert c.call('inspect_mesh', asset=d['id'])['asset'] == result
        checks.append('Curvature and oval taper change with pure preview, stable canonical sculpt/material/weights, adapted rig and drift-free A-B-A/undo')

        before = c.call('inspect')
        for change in ['identity', 'offset', 'shape', 'budget']:
            bad = copy.deepcopy(changed)
            if change == 'identity': bad['sections']['stations'][3]['id'] = 'replacement'
            if change == 'offset': bad['sections']['stations'][3]['offset'][0] = 101
            if change == 'shape': bad['sections']['cross_section'] = 'diamond'
            if change == 'budget': bad['segments'] = [63, 63, 63]
            put(bad, expected_error=True)
            assert c.call('inspect') == before
        put(changed, expected_error=True, expected_revision=before['revision']-1)
        assert c.call('inspect') == before
        # Nested angular samples retain every canonical relief sample exactly.
        fine = copy.deepcopy(changed); fine['segments'] = [16, 3, 16]
        put(fine)
        assert c.call('inspect_plant', id=d['id'])['plant']['source'] == source
        put(changed)
        checks.append('IDs/shape/offset/budget/stale revision fail atomically; bounded retessellation keeps canonical source')

        # A detail authored only in a fine sample cannot silently disappear.
        put(fine)
        samples = c.call('inspect_mesh', asset=d['id'])['asset']['geometry']['positions']
        coarse_samples = result['geometry']['positions']
        fine_point = max((p for p in samples if abs(p[1]) < 0.5),
                         key=lambda p: min(math.dist(p, q) for q in coarse_samples))
        assert min(math.dist(fine_point, q) for q in coarse_samples) > 0.005
        edit('sculpt', stroke={'kind': 'grab', 'centers': [fine_point], 'radius': 0.001, 'strength': 0.005, 'direction': [1, 0, 0]})
        detailed = c.call('inspect')
        lost = copy.deepcopy(changed); lost['sampling_tolerance'] = 0.000001
        rejected = put(lost, expected_error=True)
        assert 'resolution insufficient' in str(rejected).lower(), rejected
        assert c.call('inspect') == detailed
        c.call('undo')  # remove the fine-only stroke
        put(changed)
        checks.append('Coarsening rejects loss of an isolated fine-only sculpt sample')

        c.call('save_scene', path='examples/native-tree-trunk-redrawn.scene.json')
        c.call('step', ticks=60)
        pose = c.call('inspect_mesh_pose', instance=instance['id'])['pose']
        assert pose['positions'] != result['geometry']['positions']
        c.call('capture', path='artifacts/curved-trunk-animated.png', width=1000, height=1000)
        c.call('capture', path='artifacts/curved-trunk-repeat.png', width=1000, height=1000)
        assert (ROOT/'artifacts/curved-trunk-animated.png').read_bytes() == (ROOT/'artifacts/curved-trunk-repeat.png').read_bytes()
        c.call('save_replay', path='artifacts/curved-trunk.replay.json')
        assert c.call('run_replay', path='artifacts/curved-trunk.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/curved-trunk.replay.json', path='artifacts/curved-trunk-replay.png', width=1000, height=1000)
        assert (ROOT/'artifacts/curved-trunk-animated.png').read_bytes() == (ROOT/'artifacts/curved-trunk-replay.png').read_bytes()
        c.call('load_scene', path='examples/native-tree-trunk-redrawn.scene.json')
        assert c.call('inspect_mesh', asset=d['id'])['asset'] == result
        saved = json.loads((ROOT/'examples/native-tree-trunk-redrawn.scene.json').read_text())
        assert saved['version'] == SCENE_VERSION
        saved['version'] = 55
        (ROOT/'examples/curved-old-version.scene.json').write_text(json.dumps(saved))
        before = c.call('inspect')
        c.call('load_scene', path='examples/curved-old-version.scene.json', expected_error=True)
        assert c.call('inspect') == before
        checks.append('Rig deforms natively; repeat/live/replay GPU pixels match; source persists and scene55 refuses new section data')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window,
                  'vertices': len(result['geometry']['positions']), 'triangles': len(result['geometry']['triangles']),
                  'source_seed': 20261026, 'scope': 'native editable trunk study; central connected fork unchanged'}
        (ROOT/'artifacts/curved-plants-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
