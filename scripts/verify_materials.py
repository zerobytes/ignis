#!/usr/bin/env python3
"""Procedural material channels, metadata and native rendering through MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
import math
import struct
from pathlib import Path
from verify_lab import Client, ROOT


def f32(value):
    if isinstance(value, float): return struct.unpack('<f', struct.pack('<f', value))[0]
    if isinstance(value, list): return [f32(x) for x in value]
    if isinstance(value, dict): return {k: f32(v) for k,v in value.items()}
    return value


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    def edit(kind, **kwargs):
        return c.call('edit_mesh', edit={'kind': kind, **kwargs})
    def capture(name):
        c.call('capture', path=f'artifacts/material-{name}.png', width=960, height=720)
        return (ROOT/f'artifacts/material-{name}.png').read_bytes()
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'ignis-material-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        assert 'set_material_coordinates' in json.dumps(c.request('tools/list'))
        c.call('generate', recipe={'seed': 7, 'width': 16, 'depth': 14, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        for e in entities: e['visible'] = False
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        c.call('set_camera', camera={'position': [3, 6, 5], 'target': [0, 1.5, 0], 'fov_degrees': 36})
        c.call('set_light', light={'direction': [0.6, -1, 0.3], 'ambient': 0.20})
        edit('grid', id='unused', segments=1, size=1)
        edit('grid', id='surface', segments=6, size=4)
        edit('set_shading', asset='surface', shading='smooth')
        edit('put_instance', instance={'id': 'surface-instance', 'asset': 'surface', 'position': [0, 1.5, 0],
             'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.4, 0.3, 0.12], 'playback': None})
        neutral = capture('neutral')
        before = c.call('inspect_mesh', asset='surface')['asset']
        pose = c.call('inspect_mesh_pose', instance='surface-instance')['pose']
        material = {'pattern': {'frequency': [8, 2, 12], 'low': [0.2, 0.3, 0.2], 'high': [1.5, 1.5, 1.1],
                    'bump': 0.005, 'warp': 0.6, 'octaves': 3, 'seed': 17, 'contrast': 1.6, 'kind': 'ridged'}, 'transmission': 0}
        material = f32(material)
        edit('set_material', asset='surface', material=material)
        assert c.call('inspect_mesh', asset='surface')['asset'] == {**before, 'material': material}
        assert c.call('inspect_mesh_pose', instance='surface-instance')['pose'] == pose
        patterned = capture('pattern')
        assert patterned != neutral and patterned == capture('repeat')
        c.call('undo'); assert capture('undo') == neutral
        c.call('redo'); assert capture('redo') == patterned
        for kind in ['noise', 'cellular']:
            variant = copy.deepcopy(material); variant['pattern']['kind'] = kind
            edit('set_material', asset='surface', material=variant)
            assert capture(kind) != patterned
        edit('set_material', asset='surface', material=material)
        assert capture('kind-restored') == patterned
        edit('remove_asset', id='unused')
        assert capture('slot-reindexed') == patterned
        checks.append('MCP recipe changes native pixels without poses/geometry; undo/redo/repeat and material slot reindex remain pixel exact')
        coordinates = f32([[p[0]*0.4+3, p[1]+7, p[2]*0.8-2] for p in before['geometry']['positions']])
        edit('set_material_coordinates', asset='surface', coordinates=coordinates)
        mapped = capture('mapped')
        assert mapped != patterned
        assert c.call('inspect_mesh', asset='surface')['asset']['geometry']['material_coordinates'] == coordinates
        snapshot = c.call('inspect')
        for bad in [coordinates[:-1], [[10001, 0, 0]]*len(coordinates)]:
            c.call('edit_mesh', edit={'kind': 'set_material_coordinates', 'asset': 'surface', 'coordinates': bad}, expected_error=True)
            assert c.call('inspect') == snapshot
        for key, value in [('frequency', [0, 1, 1]), ('bump', 0.2), ('octaves', 5), ('kind', 'invalid')]:
            bad = copy.deepcopy(material); bad['pattern'][key] = value
            c.call('edit_mesh', edit={'kind': 'set_material', 'asset': 'surface', 'material': bad}, expected_error=True)
            assert c.call('inspect') == snapshot
        c.call('save_scene', path='examples/native-materials.scene.json')
        saved = c.call('inspect')['scene']
        assert saved['version'] == SCENE_VERSION
        forged = copy.deepcopy(saved); forged['version'] = 53
        (ROOT/'artifacts/material-forged.scene.json').write_text(json.dumps(forged))
        c.call('load_scene', path='artifacts/material-forged.scene.json', expected_error=True)
        assert c.call('inspect') == snapshot
        checks.append('Explicit material coordinates change mapping; channel/recipe bounds and old-version smuggling fail atomically')
        c.call('set_light', light={'direction': [0, 1, 0], 'ambient': 0.1})
        no_transmission = capture('opaque')
        edit('set_material', asset='surface', material={**material, 'transmission': 0.6})
        assert capture('transmitted') != no_transmission
        c.call('load_scene', path='examples/native-materials.scene.json')
        assert capture('loaded') == mapped
        bones = [{'name': 'root', 'parent': None, 'translation': [0,0,0], 'rotation': [0,0,0,1]}]
        weights = [{'joints': [0,0,0,0], 'weights': [1,0,0,0]} for _ in coordinates]
        edit('set_rig', asset='surface', rig={'bones': bones, 'weights': weights})
        edit('put_clip', asset='surface', clip={'id': 'turn', 'duration_ticks': 120, 'tracks': [{'bone': 0, 'keys': [
            {'tick': 0, 'translation': [0,0,0], 'rotation': [0,0,0,1]},
            {'tick': 60, 'translation': [0.1,0,0], 'rotation': [math.sin(0.3),0,0,math.cos(0.3)]},
            {'tick': 120, 'translation': [0,0,0], 'rotation': [0,0,0,1]}]}]})
        instance = c.call('inspect_meshes')['instances'][0]
        instance['playback'] = {'clip': 'turn', 'offset_ticks': 0, 'looping': True}
        edit('put_instance', instance=instance)
        start = capture('start')
        c.call('step', ticks=45)
        live = capture('live'); assert live != start
        assert c.call('inspect_mesh', asset='surface')['asset']['geometry']['material_coordinates'] == coordinates
        c.call('save_replay', path='artifacts/material.replay.json')
        assert c.call('run_replay', path='artifacts/material.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/material.replay.json', path='artifacts/material-replay.png', width=960, height=720)
        c.call('capture_replay', replay_path='artifacts/material.replay.json', path='artifacts/material-prefix.png', tick=0, width=960, height=720)
        assert (ROOT/'artifacts/material-replay.png').read_bytes() == live
        assert (ROOT/'artifacts/material-prefix.png').read_bytes() == start
        checks.append('Back-lit transmission and mapped animation work; complete/prefix replay preserves native pixels and material channels')
        definition = {'id': 'plant', 'profile': {'id': 'profile', 'width': 1, 'height': 3}, 'depth': 0.3,
                      'segments': [2,3,1], 'subdivision_levels': 0, 'sampling_tolerance': 1/1024}
        def put(d):
            c.call('edit_plant', edit={'kind': 'put', 'definition': d,
                   'detail_policy': 'physical', 'animation_translation_policy': 'physical'})
        put(definition)
        base = c.call('inspect_mesh', asset='plant')['asset']['geometry']
        channel = f32([[p[0]+7,p[1]*2+3,p[2]*3] for p in base['positions']])
        edit('set_material_coordinates', asset='plant', coordinates=channel)
        grown = copy.deepcopy(definition); grown['profile']['height'] = 5
        put(grown)
        before = c.call('inspect_plant', id='plant')['plant']
        edit('set_material', asset='plant', material=material)
        after = c.call('inspect_plant', id='plant')['plant']
        expected = copy.deepcopy(before); expected['source']['material'] = material
        assert after == expected
        put(definition)
        assert c.call('inspect_mesh', asset='plant')['asset']['geometry']['material_coordinates'] == channel
        topology = c.call('inspect_mesh_topology', asset='plant')
        edit('subdivide', asset='plant', reference=topology['reference'], components=None)
        refined = c.call('inspect_mesh', asset='plant')['asset']
        assert refined['material'] == material
        assert len(refined['geometry']['material_coordinates']) == len(refined['geometry']['positions']) > len(channel)
        checks.append('Linked plant recipe metadata never rebases canonical geometry; channels roundtrip dimensions and interpolate in native subdivision')
        report = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/material-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary, args.window)
