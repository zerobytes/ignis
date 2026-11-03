#!/usr/bin/env python3
"""Rounded, periodic native strokes: GPU filtering, independent channels and history."""
from verify_lab import REPLAY_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from png_pixels import max_error


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    def edit(kind, **kw):
        return c.call('edit_mesh', edit={'kind': kind, **kw})
    def capture(name):
        p = f'artifacts/detail-domain-{name}.png'
        c.call('capture', path=p, width=800, height=800)
        return (ROOT/p).read_bytes()
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'detail-domains', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')); assert 'rounded' in schema and 'repeat' in schema
        c.call('generate', recipe={'width': 16, 'depth': 14, 'seed': 7, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': dict(e, visible=False)} for e in entities])
        edit('grid', id='sheet', segments=2, size=2)
        edit('put_instance', instance={'id': 'one', 'asset': 'sheet', 'position': [0, 1.5, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.5, 0.45, 0.4], 'playback': None})
        c.call('set_camera', camera={'position': [2, 4, 3], 'target': [0, 1.5, 0], 'fov_degrees': 40})
        c.call('set_light', light={'direction': [-0.8, -1, -0.4], 'ambient': 0.25})
        d = {'profile': 'rounded', 'repeat': [0.5, 0.5], 'plane': 'xz', 'tint': [0.25, 0.3, 0.35], 'opacity': 0.8, 'bump': -0.004,
             'segments': [{'start': [0.125, 0.125], 'end': [0.375, 0.375], 'widths': [0.03125, 0.0625]}]}
        material = {'detail': d}
        neutral = capture('neutral')
        before = c.call('inspect'); pred = c.call('preview_mesh_edit', edit={'kind': 'set_material', 'asset': 'sheet', 'material': material})
        assert c.call('inspect') == before
        edit('set_material', asset='sheet', material=material)
        assert pred['meshes'] == c.call('inspect')['scene']['meshes']
        combined = capture('combined'); assert combined != neutral and capture('repeat') == combined
        c.call('undo'); assert capture('undo') == neutral
        c.call('redo'); assert capture('redo') == combined
        ablations = []
        for name, key in [('color', 'bump'), ('bump', 'opacity')]:
            m = copy.deepcopy(material); m['detail'][key] = 0
            edit('set_material', asset='sheet', material=m)
            p = capture(name); assert p not in [neutral, combined]; ablations.append(p)
        assert ablations[0] != ablations[1]
        band = copy.deepcopy(material); band['detail']['profile'] = 'band'
        edit('set_material', asset='sheet', material=band); assert capture('band') != combined
        edit('set_material', asset='sheet', material=material)
        geometry = c.call('inspect_mesh', asset='sheet')['asset']['geometry']
        # Material coordinates include negatives and cross many period cells.
        # Translating them by exact binary periods must preserve appearance within one 8-bit quantization level.
        coordinates = [[p[0]+2, p[1], p[2]-2] for p in geometry['positions']]
        edit('set_material_coordinates', asset='sheet', coordinates=coordinates)
        shifted = capture('translated-domain')
        assert max_error(shifted, combined) <= 1, 'Period translation exceeded one 8-bit quantization level'
        c.call('undo'); assert capture('domain-restored') == combined
        checks.append('Rounded versus band, repeat across positive/negative cells, independent color/bump, preview/history and binary-period translation within one 8-bit level')
        for change in [dict(d, repeat=[0, 1]), dict(d, repeat=[0.1, 0.1]), dict(d, repeat=[1, 10001]), dict(d, profile='sphere')]:
            state = c.call('inspect')
            c.call('edit_mesh', edit={'kind': 'set_material', 'asset': 'sheet', 'material': {'detail': change}}, expected_error=True)
            assert c.call('inspect') == state
        # PlaneXZ is dominant on this sheet: triplanar must produce the same field.
        tri = copy.deepcopy(material); tri['detail']['triplanar'] = {}
        edit('set_material', asset='sheet', material=tri); assert capture('triplanar') == combined
        c.call('save_scene', path='examples/detail-domains.scene.json'); scene = c.call('inspect')['scene']
        legacy = copy.deepcopy(scene); legacy['version'] = 70
        (ROOT/'artifacts/detail-domain-old.scene.json').write_text(json.dumps(legacy))
        state = c.call('inspect')
        err = c.call('load_scene', path='artifacts/detail-domain-old.scene.json', expected_error=True)
        assert 'version71' in str(err) and c.call('inspect') == state
        camera = scene['camera']
        c.call('set_camera', camera={'position': [0, 150, 120], 'target': [0, 1.5, 0], 'fov_degrees': 40})
        far = capture('far')
        edit('set_material', asset='sheet', material={}); assert capture('far-neutral') == far
        c.call('load_scene', path='examples/detail-domains.scene.json'); assert capture('loaded') == combined
        c.call('capture', path='artifacts/detail-domain-resize.png', width=640, height=480)
        assert capture('resize-restored') == combined
        c.call('save_replay', path='artifacts/detail-domains.replay.json')
        assert json.loads((ROOT/'artifacts/detail-domains.replay.json').read_text())['version'] == REPLAY_VERSION
        assert c.call('run_replay', path='artifacts/detail-domains.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/detail-domains.replay.json', path='artifacts/detail-domain-replay.png', width=800, height=800)
        assert (ROOT/'artifacts/detail-domain-replay.png').read_bytes() == combined
        checks.append('Invalid capsule/period/profile rollback; Scene71 gate, matching triplanar plane, subpixel fade, load/viewport and Replay75 exact pixels')
        # Hidden recipe declarations are gated before their re-evaluation.
        c.call('load_scene', path='examples/native-tree-study.scene.json'); original = c.call('inspect')['scene']
        for path in [('source',), ('assembly', 'context'), ('assembly', 'context_grafts', 0, 'source')]:
            s = copy.deepcopy(original); s['version'] = 70
            node = s['plants']['entries'][0]
            for k in path: node = node[k]
            node['material']['detail'] = d
            (ROOT/'artifacts/detail-domain-hidden.scene.json').write_text(json.dumps(s)); state = c.call('inspect')
            err = c.call('load_scene', path='artifacts/detail-domain-hidden.scene.json', expected_error=True)
            assert 'version71' in str(err) and c.call('inspect') == state
        checks.append('Profile/repeat format gate covers all hidden recipe sources before evaluation')
        report = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/detail-domains-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(); p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis'); p.add_argument('--window', action='store_true')
    a = p.parse_args(); verify(a.binary.resolve(), a.window)
