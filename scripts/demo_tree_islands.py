#!/usr/bin/env python3
"""Separate saved tree loose parts using native queries, without recipe indices."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import shutil

from demo_tree_authoring import PROJECT, OUTPUT, CAMERA, LIGHT
from demo_tree_smoothing import run as finish
from verify_lab import Client


def run(frames, input_path=None):
    input_path = input_path or PROJECT/'examples/native-tree-study.scene.json'
    snapshot = OUTPUT/'examples/islands-input.scene.json'
    if input_path.resolve() != snapshot.resolve():
        shutil.copyfile(input_path, snapshot)
    c = Client(PROJECT/'target/release/ignis', False)
    call = c.call
    def asset(id):
        return call('inspect_mesh', asset=id)['asset']
    def pose(id, tick):
        return call('inspect_mesh_pose', instance=id, tick=tick)['pose']
    def capture(name):
        return call('capture', path=f'artifacts/islands-{name}.png', width=1000, height=900)
    def signature(values):
        return Counter(tuple(p) for p in values)
    def faces(geometry):
        return Counter(tuple(tuple(geometry['positions'][i]) for i in face) for face in geometry['triangles'])
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-tree-islands', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/islands-input.scene.json')
        original = call('inspect')['scene']
        assert not any(a['id'] == 'secondary-twigs' for a in original['meshes']['assets']), 'Repeat with the unextracted islands-input fixture'
        plant = call('inspect_plant', id='branch-editable-1')['plant']
        source = asset('secondary-06')
        parent = next(i for i in original['meshes']['instances'] if i['asset'] == source['id'])
        baseline = {tick: {i['id']: pose(i['id'], tick) for i in original['meshes']['instances']} for tick in [0, 60]}
        points = baseline[0][parent['id']]['positions']
        target = [sum(p[k] for p in points)/len(points) for k in range(3)]
        call('set_camera', camera={'position': [target[0]+2.5, target[1]+1, target[2]+3.5], 'target': target, 'fov_degrees': 35})
        call('set_light', light=LIGHT)
        capture('before-rest')
        call('step', ticks=60)
        capture('before-motion')
        inspected = call('inspect_mesh_islands', asset=source['id'])
        islands = inspected['topology']['islands']
        largest = max(islands, key=lambda s: s['triangles'])
        selected = [s['id'] for s in islands if s['triangles'] < largest['triangles']]
        assert len(islands) == 7 and len(selected) == 6
        edit = {'kind': 'extract_islands', 'asset': source['id'], 'id': 'secondary-twigs', 'reference': inspected['reference'], 'islands': selected, 'mode': 'move'}
        before = call('inspect')
        preview = call('preview_mesh_edit', edit=edit)
        assert call('inspect') == before
        child = {'id': 'tree-secondary-twigs', 'asset': edit['id'], 'position': [0, 0, 0], 'rotation': [0, 0, 0, 1],
                 'scale': 1, 'color': parent['color'], 'playback': None, 'attachment': {'parent': parent['id']}}
        call('edit_scene', edits=[{'op': 'mesh', 'edit': edit}, {'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': child}}])
        meshes = call('inspect')['scene']['meshes']
        assert meshes['assets'] == preview['meshes']['assets']
        assert meshes['instances'][:-1] == original['meshes']['instances']
        pieces = [asset(source['id']), asset(edit['id'])]
        assert [len(call('inspect_mesh_islands', asset=a['id'])['topology']['islands']) for a in pieces] == [1, 6]
        for channel in ['positions', 'material_coordinates']:
            assert signature(source['geometry'][channel]) == sum((signature(a['geometry'][channel]) for a in pieces), Counter())
        assert faces(source['geometry']) == sum((faces(a['geometry']) for a in pieces), Counter())
        for a in pieces:
            for key in ['material', 'shading', 'rig', 'clips']:
                assert a.get(key) == source.get(key)
        for tick in [0, 60]:
            assert signature(baseline[tick][parent['id']]['positions']) == signature(pose(parent['id'], tick)['positions']) + signature(pose(child['id'], tick)['positions'])
            for id, old in baseline[tick].items():
                if id != parent['id']:
                    assert pose(id, tick) == old, (id, tick)
        capture('after-rest')
        assert (OUTPUT/'artifacts/islands-before-rest.png').read_bytes() == (OUTPUT/'artifacts/islands-after-rest.png').read_bytes()
        call('step', ticks=60)
        capture('after-motion')
        assert (OUTPUT/'artifacts/islands-before-motion.png').read_bytes() == (OUTPUT/'artifacts/islands-after-motion.png').read_bytes()
        before = call('inspect')
        call('edit_mesh', edit=edit, expected_error=True)
        assert call('inspect') == before
        call('undo')
        assert call('inspect')['scene']['meshes'] == original['meshes']
        call('redo')
        assert call('inspect')['scene']['meshes'] == meshes
        assert call('inspect_plant', id='branch-editable-1')['plant'] == plant
        for a in original['meshes']['assets']:
            if a['id'] != source['id']:
                assert asset(a['id']) == a
        # Exercise rigged Copy preview on the same complex scene. No source rebase,
        # instance creation or full geometry reconstruction is done by this script.
        foliage = next(a for a in meshes['assets'] if a.get('rig') and a['id'].startswith('foliage-'))
        inspected_leaf = call('inspect_mesh_islands', asset=foliage['id'])
        before = call('inspect')
        copied = call('preview_mesh_edit', edit={'kind': 'extract_islands', 'asset': foliage['id'], 'id': 'foliage-copy-probe',
            'reference': inspected_leaf['reference'], 'islands': [s['id'] for s in inspected_leaf['topology']['islands']], 'mode': 'copy'})
        assert call('inspect') == before
        expected = dict(foliage, id='foliage-copy-probe')
        assert copied['meshes']['assets'][-1] == expected
        sizes = {a['id']: len(a['geometry']['triangles']) for a in meshes['assets']}
        assert len(meshes['assets']) == 10 and len(meshes['instances']) == 32
        assert sum(len(a['geometry']['positions']) for a in meshes['assets']) == 11172
        assert sum(sizes.values()) == 16250
        assert sum(sizes[i['asset']] for i in meshes['instances']) == 64794
        call('set_camera', camera=CAMERA)
        call('save_scene', path='examples/islands-tree-candidate.scene.json')
        report = {'passed': True, 'build_id': call('inspect')['build_id'], 'native_window': False,
                  'input_sha256': hashlib.sha256(snapshot.read_bytes()).hexdigest(), 'selection_from_native_summaries': selected,
                  'before_islands': islands, 'after_parts': [{k: len(a['geometry'][k]) for k in ['positions', 'triangles']} for a in pieces],
                  'exact_channels_faces_and_material': True, 'other_assets_instances_and_plant_exact': True,
                  'rest_and_tick60_pixels_exact': True, 'rest_and_tick60_poses_exact': True, 'undo_redo_and_stale_rejection': True,
                  'rigged_full_copy_preview_exact': True, 'assets': len(meshes['assets']), 'instances': len(meshes['instances']), 'draw_triangles': 64794}
        (OUTPUT/'artifacts/islands-report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report, indent=2))
    finally:
        c.close()
    finish(frames, 4, OUTPUT/'examples/islands-tree-candidate.scene.json')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames', type=int, default=48)
    parser.add_argument('--input', type=Path)
    args = parser.parse_args()
    if args.frames < 0 or args.frames > 120 or (args.frames and 240 % args.frames):
        parser.error('frames must be0 or a positive divisor of240 up to120')
    run(args.frames, args.input)
