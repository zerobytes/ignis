#!/usr/bin/env python3
"""Edit the saved native tree: matched linear/PCHIP cases and preserved source."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import shutil

from demo_tree_authoring import PROJECT, OUTPUT, CAMERA, LIGHT
from tree_branch_plant import put
from tree_study_geometry import add, rotate
from verify_lab import Client


def run(frames, segments_y, input_path=None):
    source_path = PROJECT/'examples/native-tree-study.scene.json'
    input_path = input_path or source_path
    snapshot = OUTPUT/'examples/smoothing-input.scene.json'
    if input_path.resolve() != snapshot.resolve():
        shutil.copyfile(input_path, snapshot)
    input_digest = hashlib.sha256(snapshot.read_bytes()).hexdigest()
    input_version = json.loads(snapshot.read_text())['version']
    c = Client(PROJECT/'target/release/ignis', False)
    counts = {}

    def call(op, **kwargs):
        counts[op] = counts.get(op, 0)+1
        return c.call(op, **kwargs)

    def capture(name, width=1000, height=900):
        return call('capture', path=f'artifacts/{name}.png', width=width, height=height)

    def asset():
        return call('inspect_mesh', asset='branch-editable-1')['asset']

    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                               'clientInfo': {'name': 'ignis-tree-smoothing-lab', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/smoothing-input.scene.json')
        loaded = call('inspect')['scene']
        draw_limit = call('inspect_meshes')['limits']['draw_triangles']
        plant = call('inspect_plant', id='branch-editable-1')['plant']
        source, reference = plant['source'], plant['reference']
        assert reference['sections'].get('interpolation', 'linear') == 'linear'
        assert reference['segments'] == [4, 2, 4]
        initial_asset = asset()
        others = [a for a in loaded['meshes']['assets'] if a['id'] != initial_asset['id']]
        instances = loaded['meshes']['instances']
        instance = next(i for i in instances if i['asset'] == initial_asset['id'])
        # Parent playback may drive attachment sockets while its own geometry
        # remains static. All matched cases still use the same tick and clip.
        leaf_pose = call('inspect_mesh_pose', instance='crown-00')['pose']
        station = reference['sections']['stations'][2]
        height = reference['profile']['height']
        target = [station['offset'][0]+instance['position'][0],
                  (station['at']-0.5)*height+instance['position'][1],
                  station['offset'][1]+instance['position'][2]]
        close = {'position': [target[0]+2, target[1]+0.55, target[2]+3],
                 'target': target, 'fov_degrees': 30}
        call('set_camera', camera=close)
        # Camera/light are held fixed for all comparison captures; no stepping.
        call('set_light', light=LIGHT)

        a = copy.deepcopy(reference)
        a['sections'].pop('interpolation', None)
        b = copy.deepcopy(a)
        control = next(s for s in b['sections']['stations'] if s['id'] == 'branch1-015')
        control['offset'][0] += 0.18
        control['offset'][1] -= 0.10
        cases, evaluated, reports = [], {}, {}
        for density, segments in [('coarse', [4, 2, 4]), ('medium', [4, 4, 4]), ('dense', [4, 6, 4])]:
            for shape, drawing in [('A', a), ('B', b)]:
                for mode in ['linear', 'pchip']:
                    drawing = copy.deepcopy(drawing)
                    drawing['segments'] = segments
                    if mode == 'pchip':
                        drawing['sections']['interpolation'] = mode
                    else:
                        drawing['sections'].pop('interpolation', None)
                    cases.append((f'{mode}-{shape}-{density}', drawing))

        def check_unchanged():
            entry = call('inspect_plant', id=initial_asset['id'])['plant']
            assert entry['source'] == source and entry['reference'] == reference
            meshes = call('inspect')['scene']['meshes']
            assert [a for a in meshes['assets'] if a['id'] != initial_asset['id']] == others
            assert meshes['instances'] == instances
            assert call('inspect_mesh_pose', instance='crown-00')['pose'] == leaf_pose
            return meshes

        def draw_count(meshes):
            sizes = {a['id']: len(a['geometry']['triangles']) for a in meshes['assets']}
            return sum(sizes[i['asset']] for i in meshes['instances'])

        rejected_cases = {}
        fixed_sizes = {a['id']: len(a['geometry']['triangles']) for a in others}
        fixed_draw = sum(fixed_sizes[i['asset']] for i in instances if i['asset'] != initial_asset['id'])
        copies = sum(i['asset'] == initial_asset['id'] for i in instances)
        for name, drawing in cases:
            before = call('inspect')
            expected_triangles = {2: 512, 4: 960, 6: 1408}[drawing['segments'][1]]
            if fixed_draw + copies * expected_triangles > draw_limit:
                for op in ['preview_plant', 'edit_plant']:
                    error = call(op, expected_error=True, edit={'kind': 'put', 'definition': drawing,
                        'detail_policy': 'physical', 'animation_translation_policy': 'physical'})
                    assert f'{draw_limit} rendered triangles' in str(error)
                    assert call('inspect') == before
                rejected_cases[name] = {'reason': 'draw_budget', 'predicted_draw_triangles': fixed_draw + copies * expected_triangles}
                continue
            preview = put(call, drawing, 'preview_plant')
            assert call('inspect') == before
            predicted = next(a for a in preview['assets'] if a['id'] == initial_asset['id'])
            put(call, drawing)
            value = asset()
            assert value == predicted
            assert value['material'] == initial_asset['material']
            assert value['shading'] == initial_asset['shading']
            meshes = check_unchanged()
            errors = []
            for station in reference['sections']['stations'][2:]:
                y = (station['at']-0.5)*height
                ring = [p for p in value['geometry']['positions'] if abs(p[1]-y) < 0.000002]
                assert ring
                center = [sum(p[k] for p in ring)/len(ring) for k in range(3)]
                errors.append(math.dist(center, [station['offset'][0], y, station['offset'][1]]))
            assert max(errors) < 0.000002, (name, errors)
            topology = call('inspect_mesh_topology', asset=initial_asset['id'])['topology']
            assert topology['boundary_edges'] == topology['non_manifold_edges'] == 0
            assert not topology['isolated_vertices']
            drawn = draw_count(meshes)
            assert drawn <= draw_limit
            captured = capture('tree-smooth-'+name)
            assert captured['mesh_triangle_count'] == drawn
            evaluated[name] = value
            reports[name] = {'vertices': len(value['geometry']['positions']),
                             'triangles': len(value['geometry']['triangles']),
                             'draw_triangles': drawn, 'anchor_errors': errors}

        for density in ['coarse', 'medium', 'dense']:
            if 'linear-A-'+density in rejected_cases:
                continue
            geometry = evaluated['linear-A-'+density]['geometry']
            for mode in ['linear', 'pchip']:
                for shape in ['A', 'B']:
                    current = evaluated[f'{mode}-{shape}-{density}']['geometry']
                    assert current['triangles'] == geometry['triangles']
                    assert current['material_coordinates'] == geometry['material_coordinates']
            for shape in ['A', 'B']:
                l, p = f'linear-{shape}-{density}', f'pchip-{shape}-{density}'
                assert evaluated[l]['geometry']['positions'] != evaluated[p]['geometry']['positions']
                assert (OUTPUT/f'artifacts/tree-smooth-{l}.png').read_bytes() != (OUTPUT/f'artifacts/tree-smooth-{p}.png').read_bytes()
        # If this invocation loaded the old linear target, restore it exactly.
        if plant['definition'] == reference:
            assert evaluated['linear-A-coarse'] == initial_asset
        accepted = [(name, drawing) for name, drawing in cases if name in evaluated]
        call('undo'); assert asset() == evaluated[accepted[-2][0]]
        call('redo'); assert asset() == evaluated[accepted[-1][0]]
        for _ in range(2):
            for name, drawing in accepted:
                put(call, drawing)
                assert asset() == evaluated[name]
                check_unchanged()
        final_name = 'pchip-A-'+('medium' if segments_y == 4 else 'dense')
        final = next(d for name, d in cases if name == final_name)
        put(call, final)
        assert asset() == evaluated[final_name]
        meshes = check_unchanged()
        drawn = draw_count(meshes)
        # Account for the actual unchanged assets in the loaded scene, instead
        # of baking the old fork's triangle count into unrelated plant tests.
        other_sizes = {a['id']: len(a['geometry']['triangles']) for a in others}
        fixed_draw = sum(other_sizes[i['asset']] for i in instances if i['asset'] != initial_asset['id'])
        copies = sum(i['asset'] == initial_asset['id'] for i in instances)
        expected_branch_triangles = 960 if segments_y == 4 else 1408
        assert len(evaluated[final_name]['geometry']['triangles']) == expected_branch_triangles
        assert drawn == fixed_draw + copies * expected_branch_triangles
        assert drawn <= draw_limit
        # Isolate draw-budget rejection even when all instance slots are occupied.
        before = call('inspect')
        edits = [{'op':'mesh','edit':{'kind':'put_instance','instance':dict(i,asset='wood',playback=None,attachment=None)}} for i in instances]
        rejected = call('edit_scene', expected_error=True, edits=edits)
        assert f'{draw_limit} rendered triangles' in str(rejected)
        assert call('inspect') == before

        call('set_camera', camera=CAMERA)
        call('save_scene', path='examples/native-tree-study.scene.json')
        saved = call('inspect')['scene']
        start = call('inspect_mesh_pose', instance='crown-00')['pose']
        capture('rest', 1200, 1100)
        call('step', ticks=60)
        assert call('inspect_mesh_pose', instance='crown-00')['pose']['positions'] != start['positions']
        captured = capture('after', 1200, 1100)
        assert captured['mesh_triangle_count'] == drawn
        call('save_replay', path='artifacts/tree.replay.json')
        assert call('run_replay', path='artifacts/tree.replay.json')['meshes_matched']
        call('capture_replay', replay_path='artifacts/tree.replay.json', path='artifacts/replay.png', width=1200, height=1100)
        assert (OUTPUT/'artifacts/after.png').read_bytes() == (OUTPUT/'artifacts/replay.png').read_bytes()
        call('capture_replay', replay_path='artifacts/tree.replay.json', path='artifacts/prefix.png', tick=0, width=1200, height=1100)
        assert (OUTPUT/'artifacts/rest.png').read_bytes() == (OUTPUT/'artifacts/prefix.png').read_bytes()
        call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'remove_instance', 'id': i['id']}}
                                 for i in instances if i['id'].startswith('crown-')])
        capture('structure', 1200, 1100)
        call('undo')
        near = max((i for i in instances if i['id'].startswith('crown-')),
                   key=lambda i: i['position'][0]*0.59+i['position'][2]*0.81)
        leaf_target = add(near['position'], rotate(near['rotation'], [0, 0.2, 0.35]))
        for name, camera in [
                ('back', {'position': [-11, 7, -15], 'target': CAMERA['target'], 'fov_degrees': 34}),
                ('root-detail', {'position': [1.35, 1.05, 2], 'target': [0, 0.4, 0], 'fov_degrees': 38}),
                ('fork-detail', {'position': [1.4, 2.2, 3.5], 'target': [-0.05, 1.7, 0.1], 'fov_degrees': 40}),
                ('bark-detail', {'position': [1.3, 2.3, 2], 'target': [0.10, 2, 0.1], 'fov_degrees': 32}),
                ('leaf-detail', {'position': add(leaf_target, [1.2, 1.2, 1.7]), 'target': leaf_target, 'fov_degrees': 38})]:
            call('set_camera', camera=camera)
            call('step', ticks=60)
            capture(name)
        call('load_scene', path='examples/native-tree-study.scene.json')
        restored = call('inspect')['scene']
        assert restored['meshes'] == saved['meshes'] and restored['plants'] == saved['plants']
        assert call('inspect_mesh_pose', instance='crown-00')['pose'] == start
        hashes = []
        if frames:
            from PIL import Image
            images = []
            expected = {f'frame-{i:03}.png' for i in range(frames)}
            for old in (OUTPUT/'artifacts/frames').glob('frame-*.png'):
                if old.name not in expected:
                    old.unlink()
            for i in range(frames):
                path = f'artifacts/frames/frame-{i:03}.png'
                call('capture', path=path, width=840, height=770)
                hashes.append(hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest())
                with Image.open(OUTPUT/path) as png:
                    images.append(png.convert('RGB').quantize(colors=192))
                call('step', ticks=240//frames)
            durations = [(round((i+1)*400/frames)-round(i*400/frames))*10 for i in range(frames)]
            images[0].save(OUTPUT/'artifacts/tree.gif', save_all=True, append_images=images[1:],
                           duration=durations, loop=0, disposal=2, optimize=False)
            with Image.open(OUTPUT/'artifacts/tree.gif') as gif:
                assert gif.n_frames == frames
                assert sum((gif.seek(i), gif.info['duration'])[1] for i in range(frames)) == 4000
            assert len(set(hashes)) == frames
            assert call('inspect_mesh_pose', instance='crown-00')['pose'] == start
        shutil.copyfile(OUTPUT/'examples/native-tree-study.scene.json', source_path)
        report = {'build_id': call('inspect')['build_id'], 'native_window': False,
                  'input_sha256': input_digest, 'input_version': input_version,
                  'loaded_version': loaded['version'],
                  'input_target_equals_linear_reference': plant['definition'] == reference,
                  'source': str(source_path), 'source_bytes': source_path.stat().st_size,
                  'assets': len(meshes['assets']), 'instances': len(instances),
                  'stored_vertices': sum(len(a['geometry']['positions']) for a in meshes['assets']),
                  'stored_triangles': sum(len(a['geometry']['triangles']) for a in meshes['assets']),
                  'draw_triangles': drawn, 'draw_headroom': draw_limit-drawn,
                  'cases': reports, 'rejected_cases': rejected_cases, 'final_case': final_name,
                  'historical_source_and_reference_exact': True, 'other_assets_and_instances_exact': True,
                  'wood_including_first_fork_exact': True, 'leaf_pose_at_edit_tick_exact': True,
                  'repeat_and_undo_exact': True, 'save_load_and_replay_exact': True,
                  'prefix_replay_pixels_exact': True,
                  'rejected_budget': rejected, 'frames': frames, 'unique_frames': len(set(hashes)),
                  'gif_bytes': (OUTPUT/'artifacts/tree.gif').stat().st_size if frames else None,
                  'close_camera': close, 'camera': CAMERA, 'light': LIGHT, 'operations': counts}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({k: v for k, v in report.items() if k not in ['operations', 'cases']}, indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames', type=int, default=48)
    parser.add_argument('--segments-y', type=int, choices=[4, 6], default=4)
    parser.add_argument('--input', type=Path, help='Existing tree scene; default is the current project scene')
    args = parser.parse_args()
    if args.frames < 0 or args.frames > 120 or (args.frames and 240 % args.frames):
        parser.error('frames must be0 or a positive divisor of240 up to120')
    run(args.frames, args.segments_y, args.input)
