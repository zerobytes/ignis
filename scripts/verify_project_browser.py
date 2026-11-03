#!/usr/bin/env python3
"""Exercise the project browser and bounded library through the real MCP process."""
import argparse
import json
import os
from pathlib import Path
import tempfile
import verify_lab
from verify_lab import ROOT
from verify_projects import start


def verify(binary, window):
    if window:
        raise ValueError('Project browser regression is headless')
    original = verify_lab.ROOT
    c = start(binary)
    second = None
    checks = []
    try:
        with tempfile.TemporaryDirectory(dir=ROOT/'artifacts', prefix='browser-fixture-') as temp:
            root = Path(temp)
            library = root/'preferences/projects.json'
            schema = json.dumps(c.request('tools/list'))
            assert all(name in schema for name in ['browse_project_directory', 'create_project_folder',
                'inspect_project_library', 'use_project_library', 'register_project', 'pin_project',
                'forget_project', 'set_project_favorite', 'discover_projects'])
            before = c.call('inspect')
            parent = c.call('create_project_folder', parent=str(root), name='Meus jogos')['created_folder']
            c.call('create_project_folder', parent=str(root), name='Meus jogos', expected_error=True)
            c.call('create_project_folder', parent=str(root), name='../escape', expected_error=True)
            page = c.call('browse_project_directory', query=dict(path=str(root)))['directory']
            assert any(e['path'] == parent and e['kind'] == 'folder' for e in page['entries'])
            assert c.call('inspect') == before
            checks.append('All nine browser/library operations exposed by MCP; exclusive folder creation, traversal refusal and pure browse')

            c.call('use_project_library', path=str(library))
            project = Path(parent)/'Meu jogo'
            c.call('create_project', path=str(project), name='Meu jogo')
            verify_lab.ROOT = project
            manifest = (project/'ignis.project.json').read_bytes()
            c.call('register_project', path=str(project))
            alias = root/'Alias do jogo'
            os.symlink(project, alias, target_is_directory=True)
            c.call('pin_project', path=str(project), pinned=True)
            c.call('pin_project', path=str(alias), pinned=False)
            assert not c.call('inspect_project_library')['library']['projects'][0]['pinned']
            c.call('pin_project', path=str(project), pinned=True)
            c.call('set_project_favorite', path=parent, favorite=True)
            state = c.call('inspect_project_library')
            assert len(state['library']['projects']) == 1
            assert state['library']['projects'][0]['pinned']
            assert state['library']['favorites'] == [parent]
            page = c.call('browse_project_directory', query=dict(path=parent))['directory']
            assert any(e['path'] == str(project) and e['kind'] == 'project' for e in page['entries'])
            checks.append('Created project automatically enters bounded persistent library; pin/favorite and filesystem recognition agree')

            bad = root/'invalid-library.json'
            bad.write_text('{"version":99,"projects":[],"favorites":[]}')
            saved = library.read_bytes()
            c.call('use_project_library', path=str(bad), expected_error=True)
            assert c.call('inspect_project_library') == state and library.read_bytes() == saved
            (project/'artifacts').mkdir(exist_ok=True)
            second = start(binary)
            second.call('use_project_library', path=str(library))
            assert second.call('inspect_project_library')['library'] == state['library']
            second.close(); second = None
            checks.append('Invalid library load is atomic; independent process restores exact pins, recents and favorites')

            before = c.call('inspect')
            c.call('forget_project', path=str(alias))
            assert not c.call('inspect_project_library')['library']['projects']
            assert (project/'ignis.project.json').read_bytes() == manifest
            discovered = c.call('discover_projects', paths=[parent])
            assert discovered['found'] == 1 and not discovered['truncated']
            assert c.call('inspect') == before
            c.call('capture_editor', path='artifacts/browser-editor.png', width=1000, height=800)
            c.call('save_replay', path='artifacts/browser.replay.json')
            assert c.call('run_replay', path='artifacts/browser.replay.json')['matched']
            checks.append('Forget removes metadata only; shallow discovery restores project without changing scene/history; editor capture and replay work')
            report = dict(passed=True, checks=checks, native_window=False, build_id=c.call('inspect')['build_id'])
            (ROOT/'artifacts/project-browser-verification.json').write_text(json.dumps(report, indent=2)+'\n')
            print(json.dumps({**report, 'checks':len(checks)}))
    finally:
        if second is not None:
            second.close()
        c.close()
        verify_lab.ROOT = original


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path(__file__).resolve().parents[1]/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args = parser.parse_args()
    verify(args.binary, args.window)
