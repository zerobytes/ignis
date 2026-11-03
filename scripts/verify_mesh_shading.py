#!/usr/bin/env python3
"""Native shading mode, posed normals and canonical metadata through MCP."""
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
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'ignis-shading-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        assert 'set_shading' in json.dumps(c.request('tools/list'))
        c.call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        c.call('set_camera', camera={'position': [8, 8, 10], 'target': [0, 2, 0], 'fov_degrees': 35})
        c.call('set_light', light={'direction': [-0.6, -1, 0.3], 'ambient': 0.2})
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'hump', 'segments': 8, 'size': 3})
        c.call('edit_mesh', edit={'kind': 'sculpt', 'asset': 'hump', 'stroke': {
            'kind': 'draw', 'centers': [[0, 0, 0]], 'radius': 1.8, 'strength': 0.85, 'direction': [0, 1, 0]}})
        source = c.call('inspect_mesh', asset='hump')['asset']
        assert source['shading'] == 'flat'
        bones = [{'name': 'root', 'parent': None, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]},
                 {'name': 'tip', 'parent': 0, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]}]
        weights = [{'joints': [0, 1, 0, 0], 'weights': [1-(p[2]+1.5)/3, (p[2]+1.5)/3, 0, 0]}
                   for p in source['geometry']['positions']]
        c.call('edit_mesh', edit={'kind': 'set_rig', 'asset': 'hump', 'rig': {'bones': bones, 'weights': weights}})
        c.call('edit_mesh', edit={'kind': 'put_clip', 'asset': 'hump', 'clip': {'id': 'bend', 'duration_ticks': 120,
               'tracks': [{'bone': 1, 'keys': [{'tick': tick, 'translation': [0, 0, 0],
                    'rotation': [math.sin(angle/2), 0, 0, math.cos(angle/2)]}
                   for tick, angle in [(0, 0), (60, 0.7), (120, 0)]]}]}})
        for name, x, playback in [('static', -2, None), ('moving', 2, {'clip': 'bend', 'offset_ticks': 0, 'looping': True})]:
            c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {'id': name, 'asset': 'hump',
                   'position': [x, 2, 0], 'rotation': [0, 0, 0, 1], 'scale': 1,
                   'color': [0.24, 0.46, 0.11], 'playback': playback}})
        flat = c.call('inspect_mesh', asset='hump')['asset']
        poses = c.call('inspect_mesh_pose', instance='moving')['pose']
        c.call('capture', path='artifacts/shading-flat.png', width=960, height=720)
        c.call('edit_mesh', edit={'kind': 'set_shading', 'asset': 'hump', 'shading': 'smooth'})
        smooth = c.call('inspect_mesh', asset='hump')['asset']
        assert smooth == {**flat, 'shading': 'smooth'}
        assert c.call('inspect_mesh_pose', instance='moving')['pose'] == poses
        c.call('capture', path='artifacts/shading-smooth.png', width=960, height=720)
        assert (ROOT/'artifacts/shading-flat.png').read_bytes() != (ROOT/'artifacts/shading-smooth.png').read_bytes()
        checks.append('MCP mode changes native pixels while geometry, rig, clips and poses remain identical')
        snapshot = c.call('inspect')
        for name, mode in [('missing', 'smooth'), ('hump', 'invalid')]:
            c.call('edit_mesh', edit={'kind': 'set_shading', 'asset': name, 'shading': mode}, expected_error=True)
            assert c.call('inspect') == snapshot
        c.call('undo')
        c.call('capture', path='artifacts/shading-undo.png', width=960, height=720)
        assert (ROOT/'artifacts/shading-undo.png').read_bytes() == (ROOT/'artifacts/shading-flat.png').read_bytes()
        c.call('redo')
        c.call('capture', path='artifacts/shading-redo.png', width=960, height=720)
        assert (ROOT/'artifacts/shading-redo.png').read_bytes() == (ROOT/'artifacts/shading-smooth.png').read_bytes()
        c.call('save_scene', path='examples/native-shading.scene.json')
        saved = c.call('inspect')['scene']
        assert saved['version'] == SCENE_VERSION
        forged = copy.deepcopy(saved)
        forged['version'] = 51
        (ROOT/'artifacts/shading-forged.scene.json').write_text(json.dumps(forged))
        snapshot = c.call('inspect')
        c.call('load_scene', path='artifacts/shading-forged.scene.json', expected_error=True)
        assert c.call('inspect') == snapshot
        checks.append('Invalid mode/asset/legacy version fail atomically; undo/redo restore exact pixels at the same tick')
        c.call('step', ticks=45)
        c.call('capture', path='artifacts/shading-live.png', width=960, height=720)
        c.call('capture', path='artifacts/shading-repeat.png', width=960, height=720)
        assert (ROOT/'artifacts/shading-live.png').read_bytes() == (ROOT/'artifacts/shading-repeat.png').read_bytes()
        assert (ROOT/'artifacts/shading-live.png').read_bytes() != (ROOT/'artifacts/shading-smooth.png').read_bytes()
        c.call('save_replay', path='artifacts/shading.replay.json')
        assert c.call('run_replay', path='artifacts/shading.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/shading.replay.json', path='artifacts/shading-replay.png', width=960, height=720)
        c.call('capture_replay', replay_path='artifacts/shading.replay.json', path='artifacts/shading-prefix.png', tick=0, width=960, height=720)
        assert (ROOT/'artifacts/shading-replay.png').read_bytes() == (ROOT/'artifacts/shading-live.png').read_bytes()
        assert (ROOT/'artifacts/shading-prefix.png').read_bytes() == (ROOT/'artifacts/shading-smooth.png').read_bytes()
        c.call('load_scene', path='examples/native-shading.scene.json')
        assert c.call('inspect')['scene'] == saved
        c.call('step', ticks=10)
        c.call('reset')
        c.call('capture', path='artifacts/shading-reset.png', width=960, height=720)
        assert (ROOT/'artifacts/shading-reset.png').read_bytes() == (ROOT/'artifacts/shading-smooth.png').read_bytes()
        checks.append('Animated smooth surface, repeated captures, reset/load and replay/prefix preserve deterministic pixels')
        definition = {'id': 'plant', 'profile': {'id': 'profile', 'width': 1, 'height': 3}, 'depth': 0.3,
                      'segments': [2, 3, 1], 'subdivision_levels': 0, 'sampling_tolerance': 1/1024}
        def put(d):
            c.call('edit_plant', edit={'kind': 'put', 'definition': d,
                   'detail_policy': 'physical', 'animation_translation_policy': 'physical'})
        put(definition)
        grown = copy.deepcopy(definition)
        grown['profile']['height'] = 5
        put(grown)
        before = c.call('inspect_plant', id='plant')['plant']
        c.call('edit_mesh', edit={'kind': 'set_shading', 'asset': 'plant', 'shading': 'smooth'})
        after = c.call('inspect_plant', id='plant')['plant']
        expected = copy.deepcopy(before)
        expected['source']['shading'] = 'smooth'
        assert after == expected
        put(definition)
        assert c.call('inspect_mesh', asset='plant')['asset']['shading'] == 'smooth'
        assert c.call('inspect_plant', id='plant')['plant']['reference'] == before['reference']
        checks.append('Linked plant appearance metadata survives regeneration without rebasing canonical geometry/reference')
        report = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/shading-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary, args.window)
