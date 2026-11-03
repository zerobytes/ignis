#!/usr/bin/env python3
"""Create and animate a reusable tree through Ignis native MCP commands."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil

PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT/'artifacts/tree-demo'
(OUTPUT/'artifacts/frames').mkdir(parents=True, exist_ok=True)
(OUTPUT/'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client


def add(a, b):
    return [x+y for x, y in zip(a, b)]


def mul(a, s):
    return [x*s for x in a]


def unit(a):
    return mul(a, 1/math.sqrt(sum(x*x for x in a)))


def cross(a, b):
    return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]


def qmul(a, b):
    x, y, z, w = a
    X, Y, Z, W = b
    return [w*X+x*W+y*Z-z*Y, w*Y-x*Z+y*W+z*X,
            w*Z+x*Y-y*X+z*W, w*W-x*X-y*Y-z*Z]


def quat(axis, angle):
    return [v*math.sin(angle/2) for v in axis]+[math.cos(angle/2)]


def rotate(q, p):
    return qmul(qmul(q, p+[0]), [-q[0], -q[1], -q[2], q[3]])[:3]


def transform(p, position, rotation, scale=1):
    return add(position, rotate(rotation, mul(p, scale)))


def empty_geometry():
    return {'positions': [], 'triangles': []}


def append_geometry(dst, src, position=(0, 0, 0), rotation=(0, 0, 0, 1), scale=1):
    offset = len(dst['positions'])
    dst['positions'].extend(transform(p, position, rotation, scale) for p in src['positions'])
    dst['triangles'].extend([offset+i for i in face] for face in src['triangles'])


def tube(dst, path, radii, sides=8):
    """Explicit capped tube mesh; adjoining tubes overlap at branch junctions."""
    first = len(dst['positions'])
    for i, (p, radius) in enumerate(zip(path, radii)):
        direction = unit([b-a for a, b in zip(path[max(0, i-1)], path[min(len(path)-1, i+1)])])
        helper = [0, 1, 0] if abs(direction[1]) < 0.9 else [1, 0, 0]
        u = unit(cross(direction, helper))
        v = cross(direction, u)
        for j in range(sides):
            t = j*2*math.pi/sides
            # Consistent angular frame, with small authored bark facets.
            r = radius*(1+0.035*math.sin(j*2.4+i*0.7))
            dst['positions'].append(add(p, add(mul(u, r*math.cos(t)), mul(v, r*math.sin(t)))))
    for i in range(len(path)-1):
        for j in range(sides):
            a, b = first+i*sides+j, first+i*sides+(j+1) % sides
            dst['triangles'].extend([[a, b, a+sides], [b, b+sides, a+sides]])
    for ring, reverse in [(first, True), (first+(len(path)-1)*sides, False)]:
        for j in range(1, sides-1):
            face = [ring, ring+j, ring+j+1]
            dst['triangles'].append(face[::-1] if reverse else face)


def leaf_geometry():
    rows = [(-0.55, 0.045), (0, 0.10), (0.35, 0.45), (0.8, 0.90),
            (1.35, 1.22), (1.95, 1.28), (2.55, 1.10), (3.1, 0.80),
            (3.55, 0.43), (3.9, 0.15)]
    g = empty_geometry()
    for z, width in rows:
        for u in [-1, 0, 1]:
            camber = 0.13*(1-u*u)*math.sin(max(0, z)*math.pi/4.15)
            g['positions'].append([u*width, camber+0.025*z*z, z])
    for row in range(len(rows)-1):
        for x in range(2):
            a = row*3+x
            g['triangles'].extend([[a, a+3, a+1], [a+1, a+3, a+4]])
    tip = len(g['positions'])
    g['positions'].append([0, 0.025*4.15**2, 4.15])
    g['triangles'].extend([[27+x, tip, 28+x] for x in range(2)])
    return g


def layout():
    sprays = []
    # Asymmetric rings form a broad crown with a visible lower trunk.
    for tier, (count, height, reach) in enumerate([(5, 2.9, 1.65), (6, 3.9, 1.80),
                                                  (5, 4.95, 1.35), (2, 6.0, 0.38)]):
        for j in range(count):
            angle = j*2*math.pi/count+tier*1.1+0.17*math.sin(j*2)
            start_height = height-0.65+0.32*math.sin(j*2.3+tier)
            start = [0.13*math.sin(start_height), start_height, 0.10*math.cos(start_height)]
            reach *= 1+0.13*math.sin(j*2.7+tier)
            end = [math.sin(angle)*reach, height+0.30*math.sin(j*3+tier), math.cos(angle)*reach]
            rotation = qmul(quat([0, 1, 0], angle),
                            qmul(quat([1, 0, 0], -0.20-tier*0.09),
                                 quat([0, 0, 1], 0.20*math.sin(j*2.5+tier))))
            sprays.append({'position': end, 'rotation': rotation,
                           'scale': 0.85+0.10*math.sin(j*1.8+tier),
                           'offset': (len(sprays)*37) % 240, 'start': start})
    return sprays


def wood_geometry(sprays):
    g = empty_geometry()
    heights = [0, 0.3, 0.8, 1.5, 2.3, 3.1, 4.0, 4.9, 5.7, 6.35, 6.9]
    tube(g, [[0.13*math.sin(h), h, 0.10*math.cos(h)] for h in heights],
         [0.48, 0.39, 0.32, 0.28, 0.24, 0.20, 0.16, 0.13, 0.09, 0.055, 0.016], 12)
    for j in range(7):
        a = j*2*math.pi/7
        reach = 0.95+0.22*math.sin(j*2)
        tube(g, [[0, 0.40, 0.10], [0.32*math.sin(a), 0.17, 0.32*math.cos(a)],
                 [reach*math.sin(a), 0.035, reach*math.cos(a)]], [0.18, 0.12, 0.015])
    for spray in sprays:
        a, b = spray['start'], spray['position']
        path = [add(mul(a, 1-t), mul(b, t)) for t in [0, 0.2, 0.45, 0.7, 1]]
        for i, t in enumerate([0, 0.2, 0.45, 0.7, 1]):
            path[i][1] -= 0.16*math.sin(t*math.pi)
        tube(g, path, [0.125*(1-0.07*a[1]), 0.095, 0.070, 0.040, 0.018])
    return g


def run(frames):
    c = Client(PROJECT/'target/release/ignis', False)
    operations = []

    def call(op, **kwargs):
        result = c.call(op, **kwargs)
        if op == 'edit_mesh':
            operations.append(kwargs['edit']['kind'])
        return result

    def edit(kind, **kwargs):
        return call('edit_mesh', edit={'kind': kind, **kwargs})

    def instance(name, asset, color, position=(0, 0, 0), rotation=(0, 0, 0, 1), scale=1, offset=None):
        playback = None if offset is None else {'clip': 'breeze', 'offset_ticks': offset, 'looping': True}
        edit('put_instance', instance={'id': name, 'asset': asset, 'color': color,
                                      'position': position, 'rotation': rotation, 'scale': scale,
                                      'playback': playback})

    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'ignis-native-tree-demo', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        entities = call('inspect')['scene']['entities']
        for entity in entities:
            entity['visible'] = False
        call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        camera = {'position': [11.0, 8.0, 14.0], 'target': [0, 3.65, 0], 'fov_degrees': 34}
        call('set_camera', camera=camera)
        call('set_light', light={'direction': [-0.6, -1, -0.3], 'ambient': 0.30})

        edit('put_asset', asset={'id': 'leaf-source', 'geometry': leaf_geometry()})
        topology = call('inspect_mesh_topology', asset='leaf-source')
        edit('subdivide', asset='leaf-source', reference=topology['reference'], components=None)
        for kind, radius, strength, targets in [
                ('draw', 0.45, 0.018, [(0, 0.8), (0, 1.9), (0, 3.1)]),
                ('grab', 0.65, 0.05, [(0, 4.15)]),
                ('smooth', 0.75, 0.12, [(0, 1.4), (0, 2.7)])]:
            points = call('inspect_mesh', asset='leaf-source')['asset']['geometry']['positions']
            centers = [min(points, key=lambda p: (p[0]-x)**2+(p[2]-z)**2) for x, z in targets]
            edit('sculpt', asset='leaf-source', stroke={'kind': kind, 'centers': centers,
                 'radius': radius, 'strength': strength, 'direction': [0, 1, 0]})
        leaf = call('inspect_mesh', asset='leaf-source')['asset']['geometry']
        call('undo')
        assert call('inspect_mesh', asset='leaf-source')['asset']['geometry'] != leaf
        call('redo')
        assert call('inspect_mesh', asset='leaf-source')['asset']['geometry'] == leaf

        spray, twigs = empty_geometry(), empty_geometry()
        bones = [{'name': 'spray-root', 'parent': None, 'translation': [0, 0, 0], 'rotation': [0, 0, 0, 1]}]
        weights = []
        tube(twigs, [[0, 0, 0], [0, 0.035, 0.55], [0, 0.08, 1.15]], [0.025, 0.018, 0.008], 6)
        tube(twigs, [[0, 0.035, 0.45], [-0.30, 0.09, 0.85], [-0.48, 0.16, 1.25]], [0.016, 0.011, 0.005], 5)
        tube(twigs, [[0, 0.045, 0.60], [0.30, 0.12, 1.0], [0.48, 0.22, 1.48]], [0.016, 0.011, 0.005], 5)
        leaf_anchors = [(0, 0.20, -1.1), (0, 0.43, 1.05), (0, 0.70, -0.9),
                        (0, 0.93, 0.9), (0, 1.15, 0), (-0.30, 0.85, -1.5),
                        (-0.48, 1.25, -0.45), (0.30, 1.0, 1.5), (0.48, 1.48, 0.45)]
        for j, (x, z, angle) in enumerate(leaf_anchors):
            rotation = qmul(quat([0, 1, 0], angle),
                            qmul(quat([1, 0, 0], -0.14+0.07*math.sin(j*3)),
                                 quat([0, 0, 1], 0.15*math.sin(j*2))))
            scale = 0.22+0.025*math.sin(j*2.1)
            anchor = [x, 0.07*z+0.20*abs(x), z]
            origin = add(anchor, rotate(rotation, [0, 0, 0.55*scale]))
            append_geometry(spray, leaf, origin, rotation, scale)
            tube(twigs, [anchor, add(origin, rotate(rotation, [0, 0, 0.12*scale]))], [0.009, 0.004], 5)
            root = len(bones)
            bones.append({'name': f'leaf-{j}-base', 'parent': 0, 'translation': origin, 'rotation': rotation})
            bones.append({'name': f'leaf-{j}-tip', 'parent': root, 'translation': [0, 0, 2*scale], 'rotation': [0, 0, 0, 1]})
            for x, y, z0 in leaf['positions']:
                tip = max(0, min(1, z0/3.8))
                weights.append({'joints': [root, root+1, 0, 0], 'weights': [1-tip, tip, 0, 0]})
        edit('put_asset', asset={'id': 'leaf-spray', 'geometry': spray})
        edit('set_rig', asset='leaf-spray', rig={'bones': bones, 'weights': weights})
        tracks = []
        for i, bone in enumerate(bones):
            keys = []
            amplitude = 0.025 if i == 0 else (0.055 if i % 2 else 0.14)
            for tick in range(0, 241, 30):
                phase = tick*2*math.pi/240+i*0.47
                sway = qmul(quat([1, 0, 0], amplitude*math.sin(phase)),
                            quat([0, 0, 1], amplitude*0.35*math.cos(phase)))
                keys.append({'tick': tick, 'translation': bone['translation'],
                             'rotation': qmul(bone['rotation'], sway)})
            tracks.append({'bone': i, 'keys': keys})
        edit('put_clip', asset='leaf-spray', clip={'id': 'breeze', 'duration_ticks': 240, 'tracks': tracks})

        sprays = layout()
        wood = wood_geometry(sprays)
        # Bake petiole placement into the wood asset; no extra per-spray instances.
        for spec in sprays:
            append_geometry(wood, twigs, spec['position'], spec['rotation'], spec['scale'])
        assert len(wood['positions']) <= 4096
        draw = len(wood['triangles'])+len(spray['triangles'])*len(sprays)
        assert draw <= 32768
        edit('put_asset', asset={'id': 'wood', 'geometry': wood})
        instance('tree-wood', 'wood', [0.26, 0.115, 0.040])
        for j, spec in enumerate(sprays):
            color = [0.065+0.02*math.sin(j*1.7), 0.33+0.10*(0.5+0.5*math.sin(j*2.1)), 0.105+0.02*math.sin(j)]
            instance(f'crown-{j:02}', 'leaf-spray', color, spec['position'], spec['rotation'], spec['scale'], spec['offset'])
        call('save_scene', path='examples/native-tree.scene.json')
        shutil.copyfile(OUTPUT/'examples/native-tree.scene.json', PROJECT/'examples/native-tree.scene.json')
        start = call('inspect_mesh_pose', instance='crown-00')['pose']
        call('step', ticks=60)
        animated = call('inspect_mesh_pose', instance='crown-00')['pose']
        assert animated['positions'] != start['positions']
        call('capture', path='artifacts/tree.png', width=1200, height=1200)
        call('save_replay', path='artifacts/tree.replay.json')
        assert call('run_replay', path='artifacts/tree.replay.json')['meshes_matched']
        call('capture_replay', replay_path='artifacts/tree.replay.json', path='artifacts/tree-replay.png', width=1200, height=1200)
        assert (OUTPUT/'artifacts/tree.png').read_bytes() == (OUTPUT/'artifacts/tree-replay.png').read_bytes()
        call('load_scene', path='examples/native-tree.scene.json')
        assert call('inspect_mesh_pose', instance='crown-00')['pose'] == start

        gif = None
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
                call('capture', path=path, width=800, height=800)
                hashes.append(hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest())
                with Image.open(OUTPUT/path) as png:
                    images.append(png.convert('RGB').quantize(colors=192))
                call('step', ticks=240//frames)
            gif = OUTPUT/'artifacts/tree.gif'
            durations = [(round((i+1)*400/frames)-round(i*400/frames))*10 for i in range(frames)]
            images[0].save(gif, save_all=True, append_images=images[1:], loop=0,
                           duration=durations, disposal=2, optimize=False)
            with Image.open(gif) as preview:
                assert preview.n_frames == frames
                assert sum((preview.seek(i), preview.info['duration'])[1] for i in range(frames)) == 4000
            assert len(set(hashes)) == frames
            assert call('inspect_mesh_pose', instance='crown-00')['pose'] == start
        report = {'build_id': call('inspect')['build_id'], 'native_window': False,
                  'source': str(PROJECT/'examples/native-tree.scene.json'), 'leaf_count': len(sprays)*len(leaf_anchors),
                  'instances': len(sprays)+1, 'assets': 3, 'draw_triangles': draw,
                  'source_leaf': [len(leaf['positions']), len(leaf['triangles'])],
                  'spray': [len(spray['positions']), len(spray['triangles'])],
                  'wood': [len(wood['positions']), len(wood['triangles'])],
                  'bones_per_spray': len(bones), 'clip': 'breeze', 'duration_ticks': 240,
                  'png': str(OUTPUT/'artifacts/tree.png'), 'gif': str(gif) if gif else None,
                  'frames': frames, 'unique_frames': len(set(hashes)), 'operations': operations,
                  'checks': ['sculpt undo/redo exact', 'animated poses differ', 'save/load pose exact',
                             'live/replay pixels exact', 'loop pose exact' if frames else 'stills only']}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({k: v for k, v in report.items() if k != 'operations'}, indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames', type=int, default=48, help='0 for stills; otherwise a divisor of240, at most120')
    args = parser.parse_args()
    if args.frames < 0 or args.frames > 120 or (args.frames and 240 % args.frames):
        parser.error('frames must be0 or a positive divisor of240 up to120')
    run(args.frames)
