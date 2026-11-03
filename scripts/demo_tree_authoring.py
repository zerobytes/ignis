#!/usr/bin/env python3
"""Probe native MCP authoring with a more natural tree and comparable views."""
import argparse
from collections import Counter
import hashlib
import json
import math
import os
from pathlib import Path
import random
import shutil

from tree_study_geometry import (add, append, bezier, detailed_blade, geometry,
                                 mul, qmul, quat, rotate, sub, sweep, unit)
from tree_junction_geometry import connected_first_fork, leader_controls
import tree_branch_plant

PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT/'artifacts/tree-study'
(OUTPUT/'artifacts/frames').mkdir(parents=True, exist_ok=True)
(OUTPUT/'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client

SEED = 20261026
CAMERA = {'position': [11, 8, 15], 'target': [0, 3.65, 0], 'fov_degrees': 34}
LIGHT = {'direction': [-0.8, -1, -0.4], 'ambient': 0.28,
         'shadow': {'center': [0, 3.1, 0], 'extent': 5.8, 'resolution': 2048, 'bias': 0.0008}}
SHOOTS = 6
LEAVES_PER_SHOOT = 11


def trunk(t):
    return [0.18*math.sin(t*2.2)+0.10*t, 4.8*t, 0.15*math.sin(t*3)]


def trunk_radius(t):
    return 0.46*(1-t*21/24)**1.25+0.015


def skeleton(call, native_branch=True):
    rng = random.Random(SEED)
    wood = geometry()
    # Root collar is buried and continuous with the trunk, with broad buttresses.
    # The random sequence for upper branches is intentionally independent of roots.
    for _ in range(8):
        rng.uniform(-0.12, 0.12)
        rng.uniform(0.8, 1.1)
    junction_report = None
    branch_plan = None
    clusters = []
    branches = []
    ends = [[-1.55, 5.25, 0.70], [1.55, 4.95, 0.50], [0.2, 6.5, -1.35], [0.30, 6.7, 1.15]]
    for limb, end in enumerate(ends):
        attachment = [0.34, 0.44, 0.59, 0.72][limb]
        start = trunk(attachment)
        controls = [start, add(start, mul(sub(end, start), 0.28)),
                    add(end, [rng.uniform(-0.45, 0.45), -1.15, rng.uniform(-0.4, 0.4)]), end]
        controls[1][1] += 0.45
        p = bezier(controls, 10)
        radius = (0.46*(1-attachment*21/24)**1.25+0.015)*0.70
        if limb == 0:
            wood, junction_report = connected_first_fork(controls, trunk, trunk_radius, call)
        elif limb == 1 and native_branch:
            branch_plan = tree_branch_plant.plan(call, controls, radius)
        else:
            append(wood, sweep(p, [radius*(1-i/9)**1.2+0.004 for i in range(10)], 12, phase=limb))
        dense_path = bezier(controls, 101)
        for j in range(6):
            secondary_range = None
            t = 1 if j == 5 else 0.30+j*0.14
            anchor = dense_path[round(t*100)]
            angle = math.atan2(end[0], end[2])+[-1.0, 2.0, -0.65, -1.8, -0.4, 0.45][j]
            angle += rng.uniform(-0.18, 0.18)
            reach = rng.uniform(0.85, 1.55)
            tip = add(anchor, [math.sin(angle)*reach, rng.uniform(0.25, 0.95), math.cos(angle)*reach])
            if j == 5:
                tip = anchor
            else:
                secondary = bezier([anchor, add(anchor, [math.sin(angle)*0.4, 0.08, math.cos(angle)*0.4]),
                                    add(tip, [0, -0.25, 0]), tip], 5)
                r = min(0.065, (radius*(1-t)**1.2+0.004)*0.68)
                vertex_start, triangle_start = len(wood['positions']), len(wood['triangles'])
                append(wood, sweep(secondary, [r, r*0.75, r*0.5, r*0.26, 0.003], 7, phase=j+limb))
                secondary_range = {'vertices': [vertex_start, len(wood['positions'])],
                                   'triangles': [triangle_start, len(wood['triangles'])]}
            rotation = qmul(quat([0, 1, 0], angle),
                            qmul(quat([1, 0, 0], rng.uniform(-0.45, -0.08)),
                                 quat([0, 0, 1], rng.uniform(-0.30, 0.30))))
            clusters.append({'position': tip, 'rotation': rotation, 'scale': rng.uniform(1.20, 1.47),
                             'variant': (j+limb) % 4, 'offset': len(clusters)*29 % 240})
            branches.append({'parent': limb, 'attachment_t': t, 'start': anchor, 'tip': tip,
                             'geometry_range': secondary_range})
    clusters.append({'position': trunk(1), 'rotation': [0, 0, 0, 1], 'scale': 1.35,
                     'variant': 2, 'offset': 173})
    leader = bezier(leader_controls(trunk), 11)
    for i, scale in [(5, 1.10), (10, 1.20)]:
        clusters.append({'position': leader[i], 'rotation': quat([0, 1, 0], i*0.83),
                         'scale': scale, 'variant': i % 4, 'offset': i*17})
    return wood, clusters, branches, junction_report, branch_plan


def make_cluster(leaf, variant):
    rng = random.Random(SEED+variant*101)
    foliage, twigs = geometry(), geometry()
    bones = [{'name': 'attachment', 'parent': None, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]}]
    weights = []
    for shoot in range(SHOOTS):
        angle = shoot*2.39996+variant*0.37
        end = [math.sin(angle)*rng.uniform(0.42, 0.72), rng.uniform(-0.15, 0.85),
               math.cos(angle)*rng.uniform(0.40, 0.66)+0.27]
        # These controls are collinear: retain the same straight shoot with two
        # rings, assigning the saved vertices to the visible organic junction.
        path = bezier([[0, 0, 0], mul(end, 0.45), end], 2)
        append(twigs, sweep(path, [0.0055, 0.0012], 3, phase=shoot))
        direction = unit(end)
        for k in range(LEAVES_PER_SHOOT):
            t = 0.10+0.89*k/(LEAVES_PER_SHOOT-1)
            anchor = mul(end, 0.9*t+0.1*t*t)
            side = -1 if k % 2 else 1
            yaw = math.atan2(direction[0], direction[2])+side*rng.uniform(0.55, 1.2)
            rotation = qmul(quat([0, 1, 0], yaw),
                            qmul(quat([1, 0, 0], rng.uniform(-0.5, 0.25)),
                                 quat([0, 0, 1], rng.uniform(-0.55, 0.55))))
            scale = rng.uniform(0.72, 1.10)
            origin = add(anchor, rotate(rotation, [0, 0, 0.012*scale]))
            append(foliage, leaf, origin, rotation, scale)
            # A group shares one bending joint; zero tip weight at the petiole
            # holds its authored attachment while the blade flexes.
            if k % 4 == 0:
                joint = len(bones)
                bones.append({'name': f'shoot-{shoot}-group-{k//4}', 'parent': 0,
                              'translation': anchor, 'rotation': [0, 0, 0, 1]})
            for x, y, z in leaf['positions']:
                f = max(0, min(1, z/0.25))**1.7
                weights.append({'joints': [0, joint, 0, 0], 'weights': [1-f, f, 0, 0]})
    tracks = []
    for i in range(1, len(bones)):
        keys = []
        for tick in range(0, 241, 30):
            t = tick*2*math.pi/240+i*0.57+variant*0.31
            rotation = qmul(quat([1, 0, 0], 0.055*math.sin(t)),
                            quat([0, 0, 1], 0.025*math.sin(t+0.7)))
            keys.append({'tick': tick, 'translation': bones[i]['translation'], 'rotation': rotation})
        tracks.append({'bone': i, 'keys': keys})
    return {'geometry': foliage, 'rig': {'bones': bones, 'weights': weights},
            'clips': [{'id': 'breeze', 'duration_ticks': 240, 'tracks': tracks}]}, twigs


def run(frames, native_branch=True):
    c = Client(PROJECT/'target/release/ignis', False)
    operations = Counter()

    def call(op, **kwargs):
        operations[op] += 1
        if op == 'edit_mesh':
            operations['mesh:'+kwargs['edit']['kind']] += 1
        return c.call(op, **kwargs)

    def edit(kind, **kwargs):
        return call('edit_mesh', edit={'kind': kind, **kwargs})

    def capture(name, width=1200, height=1100):
        return call('capture', path=f'artifacts/{name}.png', width=width, height=height)

    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'ignis-tree-authoring-lab', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        shutil.copyfile(PROJECT/'examples/native-tree-study-before.scene.json', OUTPUT/'examples/baseline.scene.json')
        call('load_scene', path='examples/baseline.scene.json')
        call('set_camera', camera=CAMERA)
        call('set_light', light=LIGHT)
        call('step', ticks=60)
        capture('before')

        call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        entities = call('inspect')['scene']['entities']
        for e in entities:
            e['visible'] = False
        call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        call('set_camera', camera=CAMERA)
        call('set_light', light=LIGHT)
        edit('put_asset', asset={'id': 'blade-source', 'geometry': detailed_blade()})
        source = call('inspect_mesh', asset='blade-source')['asset']['geometry']
        # Gentle native grab with explicit scale in metres; avoid overlapping
        # vein strokes on a blade whose full length is only25cm.
        center = source['positions'][-1]
        edit('sculpt', asset='blade-source', stroke={'kind': 'grab', 'centers': [center],
             'radius': 0.065, 'strength': 0.004, 'direction': [0, 1, 0]})
        leaf = call('inspect_mesh', asset='blade-source')['asset']['geometry']
        call('undo')
        assert call('inspect_mesh', asset='blade-source')['asset']['geometry'] == source
        call('redo')
        assert call('inspect_mesh', asset='blade-source')['asset']['geometry'] == leaf

        wood, placements, branches, junction_report, branch_plan = skeleton(call, native_branch)
        assets, twigs = [], []
        for variant in range(4):
            asset, twig = make_cluster(leaf, variant)
            asset['id'] = f'foliage-{variant}'
            assets.append(asset)
            twigs.append(twig)
        for spec in placements:
            append(wood, twigs[spec['variant']], spec['position'], spec['rotation'], spec['scale'])
        if branch_plan:
            assets.append(branch_plan['asset'])
        assets.append({'id': 'wood', 'geometry': wood})
        # A real opaque soil patch intersects the buried collar. It adds no
        # simulated collision or painted shadow to the visual asset.
        soil = geometry()
        soil['positions'].append([0, 0.025, 0])
        for j in range(48):
            a = 2*math.pi*j/48
            radius = 2.2+0.12*math.sin(j*1.9)
            soil['positions'].append([math.sin(a)*radius, 0.025, math.cos(a)*radius])
        soil['triangles'] = [[0, j+1, (j+1) % 48+1] for j in range(48)]
        assets.append({'id': 'soil', 'geometry': soil})
        drawn = len(wood['triangles'])+len(soil['triangles'])
        if branch_plan:
            drawn += branch_plan['preview_triangles']
        drawn += sum(len(assets[p['variant']]['geometry']['triangles']) for p in placements)
        stored_vertices = len(leaf['positions'])+sum(len(a['geometry']['positions']) for a in assets)
        assert drawn <= 65536, drawn
        assert stored_vertices <= 16384, stored_vertices
        assert all(len(a['geometry']['positions']) <= 4096 for a in assets)
        for asset in assets:
            if branch_plan and asset['id'] == branch_plan['definition']['id']:
                tree_branch_plant.put(call, branch_plan['definition'])
                assert call('inspect_mesh', asset=asset['id'])['asset']['geometry'] == asset['geometry']
                edit('set_material_coordinates', asset=asset['id'], coordinates=asset['geometry']['positions'])
            else:
                edit('put_asset', asset={'id': asset['id'], 'geometry': asset['geometry']})
            edit('set_shading', asset=asset['id'], shading='flat' if asset['id'] == 'soil' else 'smooth')
            pattern = {'frequency': [45, 45, 45], 'low': [0.80, 0.87, 0.72], 'high': [1.12, 1.12, 0.96],
                       'bump': 0.0004, 'warp': 0.2, 'octaves': 3, 'seed': 29, 'contrast': 1.3, 'kind': 'noise'}
            transmission = 0.30
            if asset['id'] in ['wood', 'branch-editable-1']:
                pattern.update(frequency=[24, 4, 24], low=[0.62, 0.60, 0.55], high=[1.10, 1.05, 0.97],
                               bump=0.0014, warp=1.1, octaves=3, seed=137, contrast=1.2, kind='cellular')
                transmission = 0
            elif asset['id'] == 'soil':
                pattern.update(frequency=[22, 22, 22], low=[0.55, 0.55, 0.48], high=[1.35, 1.30, 1.15],
                               bump=0.003, warp=0.3, octaves=4, seed=99, contrast=1.6)
                transmission = 0
            edit('set_material', asset=asset['id'], material={'pattern': pattern, 'transmission': transmission})
            if asset.get('rig'):
                edit('set_rig', asset=asset['id'], rig=asset['rig'])
                for clip in asset['clips']:
                    edit('put_clip', asset=asset['id'], clip=clip)
        topology = call('inspect_mesh_topology', asset='wood')['topology']
        assert topology['boundary_edges'] == 0 and topology['non_manifold_edges'] == 0
        if branch_plan:
            topology = call('inspect_mesh_topology', asset=branch_plan['definition']['id'])['topology']
            assert topology['boundary_edges'] == topology['non_manifold_edges'] == 0
            assert not topology['isolated_vertices']
        instances = []
        instances.append({'id': 'tree-wood', 'asset': 'wood', 'color': [0.15, 0.13, 0.10],
                          'position': [0, 0, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'playback': None})
        instances.append({'id': 'tree-soil', 'asset': 'soil', 'color': [0.075, 0.058, 0.035],
                          'position': [0, 0, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'playback': None})
        if branch_plan:
            instances.append(branch_plan['instance'])
        colors = [[0.070, 0.260, 0.028], [0.080, 0.280, 0.032],
                  [0.085, 0.285, 0.034], [0.075, 0.270, 0.030]]
        for j, p in enumerate(placements):
            color = mul(colors[(j*7+j//3) % 4], 0.95+0.09*math.sin(j*2.4))
            instances.append({'id': f'crown-{j:02}', 'asset': f'foliage-{p["variant"]}', 'color': color,
                              'position': p['position'], 'rotation': p['rotation'], 'scale': p['scale'],
                              'playback': {'clip': 'breeze', 'offset_ticks': p['offset'], 'looping': True}})
        for i in instances:
            edit('put_instance', instance=i)
        branch_report = tree_branch_plant.exercise(call, branch_plan, capture, CAMERA) if branch_plan else None
        # Saved scene JSON uses compact f32 numbers; structured inspection uses
        # widened f64 JSON. Compare the same API representation, without epsilon.
        baseline = OUTPUT/'artifacts/legacy-branch-mesh-inspection.json'
        comparison = {'available': bool(branch_plan and baseline.exists())}
        if comparison['available']:
            previous = json.loads(baseline.read_text())
            current = call('inspect')['scene']['meshes']
            old_wood = next(a for a in previous['assets'] if a['id'] == 'wood')['geometry']
            new_wood = next(a for a in current['assets'] if a['id'] == 'wood')['geometry']
            for channel, count in [('positions', junction_report['assembly_vertices']),
                                   ('material_coordinates', junction_report['assembly_vertices']),
                                   ('triangles', junction_report['assembly_triangles'])]:
                mismatches = [(i, a, b) for i, (a, b) in enumerate(zip(old_wood[channel][:count], new_wood[channel][:count])) if a != b]
                assert not mismatches, (channel, len(mismatches), mismatches[:3])
            for asset in previous['assets']:
                if asset['id'] != 'wood':
                    assert asset == next(a for a in current['assets'] if a['id'] == asset['id'])
            assert previous['instances'] == [i for i in current['instances'] if i['id'] != branch_plan['instance']['id']]
            comparison.update(shared_fork_exact=True, other_assets_and_instances_exact=True)
        # Extra crown sets keep the32-instance cap valid, but exceed draw.
        before_rejection = call('inspect')
        probes = []
        probe_instance = next(i for i in instances if i['id'].startswith('crown-'))
        for j in range(min(3, 32-len(instances))):
            probe = dict(probe_instance, id=f'budget-probe-{j}')
            probes.append({'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': probe}})
        rejected_budget = call('edit_scene', expected_error=True, edits=probes)
        assert '65536 rendered triangles' in str(rejected_budget), rejected_budget
        assert call('inspect') == before_rejection
        call('save_scene', path='examples/native-tree-study.scene.json')
        saved_meshes = call('inspect')['scene']['meshes']
        if not native_branch:
            call('save_scene', path='examples/native-tree-study-legacy-branch.scene.json')
            baseline.write_text(json.dumps(saved_meshes)+'\n')
        shutil.copyfile(OUTPUT/'examples/native-tree-study.scene.json', PROJECT/'examples/native-tree-study.scene.json')
        start = call('inspect_mesh_pose', instance='crown-00')['pose']
        call('step', ticks=60)
        assert call('inspect_mesh_pose', instance='crown-00')['pose']['positions'] != start['positions']
        native_capture = capture('after')
        assert native_capture['mesh_triangle_count'] == drawn
        assert native_capture['mesh_vertex_count'] == drawn*3
        call('save_replay', path='artifacts/tree.replay.json')
        assert call('run_replay', path='artifacts/tree.replay.json')['meshes_matched']
        call('capture_replay', replay_path='artifacts/tree.replay.json', path='artifacts/replay.png', width=1200, height=1100)
        assert (OUTPUT/'artifacts/after.png').read_bytes() == (OUTPUT/'artifacts/replay.png').read_bytes()

        call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'remove_instance', 'id': i['id']}}
                                 for i in instances if i['id'].startswith('crown-')])
        capture('structure')
        call('undo')

        for name, camera in [
                ('back', {'position': [-11, 7.0, -15], 'target': CAMERA['target'], 'fov_degrees': 34}),
                ('root-detail', {'position': [1.35, 1.05, 2.0], 'target': [0.0, 0.4, 0.0], 'fov_degrees': 38}),
                ('fork-detail', {'position': [1.4, 2.2, 3.5], 'target': [-0.05, 1.7, 0.1], 'fov_degrees': 40}),
                ('bark-detail', {'position': [1.3, 2.3, 2.0], 'target': [0.10, 2.0, 0.1], 'fov_degrees': 32})]:
            call('set_camera', camera=camera)
            call('step', ticks=60)
            capture(name, 1000, 900)
        near = max(placements, key=lambda p: p['position'][0]*0.59+p['position'][2]*0.81)
        target = add(near['position'], rotate(near['rotation'], [0, 0.2, 0.35]))
        call('set_camera', camera={'position': add(target, [1.2, 1.2, 1.7]), 'target': target, 'fov_degrees': 38})
        call('step', ticks=60)
        capture('leaf-detail', 1000, 900)
        call('load_scene', path='examples/native-tree-study.scene.json')
        assert call('inspect')['scene']['meshes'] == saved_meshes
        assert call('inspect_mesh_pose', instance='crown-00')['pose'] == start

        hashes = []
        gif = None
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
            gif = OUTPUT/'artifacts/tree.gif'
            durations = [(round((i+1)*400/frames)-round(i*400/frames))*10 for i in range(frames)]
            images[0].save(gif, save_all=True, append_images=images[1:], duration=durations,
                           loop=0, disposal=2, optimize=False)
            with Image.open(gif) as preview:
                assert preview.n_frames == frames
                assert sum((preview.seek(i), preview.info['duration'])[1] for i in range(frames)) == 4000
            assert len(set(hashes)) == frames
            assert call('inspect_mesh_pose', instance='crown-00')['pose'] == start
        report = {'build_id': call('inspect')['build_id'], 'seed': SEED, 'native_window': False,
                  'source': str(PROJECT/'examples/native-tree-study.scene.json'), 'leaf_count': len(placements)*SHOOTS*LEAVES_PER_SHOOT,
                  'assets': len(assets)+1, 'instances': len(instances), 'draw_triangles': drawn,
                  'captured_mesh_triangles': native_capture['mesh_triangle_count'],
                  'captured_mesh_vertices': native_capture['mesh_vertex_count'],
                  'stored_vertices': stored_vertices, 'stored_triangles': len(leaf['triangles'])+sum(len(a['geometry']['triangles']) for a in assets),
                  'leaf_vertices': len(leaf['positions']), 'leaf_triangles': len(leaf['triangles']),
                  'bones_per_cluster': len(assets[0]['rig']['bones']), 'branches': branches,
                  'junction': junction_report,
                  'editable_branch': branch_report, 'branch_comparison': comparison,
                  'rejected_budget': rejected_budget,
                  'camera': CAMERA, 'light': LIGHT, 'operations': dict(operations),
                  'gif': str(gif) if gif else None, 'frames': frames, 'unique_frames': len(set(hashes)),
                  'checks': ['native junction query read-only', 'first fork assembly closed, oriented and connected',
                             'six primary attachment points preserved', 'sculpt undo/redo exact', 'draw budget rejection atomic', 'animated positions differ', 'save/load pose exact',
                             'live/replay pixels exact', 'loop pose exact' if frames else 'stills only']}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({k: v for k, v in report.items() if k not in ['branches', 'operations']}, indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames', type=int, default=48)
    parser.add_argument('--legacy-branch', action='store_true', help='Keep the old baked limb1 for comparison')
    parser.add_argument('--compare-branch', action='store_true', help='Capture a fresh baked baseline before native branch publication')
    args = parser.parse_args()
    if args.frames < 0 or args.frames > 120 or (args.frames and 240 % args.frames):
        parser.error('frames must be0 or a positive divisor of240 up to120')
    if args.compare_branch and args.legacy_branch:
        parser.error('choose --compare-branch or --legacy-branch')
    if args.compare_branch:
        run(0, native_branch=False)
        shutil.copyfile(OUTPUT/'artifacts/after.png', OUTPUT/'artifacts/branch-before.png')
    run(args.frames, native_branch=not args.legacy_branch)
