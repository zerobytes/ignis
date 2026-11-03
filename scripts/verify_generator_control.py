#!/usr/bin/env python3
"""Runtime generator controls through MCP, with isolated replay and offscreen captures."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-generator-control-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        c.call('load_scene', path='examples/npc-waves.scene.json')
        original = c.call('inspect_generators')['definitions'][0]
        g = copy.deepcopy(original)
        g.update(initially_active=False, initial_delay_ticks=2, interval_ticks=4, retire_dead_after_ticks=2)
        g['waves'][0]['delay_ticks'] = 3
        c.call('set_npc_generators', generators=[g])
        def info(): return c.call('inspect_generators')
        def state(): return info()['state']['generators'][0]
        def status(): return info()['controls'][0]
        def control(action): return c.call('control_npc_generator', generator=g['id'], action=action)
        def last(): return c.call('inspect_population')['receipts'][-1]['outcome']
        def refs():
            instances = c.call('inspect_instances')['instances']
            return [next(i['reference'] for i in instances if i['instance_id'] == f"{m['instance']['instance']:016x}") for m in state()['members'] if m]
        def replay(path):
            c.call('save_replay', path=path)
            assert c.call('run_replay', path=path)['matched']
        c.call('step', ticks=20)
        assert state()['births'] == 0 and status()['remaining_delay_ticks'] == 5
        assert control('activate')['phase'] == 'pending' and status()['status'] == 'paused'
        replay('artifacts/generator-control-pending.replay.json')
        assert c.call('inspect_population')['pending'][0]['generator'] == g['id']
        c.call('step', ticks=5)
        assert state()['births'] == 0 and last() == 'control_applied'
        c.call('step', ticks=1)
        assert state()['births'] == 2
        control('pause')
        c.call('step', ticks=1)
        frozen_rng = state()['rng']
        assert status()['status'] == 'paused'
        prefix_tick = c.call('inspect')['tick']
        prefix_state = info()
        c.call('capture', path='artifacts/generator-control-paused.png', width=800, height=600)
        checks.append('Inactive authored start freezes combined initial/wave delay; activation is queued, replayable and preserves quota/seed')

        for ref in refs():
            c.call('activate_ability', actor='player', slot=1, target=ref)
            c.call('step', ticks=1)
        c.call('step', ticks=10)
        assert info()['counts'][0]['present'] == 0
        assert state()['wave']['current'] == 0 and state()['wave']['phase'] == 'waiting_for_clear'
        assert state()['rng'] == frozen_rng
        control('pause'); c.call('step', ticks=1)
        assert last() == 'control_unchanged'
        control('activate'); c.call('step', ticks=1)
        assert state()['wave']['current'] == 1 and state()['wave']['phase'] == 'delay'
        c.call('step', ticks=20)
        remaining = status()['remaining_delay_ticks']
        control('pause'); c.call('step', ticks=30)
        assert status()['remaining_delay_ticks'] == remaining
        control('activate'); c.call('step', ticks=remaining)
        assert state()['births'] == 2
        c.call('step', ticks=1)
        assert state()['births'] == 4
        checks.append('Paused NPCs remain damageable and corpses retire; wave completion waits for resume and interwave delay retains exact remaining ticks')

        control('cancel'); c.call('step', ticks=1)
        assert status()['terminal'] == 'cancelled' and info()['counts'][0]['alive'] == 2
        control('activate'); control('pause'); control('cancel'); c.call('step', ticks=1)
        assert [r['outcome'] for r in c.call('inspect_population')['receipts'][-3:]] == ['control_terminal', 'control_terminal', 'control_unchanged']
        c.call('capture', path='artifacts/generator-control-cancelled.png', width=800, height=600)
        path = 'artifacts/generator-control.replay.json'
        replay(path)
        c.call('capture_replay', replay_path=path, path='artifacts/generator-control-prefix.png', tick=prefix_tick, width=800, height=600)
        c.call('capture_replay', replay_path=path, path='artifacts/generator-control-replay.png', width=800, height=600)
        assert c.call('inspect_replay', path=path, tick=prefix_tick)['snapshot']['generators'] == prefix_state
        assert (ROOT/'artifacts/generator-control-prefix.png').read_bytes() == (ROOT/'artifacts/generator-control-paused.png').read_bytes()
        assert (ROOT/'artifacts/generator-control-replay.png').read_bytes() == (ROOT/'artifacts/generator-control-cancelled.png').read_bytes()
        for ref in refs():
            c.call('activate_ability', actor='player', slot=1, target=ref); c.call('step', ticks=1)
        c.call('step', ticks=130)
        assert state()['births'] == 4 and info()['counts'][0]['present'] == 0
        checks.append('Cancel is terminal without killing survivors; repeated requests report outcomes, cleanup continues, full/prefix replay state and pixels match')

        before = c.call('inspect')
        c.call('control_npc_generator', generator='missing', action='pause', expected_error=True)
        c.call('control_npc_generator', generator=g['id'], action='restart', expected_error=True)
        c.call('set_npc_generators', generators=[{**g, 'initially_active': 'false'}], expected_error=True)
        assert c.call('inspect') == before
        c.call('save_scene', path='artifacts/generator-control-roundtrip.scene.json')
        c.call('load_scene', path='artifacts/generator-control-roundtrip.scene.json')
        assert status()['status'] == 'paused' and state()['births'] == 0
        c.call('set_npc_generators', generators=[{**g, 'initially_active': True}]); c.call('undo')
        assert status()['status'] == 'paused'
        c.call('redo'); assert status()['status'] == 'active'
        control('pause');control('activate');control('cancel');control('activate');c.call('step',ticks=1)
        assert status()['status'] == 'cancelled' and state()['births'] == 0 and last() == 'control_terminal'
        c.call('reset');assert status()['status'] == 'active'
        demo = {**original, 'initially_active': False}
        c.call('set_npc_generators', generators=[demo])
        c.call('save_scene', path='examples/npc-generator-control.scene.json')
        c.call('load_scene', path='examples/npc-generator-control.scene.json')
        c.call('step', ticks=60);assert state()['births'] == 0
        control('activate');c.call('step',ticks=1);assert state()['births'] == 2
        c.call('capture', path='artifacts/generator-control-demo.png', width=800, height=600)
        checks.append('Invalid commands are atomic; save/load/undo/redo/reset preserve authored activation, same-tick order, MCP-created paused demo')
        result={'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/generator-control-verification.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:
        c.close()

if __name__ == '__main__':
    p=argparse.ArgumentParser()
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    p.add_argument('--window',action='store_true')
    args=p.parse_args()
    verify(args.binary.resolve(),args.window)
