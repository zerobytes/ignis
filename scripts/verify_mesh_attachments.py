#!/usr/bin/env python3
"""Native instance/bone hierarchy, independent playback and pixel replay."""
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
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-attachments-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        assert 'attachment' in json.dumps(c.request('tools/list')['result'])
        c.call('generate', recipe={'seed': 89, 'width': 12, 'depth': 12, 'obstacles': 0})
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'sheet', 'segments': 1, 'size': 1})
        bones = [{'name': 'base', 'parent': None, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]},
                 {'name': 'socket', 'parent': 0, 'translation': [0, 1, 0], 'rotation': [0, 0, 0, 1]}]
        c.call('edit_mesh', edit={'kind': 'set_rig', 'asset': 'sheet', 'rig': {'bones': bones, 'weights': [{'joints': [0, 0, 0, 0], 'weights': [1, 0, 0, 0]}]*4}})
        c.call('edit_mesh', edit={'kind': 'put_clip', 'asset': 'sheet', 'clip': {'id': 'turn', 'duration_ticks': 120, 'tracks': [{'bone': 1, 'keys': [
            {'tick': t, 'translation': [0, 1, 0], 'rotation': [0, math.sin(a/2), 0, math.cos(a/2)]}
            for t, a in [(0, 0), (60, math.pi/2), (120, 0)]]}]}})
        parent = {'id': 'parent', 'asset': 'sheet', 'position': [0, 2, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.2, 0.2, 0.8], 'playback': {'clip': 'turn', 'offset_ticks': 0, 'looping': True}}
        child = dict(parent, id='child', position=[1, 0, 0], color=[0.2, 0.8, 0.2], playback={'clip': 'turn', 'offset_ticks': 30, 'looping': True}, attachment={'parent': 'parent', 'bone': 'socket'})
        c.call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': i}} for i in [child, parent]])
        def pose(tick): return c.call('inspect_mesh_pose', instance='child', tick=tick)['pose']
        a, b = pose(0), pose(60)
        assert a['positions'] != b['positions'] and b['clip_tick'] == 90
        assert math.dist(a['world_transform'][3][:3], [1, 3, 0]) < 0.000001
        assert math.dist(b['world_transform'][3][:3], [0, 3, -1]) < 0.000001
        checks.append('Child-before-parent hierarchy follows named bone and own clip clock with analytic world transform')

        before = c.call('inspect')
        for bad in [dict(child, attachment={'parent': 'missing'}), dict(child, attachment={'parent': 'parent', 'bone': 'missing'}), dict(parent, attachment={'parent': 'child'}), dict(child, scale=100, attachment={'parent': 'parent'})]:
            edits = [{'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': bad}}]
            if bad['scale'] == 100:
                edits.append({'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': dict(parent, scale=2)}})
            c.call('edit_scene', edits=edits, expected_error=True)
            assert c.call('inspect') == before
        for edit in [{'kind': 'remove_instance', 'id': 'parent'}, {'kind': 'set_rig', 'asset': 'sheet', 'rig': None}]:
            c.call('edit_mesh', edit=edit, expected_error=True); assert c.call('inspect') == before
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': child}, expected_revision=before['revision']-1, expected_error=True)
        assert c.call('inspect') == before
        checks.append('Missing references, cycles, composed scale, dependent removals and stale revision reject atomically')

        detached = copy.deepcopy(child); detached.pop('attachment'); detached['playback'] = None
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': detached})
        assert pose(60)['positions'] == pose(0)['positions']
        c.call('undo'); assert pose(60) == b
        c.call('redo'); assert pose(60)['positions'] == pose(0)['positions']
        c.call('undo'); assert pose(60) == b
        c.call('save_scene', path='examples/mesh-attachments.scene.json')
        saved = json.loads((ROOT/'examples/mesh-attachments.scene.json').read_text()); assert saved['version'] == SCENE_VERSION
        saved['version'] = 57
        (ROOT/'examples/old-attachments.scene.json').write_text(json.dumps(saved))
        before = c.call('inspect')
        c.call('load_scene', path='examples/old-attachments.scene.json', expected_error=True); assert c.call('inspect') == before
        checks.append('Explicit local detach, undo/redo and version58 gate preserve authored hierarchy')

        c.call('set_camera', camera={'position': [7, 7, 10], 'target': [0, 2, 0], 'fov_degrees': 40})
        c.call('step', ticks=60)
        for name in ['live', 'repeat']:
            c.call('capture', path=f'artifacts/attachments-{name}.png', width=720, height=540)
        assert (ROOT/'artifacts/attachments-live.png').read_bytes() == (ROOT/'artifacts/attachments-repeat.png').read_bytes()
        c.call('save_replay', path='artifacts/attachments.replay.json')
        assert c.call('run_replay', path='artifacts/attachments.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/attachments.replay.json', path='artifacts/attachments-replay.png', width=720, height=540)
        assert (ROOT/'artifacts/attachments-live.png').read_bytes() == (ROOT/'artifacts/attachments-replay.png').read_bytes()
        c.call('load_scene', path='examples/mesh-attachments.scene.json'); assert pose(60) == b
        checks.append('Animated ancestor, save/load and world-space pose/replay share identical native GPU pixels')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window}
        (ROOT/'artifacts/mesh-attachments-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
