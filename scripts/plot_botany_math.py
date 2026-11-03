#!/usr/bin/env python3
"""Export vector diagnostics of the independent mathematics, not an Ignis render."""
import argparse
from dataclasses import replace
from html import escape
from pathlib import Path

import botany_math as b


def project(p):
    return (p[0]*.866+p[2]*.5, p[1]-.15*p[2])


def geometry_paths(plant):
    paths = []
    for organ in plant.organs:
        spec = plant.recipe.definitions[organ.definition]
        if isinstance(spec, b.Blade):
            network = b.veins(spec)
            if hasattr(spec, "outline_points"):
                points = list(spec.outline_points(organ))
            else:
                points = [b.blade_surface(organ, spec, i/16, -1., network) for i in range(17)]
                points += [b.blade_surface(organ, spec, i/16, 1., network) for i in range(16, -1, -1)]
            color = "#c6688c" if spec.label == "petal" else "#76a67a"
            paths.append((points, color, "#3e7352", .45, True))
        else:
            points = [s.p for s in organ.samples]
            widths = [2*b.radius_at(organ, spec, (i+.5)/(len(points)-1))
                      for i in range(len(points)-1)]
            color = {"fruit": "#b86c49", "seed": "#6d5337", "pollen": "#c6a345",
                     "fleshy_pad": "#669075", "stamen": "#ad9a54"}.get(spec.label, "#795f40")
            paths.append((points, "none", color, widths, False))
    return paths


def path_bounds(paths):
    points = []
    for vertices, _, _, widths, closed in paths:
        radius = 0. if closed else max(widths)/2
        for point in vertices:
            x, y = project(point)
            points.extend(((x-radius, y-1.02*radius), (x+radius, y+1.02*radius)))
    return (min(x for x, _ in points), max(x for x, _ in points),
            min(y for _, y in points), max(y for _, y in points))


def overview(plants, *, common_scale=False, labels=None,
             heading="Plantas procedurais · laboratório matemático v1",
             subtitle="Mesmas equações, cinco receitas. Projeção ortográfica; escala ajustada em cada painel."):
    height = 180+435*((len(plants)+1)//2)
    svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="{height}" viewBox="0 0 1600 {height}">',
           f'<rect width="1600" height="{height}" fill="#f3f2ec"/>',
           '<g font-family="Helvetica,Arial,sans-serif" fill="#24372d">',
           f'<text x="50" y="53" font-size="28" font-weight="bold">{escape(heading)}</text>',
           f'<text x="50" y="83" font-size="16">{escape(subtitle)}</text>']
    titles = {"grama": "Grama · lâminas basais", "arvore": "Árvore · ramificação hierárquica",
              "arbusto_florido": "Arbusto · folhas e pétalas", "folhagem_composta": "Folhagem composta · raque e folíolos",
              "folha_pinada": "Folha pinada · perfil de tamanho dos folíolos"}
    titles.update(labels or {})
    prepared = [geometry_paths(plant) for plant in plants]
    shared_bounds = path_bounds([path for paths in prepared for path in paths]) if common_scale else None
    for index, (plant, paths) in enumerate(zip(plants, prepared)):
        ox, oy = 40+(index % 2)*780, 115+(index//2)*435
        svg += [f'<rect x="{ox}" y="{oy}" width="750" height="415" rx="12" fill="#ffffff" stroke="#d6dcd3"/>',
                f'<text x="{ox+22}" y="{oy+32}" font-size="20" font-weight="bold">{titles[plant.recipe.name]}</text>']
        xmin, xmax, ymin, ymax = shared_bounds or path_bounds(paths)
        scale = min(650/max(1e-6, xmax-xmin), 310/max(1e-6, ymax-ymin))

        def points_string(points):
            return " ".join(f"{ox+375+(x-(xmin+xmax)/2)*scale:.2f},{oy+365-(y-ymin)*scale:.2f}"
                            for x, y in map(project, points))

        for points, fill, stroke, width, closed in sorted(paths, key=lambda x: sum(p[2] for p in x[0])/len(x[0]), reverse=True):
            if closed:
                svg.append(f'<polygon points="{points_string(points)}" fill="{fill}" fill-opacity=".88" '
                           f'stroke="{stroke}" stroke-width="{width:.2f}" stroke-linejoin="round"/>')
            else:
                for a, c, thickness in zip(points, points[1:], width):
                    stroke_width = max(.35, thickness*scale)
                    svg.append(f'<polyline points="{points_string((a, c))}" fill="none" '
                               f'stroke="{stroke}" stroke-width="{stroke_width:.2f}" stroke-linecap="round"/>')
        svg.append(f'<text x="{ox+22}" y="{oy+394}" font-size="14">{len(plant.organs)} órgãos · seed {plant.recipe.seed} · sem união de superfícies nas bifurcações</text>')
    svg += [f'<text x="50" y="{height-27}" font-size="16">Diagnóstico geométrico de repouso. Não é render do Ignis nem demonstração de física ou fidelidade botânica.</text>', '</g></svg>']
    return "\n".join(svg)+"\n"


def compound_profile():
    uniform = replace(b.compound_leaf_recipe(b.Curve(), b.Curve()), name="uniforme")
    profiled = replace(b.compound_leaf_recipe(), name="perfil")
    return overview([b.generate(uniform), b.generate(profiled)], common_scale=True,
                    labels={"uniforme": "Folíolos de tamanho uniforme",
                            "perfil": "Curtos na base → maiores → menores na ponta"},
                    heading="A silhueta da folha composta vem do perfil de tamanho",
                    subtitle="Mesmo eixo, posições, ângulos, quantidade, seed e escala. Mudam apenas comprimento e largura dos folíolos.")


def leaves():
    specs = [b.Blade(b.Field(.2), b.Field(.09), relief=0., vein_pairs=0, label="Lisa"),
             b.Blade(b.Field(.2), b.Field(.09), lobes=4, lobe_depth=.6, lobe_roundness=.6,
                     relief=0., label="Lobos profundos"),
             b.Blade(b.Field(.2), b.Field(.09), lobes=4, lobe_depth=.6, lobe_roundness=2.5,
                     relief=0., label="Outra transição"),
             b.Blade(b.Field(.2), b.Field(.014), base_power=.25, tip_power=1.8,
                     relief=0., label="Lâmina estreita")]
    svg = ['<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="650">',
           '<rect width="1600" height="650" fill="#f3f2ec"/>',
           '<g font-family="Helvetica,Arial,sans-serif" fill="#24372d">',
           '<text x="45" y="48" font-size="27" font-weight="bold">Contorno e nervuras · domínios independentes</text>',
           '<text x="45" y="80" font-size="16">Mesmo comprimento físico. Lobos pinados; não representa todo tipo de contorno foliar.</text>']
    for k, spec in enumerate(specs):
        center, bottom, scale = 200+400*k, 535, 1900
        coords = []
        for side, indices in ((-1, range(129)), (1, range(128, -1, -1))):
            for i in indices:
                u = i/128
                w = b.blade_halfwidth(spec, .2, spec.width.base, u)
                coords.append(f"{center+side*w*scale:.2f},{bottom-.2*u*scale:.2f}")
        svg.append(f'<polygon points="{" ".join(coords)}" fill="#8cb18b" stroke="#3f7153" stroke-width="2"/>')
        for a, z in b.veins(spec):
            points = []
            for i in range(17):
                f = i/16
                u, v = a[0]*(1-f)+z[0]*f, a[1]*(1-f)+z[1]*f
                w = b.blade_halfwidth(spec, .2, spec.width.base, u)
                points.append(f"{center+v*w*scale:.2f},{bottom-.2*u*scale:.2f}")
            svg.append(f'<polyline points="{" ".join(points)}" fill="none" stroke="#e5e9b7" stroke-width="1.8"/>')
        svg.append(f'<text x="{center}" y="590" text-anchor="middle" font-size="20">{escape(spec.label)}</text>')
    svg += ['</g></svg>']
    return "\n".join(svg)+"\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path("artifacts/botany-math"))
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    plants = [b.generate(recipe) for recipe in b.examples().values()]
    (args.output/"overview.svg").write_text(overview(plants), encoding="utf-8")
    (args.output/"leaves.svg").write_text(leaves(), encoding="utf-8")
    (args.output/"compound-profile.svg").write_text(compound_profile(), encoding="utf-8")
    print(args.output/"overview.svg")
    print(args.output/"leaves.svg")
    print(args.output/"compound-profile.svg")


if __name__ == "__main__":
    main()
