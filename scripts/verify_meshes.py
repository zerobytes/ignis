#!/usr/bin/env python3
"""Native creation/sculpt/rig/animation through the real shared MCP dispatcher."""
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
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-mesh-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = c.request('tools/list')['result']['tools'][0]['inputSchema']
        assert 'edit_mesh' in json.dumps(schema) and 'inspect_mesh_pose' in json.dumps(schema)
        c.call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        c.call('set_camera', camera={'position': [9, 8, 12], 'target': [0, 2, 0], 'fov_degrees': 42})
        c.call('set_light', light={'direction': [0.3, -1, 0.7], 'ambient': 0.25})
        # An ordinary player/physics scene continues alongside the independent visual assets.
        runtime = c.call('inspect')['runtime_positions']
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'ribbon', 'segments': 16, 'size': 4})
        before = c.call('inspect_mesh', asset='ribbon')['asset']
        c.call('edit_mesh', edit={'kind': 'sculpt', 'asset': 'ribbon', 'stroke': {
            'kind': 'draw', 'centers': [[0, 0, 0], [0.6, 0, 0.5]], 'radius': 1.4,
            'strength': 0.45, 'direction': [0, 0, 0]}})
        sculpted = c.call('inspect_mesh', asset='ribbon')['asset']
        assert before['geometry']['positions'] != sculpted['geometry']['positions']
        assert before['geometry']['triangles'] == sculpted['geometry']['triangles']
        c.call('undo')
        assert c.call('inspect_mesh', asset='ribbon')['asset'] == before
        c.call('redo')
        assert c.call('inspect_mesh', asset='ribbon')['asset'] == sculpted
        checks.append('Procedural mesh creation and deterministic native sculpt preserve topology; undo/redo restore exact payload')

        bones = [
            {'name': 'root', 'parent': None, 'translation': [0, 0, -2], 'rotation': [0, 0, 0, 1]},
            {'name': 'middle', 'parent': 0, 'translation': [0, 0, 2], 'rotation': [0, 0, 0, 1]},
            {'name': 'tip', 'parent': 1, 'translation': [0, 0, 2], 'rotation': [0, 0, 0, 1]},
        ]
        weights = []
        for x, y, z in sculpted['geometry']['positions']:
            t = (z + 2) / 2
            low = min(1, int(t))
            f = t - low
            weights.append({'joints': [low, low + 1, 0, 0], 'weights': [1-f, f, 0, 0]})
        c.call('edit_mesh', edit={'kind': 'set_rig', 'asset': 'ribbon', 'rig': {'bones': bones, 'weights': weights}})
        instance = {'id': 'animated', 'asset': 'ribbon', 'position': [-2.8, 2.4, 0],
                    'rotation': [-math.sqrt(0.5), 0, 0, math.sqrt(0.5)], 'scale': 1,
                    'color': [0.85, 0.24, 0.09], 'playback': None}
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        bind = c.call('inspect_mesh_pose', instance='animated')['pose']
        assert len(bind['positions']) == 289 and len(bind['joints']) == 3
        angle = math.radians(55) / 2
        keys = [{'tick': t, 'translation': [0, 0, 2],
                 'rotation': [0, math.sin(a), 0, math.cos(a)]}
                for t, a in [(0, 0), (60, angle), (120, -angle), (180, 0)]]
        c.call('edit_mesh', edit={'kind': 'put_clip', 'asset': 'ribbon', 'clip': {
            'id': 'sway', 'duration_ticks': 180,
            'tracks': [{'bone': 1, 'keys': keys}, {'bone': 2, 'keys': keys}]}})
        instance['playback'] = {'clip': 'sway', 'offset_ticks': 0, 'looping': True}
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        sibling = copy.deepcopy(instance)
        sibling.update(id='offset', position=[2.8, 2.4, 0], color=[0.06, 0.55, 0.85])
        sibling['playback']['offset_ticks'] = 60
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': sibling})
        assert c.call('inspect_mesh_pose', instance='animated')['pose']['positions'] == bind['positions']
        assert c.call('inspect_mesh_pose', instance='offset')['pose']['clip_tick'] == 60
        summary = c.call('inspect_meshes')
        assert summary['assets'] == [{'id': 'ribbon', 'vertices': 289, 'triangles': 512, 'bones': 3, 'clips': ['sway']}]
        assert len(summary['instances']) == 2
        assert c.call('inspect')['runtime_positions'] == runtime
        checks.append('Native three-bone rig/weights, absolute local keyframes and independent shared-asset playback; bind identity and gameplay untouched')
        c.call('save_scene', path='examples/native-meshes.scene.json')
        scene = c.call('inspect')['scene']
        capture = c.call('capture', path='artifacts/meshes-start.png', width=960, height=720)
        assert capture['mesh_triangle_count'] == 1024 and capture['mesh_vertex_count'] == 3072
        c.call('step', ticks=45)
        live = c.call('inspect_mesh_pose', instance='animated')['pose']
        assert live['clip_tick'] == 45 and live['positions'] != bind['positions']
        snapshot = c.call('inspect')
        preview = c.call('inspect_mesh_pose', instance='animated', tick=90)['pose']
        assert preview['clip_tick'] == 90 and c.call('inspect') == snapshot
        c.call('inspect_mesh_pose', instance='animated', tick=36001, expected_error=True)
        c.call('capture', path='artifacts/meshes-live.png', width=960, height=720)
        c.call('capture', path='artifacts/meshes-repeat.png', width=960, height=720)
        assert (ROOT/'artifacts/meshes-start.png').read_bytes() != (ROOT/'artifacts/meshes-live.png').read_bytes()
        assert (ROOT/'artifacts/meshes-repeat.png').read_bytes() == (ROOT/'artifacts/meshes-live.png').read_bytes()
        c.call('save_replay', path='artifacts/meshes.replay.json')
        result = c.call('run_replay', path='artifacts/meshes.replay.json')
        assert result['matched'] and result['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/meshes.replay.json', path='artifacts/meshes-replay.png', width=960, height=720)
        c.call('capture_replay', replay_path='artifacts/meshes.replay.json', path='artifacts/meshes-prefix.png', tick=0, width=960, height=720)
        assert (ROOT/'artifacts/meshes-live.png').read_bytes() == (ROOT/'artifacts/meshes-replay.png').read_bytes()
        assert (ROOT/'artifacts/meshes-start.png').read_bytes() == (ROOT/'artifacts/meshes-prefix.png').read_bytes()
        assert c.call('inspect') == snapshot
        checks.append('Fixed-tick animation, read-only pose inspection, repeated GPU captures and replay/prefix positions/pixels agree')

        def reject(edit):
            current = c.call('inspect')
            c.call('edit_mesh', edit=edit, expected_error=True)
            assert current == c.call('inspect')

        invalid = copy.deepcopy(sculpted)
        invalid['geometry']['triangles'][0] = [0, 1, 99999]
        reject({'kind': 'put_asset', 'asset': invalid})
        invalid_rig = {'bones': copy.deepcopy(bones), 'weights': weights}
        invalid_rig['bones'][0]['parent'] = 2
        reject({'kind': 'set_rig', 'asset': 'ribbon', 'rig': invalid_rig})
        reject({'kind': 'remove_asset', 'id': 'ribbon'})
        reject({'kind': 'remove_clip', 'asset': 'ribbon', 'clip': 'sway'})
        reject({'kind': 'sculpt', 'asset': 'ribbon', 'stroke': {'kind': 'grab', 'centers': [[0, 0, 0]], 'radius': -1, 'strength': 0.5, 'direction': [1, 0, 0]}})
        reject({'kind': 'grid', 'id': 'oversized', 'segments': 64, 'size': 4})
        c.call('edit_mesh', edit={'kind': 'remove_instance', 'id': 'offset'}, expected_revision=snapshot['revision']-1, expected_error=True)
        assert c.call('inspect') == snapshot
        checks.append('Invalid indices/hierarchy/references/stroke/budgets and stale revision preserve scene/runtime/history atomically')

        # Reuse a cached renderer across shrink/grow/edit/undo/reset/viewport transitions.
        c.call('edit_mesh', edit={'kind': 'remove_instance', 'id': 'offset'})
        c.call('capture', path='artifacts/meshes-removed.png', width=960, height=720)
        c.call('undo')
        c.call('capture', path='artifacts/meshes-undo.png', width=960, height=720)
        assert (ROOT/'artifacts/meshes-start.png').read_bytes() == (ROOT/'artifacts/meshes-undo.png').read_bytes()
        c.call('redo')
        c.call('capture', path='artifacts/meshes-redo.png', width=960, height=720)
        assert (ROOT/'artifacts/meshes-removed.png').read_bytes() == (ROOT/'artifacts/meshes-redo.png').read_bytes()
        c.call('capture', path='artifacts/meshes-resized.png', width=640, height=480)
        c.call('load_scene', path='examples/native-meshes.scene.json')
        assert c.call('inspect')['scene'] == scene
        c.call('capture', path='artifacts/meshes-load.png', width=960, height=720)
        assert (ROOT/'artifacts/meshes-start.png').read_bytes() == (ROOT/'artifacts/meshes-load.png').read_bytes()
        c.call('step', ticks=17)
        c.call('reset')
        c.call('capture', path='artifacts/meshes-reset.png', width=960, height=720)
        assert (ROOT/'artifacts/meshes-start.png').read_bytes() == (ROOT/'artifacts/meshes-reset.png').read_bytes()
        checks.append('Renderer cache handles removal/growth, undo/redo, resize, save/load and reset with exact expected pixels')

        c.call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'budget', 'segments': 46, 'size': 0.8})
        budget=c.call('inspect_mesh',asset='budget')['asset'];budget['geometry']['triangles']=budget['geometry']['triangles'][:4096]
        c.call('edit_mesh',edit={'kind':'put_asset','asset':budget})
        instances = [{'id': f'budget-{n}', 'asset': 'budget', 'position': [(n % 8)-3.5, 1, (n//8)-1.5],
                      'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.3, 0.6, 0.2], 'playback': None} for n in range(32)]
        c.call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': i}} for i in instances])
        cap = c.call('capture', path='artifacts/meshes-budget.png', width=640, height=480)
        assert cap['mesh_triangle_count'] == 131072 and cap['mesh_vertex_count'] == 393216
        c.call('save_scene', path='artifacts/meshes-budget.scene.json')
        legacy = copy.deepcopy(c.call('inspect')['scene']); legacy['version'] = 64
        (ROOT/'artifacts/meshes-budget-old.scene.json').write_text(json.dumps(legacy))
        current = c.call('inspect')
        c.call('load_scene', path='artifacts/meshes-budget-old.scene.json', expected_error=True)
        assert c.call('inspect') == current
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'over', 'segments': 46, 'size': 0.8})
        over = c.call('inspect_mesh', asset='over')['asset']
        over['geometry']['triangles'] = over['geometry']['triangles'][:4097]
        c.call('edit_mesh', edit={'kind': 'put_asset', 'asset': over})
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {**instances[-1], 'asset': 'over'}})
        assert c.call('capture', path='artifacts/meshes-over.png', width=640, height=480)['mesh_triangle_count'] == 131073
        c.call('undo')
        c.call('save_replay', path='artifacts/meshes-budget.replay.json')
        assert c.call('run_replay', path='artifacts/meshes-budget.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/meshes-budget.replay.json', path='artifacts/meshes-budget-replay.png', width=640, height=480)
        assert (ROOT/'artifacts/meshes-budget.png').read_bytes() == (ROOT/'artifacts/meshes-budget-replay.png').read_bytes()
        checks.append('131072 drawn triangles reach native GPU/replay exactly; 131073 now succeeds, undo restores exact pixels, and legacy version64 rejects without mutation')
        report = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/meshes-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args()
    verify(args.binary.resolve(), args.window)
