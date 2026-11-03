#!/usr/bin/env python3
"""Verify effective posed-normal inspection through MCP without a window."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window=False):
    c = Client(binary, window)
    call = c.call
    checks = []
    def query(**args):
        return call('inspect_mesh_normals', instance='normal-one', **args)
    def capture(name):
        path = 'artifacts/normals-' + name + '.png'
        call('capture', path=path, width=640, height=480)
        return (ROOT / path).read_bytes()
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                  'clientInfo': {'name': 'normal-inspection-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        assert 'inspect_mesh_normals' in json.dumps(c.request('tools/list'))
        call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        asset = {'id': 'normal-fold', 'shading': 'flat', 'geometry': {
            'positions': [[0,0,0],[1,0,0],[0,1,0],[0,0,3]],
            'triangles': [[0,1,2],[0,3,1]]}}
        call('edit_mesh', edit={'kind': 'put_asset', 'asset': asset})
        call('edit_mesh', edit={'kind': 'put_instance', 'instance': {
            'id': 'normal-one', 'asset': 'normal-fold', 'position': [0,0,0],
            'rotation': [0,0,0,1], 'scale': 1, 'color': [.7,.6,.4]}})
        call('step', ticks=7)
        state = call('inspect')
        flat = query()['report']
        assert flat['tick'] == 7 and flat['sample_limit'] == 8
        assert flat['maximum_degrees'] == 0 and flat['world_area'] == 2
        assert flat['stage'] == 'pre_material' and flat['space'] == 'world'
        assert query(tick=0)['report']['tick'] == 0 and call('inspect') == state
        call('edit_mesh', edit={'kind': 'set_shading', 'asset': 'normal-fold', 'shading': 'smooth'})
        all_faces = query(sample_limit=32)['report']
        selected = query(faces=[0], sample_limit=1)['report']
        assert selected['samples'][0] == all_faces['samples'][0]
        assert math.isclose(selected['maximum_degrees'], math.degrees(math.atan(3)), abs_tol=1e-5)
        assert query(faces=[1,0], sample_limit=32)['report'] == all_faces
        assert query(sample_limit=0)['report']['samples'] == []
        checks.append('Typed pure world/pre-material query, explicit/current tick, area-weighted analytic normals and filtering retains all incident faces')

        image = capture('before-query'); state = call('inspect')
        for args in [dict(faces=[]), dict(faces=[0,0]), dict(faces=[2]),
                     dict(sample_limit=33), dict(tick=36001), dict(sample_limit=-1)]:
            call('inspect_mesh_normals', instance='normal-one', expected_error=True, **args)
            assert call('inspect') == state
        call('inspect_mesh_normals', instance='absent', expected_error=True)
        call('inspect_mesh_normals', instance='normal-one', unknown=1, expected_error=True)
        for _ in range(2):
            assert query(sample_limit=32)['report'] == all_faces
        assert call('inspect') == state and capture('after-query') == image
        call('undo'); assert query()['report']['shading'] == 'flat'
        call('redo'); assert query(sample_limit=32)['report'] == all_faces
        checks.append('Bounded samples/filter/schema/missing errors are atomic; queries preserve pixels, runtime, revision and undo history')

        seam = copy.deepcopy(asset); seam['shading'] = 'smooth'
        seam['geometry']['positions'] += [[0,0,0],[1,0,0]]
        seam['geometry']['triangles'][1] = [4,3,5]
        call('edit_mesh', edit={'kind': 'put_asset', 'asset': seam})
        assert query()['report']['maximum_degrees'] == 0
        cancelled = copy.deepcopy(asset); cancelled['shading'] = 'smooth'
        cancelled['geometry']['triangles'] = [[0,1,2],[0,2,1]]
        call('edit_mesh', edit={'kind': 'put_asset', 'asset': cancelled})
        r = query()['report']; assert r['smooth_fallback_corners'] == 6 and r['geometric_fallback_faces'] == 0
        assert r['samples'][0]['geometric_normal'] == [0,0,1] and r['samples'][1]['geometric_normal'] == [0,0,-1]
        checks.append('Coincident positions with separate indices retain seams; cancelling incident normals report exact per-face fallback')

        call('undo'); call('undo')
        baseline = query(sample_limit=32)['report']; image = capture('restored')
        call('save_scene', path='examples/normal-inspection.scene.json')
        call('step', ticks=10); live_image = capture('live'); call('save_replay', path='artifacts/normals.replay.json')
        assert call('run_replay', path='artifacts/normals.replay.json')['meshes_matched']
        assert query(sample_limit=32)['report'] == {**baseline, 'tick': 10}
        call('capture_replay', replay_path='artifacts/normals.replay.json', path='artifacts/normals-replay.png', width=640, height=480)
        assert (ROOT/'artifacts/normals-replay.png').read_bytes() == live_image
        call('load_scene', path='examples/normal-inspection.scene.json'); assert query(sample_limit=32)['report'] == baseline
        call('reset'); assert query(sample_limit=32)['report'] == baseline
        assert capture('loaded') == image
        checks.append('Save/load/reset/replay preserve derived normals and captures; query results track the evaluated tick without persistence changes')
        report = {'passed': True, 'checks': checks, 'build_id': call('inspect')['build_id']}
        (ROOT/'artifacts/mesh-normal-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(dict(passed=True, checks=len(checks), build_id=report['build_id'])))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary, args.window)
