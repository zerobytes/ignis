#!/usr/bin/env python3
"""Compile lawn modules and review dense native swards in an isolated MCP process."""
import argparse
import copy
import hashlib
import html
import json
import math
import random
import subprocess
import time
from pathlib import Path

from demo_groundcover import base_scene, place, publish_recipe, relative, scene_stats, setup, verify_closed
from groundcover_assets import atomic_json, source_fingerprint
from lawn_assets import DEST, ROOT, library
from verify_lab import Client

OUT = ROOT / 'artifacts/lawns'


def configure_light(c, center=(0, 0, 0), extent=2.6):
    c.call('set_light', light=dict(direction=[.45, -.82, -.35], ambient=.40, direct_intensity=1.12,
        environment=dict(sky=[.52, .65, .83], horizon=[.64, .68, .57], ground=[.075, .10, .035], axis=[0, 1, 0], intensity=.50),
        shadow=dict(center=list(center), extent=extent, resolution=2048, bias=.001)))


def sward(compiled, templates, manifest, family=None, seed=216731):
    scene = base_scene(compiled)
    scene['meshes']['instances'][0]['color'] = [.055, .050, .025]
    rng = random.Random(seed)
    records = []
    for f, center in [('esmeralda', -.99), ('japonesa', .99)]:
        if family and f != family:
            continue
        if family:
            center = 0
        entries = [e for e in manifest['entries'] if e['family'] == f]
        for z in range(7):
            for x in range(7):
                e = entries[rng.randrange(len(entries))]
                # The emerald modules meet as a carpet. Cushion modules gain
                # small offsets/height differences without opening large gaps.
                jitter = .025 if f == 'japonesa' else 0
                px = center + (x - 3) * .24 + rng.uniform(-jitter, jitter)
                pz = (z - 3) * .24 + rng.uniform(-jitter, jitter)
                y = rng.uniform(-.0015, .0015) if f == 'japonesa' else 0
                scale = rng.uniform(.94, 1.08) if f == 'japonesa' else 1
                yaw = rng.randrange(4) * math.pi / 2
                id = f'{f}_{x}_{z}'
                place(scene, templates, e, id, [px, y, pz], yaw, scale, rng.uniform(.94, 1.06))
                records.append(dict(id=id, family=f, variant=e['id'], position=[px, y - .025, pz], scale=scale, yaw=yaw))
    # Author this new static scene in material/asset runs from the beginning.
    # This does not reorder an existing scene or change renderer tie semantics.
    scene['meshes']['instances'].sort(key=lambda i: i['asset'])
    return scene, records


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    args = p.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((DEST / 'library.json').read_text())
    expected, files = library(manifest['seed'], manifest['variants'])
    assert manifest == expected
    assert all(json.loads((DEST / name).read_text()) == value for name, value in files.items())
    report = dict(components={}, captures=[])
    c = Client(args.binary, log_path=OUT / 'native.log')
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='lawns', version='1')))
        c.request('notifications/initialized', notification=True)
        schema = c.request('tools/list')['result']['tools'][0]['inputSchema']
        atomic_json(OUT / 'command-schema.json', schema)
        report['build_id'] = setup(c)
        configure_light(c)
        for entry in manifest['entries']:
            for part in entry['components']:
                recipe = copy.deepcopy(files[part['path']])
                recipe['settings']['position'] = [(entry['variant'] - (manifest['variants'] - 1) / 2) * .36, -.025, -.25 if entry['family'] == 'esmeralda' else .25]
                result = publish_recipe(c, recipe, OUT / 'publication.ignis-botany')
                report['components'][part['prefix']] = result['compiled']
        c.call('save_scene', path=relative(OUT / 'authoring.scene.json'))
        c.call('set_camera', camera=dict(position=[.9, .8, 1.1], target=[0, .01, 0], fov_degrees=45))
        c.call('capture', path=relative(OUT / 'variants.png'), width=1600, height=1100)
        compiled = json.loads((OUT / 'authoring.scene.json').read_text())
        assets = [a for a in compiled['meshes']['assets'] if a['id'] != 'gc_ground' and len(a['geometry']['triangles']) > 1]
        verify_closed(assets)
        keep = {a['id'] for a in assets}
        native_instances = [i for i in compiled['meshes']['instances'] if i['asset'] in keep]
        compiled['meshes']['assets'] = assets
        templates = {e['id']: [copy.deepcopy(i) for i in native_instances if any(i['asset'].startswith(p['prefix'] + '_') for p in e['components'])] for e in manifest['entries']}
        for entries in templates.values():
            for instance in entries:
                instance['position'] = [0, 0, 0]
        assert all(templates.values())
        atomic_json(OUT / 'templates.json', templates)
        atomic_json(OUT / 'library.scene.json', base_scene(compiled))
        atomic_json(OUT / 'library-build.json', dict(version=1, source_sha256=source_fingerprint(DEST, manifest),
            scene_sha256=hashlib.sha256((OUT / 'library.scene.json').read_bytes()).hexdigest(),
            templates_sha256=hashlib.sha256((OUT / 'templates.json').read_bytes()).hexdigest()))
        report['native_parts_closed_oriented'] = True
        report['variants'] = {e['id']: sum(len(a['geometry']['triangles']) for a in assets if a['id'] in {i['asset'] for i in templates[e['id']]}) for e in manifest['entries']}
        comparison, records = sward(compiled, templates, manifest)
        atomic_json(OUT / 'comparison.candidate.scene.json', comparison)
        atomic_json(OUT / 'placements.json', records)
        c.call('load_scene', path=relative(OUT / 'comparison.candidate.scene.json'))
        configure_light(c)
        views = dict(comparison=dict(position=[2.2, 1.7, 2.9], target=[0, .02, 0], fov_degrees=42),
            top=dict(position=[0, 3.4, 1.3], target=[0, 0, 0], fov_degrees=61),
            esmeralda_close=dict(position=[-.76, .20, .45], target=[-.99, .015, .02], fov_degrees=46),
            japonesa_close=dict(position=[1.28, .28, .48], target=[.99, .025, .02], fov_degrees=46),
            profile=dict(position=[.2, .13, 3.4], target=[0, .015, 0], fov_degrees=53))
        for name, camera in views.items():
            c.call('set_camera', camera=camera)
            if name == 'comparison':
                c.call('save_scene', path=relative(OUT / 'comparison.scene.json'))
            started = time.monotonic()
            metadata = c.call('capture', path=relative(OUT / (name + '.png')), width=1800, height=1100)
            report['captures'].append(dict(name=name, camera=camera, capture_wall_seconds=time.monotonic() - started, metadata=metadata))
            print('captured', name, flush=True)
        c.call('save_scene', path=relative(OUT / 'roundtrip.scene.json'))
        c.call('load_scene', path=relative(OUT / 'roundtrip.scene.json'))
        c.call('capture', path=relative(OUT / 'roundtrip.png'), width=1800, height=1100)
        assert (OUT / 'profile.png').read_bytes() == (OUT / 'roundtrip.png').read_bytes()
        report['save_load_png_exact'] = True
        # Still-image spatial supersampling is explicit and separate from the
        # unfiltered native captures. It does not enable runtime antialiasing.
        report['presentation_previews'] = {}
        for name in ['comparison', 'esmeralda_close', 'japonesa_close']:
            c.call('set_camera', camera=views[name])
            source = OUT / (name + '-ssaa.png')
            metadata = c.call('capture', path=relative(source), width=2048, height=1252)
            output = OUT / (name + '-preview.png')
            subprocess.run(['sips', '-z', '626', '1024', str(source), '--out', str(output)], check=True, capture_output=True)
            report['presentation_previews'][name] = dict(source=source.name, preview=output.name,
                method='native 2048x1252 raster downsampled to 1024x626 with sips', metadata=metadata)
        for family in ['esmeralda', 'japonesa']:
            scene, _ = sward(compiled, templates, manifest, family)
            atomic_json(OUT / (family + '.candidate.scene.json'), scene)
            c.call('load_scene', path=relative(OUT / (family + '.candidate.scene.json')))
            configure_light(c)
            camera = dict(position=[1.25, 1.05, 1.7], target=[0, .01, 0], fov_degrees=51)
            c.call('set_camera', camera=camera)
            c.call('save_scene', path=relative(OUT / (family + '.scene.json')))
            c.call('capture', path=relative(OUT / (family + '.png')), width=1600, height=1100)
        report['stats'] = scene_stats(comparison)
        report['source_sha256'] = source_fingerprint(DEST, manifest)
        atomic_json(OUT / 'report.json', report)
        atomic_json(OUT / 'views.json', views)
        cards = ''.join(f'<figure><a href="{n}.png"><img src="{n}.png"></a><figcaption>{html.escape(n)}</figcaption></figure>' for n in ['comparison-preview', 'variants', *views, 'esmeralda_close-preview', 'japonesa_close-preview', 'esmeralda', 'japonesa'])
        (OUT / 'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Gramas esmeralda e japonesa</title><style>body{background:#152019;color:#eee;font:16px system-ui;margin:24px}img{width:100%;max-width:1300px}figure{margin:24px 0}</style><h1>Esmeralda e japonesa</h1><p>Esmeralda à esquerda; japonesa à direita. Capturas nativas Ignis.</p>' + cards)
        print(json.dumps(dict(stats=report['stats'], variants=report['variants'], build=report['build_id'])), flush=True)
        for candidate in OUT.glob('*.candidate.scene.json'):
            candidate.unlink()
        (OUT / 'roundtrip.scene.json').unlink(missing_ok=True)
    finally:
        c.close()


if __name__ == '__main__':
    main()
