#!/usr/bin/env python3
"""Parametric profiles regenerate sculpted and animated meshes through MCP."""
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
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-plants-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        assert all(op in schema for op in ['edit_plant', 'inspect_plant', 'preview_plant'])
        c.call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        c.call('set_camera', camera={'position': [9, 6, 12], 'target': [0, 3.5, 0], 'fov_degrees': 42})
        definition = {'id': 'limb', 'profile': {'id': 'outline', 'width': 2, 'height': 4}, 'depth': 0.5, 'segments': [4, 12, 2], 'sampling_tolerance': 0.0009765625}

        def put(d, op='edit_plant', **extra):
            return c.call(op, edit={'kind': 'put', 'definition': d, 'detail_policy': 'physical', 'animation_translation_policy': 'physical'}, **extra)

        initial = c.call('inspect')
        put(definition, 'preview_plant')
        assert c.call('inspect') == initial
        put(definition)
        assert c.call('inspect_plants')['plants'] == [definition]
        assert c.call('inspect')['runtime_positions'] == initial['runtime_positions']
        c.call('edit_mesh', edit={'kind': 'sculpt', 'asset': 'limb', 'stroke': {'kind': 'grab', 'centers': [[0, 0, 0.25]], 'radius': 0.8, 'strength': 0.14, 'direction': [0, 0, 1]}})
        asset = c.call('inspect_mesh', asset='limb')['asset']
        weights = []
        for x, y, z in asset['geometry']['positions']:
            t = min(2, max(0, (y+2)/2))
            low = min(1, int(t))
            f = t-low
            weights.append({'joints': [low, low+1, 0, 0], 'weights': [1-f, f, 0, 0]})
        bones = [{'name': f'bone{i}', 'parent': i-1 if i else None, 'translation': [0, 2 if i else -2, 0], 'rotation': [0, 0, 0, 1]} for i in range(3)]
        c.call('edit_mesh', edit={'kind': 'set_rig', 'asset': 'limb', 'rig': {'bones': bones, 'weights': weights}})
        angle = math.radians(50)/2
        clip = {'id': 'bend', 'duration_ticks': 120, 'tracks': [{'bone': 1, 'keys': [{'tick': t, 'translation': [0, 2, 0], 'rotation': [0, 0, math.sin(a), math.cos(a)]} for t, a in [(0, 0), (60, angle), (120, 0)]]}]}
        c.call('edit_mesh', edit={'kind': 'put_clip', 'asset': 'limb', 'clip': clip})
        instance = {'id': 'sample', 'asset': 'limb', 'position': [0, 3.2, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.9, 0.3, 0.07], 'playback': {'clip': 'bend', 'offset_ticks': 0, 'looping': True}}
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        original = c.call('inspect_mesh', asset='limb')['asset']
        source = c.call('inspect_plant', id='limb')['plant']['source']
        c.call('capture', path='artifacts/plants-original.png', width=960, height=720)
        checks.append('Structured profile/extrusion, read-only preview, native sculpt/rig/clip and independent gameplay use shared MCP')

        changed = copy.deepcopy(definition)
        changed['profile'].update(width=3, height=6)
        changed['depth'] = 1
        put(changed)
        evaluated = c.call('inspect_mesh', asset='limb')['asset']
        assert c.call('inspect_plant', id='limb')['plant']['source'] == source
        assert evaluated['rig']['bones'][1]['translation'] == [0, 3, 0]
        assert evaluated['clips'][0]['tracks'][0]['keys'][0]['translation'] == [0, 3, 0]
        assert evaluated['clips'][0]['tracks'][0]['keys'][1]['rotation'] == original['clips'][0]['tracks'][0]['keys'][1]['rotation']
        report = c.call('inspect_plant', id='limb')['report']
        assert report['max_mapping_error'] < 1e-5 and report['max_detail_displacement'] > 0.1
        c.call('capture', path='artifacts/plants-resized.png', width=960, height=720)
        assert (ROOT/'artifacts/plants-original.png').read_bytes() != (ROOT/'artifacts/plants-resized.png').read_bytes()
        put(definition)
        assert c.call('inspect_mesh', asset='limb')['asset'] == original
        put(changed)
        put(definition)
        assert c.call('inspect_mesh', asset='limb')['asset'] == original
        checks.append('Nonuniform dimensional regeneration preserves physical detail, canonical source, bone landmarks and bind-relative clip motion without drift')

        changed['segments'] = [8, 24, 4]
        put(changed)
        result = c.call('inspect_plant', id='limb')
        assert result['report']['target_vertices'] > result['report']['source_vertices']
        assert c.call('inspect_plant', id='limb')['plant']['source'] == source
        snapshot = c.call('inspect')
        bad = copy.deepcopy(changed)
        bad['profile']['id'] = 'missing_outline'
        put(bad, expected_error=True)
        put({**changed, 'segments': [63, 63, 63]}, expected_error=True)
        put({**changed, 'segments': [1, 1, 1]}, expected_error=True)
        c.call('edit_mesh', edit={'kind': 'remove_asset', 'id': 'limb'}, expected_error=True)
        put(changed, expected_revision=snapshot['revision']-1, expected_error=True)
        assert c.call('inspect') == snapshot
        c.call('undo')
        assert c.call('inspect_mesh', asset='limb')['asset'] == original
        c.call('redo')
        assert c.call('inspect')['scene'] == snapshot['scene']
        checks.append('Retessellation reuses source correspondences; missing regions, budgets and stale revisions reject atomically; undo/redo restore source and result')

        c.call('save_scene', path='examples/native-plants.scene.json')
        c.call('capture', path='artifacts/plants-start.png', width=960, height=720)
        c.call('step', ticks=45)
        live = c.call('inspect_mesh_pose', instance='sample')['pose']
        assert live['clip_tick'] == 45
        c.call('capture', path='artifacts/plants-live.png', width=960, height=720)
        c.call('capture', path='artifacts/plants-repeat.png', width=960, height=720)
        assert (ROOT/'artifacts/plants-live.png').read_bytes() == (ROOT/'artifacts/plants-repeat.png').read_bytes()
        c.call('save_replay', path='artifacts/plants.replay.json')
        assert c.call('run_replay', path='artifacts/plants.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/plants.replay.json', path='artifacts/plants-replay.png', width=960, height=720)
        c.call('capture_replay', replay_path='artifacts/plants.replay.json', path='artifacts/plants-prefix.png', tick=0, width=960, height=720)
        assert (ROOT/'artifacts/plants-live.png').read_bytes() == (ROOT/'artifacts/plants-replay.png').read_bytes()
        assert (ROOT/'artifacts/plants-start.png').read_bytes() == (ROOT/'artifacts/plants-prefix.png').read_bytes()
        c.call('load_scene', path='examples/native-plants.scene.json')
        assert c.call('inspect')['scene'] == snapshot['scene']
        c.call('reset')
        c.call('capture', path='artifacts/plants-reset.png', width=960, height=720)
        assert (ROOT/'artifacts/plants-start.png').read_bytes() == (ROOT/'artifacts/plants-reset.png').read_bytes()
        c.call('capture', path='artifacts/plants-viewport.png', width=640, height=480)
        c.call('edit_scene', edits=[{'op': 'plant', 'edit': {'kind': 'detach', 'id': 'limb'}}, {'op': 'mesh', 'edit': {'kind': 'remove_instance', 'id': 'sample'}}, {'op': 'mesh', 'edit': {'kind': 'remove_asset', 'id': 'limb'}}])
        assert not c.call('inspect_plants')['plants']
        c.call('undo')
        assert c.call('inspect')['scene'] == snapshot['scene']
        checks.append('Save/load/reset, explicit detach/removal, animation, GPU repeat/replay/prefix pixels and viewport remain coherent')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window}
        (ROOT/'artifacts/plants-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
