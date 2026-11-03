#!/usr/bin/env python3
"""Native loose-part inspection, pure previews and exact rigged extraction via MCP."""
import argparse
import math
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-mesh-islands-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        assert all(op in schema for op in ['inspect_mesh_islands', 'preview_mesh_edit', 'extract_islands'])
        c.call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        c.call('set_camera', camera={'position': [6, 5, 10], 'target': [0, 2, 0], 'fov_degrees': 42})
        positions = [[-2, 0, 0], [-0.2, 0, 0], [-1, 2, 0], [0.2, 0, 0], [2, 0, 0], [1, 2, 0], [5, 5, 5]]
        rig = {'bones': [{'name': 'root', 'parent': None, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]}],
               'weights': [{'joints': [0, 0, 0, 0], 'weights': [1, 0, 0, 0]} for _ in positions]}
        clip = {'id': 'bend', 'duration_ticks': 120, 'tracks': [{'bone': 0, 'keys': [
            {'tick': 0, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]},
            {'tick': 60, 'translation': [0, 0.4, 0], 'rotation': [0, 0, 0.19866933, 0.9800666]},
            {'tick': 120, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]}]}]}
        source = {'id': 'parts', 'geometry': {'positions': positions, 'triangles': [[0, 1, 2], [3, 4, 5]], 'material_coordinates': positions},
                  'shading': 'smooth', 'rig': rig, 'clips': [clip]}
        c.call('edit_mesh', edit={'kind': 'put_asset', 'asset': source})
        instance = {'id': 'whole', 'asset': 'parts', 'position': [0, 1, 0], 'rotation': [0, 0, 0, 1], 'scale': 1,
                    'color': [0.1, 0.7, 0.3], 'playback': {'clip': 'bend', 'offset_ticks': 0, 'looping': True}}
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        source = c.call('inspect_mesh', asset='parts')['asset']
        inspected = c.call('inspect_mesh_islands', asset='parts')
        assert c.call('inspect_mesh_islands', asset='parts') == inspected
        assert [s['id'] for s in inspected['topology']['islands']] == [0, 1]
        assert inspected['topology']['isolated_vertices'] == [6]
        assert all(math.isclose(x, y, abs_tol=1e-7) for a, b in zip(inspected['topology']['islands'][1]['bounds'], [[0.2, 0, 0], [2, 2, 0]]) for x, y in zip(a, b))
        edit = {'kind': 'extract_islands', 'asset': 'parts', 'id': 'detached', 'reference': inspected['reference'], 'islands': [1], 'mode': 'move'}
        c.call('select_mesh_components', selection={'asset': 'parts', 'reference': inspected['reference'], 'components': {'kind': 'faces', 'indices': [1]}})
        before = c.call('inspect')
        preview = c.call('preview_mesh_edit', edit=edit)
        assert c.call('inspect') == before
        assert len(preview['meshes']['assets']) == 2 and preview['meshes']['instances'] == before['scene']['meshes']['instances']
        for invalid in [dict(edit, islands=[]), dict(edit, islands=[1, 1]), dict(edit, islands=[99]), dict(edit, islands=[0, 1]), dict(edit, id='parts')]:
            c.call('preview_mesh_edit', edit=invalid, expected_error=True)
            c.call('edit_mesh', edit=invalid, expected_error=True)
            assert c.call('inspect') == before
        checks.append('Native schema, deterministic shared-index islands/bounds/isolated vertices, pure generic preview and atomic invalid selections')
        poses = {tick: c.call('inspect_mesh_pose', instance='whole', tick=tick)['pose'] for tick in [0, 60]}
        c.call('capture', path='artifacts/islands-before.png', width=800, height=700)
        c.call('step', ticks=60)
        c.call('capture', path='artifacts/islands-before-motion.png', width=800, height=700)
        detached = dict(instance, id='piece', asset='detached')
        c.call('edit_scene', edits=[{'op': 'mesh', 'edit': edit}, {'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': detached}}])
        assert c.call('inspect_mesh_selection')['selection'] is None
        published = c.call('inspect')['scene']['meshes']
        assert published['assets'] == preview['meshes']['assets']
        for name, indices in [('parts', [0, 1, 2, 6]), ('detached', [3, 4, 5])]:
            value = c.call('inspect_mesh', asset=name)['asset']
            for channel in ['positions', 'material_coordinates']:
                assert value['geometry'][channel] == [source['geometry'][channel][i] for i in indices]
            assert value['rig']['weights'] == [source['rig']['weights'][i] for i in indices]
            assert value['rig']['bones'] == source['rig']['bones'] and value['clips'] == source['clips']
            assert value['material'] == source['material'] and value['shading'] == source['shading']
            for tick in [0, 60]:
                assert c.call('inspect_mesh_pose', instance='whole' if name == 'parts' else 'piece', tick=tick)['pose']['positions'] == [poses[tick]['positions'][i] for i in indices]
        c.call('capture', path='artifacts/islands-after.png', width=800, height=700)
        assert (ROOT/'artifacts/islands-before.png').read_bytes() == (ROOT/'artifacts/islands-after.png').read_bytes()
        c.call('step', ticks=60)
        c.call('capture', path='artifacts/islands-after-motion.png', width=800, height=700)
        assert (ROOT/'artifacts/islands-before-motion.png').read_bytes() == (ROOT/'artifacts/islands-after-motion.png').read_bytes()
        before = c.call('inspect')
        c.call('edit_mesh', edit=edit, expected_error=True)
        assert c.call('inspect') == before
        c.call('undo')
        assert c.call('inspect_mesh', asset='parts')['asset'] == source
        c.call('redo')
        assert c.call('inspect')['scene']['meshes'] == published
        checks.append('Move preserves exact local channels/weights/full skeleton/clips and world poses; explicit instances reproduce rest and animated GPU pixels; batch undo/redo and stale-reference rejection')
        reference = c.call('inspect_mesh_islands', asset='detached')['reference']
        copy_edit = {'kind': 'extract_islands', 'asset': 'detached', 'id': 'copy', 'reference': reference, 'islands': [0], 'mode': 'copy'}
        before = c.call('inspect')
        copied = c.call('preview_mesh_edit', edit=copy_edit)
        assert c.call('inspect') == before
        c.call('edit_mesh', edit=copy_edit)
        assert c.call('inspect')['scene']['meshes'] == copied['meshes']
        assert c.call('inspect_mesh', asset='detached')['asset'] == published['assets'][1]
        c.call('save_scene', path='examples/mesh-islands.scene.json')
        authored = c.call('inspect')['scene']
        c.call('step', ticks=60)
        c.call('save_replay', path='artifacts/mesh-islands.replay.json')
        assert c.call('run_replay', path='artifacts/mesh-islands.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/mesh-islands.replay.json', path='artifacts/islands-replay.png', width=800, height=700)
        assert (ROOT/'artifacts/islands-replay.png').read_bytes() == (ROOT/'artifacts/islands-after-motion.png').read_bytes()
        c.call('capture_replay', replay_path='artifacts/mesh-islands.replay.json', path='artifacts/islands-prefix.png', tick=0, width=800, height=700)
        assert (ROOT/'artifacts/islands-prefix.png').read_bytes() == (ROOT/'artifacts/islands-after.png').read_bytes()
        c.call('load_scene', path='examples/mesh-islands.scene.json')
        assert c.call('inspect')['scene'] == authored
        c.call('reset')
        c.call('capture', path='artifacts/islands-reset.png', width=800, height=700)
        assert (ROOT/'artifacts/islands-reset.png').read_bytes() == (ROOT/'artifacts/islands-after.png').read_bytes()
        c.call('capture', path='artifacts/islands-viewport.png', width=640, height=480)
        checks.append('Copy full island keeps original independent; Scene58 save/load/reset and Replay62/live/prefix GPU equality, viewport and render cache')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window}
        (ROOT/'artifacts/mesh-islands-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
