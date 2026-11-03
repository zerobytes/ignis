#!/usr/bin/env python3
"""Native material strokes, per-island domains, filtering and preservation via MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_materials import f32


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    def edit(kind, expected_error=False, **kwargs):
        return c.call('edit_mesh', expected_error=expected_error, edit={'kind': kind, **kwargs})
    def capture(name):
        c.call('capture', path=f'artifacts/surface-{name}.png', width=900, height=750)
        return (ROOT/f'artifacts/surface-{name}.png').read_bytes()
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-surface-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        assert all(op in json.dumps(c.request('tools/list')) for op in ['fit_material_coordinates', 'copy_material_coordinates', 'segments', 'widths'])
        c.call('generate', recipe={'seed': 7, 'width': 16, 'depth': 14, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        for e in entities: e['visible'] = False
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        edit('grid', id='unused', segments=1, size=1)
        edit('grid', id='surface', segments=4, size=3)
        edit('set_shading', asset='surface', shading='smooth')
        instance = {'id': 'sheet', 'asset': 'surface', 'position': [0, 1.5, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.35, 0.5, 0.22], 'playback': None}
        edit('put_instance', instance=instance)
        c.call('set_camera', camera={'position': [2.8, 5.8, 4.5], 'target': [0, 1.5, 0], 'fov_degrees': 36})
        c.call('set_light', light={'direction': [0.6, -1, 0.3], 'ambient': 0.22})
        source = c.call('inspect_mesh', asset='surface')['asset']
        fit = {'kind': 'fit_material_coordinates', 'asset': 'surface', 'per_island': True, 'bounds': [[-1, 0, 0], [1, 0, 1]]}
        before = c.call('inspect')
        preview = c.call('preview_mesh_edit', edit=fit)
        assert c.call('inspect') == before
        c.call('edit_mesh', edit=fit)
        fitted = c.call('inspect_mesh', asset='surface')['asset']
        assert fitted == preview['meshes']['assets'][1]
        assert fitted['geometry']['positions'] == source['geometry']['positions'] and fitted['geometry']['triangles'] == source['geometry']['triangles']
        detail = {'plane': 'xz', 'tint': [0.2, 0.4, 0.12], 'opacity': 0.8, 'bump': -0.003, 'segments': [
            {'start': [0, 0.05], 'end': [0, 0.95], 'widths': [0.04, 0.01]},
            {'start': [0, 0.35], 'end': [0.85, 0.60], 'widths': [0.02, 0.003]},
            {'start': [0, 0.55], 'end': [-0.75, 0.82], 'widths': [0.025, 0.004]}]}
        neutral = capture('neutral')
        material = f32({'transmission': 0.1, 'detail': detail})
        before = c.call('inspect')
        predicted = c.call('preview_mesh_edit', edit={'kind': 'set_material', 'asset': 'surface', 'material': material})
        assert c.call('inspect') == before
        edit('set_material', asset='surface', material=material)
        a = c.call('inspect_mesh', asset='surface')['asset']
        assert a == predicted['meshes']['assets'][1] and a['geometry'] == fitted['geometry']
        detailed = capture('detailed')
        assert detailed != neutral and capture('repeat') == detailed
        c.call('undo'); assert capture('undo') == neutral
        c.call('redo'); assert capture('redo') == detailed
        edit('remove_asset', id='unused'); assert capture('slot-removed') == detailed
        for name, key in [('bump-only', 'opacity'), ('color-only', 'bump')]:
            isolated = copy.deepcopy(material); isolated['detail'][key] = 0
            edit('set_material', asset='surface', material=isolated)
            pixels = capture(name); assert pixels != detailed and pixels != neutral
        edit('set_material', asset='surface', material=material); assert capture('channels-restored') == detailed
        checks.append('Native schema and pure previews; coordinate fit preserves geometry, independent line color/bump changes pixels; repeat/undo/redo/slot reindex exact')
        before = c.call('inspect')
        for change in [dict(detail, segments=[]), dict(detail, segments=detail['segments']*11), dict(detail, opacity=2), dict(detail, bump=0.1)]:
            edit('set_material', asset='surface', material={'detail': change}, expected_error=True)
            assert c.call('inspect') == before
        edit('fit_material_coordinates', asset='surface', bounds=[[1, 0, 0], [0, 1, 1]], expected_error=True)
        assert c.call('inspect') == before
        # Repeated parts receive the same domain from their own authored channel,
        # regardless of their positions in the combined asset.
        parts = copy.deepcopy(source)
        parts['id'] = 'parts'
        parts['geometry'] = {'positions': [[0, 0, 0], [1, 0, 0], [0, 0, 1], [3, 0, 0], [5, 0, 0], [3, 0, 2]],
            'triangles': [[0, 2, 1], [3, 5, 4]], 'material_coordinates': [[1, 0, 1], [2, 0, 1], [1, 0, 2], [10, 0, 10], [13, 0, 10], [10, 0, 13]]}
        edit('put_asset', asset=parts)
        edit('fit_material_coordinates', asset='parts', per_island=True, bounds=[[-1, 0, 0], [1, 0, 1]])
        coords = c.call('inspect_mesh', asset='parts')['asset']['geometry']['material_coordinates']
        assert coords[:3] == coords[3:]
        template = copy.deepcopy(parts); template['id'] = 'domain-template'
        template['geometry'] = {'positions': parts['geometry']['positions'][:3], 'triangles': [[0, 2, 1]],
            'material_coordinates': f32([[-0.7, 0, 0.2], [0.9, 0, 0.2], [-0.7, 0, 0.8]])}
        edit('put_asset', asset=template)
        before = c.call('inspect')
        edit('copy_material_coordinates', asset='parts', source='surface', per_island=True, expected_error=True)
        assert c.call('inspect') == before
        copy_edit = {'kind': 'copy_material_coordinates', 'asset': 'parts', 'source': 'domain-template', 'per_island': True}
        c.call('preview_mesh_edit', edit=copy_edit); assert c.call('inspect') == before
        c.call('edit_mesh', edit=copy_edit)
        assert c.call('inspect_mesh', asset='parts')['asset']['geometry']['material_coordinates'] == template['geometry']['material_coordinates'] * 2
        c.call('undo'); assert c.call('inspect_mesh', asset='parts')['asset']['geometry']['material_coordinates'] == coords
        c.call('redo')
        # Same posed world geometry with a translated bone and compensated instance
        # must retain the exact material image, proving rest-domain attachment.
        count = len(fitted['geometry']['positions'])
        edit('set_rig', asset='surface', rig={'bones': [{'name': 'root', 'parent': None, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]}], 'weights': [{'joints': [0, 0, 0, 0], 'weights': [1, 0, 0, 0]} for _ in range(count)]})
        edit('put_clip', asset='surface', clip={'id': 'move', 'duration_ticks': 120, 'tracks': [{'bone': 0, 'keys': [
            {'tick': t, 'translation': [0, y, 0], 'rotation': [0, 0, 0, 1]} for t, y in [(0, 0), (60, 0.5), (120, 0)]]}]})
        edit('put_instance', instance=dict(instance, position=[0, 1, 0], playback={'clip': 'move', 'offset_ticks': 60, 'looping': True}))
        assert capture('rest-domain-animation') == detailed
        edit('put_instance', instance=dict(instance, playback={'clip': 'move', 'offset_ticks': 0, 'looping': True}))
        assert capture('restored') == detailed
        checks.append('Invalid detail/fit rollback, per-island authored-domain normalization and posed compensated rigid animation retains exact detail pixels')
        # Subpixel detail fades in a fixed camera, rather than creating high-
        # contrast lines thinner than the sampling footprint.
        far = {'position': [0, 120, 100], 'target': [0, 1.5, 0], 'fov_degrees': 36}
        c.call('set_camera', camera=far); far_detail = capture('far-detail')
        edit('set_material', asset='surface', material={'transmission': 0.1}); far_plain = capture('far-neutral')
        assert far_detail == far_plain
        edit('set_material', asset='surface', material=material)
        c.call('set_camera', camera={'position': [2.8, 5.8, 4.5], 'target': [0, 1.5, 0], 'fov_degrees': 36})
        c.call('save_scene', path='examples/surface-details.scene.json')
        scene = c.call('inspect')['scene']; assert scene['version'] == SCENE_VERSION
        start = capture('start'); c.call('step', ticks=60); live = capture('live')
        c.call('save_replay', path='artifacts/surface-details.replay.json')
        assert c.call('run_replay', path='artifacts/surface-details.replay.json')['meshes_matched']
        for name, tick, expected in [('replay', None, live), ('prefix', 0, start)]:
            c.call('capture_replay', replay_path='artifacts/surface-details.replay.json', path=f'artifacts/surface-{name}.png', tick=tick, width=900, height=750)
            assert (ROOT/f'artifacts/surface-{name}.png').read_bytes() == expected
        c.call('load_scene', path='examples/surface-details.scene.json'); assert c.call('inspect')['scene'] == scene
        c.call('reset'); assert capture('reset') == start
        c.call('capture', path='artifacts/surface-viewport.png', width=640, height=480)
        checks.append('Subpixel attenuation, Scene60 load/reset and Replay64 live/prefix GPU equality with animated detail, viewport/cache')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window}
        (ROOT/'artifacts/surface-details-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(); p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis'); p.add_argument('--window', action='store_true')
    args = p.parse_args(); verify(args.binary.resolve(), args.window)
