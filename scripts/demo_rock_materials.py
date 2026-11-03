#!/usr/bin/env python3
"""Render the native, editable mountain-rock material collection without a window."""
import argparse
import copy
import hashlib
import json
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

        def asset(name, geometry):
            c.call('edit_mesh', edit={'kind': 'put_asset', 'asset': {
                'id': name, 'geometry': geometry, 'shading': 'smooth', 'material': {}}})

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
            assert info['recipe']['normal_filter'] == 'pixel'
            materials[family] = info['recipe']
            c.call('material', operation={'kind': 'save', 'asset': 'slab', 'path': f'library/{family}.ignis-material'})
            capture(family)
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
        checks.append('Four runtime families saved/rendered under two directional lights and ambient illumination')
        geometry = None
        marble_checks = []
        if not args.quick:
            checks.append('Four fixed1024 libraries reload with byte-identical native PNGs and warm uploads unchanged')
            # Same program, palette and seed; vary structural controls.
            reset(); light()
            controls = [('massive', [2, 3], .18, .1, .015), ('shattered', [8, 9], .9, .25, .025),
                        ('weathered', [4, 5], .55, .95, .055)]
            for i, (name, blocks, secondary, wear, width) in enumerate(controls):
                asset(name, panel()); instance(name, name, (i-1, 0, 0), .96)
                recipe = copy.deepcopy(materials['fractured'])
                recipe['rock'].update(blocks=blocks, secondary_joints=secondary, weathering=wear, joint_width=width)
                c.call('material', operation={'kind': 'generate', 'asset': name, 'recipe': recipe})
            capture('structure-controls', 1536, 512)
            c.call('save_scene', path='examples/structure-controls.scene.json')
            checks.append('Joint density, scale and weathering vary independently from the palette and seed')

            reset(); light(); asset('shared', panel()); rock('shared', 'ochre', unique=True)
            for i in range(4):
                instance(f'piece-{i}', 'shared', ((i % 2-.5)*1.05, (.5-i//2)*1.05, 0))
            c.call('set_camera', camera={'position': [0, 0, 2.2], 'target': [0, 0, 0], 'fov_degrees': 53.130104})
            capture('unique-instances')
            mesh = c.call('inspect_meshes')
            assert len(mesh['assets']) == 1 and len(mesh['instances']) == 4
            checks.append('Four stable instance seeds share one source geometry and recipe')

            # Explicit geometry: heightfield surface, not a solid mountain or overhang.
            # Keep the2MiB MCP envelope intact: the large authored grid travels
            # through the existing buffered local-scene reader, not inline RPC.
            scene = json.loads((OUTPUT/'examples/fractured.scene.json').read_text())
            scene['meshes']['assets'][0]['id'] = 'source'
            scene['meshes']['assets'][0]['geometry'] = panel(512)
            scene['meshes']['instances'][0].update(id='relief', asset='source')
            (OUTPUT/'examples/geometry-source.scene.json').write_text(json.dumps(scene, separators=(',', ':')))
            c.call('load_scene', path='examples/geometry-source.scene.json')
            light()
            asset('receiver', panel()); instance('receiver', 'receiver', (0, 0, -.22), 1.5, (.16, .18, .2))
            reference = c.call('inspect_mesh_topology', asset='receiver')['reference']
            c.call('material', operation={'kind': 'bake_displacement', 'instance': 'relief', 'id': 'baked', 'reference': reference})
            assert c.call('material', operation={'kind': 'inspect', 'asset': 'baked'})['recipe']['height_m'] == 0
            geometry = c.call('inspect_meshes')
            assert len(geometry['assets']) == 3
            source = next(a for a in geometry['assets'] if a['id'] == 'source')
            baked = next(a for a in geometry['assets'] if a['id'] == 'baked')
            assert source['vertices'] == baked['vertices'] == 263169
            assert source['triangles'] == baked['triangles'] == 524288
            c.call('set_camera', camera={'position': [.68, .45, 1.12], 'target': [0, 0, -.055], 'fov_degrees': 48})
            illumination = {'direction': [-.6, -.75, -1], 'ambient': .12,
                            'shadow': {'center': [0, 0, -.1], 'extent': 1.2, 'resolution': 2048,
                                       'bias': .0008, 'receiver_plane_bias': True}}
            c.call('set_light', light=illumination); capture('geometry-relief')
            c.call('save_scene', path='examples/geometry-relief.scene.json')
            illumination['direction'] = [.8, -.2, -.6]
            c.call('set_light', light=illumination); capture('geometry-relief-opposing')
            assert (OUTPUT/'artifacts/geometry-relief.png').read_bytes() != (OUTPUT/'artifacts/geometry-relief-opposing.png').read_bytes()
            checks.append('Explicit512x512 mesh bake preserves source, removes duplicate shader displacement and casts light-dependent world shadows')

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

        reset(); light()
        for i, (family, _) in enumerate(FAMILIES):
            asset(f'm-{i}', panel()); rock(f'm-{i}', family)
            instance(f'm-{i}', f'm-{i}', ((i % 2-.5)*1.04, (.5-i//2)*1.04, 0))
        c.call('set_camera', camera={'position': [0, 0, 2.2], 'target': [0, 0, 0], 'fov_degrees': 53.130104})
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
<h2>Explicit geometry and world shadows</h2><div class="pair"><img src="artifacts/geometry-relief.png"><img src="artifacts/geometry-relief-opposing.png"></div>
<p>Explicit mesh displacement on an authored grid. The source is preserved; this surface is not a solid mountain or an overhang generator.</p>'''
        (OUTPUT/'review.html').write_text('''<!doctype html><meta charset="utf-8"><title>Ignis · mountain rock materials</title>
<style>body{background:#111719;color:#e4ebe8;font:16px system-ui;max-width:1500px;margin:40px auto;padding:0 26px}p{line-height:1.6;color:#bdcac4}img{max-width:100%;display:block}.pair{display:grid;grid-template-columns:1fr 1fr;gap:18px;align-items:start}a{color:#bfd9c7}h2{margin-top:48px}</style>
<h1>Ignis · mountain rock materials</h1><p>Configurable angular faces, fractures, weathering, mineral grain and wavy layers. Left: supplied visual reference. Right: native procedural material under scene lighting. The reference pixels are not generator inputs.</p>'''+pairs+extra+'''
<h2>Collection</h2><img src="artifacts/collection.png"><p>Same source for runtime and fixed execution. See <a href="verification.json">validation and storage evidence</a> and the <a href="examples/collection.scene.json">editable native scene</a>. Human artistic acceptance is separate from automated correctness.</p>''')
        print(json.dumps({'review': str(OUTPUT/'review.html'), 'checks': len(checks), 'quick': args.quick, 'binary_sha256': binary_sha}))
    finally:
        c.close()


if __name__ == '__main__':
    main()
