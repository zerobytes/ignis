#!/usr/bin/env python3
"""Verify physical orbital travel, rather than leaf rotation, in the whirlwind."""
import argparse
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    args = parser.parse_args()
    out = ROOT / 'artifacts/wind/vortex'
    out.mkdir(parents=True, exist_ok=True)
    client = Client(args.binary.resolve(), log_path=out / 'verification-runtime.log',
                    startup_args=('--scene', 'examples/wind-vortex.scene.json'))
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                      'clientInfo': {'name': 'vortex-verification', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        positions = [[4 * math.cos(a * math.tau / 16), 4., 4 * math.sin(a * math.tau / 16)]
                     for a in range(16)]
        velocities = client.call('wind', operation={'op': 'sample', 'positions': positions,
                                                   'time_seconds': 2.})['velocity_mps']
        tangents = [(p[2] * v[0] - p[0] * v[2]) / 4 for p, v in zip(positions, velocities)]
        circulation = sum(tangents) * (math.tau * 4 / len(tangents))
        assert min(tangents) > .5 and circulation > 15., (tangents, circulation)
        sweep = [0.] * 64
        previous = None
        rows = []
        for tick in range(0, 2401, 30):
            if tick:
                client.call('step', ticks=30)
            bodies = client.call('wind', operation={'op': 'inspect'})['bodies']
            current = [body['position'] for body in bodies]
            if previous is not None:
                for i, (a, b) in enumerate(zip(previous, current)):
                    sweep[i] -= math.atan2(a[0] * b[2] - a[2] * b[0],
                                           a[0] * b[0] + a[2] * b[2])
            if tick in [240, 720, 1200, 2400]:
                airborne = [body for body in bodies if body['position'][1] > .3]
                tangent = [(b['position'][2] * b['velocity'][0] - b['position'][0] * b['velocity'][2])
                           / max(.1, math.hypot(b['position'][0], b['position'][2])) for b in airborne]
                rows.append({'seconds': tick / 60, 'airborne': len(airborne),
                             'over_90_degrees': sum(a > math.pi / 2 for a in sweep),
                             'over_180_degrees': sum(a > math.pi for a in sweep),
                             'mean_signed_sweep_degrees': sum(sweep) / 64 * 180 / math.pi,
                             'mean_radius_m': sum(math.hypot(p[0], p[2]) for p in current) / 64,
                             'mean_tangent_speed_mps': sum(tangent) / max(1, len(tangent))})
            previous = current
        assert rows[1]['over_90_degrees'] >= 40, rows
        assert rows[-1]['over_180_degrees'] >= 48, rows
        assert rows[-1]['airborne'] >= 40, rows
        assert rows[-1]['mean_tangent_speed_mps'] > .5, rows
        assert 1. < rows[-1]['mean_radius_m'] < 18., rows
        client.call('save_replay', path='artifacts/wind/vortex/current.replay.json')
        replay = client.call('run_replay', path='artifacts/wind/vortex/current.replay.json')
        assert replay['matched'] and replay['wind_matched'], replay
        report = {'passed': True, 'build_id': client.call('inspect')['build_id'],
                  'circulation_m2ps': circulation, 'minimum_ring_tangent_speed_mps': min(tangents),
                  'samples': rows, 'replay_matched': True}
        (out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))
    finally:
        client.close()


if __name__ == '__main__':
    main()
