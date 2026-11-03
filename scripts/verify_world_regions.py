#!/usr/bin/env python3
"""World residency planner: proposals only, exact addresses and no live-engine mutation."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def fixture():
    return {
        'manifest': {'version': 1, 'space_version': 1, 'world_id': 'test-world', 'revision': 1, 'seed': '18446744073709551615', 'generator': {'id': 'terrain', 'version': 3}, 'region_size_meters': 1024, 'budget': {'occupied': 2, 'requests': 2, 'probes': 64}},
        'interests': [{'id': 'player', 'position': {'cell': ['0', '0', '0'], 'offset': [0, 0, 0]}, 'priority': 0, 'load_radius': [0, 0, 0], 'retain_radius': [1, 0, 0]}],
        'observed': [{'key': {'cell': ['-1', '0', '0'], 'offset_meters': [0, 0, 0]}, 'phase': 'resident', 'pins': []}],
    }


def verify(binary, window):
    c = Client(binary, window)
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'world-regions', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        before = c.call('inspect')
        q = fixture()
        p = c.call('plan_world_regions', query=q)
        assert p['proposals_only'] and len(p['loads']) == 1 and p['loads'][0]['key']['cell'] == ['0', '0', '0']
        assert p['loads'][0]['seed'] == '15003469448002236603' and not p['unload_candidates']
        q['interests'][0]['position']['cell'][0] = '1'
        p = c.call('plan_world_regions', query=q)
        assert len(p['unload_candidates']) == 1
        q['observed'][0]['pins'] = ['actor', 'delivery']
        p = c.call('plan_world_regions', query=q)
        assert not p['unload_candidates'] and p['totals']['held_by_pins'] == 1
        q['manifest']['budget']['occupied'] = 1
        p = c.call('plan_world_regions', query=q)
        assert not p['loads'] and p['totals']['deferred_capacity'] == 1
        q = fixture()
        q['observed'] = []
        q['manifest']['budget'] = {'occupied': 16, 'requests': 16, 'probes': 9}
        base = q['interests'][0]
        base['position']['cell'] = ['9007199254740993', '0', '-9007199254740993']
        base['load_radius'] = [1, 0, 1]
        base['retain_radius'] = [2, 0, 2]
        q['interests'] = [dict(copy.deepcopy(base), id=f'player-{i:04}') for i in range(1000)]
        p = c.call('plan_world_regions', query=q)
        assert p['totals']['groups'] == 1 and p['totals']['probes'] == 9 and len(p['loads']) == 9
        q['interests'].reverse()
        assert c.call('plan_world_regions', query=q) == p
        q['manifest']['budget']['probes'] = 8
        c.call('plan_world_regions', query=q, expected_error=True)
        assert c.call('inspect') == before
        report = {'passed': True, 'native_window': window, 'build_id': before['build_id'], 'checks': ['Hysteresis and pins preserve proposed retention; proposed unload never releases occupied capacity', '1000 equivalent interests fold to nine probes beyond 2^53, input order and seeds stable, invalid budgets preserve the live engine; proposals are not installed regions']}
        (ROOT / 'artifacts/world-regions-verification.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    p.add_argument('--window', action='store_true')
    a = p.parse_args()
    verify(a.binary.resolve(), a.window)
