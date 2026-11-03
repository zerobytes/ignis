#!/usr/bin/env python3
"""Profile continuous native planet movement over MCP; no window or GPU capture.

Each sample advances exactly one simulation tick. RTT includes stdio/JSON and is
not presentation FPS. Residence/bake counters identify synchronous transitions.
"""
import argparse
import json
import statistics
import time
from pathlib import Path
from verify_lab import Client, ROOT
from open_earth_walk import init, view

OUT = ROOT / 'artifacts/planet-pacing'


def percentile(values, q):
    return sorted(values)[min(len(values)-1, int((len(values)-1)*q))]


def run(binary, output, ticks, distance, width, height, cache, paced=True, geometry=True):
    OUT.mkdir(parents=True, exist_ok=True)
    client = Client(binary, log_path=OUT/'stdio.log')
    rows = []
    try:
        init(client)
        client.call('set_execution_mode', mode='continuous')
        client.call('load_scene', path='artifacts/planet-runtime/player.scene.json')
        client.call('load_planet_runtime', path='artifacts/planet-runtime/world.json')
        client.call('step', ticks=120)
        client.call('set_player_view', view=view())
        client.call('set_planet_view', view={'viewport': [width, height], 'distance_meters': distance,
                                           'reuse_vertex_samples': cache, 'incremental_generation': paced,
                                           'atmosphere_geometry': geometry})
        previous = client.call('inspect_planet_view')
        world = client.call('inspect_planet_runtime')
        build = client.call('inspect')['build_id']
        for segment, count in [('idle', 12), ('run', ticks), ('pan', 12)]:
            client.call('set_player_view_input', input={'x': 0, 'z': -1 if segment == 'run' else 0, 'mode': 'run'})
            for index in range(count):
                started = time.perf_counter()
                if segment == 'pan':
                    client.call('look_planet_view', yaw_delta_degrees=2, tilt_delta_degrees=0)
                client.call('step', ticks=1)
                elapsed = (time.perf_counter()-started)*1000
                current = client.call('inspect_planet_view')
                physical = client.call('inspect_planet_runtime')
                row = {'segment': segment, 'index': index, 'elapsed_ms': elapsed,
                       'tick': physical['tick'], 'path_meters': physical['actual_path_meters'],
                       'plans': current['residence_plans']-previous['residence_plans'],
                       'bakes': current['geometry_builds']-previous['geometry_builds'],
                       'groups': current['group_builds']-previous['group_builds'],
                       'world_births': physical['tile_births']-world['tile_births'],
                       'view': current, 'world': physical}
                rows.append(row)
                if elapsed > 30:
                    print(json.dumps({k: row[k] for k in ['segment','index','elapsed_ms','plans','bakes','groups','world_births']}), flush=True)
                previous, world = current, physical
        summary = {}
        for segment in ['idle', 'run', 'pan']:
            samples = [r['elapsed_ms'] for r in rows if r['segment'] == segment]
            summary[segment] = {'count': len(samples), 'median_ms': statistics.median(samples),
                                'p95_ms': percentile(samples, .95), 'p99_ms': percentile(samples, .99),
                                'maximum_ms': max(samples), 'over_33ms': sum(x>33.33 for x in samples),
                                'over_100ms': sum(x>100 for x in samples)}
        before_sync = client.call('inspect_planet_runtime')
        final_view = client.call('set_planet_view', view=current['settings'])
        after_sync = client.call('inspect_planet_runtime')
        assert before_sync['actual_path_meters'] == after_sync['actual_path_meters']
        assert before_sync['tick'] == after_sync['tick']
        assert final_view['generation']['pending_tiles'] == 0
        report = {'build_id': build, 'incremental_generation': paced, 'atmosphere_geometry': geometry,
                  'final_view_after_explicit_sync': final_view, 'method': 'one tick per MCP sample, no draw/present; RTT includes IPC',
                  'summary': summary, 'samples': rows}
        (OUT/output).write_text(json.dumps(report, indent=2))
        print(json.dumps(summary, indent=2), flush=True)
    finally:
        client.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--output', default='current.json')
    parser.add_argument('--ticks', type=int, default=120)
    parser.add_argument('--distance', type=float, default=4)
    parser.add_argument('--width', type=int, default=2248)
    parser.add_argument('--height', type=int, default=1580)
    parser.add_argument('--cache-off', action='store_true')
    parser.add_argument('--eager', action='store_true')
    parser.add_argument('--geometry-off', action='store_true')
    args = parser.parse_args()
    if not 1 <= args.ticks <= 3600 or Path(args.output).name != args.output:
        parser.error('ticks1..3600 and a local output filename required')
    run(args.binary, args.output, args.ticks, args.distance, args.width, args.height, not args.cache_off, not args.eager, not args.geometry_off)
