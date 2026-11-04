#!/usr/bin/env python3
"""Build a finite 1 km emerald lawn with the existing diagnostic field renderer.

The Scene and IGFLD1 sidecar are separate. Neither the renderer nor the approved
botanical sources are changed. Approximate field scales are not fidelity proofs.
"""
import argparse
from array import array
import copy
import hashlib
import html
import json
import math
import os
from pathlib import Path
import struct
import time

from build_botanical_valley import init
from build_representative_forest import Grid
from groundcover_assets import atomic_json, source_fingerprint
from lawn_assets import DEST
from verify_lab import Client, ROOT
from verify_terrain import position

OUT = ROOT / 'artifacts/emerald-meadow'
SIZE, TERRAIN_STEP, SPACING, SIDE, TILES = 1000, 4, .125, 16, 500


def rel(p):
    return str(p.relative_to(ROOT))


def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def review(cameras):
    before = '<section class="compare"><figure><img src="rejected-walk.png"><figcaption>Antes: LOD por bloco</figcaption></figure><figure><img src="walk.png"><figcaption>Agora: LOD por folha, mesma câmera</figcaption></figure></section>' if (OUT / 'rejected-walk.png').exists() else ''
    cards = ''.join(f'<figure><a href="{n}.png"><img src="{n}.png"></a><figcaption>{html.escape(n)}</figcaption></figure>' for n in [*cameras, 'player'])
    (OUT / 'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Esmeralda 1 km²</title><style>body{background:#17211c;color:#eee;font:16px system-ui;margin:24px}img{max-width:100%;width:1400px}.compare{display:grid;grid-template-columns:1fr 1fr}figure{margin:12px}</style><h1>Esmeralda · 1 km × 1 km</h1><p>Capturas nativas sem filtro. player é a câmera real do personagem. Campo experimental vinculado pelo iniciador; SaveScene sozinho não inclui a grama.</p>' + before + cards)


def light(center=(0, 12, 0), extent=45):
    return dict(direction=[.72, -.35, -.45], ambient=.23, direct_intensity=1.12,
        environment=dict(sky=[.52, .65, .83], horizon=[.68, .75, .74],
            ground=[.075, .10, .035], axis=[0, 1, 0], intensity=.35),
        atmosphere=dict(density=.00022, color=[.66, .76, .87], sun_strength=.04,
            sun_color=[.98, .87, .64], reduce_details=False,
            silhouette_error_px=.5, color_error=.002),
        shadow=dict(center=list(center), extent=extent, resolution=2048, bias=.0002))


def document():
    return dict(version=2, space_version=1, id='emerald_meadow', revision=1,
        datum=position(0, 0), base=dict(kind='constant', height_meters=6),
        operations=[
            dict(kind='raise', center=position(-180, -70), radius_meters=650, amount_meters=9),
            dict(kind='ridge', center=position(-245, -230), azimuth_degrees=-30,
                width_meters=580, depth_meters=680, height_meters=13, phase=.4,
                crest_wander=.12, slope_exponent=1.7, folds_meters=.5, end_fade=.35),
            dict(kind='ridge', center=position(270, -160), azimuth_degrees=35,
                width_meters=560, depth_meters=650, height_meters=18, phase=1.7,
                crest_wander=.13, slope_exponent=1.7, folds_meters=.65, end_fade=.35),
            dict(kind='ridge', center=position(290, 330), azimuth_degrees=-45,
                width_meters=560, depth_meters=540, height_meters=11, phase=2.8,
                crest_wander=.12, slope_exponent=1.7, folds_meters=.45, end_fade=.35),
            dict(kind='noise', seed=218, period_meters=256, octaves=3,
                persistence=.38, amplitude_meters=1.3, style='smooth')])


def terrain(binary):
    c = Client(binary, log_path=OUT / 'terrain.log')
    try:
        init(c)
        atomic_json(OUT / 'command-schema.json', c.request('tools/list')['result']['tools'][0]['inputSchema'])
        c.call('set_execution_mode', mode='continuous')
        c.call('generate', recipe=dict(width=32, depth=32, seed=218, obstacles=0))
        state = c.call('inspect')
        player = copy.deepcopy(next(e for e in state['scene']['entities'] if e['id'] == 'player'))
        c.call('edit_scene', edits=[dict(op='remove', id=e['id']) for e in state['scene']['entities']])
        doc = document()
        result = c.call('publish_terrain_region', publication=dict(document=doc,
            region=dict(origin=position(-500, -500), cells=[250, 250], spacing_meters=TERRAIN_STEP),
            frame_origin=position(0, 0), asset='emerald_ground', instance='emerald_ground',
            color=[.13, .29, .035], collision=True, world_material_coordinates=True,
            material=dict(pattern=dict(kind='noise', frequency=[.22, .22, .22],
                low=[.85, .90, .80], high=[1.06, 1.04, .99], bump=.002,
                warp=.2, octaves=3, seed=218, contrast=.7),
                reflectance=dict(roughness=.9, f0=.015))))
        c.call('save_terrain', path=rel(OUT / 'terrain.ignis-terrain'), document=doc)
        c.call('set_light', light=light())
        c.call('save_scene', path=rel(OUT / 'terrain.scene.json'))
        atomic_json(OUT / 'terrain-build.json', dict(build_id=state['build_id'], publication=result))
        atomic_json(OUT / 'player-template.json', player)
    finally:
        c.close()


def hash32(x):
    x ^= x >> 16
    x = (x * 0x7feb352d) & 0xffffffff
    x ^= x >> 15
    x = (x * 0x846ca68b) & 0xffffffff
    return x ^ (x >> 16)


def grid(scene):
    instance = next(i for i in scene['meshes']['instances'] if i['id'] == 'emerald_ground')
    asset = next(a for a in scene['meshes']['assets'] if a['id'] == instance['asset'])
    return Grid(instance, asset)


def leaf_templates():
    manifest = json.loads((DEST / 'library.json').read_text())
    built = json.loads((ROOT / 'artifacts/lawns/library-build.json').read_text())
    assert source_fingerprint(DEST, manifest) == built['source_sha256'], 'Regenerate the native lawn library first'
    library_path = ROOT / 'artifacts/lawns/library.scene.json'
    assert digest(library_path) == built['scene_sha256']
    native = json.loads(library_path.read_text())
    assets = {a['id']: a for a in native['meshes']['assets']}
    entry = next(e for e in manifest['entries'] if e['id'] == 'esmeralda_0')
    leaves = []
    for part in entry['components']:
        recipe = json.loads((DEST / part['path']).read_text())
        g = assets[part['prefix'] + '_leaves']['geometry']
        organs = recipe['source']['organs']
        # Source7 blade_samples=4 emits17 vertices per side. Verify the native
        # layout/root identity before using its top stations for thin ribbons.
        assert len(g['positions']) == 34 * len(organs)
        normals = [[0., 0., 0.] for _ in g['positions']]
        for a, b, c in g['triangles']:
            p, q, r = [g['positions'][i] for i in (a, b, c)]
            u, v = [q[i] - p[i] for i in range(3)], [r[i] - p[i] for i in range(3)]
            n = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
            for i in (a, b, c):
                for axis in range(3):
                    normals[i][axis] += n[axis]
        for j, organ in enumerate(organs):
            start = j * 34
            points = g['positions'][start:start + 17]
            assert math.dist(points[0], organ['path'][0]) < .0002
            ns = normals[start:start + 17]
            ns = [[x / max(math.sqrt(sum(v*v for v in n)), 1e-20) for x in n] for n in ns]
            color = recipe['settings']['leaf_color']
            tint = sum(a*b for a,b in zip(color,[.2126,.7152,.0722])) / sum(a*b for a,b in zip([.13,.29,.035],[.2126,.7152,.0722]))
            leaves.append((points, ns, tint))
    # Stable spatial selection spreads representatives across the square rather
    # than taking the first row of the source. These remain approximate scales.
    leaves.sort(key=lambda leaf: hash32(int((leaf[0][0][0] + .2) * 100000) ^
                                       (int((leaf[0][0][2] + .2) * 100000) << 12)))
    levels, metadata = [], []
    for count, longitudinal, across, stride in [(192, 4, 2, 1), (192, 2, 2, 1),
            (48, 2, 1, 1), (12, 2, 1, 2), (3, 2, 1, 4), (1, 2, 1, 8)]:
        raw = array('f')
        aux = array('f')
        def station(u, v):
            if u == 0:
                return 0
            if u == 4:
                return 16
            return 1 + (u - 1) * 5 + v
        for leaf_id, (points, normals, tint) in enumerate(leaves[:count]):
            for step in range(longitudinal):
                for cross in range(across):
                    a = station(step * 4 // longitudinal, cross * 4 // across)
                    b = station((step + 1) * 4 // longitudinal, cross * 4 // across)
                    c = station((step + 1) * 4 // longitudinal, (cross + 1) * 4 // across)
                    d = station(step * 4 // longitudinal, (cross + 1) * 4 // across)
                    for tri in [(a, b, c), (a, c, d)]:
                        if len(set(tri)) < 3:
                            continue
                        for i in tri:
                            p = points[i]
                            raw.extend([*p, 0., *normals[i], 0.])
                            mid = 0 if i == 0 else 16 if i == 16 else 3 + ((i - 1) // 5) * 5
                            aux.extend([*points[0], float(leaf_id), *points[mid], tint])
        levels.append((stride, raw))
        metadata.append(aux)
    return levels, metadata, 192, dict(manifest_sha256=digest(DEST / 'library.json'),
        native_library_sha256=digest(library_path), source_sha256=built['source_sha256'],
        source_entry=entry['id'])


def refresh_templates():
    """Replace only the six representations; preserve all compiled root heights."""
    path = OUT / 'ground.ignis-field'
    candidate = OUT / 'ground.ignis-field.candidate'
    levels, metadata, _, _ = leaf_templates()
    payload_hash = hashlib.sha256()
    try:
        with path.open('rb') as old, candidate.open('wb') as new:
            header = old.read(40)
            assert header[:8] == b'IGFLD2\0\0', 'Rebuild field to generate regular terrain samples'
            old.read(32)
            values = struct.unpack('<6f2I', header[8:])
            assert values[:3] == (-500., -500., SPACING) and values[-2:] == (TILES, TILES)
            for _ in range(6):
                _, count = struct.unpack('<2I', old.read(8))
                old.seek(count * 64, 1)
            new.write(header)
            new.write(struct.pack('<8f', .004, .015, .050, .16, .50, .30, .0032, .055))
            for (stride, vertices), aux in zip(levels, metadata):
                new.write(struct.pack('<2I', stride, len(vertices) // 8))
                if __import__('sys').byteorder != 'little':
                    vertices.byteswap()
                vertices.tofile(new)
                aux.tofile(new)
            count = 0
            while block := old.read(4 * 1024 * 1024):
                payload_hash.update(block)
                new.write(block)
                count += len(block)
            assert count == TILES * TILES * SIDE * SIDE * 4
        os.replace(candidate, path)
    finally:
        candidate.unlink(missing_ok=True)
    report = json.loads((OUT / 'compile.json').read_text())
    if 'height_payload_sha256' in report:
        assert report['height_payload_sha256'] == payload_hash.hexdigest()
    report.update(triangle_templates=[len(v) // 24 for _, v in levels],
        field_sha256=digest(path), field_bytes=path.stat().st_size,
        same_height_payload_preserved=True, height_payload_sha256=payload_hash.hexdigest())
    atomic_json(OUT / 'compile.json', report)


def prepare_field():
    scene = json.loads((OUT / 'terrain.scene.json').read_text())
    surface = grid(scene)
    assert surface.bounds == [-500., -500., 500., 500.]
    points = surface.points
    n = surface.width
    levels, metadata, blades, provenance = leaf_templates()
    heights = array('f')
    occupancy = [0] * 400
    def sample(x, z):
        x, z = (x + 500) / TERRAIN_STEP, (z + 500) / TERRAIN_STEP
        ix, iz = min(249, int(x)), min(249, int(z))
        u, v = x - ix, z - iz
        a = iz * n + ix
        ya, yb, yc, yd = [points[i][1] for i in (a, a + 1, a + n, a + n + 1)]
        return ya + (yb-ya)*u + (yc-ya)*v if u+v <= 1 else yd + (yc-yd)*(1-u) + (yb-yd)*(1-v)
    started = time.monotonic()
    for tz in range(TILES):
        for tx in range(TILES):
            for z in range(SIDE):
                for x in range(SIDE):
                    wx = -500 + (tx * SIDE + x + .5) * SPACING
                    wz = -500 + (tz * SIDE + z + .5) * SPACING
                    heights.append(sample(wx, wz))
                    occupancy[(tz * SIDE + z) // 400 * 20 + (tx * SIDE + x) // 400] += 1
        if tz % 25 == 24:
            print('field rows', tz + 1, '/', TILES, flush=True)
    assert len(heights) == 64_000_000 and all(n == 160000 for n in occupancy)
    candidate = OUT / 'ground.ignis-field.candidate'
    try:
        with candidate.open('wb') as f:
            f.write(b'IGFLD2\0\0')
            f.write(struct.pack('<6f2I', -500., -500., SPACING, .13, .29, .035, TILES, TILES))
            f.write(struct.pack('<8f', .004, .015, .050, .16, .50, .30, .0032, .055))
            for (stride, vertices), aux in zip(levels, metadata):
                f.write(struct.pack('<2I', stride, len(vertices) // 8))
                if __import__('sys').byteorder != 'little':
                    vertices.byteswap()
                vertices.tofile(f)
                aux.tofile(f)
            if __import__('sys').byteorder != 'little':
                heights.byteswap()
            heights.tofile(f)
        os.replace(candidate, OUT / 'ground.ignis-field')
    finally:
        candidate.unlink(missing_ok=True)
    x, z = -105., 160.
    y = surface.sample(x, z)[0]
    player = json.loads((OUT / 'player-template.json').read_text())
    player.update(name='Explorador da esmeralda', footsteps=None)
    player['transform'] = dict(position=[x, y + .94, z], size=[.6, 1.8, .6])
    player['character_controller'] = dict(max_slope_degrees=40, slide_slope_degrees=45, snap_distance=.2)
    scene['name'] = 'Esmeralda · colinas de 1 km × 1 km'
    scene['entities'] = [player]
    cx = x - 12
    cy = surface.sample(cx, z)[0]
    cameras = dict(
        panorama=dict(position=[660, 540, 720], target=[0, 13, 0], fov_degrees=58),
        hills=dict(position=[-160, surface.sample(-160, 260)[0] + 18, 260], target=[130, 16, -220], fov_degrees=62),
        walk=dict(position=[cx, cy + 1.65, z], target=[cx, surface.sample(cx, z - 20)[0] + .5, z - 20], fov_degrees=65),
        walk_down=dict(position=[cx, cy + 1.65, z], target=[cx, surface.sample(cx, z - 2.5)[0] + .02, z - 2.5], fov_degrees=65),
        close=dict(position=[cx + .29, cy + .20, z + .48], target=[cx, cy + .015, z + .02], fov_degrees=46))
    scene['camera'] = cameras['walk']
    scene['light'] = light([x, y, z])
    atomic_json(OUT / 'world.candidate.scene.json', scene)
    atomic_json(OUT / 'views.json', cameras)
    atomic_json(OUT / 'compile.json', dict(size_meters=[1000, 1000], area_m2=1_000_000,
        terrain_grid=[n, n], terrain_triangles=2 * (n - 1)**2,
        terrain_height_range=[min(p[1] for p in points), max(p[1] for p in points)],
        population=len(heights), logical_blades=len(heights) * blades, blades_per_module=blades,
        coverage_50m_cells=occupancy, exclusions=[], spacing=SPACING,
        triangle_templates=[len(v) // 24 for _, v in levels],
        field_sha256=digest(OUT / 'ground.ignis-field'), field_bytes=(OUT / 'ground.ignis-field').stat().st_size,
        compile_seconds=time.monotonic()-started, provenance=provenance,
        height_payload_sha256=hashlib.sha256(heights).hexdigest(),
        profile='IGFLD2 continuous per-blade placement and LOD',
        limitations=['Experimental session sidecar; SaveScene does not persist or bind it',
            'Approximate distant coverage; no image equivalence certificate',
            'Native raster remains single-sample; subpixel aliasing is not solved universally']))


def capture(binary):
    previous = os.environ.get('IGNIS_HORIZON_FIELD')
    previous_proxy = os.environ.pop('IGNIS_HORIZON_PROXIES', None)
    os.environ['IGNIS_HORIZON_FIELD'] = str(OUT / 'ground.ignis-field')
    try:
        c = Client(binary, log_path=OUT / 'native.log')
    finally:
        if previous is None:
            os.environ.pop('IGNIS_HORIZON_FIELD', None)
        else:
            os.environ['IGNIS_HORIZON_FIELD'] = previous
        if previous_proxy is not None:
            os.environ['IGNIS_HORIZON_PROXIES'] = previous_proxy
    try:
        init(c)
        build = c.call('inspect')['build_id']
        c.call('set_execution_mode', mode='continuous')
        c.call('load_scene', path=rel(OUT / 'world.candidate.scene.json'))
        cameras = json.loads((OUT / 'views.json').read_text())
        c.call('set_light', light=light(cameras['walk']['position']))
        c.call('save_scene', path=rel(OUT / 'world.scene.json'))
        c.call('step', ticks=45)
        start = c.call('inspect_world')['positions']['player']
        c.call('set_input', input=dict(x=0, z=-1))
        c.call('step', ticks=120)
        c.call('set_input', input=dict(x=0, z=0))
        end = c.call('inspect_world')['positions']['player']
        controllers = c.call('inspect_character_controllers')
        assert controllers['controllers']['player']['grounded'] and math.dist(start, end) > 3
        report = dict(build_id=build, walk_start=start, walk_end=end, walk_meters=math.dist(start, end),
            controllers=controllers, collision=c.call('inspect_mesh_collisions'), captures={})
        c.call('load_scene', path=rel(OUT / 'world.scene.json'))
        cameras = json.loads((OUT / 'views.json').read_text())
        lights = {}
        for name, camera in cameras.items():
            c.call('set_camera', camera=camera)
            center = camera['position'] if name in ('walk', 'close') else [-100, 12, 160]
            lights[name] = light(center)
            c.call('set_light', light=lights[name])
            metadata = c.call('capture', path=rel(OUT / (name + '.png')), width=1800, height=1100)
            report['captures'][name] = metadata
            print('captured', name, flush=True)
        c.call('save_scene', path=rel(OUT / 'roundtrip.scene.json'))
        c.call('load_scene', path=rel(OUT / 'roundtrip.scene.json'))
        c.call('capture', path=rel(OUT / 'roundtrip.png'), width=1800, height=1100)
        assert (OUT / 'close.png').read_bytes() == (OUT / 'roundtrip.png').read_bytes()
        report['save_load_png_exact_in_bound_session'] = True
        c.call('set_player_view', view=dict(actor='player', yaw_degrees=0, pitch_degrees=-28,
            eye_height=1.65, fov_degrees=65, mode='first_person'))
        c.call('step', ticks=45)
        report['captures']['player'] = c.call('capture', path=rel(OUT / 'player.png'), width=1800, height=1100)
        report['player_view'] = c.call('inspect_player_view')
        atomic_json(OUT / 'lights.json', lights)
        atomic_json(OUT / 'native-validation.json', report)
        atomic_json(OUT / 'bundle.json', dict(version=1, scene='world.scene.json',
            scene_sha256=digest(OUT / 'world.scene.json'), field='ground.ignis-field',
            field_sha256=digest(OUT / 'ground.ignis-field'), native_build_id=build,
            native_binary=str(binary.resolve()), native_binary_sha256=digest(binary),
            binding='IGNIS_HORIZON_FIELD session sidecar; use scripts/open_emerald_meadow.py'))
        review(cameras)
        for name in ['world.candidate.scene.json', 'roundtrip.scene.json']:
            (OUT / name).unlink(missing_ok=True)
    finally:
        c.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    p.add_argument('--stage', choices=['all', 'terrain', 'field', 'templates', 'capture'], default='all')
    args = p.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    if args.stage in ('all', 'terrain'):
        terrain(args.binary)
    if args.stage in ('all', 'field'):
        prepare_field()
    if args.stage == 'templates':
        refresh_templates()
    if args.stage in ('all', 'capture'):
        if not (OUT / 'world.candidate.scene.json').exists():
            atomic_json(OUT / 'world.candidate.scene.json', json.loads((OUT / 'world.scene.json').read_text()))
        capture(args.binary)


if __name__ == '__main__':
    main()
