#!/usr/bin/env python3
"""Create a real project, author scenes/objects, reopen it and render the editor."""
import argparse
import json
from pathlib import Path
import shutil
import tempfile
import verify_lab
from verify_lab import Client, ROOT


def start(binary):
    c = Client(binary)
    c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'project-verifier', 'version': '1'}})
    c.request('notifications/initialized', notification=True)
    return c


def verify(binary, window):
    if window:
        raise ValueError('Project verification is headless; native interaction has a separate gate')
    original = verify_lab.ROOT
    checks = []
    c = start(binary)
    try:
        with tempfile.TemporaryDirectory(dir=ROOT/'artifacts', prefix='project-fixture-') as temp:
            folder = Path(temp)/'Meu jogo'
            schema = json.dumps(c.request('tools/list'))
            assert all(op in schema for op in ['create_project', 'open_project', 'inspect_project', 'new_project_scene', 'create_scene_object'])
            status = c.call('create_project', path=str(folder), name='Meu jogo')
            verify_lab.ROOT = folder
            assert status['project']['version'] == 1 and not status['modified']
            assert (folder/'ignis.project.json').is_file()
            first = status['scene_path']
            scene = c.call('inspect')['scene']
            assert {e['id'] for e in scene['entities']} == {'floor', 'player'}
            checks.append('MCP schema and project folder with portable manifest, initial floor/player and registered scene')
            for kind in ['visual', 'fixed', 'dynamic']:
                c.call('create_scene_object', kind=kind)
            selection = c.call('inspect')['selection']
            c.call('duplicate_scene_object', id=selection)
            count = len(c.call('inspect')['scene']['entities'])
            c.call('undo')
            assert len(c.call('inspect')['scene']['entities']) == count-1
            c.call('redo')
            assert len(c.call('inspect')['scene']['entities']) == count
            assert c.call('inspect_project')['modified']
            before = c.call('inspect')
            for op, args in [('new_project_scene', {'name': 'Floresta'}), ('open_project', {'path': str(folder)}), ('open_project_scene', {'path': first})]:
                c.call(op, expected_error=True, **args)
                assert c.call('inspect') == before
            checks.append('Visual/fixed/dynamic creation, selection, duplication and undo/redo; dirty switches rejected atomically')
            c.call('save_scene', path=first)
            assert not c.call('inspect_project')['modified']
            authored = c.call('inspect')['scene']
            c.call('new_project_scene', name='Floresta')
            second = c.call('inspect_project')['scene_path']
            assert second != first and len(c.call('inspect_project')['project']['scenes']) == 2
            c.call('new_project_scene', name='Floresta', expected_error=True)
            c.call('new_project_scene', name='../escape', expected_error=True)
            c.call('set_project_entry', path=second)
            c.call('open_project_scene', path=first)
            assert c.call('inspect')['scene'] == authored
            c.call('create_project', path=str(folder), name='Overwrite', expected_error=True)
            c.call('save_scene', path='../escape.json', expected_error=True)
            c.call('edit_editor', action={'action': 'activate', 'area': 3, 'panel': 'project'})
            c.call('capture_editor', path='artifacts/project-editor.png', width=1440, height=900)
            shutil.copyfile(folder/'artifacts/project-editor.png', ROOT/'artifacts/project-editor.png')
            c.call('step', ticks=30)
            c.call('save_replay', path='artifacts/project.replay.json')
            assert c.call('run_replay', path='artifacts/project.replay.json')['matched']
            checks.append('Multiple scenes, entry selection, collisions/traversal errors, exact persistence, real editor capture and gameplay replay')
            c.close()
            c = start(binary)
            c.call('open_project', path=str(folder))
            assert c.call('inspect')['scene']['name'] == 'Floresta'
            c.call('open_project_scene', path=first)
            assert c.call('inspect')['scene'] == authored
            checks.append('Independent process reopens project entry and authored objects without repository example paths')
            report = {'passed': True, 'checks': checks, 'native_window': False, 'build_id': c.call('inspect')['build_id']}
            (ROOT/'artifacts/projects-verification.json').write_text(json.dumps(report, indent=2)+'\n')
            print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()
        verify_lab.ROOT = original


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args()
    verify(args.binary.resolve(), args.window)
