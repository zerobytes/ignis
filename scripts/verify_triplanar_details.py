#!/usr/bin/env python3
"""Staged projection schema and atomic format gate; GPU mode uses triplanar_probe."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    detail = {'plane':'xy','tint': [0.3, 0.4, 0.2], 'opacity': 0.7, 'bump': 0.002,
              'segments': [{'start': [0, 0], 'end': [1, 1], 'widths': [0.1, 0.01]}],
              'triplanar': {}}
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                  'clientInfo': {'name': 'ignis-projection-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list'))
        assert 'triplanar' in schema and 'sharpness' in schema
        c.call('generate', recipe={'seed': 7, 'width': 16, 'depth': 14, 'obstacles': 0})
        c.call('edit_mesh', edit={'kind': 'grid', 'id': 'surface', 'segments': 1, 'size': 1})
        planar = copy.deepcopy(detail); planar.pop('triplanar')
        c.call('edit_mesh',edit={'kind':'set_material','asset':'surface','material':{'detail':planar}})
        c.call('edit_mesh',edit={'kind':'put_instance','instance':{'id':'surface-instance','asset':'surface','position':[0,1,0],'rotation':[0,0,0,1],'scale':1,'color':[.7,.65,.4]}})
        c.call('set_camera',camera={'position':[2,3,3],'target':[0,1,0],'fov_degrees':35})
        def capture(name):
            path=f'artifacts/projection-{name}.png';c.call('capture',path=path,width=600,height=600);return (ROOT/path).read_bytes()
        planar_pixels=capture('planar')
        c.call('step', ticks=7)
        before = c.call('inspect')
        for d in [dict(detail, triplanar={'sharpness': 0}),
                  dict(detail, triplanar={'sharpness': 9}),
                  dict(detail, triplanar={'sharpness': 2, 'unknown': 0})]:
            for op in ['preview_mesh_edit', 'edit_mesh']:
                c.call(op, edit={'kind': 'set_material', 'asset': 'surface',
                                'material': {'detail': d}}, expected_error=True)
                assert c.call('inspect') == before
        edit = {'kind':'set_material','asset':'surface','material':{'detail':detail}}
        c.call('preview_mesh_edit',edit=edit); assert c.call('inspect') == before
        c.call('edit_mesh',edit=edit)
        projected = c.call('inspect')['scene']
        assert c.call('inspect_mesh',asset='surface')['asset']['material']['detail']['triplanar']['sharpness'] == 2
        tri_pixels=capture('enabled');assert tri_pixels != planar_pixels and capture('repeat') == tri_pixels
        c.call('edit_mesh',edit={'kind':'set_material','asset':'surface','material':{'detail':planar}})
        assert capture('planar-restored') == planar_pixels
        c.call('undo');assert c.call('inspect')['scene'] == projected and capture('enabled-restored') == tri_pixels
        c.call('save_scene',path='examples/projection.scene.json')
        c.call('undo'); assert c.call('inspect')['scene'] == before['scene']
        c.call('redo'); assert c.call('inspect')['scene'] == projected
        c.call('load_scene',path='examples/projection.scene.json'); assert c.call('inspect')['scene'] == projected
        c.call('step',ticks=9);c.call('save_replay',path='artifacts/projection.replay.json')
        assert c.call('run_replay',path='artifacts/projection.replay.json')['meshes_matched']
        live=capture('live')
        for tick,pixels,name in [(9,live,'replay'),(0,tri_pixels,'prefix')]:
            path=f'artifacts/projection-{name}.png'
            c.call('capture_replay',replay_path='artifacts/projection.replay.json',path=path,tick=tick,width=600,height=600)
            assert (ROOT/path).read_bytes() == pixels

        checks.append('MCP schema exposes bounded opt-in projection; native preview/publication/save/load/replay work while invalid edits/previews preserve scene, tick, revision and undo history')
        c.call('load_scene', path='examples/native-tree-study.scene.json')
        original = c.call('inspect')['scene']
        # Include hidden recipe sources: a format gate must precede evaluation.
        locations = [('meshes', 'assets', 0)]
        for i, entry in enumerate(original.get('plants', {}).get('entries', [])):
            locations.append(('plants', 'entries', i, 'source'))
            if entry.get('assembly'):
                locations.append(('plants', 'entries', i, 'assembly', 'context'))
                for j, _ in enumerate(entry['assembly'].get('context_grafts', [])):
                    locations.append(('plants', 'entries', i, 'assembly', 'context_grafts', j, 'source'))
        for path in locations:
            s = copy.deepcopy(original)
            node = s
            for k in path: node = node[k]
            node.setdefault('material', {})['detail'] = detail
            s['version'] = 69
            (ROOT/'artifacts/projection-staged.scene.json').write_text(json.dumps(s))
            before = c.call('inspect')
            error = c.call('load_scene', path='artifacts/projection-staged.scene.json', expected_error=True)
            assert 'version70' in str(error), (path, error)
            assert c.call('inspect') == before
        checks.append(f'Scene69 refuses projection declarations in {len(locations)} visible/hidden asset sources before recipe evaluation, without mutating the runtime')
        # A legacy source normalizes to70 without enabling the optional projection.
        assert c.call('inspect')['scene'] == original
        c.call('capture', path='artifacts/projection-legacy.png', width=640, height=480)
        pixels = (ROOT/'artifacts/projection-legacy.png').read_bytes()
        c.call('reset')
        c.call('capture', path='artifacts/projection-reset.png', width=640, height=480)
        assert (ROOT/'artifacts/projection-reset.png').read_bytes() == pixels
        checks.append('Legacy scene loads and resets with identical GPU pixels; format70 supports opt-in publication')
        report = {'passed': True, 'checks': checks, 'native_window': window,
                  'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/triplanar-details-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(); p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true'); args = p.parse_args()
    verify(args.binary.resolve(), args.window)
