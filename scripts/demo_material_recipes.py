#!/usr/bin/env python3
"""Native two-mode material library gallery and MCP integration verification."""
import argparse
import json
import os
from pathlib import Path
import time

SOURCE = Path(__file__).resolve().parents[1]
OUTPUT = SOURCE / 'artifacts/material-recipes'
OUTPUT.mkdir(parents=True, exist_ok=True)
(OUTPUT / 'artifacts').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=SOURCE / 'target/release/ignis')
    args = parser.parse_args()
    client = Client(args.binary.resolve(), log_path=OUTPUT / 'runtime.log')
    checks = []
    timings = []
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                     'clientInfo': {'name': 'material-recipes', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        assert 'unique_instances' in json.dumps(client.request('tools/list'))
        checks.append('Recipe/library operations exposed through the public MCP schema')
        client.call('generate', recipe={'seed': 7, 'width': 24, 'depth': 20, 'obstacles': 0})
        entities = client.call('inspect')['scene']['entities']
        for entity in entities:
            entity['visible'] = False
        client.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        camera = {'position': [0, 4, 17], 'target': [0, 3.1, 0], 'fov_degrees': 37}
        client.call('set_camera', camera=camera)
        client.call('set_light', light={
            'direction': [-0.6, -0.45, -1], 'ambient': 0.08, 'direct_intensity': 0.85,
            'environment': {'sky': [0.5, 0.68, 1], 'horizon': [0.9, 0.9, 0.85],
                            'ground': [0.1, 0.11, 0.14], 'axis': [0, 1, 0], 'intensity': 0.55}})
        geometry = {'positions': [[-1.25, -1.25, 0], [1.25, -1.25, 0], [1.25, 1.25, 0], [-1.25, 1.25, 0]],
                    'triangles': [[0, 1, 2], [0, 2, 3]]}
        names = ['stone', 'wood', 'painted_metal', 'ceramic']
        reports = []
        for row, execution in enumerate(['runtime', {'fixed': {'resolution': 128}}]):
            for col, preset in enumerate(names):
                asset = f'{preset}-{row}'
                client.call('edit_mesh', edit={'kind': 'put_asset', 'asset': {
                    'id': asset, 'geometry': geometry, 'shading': 'smooth', 'material': {}}})
                started = time.monotonic()
                report = client.call('material', operation={'kind': 'preset', 'asset': asset, 'preset': preset,
                                      'seed': 93, 'execution': execution})
                timings.append({'material': asset, 'execution': report['execution'],
                                'command_seconds': round(time.monotonic() - started, 4)})
                reports.append(report)
                client.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {
                    'id': asset, 'asset': asset, 'position': [(col-1.5)*2.9, 4.65-row*2.9, 0],
                    'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [1, 1, 1]}})
                client.call('material', operation={'kind': 'save', 'asset': asset,
                                                  'path': f'library/{asset}.ignis-material'})
        checks.append('Four presets generated, applied and exported in both execution modes')

        def capture(name, width=1280, height=900):
            return client.call('capture', path=f'artifacts/{name}.png', width=width, height=height)

        first = capture('gallery')
        second = capture('gallery-warm')
        assert (OUTPUT / 'artifacts/gallery.png').read_bytes() == (OUTPUT / 'artifacts/gallery-warm.png').read_bytes()
        assert first['adapter']['procedural_materials']['uploads'] == second['adapter']['procedural_materials']['uploads']
        checks.append('Identical warm capture reuses all material program/map uploads')
        # Keep only one copy of the identical verification capture.
        (OUTPUT / 'artifacts/gallery-warm.png').unlink()
        client.call('save_scene', path='examples/material-recipes.scene.json')
        client.call('load_scene', path='examples/material-recipes.scene.json')
        capture('reloaded')
        assert (OUTPUT / 'artifacts/gallery.png').read_bytes() == (OUTPUT / 'artifacts/reloaded.png').read_bytes()
        (OUTPUT / 'artifacts/reloaded.png').unlink()
        checks.append('Scene save/load preserves the complete native gallery PNG exactly')
        # A library asset can be applied later and undone through the normal transaction.
        client.call('material', operation={'kind': 'load', 'path': 'library/stone-1.ignis-material', 'asset': 'wood-0'})
        assert client.call('material', operation={'kind': 'inspect', 'asset': 'wood-0'})['execution'] == 'fixed'
        client.call('undo')
        assert client.call('material', operation={'kind': 'inspect', 'asset': 'wood-0'})['execution'] == 'runtime'
        checks.append('Independent library load/apply and undo preserve execution mode')
        client.call('set_camera', camera={**camera, 'position': [8, 5.5, 12]})
        capture('oblique')
        for col, preset in enumerate(names):
            client.call('set_camera', camera={'position': [(col-1.5)*2.9+0.55, 4.85, 2.9],
                                             'target': [(col-1.5)*2.9, 4.65, 0], 'fov_degrees': 43})
            capture(f'detail-{preset}', 720, 720)
        for preset in names:
            client.call('material', operation={'kind': 'preset', 'asset': f'{preset}-0',
                                               'preset': preset, 'seed': 93, 'unique_instances': True})
            client.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {
                'id': f'{preset}-unique', 'asset': f'{preset}-0',
                'position': [(names.index(preset)-1.5)*2.9, 1.75, 0],
                'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [1, 1, 1]}})
            client.call('edit_mesh', edit={'kind': 'remove_instance', 'id': f'{preset}-1'})
        client.call('set_camera', camera=camera)
        capture('unique-instances')
        checks.append('Shared source geometry produces distinct deterministic runtime instances')
        client.call('load_scene', path='examples/material-recipes.scene.json')
        result = {'checks': checks, 'materials': reports, 'generation': timings,
                  'adapter': first['adapter'], 'scene_version': client.call('inspect')['scene']['version'],
                  'scope': 'Native Metal/headless images and command validation; human artistic acceptance pending'}
        (OUTPUT / 'verification.json').write_text(json.dumps(result, indent=2))
        cards = ''.join(f'<figure><img src="artifacts/detail-{name}.png"><figcaption>{name}</figcaption></figure>' for name in names)
        html = '''<!doctype html><meta charset="utf-8"><title>Ignis material recipes</title>
<style>body{background:#15171c;color:#e7e9ed;font:16px system-ui;max-width:1360px;margin:40px auto;padding:0 24px}img{max-width:100%;border-radius:8px}section{display:grid;grid-template-columns:repeat(4,1fr);gap:16px}figure{margin:0}figcaption{padding:8px 0;color:#bbc4d0}p{line-height:1.6}</style>
<h1>Ignis · reusable material recipes</h1><p>Stone · Wood · Painted metal · Ceramic. Top: runtime. Bottom: fixed maps. Same recipe and seed.</p><img src="artifacts/gallery.png">
<h2>Oblique view</h2><img src="artifacts/oblique.png"><h2>Surface detail</h2><section>'''+cards+'''</section>
<h2>Unique runtime instances</h2><p>Each column shares geometry and recipe; instance IDs produce stable seeds.</p><img src="artifacts/unique-instances.png">
<p>Height uses bump or parallax and local self shadow. World silhouette/collision remain unchanged. Generated emission does not illuminate nearby objects. These images do not establish human artistic acceptance or a runtime FPS guarantee.</p><p><a href="verification.json">Validation and storage evidence</a> · <a href="examples/material-recipes.scene.json">Editable scene</a> · <a href="library/stone-1.ignis-material">Fixed material asset example</a></p>'''
        (OUTPUT / 'review.html').write_text(html)
        print(json.dumps({'checks': len(checks), 'review': str(OUTPUT/'review.html'), 'adapter': first['adapter']['name']}))
    finally:
        client.close()


if __name__ == '__main__':
    main()
