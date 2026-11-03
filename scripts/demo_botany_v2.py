#!/usr/bin/env python3
"""Reproducible seven-axis diagnostics. Vector plots, no Ignis, no new packages."""
from dataclasses import asdict, replace
from html import escape
import math
from pathlib import Path

import botany_math as b
import botany_surfaces as s
import botany_junctions as j
import botany_ecology as e
import botany_dynamics as d
from plot_botany_math import overview, project


def leaves_svg():
    titles = {"platano": "5 pontas · forma inspirada em plátano",
              "mandioca": "7 pontas · recortes profundos",
              "leque": "15 pontas · distribuição em leque",
              "perfurada": "Contorno arredondado · furos e rede"}
    out = ['<svg xmlns="http://www.w3.org/2000/svg" width="1500" height="1120" viewBox="0 0 1500 1120">',
           '<rect width="1500" height="1120" fill="#f3f2ec"/>',
           '<g font-family="Helvetica,Arial,sans-serif" fill="#24372d">',
           '<text x="40" y="48" font-size="28" font-weight="bold">Folhas palmadas · pontas, recortes, arredondamento e nervuras</text>',
           '<text x="40" y="79" font-size="16">Contornos e grafos em projeção plana. Arquétipos sintéticos; sem calibração de espécie.</text>']
    for index, (name, spec) in enumerate(s.leaf_examples().items()):
        ox, oy = 30+735*(index % 2), 110+490*(index//2)
        xy = [spec.material_xy(1., -1+2*i/512) for i in range(512)]
        miny, maxy = min(y for _, y in xy), max(y for _, y in xy)
        scale = min(470., 345/max(.1, maxy-miny))
        center_y = (miny+maxy)/2

        def point(x, y):
            return f"{ox+350+x*scale:.3f},{oy+250-(y-center_y)*scale:.3f}"

        out += [f'<rect x="{ox}" y="{oy}" width="705" height="465" rx="12" fill="white" stroke="#d6dcd3"/>',
                f'<text x="{ox+20}" y="{oy+31}" font-size="20" font-weight="bold">{escape(titles[name])}</text>']
        boundary = "M"+" L".join(point(x, y) for x, y in xy)+" Z"
        for hole in spec.holes:
            boundary += " M"+" L".join(point(hole.x+hole.rx*math.cos(i*math.tau/96),
                                             hole.y+hole.ry*math.sin(i*math.tau/96))
                                        for i in range(96))+" Z"
        out.append(f'<path d="{boundary}" fill="#82a988" stroke="#37664d" stroke-width="1.5" fill-rule="evenodd"/>')
        for a, c, edge in spec.graph.segments():
            width = max(.5, edge.radius*scale/spec.width.base*2)
            out.append(f'<path d="M{point(*a)} L{point(*c)}" fill="none" stroke="#e8edbb" stroke-width="{width:.2f}"/>')
        out.append(f'<text x="{ox+20}" y="{oy+444}" font-size="14">Recorte {spec.cut_depth:.0%} · arredondamento {spec.rounding:g} · {len(spec.graph.nodes)} nós / {len(spec.graph.edges)} nervuras</text>')
    out += ['<text x="40" y="1100" font-size="15">V2 matemática independente · domínio polar com furos; a superfície 3D e suas propriedades usam os mesmos dados.</text>', '</g></svg>']
    return "\n".join(out)


def junction_svg(mesh):
    projected = [project(p) for p in mesh.vertices]
    minx, maxx = min(x for x, _ in projected), max(x for x, _ in projected)
    miny, maxy = min(y for _, y in projected), max(y for _, y in projected)
    scale = min(900/(maxx-minx), 600/(maxy-miny))
    out = ['<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="800">',
           '<rect width="1200" height="800" fill="#f3f2ec"/>',
           '<g font-family="Helvetica,Arial,sans-serif" fill="#24372d">',
           '<text x="40" y="45" font-size="27" font-weight="bold">Junção contínua · união implícita suave</text>']
    view_direction = b.cross((.866, 0., .5), (0., 1., -.15))
    for triangle in sorted(mesh.triangles, key=lambda t: sum(b.dot(mesh.vertices[i], view_direction) for i in t)):
        vertices = [mesh.vertices[i] for i in triangle]
        normal = b.unit(b.cross(b.sub(vertices[1], vertices[0]), b.sub(vertices[2], vertices[0])))
        light = max(0., b.dot(normal, b.unit((-.5, .7, 1.))))
        rgb = (int(91+80*light), int(73+70*light), int(49+54*light))
        points = " ".join(f"{600+(project(p)[0]-(minx+maxx)/2)*scale:.2f},{720-(project(p)[1]-miny)*scale:.2f}"
                          for p in vertices)
        out.append(f'<polygon points="{points}" fill="rgb{rgb}" stroke="rgb{rgb}" stroke-width=".25"/>')
    info = mesh.inspect()
    out += [f'<text x="40" y="775" font-size="16">{info["vertices"]} vértices · {info["triangles"]} triângulos · {info["boundary_or_nonmanifold_edges"]} bordas abertas/não manifold · extração local</text>', '</g></svg>']
    return "\n".join(out)


def physics_demo():
    material = b.Material(600., 1e7, 4e6)
    source = b.Recipe("haste", {"a": b.Axis(b.Field(.6), tip_ratio=1., core=material, bark=material)},
                      (b.Root("a", radius=.008),))
    model = d.ReducedModel(b.generate(source, b.Limits(segments=32)))
    state = replace(model.initial(), q=(.25, .02, .06))
    trace = []
    for i in range(241):
        trace.append(dict(t=i/120, q=state.q, energy=model.energy(state)))
        if i < 240:
            state = model.step(state, gravity=(0., 0., 0.))
    out = ['<svg xmlns="http://www.w3.org/2000/svg" width="1400" height="520">',
           '<rect width="1400" height="520" fill="#f3f2ec"/>',
           '<g font-family="Helvetica,Arial,sans-serif" fill="#24372d">',
           '<text x="40" y="45" font-size="27" font-weight="bold">Dinâmica reduzida · oscilação e dissipação de energia</text>']
    for index, (title, values, low, high) in enumerate((
            ("Ângulo de flexão (rad)", [r["q"][0] for r in trace], -.3, .3),
            ("Energia mecânica / energia inicial", [r["energy"]/trace[0]["energy"] for r in trace], 0., 1.05))):
        ox = 55+690*index
        out += [f'<rect x="{ox}" y="100" width="620" height="330" fill="white" stroke="#d6dcd3"/>',
                f'<text x="{ox}" y="84" font-size="19">{title}</text>']
        points = " ".join(f"{ox+10+i/240*600:.2f},{420-(v-low)/(high-low)*300:.2f}" for i, v in enumerate(values))
        out.append(f'<polyline points="{points}" fill="none" stroke="#478566" stroke-width="2.5"/>')
        out.append(f'<text x="{ox}" y="455" font-size="16">0 s</text><text x="{ox+575}" y="455" font-size="16">2 s</text>')
    out += ['<text x="40" y="500" font-size="15">Euler implícito · flexão/torção · estado por identidade · ensaio sem gravidade/vento para medir passividade</text>', '</g></svg>']
    return "\n".join(out), trace


def main():
    output = Path("artifacts/botany-math")
    output.mkdir(parents=True, exist_ok=True)
    (output/"v2-leaves.svg").write_text(leaves_svg(), encoding="utf-8")
    mesh = j.extract(j.branch_junction(), (-.2, -.5, -.2), (.5, .6, .24), 28)
    (output/"v2-junction.svg").write_text(junction_svg(mesh), encoding="utf-8")
    b.atomic_json(output/"v2-junction.json", asdict(mesh))
    recipes = e.organ_recipes()
    plants = [b.generate(recipe, b.Limits(segments=128)) for recipe in recipes.values()]
    labels = {"flor": "Flor · pétalas, sépalas, estames e pólen",
              "fruto_semente": "Fruto e semente · volumes próprios", "raizes": "Raízes · direção e ramificação",
              "suculenta": "Suculenta · órgãos volumosos", "trepadeira": "Trepadeira · suporte helicoidal"}
    (output/"v2-organs.svg").write_text(overview(plants, labels=labels,
        heading="Órgãos e modos de crescimento · receitas matemáticas",
        subtitle="Cinco exemplos com o mesmo gerador; projeções geométricas, sem calibração botânica."), encoding="utf-8")
    adult = b.generate(b.examples()["arbusto_florido"])
    timeline = []
    labels = {}
    for time in (2., 7., 15., 40.):
        view = e.growth_at(adult, time)
        name = "age"+str(time)
        timeline.append(replace(adult, recipe=replace(adult.recipe, name=name),
                                organs=tuple(o.shape for o in view)))
        labels[name] = f"{time:g} s · {len(view)} órgãos visíveis"
    (output/"v2-growth.svg").write_text(overview(timeline, labels=labels, common_scale=True,
        heading="Crescimento · nascimento, exposição e maturação",
        subtitle="Mesmos IDs e inserções materiais. Eixos se revelam por comprimento; lâminas crescem e abrem."), encoding="utf-8")
    shaped = b.generate(b.examples()["arvore"], environment=e.Environment(
        crown_center=(0., 2.8, 0.), crown_radii=(1.1, 1.6, 1.1), crown_start=1.1,
        dominance=.4, tip_spacing=.18, voids=(e.Sphere((.45, 2.8, 0.), .25),)))
    plain = b.generate(b.examples()["arvore"])
    sized = e.dimension_radii(shaped)
    compare = [replace(plain, recipe=replace(plain.recipe, name="authored")),
               replace(sized, recipe=replace(sized.recipe, name="crown"))]
    (output/"v2-crown.svg").write_text(overview(compare, common_scale=True,
        labels={"authored": "Regras locais", "crown": "Copa, vazio, competição e espessuras"},
        heading="Arquitetura global · admissão e dimensionamento",
        subtitle=f"{len(shaped.rejections)} candidatos rejeitados com motivo; topologia decidida antes do dimensionamento."), encoding="utf-8")
    svg, trace = physics_demo()
    (output/"v2-physics.svg").write_text(svg, encoding="utf-8")
    b.atomic_json(output/"v2-demo.json", dict(organs={p.recipe.name:len(p.organs) for p in plants},
        crown_rejections=shaped.rejections, physics=trace, junction=mesh.inspect()))
    print("Generated v2-leaves/junction/organs/growth/crown/physics.svg in", output)


if __name__ == "__main__":
    main()
