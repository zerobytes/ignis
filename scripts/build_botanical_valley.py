#!/usr/bin/env python3
"""Prepare a finite 1 km native botanical landscape; never modifies human scenes.

Build-time placement, shared meshes and full source triangles. No runtime LOD,
growth, streaming or offline beauty renderer is introduced by this composition.
"""
import argparse
import copy
import gc
import hashlib
import html
import json
import math
import random
import time
from collections import Counter
from pathlib import Path

from botany_collections import COLLECTIONS, collection
from build_representative_forest import Grid
from demo_groundcover import scene_stats
from groundcover_assets import atomic_json, source_fingerprint
from verify_lab import Client, ROOT
from verify_terrain import position

OUT = ROOT / 'artifacts/botanical-valley'
RECIPE = ROOT / 'examples/botanical-valley.recipe.json'


def rel(path):
    return str(path.relative_to(ROOT))


def init(c):
    c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                               clientInfo=dict(name='botanical-valley', version='1')))
    c.request('notifications/initialized', notification=True)


def terrain_recipe():
    # Low valley floor and three asymmetrical ridges; heights are metres.
    ridges = [((-265, -205), -28, 570, 380, 74, .4),
              ((290, -240), 38, 510, 390, 96, 1.8),
              ((320, 280), -42, 430, 310, 54, 2.9)]
    return dict(version=2, space_version=1, id='botanical_valley', revision=1,
                datum=position(0, 0), base=dict(kind='constant', height_meters=12),
                operations=[
                    dict(kind='raise', center=position(-70, -30), radius_meters=570, amount_meters=17),
                    *[dict(kind='ridge', center=position(*p), azimuth_degrees=a,
                           width_meters=w, depth_meters=d, height_meters=h, phase=f,
                           crest_wander=.19, slope_exponent=1.55, folds_meters=4.2,
                           end_fade=.28) for p, a, w, d, h, f in ridges],
                    dict(kind='noise', seed=210, period_meters=256, octaves=4,
                         persistence=.43, amplitude_meters=5, style='smooth'),
                    dict(kind='noise', seed=731, period_meters=32, octaves=3,
                         persistence=.38, amplitude_meters=.65, style='smooth'),
                    dict(kind='channel', points=[position(*p) for p in
                         [(38, 500), (54, 330), (16, 140), (-38, -40), (-4, -240), (45, -500)]],
                         width_meters=34, bed_start_meters=5, bed_end_meters=3, blend=1)],
                water_level_meters=5.25)


def path_x(z):
    return -94 + 22 * math.sin(z / 85) + 12 * math.sin(z / 31)


def field(x, z, phase=0):
    return .5 + .25 * math.sin(x / 33 + phase) + .25 * math.sin(z / 47 - x / 89 + phase * 2)


def light(center=(0, 30, 0), extent=220):
    return dict(direction=[.48, -.72, -.48], ambient=.18, direct_intensity=1.12,
                environment=dict(sky=[.46, .61, .83], horizon=[.69, .74, .72],
                                 ground=[.075, .105, .035], axis=[0, 1, 0], intensity=.32),
                atmosphere=dict(density=.00028, color=[.66, .76, .87],
                                sun_color=[.98, .79, .49], sun_strength=.08,
                                reduce_details=False, silhouette_error_px=.5, color_error=.002),
                shadow=dict(center=list(center), extent=extent, resolution=2048, bias=.0005))


def build_terrain(binary, recipe):
    c = Client(binary, log_path=OUT / 'terrain.log')
    try:
        init(c)
        c.call('set_execution_mode', mode='continuous')
        c.call('generate', recipe=dict(width=32, depth=32, seed=recipe['seed'], obstacles=0))
        initial = c.call('inspect')
        player = copy.deepcopy(next(e for e in initial['scene']['entities'] if e['id'] == 'player'))
        c.call('edit_scene', edits=[dict(op='remove', id=e['id']) for e in initial['scene']['entities']])
        n = recipe['size_meters'] // recipe['spacing_meters']
        doc = terrain_recipe()
        publication = c.call('publish_terrain_region', publication=dict(
            document=doc, region=dict(origin=position(-500, -500), cells=[n, n],
                                     spacing_meters=recipe['spacing_meters']),
            frame_origin=position(0, 0), asset='valley_ground', instance='valley_ground',
            color=[.23, .31, .105], collision=True, world_material_coordinates=True,
            material=dict(pattern=dict(kind='noise', frequency=[.085, .085, .085],
                low=[.48, .55, .32], high=[1.10, 1.03, .72], bump=.014,
                warp=.4, octaves=4, seed=210, contrast=1.3),
                reflectance=dict(roughness=.9, f0=.015))))
        c.call('save_terrain', path=rel(OUT / 'terrain.ignis-terrain'), document=doc)
        c.call('set_light', light=light())
        c.call('save_scene', path=rel(OUT / 'terrain.scene.json'))
        atomic_json(OUT / 'terrain-build.json', dict(build_id=initial['build_id'], publication=publication))
        atomic_json(OUT / 'player-template.json', player)
    finally:
        c.close()


def load_catalog(scene):
    catalog = {}
    inputs = {}
    for name in COLLECTIONS:
        cfg = collection(name)
        manifest = json.loads((cfg['destination'] / 'library.json').read_text())
        built = json.loads((cfg['output'] / 'library-build.json').read_text())
        if source_fingerprint(cfg['destination'], manifest) != built['source_sha256']:
            raise ValueError(f'Stale {name} library; regenerate it explicitly before composing')
        for filename, key in [('library.scene.json', 'scene_sha256'), ('templates.json', 'templates_sha256')]:
            if hashlib.sha256((cfg['output'] / filename).read_bytes()).hexdigest() != built[key]:
                raise ValueError(f'Changed compiled {name}/{filename}')
        compiled = json.loads((cfg['output'] / 'library.scene.json').read_text())
        templates = json.loads((cfg['output'] / 'templates.json').read_text())
        # All low vegetation variants; one intact exemplar per heavy shrub habit.
        entries = [e for e in manifest['entries'] if name in ('groundcover', 'tallgrass') or e['variant'] == 0]
        needed = {i['asset'] for e in entries for i in templates[e['id']]}
        scene['meshes']['assets'].extend(a for a in compiled['meshes']['assets'] if a['id'] in needed)
        catalog[name] = {e['id']: dict(family=e['family'], templates=templates[e['id']]) for e in entries}
        inputs[name] = dict(source_sha256=built['source_sha256'], scene_sha256=built['scene_sha256'],
                            entries=[e['id'] for e in entries])
        del compiled
        gc.collect()
        print('catalog', name, len(entries), flush=True)
    # Existing native exemplar bake: preserve its actual geometry, no replacement crowns.
    tree_path = ROOT / 'artifacts/botany-native/gallery.scene.json'
    trees = json.loads(tree_path.read_text())
    catalog['trees'] = {}
    for name in ('palmeirinha', 'platano', 'eucalipto', 'figueira'):
        templates = [i for i in trees['meshes']['instances'] if i['id'].startswith(name + '_')]
        needed = {i['asset'] for i in templates}
        scene['meshes']['assets'].extend(a for a in trees['meshes']['assets'] if a['id'] in needed)
        catalog['trees'][name] = dict(family=name, templates=templates)
    inputs['trees'] = dict(path=rel(tree_path), sha256=hashlib.sha256(tree_path.read_bytes()).hexdigest(),
                           provenance='Existing native four-exemplar gallery bake; not regenerated adult presets')
    del trees
    gc.collect()
    # The broad RS fig tree uses the corrected collar/root geometry, unchanged.
    rs_path = ROOT / 'artifacts/botany-collar/figueira.scene.json'
    rs = json.loads(rs_path.read_text())
    templates = [i for i in rs['meshes']['instances'] if i['id'].startswith('figueira_rs_')]
    if not templates:
        raise ValueError('Corrected RS fig tree templates not found')
    needed = {i['asset'] for i in templates}
    scene['meshes']['assets'].extend(a for a in rs['meshes']['assets'] if a['id'] in needed)
    catalog['trees']['figueira_rs'] = dict(family='figueira_rs', templates=templates)
    inputs['figueira_rs'] = dict(path=rel(rs_path), sha256=hashlib.sha256(rs_path.read_bytes()).hexdigest())
    del rs
    gc.collect()
    ids = [a['id'] for a in scene['meshes']['assets']]
    if len(ids) != len(set(ids)):
        raise ValueError('Asset ID collision; refuse to merge unrelated geometry')
    return catalog, inputs


def populate(scene, catalog, recipe):
    terrain_i = next(i for i in scene['meshes']['instances'] if i['id'] == 'valley_ground')
    terrain_a = next(a for a in scene['meshes']['assets'] if a['id'] == terrain_i['asset'])
    grid = Grid(terrain_i, terrain_a)
    rng = random.Random(recipe['seed'])
    counts = Counter()
    records = []
    used = set()
    tree_sites = []

    def add(name, entry_id, x, z, scale, yaw=None):
        data = catalog[name][entry_id]
        height, _ = grid.sample(x, z)
        yaw = rng.random() * math.tau if yaw is None else yaw
        rotation = [0, math.sin(yaw / 2), 0, math.cos(yaw / 2)]
        plant_id = f'v{len(records):07d}'
        tint = rng.uniform(.88, 1.12)
        members = []
        for k, template in enumerate(data['templates']):
            instance = copy.deepcopy(template)
            instance.update(id=f'{plant_id}_{k}', position=[x, height - .025, z],
                            rotation=rotation, scale=scale)
            instance['color'] = [min(1, v * tint) for v in template['color']]
            scene['meshes']['instances'].append(instance)
            members.append(instance['id'])
        scene['meshes']['objects'].append(dict(id=plant_id, members=members))
        records.append(dict(id=plant_id, collection=name, entry=entry_id,
                            position=[x, height - .025, z], scale=scale))
        counts[name] += 1
        counts[name + ':' + data['family']] += 1
        used.add((name, entry_id))

    def valid(x, z, trail=1.4, max_slope=34):
        h, up = grid.sample(x, z)
        return h > 6.0 and up > math.cos(math.radians(max_slope)) and abs(x - path_x(z)) > trail

    def choose(name, x, z):
        options = list(catalog[name])
        if name == 'groundcover':
            families = [catalog[name][e]['family'] for e in options]
            weights = []
            for f in families:
                base = 3.8 if f in ('grama_baixa', 'grama_fina', 'mato_trifolio') else .6
                if f in ('flores_brancas', 'flores_roxas'):
                    base = .12 + 3.6 * field(x, z, 2.1) ** 7
                weights.append(base * (.18 + field(x, z, len(f) * .9) ** 3))
            return rng.choices(options, weights=weights)[0]
        return rng.choices(options, weights=[.12 + field(x, z, j * .77) ** 4 for j in range(len(options))])[0]

    # Trees form groves; some mature figs are solitary landmarks.
    groves = [(-245, -150, 95), (250, -230, 95), (280, 195, 70), (-265, 310, 65)]
    names = list(catalog['trees'])
    tree_attempts = 0
    while counts['trees'] < recipe['trees']:
        tree_attempts += 1
        if tree_attempts > max(10000, recipe['trees'] * 300):
            raise ValueError('Tree spacing/grove area cannot satisfy the requested count')
        index = counts['trees']
        if index == 0:
            x, z, entry = -145., 70., 'figueira_rs'
        elif index == 1:
            x, z, entry = 120., -65., 'figueira_rs'
        else:
            gx, gz, radius = rng.choice(groves)
            x, z = gx + rng.gauss(0, radius * .55), gz + rng.gauss(0, radius * .5)
            if not (-475 < x < 475 and -475 < z < 475):
                continue
            entry = names[index % len(names)]
        if not valid(x, z, trail=12, max_slope=30):
            if index < 2:
                raise ValueError('Landmark tree site unsuitable')
            continue
        if any((x - tx) ** 2 + (z - tz) ** 2 < 15 ** 2 for tx, tz in tree_sites):
            continue
        scale = rng.uniform(.82, 1.15)
        add('trees', entry, x, z, scale)
        tree_sites.append((x, z))
    print('trees', counts['trees'], flush=True)

    # Strata are actual full geometry, distributed across the whole finite region.
    for name, key in [('groundcover', 'groundcover'), ('tallgrass', 'tallgrass'),
                      ('shrubs', 'shrubs'), ('colored-shrubs', 'colored_shrubs'),
                      ('reference-shrubs', 'reference_shrubs')]:
        target = recipe[key]
        attempts = 0
        while counts[name] < target:
            attempts += 1
            if attempts > target * 80:
                raise ValueError('Unable to satisfy planting count: ' + name)
            x, z = rng.uniform(-498, 498), rng.uniform(-498, 498)
            if not valid(x, z):
                continue
            if name == 'tallgrass' and rng.random() > .12 + .85 * field(x, z, 1.3) ** 3:
                continue
            if name in ('shrubs', 'colored-shrubs', 'reference-shrubs'):
                if abs(x - path_x(z)) < 3 or any((x - tx) ** 2 + (z - tz) ** 2 < 8 ** 2 for tx, tz in tree_sites):
                    continue
            entry = choose(name, x, z)
            if name == 'groundcover':
                scale = rng.uniform(1.0, 1.75)
            elif name == 'tallgrass':
                scale = rng.uniform(.72, 1.18)
            else:
                scale = rng.uniform(.8, 1.35)
            add(name, entry, x, z, scale)
        print('planted', name, counts[name], flush=True)

    # Dense close foregrounds are explicit authored zones, not distance culling.
    zones = [(-80, 175, 48), (125, 75, 44), (-180, -105, 46)]
    for _ in range(recipe['detail_groundcover']):
        while True:
            cx, cz, radius = rng.choice(zones)
            # Density falls continuously to zero at the edge, avoiding a wall
            # between the dense foreground and the surrounding meadow.
            a = rng.random() * math.tau
            r = radius * math.sqrt(1 - math.sqrt(1 - rng.random()))
            x, z = cx + r * math.cos(a), cz + r * math.sin(a)
            if valid(x, z):
                break
        if field(x, z, 2.1) > .64 and rng.random() < .24:
            flower_entries = [entry for entry in catalog['groundcover'] if entry.startswith(('flores_brancas_', 'flores_roxas_'))]
            entry = rng.choice(flower_entries)
        else:
            entry = choose('groundcover', x, z)
        add('groundcover', entry, x, z, rng.uniform(.85, 1.25))

    # Deliberate wildflower islands at trail bends, with genuine flower geometry.
    flowers = [entry for entry in catalog['groundcover'] if entry.startswith(('flores_brancas_', 'flores_roxas_'))]
    for cx, cz in [(-70, 165), (-105, 136), (139, 78), (-165, -103)]:
        for _ in range(400):
            a, r = rng.random() * math.tau, 4 * math.sqrt(rng.random())
            x, z = cx + r * math.cos(a), cz + r * math.sin(a)
            if valid(x, z):
                add('groundcover', rng.choice(flowers), x, z, rng.uniform(.9, 1.3))

    # Every selected heavy habit also appears around the main walk, visibly.
    for n, name in enumerate(('shrubs', 'colored-shrubs', 'reference-shrubs', 'tallgrass')):
        for j, entry in enumerate(catalog[name]):
            z = 145 - j * 4.1 - n * 12
            x = path_x(z) + (8 + n * 4) * (-1 if j % 2 else 1)
            if valid(x, z):
                add(name, entry, x, z, rng.uniform(.85, 1.1))
    missing = sorted((name, entry) for name, entries in catalog.items() for entry in entries if (name, entry) not in used)
    if missing:
        raise ValueError(f'Unrepresented selected assets: {missing}')
    return grid, records, dict(counts)


def views(grid):
    def cam(x, z, eye, target, fov=58):
        return dict(position=[x, grid.sample(x, z)[0] + eye, z], target=target, fov_degrees=fov)
    return dict(
        panorama=dict(position=[650, 640, 760], target=[0, 20, -20], fov_degrees=58),
        valley=cam(-95, 205, 4.2, [-12, 28, -90], 61),
        flowers=cam(-70, 176, 1.05, [-70, grid.sample(-70, 165)[0] + .35, 165], 58),
        grove=cam(-187, 122, 2.1, [-145, grid.sample(-145, 70)[0] + 10, 70], 58),
        ridge=cam(165, 106, 2.5, [-20, 22, -130], 62))


def prepare(binary, recipe):
    scene = json.loads((OUT / 'terrain.scene.json').read_text())
    # Keep the schema emitted by the executable, not an unbuilt parallel source.
    scene['name'] = 'Vale botânico · 1 km × 1 km'
    scene['botany'] = []
    scene['meshes']['objects'] = []
    catalog, inputs = load_catalog(scene)
    grid, records, counts = populate(scene, catalog, recipe)
    cameras = views(grid)
    scene['camera'] = cameras['valley']
    scene['light'] = light([-70, grid.sample(-70, 145)[0], 145], 180)
    # Flat water is visual only; terrain is the sole physical ground.
    scene['meshes']['assets'].append(dict(id='valley_water', shading='smooth',
        geometry=dict(positions=[[-499, 5.25, -499], [499, 5.25, -499], [-499, 5.25, 499], [499, 5.25, 499]],
                      triangles=[[0, 2, 1], [1, 2, 3]]),
        material=dict(reflectance=dict(roughness=.29, f0=.035))))
    scene['meshes']['instances'].append(dict(id='valley_water', asset='valley_water',
        position=[0, 0, 0], rotation=[0, 0, 0, 1], scale=1, color=[.045, .15, .17]))
    player = json.loads((OUT / 'player-template.json').read_text())
    x, z = path_x(195), 195
    player['transform'] = dict(position=[x, grid.sample(x, z)[0] + .94, z], size=[.6, 1.8, .6])
    player['character_controller'] = dict(max_slope_degrees=40, slide_slope_degrees=45, snap_distance=.2)
    player['footsteps'] = None
    player['name'] = 'Explorador do vale'
    scene['entities'] = [player]
    stats = scene_stats(scene)
    terrain_vertices = next(a for a in scene['meshes']['assets'] if a['id'] == 'valley_ground')['geometry']['positions']
    terrain_heights = [p[1] for p in terrain_vertices]
    report = dict(recipe=recipe, inputs=inputs, counts=counts, stats=stats,
        bounds_xz=[[-500, -500], [500, 500]], area_m2=1_000_000,
        terrain_min_max=[min(terrain_heights), max(terrain_heights)],
        native_lod=False, streaming=False, uniform_ground_coverage=False,
        water_simulation=False, runtime_fps_measured=False, artistic_acceptance=False)
    atomic_json(OUT / 'placements.json', records)
    atomic_json(OUT / 'views.json', cameras)
    atomic_json(OUT / 'composition.json', report)
    atomic_json(OUT / 'world.candidate.scene.json', scene)
    print('composition', stats, flush=True)


def validate_and_capture(binary):
    c = Client(binary, log_path=OUT / 'native.log')
    try:
        init(c)
        build_id = c.call('inspect')['build_id']
        c.call('set_execution_mode', mode='continuous')
        started = time.monotonic()
        c.call('load_scene', path=rel(OUT / 'world.candidate.scene.json'))
        load_seconds = time.monotonic() - started
        c.call('save_scene', path=rel(OUT / 'world.scene.json'))
        placement_check = verify_native_positions()
        c.call('load_scene', path=rel(OUT / 'world.scene.json'))
        c.call('step', ticks=45)
        start = c.call('inspect_world')['positions']['player']
        c.call('set_input', input=dict(x=0, z=-1))
        c.call('step', ticks=120)
        c.call('set_input', input=dict(x=0, z=0))
        end = c.call('inspect_world')['positions']['player']
        controllers = c.call('inspect_character_controllers')
        assert controllers['controllers']['player']['grounded'], controllers
        assert math.dist(start, end) > 3, (start, end)
        atomic_json(OUT / 'native-validation.json', dict(build_id=build_id, placement=placement_check,
            load_seconds=load_seconds, walk_start=start, walk_end=end,
            walk_meters=math.dist(start, end), controllers=controllers,
            collision=c.call('inspect_mesh_collisions')))
        # Restore authored actor state after the walk, then review all cameras.
        c.call('load_scene', path=rel(OUT / 'world.scene.json'))
        capture_views(c)
        (OUT / 'world.candidate.scene.json').unlink(missing_ok=True)
    finally:
        c.close()


def verify_native_positions():
    """Check persisted native transforms against the real published triangles."""
    scene = json.loads((OUT / 'world.scene.json').read_text())
    meshes = scene['meshes']
    terrain_i = next(i for i in meshes['instances'] if i['id'] == 'valley_ground')
    terrain_a = next(a for a in meshes['assets'] if a['id'] == terrain_i['asset'])
    grid = Grid(terrain_i, terrain_a)
    instances = {i['id']: i for i in meshes['instances']}
    records = json.loads((OUT / 'placements.json').read_text())
    objects = {o['id']: o['members'] for o in meshes['objects']}
    assert len(objects) == len(records)
    error = 0.
    cells = set()
    for record in records:
        members = objects[record['id']]
        first = instances[members[0]]['position']
        x, y, z = first
        assert -500 <= x <= 500 and -500 <= z <= 500
        error = max(error, abs(y + .025 - grid.sample(x, z)[0]))
        for member in members:
            assert instances[member]['position'] == first
        if record['collection'] == 'groundcover':
            cells.add((math.floor((x + 500) / 50), math.floor((z + 500) / 50)))
    assert error < .0001, error
    result = dict(plants_checked=len(records), max_root_transform_error_m=error,
                  groundcover_occupied_50m_cells=len(cells), total_50m_cells=400,
                  terrain_spacing_m=grid.xs[1] - grid.xs[0], plant_component_transforms_equal=True)
    del scene, meshes, instances, records, objects, grid
    gc.collect()
    return result


def capture_views(c):
    cameras = json.loads((OUT / 'views.json').read_text())
    report = []
    for name, camera in cameras.items():
        c.call('set_camera', camera=camera)
        center = camera['target'] if name == 'panorama' else [camera['position'][0], camera['position'][1] - 1, camera['position'][2] - 45]
        c.call('set_light', light=light(center, 256 if name == 'panorama' else 140))
        start = time.monotonic()
        result = c.call('capture', path=rel(OUT / (name + '.png')), width=2048, height=1280)
        report.append(dict(view=name, elapsed_seconds=time.monotonic() - start, metadata=result))
        print('captured', name, flush=True)
    # Save/load check uses identical camera/light and actor state.
    c.call('save_scene', path=rel(OUT / 'review.scene.json'))
    c.call('load_scene', path=rel(OUT / 'review.scene.json'))
    c.call('capture', path=rel(OUT / 'roundtrip.png'), width=2048, height=1280)
    if (OUT / 'roundtrip.png').read_bytes() != (OUT / 'ridge.png').read_bytes():
        raise ValueError('Save/load changed the last reviewed native image')
    (OUT / 'review.scene.json').unlink(missing_ok=True)
    atomic_json(OUT / 'captures.json', report)
    cards = ''.join(f'<figure><a href="{name}.png"><img src="{name}.png"></a><figcaption>{html.escape(name)}</figcaption></figure>' for name in cameras)
    (OUT / 'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Vale botânico · 1 km²</title>'
        '<style>body{margin:24px;background:#18201b;color:#eee;font:16px system-ui}figure{margin:20px 0}img{max-width:100%;width:1200px}a{color:#b6d38a}</style>'
        '<h1>Vale botânico · 1 km × 1 km</h1><p>Capturas nativas Ignis. Geometria compartilhada, terreno físico, água visual. Sem render offline.</p>' + cards)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    p.add_argument('--stage', choices=['terrain', 'compose', 'validate', 'review', 'all'], default='all')
    p.add_argument('--recipe', type=Path, default=RECIPE)
    args = p.parse_args()
    recipe = json.loads(args.recipe.read_text())
    if recipe['version'] != 1 or recipe['size_meters'] != 1000 or recipe['spacing_meters'] not in (2, 4):
        p.error('This authored source requires1km and2m/4m grid spacing')
    for key in ('groundcover', 'detail_groundcover', 'tallgrass', 'shrubs', 'colored_shrubs', 'reference_shrubs', 'trees'):
        if type(recipe[key]) is not int or not 0 <= recipe[key] <= 1_000_000:
            p.error(f'Invalid finite planting count: {key}')
    OUT.mkdir(parents=True, exist_ok=True)
    if args.stage in ('terrain', 'all'):
        build_terrain(args.binary, recipe)
    if args.stage in ('compose', 'all'):
        prepare(args.binary, recipe)
    if args.stage in ('validate', 'all'):
        validate_and_capture(args.binary)
    if args.stage == 'review':
        c = Client(args.binary, log_path=OUT / 'review.log')
        try:
            init(c)
            c.call('load_scene', path=rel(OUT / 'world.scene.json'))
            capture_views(c)
        finally:
            c.close()


if __name__ == '__main__':
    main()
