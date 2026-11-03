#!/usr/bin/env python3
"""Reauthor one existing tree secondary as a native, editable second plant."""
import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path

from tree_study_geometry import add, cross, dot, mul, qmul, rotate, sub, unit

PROJECT = Path(__file__).resolve().parents[1]
ID = 'secondary-06'
INSTANCE = 'tree-secondary-06'
CHILDREN = {'crown-06', 'tree-secondary-twigs'}
ROOT = PROJECT / 'artifacts/tree-secondary-plant'


def inverse(q):
    return [-q[0], -q[1], -q[2], q[3]]


def point(matrix, p):
    return [sum(matrix[c][r] * p[c] for c in range(3)) + matrix[3][r]
            for r in range(3)]


def fit(source, tip):
    # Known five seven-vertex rings of this authored source, before the ring
    # loft's appended vertices. No nearest surface or general mesh recognition.
    g = source['geometry']
    assert len(g['positions']) == 119 and len(g['triangles']) == 234
    assert not source.get('rig') and not source.get('clips')
    rings = [g['positions'][i * 7:(i + 1) * 7] for i in range(5)]
    centers = [[sum(p[k] for p in ring) / 7 for k in range(3)] for ring in rings]
    radii = [sum(math.dist(p, c) for p in ring) / 7 for ring, c in zip(rings, centers)]
    assert math.dist(centers[0], [0, 0, 0]) < .001
    assert math.dist(centers[-1], tip) < .001
    height = math.sqrt(dot(tip, tip))
    direction = unit(tip)
    q = unit(cross([0, 1, 0], direction) + [1 + direction[1]])
    mid = mul(tip, .5)
    local = [rotate(inverse(q), sub(c, mid)) for c in centers]
    ats = [0] + [(p[1] + height / 2) / height for p in local[1:-1]] + [1]
    assert all(0 <= a < b <= 1 for a, b in zip(ats, ats[1:]))
    controls = [[0., 0.] for _ in range(4)]
    us = [3 * (1 - t) ** 2 * t for t in ats[1:-1]]
    vs = [3 * (1 - t) * t ** 2 for t in ats[1:-1]]
    uu, uv, vv = sum(u*u for u in us), sum(u*v for u, v in zip(us, vs)), sum(v*v for v in vs)
    determinant = uu*vv - uv*uv
    assert determinant > 1e-8
    for axis, component in enumerate([0, 2]):
        values = [p[component] for p in local[1:-1]]
        uy = sum(u*y for u, y in zip(us, values))
        vy = sum(v*y for v, y in zip(vs, values))
        controls[1][axis] = (uy*vv - vy*uv) / determinant
        controls[2][axis] = (vy*uu - uy*uv) / determinant
    errors = []
    for t, p in zip(ats, local):
        c = [3*(1-t)**2*t*controls[1][k] + 3*(1-t)*t*t*controls[2][k] for k in range(2)]
        errors.append(math.dist([c[0], height*(t-.5), c[1]], p))
    assert max(errors) < .015, errors
    definition = {
        'id': ID, 'profile': {'id': 'secondary06-profile', 'width': 2*radii[0], 'height': height},
        'depth': 2*radii[0], 'sampling_tolerance': .001, 'segments': [8, 6, 8],
        'trajectory': {'kind': 'cubic_bezier', 'id': 'secondary06-axis',
                       'control_points_xz': controls, 'surface_tolerance': .001},
        'sections': {'id': 'secondary06-sections', 'cross_section': 'ellipse', 'interpolation': 'pchip',
                     'stations': [{'id': f'secondary06-{i}', 'at': t,
                                   'width': r/radii[0], 'depth': r/radii[0]}
                                  for i, (t, r) in enumerate(zip(ats, radii))]}}
    return definition, q, mid, {'center_fit_errors_m': errors, 'height_m': height,
                                'source_radii_m': radii, 'source_ring_vertices': [list(range(i*7, (i+1)*7)) for i in range(5)]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', type=Path, default=PROJECT/'examples/native-tree-study.scene.json')
    args = parser.parse_args()
    for directory in ['examples', 'artifacts']:
        (ROOT/directory).mkdir(parents=True, exist_ok=True)
    data = args.input.read_bytes()
    (ROOT/'examples/input.scene.json').write_bytes(data)
    os.environ['IGNIS_VERIFY_ROOT'] = str(ROOT)
    from verify_lab import Client
    client = Client(PROJECT/'target/release/ignis', False)
    call = client.call

    def plant_edit(d):
        return {'kind': 'put', 'definition': d, 'detail_policy': 'physical',
                'animation_translation_policy': 'physical'}

    def asset():
        return call('inspect_mesh', asset=ID)['asset']

    def pose(id, tick):
        return call('inspect_mesh_pose', instance=id, tick=tick)['pose']

    def capture(name, width=1100, height=1000):
        path = f'artifacts/{name}.png'
        call('capture', path=path, width=width, height=height)
        return (ROOT/path).read_bytes()

    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                     'clientInfo': {'name': 'tree-secondary-plant', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/input.scene.json')
        initial = call('inspect')['scene']
        assert len(initial['plants']['entries']) == 1
        old = next(a for a in initial['meshes']['assets'] if a['id'] == ID)
        children = [i for i in initial['meshes']['instances'] if i['id'] in CHILDREN]
        assert len(children) == 2
        crown = next(i for i in children if i['id'] == 'crown-06')
        definition, rotation, midpoint, report = fit(old, crown['position'])
        height = definition['profile']['height']
        before = call('inspect')
        preview = call('preview_scene_edits', edits=[
            {'op': 'mesh', 'edit': {'kind': 'remove_asset', 'id': ID}},
            {'op': 'plant', 'edit': plant_edit(definition)}])
        assert call('inspect') == before
        base = next(a for a in preview['scene']['meshes']['assets'] if a['id'] == ID)['geometry']
        # Author a small, localised new relief; this is not a claim of transferring
        # the irregularity of the previous procedural source.
        surface_point = min(base['positions'], key=lambda p: abs(p[1]) + abs(p[0]) - .2*p[2])
        rig = {'bones': [
            {'name': 'base', 'parent': None, 'translation': [0, -height/2, 0], 'rotation': [0, 0, 0, 1]},
            {'name': 'tip', 'parent': 0, 'translation': [0, height, 0], 'rotation': [0, 0, 0, 1]}],
            'weights': [{'joints': [0, 0, 0, 0], 'weights': [1, 0, 0, 0]} for _ in base['positions']]}
        instance = copy.deepcopy(next(i for i in initial['meshes']['instances'] if i['id'] == INSTANCE))
        assert instance['position'] == [0, 0, 0] and instance['rotation'] == [0, 0, 0, 1] and instance['scale'] == 1
        instance['position'], instance['rotation'] = midpoint, rotation
        coordinates = [add(midpoint, rotate(rotation, p)) for p in base['positions']]
        batch = [
            {'op': 'mesh', 'edit': {'kind': 'remove_asset', 'id': ID}},
            {'op': 'plant', 'edit': plant_edit(definition)},
            {'op': 'mesh', 'edit': {'kind': 'set_shading', 'asset': ID, 'shading': 'smooth'}},
            {'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': ID, 'material': old['material']}},
            {'op': 'mesh', 'edit': {'kind': 'set_material_coordinates', 'asset': ID, 'coordinates': coordinates}},
            {'op': 'mesh', 'edit': {'kind': 'sculpt', 'asset': ID, 'stroke': {
                'kind': 'grab', 'centers': [surface_point], 'radius': .035, 'strength': .001, 'direction': [0, 0, 1]}}},
            {'op': 'mesh', 'edit': {'kind': 'set_rig', 'asset': ID, 'rig': rig}},
            {'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': instance}}]
        for original in children:
            child = copy.deepcopy(original)
            assert child['attachment'] == {'parent': INSTANCE}
            child['attachment']['bone'] = 'tip'
            child['position'] = rotate(inverse(rotation), sub(child['position'], crown['position']))
            child['rotation'] = qmul(inverse(rotation), child['rotation'])
            batch.append({'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': child}})
        poses = {(i['id'], tick): pose(i['id'], tick) for i in initial['meshes']['instances'] for tick in [0, 60, 240]}
        before = call('inspect')
        predicted = call('preview_scene_edits', edits=batch)['scene']
        assert call('inspect') == before
        call('edit_scene', edits=batch)
        actual = call('inspect')['scene']
        assert actual == predicted
        assert len(actual['plants']['entries']) == 2
        assert next(e for e in actual['plants']['entries'] if e['definition']['id'] != ID) == initial['plants']['entries'][0]
        assert [a for a in actual['meshes']['assets'] if a['id'] != ID] == [a for a in initial['meshes']['assets'] if a['id'] != ID]
        assert [i for i in actual['meshes']['instances'] if i['id'] not in CHILDREN | {INSTANCE}] == [i for i in initial['meshes']['instances'] if i['id'] not in CHILDREN | {INSTANCE}]
        errors = {}
        for (id, tick), old_pose in poses.items():
            if id == INSTANCE:
                continue
            current = pose(id, tick)
            error = max(math.dist(p, q) for p, q in zip(old_pose['positions'], current['positions']))
            assert len(current['positions']) == len(old_pose['positions'])
            if id in CHILDREN:
                assert error < 3e-6, (id, tick, error)
            else:
                assert current == old_pose, (id, tick)
            errors[f'{id}:{tick}'] = error
        call('undo')
        assert call('inspect')['scene'] == initial
        call('redo')
        assert call('inspect')['scene'] == actual
        assert call('inspect_mesh_boundaries', asset=ID)['boundaries'] == []
        assert len(call('inspect_mesh_islands', asset=ID)['topology']['islands']) == 1
        call('save_scene', path='examples/after.scene.json')
        canonical = call('inspect_plant', id=ID)['plant']
        (ROOT/'examples/secondary.plant.json').write_text(json.dumps(canonical['definition'], indent=2)+'\n')
        original_asset = asset()
        definition = canonical['definition']
        assert call('inspect_plant', id=ID)['report']['max_detail_displacement'] > .0009
        altered = copy.deepcopy(definition)
        altered['trajectory']['control_points_xz'][1][0] += .04
        state = call('inspect')
        call('preview_plant', edit=plant_edit(altered))
        assert call('inspect') == state
        for _ in range(2):
            call('edit_plant', edit=plant_edit(altered))
            assert asset()['geometry']['positions'] != original_asset['geometry']['positions']
            assert asset()['rig'] == original_asset['rig']
            assert call('inspect_plant', id=ID)['plant']['source'] == canonical['source']
            assert call('inspect_plant', id=ID)['plant']['reference'] == canonical['reference']
            for id in CHILDREN:
                assert max(math.dist(p, q) for p, q in zip(pose(id, 60)['positions'], poses[id, 60]['positions'])) < 3e-6
            call('edit_plant', edit=plant_edit(definition))
            assert asset() == original_asset
            assert call('inspect')['scene'] == actual
        # Move the endpoint explicitly; the named tip bone now carries the same
        # crown and twig assets. Compare against the native evaluated bone delta.
        moved = copy.deepcopy(definition)
        moved['trajectory']['control_points_xz'][3][0] += .06
        old_tip = pose(INSTANCE, 60)['joints'][1][3][:3]
        call('edit_plant', edit=plant_edit(moved))
        new_pose = pose(INSTANCE, 60)
        new_tip = new_pose['joints'][1][3][:3]
        # joints are global pose matrices in asset space; world_transform maps
        # their origin into the parent's current animated frame.
        world = new_pose['world_transform']
        delta = sub(point(world, new_tip), point(world, old_tip))
        assert math.sqrt(dot(delta, delta)) > .05
        follow_error = 0.
        for id in CHILDREN:
            current = pose(id, 60)
            baseline = poses[id, 60]
            error = max(math.dist(sub(p, q), delta) for p, q in zip(current['positions'], baseline['positions']))
            follow_error = max(follow_error, error)
            assert error < 3e-6, (id, error, delta)
        call('edit_plant', edit=plant_edit(definition))
        assert call('inspect')['scene'] == actual
        bad = copy.deepcopy(definition)
        bad['trajectory']['control_points_xz'][1][0] = 100
        state = call('inspect')
        call('edit_plant', edit=plant_edit(bad), expected_error=True)
        assert call('inspect') == state
        # A denser target uses the same source, skeleton, appearance and material
        # field, then returns exactly to A.
        dense = copy.deepcopy(definition)
        dense['segments'] = [10, 8, 10]
        call('edit_plant', edit=plant_edit(dense))
        assert len(asset()['geometry']['positions']) > len(original_asset['geometry']['positions'])
        assert call('inspect_plant', id=ID)['plant']['source'] == canonical['source']
        call('edit_plant', edit=plant_edit(definition))
        assert call('inspect')['scene'] == actual
        call('load_scene', path='examples/after.scene.json')
        assert call('inspect')['scene'] == actual
        world = poses[INSTANCE, 0]['world_transform']
        target = point(world, mul(crown['position'], .5))
        camera = {'position': add(target, [2, 1.0, 3]), 'target': target, 'fov_degrees': 20}
        call('edit_plant', edit=plant_edit(altered))
        call('save_scene', path='examples/curve-edited.scene.json')
        call('set_camera', camera=camera)
        call('step', ticks=60)
        capture('curve-edited-secondary')
        for label, path in [('before', 'examples/input.scene.json'), ('after', 'examples/after.scene.json')]:
            for view, cam in [('whole', initial['camera']), ('secondary', camera)]:
                call('load_scene', path=path)
                call('set_camera', camera=cam)
                call('step', ticks=60)
                capture(f'{label}-{view}')
            call('load_scene', path=path)
            neutral = [{'op': 'mesh', 'edit': {'kind': 'remove_instance', 'id': i['id']}}
                       for i in initial['meshes']['instances'] if i['id'].startswith('crown-')]
            neutral += [{'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': a['id'], 'material': {}}}
                        for a in initial['meshes']['assets'] if a['id'] in {'wood', 'branch-editable-1', ID, 'secondary-twigs'}]
            call('edit_scene', edits=neutral)
            call('set_light', light={'direction': [-.8, -1, -.4], 'ambient': .45})
            call('set_camera', camera=camera)
            call('step', ticks=60)
            capture(f'{label}-neutral-secondary')
        call('load_scene', path='examples/after.scene.json')
        call('set_camera', camera=camera)
        prefix = capture('prefix')
        call('step', ticks=60)
        live = capture('live')
        assert live == capture('repeat')
        call('save_replay', path='artifacts/tree.replay.json')
        assert call('run_replay', path='artifacts/tree.replay.json')['meshes_matched']
        for name, tick, expected in [('replay', 60, live), ('replay-prefix', 0, prefix)]:
            call('capture_replay', replay_path='artifacts/tree.replay.json', path=f'artifacts/{name}.png',
                 tick=tick, width=1100, height=1000)
            assert (ROOT/f'artifacts/{name}.png').read_bytes() == expected
        call('reset')
        assert capture('reset') == prefix
        report.update(passed=True, build_id=call('inspect')['build_id'], input_sha256=hashlib.sha256(data).hexdigest(),
                      input_path=str(args.input.resolve()), plant_count=2, source_vertices=119,
                      target_vertices=len(original_asset['geometry']['positions']),
                      target_triangles=len(original_asset['geometry']['triangles']),
                      conversion_pose_errors_m=errors, tip_follow_error_m=follow_error,
                      max_detail_displacement_m=call('inspect_plant', id=ID)['report']['max_detail_displacement'],
                      first_plant_other_assets_exact=True, edit_cycles_source_rig_undo_load_replay_exact=True,
                      native_window=False, root_join_connected=False, tree_realism_complete=False)
        report['vertices'] = sum(len(a['geometry']['positions']) for a in actual['meshes']['assets'])
        report['triangles'] = sum(len(a['geometry']['triangles']) for a in actual['meshes']['assets'])
        assets = {a['id']: a for a in actual['meshes']['assets']}
        report['draw_triangles'] = sum(len(assets[i['asset']]['geometry']['triangles']) for i in actual['meshes']['instances'])
        (ROOT/'artifacts/report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({k: v for k, v in report.items() if k != 'conversion_pose_errors_m'}))
    finally:
        client.close()


if __name__ == '__main__':
    main()
