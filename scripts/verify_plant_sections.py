#!/usr/bin/env python3
"""Author and reshape a native sword, preserving its canonical detail and rig."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-section-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe={'seed': 7, 'width': 16, 'depth': 16, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': {**e, 'visible': False}} for e in entities if e['id'] != 'floor'])
        c.call('set_camera', camera={'position': [2.8, 3.3, 10], 'target': [0, 2.7, 0], 'fov_degrees': 37})
        c.call('set_light', light={'direction': [-0.5, -0.65, -0.75], 'ambient': 0.32})

        def definition(id, width, height, depth, stations, cross='diamond', segments=(4, 6, 4)):
            return {'id': id, 'profile': {'id': id+'_front', 'width': width, 'height': height}, 'depth': depth,
                    'sections': {'id': id+'_cuts', 'cross_section': cross,
                                 'stations': [{'id': name, 'at': at, 'width': w, 'depth': d} for name, at, w, d in stations]},
                    'segments': list(segments), 'sampling_tolerance': 0.0009765625}

        blade = definition('blade', 0.58, 3.2, 0.14, [('heel', 0, 0.8, 1), ('shoulder', 0.14, 1, 1), ('point_start', 0.8, 0.66, 0.8), ('tip', 1, 0.008, 0.1)])
        guard = definition('guard', 1.35, 0.2, 0.24, [('lower', 0, 0.88, 0.6), ('middle', 0.5, 1, 1), ('upper', 1, 0.88, 0.6)], cross='rectangle', segments=(12, 2, 2))
        grip = definition('grip', 0.24, 0.8, 0.23, [('bottom', 0, 1, 1), ('waist', 0.55, 0.8, 0.8), ('top', 1, 1, 1)], cross='rectangle', segments=(4, 8, 4))
        pommel = definition('pommel', 0.44, 0.38, 0.27, [('bottom', 0, 0.38, 0.6), ('middle', 0.45, 1, 1), ('top', 1, 0.5, 0.8)], segments=(4, 4, 4))

        def put(d, op='edit_plant', **kwargs):
            return c.call(op, edit={'kind': 'put', 'definition': d, 'detail_policy': 'physical', 'animation_translation_policy': 'physical'}, **kwargs)

        def instance(id, y, color, playback=None):
            return {'id': id+'_instance', 'asset': id, 'position': [0, y, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': color, 'playback': playback}

        for d in [blade, guard, grip, pommel]:
            put(d)
        placements = [instance('blade', 2.88, [0.61, 0.73, 0.83], {'clip': 'flex', 'offset_ticks': 0, 'looping': True}),
                      instance('guard', 1.2, [0.72, 0.44, 0.12]), instance('grip', 0.72, [0.19, 0.055, 0.022]), instance('pommel', 0.18, [0.72, 0.44, 0.12])]

        # Sculpt a short zig-zag engraving on the front ridge using native brushes.
        centers = [[0, y, 0.07] for y in [-0.8, -0.66, -0.53, -0.4]]
        c.call('edit_mesh', edit={'kind': 'sculpt', 'asset': 'blade', 'stroke': {'kind': 'grab', 'centers': centers, 'radius': 0.16, 'strength': 0.007, 'direction': [0, 0, -1]}})
        asset = c.call('inspect_mesh', asset='blade')['asset']
        weights = []
        for x, y, z in asset['geometry']['positions']:
            t = max(0, min(2, (y+1.6)/1.28))
            low = min(1, int(t)); f = t-low
            weights.append({'joints': [low, low+1, 0, 0], 'weights': [1-f, f, 0, 0]})
        bones = [{'name': 'heel', 'parent': None, 'translation': [0, -1.6, 0], 'rotation': [0, 0, 0, 1]},
                 {'name': 'middle', 'parent': 0, 'translation': [0, 1.28, 0], 'rotation': [0, 0, 0, 1]},
                 {'name': 'point', 'parent': 1, 'translation': [0, 1.28, 0], 'rotation': [0, 0, 0, 1]}]
        c.call('edit_mesh', edit={'kind': 'set_rig', 'asset': 'blade', 'rig': {'bones': bones, 'weights': weights}})
        angle = math.radians(18)/2
        c.call('edit_mesh', edit={'kind': 'put_clip', 'asset': 'blade', 'clip': {'id': 'flex', 'duration_ticks': 120,
                    'tracks': [{'bone': 1, 'keys': [{'tick': t, 'translation': [0, 1.28, 0], 'rotation': [0, 0, math.sin(a), math.cos(a)]} for t, a in [(0, 0), (60, angle), (120, 0)]]}]}})
        for item in placements:
            c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': item})
        original = c.call('inspect_mesh', asset='blade')['asset']
        source = c.call('inspect_plant', id='blade')['plant']['source']
        assert c.call('inspect_plant', id='blade')['report']['max_detail_displacement'] > 0.001
        c.call('capture', path='artifacts/sword-original.png', width=1100, height=900)
        c.call('save_scene', path='examples/native-sword.scene.json')
        checks.append('Sword authored entirely through native profiles/sections, sculpt, rig, clip and instances')

        changed = copy.deepcopy(blade)
        changed['profile'].update(width=0.82, height=4)
        changed['depth'] = 0.18
        changed['sections']['stations'][1].update(at=0.24, width=1)
        changed['sections']['stations'][2].update(at=0.72, width=0.8, depth=0.65)
        before = c.call('inspect')
        put(changed, 'preview_plant')
        assert c.call('inspect') == before
        put(changed)
        evaluated = c.call('inspect_mesh', asset='blade')['asset']
        assert c.call('inspect_plant', id='blade')['plant']['source'] == source
        assert evaluated['rig']['weights'] == original['rig']['weights']
        assert evaluated['rig']['bones'][1]['translation'] != original['rig']['bones'][1]['translation']
        assert evaluated['clips'][0]['tracks'][0]['keys'][1]['rotation'] == original['clips'][0]['tracks'][0]['keys'][1]['rotation']
        assert evaluated['clips'][0]['tracks'][0]['keys'][0]['translation'] == evaluated['rig']['bones'][1]['translation']
        # Keep heel in contact with the guard when changing the blade length.
        edited_instance = copy.deepcopy(placements[0]); edited_instance['position'][1] = 3.28
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': edited_instance})
        c.call('capture', path='artifacts/sword-redrawn.png', width=1100, height=900)
        c.call('save_scene', path='examples/native-sword-redrawn.scene.json')
        assert (ROOT/'artifacts/sword-original.png').read_bytes() != (ROOT/'artifacts/sword-redrawn.png').read_bytes()
        for _ in range(3):
            put(blade)
            assert c.call('inspect_mesh', asset='blade')['asset'] == original
            put(changed)
            assert c.call('inspect_mesh', asset='blade')['asset'] == evaluated
        checks.append('Length, silhouette, section positions and thickness change while canonical engraving, weights and rest-relative animation survive A-B-A without drift')

        # Retessellation preserves the source. Loss, domain changes and malformed sections fail.
        fine = copy.deepcopy(changed); fine['segments'] = [8, 12, 4]
        put(fine)
        snapshot = c.call('inspect')
        for mutation in ['missing', 'reorder', 'zero', 'collapse', 'cross', 'budget', 'coarse']:
            bad = copy.deepcopy(fine)
            if mutation == 'missing': bad['sections']['stations'].pop(1)
            if mutation == 'reorder': bad['sections']['stations'][1]['id'] = 'different'
            if mutation == 'zero': bad['sections']['stations'][1]['width'] = 0
            if mutation == 'collapse': bad['sections']['stations'][1]['at'] = 0
            if mutation == 'cross': bad['sections']['cross_section'] = 'rectangle'
            if mutation == 'budget': bad['segments'] = [63, 63, 63]
            if mutation == 'coarse': bad['segments'] = [1, 1, 1]
            put(bad, expected_error=True)
            assert c.call('inspect') == snapshot
        put(fine, expected_revision=snapshot['revision']-1, expected_error=True)
        c.call('undo'); assert c.call('inspect_mesh', asset='blade')['asset'] == evaluated
        c.call('redo'); assert c.call('inspect')['scene'] == snapshot['scene']
        assert c.call('inspect_plant', id='blade')['plant']['source'] == source
        checks.append('Retessellation, explicit domain identity, collapse/budget/loss rejection and stale revisions are atomic; undo/redo preserve canonical source')

        c.call('capture', path='artifacts/sword-start.png', width=1100, height=900)
        c.call('step', ticks=45)
        pose = c.call('inspect_mesh_pose', instance='blade_instance')['pose']
        assert pose['clip_tick'] == 45
        c.call('capture', path='artifacts/sword-flex.png', width=1100, height=900)
        c.call('capture', path='artifacts/sword-repeat.png', width=1100, height=900)
        assert (ROOT/'artifacts/sword-flex.png').read_bytes() == (ROOT/'artifacts/sword-repeat.png').read_bytes()
        assert (ROOT/'artifacts/sword-flex.png').read_bytes() != (ROOT/'artifacts/sword-start.png').read_bytes()
        c.call('save_replay', path='artifacts/sword.replay.json')
        assert c.call('run_replay', path='artifacts/sword.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/sword.replay.json', path='artifacts/sword-replay.png', width=1100, height=900)
        c.call('capture_replay', replay_path='artifacts/sword.replay.json', path='artifacts/sword-prefix.png', tick=0, width=1100, height=900)
        assert (ROOT/'artifacts/sword-flex.png').read_bytes() == (ROOT/'artifacts/sword-replay.png').read_bytes()
        assert (ROOT/'artifacts/sword-start.png').read_bytes() == (ROOT/'artifacts/sword-prefix.png').read_bytes()
        c.call('load_scene', path='examples/native-sword-redrawn.scene.json')
        assert c.call('inspect_mesh', asset='blade')['asset'] == evaluated
        c.call('load_scene', path='examples/native-sword.scene.json')
        assert c.call('inspect_mesh', asset='blade')['asset'] == original
        c.call('capture', path='artifacts/sword-loaded.png', width=1100, height=900)
        assert (ROOT/'artifacts/sword-original.png').read_bytes() == (ROOT/'artifacts/sword-loaded.png').read_bytes()
        checks.append('Native rig deformation, save/load, repeat captures and complete/prefix replay match GPU pixels')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window}
        (ROOT/'artifacts/plant-sections-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
