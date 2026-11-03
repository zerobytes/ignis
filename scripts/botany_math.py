#!/usr/bin/env python3
"""Independent numerical reference; no Ignis imports or asset generation.

See docs/procedural-botany-math.md. SI units, radians, right-handed +Y up.
Python >=3.10, standard library only. Run --help for diagnostic exports.
"""
from __future__ import annotations

import argparse
import bisect
from collections import deque
from dataclasses import asdict, dataclass, replace
import hashlib
import json
import math
import os
from pathlib import Path
import re
import tempfile

Vec = tuple[float, float, float]
UP: Vec = (0.0, 1.0, 0.0)
CONTEXT = {"u", "height", "radius", "level", "parent_length"}
VERSION = "botany-math-lab-1"


class Invalid(ValueError):
    """Invalid model, inadequate resolution or exhausted evaluation budget."""


def require(condition, message):
    if not condition:
        raise Invalid(message)


def add(a: Vec, b: Vec) -> Vec:
    return tuple(x + y for x, y in zip(a, b))


def sub(a: Vec, b: Vec) -> Vec:
    return tuple(x - y for x, y in zip(a, b))


def mul(a: Vec, s: float) -> Vec:
    return tuple(x * s for x in a)


def dot(a: Vec, b: Vec) -> float:
    return sum(x * y for x, y in zip(a, b))


def cross(a: Vec, b: Vec) -> Vec:
    return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def norm(a: Vec) -> float:
    return math.sqrt(dot(a, a))


def unit(a: Vec) -> Vec:
    n = norm(a)
    require(math.isfinite(n) and n > 1e-14, "zero/nonfinite direction")
    return mul(a, 1/n)


def lerp(a: Vec, b: Vec, t: float) -> Vec:
    return add(mul(a, 1-t), mul(b, t))


def rotate(v: Vec, rotation: Vec) -> Vec:
    angle = norm(rotation)
    if angle < 1e-14:
        return v
    axis = mul(rotation, 1/angle)
    c, s = math.cos(angle), math.sin(angle)
    return add(add(mul(v, c), mul(cross(axis, v), s)), mul(axis, dot(axis, v)*(1-c)))


def normal_to(t: Vec, preferred: Vec) -> Vec:
    n = sub(preferred, mul(t, dot(t, preferred)))
    if norm(n) < 1e-10:
        helper = min(((1., 0., 0.), UP, (0., 0., 1.)), key=lambda x: abs(dot(t, x)))
        n = sub(helper, mul(t, dot(t, helper)))
    return unit(n)


def transport(n: Vec, a: Vec, b: Vec) -> Vec:
    axis = cross(a, b)
    sine = norm(axis)
    cosine = max(-1., min(1., dot(a, b)))
    if sine > 1e-12:
        n = rotate(n, mul(axis, math.atan2(sine, cosine)/sine))
    elif cosine < 0:
        # The shortest rotation at 180 degrees is not unique: use old normal.
        n = rotate(n, mul(n, math.pi))
    return normal_to(b, n)


def uniform(seed: int, identity: str, channel: str) -> float:
    data = json.dumps([VERSION, seed, identity, channel], ensure_ascii=True,
                      separators=(",", ":"), allow_nan=False).encode("ascii")
    integer = int.from_bytes(hashlib.blake2b(data, digest_size=8).digest(), "big") >> 12
    # 52 bits keep both half-bin endpoints strictly inside (0,1) in binary64.
    return (integer + .5)/(1 << 52)


@dataclass(frozen=True)
class Curve:
    knots: tuple[tuple[float, float], ...] = ((0., 1.), (1., 1.))

    def at(self, x: float) -> float:
        if x <= self.knots[0][0]:
            return self.knots[0][1]
        for (a, y), (b, z) in zip(self.knots, self.knots[1:]):
            if x <= b:
                t = (x-a)/(b-a)
                return y + (z-y)*t*t*(3-2*t)
        return self.knots[-1][1]

    def validate(self):
        require(1 <= len(self.knots) <= 64, "curve needs 1..64 knots")
        require(all(math.isfinite(x) and math.isfinite(y) for x, y in self.knots),
                "nonfinite curve")
        require(all(a[0] < b[0] for a, b in zip(self.knots, self.knots[1:])),
                "curve abscissas must increase strictly")


@dataclass(frozen=True)
class Field:
    base: float
    drivers: tuple[tuple[str, Curve], ...] = ()
    spread: float = 0.

    def at(self, context: dict, seed: int, identity: str, channel: str) -> float:
        value = self.base
        if self.spread:
            value *= 1+self.spread*(2*uniform(seed, identity, channel)-1)
        for name, curve in self.drivers:
            value *= curve.at(context[name])
        require(math.isfinite(value), "nonfinite evaluated field: " + channel)
        return value

    def validate(self, nonnegative=False):
        require(math.isfinite(self.base) and math.isfinite(self.spread)
                and 0 <= self.spread < 1, "invalid field base/spread")
        require(len(self.drivers) <= len(CONTEXT), "too many field drivers")
        require(len({name for name, _ in self.drivers}) == len(self.drivers),
                "duplicate field driver")
        if nonnegative:
            require(self.base >= 0, "negative field")
        for name, curve in self.drivers:
            require(name in CONTEXT, "unknown context: " + name)
            curve.validate()
            if nonnegative:
                require(all(y >= 0 for _, y in curve.knots), "negative multiplier")


@dataclass(frozen=True)
class Growth:
    bend_n: Field = Field(0.)          # 1/m, material normal
    bend_b: Field = Field(0.)          # 1/m, material binormal
    attraction: Field = Field(0.)      # 1/m, geodesic directional relaxation
    direction: Vec = UP
    twist: Field = Field(0.)           # rad/m
    waviness: float = 0.               # 1/m
    waves: float = 1.                  # cycles per organ

    def validate(self):
        self.bend_n.validate()
        self.bend_b.validate()
        self.attraction.validate(nonnegative=True)
        self.twist.validate()
        require(len(self.direction) == 3, "growth direction needs three coordinates")
        unit(self.direction)
        require(math.isfinite(self.waviness) and self.waviness >= 0
                and math.isfinite(self.waves) and self.waves >= 0, "invalid waves")


@dataclass(frozen=True)
class Material:
    density: float = 650.
    young: float = 7e9
    shear: float = 5e8

    def validate(self):
        require(all(math.isfinite(x) and x > 0
                    for x in (self.density, self.young, self.shear)), "invalid material")


@dataclass(frozen=True)
class Rule:
    id: str
    target: str
    density: Field = Field(0.)         # stations/m
    at: tuple[float, ...] | None = None
    count: int = 1
    levels: tuple[int, int] = (0, 8)
    inclination: Field = Field(.7)     # relative to parent tangent
    phase: float = 0.                  # initial azimuth
    divergence: float = 2.399963229728653
    jitter: float = .1
    radius_ratio: float = .45
    roll: float = 0.


@dataclass(frozen=True)
class Axis:
    length: Field
    growth: Growth = Growth()
    tip_ratio: float = .08
    taper_power: float = 1.
    ellipticity: float = 1.
    bark_fraction: float = .08
    core: Material = Material()
    bark: Material = Material(400., 1e8, 5e7)
    rules: tuple[Rule, ...] = ()
    label: str = "axis"


@dataclass(frozen=True)
class Blade:
    length: Field
    width: Field
    growth: Growth = Growth()
    base_power: float = .7
    tip_power: float = 1.
    lobes: int = 0
    lobe_depth: float = 0.
    lobe_roundness: float = 1.
    asymmetry: float = 0.
    fold: float = 0.
    cup: float = 0.
    edge_wave: float = 0.
    vein_pairs: int = 5
    vein_pattern: str = "pinnate"
    vein_width: float = .0005
    relief: float = .0002
    thickness: float = .00025
    material: Material = Material(850., 2e7, 8e6)
    label: str = "leaf"


@dataclass(frozen=True)
class Root:
    definition: str
    position: Vec = (0., 0., 0.)
    direction: Vec = UP
    normal: Vec = (1., 0., 0.)
    radius: float = .04


@dataclass(frozen=True)
class Recipe:
    name: str
    definitions: dict[str, Axis | Blade]
    roots: tuple[Root, ...]
    seed: int = 42


@dataclass(frozen=True)
class Limits:
    organs: int = 10000
    depth: int = 8
    segments: int = 64
    station_bins: int = 128
    work: int = 3_000_000


@dataclass
class Budget:
    limit: int
    used: int = 0

    def take(self, n=1):
        require(type(n) is int and 0 <= n <= self.limit-self.used,
                "evaluation work budget exceeded")
        self.used += n


@dataclass(frozen=True)
class Sample:
    p: Vec
    t: Vec
    n: Vec

    @property
    def b(self):
        return cross(self.t, self.n)


@dataclass(frozen=True)
class Organ:
    id: str
    parent: str | None
    definition: str
    level: int
    insertion: float
    length: float
    radius: float
    width: float
    samples: tuple[Sample, ...]
    radius_profile: tuple[tuple[float, float], ...] = ()  # absolute carried radius, metres

    def sample(self, u: float) -> Sample:
        require(math.isfinite(u) and 0 <= u <= 1, "sample coordinate outside [0,1]")
        x = u*(len(self.samples)-1)
        i = min(int(x), len(self.samples)-2)
        f = x-i
        a, b = self.samples[i:i+2]
        t = unit(lerp(a.t, b.t, f))
        return Sample(lerp(a.p, b.p, f), t, normal_to(t, lerp(a.n, b.n, f)))


@dataclass(frozen=True)
class Plant:
    recipe: Recipe
    organs: tuple[Organ, ...]
    work: int
    rejections: tuple[tuple[str, str], ...] = ()


def validate(recipe: Recipe, limits: Limits):
    require(type(recipe.seed) is int, "seed must be integer")
    require(1 <= len(recipe.definitions) <= 128, "need 1..128 definitions")
    require(1 <= len(recipe.roots) <= 256, "need 1..256 roots")
    for name, value in asdict(limits).items():
        require(type(value) is int and value > 0, "invalid budget: " + name)
    require(2 <= limits.segments <= 4096 and 4 <= limits.station_bins <= 4096
            and limits.depth <= 32, "invalid numerical resolution/depth")
    for key, spec in recipe.definitions.items():
        require(re.fullmatch(r"[a-zA-Z0-9_-]{1,64}", key) is not None, "invalid definition id")
        require(isinstance(spec, (Axis, Blade)), "unknown organ definition")
        spec.length.validate(nonnegative=True)
        spec.growth.validate()
        if hasattr(spec, "validate_shape"):
            spec.validate_shape()
        if isinstance(spec, Axis):
            require(0 < spec.tip_ratio <= 1 and spec.taper_power > 0
                    and spec.ellipticity > 0 and 0 <= spec.bark_fraction < 1,
                    "invalid axis section")
            spec.core.validate()
            spec.bark.validate()
            require(len(spec.rules) <= 64 and len({r.id for r in spec.rules}) == len(spec.rules),
                    "duplicate/too many rules")
            for rule in spec.rules:
                require(re.fullmatch(r"[a-zA-Z0-9_-]{1,64}", rule.id) is not None,
                        "invalid rule id")
                require(rule.target in recipe.definitions, "missing rule target")
                rule.density.validate(nonnegative=True)
                rule.inclination.validate()
                require(type(rule.count) is int and 1 <= rule.count <= 64, "invalid whorl count")
                require(len(rule.levels) == 2 and all(type(x) is int for x in rule.levels)
                        and 0 <= rule.levels[0] <= rule.levels[1] <= 32, "invalid level interval")
                require(rule.jitter >= 0 and rule.radius_ratio > 0, "invalid birth rule")
                if rule.at is not None:
                    require(len(rule.at) <= 4096 and all(0 <= u <= 1 for u in rule.at)
                            and all(a < b for a, b in zip(rule.at, rule.at[1:])),
                            "explicit stations must increase within [0,1]")
        else:
            spec.width.validate(nonnegative=True)
            spec.material.validate()
            require(spec.base_power > 0 and spec.tip_power > 0 and type(spec.lobes) is int
                    and 0 <= spec.lobes <= 128 and 0 <= spec.lobe_depth < 1
                    and spec.lobe_roundness > 0 and abs(spec.asymmetry) < 1
                    and spec.thickness > 0 and spec.vein_width > 0
                    and spec.relief >= 0 and type(spec.vein_pairs) is int
                    and 0 <= spec.vein_pairs <= 64 and spec.vein_pattern in ("pinnate", "parallel"),
                    "invalid blade")
    # Catch NaN/inf in all dataclass numbers, including optional shape controls.
    def finite(value):
        if isinstance(value, dict):
            return all(finite(x) for x in value.values())
        if isinstance(value, (tuple, list)):
            return all(finite(x) for x in value)
        return not isinstance(value, float) or math.isfinite(value)
    require(finite(asdict(recipe)), "nonfinite recipe value")
    for root in recipe.roots:
        require(root.definition in recipe.definitions and root.radius > 0, "invalid root")
        require(len(root.position) == len(root.direction) == len(root.normal) == 3,
                "root needs three-dimensional vectors")
        unit(root.direction)


def radius_at(organ: Organ, spec: Axis, u: float):
    if hasattr(spec, "radial_multiplier"):
        authored = organ.radius*spec.radial_multiplier(u)
    else:
        authored = organ.radius*(spec.tip_ratio+(1-spec.tip_ratio)*(1-u)**spec.taper_power)
    if organ.radius_profile:
        carried = organ.radius_profile[-1][1]
        for (a, ra), (c, rc) in zip(organ.radius_profile, organ.radius_profile[1:]):
            if u <= c:
                t = max(0., (u-a)/(c-a))
                carried = ra*(1-t)+rc*t
                break
        return max(authored, carried)
    return authored


def context_at(organ: Organ, spec: Axis, u: float):
    return dict(u=u, height=organ.sample(u).p[1], radius=radius_at(organ, spec, u),
                level=organ.level, parent_length=organ.length)


def curvature(growth: Growth, sample: Sample, u: float, context: dict,
              seed: int, identity: str, target: Vec, phase: float) -> Vec:
    c = dict(context, u=u, height=sample.p[1])
    kn = growth.bend_n.at(c, seed, identity, "bend_n")
    kb = growth.bend_b.at(c, seed, identity, "bend_b")
    kn += growth.waviness*math.sin(math.tau*growth.waves*u+phase)
    kb += growth.waviness*math.cos(math.tau*growth.waves*u+phase)
    k = add(mul(sample.n, kn), mul(sample.b, kb))
    cosine = max(-1., min(1., dot(sample.t, target)))
    projection = sub(target, mul(sample.t, cosine))
    if norm(projection) < 1e-10:
        correction = mul(sample.n, math.pi if cosine < 0 else 0.)
    else:
        correction = mul(unit(projection), math.acos(cosine))
    return add(k, mul(correction, growth.attraction.at(c, seed, identity, "attraction")))


def path(length: float, start: Sample, growth: Growth, context: dict, seed: int,
         identity: str, limits: Limits, budget: Budget, axis: Axis | None, radius: float,
         environment=None):
    budget.take(limits.segments*2+1)
    ds = length/limits.segments
    target = unit(growth.direction)
    phase = math.tau*uniform(seed, identity, "curve_phase") if growth.waviness else 0.
    out = [start]
    for i in range(limits.segments):
        u = i/limits.segments
        multiplier = (axis.radial_multiplier(u) if axis is not None and hasattr(axis, "radial_multiplier")
                      else axis.tip_ratio+(1-axis.tip_ratio)*(1-u)**axis.taper_power if axis is not None else 1.)
        c = dict(context, radius=radius*multiplier if axis is not None else context["radius"])
        a = out[-1]
        omega = cross(a.t, curvature(growth, a, u, c, seed, identity, target, phase))
        if environment is not None:
            omega = add(omega, cross(a.t, environment.curvature(a, dict(c, u=u, height=a.p[1]))))
        omega = add(omega, mul(a.t, growth.twist.at(dict(c, u=u, height=a.p[1]),
                                                  seed, identity, "twist")))
        require(norm(omega)*ds <= .35, "insufficient curve resolution; increase segments")
        mid_t = unit(rotate(a.t, mul(omega, ds/2)))
        mid_n = normal_to(mid_t, rotate(a.n, mul(omega, ds/2)))
        half = Sample(add(a.p, mul(a.t, ds/2)), mid_t, mid_n)
        mid_c = dict(c)
        if axis is not None:
            v = (i+.5)/limits.segments
            multiplier = (axis.radial_multiplier(v) if hasattr(axis, "radial_multiplier") else
                          axis.tip_ratio+(1-axis.tip_ratio)*(1-v)**axis.taper_power)
            mid_c["radius"] = radius*multiplier
        omega_mid = cross(half.t, curvature(growth, half, (i+.5)/limits.segments,
                                          mid_c, seed, identity, target, phase))
        if environment is not None:
            omega_mid = add(omega_mid, cross(half.t, environment.curvature(
                half, dict(mid_c, u=(i+.5)/limits.segments, height=half.p[1]))))
        twist = growth.twist.at(dict(mid_c, u=(i+.5)/limits.segments, height=half.p[1]),
                                seed, identity, "twist")
        omega_mid = add(omega_mid, mul(half.t, twist))
        require(norm(omega_mid)*ds <= .35, "insufficient midpoint frame resolution")
        t = unit(rotate(a.t, mul(omega_mid, ds)))
        n = normal_to(t, rotate(a.n, mul(omega_mid, ds)))
        out.append(Sample(add(a.p, mul(mid_t, ds)), t, n))
    return tuple(out)


def stations(organ: Organ, spec: Axis, rule: Rule, seed: int,
             limits: Limits, budget: Budget):
    if rule.at is not None:
        budget.take(len(rule.at))
        return rule.at
    n = limits.station_bins
    budget.take(n+1)
    density = [rule.density.at(context_at(organ, spec, i/n), seed, organ.id,
                              rule.id+":density") for i in range(n+1)]
    h = organ.length/n
    cumulative = [0.]
    for a, b in zip(density, density[1:]):
        cumulative.append(cumulative[-1]+h*(a+b)/2)
    require(math.isfinite(cumulative[-1]), "nonfinite cumulative intensity")
    phase = uniform(seed, organ.id, rule.id+":station_phase")
    count = max(0, math.ceil(cumulative[-1]-phase))
    require(count <= limits.organs, "station count exceeds organ budget")
    budget.take(count)
    out = []
    for j in range(count):
        target = phase+j
        i = min(bisect.bisect_right(cumulative, target)-1, n-1)
        a, b = density[i:i+2]
        y = (target-cumulative[i])/h
        delta = b-a
        if abs(delta) < 1e-14:
            require(a > 0, "invalid zero-density inversion")
            f = y/a
        else:
            disc = max(0., a*a+2*delta*y)
            denominator = a+math.sqrt(disc)
            require(denominator > 0, "invalid intensity inversion")
            f = 2*y/denominator
        out.append((i+max(0., min(1., f)))/n)
    return tuple(out)


def generate(recipe: Recipe, limits: Limits = Limits(), environment=None) -> Plant:
    validate(recipe, limits)
    if environment is not None:
        environment = environment.begin()
    budget = Budget(limits.work)
    queue = deque()
    for i, root in enumerate(recipe.roots):
        t = unit(root.direction)
        start = Sample(root.position, t, normal_to(t, root.normal))
        queue.append((f"root{i}", None, root.definition, 0, 0., start, root.radius,
                      dict(u=0., height=root.position[1], radius=root.radius,
                           level=0, parent_length=0.)))
    require(len(queue) <= limits.organs, "root count exceeds organ budget")
    organs = []
    while queue:
        identity, parent, key, level, insertion, start, radius, birth = queue.popleft()
        budget.take()
        require(level <= limits.depth, "organ depth budget exceeded")
        spec = recipe.definitions[key]
        length = spec.length.at(birth, recipe.seed, identity, "length")
        width = spec.width.at(birth, recipe.seed, identity, "width") if isinstance(spec, Blade) else 0.
        require(length > 1e-9 and (not isinstance(spec, Blade) or width > 1e-9)
                and math.isfinite(radius) and radius > 0, "degenerate organ dimensions")
        samples = path(length, start, spec.growth, dict(birth, level=level), recipe.seed,
                       identity, limits, budget, spec if isinstance(spec, Axis) else None, radius,
                       environment)
        organ = Organ(identity, parent, key, level, insertion, length, radius, width, samples)
        if environment is not None and not environment.admit(organ, spec, organs, birth, recipe.seed):
            continue
        organs.append(organ)
        if isinstance(spec, Blade):
            continue
        for rule in sorted(spec.rules, key=lambda x: x.id):
            if not rule.levels[0] <= level <= rule.levels[1]:
                continue
            for j, u in enumerate(stations(organ, spec, rule, recipe.seed, limits, budget)):
                require(len(organs)+len(queue)+rule.count <= limits.organs,
                        "organ count budget exceeded")
                budget.take(rule.count)
                sample = organ.sample(u)
                birth_context = context_at(organ, spec, u)
                for k in range(rule.count):
                    child_id = f"{identity}/{rule.id}:{j}:{k}"
                    alpha = rule.inclination.at(birth_context, recipe.seed, child_id, "inclination")
                    phi = rule.phase+j*rule.divergence+math.tau*k/rule.count
                    phi += rule.jitter*(2*uniform(recipe.seed, child_id, "azimuth")-1)
                    radial = add(mul(sample.n, math.cos(phi)), mul(sample.b, math.sin(phi)))
                    t = unit(add(mul(sample.t, math.cos(alpha)), mul(radial, math.sin(alpha))))
                    n = normal_to(t, radial)
                    n = normal_to(t, rotate(n, mul(t, rule.roll)))
                    queue.append((child_id, identity, rule.target, level+1, u,
                                  Sample(sample.p, t, n), birth_context["radius"]*rule.radius_ratio,
                                  birth_context))
    result = Plant(recipe, tuple(sorted(organs, key=lambda x: x.id)), budget.used)
    if environment is not None:
        result = replace(result, rejections=environment.finish(result))
    return result


def blade_halfwidth(spec: Blade, length: float, width: float, u: float):
    del length  # Width law is dimensionless in u.
    a, b = spec.base_power, spec.tip_power
    maximum = (a/(a+b))**a*(b/(a+b))**b
    envelope = u**a*(1-u)**b/maximum
    valleys = ((1-math.cos(math.tau*spec.lobes*u))/2)**spec.lobe_roundness
    return width/2*envelope*(1-spec.lobe_depth*valleys)


def veins(spec: Blade):
    """Material-coordinate segments; geometry remains independent of relief."""
    out = [((.02, 0.), (.98, 0.))]
    for i in range(spec.vein_pairs):
        u = .12+.72*(i+.5)/max(1, spec.vein_pairs)
        for side in (-1, 1):
            if spec.vein_pattern == "parallel":
                v = side*.85*(i+1)/(spec.vein_pairs+1)
                out.append(((.05, v), (.95, v)))
            else:
                out.append(((u, 0.), (min(.97, u+.15), side*.92)))
    return tuple(out)


def segment_distance(u, v, a, b, length, width):
    p, q, x = (a[0]*length, a[1]*width/2), (b[0]*length, b[1]*width/2), (u*length, v*width/2)
    d = (q[0]-p[0], q[1]-p[1])
    denominator = d[0]*d[0]+d[1]*d[1]
    t = max(0., min(1., ((x[0]-p[0])*d[0]+(x[1]-p[1])*d[1])/denominator))
    return math.hypot(x[0]-p[0]-t*d[0], x[1]-p[1]-t*d[1])


def blade_surface(organ: Organ, spec: Blade, u: float, v: float, network=None) -> Vec:
    require(math.isfinite(u) and math.isfinite(v) and 0 <= u <= 1 and -1 <= v <= 1,
            "blade coordinate outside material domain")
    if hasattr(spec, "surface"):
        return spec.surface(organ, u, v, network)
    halfwidth = blade_halfwidth(spec, organ.length, organ.width, u)
    x = v*halfwidth*(1+spec.asymmetry*(1 if v >= 0 else -1))
    z = spec.fold*abs(x)+spec.cup*x*x/organ.width
    z += spec.edge_wave*math.sin(math.tau*3*u)*v*v*math.sin(math.pi*u)
    if spec.relief:
        for i, (a, b) in enumerate(veins(spec) if network is None else network):
            distance = segment_distance(u, v, a, b, organ.length, organ.width)
            z += spec.relief*(1 if i == 0 else .55)*math.exp(-.5*(distance/spec.vein_width)**2) \
                 *(1-v*v)*math.sin(math.pi*u)
    sample = organ.sample(u)
    return add(sample.p, add(mul(sample.n, x), mul(sample.b, z)))


def axis_section(organ: Organ, spec: Axis, u: float):
    a, b = radius_at(organ, spec, u), radius_at(organ, spec, u)*spec.ellipticity
    if a*b == 0:
        return dict(area_m2=0., mass_per_length_kg_m=0., EA_N=0., EI_N_Nm2=0.,
                    EI_B_Nm2=0., GJ_approx_Nm2=0.)
    scale = 1-spec.bark_fraction
    area = math.pi*a*b
    in_ = math.pi*a*b**3/4
    ib = math.pi*a**3*b/4
    j = math.pi*a**3*b**3/(a*a+b*b)
    core_area, core_in, core_ib, core_j = area*scale**2, in_*scale**4, ib*scale**4, j*scale**4
    return dict(area_m2=area, mass_per_length_kg_m=spec.core.density*core_area+
                spec.bark.density*(area-core_area),
                EA_N=spec.core.young*core_area+spec.bark.young*(area-core_area),
                EI_N_Nm2=spec.core.young*core_in+spec.bark.young*(in_-core_in),
                EI_B_Nm2=spec.core.young*core_ib+spec.bark.young*(ib-core_ib),
                GJ_approx_Nm2=spec.core.shear*core_j+spec.bark.shear*(j-core_j))


def properties(organ: Organ, spec: Axis | Blade, resolution=48):
    require(type(resolution) is int and 4 <= resolution <= 512, "invalid property resolution")
    if isinstance(spec, Axis):
        mass = sum(axis_section(organ, spec, (i+.5)/resolution)["mass_per_length_kg_m"]
                   for i in range(resolution))*organ.length/resolution
        return dict(mass_kg=mass, base_section=axis_section(organ, spec, 0.))
    area = 0.
    network = veins(spec)
    # Stay inside singular tips; differences are central in material coordinates.
    epsilon = min(1e-5, .05/resolution)
    for i in range(resolution):
        u = (i+.5)/resolution
        for j in range(resolution):
            v = -1+2*(j+.5)/resolution
            if hasattr(spec, "contains_uv") and not spec.contains_uv(u, v):
                continue
            du = mul(sub(blade_surface(organ, spec, u+epsilon, v, network),
                         blade_surface(organ, spec, u-epsilon, v, network)), .5/epsilon)
            dv = mul(sub(blade_surface(organ, spec, u, v+epsilon, network),
                         blade_surface(organ, spec, u, v-epsilon, network)), .5/epsilon)
            area += norm(cross(du, dv))*2/resolution**2
    volume = area*spec.thickness
    return dict(area_m2=area, volume_thin_m3=volume, mass_kg=spec.material.density*volume,
                plate_D_Nm=spec.material.young*spec.thickness**3/(12*(1-.3**2)))


LEAFLET_LENGTH_PROFILE = Curve(((0., .16), (.18, 1.), (.42, .94), (.75, .50), (1., .06)))
LEAFLET_WIDTH_PROFILE = Curve(((0., .35), (.18, 1.), (.6, .8), (1., .10)))


def compound_leaf_recipe(length_profile: Curve = LEAFLET_LENGTH_PROFILE,
                         width_profile: Curve = LEAFLET_WIDTH_PROFILE,
                         seed: int = 42) -> Recipe:
    """Pinnate leaf: leaflet dimensions depend on their insertion along the rachis.

    These are independent positive multipliers. Curve() gives uniform size.
    No new growth algorithm: both controls use the existing parent-u fields.
    """
    return Recipe("folha_pinada", {
        "rachis": Axis(Field(1.4), Growth(bend_n=Field(1.2)), rules=(
            Rule("leaflets", "leaflet", Field(30.), count=2,
                 inclination=Field(1.32), divergence=0., jitter=0., radius_ratio=.3),),
            label="rachis"),
        "leaflet": Blade(Field(.30, (("u", length_profile),)),
                         Field(.016, (("u", width_profile),)),
                         growth=Growth(bend_b=Field(.7)), base_power=.35, tip_power=1.3,
                         fold=.1, vein_pairs=2, vein_pattern="parallel", vein_width=.0002,
                         relief=.00003, thickness=.00015, label="leaflet")},
        (Root("rachis", direction=unit((.25, 1., 0.)), radius=.004),), seed)


def examples(seed=42):
    """Synthetic configurations of exactly the same evaluator."""
    leaf = Blade(Field(.18, spread=.12), Field(.075, spread=.1),
                 growth=Growth(bend_b=Field(1.), twist=Field(.6)), fold=.08, cup=.15)
    grass_blade = Blade(Field(.6, spread=.3), Field(.014, spread=.25),
                        growth=Growth(bend_b=Field(1.), attraction=Field(.5)),
                        base_power=.25, tip_power=1.8, fold=.3, vein_pattern="parallel",
                        vein_pairs=2, vein_width=.0003, relief=.00005, thickness=.00015)
    grass_roots = tuple(Root("base", position=(.035*math.cos(i*2.4), 0., .035*math.sin(i*2.4)),
                            direction=unit((.5*math.cos(i*2.4), 1., .5*math.sin(i*2.4))),
                            radius=.0015) for i in range(12))
    grass = Recipe("grama", {
        "base": Axis(Field(.035), rules=(Rule("blades", "blade", at=(.1, .8),
                     inclination=Field(.35), radius_ratio=.8),), label="basal_shoot"),
        "blade": grass_blade}, grass_roots, seed)
    tree = Recipe("arvore", {
        "trunk": Axis(Field(4., spread=.08), Growth(attraction=Field(.5), waviness=.07),
                      rules=(Rule("limbs", "limb", Field(3., (("u", Curve(((0., 0.),
                      (.22, 0.), (.4, 1.), (.88, 1.), (1., 0.)))),)),
                      inclination=Field(.95, spread=.2), radius_ratio=.36),), label="trunk"),
        "limb": Axis(Field(1.5, (("u", Curve(((0., 1.), (1., .4)))),), spread=.2),
                     Growth(attraction=Field(.6), waviness=.1), rules=(
                     Rule("twigs", "twig", Field(3.5), inclination=Field(.75), radius_ratio=.3),)),
        "twig": Axis(Field(.45, spread=.2), Growth(attraction=Field(.8)), rules=(
                     Rule("leaves", "leaf", Field(18.), inclination=Field(1.), radius_ratio=.3),)),
        "leaf": leaf}, (Root("trunk", radius=.14),), seed)
    petals = Blade(Field(.055), Field(.028), base_power=.8, tip_power=.45,
                   growth=Growth(bend_b=Field(-8.)), cup=.2, vein_pairs=0,
                   relief=0., thickness=.00015, label="petal")
    shrub = Recipe("arbusto_florido", {
        "stem": Axis(Field(.65, spread=.2), Growth(attraction=Field(1.)), rules=(
            Rule("branches", "shoot", Field(7.), inclination=Field(.9), radius_ratio=.4),
            Rule("foliage", "leaf", Field(10.), inclination=Field(1.1), radius_ratio=.3))),
        "shoot": Axis(Field(.25, spread=.2), Growth(attraction=Field(2.)), rules=(
            Rule("foliage", "leaf", Field(12.), inclination=Field(1.)),
            Rule("flower", "receptacle", at=(1.,), inclination=Field(0.), radius_ratio=.4))),
        "receptacle": Axis(Field(.01), rules=(Rule("petals", "petal", at=(1.,), count=5,
            inclination=Field(1.2), divergence=0., jitter=0., radius_ratio=.5),), label="receptacle"),
        "leaf": leaf, "petal": petals}, (Root("stem", radius=.014),), seed)
    fern = Recipe("folhagem_composta", {
        "rachis": Axis(Field(1.), Growth(bend_b=Field(.7)), rules=(
            Rule("pinnae", "pinna", Field(12., (("u", Curve(((0., 0.), (.1, 1.),
                (.85, 1.), (1., 0.)))),)), count=2, inclination=Field(1.25),
                divergence=.12, jitter=.03, radius_ratio=.35),), label="rachis"),
        "pinna": Axis(Field(.23, (("u", Curve(((0., .9), (.3, 1.), (1., .1)))),)),
            Growth(bend_b=Field(.4)), rules=(Rule("leaflets", "leaflet", Field(40.),
                count=2, inclination=Field(1.2), divergence=0., jitter=.03),), label="pinna"),
        "leaflet": Blade(Field(.035), Field(.012), lobes=3, lobe_depth=.22,
            fold=.1, vein_pairs=2, vein_width=.0002, relief=.00004, label="leaflet")},
        (Root("rachis", direction=unit((.15, 1., 0.)), radius=.004),), seed)
    return {p.name: p for p in (grass, tree, shrub, fern, compound_leaf_recipe(seed=seed))}


def atomic_json(destination: Path, value):
    """Serialize completely before touching disk; failed evaluation keeps old output."""
    data = (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2,
                       allow_nan=False)+"\n").encode("utf-8")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temp = None
    try:
        with tempfile.NamedTemporaryFile(dir=destination.parent, prefix=".botany-", delete=False) as f:
            temp = Path(f.name)
            f.write(data)
            f.flush()
            os.fsync(f.fileno())
        os.replace(temp, destination)
    finally:
        if temp is not None:
            temp.unlink(missing_ok=True)


def report(plant: Plant):
    counts = {}
    for organ in plant.organs:
        label = plant.recipe.definitions[organ.definition].label
        counts[label] = counts.get(label, 0)+1
    return dict(format=VERSION, purpose="diagnostic-report-not-ignis-asset",
                recipe=asdict(plant.recipe), counts=counts, work=plant.work,
                organs=[asdict(organ) for organ in plant.organs])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--example", choices=[*examples(), "all"], default="all")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--segments", type=int, default=64)
    parser.add_argument("--output", type=Path, default=Path("artifacts/botany-math"))
    args = parser.parse_args()
    recipes = examples(args.seed)
    for name, recipe in recipes.items():
        if args.example not in ("all", name):
            continue
        plant = generate(recipe, Limits(segments=args.segments))
        value = report(plant)
        atomic_json(args.output/(name+".json"), value)
        print(json.dumps(dict(example=name, organs=len(plant.organs), counts=value["counts"],
                              work=plant.work), ensure_ascii=False))


if __name__ == "__main__":
    main()
