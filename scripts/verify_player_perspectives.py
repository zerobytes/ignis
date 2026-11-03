#!/usr/bin/env python3
"""Critical perspective integration through MCP; captures never foreground a window."""
import argparse
import copy
import json
import math
from pathlib import Path

from verify_lab import Client, ROOT

OUT = ROOT / 'artifacts/player-perspectives'
PREFIX = 'artifacts/player-perspectives/'


def verify(binary, landscape=False):
    OUT.mkdir(parents=True, exist_ok=True)
    c = Client(binary, log_path=OUT / 'runtime.log')
    report = dict(checks=0, native_window=False, artistic_acceptance=False)
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                    clientInfo=dict(name='player-perspectives', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe=dict(width=30, depth=30, seed=186, obstacles=0))
        initial = c.call('inspect'); report['build_id'] = initial['build_id']
        player = copy.deepcopy(next(e for e in initial['scene']['entities'] if e['id'] == 'player'))
        player['transform'] = dict(position=[0, .91, 0], size=[.6, 1.8, .6])
        player['character_controller'] = dict(max_slope_degrees=40, slide_slope_degrees=45, snap_distance=.2)
        player.pop('footsteps', None)
        wall = copy.deepcopy(next(e for e in initial['scene']['entities'] if e['id'] == 'wall_north'))
        wall['id'] = 'camera_wall'; wall['name'] = 'Camera obstruction'
        wall['transform'] = dict(position=[0, 1.5, 2.5], size=[4, 3, .4])
        wall['color'] = [.4, .22, .2]
        c.call('set_execution_mode', mode='continuous')
        edits = [dict(op='remove', id=e['id']) for e in initial['scene']['entities'] if e.get('body') == 'npc']
        c.call('edit_scene', edits=edits+[dict(op='upsert', entity=player), dict(op='upsert', entity=wall)])
        authored = c.call('inspect')['scene']
        c.call('save_scene', path=PREFIX + 'demo.scene.json')
        before = c.call('inspect_world')
        legacy = dict(actor='player', yaw_degrees=0, pitch_degrees=0, eye_height=1.65, fov_degrees=75)
        first = c.call('set_player_view', view=legacy)
        assert first['view']['mode'] == 'first_person' and first['resolved']['hide_body']
        c.call('capture', path=PREFIX + 'first-person.png', width=1280, height=800)
        third = dict(**legacy, mode='third_person', third_person=dict(distance_meters=4, shoulder_offset=0))
        blocked = c.call('set_player_view', view=third)
        assert blocked['resolved']['obstructed'] and not blocked['resolved']['initial_overlap']
        assert 2 < blocked['camera']['position'][2] < 2.2, blocked
        assert c.call('inspect_world') == before
        c.call('capture', path=PREFIX + 'third-blocked.png', width=1280, height=800)
        third['yaw_degrees'] = 90
        looked = c.call('look_player_view', yaw_delta_degrees=90, pitch_delta_degrees=0)
        assert looked['yaw_degrees'] == 90
        free = c.call('inspect_player_view')
        assert not free['resolved']['obstructed'] and not free['resolved']['hide_body']
        c.call('capture', path=PREFIX + 'third-person.png', width=1280, height=800)
        invalid = copy.deepcopy(third); invalid['third_person']['distance_meters'] = -1
        c.call('set_player_view', view=invalid, expected_error=True)
        assert c.call('inspect_player_view') == free
        report['blocked'] = blocked; report['free'] = free; report['checks'] += 1

        c.call('step', ticks=40)
        start = c.call('inspect_world')['positions']['player']
        c.call('set_player_view_input', input=dict(x=0, z=-1))
        c.call('step', ticks=120)
        end = c.call('inspect_world')['positions']['player']
        grounded = c.call('inspect_character_controllers')['controllers']['player']['grounded']
        assert end[0] - start[0] > 5 and abs(end[2]-start[2]) < .01 and grounded, (start, end)
        assert c.call('inspect')['scene'] == authored
        report['walk'] = dict(before=start, after=end, distance_meters=math.dist(start, end), grounded=grounded)
        report['checks'] += 1

        c.call('load_scene', path=PREFIX + 'demo.scene.json')
        assert not c.call('inspect_player_view')['active']
        assert c.call('inspect')['scene'] == authored
        c.call('set_player_view', view=legacy)
        c.call('reset')
        assert not c.call('inspect_player_view')['active']
        c.call('set_player_view_input', input=dict(x=0, z=-1), expected_error=True)
        c.call('look_player_view', yaw_delta_degrees=90, pitch_delta_degrees=0, expected_error=True)
        report['checks'] += 1

        if landscape:
            c.call('load_scene', path='artifacts/expanded-world/world.scene.json')
            placement = json.loads((ROOT / 'artifacts/expanded-world/placement.json').read_text())
            v = dict(placement['player_view'], mode='third_person', third_person=dict(distance_meters=4, shoulder_offset=.4))
            c.call('set_player_view', view=v)
            c.call('step', ticks=40)
            start = c.call('inspect_world')['positions']['player']
            c.call('set_player_view_input', input=dict(x=0, z=-1))
            c.call('step', ticks=120)
            end = c.call('inspect_world')['positions']['player']
            grounded = c.call('inspect_character_controllers')['controllers']['player']['grounded']
            assert start[2]-end[2] > 5 and grounded, (start, end)
            report['landscape_view'] = c.call('inspect_player_view')
            report['landscape_walk'] = dict(before=start, after=end, distance_meters=math.dist(start, end), grounded=grounded)
            c.call('capture', path=PREFIX + 'landscape-third-person.png', width=1728, height=1117)
            report['checks'] += 1
        report['passed'] = True
        (OUT / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(dict(passed=True, checks=report['checks'], build_id=report['build_id'])))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    parser.add_argument('--landscape', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.landscape)
