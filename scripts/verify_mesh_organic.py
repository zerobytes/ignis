#!/usr/bin/env python3
"""Exercise the pure organic-junction preview through the real MCP dispatcher."""
import argparse
import copy
import json
import math
import struct
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-organic-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        assert 'preview_mesh_junction' in json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        c.call('generate', recipe={'seed': 85, 'width': 16, 'depth': 16, 'obstacles': 0})
        s = math.sqrt(0.5)
        normals = [[0, -1, 0], [-s, s, 0], [s, s, 0]]
        collars = [{'id': f'collar{i}', 'center': [v*1.2 for v in normal], 'normal': normal,
                    'radial': [0, 0, 1], 'radius': [0.45, 0.35, 0.3][i], 'samples': 8} for i, normal in enumerate(normals)]
        c.call('step', ticks=2); c.call('play')
        snapshot = c.call('inspect')
        raw = c.call('preview_mesh_junction', collars=collars)
        assert c.call('inspect') == snapshot
        assert raw['report']['boundary_edges'] == 24 and raw['report']['euler_characteristic'] == -1
        assert len(raw['boundaries']) == 3 and raw['collar_vertices'] == 24
        checks.append('Native MCP schema and default raw preview preserve scene/runtime/history while simulation is active')

        fitted = c.call('preview_mesh_junction', collars=collars, options={'refinement_levels': 2}, fusion={})
        assert c.call('inspect') == snapshot
        assert fitted['geometry']['positions'][:24] == raw['geometry']['positions'][:24]
        assert fitted['report']['field']['fixed_vertices'] == 48
        assert fitted['report']['field']['max_residual'] < 0.0001
        assert 0.05 < fitted['report']['volume_loss'] < 0.5
        assert fitted['geometry'] != raw['geometry']
        checks.append('Explicit fused field reshapes only the interior, preserving exact collars and bounded residual/volume diagnostics')

        continuous = c.call('preview_mesh_junction', collars=collars, options={'refinement_levels': 2},
                            fusion={'transition_lengths': [0.4,0.4,0.4]})
        assert c.call('inspect') == snapshot
        assert continuous['geometry']['positions'][:48] == fitted['geometry']['positions'][:48]
        assert continuous['geometry']['triangles'] == fitted['geometry']['triangles']
        expected_length = struct.unpack('f',struct.pack('f',0.4))[0]
        assert continuous['report']['field']['transition_lengths'] == [expected_length]*3
        assert continuous['report']['field']['max_residual'] < 0.0001
        for lengths in [[-1,0.1,0.1],[0,0.1,0.1],[0.85]*3,[0.1,0.1]]:
            c.call('preview_mesh_junction',collars=collars,fusion={'transition_lengths':lengths},expected_error=True)
            assert c.call('inspect') == snapshot
        checks.append('Opt-in tangent field retains both exact guard rings; invalid or overlapping bands reject purely even during play')

        for kind in ['radius', 'frame', 'samples', 'identity', 'unknown', 'fusion', 'count']:
            bad = copy.deepcopy(collars)
            extra = {}
            if kind == 'radius': bad[0]['radius'] = 0
            if kind == 'frame': bad[0]['normal'] = [0, 0, 0]
            if kind == 'samples': bad[0]['samples'] = 33
            if kind == 'identity': bad[1]['id'] = bad[0]['id']
            if kind == 'unknown': bad[0]['automatic_weld'] = True
            if kind == 'fusion': extra['fusion'] = {'center': [0, 30, 0]}
            if kind == 'count': bad.pop()
            c.call('preview_mesh_junction', collars=bad, expected_error=True, **extra)
            assert c.call('inspect') == snapshot
        c.call('preview_mesh_junction', collars=collars, expected_revision=snapshot['revision']-1, expected_error=True)
        assert c.call('inspect') == snapshot
        checks.append('Malformed/unsupported collars, fields, unknown keys and stale revisions reject without partial mutation')

        c.call('pause')
        c.call('edit_mesh', edit={'kind': 'put_asset', 'asset': {'id': 'organic', 'geometry': fitted['geometry'], 'shading': 'smooth'}})
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {'id': 'sample', 'asset': 'organic', 'position': [0, 3, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.2, 0.115, 0.055], 'playback': None}})
        scene = c.call('inspect')['scene']
        c.call('preview_mesh_junction', collars=collars)
        c.call('undo'); assert not c.call('inspect_meshes')['instances']
        c.call('redo'); assert c.call('inspect')['scene'] == scene
        c.call('set_camera', camera={'position': [4.6, 4.4, 8.5], 'target': [0, 3, 0], 'fov_degrees': 36})
        c.call('capture', path='artifacts/mesh-organic-live.png', width=960, height=800)
        c.call('save_scene', path='examples/native-organic-junction.scene.json')
        c.call('save_replay', path='artifacts/mesh-organic.replay.json')
        assert c.call('run_replay', path='artifacts/mesh-organic.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/mesh-organic.replay.json', path='artifacts/mesh-organic-replay.png', width=960, height=800)
        assert (ROOT/'artifacts/mesh-organic-live.png').read_bytes() == (ROOT/'artifacts/mesh-organic-replay.png').read_bytes()
        c.call('load_scene', path='examples/native-organic-junction.scene.json')
        c.call('capture', path='artifacts/mesh-organic-loaded.png', width=960, height=800)
        assert (ROOT/'artifacts/mesh-organic-live.png').read_bytes() == (ROOT/'artifacts/mesh-organic-loaded.png').read_bytes()
        checks.append('Publication uses existing put_asset; preview never enters undo history, and save/load/replay reproduce native pixels')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window}
        (ROOT/'artifacts/mesh-organic-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
