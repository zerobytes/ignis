#!/usr/bin/env python3
"""Measure current native-window cadence; close only this diagnostic window."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import time

from groundcover_assets import atomic_json
from verify_lab import Client, ROOT

OUT = ROOT / 'artifacts/emerald-meadow'


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    args = p.parse_args()
    views = json.loads((OUT / 'views.json').read_text())
    lights = json.loads((OUT / 'lights.json').read_text())
    bundle = json.loads((OUT / 'bundle.json').read_text())
    for key in ['scene', 'field']:
        assert hashlib.sha256((OUT / bundle[key]).read_bytes()).hexdigest() == bundle[key + '_sha256']
    report = dict(method='Native editor present-call cadence; no GPU timestamp or scanout claim',
        scene_sha256=bundle['scene_sha256'], field_sha256=bundle['field_sha256'],
        native_binary_sha256=hashlib.sha256(args.binary.read_bytes()).hexdigest(), rows=[])
    previous_field = os.environ.get('IGNIS_HORIZON_FIELD')
    previous_proxy = os.environ.pop('IGNIS_HORIZON_PROXIES', None)
    os.environ['IGNIS_HORIZON_FIELD'] = str(OUT / bundle['field'])
    try:
        c = Client(args.binary.resolve(), window=True, log_path=OUT / 'presentation.log',
            startup_args=('--scene', str((OUT / bundle['scene']).relative_to(ROOT)), '--continuous'))
    finally:
        if previous_field is None:
            os.environ.pop('IGNIS_HORIZON_FIELD', None)
        else:
            os.environ['IGNIS_HORIZON_FIELD'] = previous_field
        if previous_proxy is not None:
            os.environ['IGNIS_HORIZON_PROXIES'] = previous_proxy
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
            clientInfo=dict(name='emerald-measure', version='1')))
        c.request('notifications/initialized', notification=True)
        report['build_id'] = c.call('inspect')['build_id']
        report['pid'] = c.process.pid
        report['concurrent_processes_start'] = subprocess.check_output(
            ['ps', '-axo', 'pid=,pcpu=,comm='], text=True)
        c.call('set_viewport_performance', settings=dict(enabled=True, vsync=False))
        c.call('play')
        for name in ['walk', 'hills', 'panorama', 'close']:
            c.call('set_camera', camera=views[name])
            c.call('set_light', light=lights[name])
            time.sleep(2)
            samples = []
            for _ in range(10):
                time.sleep(.55)
                samples.append(c.call('inspect_viewport_performance'))
            valid = [s['sample']['fps'] for s in samples if s['sample']['fps'] is not None and s['age_ms'] < 1200]
            assert valid, f'No fresh presentation samples for {name}'
            row = dict(camera=name, fps_median=statistics.median(valid), fps_range=[min(valid), max(valid)], samples=samples)
            report['rows'].append(row)
            atomic_json(OUT / 'presentation.json', report)
            print(json.dumps(dict(camera=name, fps_median=row['fps_median'],
                viewport=samples[-1]['sample']['viewport'], mode=samples[-1]['sample']['present_mode'])), flush=True)
    finally:
        c.close()


if __name__ == '__main__':
    main()
