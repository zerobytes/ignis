#!/usr/bin/env python3
"""Planet source, closed surface, local curvature and geometric circumnavigation via MCP.
Never launches a native window. This does not claim physical character circumnavigation.
"""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window=False):
    out = ROOT / 'artifacts/planet'
    out.mkdir(parents=True, exist_ok=True)
    c = Client(binary, log_path=out / 'runtime.log')
    report = {}
    recipe = json.loads((ROOT / 'examples/planet.recipe.json').read_text())
    def query(kind, document=recipe, **kw):
        return c.call('query_planet', query={'kind': kind, 'document': document, **kw})
    def publish(document, instance, frame, subdivisions=32, collision=False, color=(.23, .43, .17), area=None):
        return c.call('publish_planet', publication={'document': document, 'asset': instance, 'instance': instance,
                      'area': area or {'kind': 'globe', 'subdivisions': subdivisions}, 'frame': frame,
                      'color': color, 'collision': collision})
    def capture(name, position, target):
        c.call('set_camera', camera={'position': position, 'target': target, 'fov_degrees': 65})
        c.call('capture', path=f'artifacts/planet/{name}.png', width=1280, height=800)
    def clear():
        c.call('generate', recipe={'width': 12, 'depth': 12, 'seed': 1, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        c.call('edit_scene', edits=[{'op': 'remove', 'id': e['id']} for e in entities])
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                  'clientInfo': {'name': 'planet-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result'])
        assert all(x in schema for x in ('query_planet', 'publish_planet', 'save_planet', 'load_planet', 'diameter_meters'))
        before = c.call('inspect')
        invalid = copy.deepcopy(recipe)
        invalid['diameter_meters'] = 0
        c.call('query_planet', query={'kind': 'inspect', 'document': invalid}, expected_error=True)
        c.call('query_planet', query={'kind': 'sample', 'document': recipe,
               'directions': [[0, 1, 0]] * 4097}, expected_error=True)
        report['build_id'] = before['build_id']
        report['source'] = query('inspect')
        assert report['source']['radius_meters'] == 400
        c.call('save_planet', path='artifacts/planet/source.ignis-planet', document=recipe)
        assert c.call('load_planet', path='artifacts/planet/source.ignis-planet')['document'] == recipe
        broken = bytearray((out / 'source.ignis-planet').read_bytes())
        broken[-1] ^= 1
        (out / 'corrupt.ignis-planet').write_bytes(broken)
        c.call('load_planet', path='artifacts/planet/corrupt.ignis-planet', expected_error=True)
        (out / 'corrupt.ignis-planet').unlink()
        # Test seam-equivalent directions without latitude/longitude addressing.
        samples = query('sample', directions=[[1, 1, 0], [0.5, 0.5, 0], [0, 1, 0], [0, -1, 0]])['samples']
        assert samples[0] == samples[1]
        assert samples[2]['gravity_mps2'][1] < 0 < samples[3]['gravity_mps2'][1]
        assert c.call('inspect') == before
        report['source_and_pure_queries'] = True
        # Heading survives a complete great circle through both poles, step by step.
        start = {'direction': [0, 1, 0], 'forward': [0, 0, -1], 'altitude_meters': 0}
        pose = copy.deepcopy(start)
        circumference = math.pi * recipe['diameter_meters']
        path = []
        for i in range(128):
            result = query('travel', pose=pose, distance_meters=circumference / 128)
            assert result['geometric_only'] and not result['collision_checked']
            pose = result['pose']
            path.append(pose)
        error = math.dist(pose['direction'], start['direction']) * 400
        assert error < 1e-8
        report['circumnavigation'] = {'steps': 128, 'reference_distance_meters': circumference,
                                     'closure_error_meters': error, 'physical_walk': False, 'poses': path}
        # Invalid publication after successful compilation cannot mutate the scene.
        clear()
        original = c.call('inspect')['scene']
        bad = {'document': recipe, 'asset': 'planet_land', 'instance': '', 'color': [.2, .5, .2],
               'area': {'kind': 'globe', 'subdivisions': 8}, 'frame': {'kind': 'centered'}}
        c.call('publish_planet', publication=bad, expected_error=True)
        assert c.call('inspect')['scene'] == original
        report['globe'] = publish(recipe, 'planet_land', {'kind': 'centered'}, subdivisions=64, collision=True)
        c.call('undo')
        assert c.call('inspect')['scene'] == original
        c.call('redo')
        light = c.call('inspect')['scene']['light']
        light['direction'] = [0.5, -0.7, -0.3]
        light['ambient'] = .45
        light['atmosphere'] = None
        light['shadow'] = None
        c.call('set_light', light=light)
        capture('orbit-terrain', [880, 630, 960], [0, 0, 0])
        ocean = copy.deepcopy(recipe)
        ocean.update(id='reference_water_surface', base_height_meters=0, operations=[])
        publish(ocean, 'ocean_reference', {'kind': 'centered'}, subdivisions=64, color=(.065, .25, .40))
        light = c.call('inspect')['scene']['light']
        light['direction'] = [0.5, -0.7, -0.3]
        light['ambient'] = .45
        light['atmosphere'] = None
        light['shadow'] = None
        c.call('set_light', light=light)
        capture('orbit', [880, 630, 960], [0, 0, 0])
        c.call('save_scene', path='artifacts/planet/orbit.scene.json')
        c.call('step', ticks=2)
        c.call('save_replay', path='artifacts/planet/globe.replay.json')
        assert c.call('run_replay', path='artifacts/planet/globe.replay.json')['meshes_matched']
        c.call('save_portable_save', path='artifacts/planet/globe.ignis-portable')
        c.call('load_portable_save', path='artifacts/planet/globe.ignis-portable')
        c.call('capture', path='artifacts/planet/orbit-restored.png', width=1280, height=800)
        assert (out / 'orbit.png').read_bytes() == (out / 'orbit-restored.png').read_bytes()
        report['publication_undo_replay_portable'] = True
        # Near-surface sphere actually falls away at the horizon. Local frame +Y is outward.
        clear()
        flat = copy.deepcopy(ocean)
        flat['id'] = 'curvature_reference'
        frame = {'kind': 'surface', 'pose': start}
        report['surface'] = publish(flat, 'curved_surface', frame, subdivisions=64, color=(.24, .46, .30))
        c.call('set_light', light=light)
        capture('horizon-eye', [0, 2, 0], [0, 2, -100])
        capture('horizon-high', [0, 60, 0], [0, 20, -200])
        c.call('save_scene', path='artifacts/planet/curvature.scene.json')
        report['horizon_2m'] = query('horizon', document=flat, altitude_meters=2)
        report['horizon_60m'] = query('horizon', document=flat, altitude_meters=60)
        # Earth-scale SOURCE; publish a ~3km patch near the pole, not the full planet in f32.
        clear()
        earth = copy.deepcopy(flat)
        earth.update(id='earth_scale', diameter_meters=12_742_000)
        earth_area = {'kind': 'tile', 'face': 'positive_y', 'level': 12, 'tile': [2048, 2048], 'subdivisions': 64}
        report['earth_local_patch'] = publish(earth, 'earth_patch', frame, area=earth_area, collision=True)
        assert report['earth_local_patch']['planet']['maximum_projection_error_meters'] < .001
        c.call('set_light', light=light)
        capture('earth-patch', [1800, 500, -400], [1500, -1, -1800])
        c.call('save_scene', path='artifacts/planet/earth-patch.scene.json')
        report.update(groups_passed=4, checks=4, passed=True, native_window=False)
        (out / 'verification.json').write_text(json.dumps(report, indent=2))
        print(json.dumps({k: v for k, v in report.items() if k in ('build_id', 'checks', 'passed', 'native_window', 'groups_passed', 'source_and_pure_queries', 'publication_undo_replay_portable')}))
    finally:
        c.close()
    return report

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    p.add_argument('--window', action='store_true', help='Accepted for runner compatibility; planet captures remain offscreen')
    args = p.parse_args()
    verify(args.binary, args.window)
