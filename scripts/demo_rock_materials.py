#!/usr/bin/env python3
"""Render the native, editable mountain-rock material collection without a window."""
import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import time

SOURCE = Path(__file__).resolve().parents[1]
OUTPUT = SOURCE / 'artifacts/rock'
(OUTPUT / 'artifacts').mkdir(parents=True, exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client

FAMILIES = [('alpine', 'Rocha alpina'), ('fractured', 'Rocha escura fraturada'),
            ('ochre', 'Paredão ocre'), ('sedimentary', 'Estratos sedimentares')]


def panel(n=1):
    positions, coordinates, triangles = [], [], []
    for y in range(n+1):
        for x in range(n+1):
            positions.append([x/n-.5, y/n-.5, 0])
            coordinates.append([x/n, y/n, 0])
    for y in range(n):
        for x in range(n):
            a = y*(n+1)+x
            triangles.extend([[a, a+1, a+n+2], [a, a+n+2, a+n+1]])
    return {'positions': positions, 'triangles': triangles, 'material_coordinates': coordinates}


def boulder(seed=0, count=32):
    """Small authored convex source mesh; independent of every material channel."""
    points = []
    for i in range(count):
        y = 1 - 2 * (i + .5) / count
        angle = i * 2.399963229728653 + seed * .41
        radius = .46 + .07 * math.sin(i * 4.31 + seed)
        ring = math.sqrt(1-y*y)
        points.append([radius * ring * math.cos(angle), radius * y * 1.07,
                       radius * ring * math.sin(angle) * .8])
    faces = []
    def sub(a, b): return [a[k]-b[k] for k in range(3)]
    def dot(a, b): return sum(x*y for x, y in zip(a, b))
    def cross(a, b): return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
    for i in range(count):
        for j in range(i+1, count):
            for k in range(j+1, count):
                normal = cross(sub(points[j], points[i]), sub(points[k], points[i]))
                if dot(normal, normal) < 1e-12: continue
                signs = [dot(normal, sub(p, points[i])) for p in points]
                if max(signs) < 1e-8: faces.append([i,j,k])
                elif min(signs) > -1e-8: faces.append([i,k,j])
    edges = {}
    for face in faces:
        for a,b in zip(face, face[1:]+face[:1]):
            key = tuple(sorted((a,b))); edges[key] = edges.get(key, 0) + 1
    assert faces and all(v == 2 for v in edges.values()), 'Closed manifold hull required'
    return {'positions': points, 'triangles': faces,
            'material_coordinates': [[p[0]+.5, p[1]+.5, p[2]] for p in points]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=SOURCE/'target/release/ignis')
    parser.add_argument('--reference', type=Path, action='append', default=[])
    parser.add_argument('--quick', action='store_true', help='Only current runtime plates and lights for visual iteration')
    args = parser.parse_args()
    binary = args.binary.resolve()
    binary_sha = hashlib.sha256(binary.read_bytes()).hexdigest()
    for i, reference in enumerate(args.reference[:4]):
        shutil.copyfile(reference, OUTPUT/f'reference-{i+1}.png')
    c = Client(binary, log_path=OUTPUT/'native.log')
    checks, fixed_reports, materials = [], [], {}
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                'clientInfo': {'name': 'rock-material-review', 'version': '1'}})
        c.request('notifications/initialized', notification=True)

        def reset():
            c.call('generate', recipe={'seed': 5, 'width': 12, 'depth': 12, 'obstacles': 0})
            entities = c.call('inspect')['scene']['entities']
            for entity in entities:
                entity['visible'] = False
            c.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
            c.call('set_camera', camera={'position': [0, 0, 1], 'target': [0, 0, 0], 'fov_degrees': 53.130104})

        def light(direction=(-.6, -.75, -1), ambient=.14):
            c.call('set_light', light={'direction': list(direction), 'ambient': ambient})

        def asset(name, geometry, shading='smooth'):
            c.call('edit_mesh', edit={'kind': 'put_asset', 'asset': {
                'id': name, 'geometry': geometry, 'shading': shading, 'material': {}}})

        def instance(name, source, position=(0, 0, 0), scale=1, color=(1, 1, 1)):
            c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': {
                'id': name, 'asset': source, 'position': list(position), 'rotation': [0, 0, 0, 1],
                'scale': scale, 'color': list(color)}})

        def rock(name, family, unique=False):
            return c.call('material', operation={'kind': 'rock', 'asset': name, 'family': family,
                                                'seed': 4271, 'unique_instances': unique, 'execution': 'runtime'})

        def capture(name, width=1024, height=None):
            return c.call('capture', path=f'artifacts/{name}.png', width=width, height=height or width)

        for family, _ in FAMILIES:
            reset(); asset('slab', panel()); instance('slab', 'slab'); light()
            info = rock('slab', family)
            assert info['recipe']['normal_filter'] == 'pixel' and info['recipe']['rock']['version'] == 2
            materials[family] = info['recipe']
            c.call('material', operation={'kind': 'save', 'asset': 'slab', 'path': f'library/{family}.ignis-material'})
            capture(family)
            target = SOURCE/f'assets/materials/rock/{family}.ignis-material'
            library = json.loads((OUTPUT/f'library/{family}.ignis-material').read_text())
            target.write_text(json.dumps(library, indent=2)+'\n')
            c.call('save_scene', path=f'examples/{family}.scene.json')
            light((.8, -.2, -.6)); capture(f'{family}-opposing')
            light(ambient=1); capture(f'{family}-ambient')
            if args.quick:
                continue
            light()
            started = time.monotonic()
            info = c.call('material', operation={'kind': 'set_execution', 'asset': 'slab', 'execution': {'fixed': {'resolution': 1024}}})
            seconds = time.monotonic()-started
            capture(f'{family}-fixed')
            original = (OUTPUT/f'artifacts/{family}-fixed.png').read_bytes()
            c.call('material', operation={'kind': 'save', 'asset': 'slab', 'path': f'library/{family}.fixed.ignis-material'})
            c.call('material', operation={'kind': 'set_execution', 'asset': 'slab', 'execution': 'runtime'})
            c.call('material', operation={'kind': 'load', 'asset': 'slab', 'path': f'library/{family}.fixed.ignis-material'})
            loaded = capture('reload')
            assert original == (OUTPUT/'artifacts/reload.png').read_bytes()
            warm = capture('warm')
            assert loaded['adapter']['procedural_materials']['uploads'] == warm['adapter']['procedural_materials']['uploads']
            for name in ['reload', 'warm']:
                (OUTPUT/f'artifacts/{name}.png').unlink()
            fixed_reports.append({'family': family, 'bake_seconds': seconds, 'sample_bytes': info['sample_bytes'],
                                  'file_bytes': (OUTPUT/f'library/{family}.fixed.ignis-material').stat().st_size,
                                  'gpu': warm['adapter']['procedural_materials']})
        checks.append('Four continuous rock2 families saved/rendered under two directional lights and ambient illumination')
        geometry = None
        marble_checks = []
        if not args.quick:
            checks.append('Four fixed1024 libraries reload with byte-identical native PNGs and warm uploads unchanged')
            # Same program, palette and seed; vary structural controls.
            reset(); light()
            controls = [('fine', .35, .08, .12), ('coarse', .95, .3, .65),
                        ('weathered', .7, .9, .9)]
            for i, (name, grain, wear, pits) in enumerate(controls):
                asset(name, panel()); instance(name, name, (i-1, 0, 0), .96)
                recipe = copy.deepcopy(materials['fractured'])
                recipe['rock'].update(grain=grain, weathering=wear, pitting=pits, inclusions=pits)
                c.call('material', operation={'kind': 'generate', 'asset': name, 'recipe': recipe})
            capture('structure-controls', 1536, 512)
            c.call('save_scene', path='examples/structure-controls.scene.json')
            checks.append('Grain, inclusions and shallow wear vary independently from palette and seed')

            reset(); light(); asset('shared', panel()); rock('shared', 'ochre', unique=True)
            for i in range(4):
                instance(f'piece-{i}', 'shared', ((i % 2-.5)*1.05, (.5-i//2)*1.05, 0))
            c.call('set_camera', camera={'position': [0, 0, 2.2], 'target': [0, 0, 0], 'fov_degrees': 53.130104})
            capture('unique-instances')
            mesh = c.call('inspect_meshes')
            assert len(mesh['assets']) == 1 and len(mesh['instances']) == 4
            checks.append('Four stable instance seeds share one source geometry and recipe')

            # Apply one independent source material to different authored meshes.
            reset(); light()
            shapes = [('flat', panel(), 'smooth'), ('angular', boulder(), 'flat'),
                      ('rounded', boulder(4, 90), 'smooth')]
            hashes = {}
            for i, (name, shape, shading) in enumerate(shapes):
                asset(name, shape, shading); instance(name, name, (i-1, 0, 0), .94)
                hashes[name] = hashlib.sha256(json.dumps(shape, sort_keys=True).encode()).hexdigest()
            c.call('set_camera', camera={'position': [0, 0, 2], 'target': [0, 0, 0], 'fov_degrees': 35})
            for name, _, _ in shapes:
                c.call('edit_mesh', edit={'kind':'set_material','asset':name,
                    'material':{'reflectance':{'roughness':.9,'f0':.04}}})
            capture('geometry-neutral', 1536, 512)
            before = c.call('inspect')['scene']['meshes']
            for name, _, _ in shapes: rock(name, 'fractured')
            after = c.call('inspect')['scene']['meshes']
            for a,b in zip(before['assets'], after['assets']):
                assert a['geometry'] == b['geometry'], 'Material must never change source geometry'
            assert before['instances'] == after['instances']
            capture('geometry-material', 1536, 512)
            light((.8, -.2, -.6)); capture('geometry-opposing', 1536, 512)
            c.call('save_scene', path='examples/geometry-material.scene.json')
            geometry = {'source_sha256':hashes, 'material_changes_geometry':False,
                        'shapes':[{'id':n,'vertices':len(g['positions']),'triangles':len(g['triangles'])} for n,g,_ in shapes]}
            checks.append('Identical material on flat, angular and rounded source meshes; geometry/instances exactly unchanged; opposing lights')

            # Preserve the accepted marble appearances. Never replace their review outputs.
            for family in ['verde_breccia', 'emerald_white_gold', 'emerald_gold']:
                baseline = SOURCE/f'artifacts/marble/artifacts/{family}.png'
                if not baseline.exists():
                    continue
                reset(); light(ambient=1); asset('slab', panel()); instance('slab', 'slab')
                c.call('material', operation={'kind': 'marble', 'asset': 'slab', 'family': family, 'seed': 931})
                capture(f'marble-{family}')
                assert baseline.read_bytes() == (OUTPUT/f'artifacts/marble-{family}.png').read_bytes(), f'Accepted marble changed: {family}'
                marble_checks.append(family)
            if marble_checks:
                checks.append('Accepted green marble native captures remain byte identical')

        # Native hero: mass and broken planes exist in authored geometry, even untextured.
        reset(); light()
        for i, (family, _) in enumerate(FAMILIES):
            asset(f'm-{i}', boulder(i), 'flat'); rock(f'm-{i}', family)
            instance(f'm-{i}', f'm-{i}', ((i % 2-.5)*1.08, (.5-i//2)*1.08, 0))
        c.call('set_camera', camera={'position': [.2, .1, 2.5], 'target': [0, 0, 0], 'fov_degrees': 53.130104})
        capture('collection', 1536)
        c.call('save_scene', path='examples/collection.scene.json')
        state = c.call('inspect')
        verify = {'checks': checks, 'quick': args.quick, 'binary_sha256': binary_sha, 'build_id': state['build_id'],
                  'scene_version': state['scene']['version'], 'fixed': fixed_reports, 'geometry': geometry,
                  'accepted_marble_checks': marble_checks, 'human_artistic_acceptance': 'pending',
                  'reference_use': 'Visual analysis only; no reference pixels enter the generator',
                  'source_sha256': {str(p.relative_to(SOURCE)): hashlib.sha256(p.read_bytes()).hexdigest()
                                    for p in [SOURCE/'src/materials/rock.rs', SOURCE/'src/materials/recipe.rs', SOURCE/'src/scene.wgsl']}}
        (OUTPUT/'verification.json').write_text(json.dumps(verify, indent=2)+'\n')
        pairs = ''
        for i, (family, label) in enumerate(FAMILIES):
            reference = f'<img src="reference-{i+1}.png">' if (OUTPUT/f'reference-{i+1}.png').exists() else ''
            pairs += f'<h2>{label}</h2><div class="pair">{reference}<img src="artifacts/{family}.png"></div>'
            pairs += f'<p><a href="library/{family}.ignis-material">Editable material</a> · <a href="artifacts/{family}-ambient.png">Mineral color / ambient cavity</a> · <a href="artifacts/{family}-opposing.png">Opposing light</a>'
            if not args.quick:
                pairs += f' · <a href="artifacts/{family}-fixed.png">Fixed1024</a>'
            pairs += '</p>'
        extra = '' if args.quick else '''<h2>Structure controls · same palette and seed</h2><img src="artifacts/structure-controls.png">
<h2>Shared geometry · distinct instance seeds</h2><img src="artifacts/unique-instances.png">
<h2>Shape belongs to the mesh</h2><div class="pair"><img src="artifacts/geometry-neutral.png"><img src="artifacts/geometry-material.png"></div>
<p>Left: neutral geometry. Right: same geometry coated with continuous stone. No displacement bake or material-derived shape. <a href="artifacts/geometry-opposing.png">Same meshes under opposing light</a>.</p>'''
        (OUTPUT/'review.html').write_text('''<!doctype html><meta charset="utf-8"><title>Ignis · mountain rock materials</title>
<style>body{background:#111719;color:#e4ebe8;font:16px system-ui;max-width:1500px;margin:40px auto;padding:0 26px}p{line-height:1.6;color:#bdcac4}img{max-width:100%;display:block}.pair{display:grid;grid-template-columns:1fr 1fr;gap:18px;align-items:start}a{color:#bfd9c7}h2{margin-top:48px}</style>
<h1>Ignis · mountain rock materials</h1><p>Continuous mineral surfaces, sparse fissures, fine grain and shallow wear. Large broken planes and silhouettes belong to the source mesh. Left: supplied visual reference. Right: native procedural material under scene lighting. The reference pixels are not generator inputs.</p><img src="artifacts/collection.png">'''+extra+pairs+'''
<h2>Collection</h2><img src="artifacts/collection.png"><p>Same source for runtime and fixed execution. See <a href="verification.json">validation and storage evidence</a> and the <a href="examples/collection.scene.json">editable native scene</a>. Human artistic acceptance is separate from automated correctness.</p>''')
        if not args.quick:
            for old in ['examples/geometry-source.scene.json', 'examples/geometry-relief.scene.json',
                        'artifacts/geometry-relief.png', 'artifacts/geometry-relief-opposing.png']:
                (OUTPUT/old).unlink(missing_ok=True)
        print(json.dumps({'review': str(OUTPUT/'review.html'), 'checks': len(checks), 'quick': args.quick, 'binary_sha256': binary_sha}))
    finally:
        c.close()


if __name__ == '__main__':
    main()
