#!/usr/bin/env python3
"""Exercise pure material-domain diagnostics through the shared MCP dispatcher."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window=False):
    client = Client(binary, window)
    call = client.call
    checks = []
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                       'clientInfo': {'name': 'material-domain-verifier', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        schema = json.dumps(client.request('tools/list')['result'])
        assert all(op in schema for op in ['inspect_mesh_material_domain', 'preview_mesh_material_domain'])
        call('generate', recipe={'seed': 91, 'width': 20, 'depth': 20, 'obstacles': 0})
        geometry = {'positions': [[0, 0, 0], [1, 0, 0], [0, 1, 0]], 'triangles': [[0, 1, 2]],
                    'material_coordinates': [[0, 0, 0], [1, 0, 0], [1, 1, 0]]}
        call('edit_mesh', edit={'kind': 'put_asset', 'asset': {'id': 'domain', 'geometry': geometry}})
        call('step', ticks=7)
        before = call('inspect')
        raw = call('preview_mesh_material_domain', source=geometry)
        named = call('inspect_mesh_material_domain', asset='domain')
        assert raw['report'] == named['report']
        assert call('preview_mesh_material_domain', source=geometry) == raw
        assert call('inspect') == before
        report = raw['report']
        assert math.isclose(report['maximum_rate']['value'], math.sqrt((3+math.sqrt(5))/2))
        assert report['selected_faces'] == 1 and report['samples'][0]['rank'] == 2
        assert report['coordinates'] == 'explicit'
        checks.append('Typed pure named/explicit diagnostics, known shear singular rates, repeat and runtime/revision/history preserved')

        projected = call('preview_mesh_material_domain', source=geometry, options={'domain': 'xz'})['report']
        assert projected['rank_one_faces'] == 1 and projected['maximum_anisotropy'] is None
        fallback = copy.deepcopy(geometry)
        del fallback['material_coordinates']
        result = call('preview_mesh_material_domain', source=fallback, options={'scale': [4, 2, 1], 'sample_limit': 0})['report']
        assert result['coordinates'] == 'local_positions_fallback' and result['samples'] == []
        assert result['minimum_rate']['value'] == 2 and result['maximum_rate']['value'] == 4
        collapsed = copy.deepcopy(geometry)
        collapsed['material_coordinates'] = [[0, 0, 0]] * 3
        result = call('preview_mesh_material_domain', source=collapsed)['report']
        assert result['rank_zero_faces'] == 1 and result['area_weighted_mean_anisotropy'] is None
        checks.append('Projection collapse distinct from spatial/shear, finite JSON, explicit positive scale and renderer-compatible fallback')

        for args in [dict(faces=[]), dict(faces=[0, 0]), dict(faces=[1]),
                     dict(options={'sample_limit': 33}), dict(options={'scale': [0, 1, 1]}),
                     dict(options={'scale': [-1, 1, 1]}), dict(options={'unknown': 3})]:
            call('preview_mesh_material_domain', source=geometry, expected_error=True, **args)
            assert call('inspect') == before
        call('inspect_mesh_material_domain', asset='missing', expected_error=True)
        call('inspect_mesh_material_domain', asset='domain', expected_revision=before['revision']-1, expected_error=True)
        assert call('inspect') == before
        # Queries must not add undo steps or change what redo restores.
        call('undo')
        call('inspect_mesh', asset='domain', expected_error=True)
        call('redo')
        assert call('inspect_mesh', asset='domain')['asset']['geometry'] == geometry
        checks.append('Selection/scale/schema/missing/stale failures are atomic and queries add no undo steps')
        report = {'passed': True, 'checks': checks, 'native_window': window,
                  'build_id': call('inspect')['build_id']}
        (ROOT/'artifacts/mesh-material-domain-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        client.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary, args.window)
