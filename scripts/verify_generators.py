#!/usr/bin/env python3
"""Seeded population replenishment through MCP, including retirement/revival and replay."""
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-generators-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe={'width': 24, 'depth': 20, 'seed': 42, 'obstacles': 0})
        def definition(id, kind, amount):
            return {'id': id, 'selection': 'actor', 'range': 20, 'cast_ticks': 0, 'cooldown_ticks': 0, 'costs': [], 'effects': [{'kind': kind, 'recipient': 'target', 'amount': amount}]}
        c.call('set_ability_catalog', catalog={'resources': [], 'abilities': [definition('kill', 'damage', 100), definition('revive', 'resurrect', 40)]})
        p = next(e for e in c.call('inspect')['scene']['entities'] if e['id'] == 'player')
        p['transform']['position'] = [0, 0.8, -4]
        p['footsteps'] = None
        p['abilities'] = {'resources': [], 'grants': ['kill', 'revive']}
        c.call('upsert', entity=p)
        t = {'id': 'guard', 'name': 'Guard', 'body': 'npc', 'color': [0.7, 0.2, 0.8], 'transform': {'position': [0, 0.8, 0], 'size': [0.6, 1.6, 0.6]}, 'npc': {'surface': 'floor', 'speed': 2, 'stop_distance': 1, 'enabled': False}, 'vitality': {'initial': 100, 'maximum': 100, 'scale': 1}}
        c.call('set_npc_templates', templates=[t])
        g = {'id': 'camp', 'template': 'guard', 'points': [[0, 0.8, 0], [3, 0.8, 0], [-3, 0.8, 0]], 'seed': 42, 'target_alive': 2, 'interval_ticks': 5, 'attempts_per_cycle': 3, 'max_births': 10}
        c.call('set_npc_generators', generators=[g])
        def info():
            return c.call('inspect_generators')
        def state():
            return info()['state']['generators'][0]
        def token(member):
            iid = f"{member['instance']['instance']:016x}"
            return next(i['reference'] for i in c.call('inspect_instances')['instances'] if i['instance_id'] == iid)
        assert state()['births'] == 0
        authored = c.call('inspect')['scene']
        c.call('step', ticks=1)
        first_state = info()
        assert state()['births'] == 2 and info()['counts'][0]['alive'] == 2
        assert c.call('inspect_population')['recorded_inputs'] == 0
        assert c.call('inspect')['scene'] == authored
        c.call('capture', path='artifacts/generators-initial.png', width=800, height=600)
        first = next(m for m in state()['members'] if m)
        old_token = token(first)
        c.call('activate_ability', actor='player', slot=1, target=old_token)
        c.call('step', ticks=6)
        assert info()['counts'][0] == {'present': 3, 'alive': 2, 'dead': 1}
        c.call('activate_ability', actor='player', slot=2, target=old_token)
        c.call('step', ticks=6)
        assert info()['counts'][0] == {'present': 3, 'alive': 3, 'dead': 0}
        assert state()['births'] == 3
        checks.append('MCP authoring, deterministic automatic births, corpse retention and revival above target without evicting individuals; no derived input log')

        c.call('capture', path='artifacts/generators-live.png', width=800, height=600)
        c.call('capture', path='artifacts/generators-repeat.png', width=800, height=600)
        c.call('save_replay', path='artifacts/generators.replay.json')
        assert c.call('run_replay', path='artifacts/generators.replay.json')['matched']
        prefix = c.call('inspect_replay', path='artifacts/generators.replay.json', tick=1)['snapshot']['generators']
        assert prefix == first_state
        c.call('capture_replay', replay_path='artifacts/generators.replay.json', path='artifacts/generators-prefix.png', tick=1, width=800, height=600)
        assert (ROOT/'artifacts/generators-initial.png').read_bytes() == (ROOT/'artifacts/generators-prefix.png').read_bytes()
        c.call('capture_replay', replay_path='artifacts/generators.replay.json', path='artifacts/generators-replay.png', width=800, height=600)
        pixels = (ROOT/'artifacts/generators-live.png').read_bytes()
        assert pixels == (ROOT/'artifacts/generators-repeat.png').read_bytes() == (ROOT/'artifacts/generators-replay.png').read_bytes()
        c.call('capture', path='artifacts/generators-viewport.png', width=640, height=480)
        checks.append('Replay79 includes generator RNG/ownership/counters/receipts; prefix and full offscreen captures match live pixels and repeated capture')

        retirement = {**g, 'target_alive': 1, 'points': [[0, 0.8, 0]], 'interval_ticks': 1, 'retire_dead_after_ticks': 2}
        c.call('set_npc_generators', generators=[retirement])
        retirement_authored = info()['definitions']
        c.call('step', ticks=1)
        old = next(m for m in state()['members'] if m)
        old_token = token(old)
        c.call('activate_ability', actor='player', slot=1, target=old_token)
        c.call('step', ticks=1)
        c.call('step', ticks=3)
        assert state()['births'] == 2
        fresh = next(m for m in state()['members'] if m)
        assert fresh['instance'] != old['instance']
        c.call('activate_ability', actor=old_token, slot=1, expected_error=True)
        c.call('despawn_actor', actor=token(fresh))
        c.call('save_replay', path='artifacts/generators-pending.replay.json')
        assert c.call('run_replay', path='artifacts/generators-pending.replay.json')['matched']
        c.call('step', ticks=1)
        assert state()['births'] == 3
        c.call('save_replay', path='artifacts/generators-retired.replay.json')
        assert c.call('run_replay', path='artifacts/generators-retired.replay.json')['matched']
        checks.append('Explicit corpse retirement and external despawn replenish with new identity; stale token rejection and pending external input replay')

        before = c.call('inspect')
        for patch in [{'interval_ticks': 0}, {'template': 'missing'}, {'attempts_per_cycle': 9}, {'points': []}, {'max_births': 0}]:
            c.call('set_npc_generators', generators=[{**g, **patch}], expected_error=True)
            assert c.call('inspect') == before
        c.call('set_npc_templates', templates=[], expected_error=True)
        assert c.call('inspect') == before
        blocked = {**g, 'points': [[0, 0.8, -4], [0, 9, 0]], 'attempts_per_cycle': 8, 'interval_ticks': 1}
        c.call('set_npc_generators', generators=[blocked])
        ids = c.call('inspect_instances')['instances']
        c.call('step', ticks=130)
        assert state()['births'] == 0 and state()['attempts'] == 260
        assert c.call('inspect_instances')['instances'] == ids
        assert len(info()['state']['receipts']) == 256 and info()['state']['dropped_receipts'] == 4
        assert {r['outcome'] for r in info()['state']['receipts']} == {'occupied', 'unsupported_position'}
        c.call('undo')
        assert state()['births'] == 0 and info()['definitions'] == retirement_authored
        c.call('redo')
        c.call('reset')
        assert state()['attempts'] == 0
        checks.append('Atomic invalid edits/template dependency, occupied/unsupported retries consume no identity, bounded receipts and attempts; undo/redo/reset rebuild from authorship')

        # Human demo: death/removal after three seconds; another available point can replenish sooner.
        demo = {**g, 'interval_ticks': 60, 'max_births': 20, 'retire_dead_after_ticks': 180}
        c.call('set_npc_generators', generators=[demo])
        c.call('set_camera', camera={'position': [9, 12, 12], 'target': [0, 0.5, -1.5], 'fov_degrees': 48})
        c.call('save_scene', path='examples/npc-generators.scene.json')
        c.call('step', ticks=1)
        c.call('capture', path='artifacts/generators-demo.png', width=800, height=600)
        result = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/generators-verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': result['build_id']}))
    finally:
        c.close()

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args()
    verify(args.binary.resolve(), args.window)
