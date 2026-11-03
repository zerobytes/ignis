#!/usr/bin/env python3
"""Explicit directional shadow volumes through native MCP rendering/replay."""
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
    def capture(name, width=960, height=720):
        c.call('capture', path=f'artifacts/shadow-{name}.png', width=width, height=height)
        return (ROOT/f'artifacts/shadow-{name}.png').read_bytes()
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'ignis-shadow-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe={'seed': 7, 'width': 16, 'depth': 14, 'obstacles': 0})
        scene = c.call('inspect')['scene']
        edits = []
        for e in scene['entities']:
            e['visible'] = e['id'] == 'floor'
            edits.append({'op': 'upsert', 'entity': e})
        box = copy.deepcopy(next(e for e in scene['entities'] if e['id'] == 'wall_north'))
        box.update(id='shadow-box', name='Shadow caster', visible=True, color=[0.8, 0.3, 0.05])
        box['transform'] = {'position': [-1.4, 1.5, 0], 'size': [1.3, 3, 1.3]}
        edits.append({'op': 'upsert', 'entity': box})
        c.call('edit_scene', edits=edits)
        c.call('set_camera', camera={'position': [7, 7, 9], 'target': [0, 0.8, 0], 'fov_degrees': 43})
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'sail', 'size': 2.5, 'segments': 4})
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {'id': 'mesh-caster', 'asset': 'sail',
               'position': [1.5, 2, 0], 'rotation': [math.sin(0.5), 0, 0, math.cos(0.5)],
               'scale': 1, 'color': [0.12, 0.5, 0.08], 'playback': None}})
        light = {'direction': [0.6, -1, 0.3], 'ambient': 0.18}
        c.call('set_light', light=light)
        no_shadow = capture('off')
        shadow = {'center': [0, 1, 0], 'extent': 5, 'resolution': 1024, 'bias': 0.0005}
        light['shadow'] = shadow
        poses = c.call('inspect_mesh_pose', instance='mesh-caster')['pose']
        c.call('set_light', light=light)
        on = capture('on')
        assert on != no_shadow and on == capture('repeat')
        assert c.call('inspect_mesh_pose', instance='mesh-caster')['pose'] == poses
        c.call('undo'); assert capture('undo') == no_shadow
        c.call('redo'); assert capture('redo') == on
        checks.append('Native box/mesh shadow changes pixels, repeated capture deterministic, undo/redo exact and poses unchanged')
        snapshot = c.call('inspect')
        for field, value in [('resolution', 4096), ('extent', 0), ('bias', -0.1)]:
            bad = copy.deepcopy(light)
            bad['shadow'][field] = value
            c.call('set_light', light=bad, expected_error=True)
            assert c.call('inspect') == snapshot
        c.call('save_scene', path='examples/native-shadows.scene.json')
        saved = c.call('inspect')['scene']
        assert saved['version'] == SCENE_VERSION
        forged = copy.deepcopy(saved)
        forged['version'] = 52
        (ROOT/'artifacts/shadow-forged.scene.json').write_text(json.dumps(forged))
        c.call('load_scene', path='artifacts/shadow-forged.scene.json', expected_error=True)
        assert c.call('inspect') == snapshot
        checks.append('Bounds/resolution/bias and legacy capability smuggling fail atomically')
        # Both opaque geometry paths cast; removing either invalidates depth.
        c.call('edit_mesh', edit={'kind': 'remove_instance', 'id': 'mesh-caster'})
        assert capture('mesh-removed') != on
        c.call('undo'); assert capture('mesh-restored') == on
        c.call('remove', id='shadow-box')
        assert capture('box-removed') != on
        c.call('undo'); assert capture('box-restored') == on
        capture('viewport', 640, 480)
        assert capture('viewport-restored') == on
        for size in [512, 2048, 1024]:
            changed = copy.deepcopy(light)
            changed['shadow']['resolution'] = size
            c.call('set_light', light=changed)
            capture(f'resolution-{size}')
        assert capture('resolution-restored') == on
        changed = copy.deepcopy(light)
        changed['direction'] = [-0.6, -1, -0.3]
        c.call('set_light', light=changed)
        assert capture('light-moved') != on
        c.call('load_scene', path='examples/native-shadows.scene.json')
        assert capture('loaded') == on
        checks.append('Removing/restoring box and mesh casters, viewport/map resize, light motion and load invalidate/reuse shadow resources correctly')
        c.call('set_input', input={'x': 1, 'z': 0})
        c.call('step', ticks=20)
        live = capture('live')
        c.call('save_replay', path='artifacts/shadow.replay.json')
        assert c.call('run_replay', path='artifacts/shadow.replay.json')['matched']
        c.call('capture_replay', replay_path='artifacts/shadow.replay.json', path='artifacts/shadow-replay.png', width=960, height=720)
        c.call('capture_replay', replay_path='artifacts/shadow.replay.json', path='artifacts/shadow-prefix.png', tick=0, width=960, height=720)
        assert (ROOT/'artifacts/shadow-replay.png').read_bytes() == live
        assert (ROOT/'artifacts/shadow-prefix.png').read_bytes() == on
        c.call('reset'); assert capture('reset') == on
        report = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        checks.append('Shadowed execution/reset and complete/prefix replay preserve native pixels')
        (ROOT/'artifacts/shadow-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary, args.window)
