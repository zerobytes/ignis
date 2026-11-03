#!/usr/bin/env python3
"""Critical MCP/save/replay/render contract for physical free meshes and wind."""
import argparse
import copy
import json
import math
import time
from pathlib import Path
from demo_wind import recipe
from botany_exemplars import Builder
from verify_lab import Client, ROOT


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    out = ROOT / 'artifacts/wind'
    out.mkdir(parents=True, exist_ok=True)
    c = Client(args.binary, window=args.window, log_path=out / 'verification-runtime.log')
    report = {'groups': [], 'timings': []}
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'wind-verification', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = c.request('tools/list')['result']['tools'][0]['inputSchema']
        assert 'Wind' in json.dumps(schema) or 'wind' in json.dumps(schema)
        c.call('generate', recipe={'width': 24., 'depth': 24., 'seed': 42, 'obstacles': 0})
        source = recipe(c.call('inspect')['scene'], 64)
        path = 'artifacts/wind/verification.scene.json'
        (ROOT / path).write_text(json.dumps(source))
        c.call('load_scene', path=path)
        initial = c.call('wind', operation={'op': 'inspect'})['bodies']
        c.call('capture', path='artifacts/wind/initial.png', width=960, height=640)
        c.call('capture', path='artifacts/wind/repeated.png', width=960, height=640)
        assert (out / 'initial.png').read_bytes() == (out / 'repeated.png').read_bytes()
        report['groups'].append('schema/authored meshes/repeated GPU capture')

        samples = []
        h = 1e-4
        for i in range(30):
            p = [math.sin(i * 1.7) * 4., 1.8 + math.cos(i * .6) * 3., math.cos(i * 1.3) * 4.]
            for axis in range(3):
                for sign in [1, -1]:
                    q = p[:]
                    q[axis] += h * sign
                    samples.append(q)
        velocities = c.call('wind', operation={'op': 'sample', 'positions': samples, 'time_seconds': 3.})['velocity_mps']
        divergence = [sum((velocities[i * 6 + a * 2][a] - velocities[i * 6 + a * 2 + 1][a]) / (2 * h) for a in range(3)) for i in range(30)]
        report['max_sampled_divergence_per_second'] = max(map(abs, divergence))
        assert max(map(abs, divergence)) < 1e-6
        report['groups'].append('spatial analytic field/divergence')

        before = c.call('inspect')
        report['build_id'] = before['build_id']
        bad = copy.deepcopy(source['wind'])
        bad['bodies'][0]['panels'][0]['vertices'] = [[0., 0., 0.]] * 3
        c.call('wind', operation={'op': 'replace', 'document': bad}, expected_error=True)
        after = c.call('inspect')
        assert before['revision'] == after['revision'] and before['runtime_wind'] == after['runtime_wind']
        c.call('start_multiplayer_host', address='127.0.0.1:0', capacity=1, expected_error=True)
        report['groups'].append('atomic invalid authoring/unsupported replication boundary')

        for _ in range(3):
            started = time.perf_counter()
            c.call('step', ticks=60)
            report['timings'].append({'bodies': 64, 'ticks': 60, 'wall_seconds': time.perf_counter() - started})
        actual = c.call('wind', operation={'op': 'inspect'})
        assert sum(math.dist(a['position'], b['position']) > .1 for a, b in zip(initial, actual['bodies'])) > 50
        assert any(a['rotation'] != b['rotation'] for a, b in zip(initial, actual['bodies']))
        assert actual['aerodynamic_loads_last_tick']
        report['sample_work_per_tick'] = actual['sample_work_per_tick']
        c.call('save_checkpoint', path='artifacts/wind/current.ignis-save')
        c.call('save_portable_save', path='artifacts/wind/current.ignis-portable')
        c.call('save_replay', path='artifacts/wind/current.replay.json')
        c.call('capture', path='artifacts/wind/physical.png', width=960, height=640)
        c.call('capture_replay', replay_path='artifacts/wind/current.replay.json', path='artifacts/wind/replay.png', tick=None, width=960, height=640)
        assert (out / 'physical.png').read_bytes() == (out / 'replay.png').read_bytes()
        assert (out / 'physical.png').read_bytes() != (out / 'initial.png').read_bytes()
        replay = c.call('run_replay', path='artifacts/wind/current.replay.json')
        assert replay['matched'] and replay['wind_matched']
        report['groups'].append('real translation/rotation/loads/replay pixels and primaries')

        c.call('step', ticks=60)
        continued = c.call('wind', operation={'op': 'inspect'})['bodies']
        c.call('load_checkpoint', path='artifacts/wind/current.ignis-save', mode=None)
        assert c.call('wind', operation={'op': 'inspect'})['bodies'] == actual['bodies']
        c.call('step', ticks=60)
        assert c.call('wind', operation={'op': 'inspect'})['bodies'] == continued
        c.call('load_portable_save', path='artifacts/wind/current.ignis-portable', mode=None)
        assert c.call('wind', operation={'op': 'inspect'})['bodies'] == actual['bodies']
        c.call('step', ticks=60)
        portable_continued = c.call('wind', operation={'op': 'inspect'})['bodies']
        error = max(math.dist(a['position'], b['position']) for a, b in zip(continued, portable_continued))
        report['portable_continuation_max_position_error_m'] = error
        assert error < .03
        report['groups'].append('native exact continuation/portable canonical primaries')

        c.call('wind', operation={'op': 'open', 'open': True})
        c.call('capture_editor', path='artifacts/wind/editor.png', width=1280, height=800)
        c.call('wind', operation={'op': 'save', 'path': 'artifacts/wind/current.ignis-wind'})
        c.call('wind', operation={'op': 'replace', 'document': {'version': 1}})
        assert not c.call('wind', operation={'op': 'inspect'})['bodies']
        c.call('undo')
        assert len(c.call('wind', operation={'op': 'inspect'})['bodies']) == 64
        c.call('wind', operation={'op': 'load', 'path': 'artifacts/wind/current.ignis-wind'})
        report['groups'].append('editor/offscreen panel/source save-load/undo')

        # Protect the demonstrated behavior after the initial launch transient.
        c.call('reset')
        transport = []
        for start, end in [(8, 12), (16, 20), (24, 28), (56, 60)]:
            current = c.call('inspect')['tick']
            c.call('step', ticks=start * 60 - current)
            before = c.call('wind', operation={'op': 'inspect'})['bodies']
            c.call('step', ticks=(end - start) * 60)
            after = c.call('wind', operation={'op': 'inspect'})['bodies']
            travel = [math.dist(a['position'], b['position']) for a, b in zip(before, after)]
            row = {'window_seconds': [start, end],
                   'bodies_travelling_over_one_meter': sum(d > 1. for d in travel),
                   'airborne_at_end': sum(b['position'][1] > .3 for b in after),
                   'mean_displacement_m': sum(travel) / len(travel),
                   'mean_x_displacement_m': sum(b['position'][0] - a['position'][0]
                                                for a, b in zip(before, after)) / len(after)}
            assert row['bodies_travelling_over_one_meter'] >= 48, row
            assert row['airborne_at_end'] >= 40, row
            transport.append(row)
        assert transport[0]['mean_x_displacement_m'] < -.25, transport
        assert transport[1]['mean_x_displacement_m'] > .25, transport
        report['late_transport'] = transport
        report['groups'].append('late gust re-entrainment and opposite horizontal transport through 60 seconds')

        calm = copy.deepcopy(source['wind'])
        calm['field'] = {}
        c.call('wind', operation={'op': 'replace', 'document': calm})
        c.call('step', ticks=600)
        settled = c.call('wind', operation={'op': 'inspect'})['bodies']
        assert all(-.02 < b['position'][1] < .15 for b in settled), settled
        c.call('capture', path='artifacts/wind/settled.png', width=960, height=640)
        report['groups'].append('gravity and convex physical floor contacts')
        # The existing XPBD adapter must consume the shared field only on opt-in.
        c.call('generate', recipe={'width': 8., 'depth': 8., 'seed': 42, 'obstacles': 0})
        c.call('wind', operation={'op': 'replace', 'document': {'field': {'mean': [5., 0., 0.]}}})
        plant = Builder('Wind coupling fixture')
        stem = plant.axis('stem', None, 0., (0., 1., 0.), 1., .015)
        plant.leaf('blade', stem, .7, (1., .2, 0.), .3, .1, normal=(0., 0., 1.))
        fixture = plant.recipe('wind-plant')
        plant_path = 'artifacts/wind/plant.ignis-botany'
        (ROOT / plant_path).write_text(json.dumps(fixture))
        c.call('botany', operation={'kind': 'load', 'path': plant_path, 'publish': True})
        impulses = []
        for enabled in [False, True]:
            settings = {'gravity': [0., 0., 0.], 'wind': [0., 0., 0.], 'ground': None,
                        'use_scene_wind': enabled, 'self_contact': False}
            c.call('botany', operation={'kind': 'physics', 'prefix': 'wind-plant',
                                       'action': {'kind': 'start', 'settings': settings}})
            state = c.call('botany', operation={'kind': 'physics', 'prefix': 'wind-plant',
                                               'action': {'kind': 'step', 'ticks': 60}})
            impulses.append(math.sqrt(sum(x * x for x in state['force_impulse'])))
        assert impulses[0] < 1e-10 and impulses[1] > 1e-8, impulses
        report['botanical_external_impulse_kg_mps'] = impulses
        report['groups'].append('botanical opt-in shares scene wind; legacy calm remains calm')
        report['passed'] = True
        report['checks'] = len(report['groups'])
        (out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    main()
