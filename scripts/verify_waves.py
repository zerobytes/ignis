#!/usr/bin/env python3
"""Finite waves exercised through the real MCP dispatcher, with replay and offscreen evidence."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-waves-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        c.call('load_scene', path='examples/npc-generators.scene.json')
        t = c.call('inspect_templates')['templates'][0]
        elite = copy.deepcopy(t)
        elite.update(id='elite', name='Second wave', color=[0.9, 0.35, 0.1])
        elite['vitality']['initial'] = 40
        boss = copy.deepcopy(t)
        boss.update(id='boss', name='Final wave', color=[0.15, 0.5, 0.95])
        boss['vitality'].update(initial=200, maximum=200)
        c.call('set_npc_templates', templates=[t, elite, boss])
        g = {'id': 'encounter', 'template': 'guard', 'points': [[0, 0.8, 0], [3, 0.8, 0], [-3, 0.8, 0]], 'seed': 42, 'target_alive': 2, 'interval_ticks': 1, 'attempts_per_cycle': 3, 'max_births': 3, 'initial_delay_ticks': 2, 'retire_dead_after_ticks': 0, 'waves': [{'count': 2, 'delay_ticks': 3}, {'count': 1, 'template': 'elite', 'delay_ticks': 2}]}
        c.call('set_npc_generators', generators=[g])
        def info():
            return c.call('inspect_generators')
        def state():
            return info()['state']['generators'][0]
        def wave():
            return info()['waves'][0]
        def members():
            return [m for m in state()['members'] if m]
        def instance(m):
            iid = f"{m['instance']['instance']:016x}"
            return next(i for i in c.call('inspect_instances')['instances'] if i['instance_id'] == iid)
        def cast(slot, reference):
            c.call('activate_ability', actor='player', slot=slot, target=reference)
            c.call('step', ticks=1)
        assert wave()['phase'] == 'delay'
        c.call('step', ticks=5)
        assert state()['births'] == 0
        c.call('step', ticks=1)
        assert wave()['phase'] == 'waiting_for_clear' and state()['births'] == 2
        prefix_tick = c.call('inspect')['tick']
        prefix = info()
        c.call('capture', path='artifacts/waves-first.png', width=800, height=600)
        refs = [instance(m)['reference'] for m in members()]
        cast(1, refs[0])
        c.call('step', ticks=1)
        assert wave()['phase'] == 'waiting_for_clear' and state()['births'] == 2
        cast(1, refs[1])
        c.call('step', ticks=1)
        assert wave()['index'] == 1 and wave()['phase'] == 'delay' and state()['births'] == 2
        remaining = wave()['due_tick'] - c.call('inspect')['tick']
        if remaining:
            c.call('step', ticks=remaining)
        assert state()['births'] == 2
        c.call('step', ticks=1)
        assert state()['births'] == 3 and wave()['births'] == 1 and wave()['phase'] == 'waiting_for_clear'
        fresh = instance(members()[0])
        assert members()[0]['wave'] == 1
        assert c.call('inspect_vitality')['actors'][fresh['authored_id']]['units'] == 40
        assert c.call('inspect_population')['recorded_inputs'] == 0
        c.call('activate_ability', actor=refs[0], slot=1, expected_error=True)
        checks.append('MCP finite quotas, exact initial/interwave delays, no replenishment within completed quota, template switch and fresh identity')

        c.call('capture', path='artifacts/waves-second.png', width=800, height=600)
        c.call('capture', path='artifacts/waves-repeat.png', width=800, height=600)
        c.call('save_replay', path='artifacts/waves.replay.json')
        assert c.call('run_replay', path='artifacts/waves.replay.json')['matched']
        assert c.call('inspect_replay', path='artifacts/waves.replay.json', tick=prefix_tick)['snapshot']['generators'] == prefix
        c.call('capture_replay', replay_path='artifacts/waves.replay.json', path='artifacts/waves-prefix.png', tick=prefix_tick, width=800, height=600)
        c.call('capture_replay', replay_path='artifacts/waves.replay.json', path='artifacts/waves-replay.png', width=800, height=600)
        assert (ROOT/'artifacts/waves-first.png').read_bytes() == (ROOT/'artifacts/waves-prefix.png').read_bytes()
        pixels = (ROOT/'artifacts/waves-second.png').read_bytes()
        assert pixels == (ROOT/'artifacts/waves-repeat.png').read_bytes() == (ROOT/'artifacts/waves-replay.png').read_bytes()
        c.call('despawn_actor', actor=fresh['reference'])
        c.call('save_replay', path='artifacts/waves-pending.replay.json')
        assert c.call('run_replay', path='artifacts/waves-pending.replay.json')['matched']
        c.call('step', ticks=1)
        assert wave()['phase'] == 'completed'
        done = wave()['completed_tick']
        c.call('step', ticks=10)
        assert wave()['completed_tick'] == done and state()['births'] == 3
        assert [e['kind'] for e in info()['state']['wave_events']] == ['started', 'cleared', 'started', 'completed']
        c.call('save_replay', path='artifacts/waves-complete.replay.json')
        assert c.call('run_replay', path='artifacts/waves-complete.replay.json')['matched']
        checks.append('Replay79/pending inputs reproduce phase, membership and events; prefix/final/repeated pixels match; completed sequence stays finite')

        retained = {**g, 'initial_delay_ticks': 0, 'target_alive': 1, 'retire_dead_after_ticks': None, 'waves': [{'count': 1}, {'count': 1, 'template': 'elite', 'delay_ticks': 3}]}
        c.call('set_npc_generators', generators=[retained])
        c.call('step', ticks=1)
        old = instance(members()[0])
        cast(1, old['reference'])
        c.call('step', ticks=1)
        assert wave()['index'] == 1
        cast(2, old['reference'])
        c.call('step', ticks=3)
        assert wave()['index'] == 1 and state()['births'] == 1 and wave()['waiting_for'] == 'living_limit'
        c.call('despawn_actor', actor=old['reference'])
        c.call('step', ticks=1)
        assert state()['births'] == 2 and members()[0]['wave'] == 1
        removed = {**retained, 'waves': [{'count': 1, 'completion': 'removed'}]}
        c.call('set_npc_generators', generators=[removed])
        c.call('step', ticks=1)
        corpse = instance(members()[0])
        cast(1, corpse['reference'])
        c.call('step', ticks=3)
        assert wave()['phase'] == 'waiting_for_clear' and wave()['alive'] == 0 and wave()['present'] == 1
        c.call('despawn_actor', actor=corpse['reference'])
        c.call('step', ticks=1)
        assert wave()['phase'] == 'completed'
        checks.append('Revived earlier wave does not rewind and counts toward living limit; removed completion waits for explicit corpse removal')

        before = c.call('inspect')
        for waves in [[{'count': 0}], [{'count': 1, 'template': 'missing'}], [{'count': 4}], [{'count': 1}] * 17, [{'count': 1, 'completion': 'unknown'}]]:
            c.call('set_npc_generators', generators=[{**g, 'waves': waves}], expected_error=True)
            assert c.call('inspect') == before
        c.call('set_npc_generators', generators=[g])
        authored = info()['definitions']
        c.call('step', ticks=6)
        c.call('set_npc_templates', templates=[t], expected_error=True)
        c.call('set_npc_generators', generators=[])
        c.call('undo')
        assert info()['definitions'] == authored and state()['births'] == 0 and not info()['state']['wave_events']
        c.call('redo')
        assert not info()['definitions']
        c.call('undo')
        c.call('reset')
        assert state()['births'] == 0
        c.call('save_scene', path='artifacts/waves-roundtrip.scene.json')
        c.call('load_scene', path='artifacts/waves-roundtrip.scene.json')
        assert info()['definitions'] == authored
        checks.append('Invalid wave quotas/templates/conditions/budgets preserve state; template dependency, edit/undo/redo/reset and authored scene roundtrip')

        demo = {**g, 'initial_delay_ticks': 0, 'interval_ticks': 30, 'max_births': 5, 'retire_dead_after_ticks': 120,
                'waves': [{'count': 2}, {'count': 2, 'template': 'elite', 'delay_ticks': 120}, {'count': 1, 'template': 'boss', 'delay_ticks': 120}]}
        c.call('set_npc_generators', generators=[demo])
        c.call('save_scene', path='examples/npc-waves.scene.json')
        c.call('load_scene', path='examples/npc-waves.scene.json')
        c.call('step', ticks=1)
        c.call('capture', path='artifacts/waves-demo.png', width=800, height=600)
        for index in range(3):
            if wave()['phase'] == 'delay':
                c.call('step', ticks=max(1, wave()['due_tick'] - c.call('inspect')['tick'] + 1))
            assert wave()['index'] == index and wave()['phase'] == 'waiting_for_clear'
            current = [instance(m) for m in members() if m['wave'] == index]
            assert len(current) == (1 if index == 2 else 2)
            if index == 2:
                assert c.call('inspect_vitality')['actors'][current[0]['authored_id']]['units'] == 200
            for actor in current:
                cast(1, actor['reference'])
                if index == 2:
                    assert c.call('inspect_vitality')['actors'][actor['authored_id']]['units'] == 100
                    cast(1, actor['reference'])
            c.call('step', ticks=1)
        assert wave()['phase'] == 'completed' and state()['births'] == 5
        c.call('save_replay', path='artifacts/waves-demo.replay.json')
        assert c.call('run_replay', path='artifacts/waves-demo.replay.json')['matched']
        checks.append('Saved three-wave demo completes five births with two-second intermissions and a final200-health NPC requiring two100-damage hits')
        result = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/waves-verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': result['build_id']}))
    finally:
        c.close()

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args()
    verify(args.binary.resolve(), args.window)
