"""Native editable secondary primary branch for the shared tree experiment."""
import copy
import math
from tree_study_geometry import bezier


def plan(call, controls, radius):
    parameters = [0, 0.15, 0.30, 0.44, 0.58, 0.72, 0.86, 1]
    dense = bezier(controls, 101)
    points = [dense[round(t*100)] for t in parameters]
    start, end = points[0], points[-1]
    height = end[1]-start[1]
    base_radius = radius+0.004
    stations = [{'id': f'branch1-{round(t*100):03}', 'at': (p[1]-start[1])/height,
                 'offset': [p[0]-start[0], p[2]-start[2]],
                 'width': (radius*(1-t)**1.2+0.004)/base_radius,
                 'depth': (radius*(1-t)**1.2+0.004)/base_radius}
                for t, p in zip(parameters, points)]
    definition = {'id': 'branch-editable-1',
                  'profile': {'id': 'branch1-profile', 'width': base_radius*2, 'height': height},
                  'depth': base_radius*2,
                  'sections': {'id': 'branch1-sections', 'cross_section': 'ellipse', 'stations': stations},
                  'segments': [4, 2, 4], 'subdivision_levels': 0, 'sampling_tolerance': 0.001}
    instance = {'id': 'tree-branch-editable-1', 'asset': definition['id'],
                'position': [start[0], (start[1]+end[1])/2, start[2]],
                'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.15, 0.13, 0.10], 'playback': None}
    before = call('inspect')
    preview = put(call, definition, 'preview_plant')
    assert call('inspect') == before
    asset = next(a for a in preview['assets'] if a['id'] == definition['id'])
    return {'definition': definition, 'instance': instance, 'asset': asset,
            'anchors': points[2:], 'control_id': stations[1]['id'],
            'preview_vertices': len(asset['geometry']['positions']),
            'preview_triangles': len(asset['geometry']['triangles'])}


def put(call, definition, operation='edit_plant'):
    return call(operation, edit={'kind': 'put', 'definition': definition,
                                'detail_policy': 'physical', 'animation_translation_policy': 'physical'})


def exercise(call, branch, capture, camera):
    id = branch['definition']['id']
    a = call('inspect_mesh', asset=id)['asset']
    source = call('inspect_plant', id=id)['plant']['source']
    current = call('inspect')['scene']
    others = [v for v in current['meshes']['assets'] if v['id'] != id]
    instances = current['meshes']['instances']
    leaf_pose = call('inspect_mesh_pose', instance='crown-00')['pose']
    target = branch['anchors'][0]
    call('set_camera', camera={'position': [target[0]+2.0, target[1]+0.55, target[2]+3.0],
                              'target': target, 'fov_degrees': 30})
    capture('editable-branch-A', 1000, 900)
    changed = copy.deepcopy(branch['definition'])
    control = next(s for s in changed['sections']['stations'] if s['id'] == branch['control_id'])
    control['offset'][0] += 0.18
    control['offset'][1] -= 0.10
    before = call('inspect')
    put(call, changed, 'preview_plant')
    assert call('inspect') == before
    put(call, changed)
    b = call('inspect_mesh', asset=id)['asset']
    assert b['geometry']['positions'] != a['geometry']['positions']
    assert b['geometry']['material_coordinates'] == a['geometry']['material_coordinates']
    assert b['material'] == a['material'] and b['shading'] == a['shading']
    assert call('inspect_plant', id=id)['plant']['source'] == source
    scene_b = call('inspect')['scene']
    assert [v for v in scene_b['meshes']['assets'] if v['id'] != id] == others
    assert scene_b['meshes']['instances'] == instances
    assert call('inspect_mesh_pose', instance='crown-00')['pose'] == leaf_pose
    anchor_errors = []
    for p in branch['anchors']:
        local_y = p[1]-branch['instance']['position'][1]
        ring = [v for v in b['geometry']['positions'] if abs(v[1]-local_y) < 0.000002]
        assert ring
        center = [sum(v[i] for v in ring)/len(ring)+branch['instance']['position'][i] for i in range(3)]
        anchor_errors.append(math.dist(center, p))
    assert max(anchor_errors) < 0.000002, anchor_errors
    capture('editable-branch-B', 1000, 900)
    call('undo'); assert call('inspect_mesh', asset=id)['asset'] == a
    call('redo'); assert call('inspect_mesh', asset=id)['asset'] == b
    for _ in range(2):
        put(call, branch['definition']); assert call('inspect_mesh', asset=id)['asset'] == a
        put(call, changed); assert call('inspect_mesh', asset=id)['asset'] == b
    put(call, branch['definition'])
    assert call('inspect_mesh', asset=id)['asset'] == a
    assert call('inspect_plant', id=id)['plant']['source'] == source
    call('set_camera', camera=camera)
    return {'id': id, 'definition': branch['definition'],
            'vertices': branch['preview_vertices'], 'triangles': branch['preview_triangles'],
            'anchor_errors': anchor_errors, 'canonical_source_preserved': True,
            'other_assets_and_instances_unchanged': True, 'material_coordinates_preserved': True,
            'leaf_pose_at_edit_tick_preserved': True,
            'reversible_edit': True, 'wood_static': True, 'shared_fork_unchanged': True}
