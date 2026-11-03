#!/usr/bin/env python3
"""Physical XYZ guidance, exact checkpoint continuation and replay through MCP."""
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_geometry import box


def actor(name, start, end, curved):
    e = box(name, start, [0.6, 0.6, 0.6], body='npc',
            color=(0.85, 0.3, 0.6) if name == 'a' else (0.2, 0.65, 0.95))
    direction = [end[i]-start[i] for i in range(3)]
    e['character_controller'] = {}
    e['npc'] = {'surface': 'floor', 'speed': 2, 'stop_distance': 0.2, 'enabled': True,
                'mobility': {'walk': True, 'fly_speed': 2},
                'mind': {'intelligence': 1},
                'circulation': {'radius': 6, 'horizon_seconds': 2, 'personal_space': 0.2},
                'objectives': [{'priority': 0, 'when': 'always', 'action': {
                    'kind': 'guard_3d', 'point': end, 'arrival_distance': 0.2}}]}
    if curved:
        e['npc']['flight'] = {'mode': 'curved', 'acceleration': 4, 'deceleration': 4,
                              'turn_degrees_per_second': 180, 'smoothing_seconds': 0.05,
                              'minimum_speed': 0, 'can_hover': True,
                              'initial_direction': direction}
    return e


def prepare(c, curved=True, vertical=False, corridor=False, convergence=False):
    c.call('generate', recipe={'width': 30, 'depth': 24, 'seed': 42, 'obstacles': 0})
    player = next(e for e in c.call('inspect')['scene']['entities'] if e['id'] == 'player')
    player['transform']['position'] = [-10, 0.8, 8]
    player.pop('footsteps', None)
    a, b = ([0, 2, 0], [0, 8, 0]) if vertical else ([-4, 4, 0], [4, 4, 0])
    ends = [[-0.35, 4, 0.1], [0.35, 4, -0.1]] if convergence else [b, a]
    entities = [player, actor('a', a, ends[0], curved), actor('b', b, ends[1], curved)]
    if corridor:
        entities += [box('north-wall', [0, 4, -0.7], [14, 2, 0.3]),
                     box('south-wall', [0, 4, 0.7], [14, 2, 0.3])]
    nodes = [[p[0], p[1]-0.3, p[2]] for p in [a, b, *ends]]
    if convergence:
        links = [{'from': 0, 'to': 2, 'mode': 'fly'}, {'from': 1, 'to': 3, 'mode': 'fly'}]
    else:
        nodes = nodes[:2]
        links = [{'from': 0, 'to': 1, 'mode': 'fly'}, {'from': 1, 'to': 0, 'mode': 'fly'}]
    c.call('set_spatial_network', network={'nodes': nodes, 'links': links})
    c.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
    c.call('set_camera', camera={'position': [10, 7, 0.1], 'target': [0, 4, 0], 'fov_degrees': 55})
    return entities


def verify(binary, window):
    c = Client(binary, window)
    checks, metrics = [], []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'ignis-air-circulation', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        for curved, vertical, corridor, convergence in [(False, False, False, False),
                (True, False, True, False), (True, True, False, False), (True, False, False, True)]:
            prepare(c, curved, vertical, corridor, convergence)
            if corridor:
                c.call('set_debug_view', npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)
                c.call('save_scene', path='examples/npc-air-circulation.scene.json')
            avoiding = False
            minimum = 100
            closest_tick = 0
            for tick in range(5, 601, 5):
                c.call('step', ticks=5)
                snap = c.call('inspect')
                positions = snap['runtime_positions']
                separation = max(abs(positions['a'][i]-positions['b'][i]) for i in range(3))
                if separation < minimum:
                    minimum, closest_tick = separation, tick
                assert minimum > 0.57, (tick, positions)
                states = c.call('inspect_npc_circulation')['npcs']
                avoiding |= any(v['state']['air'] is not None and v['state']['reason'] == 'avoiding'
                                for v in states.values() if v['state'] and 'air' in v['state'])
                if corridor and tick == 90:
                    saved = snap
                    c.call('save_checkpoint', path='artifacts/npc-air.ignis-save')
                    c.call('capture', path='artifacts/npc-air-prefix.png', width=1000, height=700)
                if corridor and tick == 180:
                    continuation = snap
                    c.call('capture', path='artifacts/npc-air-crossing.png', width=1000, height=700)
                    c.call('load_checkpoint', path='artifacts/npc-air.ignis-save', mode='recorded')
                    assert c.call('inspect')['runtime_positions'] == saved['runtime_positions']
                    c.call('step', ticks=90)
                    after = c.call('inspect')
                    for key in ['runtime_positions', 'npcs', 'mobility', 'character_controllers']:
                        if key in after:
                            assert after[key] == continuation[key], key
            assert avoiding, (curved, vertical, corridor, convergence)
            assert all(v['memory'] is None for v in c.call('inspect_npcs')['npcs'].values())
            replay = 'artifacts/npc-air.replay.json'
            c.call('save_replay', path=replay)
            assert c.call('run_replay', path=replay)['matched']
            if corridor:
                c.call('capture_replay', replay_path=replay, path='artifacts/npc-air-prefix-replay.png',
                       tick=0, width=1000, height=700)
                c.call('capture_replay', replay_path=replay, path='artifacts/npc-air-crossing.png',
                       tick=max(0, closest_tick-90), width=1000, height=700)
                assert (ROOT/'artifacts/npc-air-prefix.png').read_bytes() == (ROOT/'artifacts/npc-air-prefix-replay.png').read_bytes()
            metrics.append({'curved': curved, 'vertical': vertical, 'corridor': corridor,
                            'convergence': convergence, 'min_box_axis_distance': minimum, 'closest_tick': closest_tick,
                            'end_phases': {k: v['phase'] for k, v in snap['mobility'].items()}})
        checks.append('Straight/curved XYZ crossing, vertical opposition, corridor constraints and convergence preserve physical separation and anonymous sensing')
        checks.append('Airborne checkpoint resumes exactly; baseline replay and offscreen prefix pixels match')
        result = {'passed': True, 'checks': checks, 'metrics': metrics, 'native_window': window,
                  'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/npc-air-circulation-verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': result['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    a = p.parse_args()
    verify(a.binary.resolve(), a.window)
