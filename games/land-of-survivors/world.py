#!/usr/bin/env python3
"""Open or inspect this project's native world using the current Ignis executable."""
import argparse
import json
import math
from pathlib import Path
import sys

PROJECT = Path(__file__).resolve().parent
ENGINE = PROJECT.parents[1]
sys.path.insert(0, str(ENGINE / 'scripts'))
import verify_lab


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--open', action='store_true', help='Open the native globe window')
    parser.add_argument('--view', choices=['region', 'ground', 'globe'], default='region')
    parser.add_argument('--binary', type=Path, default=ENGINE / 'target/release/ignis')
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error('Build Ignis first; this project uses its current executable without copying it')
    source = json.loads((PROJECT / 'world/sphere.planet.json').read_text())
    config = json.loads((PROJECT / 'world/runtime.json').read_text())
    if config['document'] != source:
        parser.error('The runtime and authored planet source differ; update the runtime explicitly')
    artifacts = PROJECT / 'artifacts'
    artifacts.mkdir(exist_ok=True)
    verify_lab.ROOT = PROJECT
    client = verify_lab.Client(binary, window=args.open, log_path=artifacts / 'world.log')
    try:
        client.request('initialize', {
            'protocolVersion': '2025-11-25', 'capabilities': {},
            'clientInfo': {'name': 'land-of-survivors-world', 'version': '1'},
        })
        client.request('notifications/initialized', notification=True)
        project = client.call('open_project', path=str(PROJECT))
        project = client.call('open_project_scene', path='scenes/Principal.scene.json')
        native_source = client.call('load_planet', path='world/sphere.ignis-planet')['document']
        if native_source != source:
            raise ValueError('Native planet file differs from the authored source')
        dimensions = client.call('query_planet', query={'kind': 'inspect', 'document': source})
        client.call('set_execution_mode', mode='continuous')
        client.call('load_planet_runtime', path='world/runtime.json')
        view = json.loads((PROJECT / 'world/view.json').read_text())
        if args.view == 'globe':
            view.update(distance_meters=source['diameter_meters'] * 1.5)
        elif args.view == 'ground':
            view.update(distance_meters=0, yaw_degrees=-48, tilt_degrees=8)
        client.call('set_planet_view', view=view)
        if args.view == 'globe':
            client.call('frame_planet_view')
        state = client.call('inspect_planet_view')
        if not state['all_six_faces_covered'] or not state['conforming_edges']:
            raise ValueError('The native globe does not have complete conforming coverage')
        if args.open:
            print('World open. Mouse wheel: zoom; G: globe/surface; F: surface; P: play/pause. Close the window to exit.', flush=True)
            client.process.wait()
        else:
            # Confirm the native source remains readable at poles and cube-face directions.
            samples = client.call('query_planet', query={
                'kind': 'sample', 'document': source,
                'directions': [[1, 0, 0], [-1, 0, 0], [0, 1, 0],
                               [0, -1, 0], [0, 0, 1], [0, 0, -1]],
            })['samples']
            before = client.call('inspect_planet_runtime')
            client.call('capture', path='artifacts/world.png', width=1280, height=800)
            after = client.call('inspect_planet_runtime')
            if after != before:
                raise ValueError('Inspection changed the world simulation')
            report = {
                'build_id': client.call('inspect')['build_id'],
                'project': project,
                'radius_meters': dimensions['radius_meters'],
                'diameter_meters': source['diameter_meters'],
                'circumference_meters': dimensions['circumference_meters'],
                'surface_area_square_kilometers': dimensions['surface_area_square_meters'] / 1e6,
                'samples': samples,
                'source_roundtrip': True,
                'view_preserved_simulation': True,
                'view': client.call('inspect_planet_view'),
                'native_runtime': before,
                'window_opened': False,
            }
            if not math.isclose(report['radius_meters'], 6_371_000, rel_tol=0, abs_tol=1e-6):
                raise ValueError('The world must retain its agreed Earth-size reference radius')
            (artifacts / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
            print(json.dumps({
                'verified': True, 'diameter_km': source['diameter_meters'] / 1000,
                'relief_operations': len(source['operations']), 'window': False,
            }))
    finally:
        client.close()


if __name__ == '__main__':
    main()
