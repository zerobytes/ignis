#!/usr/bin/env python3
"""Original birch study: seeded native organs/materials, never game meshes.

Reference inventory: docs/genshin-birch-analysis.md. Python authors the graph;
the existing native Ignis botany/material compilers produce all output meshes.
"""
import argparse
import json
import math
from pathlib import Path

import botany_math as b
from botany_exemplars import Builder, curved, radius_at
from groundcover_assets import ROOT, atomic_json

DEST = ROOT / 'assets/botany/genshin-birch'
RECIPE = ROOT / 'examples/genshin-birch.recipe.json'


def graph(cfg, variant, index):
    seed = cfg['seed'] + index * 1009
    x = Builder('Birch study · ' + variant['name'], seed)
    x.axis_samples = cfg['axis_samples']
    height, reach = variant['height_m'], variant['reach_m']
    lean = variant['lean']
    curved(x, 'trunk', None, 0., (lean[0], height * .97, lean[1]),
           (0., 1., 0.), variant['trunk_radius_m'], tip=.075,
           end_direction=(lean[0] * .2, 1., lean[1] * .2), arch=0.,
           noise=.055, taper=1.15)
    tips = []
    inner_sprays = []
    core_shoots = []

    def fork(host, depth, azimuth, lobe_radius):
        if depth == cfg['fork_depth']:
            tips.append((host, azimuth))
            return
        f = x.sample(host, 1.)
        for side in range(2):
            ident = f'{host}/f{side}'
            a = azimuth + (1 if side else -1) * (.63 + .32 * b.uniform(seed, ident, 'splay'))
            length = max(.24, lobe_radius * (.46 + .17 * b.uniform(seed, ident, 'fork_length')))
            direction = b.unit((math.cos(a), .60 + .75 * b.uniform(seed, ident, 'lift'), math.sin(a)))
            direction = b.unit(b.lerp(f.t, direction, .70))
            endpoint = b.add(f.p, b.mul(direction, length))
            radius = radius_at(x, host, 1.) * .69
            curved(x, ident, host, 1., endpoint, direction, radius,
                   tip=.79 if depth + 1 < cfg['fork_depth'] else .25,
                   arch=.025, noise=.014, taper=1.15)
            x.by_id[ident]['connection'] = 'fork'
            if depth < 2:
                inner_sprays.append((ident, a))
            fork(ident, depth + 1, a, lobe_radius * .74)

    n = cfg['primary_branches']
    for j in range(n):
        v = j / (n - 1)
        ident = f'branch{j:02d}'
        # No identical rings: golden-angle insertion, independent height and reach.
        u = variant['crown_base'] + (.945 - variant['crown_base']) * v
        u += .012 * (b.uniform(seed, ident, 'height') - .5)
        f = x.sample('trunk', u)
        a = j * 2.3999632297 + variant['phase'] + .36 * (b.uniform(seed, ident, 'azimuth') - .5)
        spread = reach * .68 * (1. - .82 * v ** 1.25) * (.84 + .25 * b.uniform(seed, ident, 'reach'))
        lift = -.08 + .13 * (1. - v) + .18 * b.uniform(seed, ident, 'upturn')
        endpoint = b.add(f.p, (spread * math.cos(a), lift, spread * math.sin(a)))
        r = min(radius_at(x, 'trunk', u) * .46, .052 * (1. - .69 * v))
        curved(x, ident, 'trunk', u, endpoint, (math.cos(a), .48, math.sin(a)),
               r, end_direction=(math.cos(a) * .52, 1., math.sin(a) * .52),
               tip=.42, arch=.07, noise=.035, taper=1.25)
        fork(ident, 0, a, .70 * (1. - .62 * v))
        # Interior ascending shoots connect the outer sprays visually and botanically.
        f = x.sample(ident, .57)
        direction = b.unit(b.lerp(b.UP, b.normal_to(f.t, b.UP), .7))
        shoot = f'{ident}/interior'
        curved(x, shoot, ident, .57, b.add(f.p, b.mul(direction, .95)),
               direction, .003, tip=.40, arch=.025, noise=.012)
        core_shoots.append((shoot, a))

    def leaves(host, phase, count):
        for k in range(count):
            lid = f'{host}/leaf{k:02d}'
            lu = .06 + .92 * k / (count - 1)
            lf = x.sample(host, lu)
            angle = phase + k * 2.399963 + .7 * b.uniform(seed, lid, 'azimuth')
            d = b.unit(b.add(b.add(b.mul(lf.n, math.cos(angle)), b.mul(lf.b, math.sin(angle))), b.mul(lf.t, .37)))
            normal = b.normal_to(d, b.cross(b.UP, d))
            normal = b.rotate(normal, b.mul(d, .7 * (b.uniform(seed, lid, 'roll') - .5)))
            size = .68 + .50 * b.uniform(seed, lid, 'size')
            x.leaf(lid, host, lu, d, cfg['leaf_length_m'] * size,
                   cfg['leaf_width_m'] * size, normal=normal, bend=-.08,
                   target=(0., -1., 0.), attraction=.6)
            shape = x.by_id[lid]['shape']
            shape.update(fold=.055, cup=.055)
            shape['outline'].update(base_power=.48, tip_power=1.15)

    for host, phase in inner_sprays:
        leaves(host, phase, 16)
    for host, phase in core_shoots:
        leaves(host, phase, 30)

    for host, phase in tips:
        for j in range(cfg['shoots_per_tip']):
            ident = f'{host}/shoot{j}'
            u = .22 + .72 * (j + .5) / cfg['shoots_per_tip']
            f = x.sample(host, u)
            a = phase + j * 2.39996 + .45 * b.uniform(seed, ident, 'rotation')
            lateral = b.add(b.mul(f.n, math.cos(a)), b.mul(f.b, math.sin(a)))
            direction = b.unit(b.add(b.mul(f.t, .22), b.mul(lateral, .98)))
            length = .62 + .38 * b.uniform(seed, ident, 'length')
            curved(x, ident, host, u, b.add(f.p, b.mul(direction, length)), direction,
                   min(.0024, radius_at(x, host, u) * .42), tip=.47,
                   arch=.016, noise=.012, taper=1.15)
            leaves(ident, phase, cfg['leaves_per_shoot'])
    r = x.recipe(variant['id'], bark_color=[1., 1., 1.], leaf_color=[1., 1., 1.])
    r['version'] = r['source']['version'] = 7
    r['settings'].update(axis_samples=cfg['axis_samples'], sides=cfg['sides'], blade_samples=cfg['blade_samples'])
    return r


def sample(rgb, roughness=.85, height=.5, transmission=0.):
    return dict(color_roughness=[*rgb, roughness], response=[0., height, 1., 1.], emission=[0., 0., 0., transmission])


def layer(field, frequency, rgb, seed, threshold=(.2, .8), strength=1., warp=.2):
    return dict(field=field, frequency=[int(round(f)) for f in frequency], seed=seed, octaves=2,
                warp=warp, threshold=threshold, strength=strength, invert=False,
                target=sample(rgb))


def materials(seed):
    common = dict(seed=seed % 65536, unique_instances=False, plane='xy', normal_filter='pixel', view_steps=8, shadow_steps=2)
    bark = dict(common, tile_size=[1., 1.], base=sample([.73, .75, .65]),
                height_m=.0006, relief='bump', layers=[
        layer('noise', [2.7, 2.5], [.52, .57, .52], 2201, strength=.45, warp=.32),
        layer('noise', [3.8, 26.], [.075, .074, .046], 2202, threshold=(.64, .76), warp=.22),
        layer('noise', [7., 64.], [.23, .24, .17], 2203, threshold=(.69, .79), strength=.85, warp=.16),
        layer('noise', [22., 9.], [.48, .51, .46], 2204, threshold=(.76, .90), strength=.35)])
    leaves = dict(common, tile_size=[1.4, 1.4], base=sample([.65, .32, .010], .86, transmission=.12),
                  height_m=.0002, relief='bump', layers=[
        layer('noise', [2.4, 2.4], [.96, .66, .024], 2211, strength=1., warp=.45),
        layer('noise', [17., 17.], [.38, .17, .006], 2212, threshold=(.44, .80), strength=.75),
        layer('ridged', [50., 30.], [.92, .58, .025], 2213, threshold=(.77, .94), strength=.16)])
    return dict(wood=bark, leaves=leaves)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--recipe', type=Path, default=RECIPE)
    p.add_argument('--output', type=Path, default=DEST)
    args = p.parse_args()
    cfg = json.loads(args.recipe.read_text())
    manifest = dict(recipe=str(args.recipe.relative_to(ROOT)), entries=[])
    for i, variant in enumerate(cfg['variants']):
        r = graph(cfg, variant, i)
        path = args.output / (variant['id'] + '.ignis-botany')
        atomic_json(path, r)
        manifest['entries'].append(dict(**variant, seed=r['source']['seed'], path=path.name,
            materials={part: variant['id'] + '_' + part + '.ignis-material' for part in ['wood', 'leaves']},
            organs=len(r['source']['organs']), leaves=sum(o['shape']['kind'] == 'blade' for o in r['source']['organs'])))
    atomic_json(args.output / 'materials.recipe.json', materials(cfg['seed']))
    atomic_json(args.output / 'library.json', manifest)
    print(json.dumps(manifest, indent=2))


if __name__ == '__main__':
    main()
