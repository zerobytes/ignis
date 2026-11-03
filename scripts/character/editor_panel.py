#!/usr/bin/env python3
"""Real-source integration check; captures are optional and require a free GPU slot."""
import argparse
import json
from pathlib import Path
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from verify_lab import Client


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--capture', action='store_true')
    args = parser.parse_args()
    output = ROOT / 'artifacts/character-panel'
    output.mkdir(parents=True, exist_ok=True)
    client = Client(ROOT / 'target/release/ignis', log_path=output / 'runtime.log')
    call = client.call
    report = {'checks': []}
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                      'clientInfo': {'name': 'character-editor', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        call('load_scene', path='artifacts/character-base/female.scene.json')
        publication = json.loads((ROOT / 'artifacts/character-base/publication.json').read_text())
        if not call('inspect')['scene'].get('characters'):
            call('connect_character_source', path='artifacts/character-base/female.ignis-character',
                 configuration=publication['configuration'], placement=publication['placement'])
        call('open_character_panel', open=True, prefix='female')
        initial = call('inspect_character_editor')
        assert initial['ready'] and len(initial['parameters']) == 21, initial
        report['checks'].append('21 native parameter descriptors loaded from the real female source')
        binding = initial['binding']
        call('save_character_configuration', prefix='female',
             path='artifacts/character-panel/female.ignis-character-config')
        report['slider_seconds'] = []
        for value in [.61, .68]:
            started = time.perf_counter()
            call('set_character_parameters', prefix='female',
                 configuration={'parameters': {'stature': value}}, gesture='one-drag')
            report['slider_seconds'].append(time.perf_counter() - started)
        edited = call('inspect_character_editor')['binding']
        assert abs(edited['recipe']['configuration']['parameters']['stature'] - .68) < 1e-6
        assert edited['bake_sha256'] != binding['bake_sha256']
        call('undo')
        assert call('inspect_character_editor')['binding'] == binding
        call('redo')
        assert call('inspect_character_editor')['binding'] == edited
        report['checks'].append('two slider updates coalesce into one undo/redo with configuration and all parts')
        call('set_character_parameters', prefix='female', configuration={'parameters': {'adiposity': 2}}, expected_error=True)
        assert call('inspect_character_editor')['binding'] == edited
        call('load_character_configuration', path='artifacts/character-panel/female.ignis-character-config')
        assert call('inspect_character_editor')['binding'] == binding
        report['checks'].append('invalid parameters are atomic; exporting/importing restores the complete configuration')
        # Update the single current editable demo, preserving its scene and geometry.
        call('save_scene', path='artifacts/character-base/female.scene.json')
        call('load_scene', path='artifacts/character-base/female.scene.json')
        assert call('inspect_character_editor')['binding'] == binding
        call('save_portable_save', path='artifacts/character-panel/female.ignis-portable')
        call('load_portable_save', path='artifacts/character-panel/female.ignis-portable', mode='recorded')
        assert call('inspect_character_editor')['binding'] == binding
        call('save_replay', path='artifacts/character-panel/female.replay.json')
        assert call('run_replay', path='artifacts/character-panel/female.replay.json')['matched']
        call('open_character_panel', open=True, prefix='female')
        report['checks'].append('scene, portable-save and replay roundtrips preserve recipe and bake')
        if args.capture:
            report['capture'] = call('capture_editor', path='artifacts/character-panel/editor.png', width=1600, height=1000)
        report['editor'] = call('inspect_character_editor')
        report['build'] = call('inspect').get('build_id')
        (output / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps({'checks': report['checks'], 'ready': report['editor']['ready']}))
    finally:
        client.close()


if __name__ == '__main__':
    main()
