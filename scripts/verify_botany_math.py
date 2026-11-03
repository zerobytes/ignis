#!/usr/bin/env python3
"""Risk checks + numerical experiments for the independent botany reference.

Persistent guards protect identity, bounded generation and atomic report output.
Analytic oracles measure numerical error; no engine, GUI, GPU or dependencies.
"""
import argparse
from dataclasses import asdict, replace
import json
import math
from pathlib import Path
import tempfile

import botany_math as b


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def rejects(call):
    try:
        call()
    except (b.Invalid, ValueError):
        return
    raise AssertionError("invalid input unexpectedly accepted")


def identity_and_integrity():
    """Wrong organ identity would invalidate future attachments/edit ownership."""
    recipe = b.examples()["arbusto_florido"]
    plant = b.generate(recipe)
    snapshot = asdict(recipe)
    check(plant == b.generate(recipe), "same evaluation drifted")
    defs = dict(recipe.definitions)
    stem = defs["stem"]
    defs["stem"] = replace(stem, rules=tuple(reversed(stem.rules)))
    defs["unused"] = b.Blade(b.Field(.03), b.Field(.01))
    reordered = b.generate(replace(recipe, definitions=defs))
    check(plant.organs == reordered.organs, "rule/dictionary order changed existing identities")
    defs["stem"] = replace(stem, rules=stem.rules+(
        b.Rule("independent", "leaf", at=(.5,), inclination=b.Field(.8)),))
    added = {o.id: o for o in b.generate(replace(recipe, definitions=defs)).organs}
    check(all(added[o.id] == o for o in plant.organs), "independent new rule changed old organs")
    check(snapshot == asdict(recipe), "evaluation mutated recipe")
    max_frame_error = max_attachment_error = 0.
    counts = {}
    for name, source in b.examples().items():
        p = b.generate(source)
        lookup = {o.id: o for o in p.organs}
        check(len(lookup) == len(p.organs), "duplicate identity")
        counts[name] = len(p.organs)
        for organ in p.organs:
            if organ.parent is not None:
                parent = lookup[organ.parent]
                check(parent.level+1 == organ.level, "broken parent/depth relation")
                error = b.norm(b.sub(organ.samples[0].p, parent.sample(organ.insertion).p))
                max_attachment_error = max(max_attachment_error, error)
            for sample in organ.samples:
                error = max(abs(b.norm(sample.t)-1), abs(b.norm(sample.n)-1),
                            abs(b.dot(sample.t, sample.n)), abs(b.norm(sample.b)-1))
                max_frame_error = max(max_frame_error, error)
                check(all(math.isfinite(x) for x in sample.p+sample.t+sample.n), "nonfinite structure")
    check(max_frame_error < 1e-12 and max_attachment_error < 1e-12, "corrupt frame/insertion")
    return dict(counts=counts, max_frame_error=max_frame_error,
                max_attachment_error_m=max_attachment_error)


def bounded_and_atomic():
    """Failure must neither publish a partial structure nor overwrite old output."""
    recipe = b.examples()["arvore"]
    before = asdict(recipe)
    for limits in (b.Limits(organs=2), b.Limits(work=3), b.Limits(depth=1)):
        rejects(lambda: b.generate(recipe, limits))
    cyclic = b.Recipe("recursive", {"a": b.Axis(b.Field(.1), rules=(
        b.Rule("next", "a", at=(1.,), inclination=b.Field(0.)),))}, (b.Root("a"),))
    rejects(lambda: b.generate(cyclic, b.Limits(depth=3)))
    defs = dict(recipe.definitions)
    defs["trunk"] = replace(defs["trunk"], rules=(b.Rule("missing", "absent"),))
    rejects(lambda: b.generate(replace(recipe, definitions=defs)))
    defs["trunk"] = replace(recipe.definitions["trunk"], length=b.Field(float("nan")))
    rejects(lambda: b.generate(replace(recipe, definitions=defs)))
    defs["trunk"] = replace(recipe.definitions["trunk"], rules=(
        b.Rule("duplicate", "leaf"), b.Rule("duplicate", "leaf")))
    rejects(lambda: b.generate(replace(recipe, definitions=defs)))
    rejects(lambda: b.Curve(((1., 0.), (1., 1.))).validate())
    # An overcurved/underresolved curve must be refused, never softened.
    sharp = b.Recipe("sharp", {"a": b.Axis(b.Field(1.), b.Growth(bend_n=b.Field(100.)))},
                     (b.Root("a"),))
    rejects(lambda: b.generate(sharp))
    check(before == asdict(recipe), "failed evaluation mutated source")
    with tempfile.TemporaryDirectory(prefix="botany-risk-") as directory:
        destination = Path(directory)/"current.json"
        destination.write_text("previous valid report\n", encoding="utf-8")
        rejects(lambda: b.atomic_json(destination, {"bad": float("nan")}))
        check(destination.read_text() == "previous valid report\n", "failed write replaced old data")
        try:
            plant = b.generate(recipe, b.Limits(organs=2))
            b.atomic_json(destination, b.report(plant))
        except b.Invalid:
            pass
        check(destination.read_text() == "previous valid report\n", "partial generation published")
        value = b.report(b.generate(b.examples()["grama"]))
        b.atomic_json(destination, value)
        loaded = json.loads(destination.read_text())
        check(loaded["format"] == b.VERSION and len(loaded["organs"]) == 36,
              "report roundtrip lost structure/version")
        check(not list(Path(directory).glob(".botany-*")), "temporary output leaked")
    return dict(identity_guards="passed", budget_guards="passed", atomic_output="passed")


def numeric_oracles():
    metrics = {}
    # Unit-speed straight/circular curves have known endpoints.
    circle = b.Recipe("circle", {"a": b.Axis(b.Field(1.), b.Growth(bend_n=b.Field(1.)))},
                      (b.Root("a", radius=.1),))
    expected = (1-math.cos(1), math.sin(1), 0.)
    errors = []
    for n in (16, 32, 64, 128):
        o = b.generate(circle, b.Limits(segments=n)).organs[0]
        errors.append(b.norm(b.sub(o.samples[-1].p, expected)))
        check(abs(sum(b.norm(b.sub(a.p, c.p)) for a, c in zip(o.samples, o.samples[1:]))-1) < 1e-12,
              "discrete arc length drifted")
    check(all(a/c > 3.9 for a, c in zip(errors, errors[1:])) and errors[-1] < 2.5e-6,
          "circle integration did not converge quadratically")
    metrics["circle_endpoint_errors_m_16_32_64_128"] = errors

    # Antiparallel direction has an explicit deterministic turn-plane policy.
    turned = b.Recipe("turn", {"a": b.Axis(b.Field(3.), b.Growth(attraction=b.Field(2.)))},
                      (b.Root("a", direction=(0., -1., 0.)),))
    o = b.generate(turned, b.Limits(segments=128)).organs[0]
    metrics["antiparallel_final_up_dot"] = b.dot(o.samples[-1].t, b.UP)
    check(metrics["antiparallel_final_up_dot"] > .99, "antiparallel attraction stuck")

    # Constant station intensity: two events/m on a one-metre axis.
    rule = b.Rule("constant", "a", b.Field(2.))
    spec = circle.definitions["a"]
    o = b.generate(circle).organs[0]
    events = b.stations(o, spec, rule, 42, b.Limits(), b.Budget(10000))
    phase = b.uniform(42, o.id, "constant:station_phase")
    check(len(events) == 2 and max(abs(u-(phase+i)/2) for i, u in enumerate(events)) < 1e-12,
          "station intensity has wrong units/inverse")
    gate = b.Field(5., (("u", b.Curve(((0., 0.), (.3, 0.), (.5, 1.), (1., 1.)))),))
    gated = replace(rule, density=gate)
    event_sets = [b.stations(o, spec, gated, 42, b.Limits(station_bins=n), b.Budget(10000))
                  for n in (64, 128, 256)]
    check(all(all(u >= .3 for u in s) for s in event_sets), "event born in zero-intensity interval")
    check(len({len(s) for s in event_sets}) == 1, "station convergence changed count")
    diffs = [max(abs(a-c) for a, c in zip(x, y)) for x, y in zip(event_sets, event_sets[1:])]
    metrics["station_refinement_max_delta_u_64_128_256"] = diffs
    check(diffs[-1] < 1e-4, "station inversion underresolved")

    # Radius and world height can independently cause births on the same axis.
    thick = b.Field(12., (("radius", b.Curve(((0., 0.), (.025, 0.), (.03, 1.), (.04, 1.)))),))
    high = b.Field(12., (("height", b.Curve(((0., 0.), (.65, 0.), (.7, 1.), (1., 1.)))),))
    conditional = b.Recipe("conditional", {
        "a": b.Axis(b.Field(1.), tip_ratio=.1, rules=(
            b.Rule("by_radius", "leaf", thick), b.Rule("by_height", "leaf", high))),
        "leaf": b.Blade(b.Field(.05), b.Field(.02))}, (b.Root("a", radius=.04),))
    conditional_plant = b.generate(conditional)
    radius_births = [o for o in conditional_plant.organs if "/by_radius:" in o.id]
    height_births = [o for o in conditional_plant.organs if "/by_height:" in o.id]
    check(radius_births and height_births and all(o.samples[0].p[1] < .42 for o in radius_births)
          and all(o.samples[0].p[1] > .65 for o in height_births),
          "world height/radius alternatives were coupled or used wrong context")
    metrics["independent_radius_height_birth_counts"] = [len(radius_births), len(height_births)]

    # A parabolic, flat blade has analytic area 2*L*W/3.
    leaf = b.Blade(b.Field(.2), b.Field(.08), base_power=1., tip_power=1., relief=0., vein_pairs=0)
    o = b.generate(b.Recipe("leaf", {"leaf": leaf}, (b.Root("leaf"),))).organs[0]
    exact = 2*.2*.08/3
    areas = [b.properties(o, leaf, n)["area_m2"] for n in (12, 24, 48)]
    leaf_errors = [abs(a-exact)/exact for a in areas]
    check(all(a/c > 3.9 for a, c in zip(leaf_errors, leaf_errors[1:]))
          and leaf_errors[-1] < .00022, "blade area/mass quadrature failed analytic oracle")
    metrics["flat_blade_area_relative_error_12_24_48"] = leaf_errors

    # A homogeneous circular constant section has analytic mass and stiffness.
    material = b.Material(500., 1e9, 4e8)
    cylinder = b.Axis(b.Field(2.), tip_ratio=1., bark_fraction=.2, core=material, bark=material)
    o = b.generate(b.Recipe("cylinder", {"a": cylinder}, (b.Root("a", radius=.1),))).organs[0]
    props = b.properties(o, cylinder)
    section = props["base_section"]
    mass = 500*math.pi*.1**2*2
    ei = 1e9*math.pi*.1**4/4
    check(abs(props["mass_kg"]/mass-1) < 1e-12
          and abs(section["EI_N_Nm2"]/ei-1) < 1e-12
          and abs(section["GJ_approx_Nm2"]/(4e8*math.pi*.1**4/2)-1) < 1e-12,
          "mass/section material integration failed")
    metrics["homogeneous_cylinder_mass_relative_error"] = abs(props["mass_kg"]/mass-1)

    # Measure surface quadrature convergence on a genuinely curved/folded/veined leaf.
    curved = replace(leaf, growth=b.Growth(bend_b=b.Field(3.), twist=b.Field(1.)),
                     fold=.1, cup=.3, vein_pairs=4, relief=.00015, vein_width=.001)
    o = b.generate(b.Recipe("curved", {"leaf": curved}, (b.Root("leaf"),))).organs[0]
    surface_props = [b.properties(o, curved, n) for n in (24, 48, 96)]
    deltas = [abs(a["area_m2"]/c["area_m2"]-1) for a, c in zip(surface_props, surface_props[1:])]
    metrics["curved_blade_area_relative_delta_24_48_96"] = deltas
    check(deltas[-1] < .003, "curved leaf quadrature needs further refinement")
    metrics["curved_blade_properties_96"] = surface_props[-1]
    return metrics


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path("artifacts/botany-math/verification.json"))
    args = parser.parse_args()
    value = dict(format=b.VERSION, structure=identity_and_integrity(),
                 risk_guards=bounded_and_atomic(), numerical=numeric_oracles())
    b.atomic_json(args.output, value)
    print(json.dumps(value, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
