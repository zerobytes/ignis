#!/usr/bin/env python3
"""Evaluate generic stroke controls on the same integrated tree through MCP."""
import argparse
import copy
import hashlib
import json
import os
import random
from pathlib import Path

PROJECT = Path(__file__).resolve().parents[1]
WOOD = {'wood', 'branch-editable-1', 'secondary-06', 'secondary-twigs'}
CAMS = {
    'neck': {'position': [1.8, 2.9, 4], 'target': [-0.57, 2.65, 0.30], 'fov_degrees': 19},
    'base': {'position': [1.8, 1.35, 3.2], 'target': [0, 0.9, 0], 'fov_degrees': 25},
    'fork': {'position': [3.5, 3.2, 4.7], 'target': [0.55, 2.95, 0.13], 'fov_degrees': 25},
}


def fissures(seed=137):
    """Authored test patch, not a botanical feature of the engine."""
    rng = random.Random(seed)
    segments = []
    for col in range(8):
        x = 0.018 + col * 0.030
        start = rng.uniform(0.028, 0.30)
        end = min(0.47, start+rng.uniform(0.10, 0.28))
        points = [[x+rng.uniform(-0.007, 0.007), start+(end-start)*k/3] for k in range(4)]
        width = rng.uniform(0.0040, 0.0065)
        widths = [width*0.5, width, width*rng.uniform(0.6, 1), width*0.4]
        for k in range(3):
            segments.append({'start': points[k], 'end': points[k+1], 'widths': widths[k:k+2]})
        # A short secondary crack branches from each main groove.
        tip = [points[1][0] + (0.009 if col % 2 == 0 else -0.009), points[1][1]+0.035]
        segments.append({'start': points[1], 'end': tip, 'widths': [width*0.65, width*0.15]})
    assert len(segments) == 32
    return segments


def material(original, thin=False):
    m = copy.deepcopy(original)
    m['pattern'].update(kind='cellular', frequency=[40, 8, 40], low=[0.78, 0.74, 0.66], high=[1.20, 1.12, 1.02],
                        bump=0.0013, warp=1.8, octaves=2, contrast=1.0)
    m['detail'] = {'profile': 'rounded', 'repeat': [0.25, 0.5], 'plane': 'xy', 'triplanar': {'sharpness': 2},
                   'tint': [0.40, 0.37, 0.34], 'opacity': 0.65 if not thin else 0.28,
                   'bump': -0.0009 if not thin else -0.0003, 'segments': fissures()}
    m['reflectance'] = {'roughness': 0.90, 'f0': 0.035}
    return m


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--input', type=Path, required=True)
    args = p.parse_args()
    root = PROJECT/'artifacts/tree-realism'
    for name in ['examples', 'artifacts']: (root/name).mkdir(parents=True, exist_ok=True)
    source = args.input.read_bytes()
    target = root/'examples/input.scene.json'
    if args.input.resolve() != target.resolve(): target.write_bytes(source)
    os.environ['IGNIS_VERIFY_ROOT'] = str(root)
    from verify_lab import Client
    c = Client(PROJECT/'target/release/ignis', False)
    call = c.call
    def capture(name):
        path = f'artifacts/{name}.png'
        call('capture', path=path, width=1100, height=1100)
        return (root/path).read_bytes()
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'tree-bark-study', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/input.scene.json')
        before = call('inspect')['scene']
        call('save_scene', path='examples/before.scene.json')
        edits = [{'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': a['id'], 'material': material(a['material'], a['id'] in {'secondary-06', 'secondary-twigs'})}}
                 for a in before['meshes']['assets'] if a['id'] in WOOD]
        state = call('inspect'); pred = call('preview_scene_edits', edits=edits)['scene']
        assert call('inspect') == state
        call('edit_scene', edits=edits); after = call('inspect')['scene']; assert pred == after
        for a, b in zip(before['meshes']['assets'], after['meshes']['assets']):
            for key in ['id', 'geometry', 'rig', 'clips', 'shading']: assert a.get(key) == b.get(key)
        assert before['meshes']['instances'] == after['meshes']['instances']
        for a, b in zip(before['plants']['entries'], after['plants']['entries']):
            assert a['definition'] == b['definition'] and a['reference'] == b['reference']
            for key in ['geometry', 'rig', 'clips']: assert a['source'].get(key) == b['source'].get(key)
        call('save_scene', path='examples/after.scene.json')
        call('undo'); assert call('inspect')['scene'] == before
        call('redo'); assert call('inspect')['scene'] == after
        images = {}
        for mode in ['before', 'after']:
            for name, camera in dict(CAMS, whole=before['camera']).items():
                for channel in ['authored', 'neutral']:
                    call('load_scene', path=f'examples/{mode}.scene.json')
                    if channel == 'neutral':
                        call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': a['id'], 'material': {}}} for a in after['meshes']['assets'] if a['id'] in WOOD])
                    call('set_camera', camera=camera); call('step', ticks=60)
                    images[mode, name, channel] = capture(f'{mode}-{channel}-{name}')
        for name in dict(CAMS, whole=None): assert images['before', name, 'neutral'] == images['after', name, 'neutral']
        # Separate channels on the finished whole document, with the exact same
        # close camera, pose and lighting. No artificial rig replacement.
        for mode, key in [('color', 'bump'), ('bump', 'opacity')]:
            call('load_scene', path='examples/after.scene.json')
            edits = []
            for a in after['meshes']['assets']:
                if a['id'] in WOOD:
                    m = copy.deepcopy(a['material']); m['detail'][key] = 0
                    # Isolate strokes from the 3D pattern for this diagnostic.
                    m.pop('pattern', None)
                    edits.append({'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': a['id'], 'material': m}})
            call('edit_scene', edits=edits); call('set_camera', camera=CAMS['neck']); call('step', ticks=60)
            capture(f'after-{mode}-neck')
        call('load_scene', path='examples/after.scene.json'); assert call('inspect')['scene'] == after
        call('step', ticks=60); live = capture('live'); assert capture('repeat') == live
        call('save_replay', path='artifacts/bark.replay.json'); assert call('run_replay', path='artifacts/bark.replay.json')['meshes_matched']
        call('capture_replay', replay_path='artifacts/bark.replay.json', path='artifacts/replay.png', tick=60, width=1100, height=1100)
        assert (root/'artifacts/replay.png').read_bytes() == live
        report = {'passed': True, 'build_id': call('inspect')['build_id'], 'input_sha256': hashlib.sha256(source).hexdigest(),
                  'geometry_rig_clips_instances_exact': True, 'four_neutral_views_exact': True,
                  'native_window': False, 'visual_realism_complete': False}
        (root/'artifacts/report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report))
    finally: c.close()


if __name__ == '__main__': main()
