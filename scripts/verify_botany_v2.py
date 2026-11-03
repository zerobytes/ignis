#!/usr/bin/env python3
"""Critical integration/state/topology guards and direct numerical experiments."""
from dataclasses import asdict, replace
import json
import math
from pathlib import Path

import botany_math as b
import botany_surfaces as s
import botany_junctions as j
import botany_ecology as e
import botany_dynamics as d
from verify_botany_math import check, rejects


def surfaces_and_topology():
    metrics = {}
    for name, spec in s.leaf_examples().items():
        plant = s.leaf_plant(spec, name)
        spec.graph.validate()
        for a, c, _ in spec.graph.segments():
            check(all(spec.inside(a[0]+k/64*(c[0]-a[0]), a[1]+k/64*(c[1]-a[1]))
                      for k in range(65)), "vein traverses excluded domain")
        material = spec.graph.mechanics(.2, .2)
        check(material["additive_mass_kg"] > 0
              and all(material["integrated_directional_EI_Nm3"][i][i] > 0 for i in (0, 1)),
              "invalid vascular reinforcement")
        metrics[name] = dict(tips=spec.tips, nodes=len(spec.graph.nodes), edges=len(spec.graph.edges),
                             cycles=len(spec.graph.edges)-len(spec.graph.nodes)+1,
                             mass_veins_kg=material["additive_mass_kg"])
        check(all(math.isfinite(x) for p in spec.outline_points(plant.organs[0]) for x in p),
              "nonfinite contour")
    plain = replace(s.leaf_examples()["perfurada"], relief=0., fold=0., cup=0.)
    solid = replace(plain, holes=())
    op, os = s.leaf_plant(plain).organs[0], s.leaf_plant(solid).organs[0]
    hole_area = sum(math.pi*h.rx*h.ry*op.length*op.width for h in plain.holes)
    area_with = b.properties(op, plain, 160)["area_m2"]
    area_without = b.properties(os, solid, 160)["area_m2"]
    error = abs((area_without-area_with)/hole_area-1)
    check(error < .12 and area_with < area_without, "hole quadrature failed area oracle")
    metrics["hole_area_relative_error_160"] = error
    rejects(lambda: b.generate(b.Recipe("bad", {"leaf": replace(plain, holes=(s.Hole(10., 0., .1, .1),))},
                                        (b.Root("leaf"),))))

    sphere = j.Junction((j.Ellipsoid((0., 0., 0.), (.1, .1, .1)),), .01)
    exact_volume = 4*math.pi*.1**3/3
    errors = []
    for n in (12, 24):
        mesh = j.extract(sphere, (-.15, -.15, -.15), (.15, .15, .15), n)
        info = mesh.inspect()
        check(info["boundary_or_nonmanifold_edges"] == 0, "nonmanifold implicit surface")
        errors.append(abs(info["signed_volume_m3"]/exact_volume-1))
    check(errors[1] < errors[0] and errors[1] < .02, "implicit volume did not converge")
    branch = j.extract(j.branch_junction(), (-.2, -.5, -.2), (.5, .6, .24), 24)
    metrics["junction"] = branch.inspect()
    metrics["sphere_volume_relative_error_12_24"] = errors
    rejects(lambda: j.extract(sphere, (-.01, -.01, -.01), (.01, .01, .01), 12))
    rejects(lambda: j.extract(sphere, (-1., -1., -1.), (1., 1., 1.), 100))
    return metrics


def architecture_growth_organs():
    recipe = b.examples()["arbusto_florido"]
    original = b.generate(recipe)
    environment = e.Environment(crown_center=(0., .6, 0.), crown_radii=(.3, .6, .3),
                                voids=(e.Sphere((.15, .65, 0.), .07),), tip_spacing=.08,
                                dominance=.3)
    shaped = b.generate(recipe, environment=environment)
    check(shaped.rejections and len(shaped.organs) < len(original.organs), "crown admission not exercised")
    sized = e.dimension_radii(original)
    check([o.id for o in sized.organs] == [o.id for o in original.organs], "sizing changed ownership")
    for current, authored in zip(sized.organs, original.organs):
        spec = original.recipe.definitions[current.definition]
        if isinstance(spec, b.Axis):
            check(all(b.radius_at(current, spec, i/64) >= b.radius_at(authored, spec, i/64)
                      for i in range(65)), "sizing thinned authored source")
    check(e.dimension_radii(sized).organs == sized.organs, "pipe sizing accumulated on repeated evaluation")
    directional = b.Recipe("directional", {"a": b.Axis(b.Field(1.))}, (b.Root("a"),))
    plain = b.generate(directional)
    light = b.generate(directional, environment=e.Environment(light=(2., 1., 0.), light_strength=.6))
    check(light.organs[0].samples[-1].p[0] > plain.organs[0].samples[-1].p[0]+.1,
          "light field did not affect growth")
    obstacle = e.Sphere((.18, .55, 0.), .08)
    avoided = b.generate(directional, environment=e.Environment(obstacles=(obstacle,), clearance=.25, avoidance=.8))
    check(avoided.organs[0].samples[-1].p[0] < -.005, "obstacle field did not bend growth")
    rejects(lambda: b.generate(recipe, environment=e.Environment(max_queries=1)))

    lookup = {o.id: o for o in original.organs}
    schedule = e.GrowthSchedule()
    for time in (0., 2., 5., 10., 20., 40.):
        view = e.growth_at(original, time)
        active = {o.shape.id: o for o in view}
        for growing in view:
            o = growing.shape
            if o.parent is not None:
                check(o.parent in active, "child appeared before parent")
                parent = active[o.parent]
                u = o.insertion/parent.material_extent
                check(u <= 1+1e-10, "parent has not reached child insertion")
                expected = parent.shape.sample(min(1., u)).p
                # Two interpolations of a curved discrete axis are approximate.
                check(b.norm(b.sub(expected, o.samples[0].p)) < .0002, "growth detached child")
    mature = e.growth_at(original, 100.)
    check({o.shape.id for o in mature} == set(lookup), "maturation lost identities")
    check(all(o.maturity == 1 for o in mature), "maturation incomplete")
    check(all(o.shape == lookup[o.shape.id] for o in mature), "maturation changed canonical source")
    state = e.PruningState(original)
    target = next(o for o in original.organs if o.parent is not None and o.width == 0)
    cut = e.prune(state, target.id)
    check(e.descendants(original, target.id) <= cut.retired, "pruning left descendant ownership")
    before = asdict(cut.plant)
    rejects(lambda: e.regrow(cut, target.parent, target.insertion, "absent"))
    check(asdict(cut.plant) == before, "failed regrowth mutated source")
    new = e.regrow(cut, target.parent, target.insertion, target.definition, seconds=100.)
    check(not (set(o.id for o in new.plant.organs) & cut.retired), "regrowth recycled retired identity")
    check({o.id for o in cut.plant.organs} <= {o.id for o in new.plant.organs}, "regrowth lost survivors")
    prefix = new.birth_overrides[-1][0]
    check(not any(o.shape.id.startswith(prefix) for o in e.growth_of(new, 99.)),
          "regrowth appeared before its recorded birth")
    check(any(o.shape.id.startswith(prefix) for o in e.growth_of(new, 101.)), "regrowth failed to mature")

    counts = {}
    for name, source in e.organ_recipes().items():
        counts[name] = len(b.generate(source, b.Limits(segments=128)).organs)
    helix = e.organ_recipes()["trepadeira"]
    errors = []
    for n in (128, 256):
        vine = next(o for o in b.generate(helix, b.Limits(segments=n)).organs if o.parent is None)
        errors.append(max(b.norm(b.sub(sample.p, (.06*math.cos(math.tau*3*i/n),
                          .3*i/n, .06*math.sin(math.tau*3*i/n)))) for i, sample in enumerate(vine.samples)))
    check(errors[-1] < .0005 and errors[-1] < errors[0], "support helix failed analytic centerline")
    solid = s.SolidAxis(b.Field(.2), core=b.Material(900., 1e7, 4e6),
                       bark=b.Material(900., 1e7, 4e6), ellipticity=.5)
    fruit = b.generate(b.Recipe("solid", {"a": solid}, (b.Root("a", radius=.06),))).organs[0]
    exact_mass = 900*4*math.pi*.06*(.06*.5)*(.2/2)/3
    mass_error = abs(b.properties(fruit, solid, 96)["mass_kg"]/exact_mass-1)
    check(mass_error < .0001, "fruit volume/mass failed ellipsoid oracle")
    return dict(original_organs=len(original.organs), crown_organs=len(shaped.organs),
                rejected=shaped.rejections, pipe_root_radius_before=original.organs[0].radius,
                pipe_root_radius_after=b.radius_at(sized.organs[0], recipe.definitions[sized.organs[0].definition], 0.),
                retired=len(cut.retired),
                regrown=len(new.plant.organs)-len(cut.plant.organs), organs=counts,
                helix_max_point_errors_128_256_m=errors, solid_mass_relative_error=mass_error)


def physical_integration():
    material = b.Material(600., 1e7, 4e6)
    axis = b.Axis(b.Field(.6), tip_ratio=1., core=material, bark=material)
    recipe = b.Recipe("rod", {"a": axis}, (b.Root("a", radius=.008),))
    plant = b.generate(recipe, b.Limits(segments=32))
    model = d.ReducedModel(plant, d.Response(damping_mass=2.))
    state = replace(model.initial(), q=(.15, .02, .08))
    start_energy = previous = model.energy(state)
    for _ in range(240):
        state = model.step(state, gravity=(0., 0., 0.))
        energy = model.energy(state)
        check(energy <= previous+1e-12, "passive physical solver created energy")
        previous = energy
    check(previous < start_energy*.5, "damping did not dissipate")
    value = asdict(state)
    check(model.restore(json.loads(json.dumps(value))) == state, "physical state roundtrip drifted")
    wrong = dict(value, model="wrong-source")
    rejects(lambda: model.restore(wrong))
    wrong = dict(value, q=[float("nan")]*3)
    rejects(lambda: model.restore(wrong))
    before = asdict(state)
    rejects(lambda: model.step(state, external=(float("nan"), 0., 0.)))
    check(asdict(state) == before, "failed step mutated published physical state")

    static = model.initial()
    force, length = .02, .6
    generalized = (2*force*length/3, 0., 0.)
    for _ in range(2200):
        static = model.step(static, dt=.01, gravity=(0., 0., 0.), external=generalized)
    posed = model.pose(static.q)
    measured = posed.organs[0].samples[-1].p[0]
    ei = material.young*math.pi*.008**4/4
    exact = force*length**3/(3*ei)
    error = abs(measured/exact-1)
    check(error < .02, "cantilever tip-load oracle failed")

    with_leaf = replace(recipe, definitions={"a": replace(axis, rules=(
        b.Rule("leaf", "leaf", at=(1.,), inclination=b.Field(1.)),)),
        "leaf": b.Blade(b.Field(.2), b.Field(.08), relief=0.)})
    loaded = d.ReducedModel(b.generate(with_leaf, b.Limits(segments=32)))
    _, m1 = model.mass_matrix()
    _, m2 = loaded.mass_matrix()
    check(m2[0][0] > m1[0][0], "descendant leaf mass did not reach parent inertia")
    branching = replace(recipe, definitions={"a": replace(axis, rules=(
        b.Rule("branch", "branch", at=(.7,), inclination=b.Field(.8)),)),
        "branch": replace(axis, length=b.Field(.3))})
    coupled = d.ReducedModel(b.generate(branching, b.Limits(segments=32)))
    active, matrix = coupled.mass_matrix()
    check(abs(matrix[0][3]) > 1e-8, "parent/child mass coupling vanished")
    coupled_state = replace(coupled.initial(), q=(.1, .02, .04, -.08, .03, 0.))
    posed_coupled = coupled.pose(coupled_state.q)
    parent, child = sorted(posed_coupled.organs, key=lambda o: o.level)
    check(b.norm(b.sub(child.samples[0].p, parent.sample(child.insertion).p)) < 1e-12,
          "dynamic child detached from parent")
    for source, posed_organ in zip(coupled.plant.organs, posed_coupled.organs):
        check(all(abs(b.norm(b.sub(a.p, c.p))-b.norm(b.sub(x.p, y.p))) < 1e-12
                  for a, c, x, y in zip(source.samples, source.samples[1:],
                                       posed_organ.samples, posed_organ.samples[1:])),
              "dynamic reconstruction stretched segments")
    old_energy = coupled.energy(coupled_state)
    for _ in range(30):
        coupled_state = coupled.step(coupled_state, gravity=(0., 0., 0.))
        energy = coupled.energy(coupled_state)
        check(energy <= old_energy+1e-12, "coupled solver created energy")
        old_energy = energy
    windy = model.initial()
    for _ in range(120):
        windy = model.step(windy, gravity=(0., 0., 0.), wind=(1., 0., 0.))
    check(abs(windy.q[0]) > 1e-5, "wind did not drive deformation")
    # Choose a stiffness whose gravity equilibrium remains in the reduced domain.
    stiff_material = b.Material(600., 4e7, 1.6e7)
    horizontal = replace(recipe, definitions={"a": replace(axis, core=stiff_material, bark=stiff_material)},
                         roots=(b.Root("a", direction=(1., 0., 0.), normal=b.UP, radius=.008),))
    gravity_model = d.ReducedModel(b.generate(horizontal, b.Limits(segments=32)))
    gravity = gravity_model.initial()
    for _ in range(120):
        gravity = gravity_model.step(gravity)
    check(gravity.q[0] < -.001, "gravity did not bend horizontal rod")
    rejects(lambda: model.step(replace(model.initial(), q=(.7, 0., 0.))))

    plastic_model = d.ReducedModel(plant, d.Response(yield_strain=.001, fracture_strain=.02))
    plastic = plastic_model.step(replace(plastic_model.initial(), q=(.15, 0., 0.)),
                                 dt=.001, gravity=(0., 0., 0.))
    check(abs(plastic.plastic[0]) > .01, "plastic return mapping not exercised")
    fracture_model = d.ReducedModel(loaded.plant, d.Response(yield_strain=.0005, fracture_strain=.002))
    fractured = fracture_model.step(replace(fracture_model.initial(), q=(.2, 0., 0.)),
                                    dt=.001, gravity=(0., 0., 0.))
    check(set(fractured.broken) == set(o.id for o in loaded.plant.organs), "fracture lost subtree identity")
    fracture_model.validate(fractured)
    fracture_model.step(fractured, gravity=(0., 0., 0.))
    # Reinforcement must enter physical mass/stiffness, rather than only shading.
    vascular = replace(s.leaf_examples()["platano"], relief=0.)
    reinforced = d.ReducedModel(s.leaf_plant(vascular), include_blades=True)
    weak = d.ReducedModel(s.leaf_plant(replace(vascular, vein_radius=.0001)), include_blades=True)
    _, reinforced_mass = reinforced.mass_matrix()
    _, weak_mass = weak.mass_matrix()
    check(reinforced.stiffness[1] > weak.stiffness[1]
          and reinforced_mass[0][0] > weak_mass[0][0], "vascular mechanics were not consumed")
    check(weak.stiffness[1] < weak.stiffness[0]*1e-4,
          "thin lamina lost weak out-of-plane bending axis")
    return dict(dof=model.ndof, particles=len(model.particles), energy_initial_J=start_energy,
                energy_after_240_steps_J=previous, static_tip_displacement_m=measured,
                cantilever_exact_m=exact, cantilever_relative_error=error,
                mass_parent_inertia=m1[0][0], mass_with_leaf_inertia=m2[0][0],
                wind_q=windy.q, gravity_q=gravity.q, plastic_q=plastic.plastic,
                coupled_dofs=coupled.ndof, coupled_mass_offdiagonal=matrix[0][3],
                reinforced_blade_stiffness_Nm=reinforced.stiffness[1],
                weak_vein_blade_stiffness_Nm=weak.stiffness[1],
                broken=fractured.broken)


def main():
    report = dict(format="botany-math-v2-verification", surfaces=surfaces_and_topology(),
                  architecture_growth_organs=architecture_growth_organs(), physics=physical_integration())
    b.atomic_json(Path("artifacts/botany-math/v2-verification.json"), report)
    print(json.dumps(report, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
