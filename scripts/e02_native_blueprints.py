"""Generate native structural plant sources and draw their inspected projections.

No previous house recipe/scene is imported. The 2D sheets are a client viewer,
not a claimed native blueprint-image exporter. Requires Pillow for diagrams.
"""
import hashlib
import json
import math
from pathlib import Path
import subprocess
import shutil

from PIL import Image, ImageDraw, ImageFilter, ImageFont

PROJECT = Path(__file__).resolve().parents[1]
ROOT = PROJECT / 'artifacts/house-e02-blueprints'
BINARY = '/Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis'
SOURCE = Path('/Volumes/Zero/EpicGames/Unreal Projects/LandOfChaos/docs/art/settlements/residential-concepts-v2/E02.png')
CAMERA = {'position': [12, 11, 18], 'target': [-.2, 3.5, -.1], 'fov_degrees': 34}


def write(name, data):
    (ROOT / name).write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')


class MCP:
    def __init__(self):
        ROOT.mkdir(parents=True, exist_ok=True)
        self.log = (ROOT / 'stderr.log').open('w')
        self.p = subprocess.Popen([BINARY, '--root', str(ROOT), '--mcp'], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=self.log, text=True)
        self.seq = 0
        self.revision = None
        write('initialize.json', self.rpc('initialize', {'protocolVersion': '2024-11-05',
              'capabilities': {}, 'clientInfo': {'name': 'native-blueprint-study', 'version': '1'}}))
        self.p.stdin.write(json.dumps({'jsonrpc': '2.0', 'method': 'notifications/initialized'}) + '\n')
        self.p.stdin.flush()

    def rpc(self, method, params):
        self.seq += 1
        self.p.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': self.seq, 'method': method,
                                      'params': params}, allow_nan=False) + '\n')
        self.p.stdin.flush()
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError('MCP closed; see stderr.log')
            r = json.loads(line)
            if r.get('id') == self.seq:
                if 'error' in r:
                    raise RuntimeError(r['error'])
                return r['result']

    def command(self, op, **kw):
        args = {'command': {'op': op, **kw}}
        if self.revision is not None:
            args['expected_revision'] = self.revision
        r = self.rpc('tools/call', {'name': 'engine_command', 'arguments': args})
        if r.get('isError'):
            raise RuntimeError(r)
        data = r.get('structuredContent')
        if data is None:
            data = json.loads(next(c['text'] for c in r['content'] if c['type'] == 'text'))
        if 'revision' in data:
            self.revision = data['revision']
        return data

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=30)
        self.log.close()


def definition(id, w, h, depth, stations=None, cross='rectangle', controls=None, segments=None):
    d = {'id': id, 'profile': {'id': id+'-profile', 'width': w, 'height': h},
         'depth': depth, 'segments': segments or [1, 1, 1]}
    if stations or cross != 'rectangle':
        stations = stations or [(0, 1, 1), (1, 1, 1)]
        d['sections'] = {'id': id+'-sections', 'cross_section': cross,
                         'stations': [{'id': 's'+str(i), 'at': at, 'width': width, 'depth': dep}
                                      for i, (at, width, dep) in enumerate(stations)]}
    if controls:
        # Parallel station planes are appropriate for a thin roof extrusion.
        # A normal-following trajectory rejects these bends in the current build.
        # Keep the requested curve in explicit native offsets instead of flattening it.
        points = []
        for i in range(13):
            t = i/12
            weights = [(1-t)**3, 3*(1-t)**2*t, 3*(1-t)*t*t, t**3]
            offset = [sum(weights[j]*controls[j][axis] for j in range(4)) for axis in range(2)]
            points.append({'id': 's'+str(i), 'at': t, 'width': 1, 'depth': 1, 'offset': offset})
        d['sections'] = {'id': id+'-sections', 'cross_section': 'rectangle',
                         'interpolation': 'pchip', 'stations': points}
        d['segments'] = [2, 2, 1]
    return d


def recipe():
    sources = []
    instances = []
    meta = {}
    stone = [.67, .61, .49]
    roof = [.20, .28, .23]
    # Local Y -> world X; local X -> world Z; local Z -> world Y.
    roof_rotation = [-.5, -.5, -.5, .5]
    rail_rotation = [0, 0, -math.sqrt(.5), math.sqrt(.5)]

    def source(d, title, role):
        sources.append(d)
        meta[d['id']] = {'title': title, 'role': role}
        return d['id']

    def place(asset, p, color=stone, rotation=None, scale=1):
        i = sum(x['asset'] == asset for x in instances)
        instances.append({'id': asset+'-'+str(i), 'asset': asset, 'position': p,
                          'rotation': rotation or [0, 0, 0, 1], 'scale': scale, 'color': color})

    floor = source(definition('bp-patio', 12, .45, 8.6, cross='ellipse', segments=[6, 1, 6]),
                   '01 Pátio elevado', 'floor')
    place(floor, [0, .375, .3])
    wing = source(definition('bp-wing-wall', 6.4, 2.7, .25), '02 Parede da ala', 'wall')
    place(wing, [-2.3, 1.95, -.9])
    side = source(definition('bp-main-side', .25, 3.9, 5.3), '03 Lateral do corpo alto', 'wall')
    place(side, [4.9, 2.55, -1.2])
    pier = source(definition('bp-main-pier', .42, 4.4, .42), '04 Pilares da fachada alta', 'wall')
    for x in [1.1, 4.9]:
        place(pier, [x, 2.8, 1.45])
    column = source(definition('bp-column', .48, 2.8, .48,
                    stations=[(0, 1, 1), (.07, 1, 1), (.1, .8, .8), (.16, .5, .5),
                              (.82, .5, .5), (.88, .7, .7), (.94, .9, .9), (1, 1, 1)]),
                    '05 Coluna: base / fuste / capitel', 'column')
    for x in [-5.15, -3.35, -1.55, .25]:
        place(column, [x, 2, 1.65])
    for id, title, w, span, controls, pos in [
        ('bp-main-roof', '06 Cobertura principal: pano direito', 5.7, 4.75,
         [[0, 2.6], [0, .7], [0, -2.8], [0, -2.45]], [3.05, 6.15, -1.35]),
        ('bp-main-back-roof', '07 Cobertura principal: pano esquerdo', 5.7, 1.4,
         [[0, -1.9], [0, -.7], [0, 1.7], [0, 2.6]], [-.025, 6.15, -1.35]),
        ('bp-wing-roof', '08 Cobertura superior da ala', 3.5, 7.2,
         [[0, .35], [0, -.5], [0, -.4], [0, .55]], [-2.35, 4.9, -1.8]),
        ('bp-porch-roof', '09 Cobertura baixa da varanda', 2.8, 7.4,
         [[0, .45], [0, -.35], [0, -.2], [0, .25]], [-2.45, 3.7, .9]),
    ]:
        asset = source(definition(id, w, span, .13, controls=controls, segments=[2, 28, 1]), title, 'roof')
        place(asset, pos, roof, roof_rotation)
    arch = source(definition('bp-window-arch', .16, 3.6, .24,
                  controls=[[-1.5, 0], [-1.5, 0], [-.8, 0], [0, 0]], segments=[1, 28, 1]),
                  '10 Meia moldura curva da janela', 'opening')
    place(arch, [3, 2.95, 1.48])
    place(arch, [3, 2.95, 1.48], rotation=[0, 1, 0, 0])
    door = source(definition('bp-door-profile', 1.1, 2.3, .08,
                  stations=[(0, 1, 1), (.6, 1, 1), (.82, .76, 1), (.94, .43, 1), (1, .04, 1)]),
                  '11 Contorno das portas; motivo pendente', 'opening')
    for x in [-4.75, .1]:
        place(door, [x, 1.75, -.73], [.31, .19, .09])
    for id, title, span, controls, x in [
        ('bp-left-rail', '12 Travessa curva esquerda', 5,
         [[0, 0], [0, 1.7], [0, 2.5], [0, 2.5]], -2.75),
        ('bp-right-rail', '13 Travessa curva direita', 2.4,
         [[0, 2.5], [0, 2.5], [0, 1.6], [0, .4]], 3.1),
    ]:
        a = source(definition(id, .15, span, .15, controls=controls, segments=[1, 24, 1]), title, 'rail')
        for y in [.68, 1.53]:
            place(a, [x, y, 1.1], rotation=rail_rotation)
    baluster = source(definition('bp-rail-post', .20, .85, .20), '14 Apoio da balaustrada', 'rail')
    for x, z in [(-4.8, 1.5), (-3, 2.9), (-.5, 3.6), (2, 3.6), (3.1, 3.15), (4.25, 1.7)]:
        place(baluster, [x, 1.10, z])
    step = source(definition('bp-entry-step', 3.8, .13, 1.7, cross='ellipse', segments=[6, 1, 6]),
                  '15 Degrau elíptico do acesso', 'step')
    for i, scale in enumerate([1.08, .99, .9]):
        place(step, [.8, .12+i*.13, 4.2-i*.12], scale=scale)
    dormer = source(definition('bp-dormer', 1.1, 1.6, 1.3,
                    stations=[(0, 1, 1), (.55, 1, 1), (1, .04, 1)]),
                    '16 Volume da água-furtada; vão pendente', 'roof')
    place(dormer, [4.5, 5.25, -2.1], roof)
    return sources, instances, meta


def rotate(v, q):
    x, y, z, w = q
    a, b, c = v
    uv = [y*c-z*b, z*a-x*c, x*b-y*a]
    uuv = [y*uv[2]-z*uv[1], z*uv[0]-x*uv[2], x*uv[1]-y*uv[0]]
    return [v[i]+2*(w*uv[i]+uuv[i]) for i in range(3)]


def font(size):
    return ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf', size)


def diagram(geometries, axes, size, bounds=None):
    """Filled projected triangles and mask-derived outlines of inspected geometry."""
    w, h = size
    points = [p for g in geometries for p in g['positions']]
    if not points:
        raise ValueError('No inspected geometry for diagram')
    if bounds is None:
        bounds = [min(p[axes[0]] for p in points), min(p[axes[1]] for p in points),
                  max(p[axes[0]] for p in points), max(p[axes[1]] for p in points)]
    x0, y0, x1, y1 = bounds
    scale = min((w-50)/max(.01, x1-x0), (h-50)/max(.01, y1-y0))
    def xy(p):
        return ((w-(x1-x0)*scale)/2+(p[axes[0]]-x0)*scale,
                h-(h-(y1-y0)*scale)/2-(p[axes[1]]-y0)*scale)
    image = Image.new('RGB', size, '#102b47')
    draw = ImageDraw.Draw(image)
    for x in range(0, w, 32): draw.line((x, 0, x, h), fill='#193954')
    for y in range(0, h, 32): draw.line((0, y, w, y), fill='#193954')
    for g in geometries:
        mask = Image.new('L', size, 0)
        md = ImageDraw.Draw(mask)
        for t in g['triangles']:
            md.polygon([xy(g['positions'][i]) for i in t], fill=255)
        image.paste('#315979', (0, 0), mask)
        edge = ImageChops_subtract(mask.filter(ImageFilter.MaxFilter(3)), mask.filter(ImageFilter.MinFilter(3)))
        image.paste('#d3edf8', (0, 0), edge)
    return image


def ImageChops_subtract(a, b):
    from PIL import ImageChops
    return ImageChops.subtract(a, b)


def sheets(plants, meshes, instances, meta):
    # The only geometry input is the asset read from inspect_plant, not a second drawing recipe.
    geometries = {id: asset['geometry'] for id, asset in meshes.items()}
    sheet = Image.new('RGB', (1920, 1920), '#102b47')
    draw = ImageDraw.Draw(sheet)
    draw.text((35, 22), 'E02 / FONTES PARAMÉTRICAS NATIVAS', font=font(30), fill='#e2f3fa')
    draw.text((35, 65), 'Malhas de inspect_mesh; fontes de inspect_plant. Escalas locais distintas por quadro.',
              font=font(19), fill='#9cbdd4')
    for i, (id, p) in enumerate(plants.items()):
        p = p['plant']
        x = (i % 4)*480
        y = 112+(i//4)*445
        g = geometries[id]
        # Curves in YZ for roofs, XY for arches/columns, YZ for plan-curved rails.
        role = meta[id]['role']
        curved = any('offset' in s for s in p['definition'].get('sections', {}).get('stations', []))
        axes = (1, 2) if role in ['roof', 'rail'] and curved else (0, 1)
        if role in ['floor', 'step']: axes = (0, 2)
        panel = diagram([g], axes, (450, 305))
        sheet.paste(panel, (x+15, y+32))
        draw.text((x+18, y), meta[id]['title'], font=font(18), fill='#e2f3fa')
        d = p['definition']
        dims = f"W {d['profile']['width']:g}  H {d['profile']['height']:g}  D {d['depth']:g}"
        draw.text((x+18, y+347), dims+'  / unidades de trabalho', font=font(16), fill='#9cbdd4')
        draw.text((x+18, y+374), f"{len(g['positions'])} vértices / {id}", font=font(15), fill='#9cbdd4')
    sheet.save(ROOT/'sources.png')
    world = []
    for instance in instances:
        g = geometries[instance['asset']]
        points = []
        for v in g['positions']:
            rv = rotate([a*instance['scale'] for a in v], instance['rotation'])
            points.append([rv[i]+instance['position'][i] for i in range(3)])
        world.append({'id': instance['id'], 'asset': instance['asset'], 'positions': points,
                      'triangles': g['triangles']})
    board = Image.new('RGB', (1800, 1400), '#102b47')
    dr = ImageDraw.Draw(board)
    dr.text((35, 20), 'E02 / IMPLANTAÇÃO E ELEVAÇÕES DA HIPÓTESE ESTRUTURAL', font=font(29), fill='#e2f3fa')
    dr.text((35, 66), 'Fonte única: plantas e instâncias publicadas no MCP. Sem cômodos/interior inferidos.', font=font(20), fill='#9cbdd4')
    layout = [g for g in world if meta[g['asset']]['role'] != 'roof']
    board.paste(diagram(layout, (0, 2), (850, 610)), (25, 145))
    board.paste(diagram(world, (0, 1), (850, 610)), (915, 145))
    roofs = [g for g in world if meta[g['asset']]['role'] == 'roof']
    board.paste(diagram(roofs, (0, 2), (850, 500)), (25, 835))
    board.paste(diagram(world, (2, 1), (850, 500)), (915, 835))
    for x, y, title in [(35, 112, '01 / Vista superior sem coberturas — X/Z'),
                         (925, 112, '02 / Elevação frontal — X/Y'),
                         (35, 792, '03 / Projeção das coberturas — X/Z'),
                         (925, 792, '04 / Elevação lateral — Z/Y')]:
        dr.text((x, y), title, font=font(22), fill='#e2f3fa')
    dr.text((35, 1360), 'Estudo estrutural incompleto. Ângulos, profundidades e fundos são hipóteses; não é planta métrica da referência.',
            font=font(20), fill='#f6c979')
    board.save(ROOT/'blueprints.png')


def main():
    sources, instances, meta = recipe()
    write('definitions.json', {'definitions': sources, 'instances': instances, 'metadata': meta,
          'source_sha256': hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
          'stage': 'structural hypotheses; not accepted as reference reconstruction'})
    mcp = MCP()
    try:
        write('schema.json', mcp.rpc('tools/list', {}))
        limits = mcp.command('inspect_meshes')['limits']
        write('limits.json', limits)
        state = mcp.command('inspect')
        edits = [{'op': 'remove', 'id': e['id']} for e in state['scene']['entities'] if e['id'] != 'player']
        edits.append({'op': 'upsert', 'entity': {'id': 'player', 'name': 'outside study',
            'transform': {'position': [35, 1, 35], 'size': [.7, 1.6, .7]}, 'body': 'player',
            'visible': False, 'color': [.5, .5, .5]}})
        mcp.command('edit_scene', edits=edits)
        previews = {}
        for d in sources:
            edit = {'kind': 'put', 'definition': d, 'detail_policy': 'physical',
                    'animation_translation_policy': 'physical'}
            try:
                previews[d['id']] = mcp.command('preview_plant', edit=edit)
            except RuntimeError as error:
                write('refused-source.json', {'definition': d, 'error': str(error)})
                raise RuntimeError('Native plant refused: '+d['id']) from error
            write('previews.json', previews)
            mcp.command('edit_plant', edit=edit)
        write('previews.json', previews)
        mcp.command('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': i}}
                                       for i in instances])
        plants = {d['id']: mcp.command('inspect_plant', id=d['id']) for d in sources}
        meshes = {d['id']: mcp.command('inspect_mesh', asset=d['id'])['asset'] for d in sources}
        write('plants.json', plants)
        write('meshes.json', meshes)
        write('counts.json', mcp.command('inspect_meshes'))
        mcp.command('set_camera', camera=CAMERA)
        write('capture.json', mcp.command('capture', path='render.png', width=1402, height=1122))
        shutil.copyfile(SOURCE, ROOT/'reference.png')
        regions = [
            {'id': 'roofs', 'x': 70, 'y': 10, 'width': 1300, 'height': 545},
            {'id': 'porch', 'x': 95, 'y': 460, 'width': 755, 'height': 295},
            {'id': 'great-window', 'x': 830, 'y': 350, 'width': 320, 'height': 410},
            {'id': 'courtyard', 'x': 120, 'y': 680, 'width': 1040, 'height': 360},
        ]
        write('comparison.json', mcp.command('compare_images', reference_path='reference.png',
              candidate_path='render.png', regions=regions, difference_path='difference.png'))
        for region in regions:
            x, y, w, h = [region[k] for k in ['x', 'y', 'width', 'height']]
            for prefix, filename in [('reference', 'reference.png'), ('candidate', 'render.png')]:
                with Image.open(ROOT/filename) as im:
                    im.crop((x, y, x+w, y+h)).save(ROOT/(region['id']+'-'+prefix+'.png'))
        write('regions.json', regions)
        mcp.command('capture', path='repeated.png', width=1402, height=1122)
        mcp.command('save_scene', path='house.scene.json')
        mcp.command('load_scene', path='house.scene.json')
        mcp.command('capture', path='reloaded.png', width=1402, height=1122)
        expected = hashlib.sha256((ROOT/'render.png').read_bytes()).hexdigest()
        checks = {'repeat_pixels': hashlib.sha256((ROOT/'repeated.png').read_bytes()).hexdigest() == expected,
                  'reload_pixels': hashlib.sha256((ROOT/'reloaded.png').read_bytes()).hexdigest() == expected,
                  'all_sources_native': all(plants[d['id']]['plant']['definition']['id'] == d['id'] for d in sources),
                  'source_equals_evaluated_mesh': all(plants[id]['plant']['source']['geometry'] == a['geometry'] for id, a in meshes.items()),
                  'scope': 'static structural study'}
        write('verification.json', checks)
        if any(v is not True for k, v in checks.items() if k != 'scope'): raise AssertionError(checks)
        print(json.dumps({'plant_keys': {k: list(v) for k, v in list(plants.items())[:1]}, 'checks': checks}))
    finally:
        mcp.close()
    sheets(plants, meshes, instances, meta)
    summary = {'definitions': [plants[d['id']]['plant']['definition'] for d in sources],
               'instances': instances, 'metadata': meta, 'verification': checks,
               'scope': 'Native structural sources; omitted details remain pending, no fidelity acceptance'}
    (PROJECT/'docs/references/image-to-3d/e02-native-blueprints.json').write_text(
        json.dumps(summary, indent=2, ensure_ascii=False)+'\n')


if __name__ == '__main__':
    main()
