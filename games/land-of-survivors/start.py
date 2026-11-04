#!/usr/bin/env python3
"""Inspect or open the empty native front-end scene with the current Ignis executable."""
import argparse
import json
from pathlib import Path
import sys

PROJECT = Path(__file__).resolve().parent
ENGINE = PROJECT.parents[1]
sys.path.insert(0, str(ENGINE / 'scripts'))
import verify_lab

ENTRY = 'scenes/Inicio.scene.json'


def assert_stage(scene):
    if len(scene['entities']) != 1:
        raise ValueError('The front-end stage must contain only its ground')
    floor = scene['entities'][0]
    if floor['id'] != 'front_end_ground' or floor['body'] != 'fixed':
        raise ValueError('Unexpected front-end stage content')
    if floor['transform'] != {'position': [0., -.5, 0.], 'size': [200., 1., 200.]}:
        raise ValueError('The stage must be200x200m, centered at0 with its top atY=0')
    if scene.get('characters') or scene.get('botany') or scene.get('meshes', {}).get('instances'):
        raise ValueError('The front-end stage must remain empty')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    display = parser.add_mutually_exclusive_group()
    display.add_argument('--open', action='store_true', help='Open the native scene window')
    display.add_argument('--editor', action='store_true', help='Open this project in the native editor')
    parser.add_argument('--binary', type=Path, default=ENGINE / 'target/release/ignis')
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error('Build Ignis first; this scene uses the current executable')
    out = PROJECT / 'artifacts/front-end'
    out.mkdir(parents=True, exist_ok=True)
    verify_lab.ROOT = PROJECT
    client = verify_lab.Client(binary, window=args.open or args.editor,
                               startup_args=('--scene', ENTRY, '--editor') if args.editor else ('--scene', ENTRY),
                               log_path=out / 'runtime.log')
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                     'clientInfo': {'name': 'los-front-end', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        project = client.call('open_project', path=str(PROJECT))
        if project['project']['entry_scene'] != ENTRY or project['scene_path'] != ENTRY:
            raise ValueError('The front-end stage must be the project entry')
        snapshot = client.call('inspect')
        scene = snapshot['scene']
        if args.open or args.editor:
            print('Front-end stage open:200x200m, empty. Close this window to exit.', flush=True)
            client.process.wait()
            return
        assert_stage(scene)
        client.call('capture', path='artifacts/front-end/start.png', width=1280, height=800)
        camera = scene['camera']
        client.call('set_camera', camera={'position': [180, 150, 180], 'target': [0, 0, 0], 'fov_degrees': 48})
        client.call('capture', path='artifacts/front-end/overview.png', width=1280, height=800)
        client.call('set_camera', camera=camera)
        restored = client.call('inspect')['scene']
        if restored != scene:
            raise ValueError('Capture changed the authored stage')
        client.call('save_scene', path=ENTRY)
        client.call('open_project_scene', path='scenes/Principal.scene.json')
        planet_scene = client.call('inspect')['scene']
        if not any(e['id'] == 'player' for e in planet_scene['entities']):
            raise ValueError('The retained planet scene lost its technical observer')
        client.call('open_project_scene', path=ENTRY)
        if client.call('inspect')['scene'] != scene:
            raise ValueError('The stage changed after native save/load')
        report = {'build_id': snapshot['build_id'], 'entry_scene': ENTRY,
                  'size_meters': [200, 200], 'area_square_meters': 40000,
                  'bounds_xz_meters': [-100, 100], 'ground_top_y_meters': 0,
                  'entities': 1, 'players': 0, 'npcs': 0,
                  'native_roundtrip': True, 'original_camera_preserved': True,
                  'planet_scene_preserved': True, 'window_opened': False,
                  'menus_implemented': False, 'character_selector_implemented': False}
        (out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps({'verified': True, 'size_meters': [200, 200], 'entry_scene': ENTRY}))
    finally:
        client.close()


if __name__ == '__main__':
    main()
