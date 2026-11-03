#!/usr/bin/env python3
"""Critical MCP boundary: large global addresses remain exact and queries cannot mutate a game."""
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'world-space', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        before = c.call('inspect')
        origin = {'cell': ['9007199254740993', '-9007199254740993', '0'], 'offset': [1023.5, 0.25, 0]}
        shifted = c.call('query_world_space', query={'kind': 'translate', 'point': origin, 'delta': [1.25, -0.5, 0]})['point']
        assert shifted == {'cell': ['9007199254740994', '-9007199254740994', '0'], 'offset': [0.75, 1023.75, 0]}, shifted
        result = c.call('query_world_space', query={'kind': 'project', 'point': shifted, 'origin': origin, 'radius_meters': 4, 'max_error_meters': 0})
        assert result['local'] == [1.25, -0.5, 0] and result['origin'] == origin
        normalized = c.call('query_world_space', query={'kind': 'normalize', 'point': {'cell': ['0'] * 3, 'offset': [-0.5, 1024.25, 0]}})['point']
        assert normalized == {'cell': ['-1', '1', '0'], 'offset': [1023.5, 0.25, 0]}
        c.call('query_world_space', query={'kind': 'translate', 'point': {'cell': ['9223372036854775807', '0', '0'], 'offset': [1023.5, 0, 0]}, 'delta': [1, 0, 0]}, expected_error=True)
        c.call('query_world_space', query={'kind': 'project', 'point': shifted, 'origin': {'cell': ['0'] * 3, 'offset': [0] * 3}, 'radius_meters': 4, 'max_error_meters': 0}, expected_error=True)
        assert c.call('inspect') == before
        report = {'passed': True, 'native_window': window, 'build_id': before['build_id'], 'checks': ['Exact signed i64 cells across JSON/MCP, normalization and nearby f32 projection beyond 2^53', 'Overflow/out-of-window refusal preserves the live engine; no streaming or rebasing claimed']}
        (ROOT / 'artifacts/world-space-verification.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
