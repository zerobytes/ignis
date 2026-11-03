"""Authorial crown fields, material growth, pruning and reusable organ recipes."""
from dataclasses import dataclass, replace
import math

import botany_math as b
from botany_surfaces import PalmateBlade, SolidAxis


@dataclass(frozen=True)
class Sphere:
    center: b.Vec
    radius: float


@dataclass(frozen=True)
class Environment:
    crown_center: b.Vec | None = None
    crown_radii: b.Vec = (2., 2., 2.)
    crown_start: float = .5
    crown_strength: float = .4
    voids: tuple[Sphere, ...] = ()
    obstacles: tuple[Sphere, ...] = ()
    clearance: float = .08
    avoidance: float = 1.
    light: b.Vec | None = None
    light_strength: float = 0.
    dominance: float = 0.
    tip_spacing: float = 0.
    max_queries: int = 200000

    def begin(self):
        b.require(len(self.crown_radii) == 3 and all(math.isfinite(x) and x > 0 for x in self.crown_radii)
                  and all(math.isfinite(x) for x in (self.crown_start, self.crown_strength,
                    self.clearance, self.avoidance, self.light_strength, self.dominance, self.tip_spacing))
                  and 0 <= self.dominance <= 1 and self.clearance > 0
                  and min(self.avoidance, self.light_strength, self.crown_strength, self.tip_spacing) >= 0
                  and type(self.max_queries) is int and self.max_queries > 0, "invalid environment")
        for vector in (self.crown_center, self.light):
            b.require(vector is None or len(vector) == 3 and all(math.isfinite(x) for x in vector),
                      "invalid environmental target")
        for sphere in self.voids+self.obstacles:
            b.require(math.isfinite(sphere.radius) and sphere.radius > 0 and len(sphere.center) == 3
                      and all(math.isfinite(x) for x in sphere.center), "invalid environment sphere")
        return _EnvironmentRun(self)


class _EnvironmentRun:
    def __init__(self, settings):
        self.settings = settings
        self.budget = b.Budget(settings.max_queries)
        self.rejections = []

    def curvature(self, sample, context):
        self.budget.take()
        s = self.settings
        force = (0., 0., 0.)
        if s.light is not None:
            d = b.sub(s.light, sample.p)
            if b.norm(d) > 1e-10:
                force = b.mul(b.unit(d), s.light_strength)
        if s.crown_center is not None and sample.p[1] >= s.crown_start:
            local = b.sub(sample.p, s.crown_center)
            radius = math.sqrt(sum((x/r)**2 for x, r in zip(local, s.crown_radii)))
            if radius > .65 and b.norm(local) > 1e-10:
                force = b.add(force, b.mul(b.unit(local), -s.crown_strength*(radius-.65)))
        for sphere in s.voids+s.obstacles:
            self.budget.take()
            d = b.sub(sample.p, sphere.center)
            distance = b.norm(d)
            reach = sphere.radius+s.clearance+context["radius"]
            if distance < reach:
                direction = b.unit(d) if distance > 1e-10 else sample.n
                force = b.add(force, b.mul(direction, s.avoidance*(reach-distance)/s.clearance))
        return b.sub(force, b.mul(sample.t, b.dot(sample.t, force)))

    def admit(self, organ, spec, accepted, birth, seed):
        s = self.settings
        reason = None
        if organ.parent is not None and isinstance(spec, b.Axis):
            suppression = s.dominance*math.exp(-(1-birth["u"])/.25)
            if b.uniform(seed, organ.id, "apical_dominance") < suppression:
                reason = "apical_dominance"
        points = [sample.p for sample in organ.samples]
        if isinstance(spec, b.Blade):
            if hasattr(spec, "outline_points"):
                points.extend(spec.outline_points(organ, 128))
            else:
                points.extend(b.blade_surface(organ, spec, i/32, side)
                              for i in range(33) for side in (-1., 1.))
        padding = (spec.thickness/2 if isinstance(spec, b.Blade) else
                   organ.radius*max(1., spec.ellipticity))
        for point in points:
            self.budget.take()
            if (s.crown_center is not None and organ.parent is not None
                    and point[1] >= s.crown_start):
                if (math.sqrt(sum(((x-c)/r)**2 for x, c, r in zip(point, s.crown_center, s.crown_radii)))
                        +padding/min(s.crown_radii) > 1):
                    reason = reason or "crown_envelope"
            for sphere in s.voids+s.obstacles:
                self.budget.take()
                if b.norm(b.sub(point, sphere.center)) < sphere.radius+padding:
                    reason = reason or "excluded_volume"
        if organ.parent is not None and isinstance(spec, b.Axis) and s.tip_spacing:
            for previous in accepted:
                self.budget.take()
                if (previous.level == organ.level and previous.width == 0
                        and b.norm(b.sub(previous.samples[-1].p, organ.samples[-1].p)) < s.tip_spacing):
                    reason = reason or "tip_competition"
        if reason:
            self.rejections.append((organ.id, reason))
            return False
        return True

    def finish(self, plant):
        b.require(bool(plant.organs), "environment rejected every root")
        return tuple(self.rejections)


def dimension_radii(plant: b.Plant, exponent=2., leaf_demand=.03):
    """Postorder sizing; preserve topology and authored minimum radius.

    At each insertion section, support all distal child demands plus the authored
    terminal demand. No feedback into the rules that created the graph.
    """
    b.require(math.isfinite(exponent) and 1 <= exponent <= 4 and 0 <= leaf_demand <= 1,
              "invalid pipe sizing")
    children = {o.id: [] for o in plant.organs}
    for organ in plant.organs:
        if organ.parent is not None:
            children[organ.parent].append(organ)
    result = {}
    for organ in sorted(plant.organs, key=lambda o: -o.level):
        spec = plant.recipe.definitions[organ.definition]
        if isinstance(spec, b.Blade):
            result[organ.id] = organ
            continue
        b.require(not isinstance(spec, SolidAxis), "pipe sizing is not a fruit/seed model")
        demands = []
        for child in children[organ.id]:
            ready = result[child.id]
            radius = (math.sqrt(ready.length*ready.width/math.pi)*leaf_demand if ready.width else
                      b.radius_at(ready, plant.recipe.definitions[ready.definition], 0.))
            demands.append((child.insertion, radius**exponent))
        terminal = b.radius_at(replace(organ, radius_profile=()), spec, 1.)**exponent
        locations = sorted({0., 1., *(at for at, _ in demands)})
        profile = []
        for i, u in enumerate(locations):
            load = terminal+sum(demand for at, demand in demands if at >= u)
            profile.append((u, load**(1/exponent)))
            if i+1 < len(locations) and any(at == u for at, _ in demands):
                v = u+min(.02, (locations[i+1]-u)/2)
                distal = terminal+sum(demand for at, demand in demands if at > u)
                profile.append((v, distal**(1/exponent)))
        b.require(all(math.isfinite(r) for _, r in profile), "pipe sizing overflow")
        result[organ.id] = replace(organ, radius_profile=tuple(profile))
    return replace(plant, organs=tuple(result[o.id] for o in plant.organs))


def smooth(t):
    t = max(0., min(1., t))
    return t*t*(3-2*t)


def inverse_smooth(value):
    lo, hi = 0., 1.
    for _ in range(48):
        middle = (lo+hi)/2
        if smooth(middle) < value:
            lo = middle
        else:
            hi = middle
    return (lo+hi)/2


@dataclass(frozen=True)
class GrowthSchedule:
    axis_seconds: float = 8.
    blade_seconds: float = 3.
    delay: float = .25
    radius_power: float = .5
    opening_angle: float = 1.2

    def validate(self):
        b.require(all(math.isfinite(x) for x in (self.axis_seconds, self.blade_seconds,
                    self.delay, self.radius_power, self.opening_angle))
                  and min(self.axis_seconds, self.blade_seconds, self.radius_power) > 0
                  and self.delay >= 0, "invalid growth schedule")


@dataclass(frozen=True)
class GrowingOrgan:
    shape: b.Organ
    born_at: float
    maturity: float
    radius_scale: float
    material_extent: float


def growth_at(plant: b.Plant, seconds: float, schedule=GrowthSchedule(), born_at=None):
    schedule.validate()
    b.require(math.isfinite(seconds) and seconds >= 0, "invalid growth time")
    births, durations, output = {}, {}, []
    overrides = born_at or {}
    lookup = {o.id: o for o in plant.organs}
    b.require(set(overrides) <= set(lookup) and all(math.isfinite(t) and t >= 0 for t in overrides.values()),
              "invalid growth birth overrides")
    for organ in sorted(plant.organs, key=lambda o: (o.level, o.id)):
        spec = plant.recipe.definitions[organ.definition]
        birth = (0. if organ.parent is None else births[organ.parent]
                 +durations[organ.parent]*inverse_smooth(organ.insertion)+schedule.delay)
        birth = max(birth, overrides.get(organ.id, 0.))
        duration = schedule.blade_seconds if isinstance(spec, b.Blade) else schedule.axis_seconds
        births[organ.id], durations[organ.id] = birth, duration
        g = smooth((seconds-birth)/duration)
        if g < 1e-8:
            continue
        if g == 1.:
            output.append(GrowingOrgan(organ, birth, 1., 1., 1.))
            continue
        if isinstance(spec, b.Axis):
            samples = tuple(organ.sample(g*i/(len(organ.samples)-1)) for i in range(len(organ.samples)))
        else:
            origin = organ.samples[0]
            if organ.parent is not None:
                target = lookup[organ.parent].sample(organ.insertion).t
                turn = b.cross(origin.t, target)
                angle = min(schedule.opening_angle, math.acos(max(-1., min(1., b.dot(origin.t, target)))))
                axis = b.unit(turn) if b.norm(turn) > 1e-10 else origin.n
                roll = b.mul(axis, angle*(1-g))
            else:
                roll = b.mul(origin.t, schedule.opening_angle*(1-g))
            samples = tuple(b.Sample(b.add(origin.p, b.mul(b.rotate(b.sub(s.p, origin.p), roll), g)),
                                     b.rotate(s.t, roll), b.rotate(s.n, roll)) for s in organ.samples)
        profile = (tuple((i/64, b.radius_at(organ, spec, g*i/64)*g**schedule.radius_power)
                         for i in range(65)) if isinstance(spec, b.Axis) else ())
        shape = replace(organ, length=organ.length*g, width=organ.width*g,
                        radius=organ.radius*g**schedule.radius_power, samples=samples,
                        radius_profile=profile)
        output.append(GrowingOrgan(shape, birth, g, g**schedule.radius_power,
                                   g if isinstance(spec, b.Axis) else 1.))
    return tuple(output)


def descendants(plant, root):
    lookup = {o.id: o for o in plant.organs}
    b.require(root in lookup, "unknown organ identity")
    removed = {root}
    for organ in sorted(plant.organs, key=lambda o: o.level):
        if organ.parent in removed:
            removed.add(organ.id)
    return removed


@dataclass(frozen=True)
class PruningState:
    plant: b.Plant
    retired: frozenset[str] = frozenset()
    serial: int = 0
    birth_overrides: tuple[tuple[str, float], ...] = ()


def prune(state: PruningState, identity):
    removed = descendants(state.plant, identity)
    return PruningState(replace(state.plant, organs=tuple(o for o in state.plant.organs if o.id not in removed)),
                        state.retired | removed, state.serial,
                        tuple((i, t) for i, t in state.birth_overrides if i not in removed))


def regrow(state: PruningState, parent_id, insertion, definition, radius=.003, limits=b.Limits(),
           seconds=0., environment=None):
    lookup = {o.id: o for o in state.plant.organs}
    b.require(parent_id in lookup and definition in state.plant.recipe.definitions,
              "regrowth needs a live parent and valid definition")
    b.require(math.isfinite(seconds) and seconds >= 0, "invalid regrowth time")
    parent = lookup[parent_id]
    sample = parent.sample(insertion)
    prefix = f"{parent_id}/regrowth{state.serial+1}"
    recipe = replace(state.plant.recipe, roots=(b.Root(definition, sample.p, sample.t, sample.n, radius),))
    generated = b.generate(recipe, limits, environment=environment)
    sprouts = []
    for o in generated.organs:
        identity = prefix+o.id[len("root0"):]
        parent_ref = parent_id if o.parent is None else prefix+o.parent[len("root0"):]
        b.require(identity not in lookup and identity not in state.retired, "regrowth reused identity")
        sprouts.append(replace(o, id=identity, parent=parent_ref, level=o.level+parent.level+1,
                               insertion=insertion if o.parent is None else o.insertion))
    b.require(len(sprouts)+len(lookup) <= limits.organs
              and all(o.level <= limits.depth for o in sprouts), "regrowth budget exceeded")
    plant = replace(state.plant, organs=tuple(sorted((*state.plant.organs, *sprouts), key=lambda o: o.id)))
    return PruningState(plant, state.retired, state.serial+1,
                        state.birth_overrides+((prefix, seconds),))


def growth_of(state: PruningState, seconds, schedule=GrowthSchedule()):
    return growth_at(state.plant, seconds, schedule, dict(state.birth_overrides))


def organ_recipes():
    soft = b.Material(950., 3e6, 1e6)
    petal = b.Blade(b.Field(.09), b.Field(.05), growth=b.Growth(bend_b=b.Field(-4.)),
                    relief=0., vein_pairs=0, label="petal")
    flower = b.Recipe("flor", {
        "stem": b.Axis(b.Field(.25), rules=(
            b.Rule("petals", "petal", at=(1.,), count=6, inclination=b.Field(1.25), jitter=0.),
            b.Rule("sepals", "sepal", at=(.97,), count=5, inclination=b.Field(1.8), jitter=0.),
            b.Rule("stamens", "stamen", at=(1.,), count=8, inclination=b.Field(.5), jitter=0.),
            b.Rule("ovary", "ovary", at=(1.,), inclination=b.Field(0.), radius_ratio=30.))),
        "petal": petal, "sepal": replace(petal, length=b.Field(.045), width=b.Field(.018), label="sepal"),
        "stamen": b.Axis(b.Field(.04), rules=(b.Rule("pollen", "pollen", at=(1.,),
                           inclination=b.Field(0.), radius_ratio=2.),), label="stamen"),
        "pollen": SolidAxis(b.Field(.012), core=soft, bark=soft, label="pollen"),
        "ovary": SolidAxis(b.Field(.018), core=soft, bark=soft, rules=(
            b.Rule("style", "style", at=(.9,), inclination=b.Field(0.), radius_ratio=.2),), label="ovary"),
        "style": b.Axis(b.Field(.035), rules=(b.Rule("stigma", "stigma", at=(1.,),
                         inclination=b.Field(0.), radius_ratio=5.),), label="style"),
        "stigma": SolidAxis(b.Field(.006), label="stigma")},
        (b.Root("stem", radius=.004),))
    fruit = b.Recipe("fruto_semente", {
        "stem": b.Axis(b.Field(.10), rules=(b.Rule("fruit", "fruit", at=(1.,),
             inclination=b.Field(0.), radius_ratio=80.),)),
        "fruit": SolidAxis(b.Field(.13), core=soft, bark=soft, label="fruit"),
        "seed": SolidAxis(b.Field(.018), ellipticity=.6, label="seed")},
        (b.Root("stem", radius=.008), b.Root("seed", position=(.12, 0., 0.), radius=.006)))
    roots = b.Recipe("raizes", {
        "root": b.Axis(b.Field(.6), b.Growth(attraction=b.Field(1.), direction=(0., -1., 0.)),
            rules=(b.Rule("laterals", "lateral", b.Field(9.), inclination=b.Field(1.)),), label="root"),
        "lateral": b.Axis(b.Field(.25), b.Growth(attraction=b.Field(1.), direction=(0., -1., 0.)),
                          label="lateral_root")}, (b.Root("root", direction=(0., -1., 0.), radius=.025),))
    succulent = b.Recipe("suculenta", {
        "stem": b.Axis(b.Field(.10), core=soft, bark=soft, rules=(
            b.Rule("pads", "pad", at=(.4, .8), count=5, inclination=b.Field(1.), jitter=0.),)),
        "pad": SolidAxis(b.Field(.18), ellipticity=.18, core=soft, bark=soft, label="fleshy_pad")},
        (b.Root("stem", radius=.055),))
    radius, pitch, turns = .06, .10, 3
    a = pitch/math.tau
    scale = math.sqrt(radius*radius+a*a)
    climber = b.Recipe("trepadeira", {
        "vine": b.Axis(b.Field(scale*math.tau*turns),
            b.Growth(bend_n=b.Field(radius/scale**2), twist=b.Field(-a/scale**2)),
            rules=(b.Rule("leaves", "leaf", b.Field(12.), inclination=b.Field(1.)),), label="vine"),
        "leaf": b.Blade(b.Field(.06), b.Field(.035), relief=0.)},
        (b.Root("vine", position=(radius, 0., 0.), direction=b.unit((0., a, radius)),
                normal=(-1., 0., 0.), radius=.003),))
    return {p.name: p for p in (flower, fruit, roots, succulent, climber)}
