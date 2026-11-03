#!/usr/bin/env python3
"""Revision-scoped mesh components and conforming linear subdivision through MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-mesh-topology-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        assert all(op in schema for op in ['inspect_mesh_topology', 'select_mesh_components', 'inspect_mesh_selection', 'subdivide'])
        c.call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        c.call('set_camera', camera={'position': [9, 8, 13], 'target': [0.8, 2.4, 0], 'fov_degrees': 42})
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'patch', 'segments': 1, 'size': 4})

        def topology(asset):
            return c.call('inspect_mesh_topology', asset=asset)

        def subdivide(asset, components=None, **extra):
            return c.call('edit_mesh', edit={'kind': 'subdivide', 'asset': asset, 'reference': topology(asset)['reference'], 'components': components}, **extra)

        original = c.call('inspect_mesh', asset='patch')['asset']
        t = topology('patch')
        assert len(t['topology']['edges']) == 5 and t['topology']['boundary_edges'] == 4
        assert t['topology']['non_manifold_edges'] == 0
        before = c.call('inspect')
        for components in [{'kind': 'vertices', 'indices': [3, 0]}, {'kind': 'edges', 'vertices': [[2, 1]]}, {'kind': 'faces', 'indices': [0]}]:
            selected = {'asset': 'patch', 'reference': t['reference'], 'components': components}
            result = c.call('select_mesh_components', selection=selected)
            assert c.call('inspect_mesh_selection') == result
        after = c.call('inspect')
        for value in [before, after]:
            value.pop('mesh_selection')
            value.pop('view_revision')
        assert before == after
        snapshot = c.call('inspect')
        c.call('select_mesh_components', selection={**selected, 'components': {'kind': 'faces', 'indices': [999]}}, expected_error=True)
        assert c.call('inspect') == snapshot
        c.call('edit_mesh', edit={'kind': 'subdivide', **selected})
        partial = c.call('inspect_mesh', asset='patch')['asset']
        assert len(partial['geometry']['positions']) == 7 and len(partial['geometry']['triangles']) == 6
        assert partial['geometry']['positions'][:4] == original['geometry']['positions']
        assert c.call('inspect_mesh_selection')['selection'] is None
        snapshot = c.call('inspect')
        c.call('select_mesh_components', selection=selected, expected_error=True)
        c.call('edit_mesh', edit={'kind': 'subdivide', **selected}, expected_error=True)
        assert c.call('inspect') == snapshot
        c.call('undo')
        assert c.call('inspect_mesh', asset='patch')['asset'] == original
        c.call('redo')
        assert c.call('inspect_mesh', asset='patch')['asset'] == partial
        checks.append('Typed vertices/edges/faces, read-only incidence, session selection, conforming local splits and stale-reference atomic rejection')

        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'patch', 'segments': 1, 'size': 4})
        subdivide('patch')
        subdivide('patch')
        refined = c.call('inspect_mesh', asset='patch')['asset']
        assert len(refined['geometry']['positions']) == 25 and len(refined['geometry']['triangles']) == 32
        center = next(i for i, p in enumerate(refined['geometry']['positions']) if p == [0, 0, 0])
        assert center >= 4
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {'id': 'patch-instance', 'asset': 'patch', 'position': [-2.5, 2, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.85, 0.25, 0.08], 'playback': None}})
        c.call('capture', path='artifacts/mesh-topology-flat.png', width=960, height=720)
        c.call('edit_mesh', edit={'kind': 'sculpt', 'asset': 'patch', 'stroke': {'kind': 'grab', 'centers': [[0, 0, 0]], 'radius': 0.6, 'strength': 0.8, 'direction': [0, 1, 0]}})
        assert math.isclose(c.call('inspect_mesh', asset='patch')['asset']['geometry']['positions'][center][1], 0.8, abs_tol=1e-6)
        c.call('capture', path='artifacts/mesh-topology-detail.png', width=960, height=720)
        assert (ROOT/'artifacts/mesh-topology-flat.png').read_bytes() != (ROOT/'artifacts/mesh-topology-detail.png').read_bytes()
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'over-budget', 'segments': 63, 'size': 2})
        subdivide('over-budget')
        snapshot = c.call('inspect')
        subdivide('over-budget', expected_error=True)
        assert c.call('inspect') == snapshot
        c.call('edit_mesh', edit={'kind': 'remove_asset', 'id': 'over-budget'})
        checks.append('Uniform shared midpoints preserve source vertices, unlock new sculpt samples, preflight budgets and invalidate render geometry')

        definition = {'id': 'limb', 'profile': {'id': 'outline', 'width': 2, 'height': 4}, 'depth': 0.5, 'segments': [1, 1, 1], 'sampling_tolerance': 0.0009765625}

        def put(d, **extra):
            return c.call('edit_plant', edit={'kind': 'put', 'definition': d, 'detail_policy': 'physical', 'animation_translation_policy': 'physical'}, **extra)

        put(definition)
        source = c.call('inspect_plant', id='limb')['plant']['source']
        subdivide('limb')
        subdivide('limb')
        plant = c.call('inspect_plant', id='limb')['plant']
        assert plant['definition']['subdivision_levels'] == 2 and plant['source'] == source
        assert len(c.call('inspect_mesh', asset='limb')['asset']['geometry']['positions']) == 150
        one = {**definition, 'subdivision_levels': 1}
        put(one)
        c.call('edit_mesh', edit={'kind': 'sculpt', 'asset': 'limb', 'stroke': {'kind': 'grab', 'centers': [[0, 0, 0.25]], 'radius': 0.01, 'strength': 0.05, 'direction': [0, 0, 1]}})
        source = c.call('inspect_plant', id='limb')['plant']['source']
        snapshot = c.call('inspect')
        put(definition, expected_error=True)
        subdivide('limb', {'kind': 'faces', 'indices': [0]}, expected_error=True)
        assert c.call('inspect') == snapshot
        dense = copy.deepcopy(one)
        dense.update(segments=[2, 2, 2], subdivision_levels=2, depth=1)
        dense['profile'].update(width=3, height=6)
        put(dense)
        assert c.call('inspect_plant', id='limb')['plant']['source'] == source
        put(one)
        assert c.call('inspect')['scene'] == snapshot['scene']
        checks.append('Linked uniform subdivision retains source, tracks all refined samples and rejects midpoint-detail loss or unrepresentable local edits')

        bones = [{'name': 'root', 'parent': None, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]}]
        clip = {'id': 'turn', 'duration_ticks': 120, 'tracks': [{'bone': 0, 'keys': [{'tick': t, 'translation': [0, 0, 0], 'rotation': [0, math.sin(a/2), 0, math.cos(a/2)]} for t, a in [(0, 0), (60, 0.6), (120, 0)]]}]}
        for asset, position, color in [('patch', [-2.5, 2, 0], [0.85, 0.25, 0.08]), ('limb', [2.5, 2.5, 0], [0.08, 0.55, 0.85])]:
            count = len(c.call('inspect_mesh', asset=asset)['asset']['geometry']['positions'])
            c.call('edit_mesh', edit={'kind': 'set_rig', 'asset': asset, 'rig': {'bones': bones, 'weights': [{'joints': [0, 0, 0, 0], 'weights': [1, 0, 0, 0]} for _ in range(count)]}})
            c.call('edit_mesh', edit={'kind': 'put_clip', 'asset': asset, 'clip': clip})
            c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {'id': f'{asset}-instance', 'asset': asset, 'position': position, 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': color, 'playback': {'clip': 'turn', 'offset_ticks': 0, 'looping': True}}})
        source = c.call('inspect_plant', id='limb')['plant']['source']
        subdivide('limb')
        assert c.call('inspect_plant', id='limb')['plant']['source'] == source
        assert len(c.call('inspect_mesh', asset='limb')['asset']['rig']['weights']) == 150
        c.call('save_scene', path='examples/native-mesh-topology.scene.json')
        authored = c.call('inspect')['scene']
        assert authored['version'] == SCENE_VERSION
        c.call('capture', path='artifacts/mesh-topology-start.png', width=960, height=720)
        c.call('step', ticks=30)
        live = c.call('inspect_mesh_pose', instance='limb-instance')['pose']
        c.call('capture', path='artifacts/mesh-topology-live.png', width=960, height=720)
        c.call('capture', path='artifacts/mesh-topology-repeat.png', width=960, height=720)
        assert (ROOT/'artifacts/mesh-topology-live.png').read_bytes() == (ROOT/'artifacts/mesh-topology-repeat.png').read_bytes()
        c.call('save_replay', path='artifacts/mesh-topology.replay.json')
        assert c.call('run_replay', path='artifacts/mesh-topology.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/mesh-topology.replay.json', path='artifacts/mesh-topology-replay.png', width=960, height=720)
        c.call('capture_replay', replay_path='artifacts/mesh-topology.replay.json', path='artifacts/mesh-topology-prefix.png', tick=0, width=960, height=720)
        assert (ROOT/'artifacts/mesh-topology-live.png').read_bytes() == (ROOT/'artifacts/mesh-topology-replay.png').read_bytes()
        assert (ROOT/'artifacts/mesh-topology-start.png').read_bytes() == (ROOT/'artifacts/mesh-topology-prefix.png').read_bytes()
        selected = {'asset': 'limb', 'reference': topology('limb')['reference'], 'components': {'kind': 'vertices', 'indices': [0]}}
        c.call('select_mesh_components', selection=selected)
        assert c.call('inspect_mesh_pose', instance='limb-instance')['pose'] == live
        c.call('load_scene', path='examples/native-mesh-topology.scene.json')
        assert c.call('inspect')['scene'] == authored
        assert c.call('inspect_mesh_selection')['selection'] is None
        c.call('select_mesh_components', selection=selected, expected_error=True)
        c.call('reset')
        c.call('capture', path='artifacts/mesh-topology-reset.png', width=960, height=720)
        assert (ROOT/'artifacts/mesh-topology-start.png').read_bytes() == (ROOT/'artifacts/mesh-topology-reset.png').read_bytes()
        c.call('capture', path='artifacts/mesh-topology-viewport.png', width=640, height=480)
        checks.append('Refined source rebase with rig/clips, selection-independent animation, Scene50/save/load/reset and Replay54/live/prefix GPU equality')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window}
        (ROOT/'artifacts/mesh-topology-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
