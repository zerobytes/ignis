#!/usr/bin/env python3
"""Author generic material details on the saved tree; fixed-view native comparisons."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import shutil
from demo_tree_authoring import PROJECT, OUTPUT, CAMERA, LIGHT
from demo_tree_smoothing import run as finish
from verify_lab import Client


def leaf_material():
    strokes = [{'start': [-0.025, 0.015], 'end': [-0.025, 0.985], 'widths': [0.017, 0.004]}]
    for side in [-1, 1]:
        for j, (z, reach) in enumerate([(0.14, 0.56), (0.28, 0.84), (0.43, 0.88), (0.58, 0.73), (0.73, 0.47)]):
            z += (0.018 if side > 0 else 0)
            start = [-0.025, z]
            middle = [side*reach*0.50, z+0.055]
            end = [side*reach, z+0.18]
            strokes += [{'start': start, 'end': middle, 'widths': [0.008, 0.005]},
                        {'start': middle, 'end': end, 'widths': [0.005, 0.0015]},
                        {'start': middle, 'end': [side*reach*0.76, z+0.21], 'widths': [0.003, 0.001]}]
    return {'pattern': {'frequency': [9, 1, 24], 'low': [0.82, 0.88, 0.65], 'high': [1.22, 1.14, 1.02],
            'bump': 0.00008, 'warp': 0.4, 'octaves': 3, 'seed': 29, 'contrast': 1.1, 'kind': 'noise'},
            'transmission': 0.42, 'detail': {'plane': 'xz', 'tint': [1.35, 1.24, 0.80], 'opacity': 0.55, 'bump': 0.00016, 'segments': strokes}}


def bark_material():
    strokes = []
    # General polylines in the rest XY plane; not a special shader pattern.
    for k in range(8):
        x = -0.58+k*0.165
        nodes = [[x+0.025*math.sin(k*2+j*1.7), -3+j*2.3] for j in range(5)]
        for j in range(4):
            strokes.append({'start': nodes[j], 'end': nodes[j+1], 'widths': [0.007+0.002*math.sin(k+j)**2, 0.005+0.002*math.sin(k+j+1)**2]})
    return {'pattern': {'frequency': [38, 1.5, 38], 'low': [0.62, 0.59, 0.51], 'high': [1.32, 1.22, 1.06],
            'bump': 0.0007, 'warp': 1.2, 'octaves': 3, 'seed': 137, 'contrast': 1.4, 'kind': 'ridged'},
            'transmission': 0, 'detail': {'plane': 'xy', 'tint': [0.55, 0.52, 0.46], 'opacity': 0.55, 'bump': -0.0011, 'segments': strokes}}


def run(frames, input_path=None):
    input_path = input_path or PROJECT/'examples/native-tree-study.scene.json'
    snapshot = OUTPUT/'examples/surface-input.scene.json'
    if snapshot.resolve() != input_path.resolve(): shutil.copyfile(input_path, snapshot)
    c = Client(PROJECT/'target/release/ignis', False); call = c.call
    def asset(id): return call('inspect_mesh', asset=id)['asset']
    def pose(id, tick): return call('inspect_mesh_pose', instance=id, tick=tick)['pose']
    def capture(name, width=1000, height=900):
        call('capture', path=f'artifacts/surface-{name}.png', width=width, height=height)
        return hashlib.sha256((OUTPUT/f'artifacts/surface-{name}.png').read_bytes()).hexdigest()
    cameras = {'whole': CAMERA, 'bark': {'position': [1.3, 2.3, 2], 'target': [0.10, 2, 0.1], 'fov_degrees': 32},
               'fork': {'position': [1.4, 2.2, 3.5], 'target': [-0.05, 1.7, 0.1], 'fov_degrees': 40}}
    def leaf_preview(name, override=None):
        before = call('inspect')['scene']
        edits = [{'op': 'mesh', 'edit': {'kind': 'remove_instance', 'id': i['id']}} for i in before['meshes']['instances']]
        if override: edits.append({'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': 'blade-source', 'material': override}})
        edits.append({'op': 'mesh', 'edit': {'kind': 'set_shading', 'asset': 'blade-source', 'shading': 'smooth'}})
        edits += [{'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': {'id': 'leaf-preview', 'asset': 'blade-source',
            'position': [0, 3.2, 0], 'rotation': [0, 0, 0, 1], 'scale': 3, 'color': [0.09, 0.26, 0.045], 'playback': None}}},
            {'op': 'set_camera', 'camera': {'position': [0.38, 4.3, 1.05], 'target': [0, 3.2, 0.36], 'fov_degrees': 32}}]
        call('edit_scene', edits=edits); digest=capture(name)
        call('undo'); assert call('inspect')['scene'] == before
        return digest
    def crown_leaf_preview(name):
        before = call('inspect')['scene']
        inspected = call('inspect_mesh_islands', asset='foliage-0')
        part = {'kind': 'extract_islands', 'asset': 'foliage-0', 'id': 'crown-leaf-preview',
                'reference': inspected['reference'], 'islands': [inspected['topology']['islands'][0]['id']], 'mode': 'copy'}
        edits = [{'op': 'mesh', 'edit': part}]
        edits += [{'op': 'mesh', 'edit': {'kind': 'remove_instance', 'id': i['id']}} for i in before['meshes']['instances']]
        edits.append({'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': {'id': 'crown-leaf-view', 'asset': 'crown-leaf-preview',
            'position': [0, 3.2, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.09, 0.26, 0.045],
            'playback': {'clip': 'breeze', 'offset_ticks': 0, 'looping': True}}}})
        call('edit_scene', edits=edits)
        points = pose('crown-leaf-view', 0)['positions']
        triangles = asset('crown-leaf-preview')['geometry']['triangles']
        normal = [0., 0., 0.]
        for a, b, d in triangles:
            u = [points[b][k]-points[a][k] for k in range(3)]; v = [points[d][k]-points[a][k] for k in range(3)]
            cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
            normal = [normal[k]+cross[k] for k in range(3)]
        n = math.sqrt(sum(x*x for x in normal)); normal = [x/n for x in normal]
        centre = [(min(p[k] for p in points)+max(p[k] for p in points))/2 for k in range(3)]
        distance = max(max(p[k] for p in points)-min(p[k] for p in points) for k in range(3))*2.5
        direction = [normal[0]+0.18, normal[1]+0.1, normal[2]+0.22]
        length = math.sqrt(sum(x*x for x in direction)); direction = [x/length for x in direction]
        call('set_camera', camera={'position': [centre[k]+direction[k]*distance for k in range(3)], 'target': centre, 'fov_degrees': 32})
        digest = capture(name)
        call('undo'); call('undo'); assert call('inspect')['scene'] == before
        return digest
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-tree-surfaces', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/surface-input.scene.json')
        initial = call('inspect')['scene']; original_assets = initial['meshes']['assets']; instances = initial['meshes']['instances']
        initial_plant = call('inspect_plant', id='branch-editable-1')['plant']
        baseline = {tick: {i['id']: pose(i['id'], tick) for i in instances} for tick in [0, 60]}
        call('set_light', light=LIGHT)
        views = {}
        for name, camera in cameras.items():
            call('set_camera', camera=camera); call('step', ticks=60); views[name] = {'before': capture('before-'+name)}
        old_leaf = next(a for a in original_assets if a['id']=='foliage-0')['material']
        views['leaf'] = {'before': leaf_preview('before-leaf', old_leaf)}
        views['crown_leaf'] = {'before': crown_leaf_preview('before-crown-leaf')}
        targets = [a['id'] for a in original_assets if a['id'].startswith('foliage-') or a['id']=='blade-source']
        domains = {id: len(call('inspect_mesh_islands', asset=id)['topology']['islands']) for id in targets}
        assert domains['blade-source'] == 1 and all(n == 66 for id, n in domains.items() if id.startswith('foliage-'))
        assert targets[0] == 'blade-source'
        edits = []
        for id in targets:
            edits += [{'op': 'mesh', 'edit': ({'kind': 'fit_material_coordinates', 'asset': id, 'per_island': True, 'bounds': [[-1, 0, 0], [1, 0, 1]]} if id == 'blade-source' else {'kind': 'copy_material_coordinates', 'asset': id, 'source': 'blade-source', 'per_island': True})},
                      {'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': id, 'material': leaf_material()}}]
        for id in ['wood', 'branch-editable-1', 'secondary-06', 'secondary-twigs']:
            edits.append({'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': id, 'material': bark_material()}})
        before = call('inspect')
        predicted = call('preview_mesh_edit', edit=edits[0]['edit']); assert call('inspect') == before
        call('edit_scene', edits=edits)
        changed = call('inspect')['scene']['meshes']
        assert changed['instances'] == instances
        for original in original_assets:
            after = asset(original['id'])
            for key in ['positions', 'triangles']: assert after['geometry'][key] == original['geometry'][key]
            for key in ['rig', 'clips', 'shading']: assert after.get(key) == original.get(key)
            if original['id'] not in targets: assert after['geometry'] == original['geometry']
        template = asset('blade-source')['geometry']['material_coordinates']
        for id in targets:
            if id != 'blade-source':
                assert asset(id)['geometry']['material_coordinates'] == template * domains[id]
        for tick in [0, 60]:
            for id, p in baseline[tick].items(): assert pose(id, tick) == p
        plant = call('inspect_plant', id='branch-editable-1')['plant']
        for key in ['definition', 'reference']: assert plant[key] == initial_plant[key]
        for key in ['geometry', 'rig', 'clips', 'shading']: assert plant['source'].get(key) == initial_plant['source'].get(key)
        c0 = next(a for a in changed['assets'] if a['id']==targets[0])['geometry']['material_coordinates']
        assert c0 == predicted['meshes']['assets'][[a['id'] for a in original_assets].index(targets[0])]['geometry']['material_coordinates']
        call('undo'); assert call('inspect')['scene']['meshes'] == initial['meshes']
        call('redo'); assert call('inspect')['scene']['meshes'] == changed
        for name, camera in cameras.items():
            call('set_camera', camera=camera); call('step', ticks=60); views[name]['after'] = capture('after-'+name)
            assert views[name]['before'] != views[name]['after']
        call('set_camera', camera=cameras['bark'])
        without_bark_detail = []
        for id in ['wood', 'branch-editable-1', 'secondary-06', 'secondary-twigs']:
            value = copy.deepcopy(asset(id)['material']); value.pop('detail')
            without_bark_detail.append({'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': id, 'material': value}})
        call('edit_scene', edits=without_bark_detail); call('step', ticks=60)
        views['bark']['without_detail'] = capture('bark-without-detail')
        assert views['bark']['without_detail'] != views['bark']['after']
        call('undo'); call('step', ticks=60)
        assert capture('bark-detail-restored') == views['bark']['after']
        views['crown_leaf']['after'] = crown_leaf_preview('after-crown-leaf')
        assert views['crown_leaf']['after'] != views['crown_leaf']['before']
        views['leaf']['after'] = leaf_preview('after-leaf'); assert views['leaf']['before'] != views['leaf']['after']
        # Ablations isolate the new detail layer from the base material/geometry.
        current = asset('blade-source')['material']; without = copy.deepcopy(current); without.pop('detail')
        views['leaf']['without_detail'] = leaf_preview('leaf-without-detail', without)
        assert views['leaf']['without_detail'] != views['leaf']['after']
        call('set_camera', camera=CAMERA); call('save_scene', path='examples/surface-tree-candidate.scene.json')
        report = {'passed': True, 'build_id': call('inspect')['build_id'], 'native_window': False,
                  'input_sha256': hashlib.sha256(snapshot.read_bytes()).hexdigest(), 'geometry_rig_clips_and_poses_exact': True,
                  'plant_source_geometry_and_reference_exact': True, 'material_channels_changed_only_for_leaves': True, 'template_correspondence_exact_in_all_leaf_islands': True,
                  'leaf_strokes': len(leaf_material()['detail']['segments']), 'bark_strokes': len(bark_material()['detail']['segments']),
                  'domains_from_native_inspection': domains, 'views': views, 'assets': len(changed['assets']), 'instances': len(instances)}
        (OUTPUT/'artifacts/surface-report.json').write_text(json.dumps(report, indent=2)+'\n'); print(json.dumps(report, indent=2))
    finally: c.close()
    finish(frames, 4, OUTPUT/'examples/surface-tree-candidate.scene.json')


if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--frames',type=int,default=48);p.add_argument('--input',type=Path);a=p.parse_args()
    if a.frames < 0 or a.frames > 120 or (a.frames and 240 % a.frames): p.error('frames must be0 or a positive divisor of240 up to120')
    run(a.frames,a.input)
