#!/usr/bin/env python3
"""Bind an existing tree secondary and its animated crown to a named native bone."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct

from demo_tree_authoring import PROJECT, OUTPUT, CAMERA, LIGHT, skeleton, make_cluster
from demo_tree_smoothing import run as finish
from tree_study_geometry import append, rotate
from tree_branch_plant import put
from verify_lab import Client


def matrix(a, b):
    return [[sum(a[k][r]*b[c][k] for k in range(4)) for r in range(4)] for c in range(4)]


def origin(m):
    return m[3][:3]


def run(frames, input_path=None):
    input_path = input_path or PROJECT/'examples/native-tree-study.scene.json'
    snapshot = OUTPUT/'examples/attachment-input.scene.json'
    if snapshot.resolve() != input_path.resolve():
        shutil.copyfile(input_path, snapshot)
    c = Client(PROJECT/'target/release/ignis', False)
    call = c.call
    def edit(kind, **kwargs):
        return call('edit_mesh', edit={'kind': kind, **kwargs})
    def asset(id):
        return call('inspect_mesh', asset=id)['asset']
    def pose(id, tick=0):
        return call('inspect_mesh_pose', instance=id, tick=tick)['pose']
    def capture(name):
        return call('capture', path=f'artifacts/attachment-{name}.png', width=1000, height=900)
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-tree-attachments', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/attachment-input.scene.json')
        before_scene = call('inspect')['scene']
        assert not any(i.get('attachment') for i in before_scene['meshes']['instances']), 'Use the unbound attachment-input fixture to repeat this authoring step'
        initial_plant = call('inspect_plant', id='branch-editable-1')['plant']
        wood = asset('wood')
        leaf = asset('blade-source')['geometry']
        generated, placements, branches, junction, _ = skeleton(call)
        twig_ranges = []
        for placement in placements:
            _, twig = make_cluster(leaf, placement['variant'])
            start = len(generated['positions'])
            append(generated, twig, placement['position'], placement['rotation'], placement['scale'])
            twig_ranges.append(range(start, len(generated['positions'])))
        f32 = lambda v: struct.unpack('f', struct.pack('f', v))[0]
        for channel in ['positions', 'material_coordinates']:
            assert [[f32(v) for v in p] for p in generated[channel]] == wood['geometry'][channel], channel
        assert generated['triangles'] == wood['geometry']['triangles']
        selected = next(b for b in branches if b['parent'] == 1 and b['attachment_t'] == 0.30)
        start, end = selected['geometry_range']['vertices']
        removed = set(range(start, end)) | set(twig_ranges[6])
        assert len(removed) == 71
        source_geometry = wood['geometry']

        target = initial_plant['definition']
        reference = initial_plant['reference']
        put(call, reference)
        restored = asset('branch-editable-1')['geometry']
        restoration_error = max(math.dist(p, q) for p, q in zip(restored['positions'], initial_plant['source']['geometry']['positions']))
        station = reference['sections']['stations'][2]
        height = reference['profile']['height']
        root = [0, -height/2, 0]
        socket = [station['offset'][0], station['at']*height, station['offset'][1]]
        rig = {'bones': [{'name': 'base', 'parent': None, 'translation': root, 'rotation': [0, 0, 0, 1]},
                         {'name': 'secondary-06', 'parent': 0, 'translation': socket, 'rotation': [0, 0, 0, 1]}],
               'weights': [{'joints': [0, 0, 0, 0], 'weights': [1, 0, 0, 0]} for _ in asset('branch-editable-1')['geometry']['positions']]}
        keys = []
        for tick in range(0, 241, 30):
            angle = math.radians(1.5)*math.sin(tick*2*math.pi/240)/2 if tick not in [0, 240] else 0
            keys.append({'tick': tick, 'translation': socket, 'rotation': [0, 0, math.sin(angle), math.cos(angle)]})
        # Direct rig/clip authoring explicitly rebases under the existing plant
        # contract. Keep the original tessellation and measure f32 reconstruction
        # rather than claiming this metadata edit preserves source bits.
        edit('set_rig', asset='branch-editable-1', rig=rig)
        edit('put_clip', asset='branch-editable-1', clip={'id': 'secondary-breeze', 'duration_ticks': 240, 'tracks': [{'bone': 1, 'keys': keys}]})
        before_channel_restore = call('inspect_plant', id='branch-editable-1')['plant']['source']['geometry']['material_coordinates']
        channel_drift = max(math.dist(p, q) for p, q in zip(before_channel_restore, initial_plant['source']['geometry']['material_coordinates']))
        # This is an explicit authored channel restoration in the original
        # coarse reference, not a resampled preview replacing its source.
        edit('set_material_coordinates', asset='branch-editable-1', coordinates=initial_plant['source']['geometry']['material_coordinates'])
        enriched = call('inspect_plant', id='branch-editable-1')['plant']
        assert enriched['reference'] == reference
        for field in ['material', 'shading']:
            assert enriched['source'][field] == initial_plant['source'][field]
        assert enriched['source']['geometry']['triangles'] == initial_plant['source']['geometry']['triangles']
        source_errors = {field: max(math.dist(p, q) for p, q in zip(enriched['source']['geometry'][field], initial_plant['source']['geometry'][field]))
                         for field in ['positions', 'material_coordinates']}
        assert max(source_errors.values()) < 0.000003, source_errors
        assert enriched['source']['geometry'] == initial_plant['source']['geometry']
        put(call, target)
        old_branch = next(a for a in before_scene['meshes']['assets'] if a['id'] == 'branch-editable-1')
        branch_error = max(math.dist(p, q) for p, q in zip(asset('branch-editable-1')['geometry']['positions'], old_branch['geometry']['positions']))
        assert branch_error < 0.000003
        parent = next(i for i in before_scene['meshes']['instances'] if i['id'] == 'tree-branch-editable-1')
        parent_pose = pose(parent['id'])
        anchor = origin(matrix(parent_pose['world_transform'], parent_pose['joints'][1]))

        def subset(indices, local=False):
            ids = sorted(indices)
            mapping = {old: new for new, old in enumerate(ids)}
            triangles = []
            for triangle in source_geometry['triangles']:
                inside = [i in mapping for i in triangle]
                assert all(inside) or not any(inside), 'Extraction crosses a shared boundary'
                if all(inside):
                    triangles.append([mapping[i] for i in triangle])
            return {'positions': [[p[k]-anchor[k] if local else p[k] for k in range(3)] for p in (source_geometry['positions'][i] for i in ids)],
                    'material_coordinates': [source_geometry['material_coordinates'][i] for i in ids], 'triangles': triangles}
        kept = subset(set(range(len(source_geometry['positions'])))-removed)
        detached = subset(removed, True)
        assert kept['positions'][:junction['assembly_vertices']] == source_geometry['positions'][:junction['assembly_vertices']]
        assert kept['triangles'][:junction['assembly_triangles']] == source_geometry['triangles'][:junction['assembly_triangles']]
        secondary_asset = dict(wood, id='secondary-06', geometry=detached)
        secondary_instance = {'id': 'tree-secondary-06', 'asset': secondary_asset['id'], 'position': [0, 0, 0], 'rotation': [0, 0, 0, 1],
                              'scale': 1, 'color': parent['color'], 'playback': None,
                              'attachment': {'parent': parent['id'], 'bone': 'secondary-06'}}
        crown = copy.deepcopy(next(i for i in before_scene['meshes']['instances'] if i['id'] == 'crown-06'))
        old_crown_pose = pose(crown['id'])
        old_crown_60 = pose(crown['id'], 60)
        crown['position'] = [p-a for p, a in zip(crown['position'], anchor)]
        crown['attachment'] = {'parent': secondary_instance['id']}
        parent = dict(parent, playback={'clip': 'secondary-breeze', 'offset_ticks': 0, 'looping': True})
        call('edit_scene', edits=[{'op': 'mesh', 'edit': e} for e in [
            {'kind': 'put_asset', 'asset': dict(wood, geometry=kept)}, {'kind': 'put_asset', 'asset': secondary_asset},
            {'kind': 'put_instance', 'instance': secondary_instance}, {'kind': 'put_instance', 'instance': crown}, {'kind': 'put_instance', 'instance': parent}]])
        rest = pose(secondary_instance['id'])
        extraction_error = max(math.dist(p, source_geometry['positions'][i]) for p, i in zip(rest['positions'], sorted(removed)))
        crown_error = max(math.dist(p, q) for p, q in zip(pose(crown['id'])['positions'], old_crown_pose['positions']))
        assert max(extraction_error, crown_error) < 0.000003
        source = call('inspect_plant', id='branch-editable-1')['plant']['source']
        baseline_secondary = asset('secondary-06')
        baseline_crown = asset(crown['asset'])
        call('set_light', light=LIGHT)
        call('set_camera', camera={'position': [anchor[0]+2.5, anchor[1]+1, anchor[2]+3.5], 'target': selected['tip'], 'fov_degrees': 42})
        capture('rest')
        changed = copy.deepcopy(target)
        node = changed['sections']['stations'][2]
        node['offset'][0] += 0.15
        node['offset'][1] -= 0.10
        before = call('inspect')
        put(call, changed, 'preview_plant'); assert call('inspect') == before
        put(call, changed)
        moved = pose(secondary_instance['id'])
        moved_crown = pose(crown['id'])
        delta = [moved['positions'][0][k]-rest['positions'][0][k] for k in range(3)]
        assert math.dist(delta, [0.15, 0, -0.10]) < 0.000003
        for p, q in zip(moved_crown['positions'], old_crown_pose['positions']):
            assert math.dist([p[k]-q[k] for k in range(3)], delta) < 0.000003
        assert asset('secondary-06') == baseline_secondary and asset(crown['asset']) == baseline_crown
        assert call('inspect_plant', id='branch-editable-1')['plant']['source'] == source
        capture('edited')
        call('undo'); assert pose(secondary_instance['id']) == rest
        call('redo'); assert pose(secondary_instance['id']) == moved
        put(call, target); assert pose(secondary_instance['id']) == rest
        assert pose(secondary_instance['id'], 60)['positions'] != rest['positions']
        assert pose(crown['id'], 60)['positions'] != pose(crown['id'])['positions']
        angle = math.radians(1.5)/2
        q = [0, 0, math.sin(angle), math.cos(angle)]
        for p, original in zip(pose(crown['id'], 60)['positions'], old_crown_60['positions']):
            rotated = rotate(q, [original[k]-anchor[k] for k in range(3)])
            assert math.dist(p, [anchor[k]+rotated[k] for k in range(3)]) < 0.000005
        assert pose(parent['id'], 60)['positions'] == pose(parent['id'])['positions']
        current = call('inspect')['scene']['meshes']
        for old in before_scene['meshes']['assets']:
            if old['id'] not in ['wood', 'branch-editable-1']:
                assert next(a for a in current['assets'] if a['id'] == old['id']) == old
        changed_instances = {parent['id'], crown['id']}
        assert [i for i in current['instances'] if i['id'] not in changed_instances | {secondary_instance['id']}] == [i for i in before_scene['meshes']['instances'] if i['id'] not in changed_instances]
        assert len(current['assets']) == 9 and len(current['instances']) == 31
        assert sum(len(a['geometry']['positions']) for a in current['assets']) == 11172
        sizes = {a['id']: len(a['geometry']['triangles']) for a in current['assets']}
        assert sum(sizes[i['asset']] for i in current['instances']) == 64794
        before_rejection = call('inspect')
        wrong = copy.deepcopy(asset('branch-editable-1')['rig'])
        wrong['bones'][1]['name'] = 'missing-socket'
        call('edit_mesh', edit={'kind': 'set_rig', 'asset': 'branch-editable-1', 'rig': wrong}, expected_error=True)
        assert call('inspect') == before_rejection
        call('edit_mesh', edit={'kind': 'remove_instance', 'id': parent['id']}, expected_error=True)
        assert call('inspect') == before_rejection
        call('step', ticks=60); capture('animated')
        call('set_camera', camera=CAMERA)
        call('save_scene', path='examples/attached-tree-candidate.scene.json')
        report = {'build_id': call('inspect')['build_id'], 'input_sha256': hashlib.sha256(snapshot.read_bytes()).hexdigest(),
                  'removed_vertices': len(removed), 'removed_triangles': len(detached['triangles']),
                  'extraction_rest_error': extraction_error, 'crown_rest_error': crown_error,
                  'reference_reconstruction_error_before_source_authoring': restoration_error,
                  'authored_source_errors': source_errors, 'branch_after_rig_error': branch_error,
                  'material_channel_drift_before_explicit_restore': channel_drift,
                  'edited_delta': delta, 'first_fork_exact': True, 'source_reference_exact_after_rig_authoring': True,
                  'runtime_bone_motion_and_child_playback': True, 'native_window': False}
        report['other_assets_and_instances_exact'] = True
        report['referenced_bone_and_parent_removal_atomic'] = True
        (OUTPUT/'artifacts/attachment-report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report, indent=2))
    finally:
        c.close()
    # Reuse the matched-mode/retessellation/persistence/replay/GIF checks on the
    # bound tree. No geometry is reconstructed by this final verification.
    finish(frames, 4, OUTPUT/'examples/attached-tree-candidate.scene.json')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames', type=int, default=48)
    parser.add_argument('--input', type=Path)
    args = parser.parse_args()
    if args.frames < 0 or args.frames > 120 or (args.frames and 240 % args.frames):
        parser.error('frames must be0 or a positive divisor of240 up to120')
    run(args.frames, args.input)
