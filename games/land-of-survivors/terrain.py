#!/usr/bin/env python3
"""Author a bounded original relief study; native Ignis evaluates all terrain."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import sys
import time

ROOT = Path(__file__).resolve().parent
ENGINE = ROOT.parents[1]
sys.path.insert(0, str(ENGINE / 'scripts'))
import verify_lab

RADIUS = 6_371_000


def direction(x, z):
    distance = math.hypot(x, z)
    if not distance:
        return [0, 1, 0]
    angle = distance / RADIUS
    return [x / distance * math.sin(angle), math.cos(angle), z / distance * math.sin(angle)]


def location(x, z):
    values = [v * RADIUS for v in direction(x, z)]
    cells = [math.floor(v / 1024) for v in values]
    return {'cell': [str(c) for c in cells],
            'offset': [v - c * 1024 for v, c in zip(values, cells)]}


def draft_for(recipe, terraced):
    nodes = []

    def add(kind, **fields):
        nodes.append({'kind': kind, **fields})
        return len(nodes) - 1

    def binary(kind, a, b, **fields):
        return add(kind, a=a, b=b, **fields)

    def remap(input_slot, pairs):
        return add('remap', input=input_slot,
                   knots=[{'x': x, 'value': value, 'derivative': 0} for x, value in pairs])

    support = add('raise', center=location(0, -1600),
                  radius_meters=recipe['support_radius_meters'], amount_meters=1)
    noise = add('noise', seed=recipe['seed'], period_meters=1024, octaves=2,
                persistence=.5, amplitude_meters=7, style='smooth')
    plain = binary('multiply', support, noise)
    ridge = None
    for chain in recipe['chains']:
        curve = add('curve', points=[location(*point) for point in chain['points']],
                    width_meters=chain['width'], height_meters=chain['height'],
                    left_exponent=chain['left_exponent'], right_exponent=chain['right_exponent'],
                    roundness=.8)
        for peak in chain.get('peaks', []):
            accent = add('raise', center=location(*peak['center']),
                         radius_meters=peak['radius'], amount_meters=peak['height'])
            curve = binary('add', curve, accent)
        envelope = add('raise', center=location(*chain['envelope_center']),
                       radius_meters=chain['envelope_radius'], amount_meters=1)
        curve = binary('multiply', curve, envelope)
        if terraced and chain['name'] == 'eastern_shelves':
            curve = remap(curve, recipe['terraces'])
        ridge = curve if ridge is None else binary('smooth_maximum', ridge, curve, transition_meters=18)
    terrain = binary('add', plain, ridge)
    # A flat core and a narrow feathered edge are authored with the common graph.
    # Level alone has a radial soft falloff and no independently controlled core.
    shelf = recipe['shelf']
    mask = add('raise', center=location(*shelf['center']), radius_meters=shelf['radius'], amount_meters=1)
    mask = remap(mask, [(0, 0), (.35, 0), (.65, 1), (1, 1)])
    target = add('constant', value=shelf['height'])
    terrain = add('mix', a=terrain, b=target, mask=mask)
    for name in ('valley', 'side_pass'):
        channel = recipe[name]
        terrain = add('channel', input=terrain, points=[location(*p) for p in channel['points']],
                      width_meters=channel['width'], bed_start_meters=channel['bed_start'],
                      bed_end_meters=channel['bed_end'], blend=1)
    return {'nodes': nodes, 'output': terrain}


def init(client):
    client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                   'clientInfo': {'name': 'los-terrain-study', 'version': '1'}})
    client.request('notifications/initialized', notification=True)


def write_review(out, report):
    rows = []
    for name, title in [('ground', 'Vale ao nível do chão'), ('west', 'Vista da crista oeste'),
                        ('east', 'Vista da encosta leste'), ('oblique', 'Vista oblíqua'),
                        ('aerial', 'Vista do alto')]:
        cards = []
        for variant, label in [('smooth', 'Controle suave'), ('terraced', 'Patamares regionais')]:
            detail = report['variants'].get(variant)
            if detail is None:
                continue
            frame = next(c for c in detail['captures'] if c['name'] == name)
            state = frame['view']
            limited = 'Limite de detalhe atingido' if state['quality_limited'] else 'Bound solicitado atendido'
            cards.append(f'<figure><a href="{variant}-{name}.png"><img src="{variant}-{name}.png" alt="{label}: {title}"></a>'
                         f'<figcaption><b>{label}</b> · {limited}<br>{state["resident_tiles"]} tiles · '
                         f'bound {state["maximum_geometric_bound_pixels"]:.2f} px</figcaption></figure>')
        rows.append(f'<section><h2>{title}</h2><div class="pair">{"".join(cards)}</div></section>')
    diagnostic = ''
    if (out / 'grid-limit.png').is_file():
        diagnostic = '<section><h2>Limite encontrado no primeiro ensaio</h2><p>Perfil com faixas estreitas aplicado a todas as cristas, em grade mais grossa. A imagem registra o serrilhado observado; composição e resolução mudaram nas versões acima, portanto esta comparação não isola uma causa única.</p><a href="grid-limit.png"><img src="grid-limit.png" alt="Serrilhado do primeiro ensaio"></a></section>'
    page = f'''<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Land of Survivors — estudo de relevo</title><style>
body{{background:#171a1b;color:#e5e7e8;font:16px/1.5 system-ui;margin:0 auto;max-width:1500px;padding:32px}}
h1,h2{{line-height:1.2}}p{{max-width:95ch;color:#bcc5c8}}.pair{{display:grid;grid-template-columns:1fr 1fr;gap:20px}}
figure{{margin:0;background:#242a2d}}img{{display:block;width:100%;height:auto}}figcaption{{padding:12px}}section{{margin:36px 0}}a{{color:#bbdbe4}}code{{word-break:break-all}}@media(max-width:800px){{.pair{{grid-template-columns:1fr}}}}
</style><h1>Land of Survivors — estudo de relevo</h1><p>Geografia original em planeta de 12.742 km de diâmetro. Duas composições nas mesmas câmeras, com material neutro. Capturas do renderer Ignis, sem imagens geradas ou vegetação. Clique para examinar cada imagem.</p>
<p>O bound é um limite conservador da seleção geométrica, não erro medido dos pixels nem FPS. As vistas de chão ainda esgotam o orçamento de detalhe. O aceite artístico permanece aberto.</p>
{''.join(rows)}{diagnostic}<p>Relatório completo: <a href="verification.json">verification.json</a>. Build: <code>{report['build_id']}</code>.</p></html>'''
    (out / 'review.html').write_text(page)


def prepare(binary, variants, publish):
    out = ROOT / 'artifacts/terrain'
    out.mkdir(parents=True, exist_ok=True)
    recipe = json.loads((ROOT / 'world/terrain.recipe.json').read_text())
    verify_lab.ROOT = ROOT
    client = verify_lab.Client(binary, log_path=out / 'runtime.log')
    report = {'variants': {}, 'human_visual_acceptance': False, 'fps_verified': False}
    try:
        init(client)
        client.call('open_project', path=str(ROOT))
        client.call('open_project_scene', path='scenes/Principal.scene.json')
        snapshot = client.call('inspect')
        report['build_id'] = snapshot['build_id']
        scene = copy.deepcopy(snapshot['scene'])
        scene['light']['atmosphere']['density'] = .0016
        scene['light']['atmosphere']['reduce_details'] = False
        scene['light']['ambient'] = .25
        client.call('set_execution_mode', mode='continuous')
        winning = None
        for variant in variants:
            started = time.monotonic()
            print(json.dumps({'preparing': variant}), flush=True)
            draft = draft_for(recipe, variant == 'terraced')
            program = client.call('query_planet', query={'kind': 'prepare_surface', 'draft': draft})['program']
            doc = json.loads((ROOT / 'world/sphere.planet.json').read_text())
            doc.update(revision=2, operations=[{'kind': 'surface', 'program': program}])
            config = json.loads((ROOT / 'world/runtime.json').read_text())
            config.update(document=doc, level=15, subdivisions=32, physical_radius_meters=128,
                          color=[.47, .5, .45], material={}, compile_distant_meshes=False)
            observer_direction = direction(*recipe['observer_site_meters'])
            observer_height = client.call('query_planet', query={
                'kind': 'sample', 'document': doc, 'directions': [observer_direction]})['samples'][0]['height_meters']
            config['spawn'] = {'direction': observer_direction, 'forward': [0, 0, -1],
                               'altitude_meters': observer_height + .91}
            positions = [(x, z) for z in range(500, -4501, -125) for x in range(-2500, 2501, 125)]
            samples = client.call('query_planet', query={
                'kind': 'sample', 'document': doc,
                'directions': [direction(x, z) for x, z in positions]})['samples']
            heights = [s['height_meters'] for s in samples]
            detail = {'nodes': len(draft['nodes']), 'height_range_meters': [min(heights), max(heights)],
                      'profiles': [{'x': x, 'z': z, 'height': sample['height_meters']}
                                   for (x, z), sample in zip(positions, samples)], 'captures': []}
            client.call('set_planet_view', view=None)
            client.call('set_planet_runtime', config=None)
            client.call('load_scene', path='scenes/Principal.scene.json')
            client.call('set_light', light=scene['light'])
            client.call('set_planet_runtime', config=config)
            client.call('step', ticks=120)
            baseline = client.call('inspect_planet_runtime')
            detail['start'] = baseline
            if not baseline['controller']['grounded']:
                raise ValueError(f'Observer not grounded: {baseline}')
            views = [
                ('ground', dict(distance_meters=0, yaw_degrees=0, tilt_degrees=4)),
                ('west', dict(distance_meters=0, yaw_degrees=-48, tilt_degrees=8)),
                ('east', dict(distance_meters=0, yaw_degrees=48, tilt_degrees=8)),
                ('oblique', dict(distance_meters=1400, yaw_degrees=-20, tilt_degrees=42)),
                ('aerial', dict(distance_meters=2600, yaw_degrees=0, tilt_degrees=15)),
            ]
            for name, view in views:
                settings = dict(viewport=[1280, 800], max_tiles=1536, max_error_pixels=2,
                                incremental_generation=False, atmosphere_optical_depth=.18,
                                atmosphere_height_meters=8000, **view)
                t = time.monotonic()
                client.call('set_planet_view', view=settings)
                path = f'artifacts/terrain/{variant}-{name}.png'
                client.call('capture', path=path, width=1280, height=800)
                state = client.call('inspect_planet_view')
                detail['captures'].append({'name': name, 'path': path, 'view': state,
                                           'cold_command_seconds': time.monotonic() - t})
                print(json.dumps({'captured': f'{variant}-{name}', 'tiles': state['resident_tiles'],
                                  'limited': state['quality_limited']}), flush=True)
            if client.call('inspect_planet_runtime') != baseline:
                raise ValueError('Inspection changed physical observer state')
            source_path = f'artifacts/terrain/{variant}.ignis-planet'
            client.call('save_planet', path=source_path, document=doc)
            if client.call('load_planet', path=source_path)['document'] != doc:
                raise ValueError('Native source roundtrip changed relief')
            detail['source_roundtrip'] = True
            if variant == 'terraced':
                client.call('set_planet_view', view=None)
                client.call('set_player_view', view={
                    'actor': 'player', 'yaw_degrees': 0, 'pitch_degrees': 0,
                    'eye_height': 1.65, 'fov_degrees': 60, 'mode': 'first_person'})
                client.call('set_player_view_input', input={'x': 0, 'z': -1, 'mode': 'run'})
                client.call('step', ticks=600)
                client.call('set_player_view_input', input={'x': 0, 'z': 0})
                client.call('step', ticks=120)
                walked = client.call('inspect_planet_runtime')
                detail['walk'] = walked
                if not walked['controller']['grounded'] or walked['distance_from_spawn_meters'] < 40:
                    raise ValueError('The native observer failed the short valley traversal')
            detail['seconds'] = time.monotonic() - started
            report['variants'][variant] = detail
            if variant == 'terraced':
                winning = (doc, config, scene)
            (out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        if publish:
            if winning is None:
                raise ValueError('Publication requires the terraced candidate')
            doc, config, scene = winning
            client.call('set_planet_view', view=None)
            client.call('set_planet_runtime', config=None)
            client.call('load_scene', path='scenes/Principal.scene.json')
            client.call('set_light', light=scene['light'])
            client.call('save_scene', path='scenes/Principal.scene.json')
            client.call('save_planet', path='world/sphere.ignis-planet', document=doc)
            (ROOT / 'world/sphere.planet.json').write_text(json.dumps(doc, indent=2) + '\n')
            (ROOT / 'world/runtime.json').write_text(json.dumps(config, indent=2) + '\n')
            regional_view = dict(distance_meters=1400, yaw_degrees=-20, tilt_degrees=42,
                                 viewport=[1280, 800], max_tiles=1536, max_error_pixels=2,
                                 incremental_generation=True, atmosphere_optical_depth=.18,
                                 atmosphere_height_meters=8000)
            (ROOT / 'world/view.json').write_text(json.dumps(regional_view, indent=2) + '\n')
            report['published'] = True
        report['binary_sha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
        (out / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
        write_review(out, report)
        print(json.dumps({'complete': True, 'published': bool(publish)}), flush=True)
    finally:
        client.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--publish', action='store_true')
    p.add_argument('--variant', choices=['both', 'smooth', 'terraced'], default='both')
    p.add_argument('--binary', type=Path, default=ENGINE / 'target/release/ignis')
    a = p.parse_args()
    if a.publish and a.variant == 'smooth':
        p.error('--publish requires --variant terraced or both')
    prepare(a.binary.resolve(), ['smooth', 'terraced'] if a.variant == 'both' else [a.variant], a.publish)
