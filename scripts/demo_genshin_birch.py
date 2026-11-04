#!/usr/bin/env python3
"""Compile and review three birches through an isolated native MCP process."""
import argparse
import copy
import hashlib
import html
import json
import time
from pathlib import Path

from demo_groundcover import fit, relative, setup, verify_closed
from genshin_birch import DEST
from groundcover_assets import ROOT, atomic_json
from verify_lab import Client

OUT = ROOT / 'artifacts/genshin-birch'


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    p.add_argument('--structure-only', action='store_true')
    p.add_argument('--only', choices=['birch_0', 'birch_1', 'birch_2'])
    args = p.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((DEST / 'library.json').read_text())
    entries = [e for e in manifest['entries'] if not args.only or e['id'] == args.only]
    c = Client(args.binary, log_path=OUT / 'runtime.log')
    report = dict(variants={}, structure_only=args.structure_only)
    def save(path):
        c.call('save_scene', path=relative(path))
        return json.loads(path.read_text())
    def material(asset, recipe):
        c.call('material', operation=dict(kind='generate', asset=asset, recipe=recipe,
               execution=dict(fixed=dict(resolution=256))))
        c.call('material', operation=dict(kind='save', asset=asset, path=relative(DEST / (asset + '.ignis-material'))))
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='birch-review', version='1')))
        c.request('notifications/initialized', notification=True)
        schema = c.request('tools/list')['result']['tools'][0]['inputSchema']
        atomic_json(OUT / 'schema.json', schema)
        assert all(word in json.dumps(schema) for word in ['botany', 'material', 'blade_samples'])
        report['build_id'] = setup(c)
        c.call('set_light', light=dict(direction=[.5, -.82, -.35], ambient=.24, direct_intensity=1.15,
            environment=dict(sky=[.46, .61, .78], horizon=[.59, .65, .62], ground=[.16, .19, .08], axis=[0, 1, 0], intensity=.35),
            shadow=dict(center=[0, 4, 0], extent=12, resolution=2048, bias=.0005)))
        ground = dict(id='birch_ground', geometry=dict(positions=[[-80, -.025, -80], [80, -.025, -80], [-80, -.025, 80], [80, -.025, 80]], triangles=[[0, 2, 1], [1, 2, 3]]))
        instance = dict(id='birch_ground', asset='birch_ground', position=[0, 0, 0], rotation=[0, 0, 0, 1], scale=1, color=[.23, .29, .12])
        c.call('edit_scene', edits=[dict(op='mesh', edit=dict(kind='put_asset', asset=ground)), dict(op='mesh', edit=dict(kind='put_instance', instance=instance))])
        c.call('save_scene', path=relative(OUT / 'empty.scene.json'))
        appearance = json.loads((DEST / 'materials.recipe.json').read_text())
        for e in entries:
            c.call('load_scene', path=relative(OUT / 'empty.scene.json'))
            source = json.loads((DEST / e['path']).read_text())
            source_path = DEST / e['path']
            if args.structure_only:
                source['settings'].update(bark_color=[.34, .34, .34], leaf_color=[.34, .34, .34])
                source_path = OUT / 'review.ignis-botany'
                atomic_json(source_path, source)
            start = time.monotonic()
            compiled = c.call('botany', operation=dict(kind='load', path=relative(source_path), publish=True))['compiled']
            report['variants'][e['id']] = dict(compile_seconds=time.monotonic() - start, compiled=compiled)
            lo, hi = compiled['bounds_min'], compiled['bounds_max']
            # Structural volume gate precedes appearance: views span depth, not a billboard.
            if args.structure_only:
                for name, direction in [('front', (.08, .12, 1)), ('side', (1, .15, -.06)), ('back', (-.1, .12, -1)), ('top', (.01, 1, .02))]:
                    fit(c, lo, hi, OUT / f"{e['id']}-gray-{name}.png", direction=direction, width=1100, height=1100)
                source['settings']['leaf_density'] = 0.
                atomic_json(OUT / 'review.ignis-botany', source)
                c.call('botany', operation=dict(kind='load', path=relative(OUT / 'review.ignis-botany'), publish=True))
                fit(c, lo, hi, OUT / f"{e['id']}-structure.png", direction=(.08, .12, 1), width=1100, height=1100)
                continue
            for part, recipe in appearance.items():
                material(e['id'] + '_' + part, recipe)
            # Source settings may be edited later; native preservation keeps generated materials.
            before = save(OUT / f"{e['id']}.scene.json")
            owned = [a for a in before['meshes']['assets'] if a['id'].startswith(e['id'] + '_')]
            verify_closed(owned)
            report['variants'][e['id']]['closed_oriented'] = True
            report['variants'][e['id']]['parts'] = {a['id']: dict(vertices=len(a['geometry']['positions']), triangles=len(a['geometry']['triangles'])) for a in owned}
            for name, direction in [('front', (.08, .12, 1)), ('side', (1, .15, -.06)), ('back', (-.1, .12, -1))]:
                fit(c, lo, hi, OUT / f"{e['id']}-{name}.png", direction=direction, width=1400, height=1400)
            # Fixed full-tree camera for persistence comparison.
            fit(c, lo, hi, OUT / f"{e['id']}.png", direction=(.08, .12, 1), width=1400, height=1400)
            save(OUT / f"{e['id']}.scene.json")
            c.call('load_scene', path=relative(OUT / f"{e['id']}.scene.json"))
            c.call('capture', path=relative(OUT / 'roundtrip.png'), width=1400, height=1400)
            assert (OUT / f"{e['id']}.png").read_bytes() == (OUT / 'roundtrip.png').read_bytes()
            report['variants'][e['id']]['save_load_pixels_equal'] = True
            if e['id'] == 'birch_0':
                c.call('set_camera', camera=dict(position=[1.5, 1.7, 3.0], target=[0., 1.7, 0.], fov_degrees=34))
                c.call('capture', path=relative(OUT / 'bark.png'), width=1200, height=1100)
                # A whole leafy lobe, with support branches and gaps retained in context.
                c.call('set_camera', camera=dict(position=[2.4, 4.0, 4.5], target=[.9, 3.9, 0.], fov_degrees=35))
                c.call('capture', path=relative(OUT / 'foliage.png'), width=1200, height=1100)
        if not args.structure_only:
            c.call('load_scene', path=relative(OUT / 'empty.scene.json'))
            gallery_bounds = [[], []]
            for index, e in enumerate(entries):
                source = json.loads((DEST / e['path']).read_text())
                x = (index - (len(entries) - 1) / 2) * 5.8
                source['settings']['position'] = [x, 0., 0.]
                atomic_json(OUT / 'review.ignis-botany', source)
                result = c.call('botany', operation=dict(kind='load', path=relative(OUT / 'review.ignis-botany'), publish=True))['compiled']
                gallery_bounds[0].append(result['bounds_min'])
                gallery_bounds[1].append(result['bounds_max'])
                for part in appearance:
                    c.call('material', operation=dict(kind='load', asset=e['id'] + '_' + part, path=relative(DEST / (e['id'] + '_' + part + '.ignis-material'))))
            lo = [min(v[k] for v in gallery_bounds[0]) for k in range(3)]
            hi = [max(v[k] for v in gallery_bounds[1]) for k in range(3)]
            fit(c, lo, hi, OUT / 'gallery.png', direction=(.02, .08, 1), width=2048, height=1250)
            save(OUT / 'gallery.scene.json')
            c.call('load_scene', path=relative(OUT / 'gallery.scene.json'))
            c.call('capture', path=relative(OUT / 'gallery-roundtrip.png'), width=2048, height=1250)
            assert (OUT / 'gallery.png').read_bytes() == (OUT / 'gallery-roundtrip.png').read_bytes()
            report['gallery_roundtrip_pixels_equal'] = True
        report['source_sha256'] = {e['path']: hashlib.sha256((DEST / e['path']).read_bytes()).hexdigest() for e in entries}
        atomic_json(OUT / ('structure-report.json' if args.structure_only else 'verification.json'), report)
        (OUT / 'review.ignis-botany').unlink(missing_ok=True)
        if not args.structure_only:
            items = [('gallery', 'Native variants')] + [(e['id'], e['name']) for e in entries]
            if any(e['id'] == 'birch_0' for e in entries):
                items += [('bark', 'Procedural bark'), ('foliage', 'Native leaves')]
            (OUT / 'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Birch study</title><style>body{background:#20242a;color:#eee;font:16px system-ui;max-width:1400px;margin:auto}img{max-width:100%}h2{margin-top:2em}</style><h1>Birch · native procedural study</h1><p>Reference-guided original assets; no extracted game meshes/textures. Artistic acceptance pending.</p>' + ''.join(f'<h2>{html.escape(label)}</h2><img src="{name}.png">' for name, label in items))
        print(json.dumps(report, indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    main()
