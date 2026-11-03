#!/usr/bin/env python3
"""Author, sculpt, rig and capture a stylized leaf using native MCP operations."""
import argparse
import json
import math
import os
from pathlib import Path
import shutil

PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT / 'artifacts/leaf-demo'
(OUTPUT/'artifacts/frames').mkdir(parents=True, exist_ok=True)
(OUTPUT/'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client

ROWS = [(-0.55, 0.045), (0, 0.10), (0.35, 0.45), (0.8, 0.90),
        (1.35, 1.22), (1.95, 1.28), (2.55, 1.10), (3.1, 0.80),
        (3.55, 0.43), (3.9, 0.15)]


def base_geometry():
    positions = [[u*w, 0, z] for z, w in ROWS for u in [-1, -0.5, 0, 0.5, 1]]
    triangles = []
    for row in range(len(ROWS)-1):
        for x in range(4):
            a = row*5+x
            triangles.extend([[a, a+5, a+1], [a+1, a+5, a+6]])
    tip = len(positions)
    positions.append([0, 0, 4.15])
    for x in range(4):
        triangles.append([(len(ROWS)-1)*5+x, tip, (len(ROWS)-1)*5+x+1])
    return {'positions': positions, 'triangles': triangles}


def quat_xz(x, z):
    # Quaternion product qx*qz in XYZW order.
    sx, cx, sz, cz = math.sin(x/2), math.cos(x/2), math.sin(z/2), math.cos(z/2)
    return [sx*cz, -sx*sz, cx*sz, cx*cz]


def run(frames):
    c = Client(PROJECT/'target/release/ignis', False)
    operations = []

    def call(op, **kwargs):
        result = c.call(op, **kwargs)
        if op in ['edit_mesh', 'edit_scene']:
            operations.append({'op': op, 'kind': kwargs.get('edit', {}).get('kind')})
        return result

    def inspect():
        return call('inspect_mesh', asset='leaf')['asset']

    def nearby(points, x, z):
        return min(points, key=lambda p: (p[0]-x)**2+(p[2]-z)**2)

    def sculpt(kind, targets, radius, strength, direction=(0, 1, 0)):
        points = inspect()['geometry']['positions']
        centers = [nearby(points, x, z) for x, z in targets]
        call('edit_mesh', edit={'kind': 'sculpt', 'asset': 'leaf', 'stroke': {
            'kind': kind, 'centers': centers, 'radius': radius,
            'strength': strength, 'direction': direction}})

    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'ignis-native-leaf-demo', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('generate', recipe={'seed': 7, 'width': 20, 'depth': 20, 'obstacles': 0})
        entities = call('inspect')['scene']['entities']
        for entity in entities:
            entity['visible'] = False
        call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        call('set_camera', camera={'position': [2.8, 7.2, 4.4], 'target': [0, 1.25, 0.15], 'fov_degrees': 28})
        call('set_light', light={'direction': [-0.6, -1, 0.4], 'ambient': 0.20})
        base = base_geometry()
        call('edit_mesh', edit={'kind': 'put_asset', 'asset': {'id': 'leaf', 'geometry': base}})
        stages = [{'stage': 'base', 'vertices': len(base['positions']), 'triangles': len(base['triangles'])}]
        for level in range(2):
            topology = call('inspect_mesh_topology', asset='leaf')
            call('edit_mesh', edit={'kind': 'subdivide', 'asset': 'leaf',
                                   'reference': topology['reference'], 'components': None})
            geometry = inspect()['geometry']
            stages.append({'stage': f'subdivision_{level+1}', 'vertices': len(geometry['positions']), 'triangles': len(geometry['triangles'])})
        instance = {'id': 'leaf-display', 'asset': 'leaf', 'position': [0, 1.2, -1.8],
                    'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [0.055, 0.42, 0.21], 'playback': None}
        call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        call('capture', path='artifacts/leaf-base.png', width=1080, height=840)

        sculpt('draw', [(0, z) for z in [0.4, 1.1, 1.8, 2.5, 3.2, 3.7]], 0.95, 0.12)
        sculpt('grab', [(-w, z) for z, w in ROWS[3:9]], 0.48, -0.14)
        sculpt('grab', [(w, z) for z, w in ROWS[3:9]], 0.48, -0.085)
        sculpt('grab', [(0, 4.15)], 0.85, 0.26, (0.12, 1, 0))
        sculpt('draw', [(0, -0.2+i*0.24) for i in range(18)], 0.17, 0.07)
        ribs = []
        for side in [-1, 1]:
            for z, width in [(0.8, 0.9), (1.35, 1.22), (1.95, 1.28), (2.55, 1.1), (3.1, 0.8)]:
                for u in [0.25, 0.45, 0.65, 0.85]:
                    ribs.append((side*u*width, z+u*0.27))
        sculpt('draw', ribs, 0.13, 0.035)
        sculpted = inspect()
        assert sculpted['geometry']['triangles'] == geometry['triangles']
        call('undo')
        assert inspect()['geometry']['positions'] != sculpted['geometry']['positions']
        call('redo')
        assert inspect() == sculpted
        call('capture', path='artifacts/leaf-sculpt.png', width=1080, height=840)

        bones = [{'name': name, 'parent': i-1 if i else None,
                  'translation': [0, 0, 1.5 if i else 0], 'rotation': [0, 0, 0, 1]}
                 for i, name in enumerate(['stem', 'middle', 'tip'])]
        weights = []
        for x, y, z in sculpted['geometry']['positions']:
            t = max(0, min(2, z/1.5))
            low = min(1, int(t))
            fraction = t-low
            weights.append({'joints': [low, low+1, 0, 0], 'weights': [1-fraction, fraction, 0, 0]})
        call('edit_mesh', edit={'kind': 'set_rig', 'asset': 'leaf', 'rig': {'bones': bones, 'weights': weights}})
        tracks = []
        for i, (amplitude, phase) in enumerate([(0.035, 0), (0.13, 0.4), (0.24, 1.0)]):
            keys = []
            for tick in range(0, 241, 30):
                t = 2*math.pi*tick/240
                keys.append({'tick': tick, 'translation': bones[i]['translation'],
                             'rotation': quat_xz(-amplitude*math.sin(t+phase), 0.04*i*math.sin(t))})
            tracks.append({'bone': i, 'keys': keys})
        call('edit_mesh', edit={'kind': 'put_clip', 'asset': 'leaf',
                               'clip': {'id': 'breeze', 'duration_ticks': 240, 'tracks': tracks}})
        instance['playback'] = {'clip': 'breeze', 'offset_ticks': 0, 'looping': True}
        call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        call('save_scene', path='examples/native-leaf.scene.json')
        source = OUTPUT/'examples/native-leaf.scene.json'
        shutil.copyfile(source, PROJECT/'examples/native-leaf.scene.json')
        start = call('inspect_mesh_pose', instance='leaf-display')['pose']
        call('capture', path='artifacts/leaf-start.png', width=1080, height=840)
        call('step', ticks=60)
        animated = call('inspect_mesh_pose', instance='leaf-display')['pose']
        assert animated['positions'] != start['positions']
        call('capture', path='artifacts/leaf.png', width=1080, height=840)
        call('save_replay', path='artifacts/leaf.replay.json')
        assert call('run_replay', path='artifacts/leaf.replay.json')['meshes_matched']
        call('capture_replay', replay_path='artifacts/leaf.replay.json', path='artifacts/leaf-replay.png', width=1080, height=840)
        assert (OUTPUT/'artifacts/leaf.png').read_bytes() == (OUTPUT/'artifacts/leaf-replay.png').read_bytes()
        call('load_scene', path='examples/native-leaf.scene.json')
        assert call('inspect_mesh_pose', instance='leaf-display')['pose'] == start

        gif = None
        if frames:
            from PIL import Image
            images = []
            step = 240//frames
            expected = {f'frame-{i:03}.png' for i in range(frames)}
            for old in (OUTPUT/'artifacts/frames').glob('frame-*.png'):
                if old.name not in expected:
                    old.unlink()
            for i in range(frames):
                path = f'artifacts/frames/frame-{i:03}.png'
                call('capture', path=path, width=720, height=640)
                with Image.open(OUTPUT/path) as png:
                    images.append(png.convert('RGB').quantize(colors=192))
                call('step', ticks=step)
            gif = OUTPUT/'artifacts/leaf.gif'
            images[0].save(gif, save_all=True, append_images=images[1:], loop=0,
                           duration=[(round((i+1)*400/frames)-round(i*400/frames))*10 for i in range(frames)],
                           disposal=2, optimize=False)
        final = inspect()
        report = {'build_id': call('inspect')['build_id'], 'native_window': False,
                  'source': str(PROJECT/'examples/native-leaf.scene.json'), 'stages': stages,
                  'bones': len(final['rig']['bones']), 'clip': 'breeze',
                  'png': str(OUTPUT/'artifacts/leaf.png'), 'gif': str(gif) if gif else None,
                  'frames': frames, 'duration_ticks': 240, 'operations': operations,
                  'checks': ['sculpt undo/redo exact', 'animated poses differ', 'save/load pose exact', 'live/replay pixels exact']}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({k: v for k, v in report.items() if k != 'operations'}, indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames', type=int, default=48, help='0 for stills only; otherwise a divisor of240, at most120')
    args = parser.parse_args()
    if args.frames < 0 or args.frames > 120 or (args.frames and 240 % args.frames):
        parser.error('frames must be0 or a positive divisor of240 up to120')
    run(args.frames)
