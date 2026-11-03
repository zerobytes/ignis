#!/usr/bin/env python3
"""Rebuild editable native effects and isolated MCP evidence; preserve human world/window."""
import argparse
import copy
import hashlib
import html
import json
import os
from pathlib import Path
import shutil

PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT / 'artifacts/particles'
for folder in (OUTPUT, OUTPUT / 'artifacts', OUTPUT / 'examples', OUTPUT / 'assets/particles'):
    folder.mkdir(parents=True, exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client


def verify(binary):
    client = Client(binary, log_path=OUTPUT / 'artifacts/runtime.log')
    checks = []
    captures = {}

    def op(kind, expected_error=False, **kw):
        return client.call('particles', operation={'kind': kind, **kw}, expected_error=expected_error)

    def capture(name):
        data = client.call('capture', path=f'artifacts/{name}.png', width=1280, height=800)
        captures[name] = data
        return (OUTPUT / f'artifacts/{name}.png').read_bytes()

    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                      'clientInfo': {'name': 'particles-gallery', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        tools = client.request('tools/list')['result']['tools']
        assert '"particles"' in json.dumps(tools).lower()
        checks.append('MCP schema exposes typed particle operations')
        client.call('generate', recipe={'seed': 7, 'width': 24, 'depth': 20, 'obstacles': 0})
        entities = client.call('inspect')['scene']['entities']
        for entity in entities:
            entity['visible'] = False
        client.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        client.call('set_camera', camera={'position': [0, 3, 13], 'target': [0, 2, 0], 'fov_degrees': 44})
        client.call('set_light', light={'direction': [-0.4, -1, -0.25], 'ambient': 0.3, 'direct_intensity': 0.7})
        op('preset', preset='sparks', id='sparks')
        op('preset', preset='smoke', id='smoke')
        document = op('inspect')['document']
        document['instances'][0]['position'] = {'cell': ['-1', '0', '0'], 'offset': [1021.5, 0, 0]}
        document['instances'][1]['position']['offset'] = [2.5, 0, 0]
        # Keep source data generic: one floor asset, no mesh/actor per particle.
        client.call('edit_mesh', edit={'kind': 'put_asset', 'asset': {
            'id': 'floor', 'shading': 'flat', 'material': {}, 'geometry': {
                'positions': [[-10, 0, -6], [10, 0, -6], [10, 0, 6], [-10, 0, 6]],
                'triangles': [[0, 2, 1], [0, 3, 2]]}}})
        client.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {
            'id': 'floor', 'asset': 'floor', 'position': [0, -0.05, 0], 'rotation': [0, 0, 0, 1],
            'scale': 1, 'color': [0.08, 0.1, 0.14]}})
        op('replace', document=document)
        for name in ('sparks', 'smoke'):
            op('save_source', id=name, path=f'assets/particles/{name}.ignis-vfx')
        original = copy.deepcopy(document)
        bad = copy.deepcopy(document)
        bad['instances'][0]['source'] = 'missing'
        op('replace', document=bad, expected_error=True)
        assert op('inspect')['document'] == original
        checks.append('Invalid source reference leaves document unchanged')
        frames = {}
        for tick in (0, 45, 90, 150, 240):
            op('preview', tick=tick)
            state = op('inspect')
            frames[tick] = state['presentation']['stats']
            capture(f'tick-{tick}')
        op('preview', tick=150)
        a = capture('warm')
        b = capture('same-time')
        assert a == b
        checks.append('Repeated native same-time capture PNG exact')
        for tick in (3000, 0, 150):
            op('preview', tick=tick)
        assert capture('seek-return') == a
        checks.append('Native seek-return PNG exact')
        op('save_source', id='sparks', path='../escape.ignis-vfx', expected_error=True)
        op('load_source', path='assets/particles/sparks.ignis-vfx')
        assert op('inspect')['document'] == original
        checks.append('Source round trip preserves placements; path traversal rejected')
        op('preview', tick=None)
        client.call('save_scene', path='examples/particles.scene.json')
        client.call('step', ticks=150)
        expected = op('inspect')['presentation']['sample']
        saved_image = capture('world-150')
        client.call('save_replay', path='artifacts/particles.replay.json')
        client.call('save_portable_save', path='artifacts/particles.ignis-portable')
        client.call('load_portable_save', path='artifacts/particles.ignis-portable')
        assert op('inspect')['presentation']['sample'] == expected
        assert capture('restored') == saved_image
        checks.append('Portable restore preserves world clock and native PNG')
        client.call('load_scene', path='examples/particles.scene.json')
        client.call('step', ticks=150)
        assert op('inspect')['presentation']['sample'] == expected
        checks.append('Scene round trip reconstructs same particle samples')
        op('open', open=True)
        client.call('capture_editor', path='artifacts/editor.png', width=1440, height=960)
        checks.append('Native editor panel renders through shared API; human input acceptance pending')
        (PROJECT / 'assets/particles').mkdir(parents=True, exist_ok=True)
        for name in ('sparks', 'smoke'):
            shutil.copyfile(OUTPUT / f'assets/particles/{name}.ignis-vfx', PROJECT / f'assets/particles/{name}.ignis-vfx')
        example = json.loads((OUTPUT / 'examples/particles.scene.json').read_text())
        # This example needs particle Scene113; later unrelated scene features are absent.
        example['version'] = 113
        (PROJECT / 'examples/particles.scene.json').write_text(json.dumps(example, indent=2) + '\n')
        report = {'checks': checks, 'frames': frames, 'captures': captures, 'source_sha256': {
            n: hashlib.sha256((PROJECT / f'assets/particles/{n}.ignis-vfx').read_bytes()).hexdigest()
            for n in ('sparks', 'smoke')}}
        (OUTPUT / 'verification.json').write_text(json.dumps(report, indent=2))
        cards = ''.join(f'<figure><img src="artifacts/tick-{t}.png"><figcaption>Tick {t} · {frames[t]["alive"]} sprites</figcaption></figure>' for t in frames)
        (OUTPUT / 'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Ignis particles</title><style>body{background:#101723;color:#eef;font:16px system-ui;margin:32px}img{max-width:100%}figure{margin:28px 0}li{margin:8px}</style><h1>Native particle foundation</h1><p>Seeded sparks and sprite smoke. Artistic acceptance pending.</p><ul>' + ''.join(f'<li>{html.escape(c)}</li>' for c in checks) + '</ul>' + cards + '<h2>Editor</h2><img src="artifacts/editor.png">')
        print(json.dumps({'checks': len(checks), 'frames': frames, 'review': str(OUTPUT / 'review.html')}))
    finally:
        client.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=PROJECT / 'target/release/ignis')
    args = parser.parse_args()
    verify(args.binary.resolve())
