#!/usr/bin/env python3
"""Author the wind/leaves scene through the real MCP service, without a window."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def recipe(scene, count=64):
    scene['name'] = 'Wind — free aerodynamic leaves'
    scene['camera'].update({'position': [14., 12., 20.], 'target': [1.5, 5., 0.]})
    scene['entities'] = [e for e in scene['entities'] if e['id'] == 'floor']
    for e in scene['entities']:
        e['color'] = [.24, .27, .25]
        e['transform']['size'][0] = e['transform']['size'][2] = 256.
    # Fixed two-meter references make translation legible beside rotation.
    floor = scene['entities'][0]
    for i, (x, z) in enumerate([(-4., 0.), (0., -4.), (4., 0.)]):
        marker = copy.deepcopy(floor)
        marker.update(id=f'wind-reference-{i}', name='2 m reference', body=None,
                      color=[.55, .65, .7],
                      transform={'position': [x, 1., z], 'size': [.06, 2., .06]})
        scene['entities'].append(marker)
    # A pair of triangular midsurfaces, with an actual thin closed visible skin.
    outline = [[-.16, 0., 0.], [0., .012, .075], [.16, 0., 0.], [0., -.006, -.075]]
    positions = [[x, y + sign * .001, z] for sign in [1, -1] for x, y, z in outline]
    triangles = [[0, 1, 2], [0, 2, 3], [4, 6, 5], [4, 7, 6]]
    for a, b in [(0, 1), (1, 2), (2, 3), (3, 0)]:
        triangles += [[a, b + 4, b], [a, a + 4, b + 4]]
    scene['meshes'] = {'assets': [{'id': 'dry-leaf', 'geometry': {'positions': positions, 'triangles': triangles},
                                  'material': {'reflectance': {'roughness': .85, 'f0': .04}}}], 'instances': []}
    # An eight-second lifting gust re-entrains settled sheets. Horizontal
    # components turn over sixteen seconds; these remain airflow velocities,
    # never assignments to body trajectories or orientations.
    field = {'mean': [0., 1.5, 0.],
             'gusts': [{'amplitude': [1.2, 0., 0.], 'period_seconds': 16., 'phase_radians': 0.},
                       {'amplitude': [0., 0., 1.2], 'period_seconds': 16., 'phase_radians': math.pi / 2},
                       {'amplitude': [0., 2.5, 0.], 'period_seconds': 8., 'phase_radians': 0.}],
             'turbulence': {'seed': 2718, 'modes': 16, 'rms_speed': .5, 'min_wavelength': 1.,
                            'max_wavelength': 12., 'advection': [.7, .1, 0.]},
             'jets': [{'origin': [0., 0., -2.], 'direction': [1., 0., 0.], 'transverse': [0., 0., 1.], 'speed': .8, 'width': 1.5},
                      {'origin': [0., 0., 2.], 'direction': [-1., 0., 0.], 'transverse': [0., 0., 1.], 'speed': .8, 'width': 1.5}],
             'vortices': [{'center': [0., 3., 0.], 'axis': [0., 1., 0.], 'velocity': [0., 0., 0.],
                           'radius': 7., 'half_height': 8., 'peak_swirl_speed': 1.5, 'updraft_speed': 1.,
                           'start_seconds': 0., 'duration_seconds': None}]}
    bodies = []
    for i in range(count):
        angle = i * 2.399963229728653
        radius = .5 + 2.5 * math.sqrt((i + .5) / count)
        tilt = .1 * math.sin(i * .9)
        scene['meshes']['instances'].append({'id': f'leaf-{i}', 'asset': 'dry-leaf',
            'position': [math.cos(angle) * radius, .7 + (i % 7) * .4, math.sin(angle) * radius],
            'rotation': [0., 0., math.sin(tilt), math.cos(tilt)], 'scale': 1.,
            'color': [.45 + (i % 4) * .1, .23 + (i % 3) * .1, .055], 'playback': None})
        bodies.append({'instance': f'leaf-{i}', 'sheet_mass_kg': .0005 + (i % 5) * .00005,
                       'ballast_mass_kg': .00003, 'ballast_position': [-.1, 0., 0.],
                       'thickness_m': .002, 'normal_drag': 1.8, 'tangent_drag': [.03, .02],
                       'angular_velocity': [0., .3 * math.sin(i), .4 * math.cos(i)],
                       'panels': [{'vertices': [outline[j] for j in [0, 1, 2]]},
                                  {'vertices': [outline[j] for j in [0, 2, 3]]}]})
    scene['wind'] = {'version': 1, 'field': field, 'air_density': 1.225, 'substeps': 4, 'bodies': bodies}
    return scene


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    args = parser.parse_args()
    out = ROOT / 'artifacts/wind'
    out.mkdir(parents=True, exist_ok=True)
    c = Client(args.binary, log_path=out / 'demo-runtime.log')
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'wind-demo', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe={'width': 24., 'depth': 24., 'seed': 42, 'obstacles': 0})
        scene = recipe(c.call('inspect')['scene'])
        path = 'examples/wind-leaves.scene.json'
        (ROOT / path).write_text(json.dumps(scene, indent=2) + '\n')
        c.call('load_scene', path=path)
        c.call('save_scene', path=path)
        c.call('wind', operation={'op': 'save', 'path': 'assets/wind/leaves.ignis-wind'})
        c.call('capture', path='artifacts/wind/start.png', width=1280, height=800)
        for tick in [60, 180, 360]:
            previous = c.call('inspect')['tick']
            c.call('step', ticks=tick - previous)
            c.call('capture', path=f'artifacts/wind/tick-{tick}.png', width=1280, height=800)
        state = c.call('wind', operation={'op': 'inspect'})
        state['build_id'] = c.call('inspect')['build_id']
        (out / 'demo-state.json').write_text(json.dumps(state, indent=2))
        print(path)
    finally:
        c.close()


if __name__ == '__main__':
    main()
