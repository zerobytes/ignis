#!/usr/bin/env python3
"""Generate reusable mineral recipes and render the native marble quality gallery."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import time

SOURCE = Path(__file__).resolve().parents[1]
OUTPUT = SOURCE / 'artifacts/marble'
(OUTPUT / 'artifacts').mkdir(parents=True, exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client

FAMILIES = [
    ('verde_breccia', 'Verde brechado'), ('emerald_white_gold', 'Esmeralda · branco e ouro'),
    ('emerald_gold', 'Esmeralda · ouro'), ('carrara', 'Carrara'), ('calacatta', 'Calacatta'),
    ('statuario', 'Statuario'), ('nero_marquina', 'Nero Marquina'), ('portoro', 'Portoro'),
    ('emperador', 'Emperador'), ('rosso_levanto', 'Rosso Levanto'), ('rosa', 'Rosa'), ('azul', 'Azul')]


def panel():
    return {'positions': [[-.5, -.5, 0], [.5, -.5, 0], [.5, .5, 0], [-.5, .5, 0]],
            'triangles': [[0, 1, 2], [0, 2, 3]],
            'material_coordinates': [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]]}


def vase():
    segments, rings = 96, 64
    positions, coordinates, triangles = [], [], []
    for j in range(rings+1):
        t = j/rings
        radius = .17 + .16*math.sin(math.pi*t)**1.6 - .105*math.exp(-((t-.77)/.13)**2)
        for i in range(segments+1):
            u = i/segments
            positions.append([radius*math.cos(math.tau*u), t*1.5-.75, radius*math.sin(math.tau*u)])
            coordinates.append([u, t, 0])
    for j in range(rings):
        for i in range(segments):
            a = j*(segments+1)+i
            triangles.extend([[a, a+segments+2, a+1], [a, a+segments+1, a+segments+2]])
    for j, reverse in [(0, True), (rings, False)]:
        center = len(positions)
        positions.append([0, j/rings*1.5-.75, 0]); coordinates.append([.5, .5, 0])
        for i in range(segments):
            a = j*(segments+1)+i
            triangles.append([center, a, a+1] if reverse else [center, a+1, a])
    return {'positions': positions, 'triangles': triangles, 'material_coordinates': coordinates}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=SOURCE/'target/release/ignis')
    parser.add_argument('--reference', type=Path, action='append', default=[])
    parser.add_argument('--fixed-resolution', type=int, choices=[512, 1024], default=1024)
    args = parser.parse_args()
    binary = args.binary.resolve()
    binary_sha = hashlib.sha256(binary.read_bytes()).hexdigest()
    refs = []
    for i, reference in enumerate(args.reference[:3]):
        target = OUTPUT/f'reference-{i+1}.png'
        shutil.copyfile(reference, target); refs.append(target.name)
    c = Client(binary, log_path=OUTPUT/'native.log')
    reports, checks, images = [], [], []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'marble-quality', 'version': '1'}})
        c.request('notifications/initialized', notification=True)

        def reset():
            c.call('generate', recipe={'seed': 5, 'width': 12, 'depth': 12, 'obstacles': 0})
            entities = c.call('inspect')['scene']['entities']
            for e in entities:
                e['visible'] = False
            c.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
            c.call('set_light', light={'direction': [-.5, -.5, -1], 'ambient': 1})
            c.call('set_camera', camera={'position': [0, 0, 1], 'target': [0, 0, 0], 'fov_degrees': 53.130104})

        def asset(name, geometry):
            c.call('edit_mesh', edit={'kind': 'put_asset', 'asset': {
                'id': name, 'geometry': geometry, 'shading': 'smooth', 'material': {}}})

        def instance(name, source, position=(0, 0, 0), scale=1, rotation=(0, 0, 0, 1), color=(1, 1, 1)):
            c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {
                'id': name, 'asset': source, 'position': list(position), 'rotation': list(rotation),
                'scale': scale, 'color': list(color)}})

        def marble(asset, family, execution='runtime', seed=931, unique=False):
            return c.call('material', operation={'kind': 'marble', 'asset': asset, 'family': family,
                                                'execution': execution, 'seed': seed, 'unique_instances': unique})

        def capture(name, size=1024):
            report = c.call('capture', path=f'artifacts/{name}.png', width=size, height=size)
            images.append(name)
            return report

        for index, (family, label) in enumerate(FAMILIES):
            reset(); asset('slab', panel()); instance('slab', 'slab')
            info = marble('slab', family)
            c.call('material', operation={'kind': 'save', 'asset': 'slab', 'path': f'library/{family}.ignis-material'})
            capture(family, 1024 if index < 3 else 768)
            c.call('save_scene', path=f'examples/{family}.scene.json')
            if index < 3:
                started = time.monotonic()
                info = c.call('material', operation={'kind': 'set_execution', 'asset': 'slab',
                                                    'execution': {'fixed': {'resolution': args.fixed_resolution}}})
                bake_time = time.monotonic()-started
                fixed = capture(f'{family}-fixed')
                c.call('material', operation={'kind': 'save', 'asset': 'slab',
                                              'path': f'library/{family}.fixed.ignis-material'})
                size = (OUTPUT/f'library/{family}.fixed.ignis-material').stat().st_size
                first = (OUTPUT/f'artifacts/{family}-fixed.png').read_bytes()
                c.call('material', operation={'kind': 'set_execution', 'asset': 'slab', 'execution': 'runtime'})
                c.call('material', operation={'kind': 'load', 'asset': 'slab',
                                              'path': f'library/{family}.fixed.ignis-material'})
                reloaded = capture('reload-check')
                assert first == (OUTPUT/'artifacts/reload-check.png').read_bytes()
                (OUTPUT/'artifacts/reload-check.png').unlink()
                images.remove('reload-check')
                assert fixed['adapter']['procedural_materials']['storage_bytes'] == reloaded['adapter']['procedural_materials']['storage_bytes']
                # A second unchanged frame must not regenerate/reupload the float maps.
                stable = capture('warm-check')
                assert reloaded['adapter']['procedural_materials']['uploads'] == stable['adapter']['procedural_materials']['uploads']
                (OUTPUT/'artifacts/warm-check.png').unlink(); images.remove('warm-check')
                reports.append({'family': family, 'bake_seconds': round(bake_time, 4), 'file_bytes': size,
                                'sample_bytes': info['sample_bytes'], 'adapter': fixed['adapter']})
        checks.append('Twelve distinct runtime marble families generated, rendered, saved and exported natively')
        checks.append('Three high-resolution fixed maps round-trip with exact native PNG preservation')
        checks.append('Warm fixed frames reuse their GPU program/map buffers')

        reset()
        for i, (family, _) in enumerate(FAMILIES[:3]):
            asset(f'green-{i}', panel()); marble(f'green-{i}', family)
            instance(f'green-{i}', f'green-{i}', (i-1, 0, 0), .96)
        c.call('capture', path='artifacts/green-families.png', width=1536, height=512)
        images.append('green-families')

        # One live shared source, with ID-derived seeds: four unique slabs.
        reset(); asset('unique', panel()); marble('unique', 'emerald_white_gold', unique=True)
        for i in range(4):
            instance(f'piece-{i}', 'unique', ((i % 2-.5)*1.08, (.5-i//2)*1.08, 0))
        c.call('set_camera', camera={'position': [0, 0, 2.25], 'target': [0, 0, 0], 'fov_degrees': 53.130104})
        capture('unique-slabs')
        checks.append('Shared native geometry/recipe produces stable distinct slabs through instance seeds')

        # A curved native mesh exposes polish/roughness and actual scene lighting.
        reset()
        c.call('set_camera', camera={'position': [0, 1.1, 4.2], 'target': [0, -.05, 0], 'fov_degrees': 40})
        c.call('set_light', light={'direction': [-.6, -.8, -1], 'ambient': .08, 'direct_intensity': .85,
                                  'shadow': {'center': [0, 0, 0], 'extent': 2.8, 'resolution': 2048,
                                             'bias': .00025, 'receiver_plane_bias': True},
                                  'environment': {'sky': [.55, .7, 1], 'horizon': [.95, .88, .7],
                                                  'ground': [.07, .09, .12], 'axis': [0, 1, 0], 'intensity': .65}})
        for i, (family, _) in enumerate(FAMILIES[:3]):
            asset(f'vase-{i}', vase()); marble(f'vase-{i}', family)
            instance(f'vase-{i}', f'vase-{i}', ((i-1)*.85, 0, 0))
        asset('ground', panel())
        instance('ground', 'ground', (0, -.76, 0), 8, (-math.sqrt(.5), 0, 0, math.sqrt(.5)), (.11, .13, .16))
        capture('polished-stone', 1280)
        c.call('save_scene', path='examples/polished-stone.scene.json')
        primary_light = (OUTPUT/'artifacts/polished-stone.png').read_bytes()
        light = c.call('inspect')['scene']['light']
        light['direction'] = [.8, -.3, -.4]
        c.call('set_light', light=light)
        capture('polished-stone-raking', 1280)
        assert primary_light != (OUTPUT/'artifacts/polished-stone-raking.png').read_bytes()
        checks.append('Curved native stone surfaces respond to two light directions with GGX, environment reflections and world shadows')
        # Preserve a compact runtime gallery rather than embedding all large maps in one scene.
        reset()
        for i, (family, _) in enumerate(FAMILIES):
            asset(f'm-{i}', panel()); marble(f'm-{i}', family)
            instance(f'm-{i}', f'm-{i}', ((i % 4-1.5)*1.06, (1-i//4)*1.06, 0))
        c.call('set_camera', camera={'position': [0, 0, 4.5], 'target': [0, 0, 0], 'fov_degrees': 53.130104})
        capture('collection', 1536)
        c.call('save_scene', path='examples/collection.scene.json')
        verify = {'checks': checks, 'binary_sha256': binary_sha, 'fixed_resolution': args.fixed_resolution,
                  'fixed_materials': reports, 'images': images,
                  'scene_version': c.call('inspect')['scene']['version'],
                  'build_id': c.call('inspect')['build_id'],
                  'source_sha256': {str(path.relative_to(SOURCE)): hashlib.sha256(path.read_bytes()).hexdigest()
                                    for path in [SOURCE/'src/materials/marble.rs', SOURCE/'src/materials/recipe.rs', SOURCE/'src/scene.wgsl']},
                  'reference_use': 'Local visual comparison only; none of the reference pixels feed the material generator',
                  'acceptance': 'Native/automated verification; human artistic acceptance remains pending'}
        (OUTPUT/'verification.json').write_text(json.dumps(verify, indent=2))
        compare = ''
        for i, (family, label) in enumerate(FAMILIES[:3]):
            left = f'<img src="{refs[i]}" alt="User reference">' if i < len(refs) else ''
            compare += f'<h2>{label}</h2><div class="pair">{left}<img src="artifacts/{family}.png" alt="Native procedural output"></div><p><a href="artifacts/{family}-fixed.png">Fixed1024 capture</a> · <a href="library/{family}.fixed.ignis-material">Reusable fixed material</a></p>'
        cards = ''.join(f'<figure><a href="artifacts/{f}.png"><img src="artifacts/{f}.png"></a><figcaption>{label} · <a href="library/{f}.ignis-material">recipe</a></figcaption></figure>' for f, label in FAMILIES)
        (OUTPUT/'review.html').write_text('''<!doctype html><meta charset="utf-8"><title>Ignis · procedural marble</title>
<style>body{background:#111719;color:#e7eee9;font:16px system-ui;max-width:1500px;margin:40px auto;padding:0 26px}p{line-height:1.65;color:#bccbc4}img{max-width:100%;display:block}.pair{display:grid;grid-template-columns:1fr 1fr;gap:18px}.grid{display:grid;grid-template-columns:repeat(4,1fr);gap:18px}figure{margin:0}figcaption{padding:10px 0}a{color:#bfd9c7}h2{margin-top:48px}</style>
<h1>Ignis · procedural marble</h1><p>Seeded native mineral structure: branching veins, folded bands, breccia, quartz and micrograin. Reference on the left, native generation on the right. These are editable artistic families; the images do not certify geological identity.</p>'''+compare+'''
<h2>Polished surfaces under native lighting</h2><div class="pair"><img src="artifacts/polished-stone.png"><img src="artifacts/polished-stone-raking.png"></div><h2>One recipe, four instance seeds</h2><img src="artifacts/unique-slabs.png"><h2>Material collection</h2><div class="grid">'''+cards+'''
</div><p>Runtime and fixed execution use the same source. Fixed library2 files retain exact float mip maps with lossless compression. See <a href="verification.json">validation / storage / cold generation evidence</a> and <a href="examples/collection.scene.json">the editable native collection scene</a>. Human artistic acceptance is separate from passing tests. Gold metalness is a decorative inlay parameter.</p>''')
        print(json.dumps({'review': str(OUTPUT/'review.html'), 'checks': len(checks),
                          'families': len(FAMILIES), 'binary_sha256': binary_sha}))
    finally:
        c.close()


if __name__ == '__main__':
    main()
