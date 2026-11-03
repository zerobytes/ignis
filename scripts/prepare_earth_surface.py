#!/usr/bin/env python3
"""Compose and inspect Surface in the actual Earth-sized runtime, through MCP.

Cold authoring only: Python writes a bounded graph, never samples a height or
generates geometry. Rust owns source evaluation, collision, residence and views.
"""
import argparse
import copy
import hashlib
import json
from verify_lab import Client, ROOT
from open_earth_walk import init, view, demo_locomotion
from surface_geography import draft_for, direction

OUT = ROOT / 'artifacts/earth-surface'
SCENE = 'artifacts/earth-surface/player.scene.json'
CONFIG = 'artifacts/earth-surface/world.json'


def prepare(binary, recipe_path, capture=True):
    OUT.mkdir(parents=True, exist_ok=True)
    recipe = json.loads((ROOT / recipe_path).read_text())
    draft = draft_for(recipe)
    if draft != draft_for(recipe):
        raise ValueError('Authoring must be repeatable')
    c = Client(binary, log_path=OUT / 'prepare.log')
    try:
        init(c)
        initial = c.call('inspect')
        document = dict(version=2, space_version=1, id='earth_surface', revision=1,
                        center=recipe['center'], diameter_meters=recipe['diameter_meters'],
                        surface_gravity_mps2=recipe['gravity_mps2'], base_height_meters=0,
                        operations=[{'kind': 'surface', 'program': c.call('query_planet',
                            query={'kind': 'prepare_surface', 'draft': draft})['program']}])
        height = c.call('query_planet', query={'kind': 'sample', 'document': document,
                       'directions': [[0, 1, 0]]})['samples'][0]['height_meters']
        # Surface replaces the accumulated height: normalize inside its graph.
        draft['nodes'].append({'kind': 'constant', 'value': -height})
        draft['nodes'].append({'kind': 'add', 'a': draft['output'], 'b': len(draft['nodes'])-1})
        draft['output'] = len(draft['nodes'])-1
        document['operations'][0]['program'] = c.call('query_planet',
            query={'kind': 'prepare_surface', 'draft': draft})['program']
        grid = [(x, z) for z in [0, -3000, -7000, -12000, -20000]
                for x in [-6000, -4000, -2000, 0, 2000, 4000, 6000]]
        samples = c.call('query_planet', query={'kind': 'sample', 'document': document,
            'directions': [direction(recipe['diameter_meters']/2, x, z) for x, z in grid]})['samples']
        if samples != c.call('query_planet', query={'kind': 'sample', 'document': document,
                'directions': [direction(recipe['diameter_meters']/2, x, z) for x, z in grid]})['samples']:
            raise ValueError('Native source must be reproducible')
        scene = copy.deepcopy(initial['scene'])
        player = next(e for e in scene['entities'] if e['id'] == 'player')
        player.update(transform={'position': [0, .91, 0], 'size': [.6, 1.8, .6]},
                      footsteps=None, avatar=None, abilities=None, vitality=None, trigger=None,
                      locomotion=demo_locomotion(), character_controller={
                          'max_slope_degrees': 45, 'slide_slope_degrees': 50,
                          'offset': .01, 'snap_distance': .2, 'gravity': 9.81})
        scene.update(entities=[player], recipe=None, name='Ignis — Earth / native Surface geography')
        scene['camera'].update(position=[12, 7, 7], target=[-14, .91, -8])
        scene['light'].update(direction=[.7, -.6, -.35], ambient=.3, shadow=None, environment=None,
            atmosphere={'density': recipe['atmosphere_density'], 'reduce_details': True,
                        'silhouette_error_px': .5, 'color_error': .002,
                        'color': [.48, .6, .72], 'sun_color': [1, .85, .65], 'sun_strength': .15})
        material = copy.deepcopy(recipe['material'])
        material['pattern']['seed'] = recipe['seed'] & 0xffff
        config = dict(document=document, actor='player',
                      spawn={'direction': [0, 1, 0], 'forward': [0, 0, -1], 'altitude_meters': .91},
                      **recipe['runtime'], color=recipe['color'], material=material,
                      compile_distant_meshes=True)
        (ROOT / SCENE).write_text(json.dumps(scene, separators=(',', ':')))
        (ROOT / CONFIG).write_text(json.dumps(config, indent=2))
        source = 'artifacts/earth-surface/earth.ignis-planet'
        c.call('save_planet', path=source, document=document)
        if c.call('load_planet', path=source)['document'] != document:
            raise ValueError('Planet persistence changed the native program')
        print(json.dumps({'source_ready': True, 'nodes': len(draft['nodes'])}), flush=True)
        c.call('set_execution_mode', mode='continuous')
        c.call('load_scene', path=SCENE)
        c.call('load_planet_runtime', path=CONFIG)
        c.call('set_player_view', view=view())
        c.call('step', ticks=120)
        start = c.call('inspect_planet_runtime')
        if not start['controller']['grounded']:
            raise ValueError(f'Spawn is not grounded: {start}')
        captures = []
        def shot(name):
            if capture:
                print(json.dumps({'capture': name}), flush=True)
                captures.append(c.call('capture', path=f'artifacts/earth-surface/{name}.png', width=1280, height=800))
        for yaw in (0, -90, 90, 180):
            v = view(); v['yaw_degrees'] = yaw
            c.call('set_player_view', view=v)
            shot(f'ground-{yaw}')
        v = view(); v.update(mode='third_person', pitch_degrees=-15)
        c.call('set_player_view', view=v)
        shot('third-person')
        c.call('set_planet_view', view={'distance_meters': 4000, 'tilt_degrees': 55, 'viewport': [1280, 800]})
        shot('aerial')
        c.call('set_planet_view', view={'distance_meters': recipe['diameter_meters']*1.5, 'viewport': [1280, 800]})
        shot('globe')
        globe = c.call('inspect_planet_view')
        if not globe['all_six_faces_covered'] or not globe['conforming_edges']:
            raise ValueError('Orbital camera did not preserve whole-planet coverage')
        if c.call('inspect_planet_runtime') != start:
            raise ValueError('Looking around/zooming moved the player')
        c.call('set_planet_view', view=None)
        c.call('set_player_view', view=view())
        c.call('set_player_view_input', input={'x': 0, 'z': -1, 'mode': 'run'})
        c.call('step', ticks=180)
        c.call('set_player_view_input', input={'x': 0, 'z': 0})
        c.call('step', ticks=180)
        moved = c.call('inspect_planet_runtime')
        if not moved['controller']['grounded'] or moved['distance_from_spawn_meters'] < 500:
            raise ValueError(f'Actual motor traversal failed: {moved}')
        shot('ground-after-walk')
        report = dict(build_id=initial['build_id'], recipe=recipe_path, diameter_meters=recipe['diameter_meters'],
                      binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                      scene_sha256=hashlib.sha256((ROOT / SCENE).read_bytes()).hexdigest(),
                      config_sha256=hashlib.sha256((ROOT / CONFIG).read_bytes()).hexdigest(),
                      source_nodes=len(draft['nodes']), start=start, moved=moved, globe=globe,
                      relief=[{'x': x, 'z': z, 'height': s['height_meters']} for (x,z), s in zip(grid, samples)],
                      captures=captures, native_window=False, automated_passed=True,
                      human_visual_acceptance=False, fps_verified=False)
        (OUT / 'verification.json').write_text(json.dumps(report, indent=2))
        print(json.dumps({'verified': True, 'distance_meters': moved['distance_from_spawn_meters']}), flush=True)
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--recipe', default='examples/earth-surface.recipe.json')
    p.add_argument('--no-captures', action='store_true')
    a = p.parse_args()
    prepare(ROOT / 'target/release/ignis', a.recipe, not a.no_captures)
