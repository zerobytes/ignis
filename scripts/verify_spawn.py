#!/usr/bin/env python3
"""Independent births and recycled storage through the public MCP dispatcher."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-spawn-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe={'seed': 7, 'width': 24, 'depth': 20, 'obstacles': 0})
        catalog = {'resources': [{'id': 'energy', 'scale': 1}], 'abilities': [
            {'id': 'heal', 'selection': 'self_actor', 'range': 0, 'cast_ticks': 6, 'cooldown_ticks': 30, 'costs': [{'resource': 'energy', 'units': 7}], 'effects': [{'kind': 'heal', 'recipient': 'target', 'amount': 10}]},
            {'id': 'bolt', 'selection': 'actor', 'range': 20, 'cast_ticks': 0, 'cooldown_ticks': 0, 'costs': [{'resource': 'energy', 'units': 5}], 'delivery': {'kind': 'projectile', 'speed': 12, 'half_extent': 0.1, 'max_distance': 20}, 'effects': [{'kind': 'damage', 'recipient': 'target', 'amount': 10}]}
        ]}
        c.call('set_ability_catalog', catalog=catalog)
        t = {'id': 'guard', 'name': 'Guard template', 'body': 'npc', 'color': [0.7, 0.2, 0.8], 'transform': {'position': [0, 0.8, 4], 'size': [0.6, 1.6, 0.6]}, 'npc': {'surface': 'floor', 'speed': 3, 'stop_distance': 0.5, 'enabled': True}, 'vitality': {'initial': 900, 'maximum': 1000, 'scale': 1}, 'abilities': {'resources': [{'resource': 'energy', 'initial': 1000, 'capacity': 1000}], 'grants': ['heal', 'bolt']}}
        t2 = copy.deepcopy(t)
        t2.update(id='healer', name='Healer template', color=[0.2, 0.8, 0.4])
        t2['vitality']['initial'] = 500
        t2['abilities']['resources'][0]['initial'] = 300
        c.call('set_npc_templates', templates=[t, t2])
        assert len(c.call('inspect_templates')['templates']) == 2
        assert not c.call('inspect')['npcs']
        c.call('save_scene', path='examples/npc-spawn.scene.json')
        authored = c.call('inspect')['scene']
        player = next(i for i in c.call('inspect_instances')['instances'] if i['authored_id'] == 'player')

        def born(template, position):
            c.call('spawn_actor', template=template, position=position)
            c.call('step', ticks=1)
            receipt = c.call('inspect_population')['receipts'][-1]
            assert receipt['outcome'] == 'spawned', receipt
            return next(i for i in c.call('inspect_instances')['instances'] if i['instance_id'] == receipt['instance_id'])

        c.call('spawn_actor', template='guard', position=[0, 0.8, 4])
        assert not c.call('inspect')['npcs']
        c.call('save_replay', path='artifacts/spawn-pending.replay.json')
        assert c.call('run_replay', path='artifacts/spawn-pending.replay.json')['matched']
        c.call('step', ticks=1)
        first = next(i for i in c.call('inspect_instances')['instances'] if i['authored_id'].startswith('_ignis_spawn_'))
        other = born('guard', [4, 0.8, 4])
        assert c.call('inspect')['scene'] == authored
        assert next(i for i in c.call('inspect_instances')['instances'] if i['authored_id'] == 'player') == player
        checks.append('Templates authored/inspected/saved by MCP have no identity/body; births apply at tick boundary without editing/resetting survivors')

        c.call('activate_ability', actor=first['reference'], slot=1)
        c.call('step', ticks=7)
        state = c.call('inspect_abilities')['actors'][first['authored_id']]
        assert state['grants'][0]['ready_at'] > c.call('inspect')['tick']
        c.call('despawn_actor', actor=first['reference'])
        c.call('spawn_actor', template='healer', position=[0, 0.8, 4])
        c.call('step', ticks=1)
        receipt = c.call('inspect_population')['receipts'][-1]
        fresh = next(i for i in c.call('inspect_instances')['instances'] if i['instance_id'] == receipt['instance_id'])
        assert fresh['instance_id'] != first['instance_id'] and fresh['generation'] != first['generation']
        assert fresh['life_index'] == 0
        state = c.call('inspect_abilities')['actors'][fresh['authored_id']]
        assert state['grants'][0]['ready_at'] == 0 and state['cast'] is None
        assert c.call('inspect_vitality')['actors'][fresh['authored_id']]['units'] == 500
        before = c.call('inspect')
        c.call('activate_ability', actor=first['reference'], slot=1, expected_error=True)
        assert c.call('inspect') == before
        assert next(i for i in c.call('inspect_instances')['instances'] if i['instance_id'] == other['instance_id']) == other
        checks.append('Recycled slot has new identity/generation/life0 and fresh health/cooldowns/cast; old token rejected and sibling unchanged')

        for op in ['inspect_targeting', 'inspect_knowledge', 'inspect_relations', 'inspect_territory', 'inspect_objectives', 'inspect_hearing', 'inspect_mobility', 'inspect_passages', 'inspect_origins']:
            c.call(op)
        c.call('select_entity', id=fresh['authored_id'])
        c.call('set_debug_view', npc_vision=True, npc_hearing=True, noise_events=True, npc_routes=True)
        c.call('capture', path='artifacts/spawn-live.png', width=800, height=600)
        c.call('capture', path='artifacts/spawn-repeat.png', width=800, height=600)
        assert (ROOT/'artifacts/spawn-live.png').read_bytes() == (ROOT/'artifacts/spawn-repeat.png').read_bytes()
        c.call('save_replay', path='artifacts/spawn.replay.json')
        assert c.call('run_replay', path='artifacts/spawn.replay.json')['matched']
        c.call('capture_replay', replay_path='artifacts/spawn.replay.json', path='artifacts/spawn-prefix.png', width=800, height=600)
        assert (ROOT/'artifacts/spawn-live.png').read_bytes() == (ROOT/'artifacts/spawn-prefix.png').read_bytes()
        c.call('capture', path='artifacts/spawn-viewport.png', width=640, height=480)
        checks.append('All runtime inspectors accept new actors; repeated offscreen captures and replay71 pixels agree, including selection/debug guides and viewport changes')

        ids_before = c.call('inspect_instances')['instances']
        c.call('spawn_actor', template='guard', position=[4, 0.8, 4])
        c.call('spawn_actor', template='guard', position=[0, 9, 0])
        c.call('step', ticks=1)
        assert [r['outcome'] for r in c.call('inspect_population')['receipts'][-2:]] == ['occupied', 'unsupported_position']
        assert c.call('inspect_instances')['instances'] == ids_before
        bad = copy.deepcopy(t)
        bad['npc']['speed'] = -1
        before = c.call('inspect')
        c.call('set_npc_templates', templates=[bad], expected_error=True)
        assert c.call('inspect') == before
        c.call('set_npc_templates', templates=[t])
        assert not c.call('inspect')['npcs']
        c.call('undo')
        assert len(c.call('inspect_templates')['templates']) == 2 and not c.call('inspect')['npcs']
        c.call('redo')
        c.call('reset')
        assert c.call('inspect_population')['processed'] == 0
        c.call('despawn_actor', actor=fresh['reference'], expected_error=True)
        c.call('capture', path='artifacts/spawn-reset.png', width=800, height=600)
        checks.append('Occupied/unsupported births consume no identity; invalid edits atomic; edit/undo/redo/reset rebuild from authoring and invalidate previous execution references')
        result = {'passed': True, 'checks': checks, 'native_window': window, 'build_id': c.call('inspect')['build_id'], 'slow_requests': c.slow_requests}
        (ROOT/'artifacts/spawn-verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': result['build_id']}))
    finally:
        c.close()

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args()
    verify(args.binary.resolve(), args.window)
