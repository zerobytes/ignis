#!/usr/bin/env python3
"""Reusable native panel construction; parameter changes never supply triangles."""
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT
from reconstruction_structure import inspect_surface


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'panel-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = c.request('tools/list')['result']['tools'][0]['inputSchema']
        assert 'panel' in json.dumps(schema) and 'openings' in json.dumps(schema)
        c.call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        panel = {'kind': 'panel', 'id': 'wall', 'dimensions': [6, 4, 0.3],
                 'openings': [{'min': [-2, -2], 'max': [-1, 0.5]}, {'min': [0, -0.5], 'max': [1.5, 1]}]}
        before = c.call('inspect')
        preview = c.call('preview_mesh_edit', edit=panel)
        assert c.call('inspect') == before
        c.call('edit_mesh', edit=panel, expected_revision=before['revision'])
        asset = c.call('inspect_mesh', asset='wall')['asset']
        assert asset == preview['meshes']['assets'][0]
        audit = inspect_surface(asset['geometry'])
        assert audit['solid_preflight_passed'], audit
        assert abs(audit['components'][0]['signed_volume_diagnostic'] - 5.775) < 1e-5
        topology = c.call('inspect_mesh_topology', asset='wall')['topology']
        assert topology['boundary_edges'] == topology['non_manifold_edges'] == 0
        checks.append('Schema, pure preview, indexed closed surface, orientation and analytic material volume')
        changed = {**panel, 'dimensions': [8, 5, 0.5]}
        c.call('edit_mesh', edit=changed)
        assert c.call('inspect_mesh', asset='wall')['asset'] != asset
        c.call('undo')
        assert c.call('inspect_mesh', asset='wall')['asset'] == asset
        c.call('redo')
        c.call('edit_mesh', edit=panel)
        assert c.call('inspect_mesh', asset='wall')['asset'] == asset
        checks.append('Parameter-only A-B-A regeneration and exact undo/redo')

        def reject(edit, op='edit_mesh', **extra):
            state = c.call('inspect')
            c.call(op, edit=edit, expected_error=True, **extra)
            assert c.call('inspect') == state
            assert c.call('inspect_mesh', asset='wall')['asset'] == asset

        for invalid in [
            {**panel, 'dimensions': [0, 4, 0.3]},
            {**panel, 'openings': [{'min': [-3, -1], 'max': [-2, 1]}]},
            {**panel, 'openings': [{'min': [0, 0], 'max': [1, 1]}, {'min': [0.5, 0.5], 'max': [1.5, 1.5]}]},
            {**panel, 'openings': panel['openings'] * 9},
        ]:
            reject(invalid)
            reject(invalid, 'preview_mesh_edit')
        reject(panel, expected_revision=c.call('inspect')['revision'] - 1)
        checks.append('Invalid dimensions, side breach, overlapping openings, count and stale revision are atomic')
        state = c.call('inspect')
        c.call('edit_scene', edits=[{'op': 'mesh', 'edit': {**panel, 'id': 'temporary'}},
                                   {'op': 'mesh', 'edit': {**panel, 'dimensions': [0, 4, 1]}}], expected_error=True)
        assert c.call('inspect') == state
        c.call('inspect_mesh', asset='temporary', expected_error=True)
        checks.append('Failed mixed batch preserves scene and history')

        # A second use has unrelated dimensions/layout, using the identical native command.
        screen = {'kind': 'panel', 'id': 'screen', 'dimensions': [2, 6, 0.1],
                  'openings': [{'min': [-0.6, -2], 'max': [0.6, 2]}]}
        c.call('edit_mesh', edit=screen)
        assert inspect_surface(c.call('inspect_mesh', asset='screen')['asset']['geometry'])['solid_preflight_passed']
        for name, pos, rotation, scale in [('wall', [-2, 2, 0], [0, 0, 0, 1], 1),
                                            ('screen', [3, 2.4, 1], [0, 0.70710677, 0, 0.70710677], 0.8)]:
            c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {'id': name, 'asset': name,
                   'position': pos, 'rotation': rotation, 'scale': scale, 'color': [0.72, 0.64, 0.48], 'playback': None}})
        pose = c.call('inspect_mesh_pose', instance='screen')['pose']['positions']
        assert max(p[0] for p in pose) - min(p[0] for p in pose) < 0.081
        checks.append('Second panel layout and rotated/scaled world instance use the same constructor')

        definition = {'id': 'linked', 'profile': {'id': 'outline', 'width': 1, 'height': 2},
                      'depth': 0.2, 'segments': [1, 1, 1]}
        c.call('edit_plant', edit={'kind': 'put', 'definition': definition,
               'detail_policy': 'physical', 'animation_translation_policy': 'physical'})
        reject({**panel, 'id': 'linked'})
        reject({**panel, 'id': 'linked'}, 'preview_mesh_edit')
        checks.append('Linked procedural source cannot be overwritten by panel in edit or preview')
        c.call('set_camera', camera={'position': [9, 7, 12], 'target': [0, 2.5, 0], 'fov_degrees': 42})
        c.call('capture', path='artifacts/panels-front.png', width=960, height=720)
        c.call('save_scene', path='examples/native-panels.scene.json')
        scene = c.call('inspect')['scene']
        c.call('set_camera', camera={'position': [-8, 6, -10], 'target': [0, 2.5, 0], 'fov_degrees': 42})
        c.call('capture', path='artifacts/panels-back.png', width=960, height=720)
        c.call('load_scene', path='examples/native-panels.scene.json')
        assert c.call('inspect')['scene'] == scene
        c.call('capture', path='artifacts/panels-loaded.png', width=960, height=720)
        assert (ROOT/'artifacts/panels-front.png').read_bytes() == (ROOT/'artifacts/panels-loaded.png').read_bytes()
        checks.append('Scene persistence and independent back/front captures; load pixels exactly restored')
        c.call('save_replay', path='artifacts/panels.replay.json')
        assert c.call('run_replay', path='artifacts/panels.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/panels.replay.json', path='artifacts/panels-replay.png', width=960, height=720)
        assert (ROOT/'artifacts/panels-front.png').read_bytes() == (ROOT/'artifacts/panels-replay.png').read_bytes()
        checks.append('Static mesh replay and offscreen GPU pixels match; no gameplay/collision claim')
        report = {'passed': True, 'checks': checks, 'audit': audit, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/panels-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args()
    verify(args.binary.resolve(), args.window)
