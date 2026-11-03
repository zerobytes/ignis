#!/usr/bin/env python3
"""Common relief tools through real MCP, in an isolated process with no native window."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT

RADIUS = 6_371_000.0
OUT = ROOT / 'artifacts/surface-program'


def position(x, y=0, z=0):
    values = [x, y, z]
    cells = [math.floor(v / 1024) for v in values]
    return {'cell': [str(c) for c in cells], 'offset': [v-c*1024 for v, c in zip(values, cells)]}


def unit(x, z):
    distance = math.hypot(x, z)
    if distance == 0:
        return [0, 1, 0]
    angle = distance / RADIUS
    return [x/distance*math.sin(angle), math.cos(angle), z/distance*math.sin(angle)]


def location(x, z, spherical=False):
    return position(*(v*RADIUS for v in unit(x, z))) if spherical else position(x, 0, z)


def program(spherical=False):
    # Exactly the same recipe, controls and operators. Only authored locations use the domain.
    anchor = lambda x, z: location(x, z, spherical)
    nodes = [
        {'kind': 'noise', 'seed': 187, 'period_meters': 4096, 'octaves': 5,
         'persistence': .5, 'amplitude_meters': 1, 'style': 'smooth'},
        {'kind': 'remap', 'input': 0, 'knots': [
            {'x': -1, 'value': -200, 'derivative': 0},
            {'x': -.2, 'value': -50, 'derivative': 60},
            {'x': .2, 'value': 50, 'derivative': 60},
            {'x': 1, 'value': 180, 'derivative': 0}]},
        {'kind': 'curve', 'points': [anchor(-400, -150), anchor(-150, 80), anchor(60, 20), anchor(350, 200)],
         'width_meters': 180, 'height_meters': 220, 'left_exponent': 1.2, 'right_exponent': 3, 'roundness': .15},
        {'kind': 'curve', 'points': [anchor(-150, 80), anchor(0, 240), anchor(180, 350)],
         'width_meters': 110, 'height_meters': 150, 'left_exponent': 1.4, 'right_exponent': 3, 'roundness': .2},
        {'kind': 'smooth_maximum', 'a': 2, 'b': 3, 'transition_meters': 30},
        {'kind': 'noise', 'seed': 190, 'period_meters': 256, 'octaves': 4,
         'persistence': .5, 'amplitude_meters': 15, 'style': 'ridged'},
        {'kind': 'add', 'a': 4, 'b': 5},
        {'kind': 'add', 'a': 1, 'b': 6},
        {'kind': 'channel', 'input': 7,
         'points': [anchor(-400, 250), anchor(-180, 130), anchor(0, -30), anchor(250, -200), anchor(400, -260)],
         'width_meters': 35, 'bed_start_meters': 30, 'bed_end_meters': -20, 'blend': 1},
        {'kind': 'level', 'input': 8, 'center': anchor(-300, -250), 'radius_meters': 120,
         'height_meters': 80, 'slope': [.01, .03], 'blend': 1},
        {'kind': 'clamp', 'input': 9, 'minimum': -350, 'maximum': 400}]
    return {'version': 1, 'nodes': nodes, 'output': 10, 'range_meters': [-501, 501]}


def documents():
    common = {'space_version': 1, 'id': 'common_geography', 'revision': 1}
    flat = dict(common, version=3, datum=position(0), base={'kind': 'constant', 'height_meters': 0},
                operations=[{'kind': 'surface', 'program': program()}])
    globe = dict(common, version=2, center=position(0), diameter_meters=RADIUS*2,
                 surface_gravity_mps2=9.81, base_height_meters=0,
                 operations=[{'kind': 'surface', 'program': program(True)}])
    return flat, globe


def verify(binary, captures=False):
    OUT.mkdir(parents=True, exist_ok=True)
    c = Client(binary, log_path=OUT / 'runtime.log')
    report = {'native_window': False}
    flat, globe = documents()
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                  'clientInfo': {'name': 'common-surface', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result'])
        assert all(x in schema for x in ['range_meters', 'left_exponent', 'right_exponent', 'roundness'])
        before = c.call('inspect')
        report['build_id'] = before['build_id']
        for name, doc in [('terrain', flat), ('planet', globe)]:
            draft = {k: doc['operations'][0]['program'][k] for k in ['nodes', 'output']}
            prepared = c.call(f'query_{name}', query={'kind': 'prepare_surface', 'draft': draft})['program']
            doc['operations'][0]['program'] = prepared
            assert prepared['range_meters'][0] < prepared['range_meters'][1]
        samples = [(x, z) for z in range(-400, 401, 40) for x in range(-400, 401, 40)]
        a = c.call('query_terrain', query={'kind': 'sample', 'document': flat,
                    'points': [location(x, z) for x, z in samples]})['samples']
        b = c.call('query_planet', query={'kind': 'sample', 'document': globe,
                    'directions': [unit(x, z) for x, z in samples]})['samples']
        assert all(-501 <= s['height_meters'] <= 501 for s in a+b)
        assert max(s['height_meters'] for s in a)-min(s['height_meters'] for s in a) > 100
        assert max(s['height_meters'] for s in b)-min(s['height_meters'] for s in b) > 100
        for name, doc in [('terrain', flat), ('planet', globe)]:
            path = f'artifacts/surface-program/source.ignis-{name}'
            c.call(f'save_{name}', path=path, document=doc)
            loaded = c.call(f'load_{name}', path=path)['document']
            expected = dict(doc, water_level_meters=None) if name == 'terrain' else doc
            assert loaded == expected
            bad = copy.deepcopy(doc)
            bad['operations'][0]['program']['range_meters'] = [0, 1]
            c.call(f'query_{name}', query={'kind': 'inspect', 'document': bad}, expected_error=True)
        # Known masks/metric width: both domains are tested without differing global noise slices.
        for name, doc in [('terrain', flat), ('planet', globe)]:
            pure = copy.deepcopy(doc)
            curve = copy.deepcopy(doc['operations'][0]['program']['nodes'][2])
            curve.update(points=[location(-100, 0, name == 'planet'), location(100, 0, name == 'planet')],
                         width_meters=100, height_meters=80, left_exponent=1, right_exponent=3, roundness=0)
            pure['operations'][0]['program'] = {'version': 1, 'nodes': [curve], 'output': 0, 'range_meters': [-1, 81]}
            kw = {'points': [location(0, z) for z in [-120, -75, 0, 75, 120]]} if name == 'terrain' else {
                'directions': [unit(0, z) for z in [-120, -75, 0, 75, 120]]}
            result = c.call(f'query_{name}', query=dict(kind='sample', document=pure, **kw))['samples']
            heights = [s['height_meters'] for s in result]
            expected = [0, 20, 80, 1.25, 0]
            assert max(abs(x-y) for x, y in zip(heights, expected)) < 1e-5, heights
        for name, doc in [('terrain', flat), ('planet', globe)]:
            existing = copy.deepcopy(doc)
            if name == 'terrain':
                existing['base']['height_meters'] = 20
            else:
                existing['base_height_meters'] = 20
            draft = {'nodes': [
                {'kind': 'input', 'minimum_meters': 20, 'maximum_meters': 20},
                {'kind': 'level', 'input': 0, 'center': location(0, 0, name == 'planet'),
                 'radius_meters': 100, 'height_meters': 7, 'slope': [0, 0], 'blend': 1}], 'output': 1}
            existing['operations'][0]['program'] = c.call(f'query_{name}', query={
                'kind': 'prepare_surface', 'draft': draft})['program']
            locations = {'points': [position(0)]} if name == 'terrain' else {'directions': [[0, 1, 0]]}
            leveled = c.call(f'query_{name}', query=dict(kind='sample', document=existing, **locations))
            assert leveled['samples'][0]['height_meters'] == 7
            bad_input = copy.deepcopy(existing)
            if name == 'terrain':
                bad_input['base']['height_meters'] = 21
            else:
                bad_input['base_height_meters'] = 21
            c.call(f'query_{name}', query={'kind': 'inspect', 'document': bad_input}, expected_error=True)
        assert c.call('inspect') == before
        report['pure_queries_formats_invalid_certificates_metric_asymmetry'] = True
        report['input_height_leveling_and_forged_input_rejection'] = True
        # Publication/collision/undo follow the existing transactional engine path.
        c.call('generate', recipe={'width': 12, 'depth': 12, 'seed': 1, 'obstacles': 0})
        state = c.call('inspect')
        c.call('edit_scene', edits=[{'op': 'remove', 'id': e['id']} for e in state['scene']['entities']])
        original = c.call('inspect')['scene']
        publication = {'document': flat, 'patch': {'origin': position(-512, 0, -512), 'cells': [127, 127], 'spacing_meters': 8},
                       'stride': 1, 'asset': 'common_terrain', 'instance': 'common_terrain', 'frame_origin': position(0),
                       'color': [.38, .46, .32], 'collision': True}
        bad = copy.deepcopy(publication)
        bad['instance'] = ''
        c.call('publish_terrain', publication=bad, expected_error=True)
        assert c.call('inspect')['scene'] == original
        result = c.call('publish_terrain', publication=publication)
        assert result['terrain_format'] == 3 and not result['visual_only']
        published = c.call('inspect')['scene']
        c.call('undo')
        assert c.call('inspect')['scene'] == original
        c.call('redo')
        assert c.call('inspect')['scene'] == published
        if captures:
            light = published['light']
            light.update(direction=[.5, -.7, -.3], ambient=.35, atmosphere=None, shadow=None)
            c.call('set_light', light=light)
            c.call('set_camera', camera={'position': [1000, 650, 900], 'target': [0, 30, 0], 'fov_degrees': 60})
            c.call('capture', path='artifacts/surface-program/plane.png', width=960, height=720)
        c.call('save_scene', path='artifacts/surface-program/plane.scene.json')
        c.call('save_replay', path='artifacts/surface-program/plane.replay.json')
        assert c.call('run_replay', path='artifacts/surface-program/plane.replay.json')['meshes_matched']
        report['terrain_publication_collision_undo_replay'] = True
        # Real planetary compilation on all four neighboring tiles around the authored anchor.
        c.call('generate', recipe={'width': 12, 'depth': 12, 'seed': 1, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        c.call('edit_scene', edits=[{'op': 'remove', 'id': e['id']} for e in entities])
        frame = {'kind': 'surface' , 'pose': {'direction': [0, 1, 0], 'forward': [0, 0, -1], 'altitude_meters': 0}}
        tiles = {}
        for x in [16383, 16384]:
            for y in [16383, 16384]:
                name = f'globe_{x}_{y}'
                result = c.call('publish_planet', publication={'document': globe,
                    'area': {'kind': 'tile', 'face': 'positive_y', 'level': 15, 'tile': [x, y], 'subdivisions': 64},
                    'frame': frame, 'asset': name, 'instance': name, 'color': [.38, .46, .32], 'collision': True})
                assert not result['visual_only']
                tiles[x, y] = c.call('inspect_mesh', asset=name)['asset']['geometry']['positions']
        for y in [16383, 16384]:
            a = {tuple(p) for p in tiles[16383, y] if p[0] == 0}
            b = {tuple(p) for p in tiles[16384, y] if p[0] == 0}
            assert len(a) == 65 and a == b
        for x in [16383, 16384]:
            a = {tuple(p) for p in tiles[x, 16383] if p[2] == 0}
            b = {tuple(p) for p in tiles[x, 16384] if p[2] == 0}
            assert len(a) == 65 and a == b
        report['planet_neighbor_positions_exact'] = True
        report['planet_publication_collision'] = True
        if captures:
            c.call('set_light', light=light)
            c.call('set_camera', camera={'position': [1000, 650, 900], 'target': [0, 30, 0], 'fov_degrees': 60})
            c.call('capture', path='artifacts/surface-program/planet.png', width=960, height=720)
        c.call('save_scene', path='artifacts/surface-program/planet.scene.json')
        report.update(passed=True, groups_passed=4, captures=captures)
        (OUT / 'verification.json').write_text(json.dumps(report, indent=2))
        (OUT / 'plane-source.json').write_text(json.dumps(flat, indent=2))
        (OUT / 'planet-source.json').write_text(json.dumps(globe, indent=2))
        print(json.dumps(report))
        return report
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    parser.add_argument('--captures', action='store_true')
    args = parser.parse_args()
    verify(args.binary, args.captures)
