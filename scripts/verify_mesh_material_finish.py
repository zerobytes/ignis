#!/usr/bin/env python3
"""Native pure finish, affine interior, transactional publication and persistence."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window=False):
    client = Client(binary, window)
    call = client.call
    checks = []
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                       'clientInfo': {'name': 'material-finish-verifier', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        assert 'preview_mesh_material_finish' in json.dumps(client.request('tools/list')['result'])
        call('generate', recipe={'seed': 32, 'width': 20, 'depth': 20, 'obstacles': 0})
        positions = [[x, y, 0] for x in [-1, 0, .25, .6, .1] for y in [0, 1]]
        coords = copy.deepcopy(positions)
        coords[8:10] = copy.deepcopy(coords[2:4])
        geometry = {'positions': positions, 'material_coordinates': coords,
                    'triangles': [[0, 2, 1], [1, 2, 3], [2, 8, 3], [3, 8, 9],
                                  [8, 4, 9], [9, 4, 5], [4, 6, 5], [5, 6, 7]]}
        call('edit_mesh', edit={'kind': 'put_asset', 'asset': {'id': 'finish', 'geometry': geometry}})
        source = call('inspect_mesh', asset='finish')['asset']
        plan = {'retained_vertices': 4, 'retained_faces': 2, 'iso_vertices': 4, 'pairs': [[2, 8], [3, 9]]}
        options = {'reference': {'columns': [[1, 0, 0], [0, 1, 0], [0, 0, 1]], 'offset': [0, 0, 0]},
                   'iterations': 64, 'tolerance': .00001, 'max_change': 1, 'max_anisotropy': 100}
        call('step', ticks=9)
        before = call('inspect')
        result = call('preview_mesh_material_finish', source=source, plan=plan, options=options)
        assert call('preview_mesh_material_finish', source=source, plan=plan, options=options) == result
        assert call('inspect') == before
        report = result['report']
        assert report['iterations'] == 0 and report['residual_max'] == 0
        assert report['changed_guards'] == 2 and report['changed_interior'] == 0
        assert report['before']['guard']['near_singular_faces'] == 2
        assert report['after']['guard']['near_singular_faces'] == 0
        assert result['asset']['geometry']['positions'] == source['geometry']['positions']
        assert result['asset']['geometry']['triangles'] == source['geometry']['triangles']
        assert result['asset']['geometry']['material_coordinates'][4:8] == source['geometry']['material_coordinates'][4:8]
        assert result['asset']['geometry']['material_coordinates'][8:] == source['geometry']['positions'][8:]
        checks.append('Pure deterministic final-geometry finish, affine zeroRHS and exact correct interior/geometry/source with repaired guard rank')
        for case in range(7):
            s = copy.deepcopy(source)
            p = copy.deepcopy(plan)
            o = copy.deepcopy(options)
            if case == 0:
                del s['geometry']['material_coordinates']
            elif case == 1:
                p['pairs'] *= 2
            elif case == 2:
                p['iso_vertices'] = 999
            elif case == 3:
                o['max_change'] = .00001
            elif case == 4:
                o['iterations'] = 65
            elif case == 5:
                s['geometry']['material_coordinates'] = [[0, 0, 0]] * len(positions)
            else:
                o['reference']['unknown'] = 1
            call('preview_mesh_material_finish', source=s, plan=p, options=o, expected_error=True)
            assert call('inspect') == before
        call('preview_mesh_material_finish', source=source, plan=plan, options=options,
             expected_revision=before['revision']-1, expected_error=True)
        assert call('inspect') == before
        checks.append('Missing channel/invalid provenance/envelope/budget/authored rank/schema/stale errors preserve complete runtime/history')
        call('edit_mesh', edit={'kind': 'put_asset', 'asset': result['asset']})
        assert call('inspect_mesh', asset='finish')['asset'] == result['asset']
        call('undo')
        assert call('inspect_mesh', asset='finish')['asset'] == source
        call('redo')
        assert call('inspect_mesh', asset='finish')['asset'] == result['asset']
        call('save_scene', path='examples/material-finish.scene.json')
        call('step', ticks=11)
        call('save_replay', path='artifacts/material-finish.replay.json')
        assert call('run_replay', path='artifacts/material-finish.replay.json')['meshes_matched']
        call('load_scene', path='examples/material-finish.scene.json')
        assert call('inspect_mesh', asset='finish')['asset'] == result['asset']
        checks.append('Explicit mesh publication, undo/redo and existing scene/replay persistence retain finished coordinates without new formats')
        summary = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': call('inspect')['build_id']}
        (ROOT/'artifacts/mesh-material-finish-verification.json').write_text(json.dumps(summary, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': summary['build_id']}))
    finally:
        client.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--window', action='store_true')
    args = p.parse_args()
    verify(args.binary, args.window)
