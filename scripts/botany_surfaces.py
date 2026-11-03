"""Palmate laminae and independent vascular graphs; all lengths are SI."""
from dataclasses import dataclass
from functools import cached_property
import math

import botany_math as b


@dataclass(frozen=True)
class Hole:
    x: float
    y: float
    rx: float
    ry: float

    def contains(self, x, y):
        return ((x-self.x)/self.rx)**2+((y-self.y)/self.ry)**2 < 1


@dataclass(frozen=True)
class VeinEdge:
    a: int
    c: int
    order: int
    radius: float


@dataclass(frozen=True)
class VeinGraph:
    nodes: tuple[tuple[float, float], ...]  # x/W, y/L
    edges: tuple[VeinEdge, ...]

    def validate(self):
        b.require(2 <= len(self.nodes) <= 4096 and len(self.edges) <= 8192, "vein graph budget")
        b.require(all(len(p) == 2 and all(math.isfinite(x) for x in p) for p in self.nodes),
                  "nonfinite vascular node")
        pairs = set()
        adjacency = [[] for _ in self.nodes]
        for edge in self.edges:
            b.require(type(edge.a) is int and type(edge.c) is int and type(edge.order) is int
                      and 0 <= edge.a < len(self.nodes) and 0 <= edge.c < len(self.nodes)
                      and edge.a != edge.c and math.isfinite(edge.radius) and edge.radius > 0
                      and edge.order >= 0, "invalid vein edge")
            pair = tuple(sorted((edge.a, edge.c)))
            b.require(pair not in pairs, "duplicate vein edge")
            pairs.add(pair)
            adjacency[edge.a].append(edge.c)
            adjacency[edge.c].append(edge.a)
        visited, todo = set(), [0]
        while todo:
            node = todo.pop()
            if node not in visited:
                visited.add(node)
                todo.extend(adjacency[node])
        b.require(len(visited) == len(self.nodes), "disconnected vascular graph")

    def segments(self):
        return tuple((self.nodes[e.a], self.nodes[e.c], e) for e in self.edges)

    def mechanics(self, length, width, young=2e8, density=950.):
        # Effective cylindrical reinforcements, not an anatomical volume partition.
        b.require(all(math.isfinite(x) and x > 0 for x in (length, width, young, density)),
                  "invalid vascular material/scale")
        volume = 0.
        tensor = [[0., 0.], [0., 0.]]
        for a, c, edge in self.segments():
            dx, dy = (c[0]-a[0])*width, (c[1]-a[1])*length
            distance = math.hypot(dx, dy)
            b.require(distance > 1e-12, "degenerate vascular segment")
            volume += math.pi*edge.radius**2*distance
            stiffness = young*math.pi*edge.radius**4/4*distance
            direction = (dx/distance, dy/distance)
            for i in range(2):
                for j in range(2):
                    tensor[i][j] += stiffness*direction[i]*direction[j]
        return dict(additive_volume_m3=volume, additive_mass_kg=volume*density,
                    integrated_directional_EI_Nm3=tensor)


@dataclass(frozen=True)
class PalmateBlade(b.Blade):
    tips: int = 5
    spread: float = 2.25             # half angular span around +axial direction
    cut_depth: float = .55
    rounding: float = .75
    valley_rounding: float = 2.
    tip_lengths: tuple[float, ...] = ()
    tip_angles: tuple[float, ...] = ()
    base_radius: float = .30
    holes: tuple[Hole, ...] = ()
    vein_orders: int = 3
    reticulate: bool = False
    vein_radius: float = .00035

    @property
    def angles(self):
        return self.tip_angles or tuple(-self.spread+2*self.spread*i/(self.tips-1)
                                       for i in range(self.tips))

    @property
    def lengths(self):
        return self.tip_lengths or (1.,)*self.tips

    def validate_shape(self):
        b.require(type(self.tips) is int and 3 <= self.tips <= 32
                  and 0 < self.spread < math.pi and 0 <= self.cut_depth < .995
                  and .2 <= self.rounding <= 4 and 1.1 <= self.valley_rounding <= 4
                  and 0 < self.base_radius <= 1,
                  "invalid palmate contour")
        b.require(len(self.angles) == len(self.lengths) == self.tips
                  and all(-math.pi < a < math.pi for a in self.angles)
                  and all(a < c for a, c in zip(self.angles, self.angles[1:]))
                  and all(.05 <= x <= 2 for x in self.lengths), "invalid palmate tips")
        b.require(1 <= self.vein_orders <= 3 and 0 < self.vein_radius < .01
                  and len(self.holes) <= 32, "invalid vein/hole parameters")
        for hole in self.holes:
            b.require(hole.rx > 0 and hole.ry > 0, "invalid hole")
            # Require a closed hole strictly inside the lamina, disjoint from others.
            b.require(all(self.inside(hole.x+hole.rx*math.cos(i*math.tau/64),
                                     hole.y+hole.ry*math.sin(i*math.tau/64), holes=False)
                          for i in range(64)), "hole reaches leaf margin")
            b.require(not hole.contains(0., 0.) and not hole.contains(0., self.hub_y),
                      "hole removes vascular origin")
        for i, h in enumerate(self.holes):
            for k in self.holes[i+1:]:
                b.require(math.hypot(h.x-k.x, h.y-k.y) > max(h.rx, h.ry)+max(k.rx, k.ry),
                          "holes overlap or uncertain separation")

    @cached_property
    def knots(self):
        out = [(-math.pi, self.base_radius)]
        out.append(((-math.pi+self.angles[0])/2,
                    min(self.base_radius, self.lengths[0])*(1-self.cut_depth)))
        for i, (a, r) in enumerate(zip(self.angles, self.lengths)):
            if i:
                previous_angle, previous_radius = self.angles[i-1], self.lengths[i-1]
                out.append(((a+previous_angle)/2, min(r, previous_radius)*(1-self.cut_depth)))
            out.append((a, r))
        out.append(((math.pi+self.angles[-1])/2,
                    min(self.base_radius, self.lengths[-1])*(1-self.cut_depth)))
        out.append((math.pi, self.base_radius))
        return tuple(out)

    def radial(self, angle):
        for (a, ra), (c, rc) in zip(self.knots, self.knots[1:]):
            if angle <= c:
                t = max(0., (angle-a)/(c-a))
                def rise(x):
                    numerator = x**self.valley_rounding
                    return numerator/(numerator+(1-x)**self.rounding)
                return (ra+(rc-ra)*rise(t) if rc >= ra else
                        rc+(ra-rc)*rise(1-t))
        return self.base_radius

    @cached_property
    def x_extent(self):
        return max(abs(math.sin(i*math.tau/2048-math.pi)*self.radial(i*math.tau/2048-math.pi))
                   for i in range(2049))

    @property
    def hub_y(self):
        return self.base_radius/(1+self.base_radius)

    def material_xy(self, radial, angular):
        theta = math.pi*angular
        r = radial*self.radial(theta)
        return (math.sin(theta)*r/(2*self.x_extent),
                (self.base_radius+math.cos(theta)*r)/(1+self.base_radius))

    def inside(self, x, y, holes=True):
        xx, yy = x*2*self.x_extent, y*(1+self.base_radius)-self.base_radius
        theta = math.atan2(xx, yy)
        return (math.hypot(xx, yy) <= self.radial(theta)+1e-10
                and (not holes or not any(h.contains(x, y) for h in self.holes)))

    def contains_uv(self, radial, angular):
        return self.inside(*self.material_xy(radial, angular))

    def first_hole(self, a, c):
        first = 1.
        for hole in self.holes:
            x, y = (a[0]-hole.x)/hole.rx, (a[1]-hole.y)/hole.ry
            dx, dy = (c[0]-a[0])/hole.rx, (c[1]-a[1])/hole.ry
            aa, bb, cc = dx*dx+dy*dy, 2*(x*dx+y*dy), x*x+y*y-1
            disc = bb*bb-4*aa*cc
            if aa > 0 and disc > 0:
                entry = (-bb-math.sqrt(disc))/(2*aa)
                exit_ = (-bb+math.sqrt(disc))/(2*aa)
                if exit_ > 0 and entry < 1:
                    first = min(first, max(0., entry))
        return first

    def uv_from_xy(self, x, y):
        xx, yy = x*2*self.x_extent, y*(1+self.base_radius)-self.base_radius
        theta = math.atan2(xx, yy)
        return (math.hypot(xx, yy)/self.radial(theta), theta/math.pi)

    @cached_property
    def graph(self):
        nodes = [(0., 0.), (0., self.hub_y)]
        edges = [VeinEdge(0, 1, 0, self.vein_radius)]

        def connect(a, point, order, radius):
            start = nodes[a]
            # Clip every segment before a hole or concave margin (sampled admission).
            fraction = self.first_hole(start, point)
            if fraction < 1:
                fraction *= 1-1e-7
            for i in range(1, 65):
                f = i/64
                if not self.inside(start[0]+f*(point[0]-start[0]), start[1]+f*(point[1]-start[1])):
                    fraction = min(fraction, (i-1)/64)
                    break
            if fraction < .04:
                return a
            end = (start[0]+fraction*(point[0]-start[0]), start[1]+fraction*(point[1]-start[1]))
            nodes.append(end)
            edges.append(VeinEdge(a, len(nodes)-1, order, radius))
            return len(nodes)-1

        for angle in self.angles:
            previous = 1
            for level, f in enumerate((.3, .6, .94)):
                point = self.material_xy(f, angle/math.pi)
                main = connect(previous, point, 0, self.vein_radius*(1-.5*f))
                previous = main
                if self.vein_orders >= 2:
                    for side in (-1, 1):
                        target_angle = angle+side*.25*self.spread/self.tips
                        endpoint = self.material_xy(min(.98, f+.15), target_angle/math.pi)
                        secondary = connect(main, endpoint, 1, self.vein_radius*.42)
                        if self.vein_orders == 3 and secondary != main:
                            tertiary = self.material_xy(min(.99, f+.20),
                                                       (target_angle+side*.08)/math.pi)
                            connect(secondary, tertiary, 2, self.vein_radius*.18)
        if self.reticulate:
            existing = {tuple(sorted((e.a, e.c))) for e in edges}
            candidates = []
            for i, a in enumerate(nodes):
                if i < 2:
                    continue
                for j in range(i+1, len(nodes)):
                    c = nodes[j]
                    distance = math.hypot(a[0]-c[0], a[1]-c[1])
                    if ((i, j) not in existing and .02 < distance < .11 and self.first_hole(a, c) >= 1
                        and all(self.inside(a[0]+k/32*(c[0]-a[0]), a[1]+k/32*(c[1]-a[1]))
                                for k in range(33))):
                        candidates.append((distance, i, j))
            for _, i, j in sorted(candidates)[:len(nodes)//2]:
                edges.append(VeinEdge(i, j, 2, self.vein_radius*.16))
        result = VeinGraph(tuple(nodes), tuple(edges))
        result.validate()
        return result

    def surface_xy(self, organ, x, y, graph=None):
        along = max(0., min(1., y))
        sample = organ.sample(along)
        center = b.add(sample.p, b.mul(sample.t, (y-along)*organ.length))
        xx = x*organ.width
        z = self.fold*abs(xx)+self.cup*xx*xx/organ.width
        if self.relief:
            for a, c, edge in (graph or self.graph).segments():
                distance = b.segment_distance(y, x*2, (a[1], a[0]*2), (c[1], c[0]*2),
                                              organ.length, organ.width)
                z += self.relief/(1+edge.order)*math.exp(-.5*(distance/self.vein_width)**2)
        return b.add(center, b.add(b.mul(sample.n, xx), b.mul(sample.b, z)))

    def surface(self, organ, radial, angular, network=None):
        return self.surface_xy(organ, *self.material_xy(radial, angular))

    def outline_points(self, organ, count=256):
        return tuple(self.surface(organ, 1., -1+2*i/count) for i in range(count))


@dataclass(frozen=True)
class SolidAxis(b.Axis):
    """Ellipsoidal solid of revolution with an elliptical transverse section."""
    def radial_multiplier(self, u):
        return math.sqrt(max(0., 1-(2*u-1)**2))


def leaf_examples():
    return {
        "platano": PalmateBlade(b.Field(.22), b.Field(.24), tips=5, cut_depth=.42,
                                rounding=.55, tip_lengths=(.72, .9, 1., .9, .72),
                                relief=.0002, vein_width=.0012, reticulate=True),
        "mandioca": PalmateBlade(b.Field(.26), b.Field(.28), tips=7, spread=2.25,
                                 cut_depth=.93, rounding=.55,
                                 tip_lengths=(.65, .85, .96, 1., .96, .85, .65),
                                 vein_width=.0008, relief=.00012),
        "leque": PalmateBlade(b.Field(.23), b.Field(.30), tips=15, spread=1.95,
                              cut_depth=.48, rounding=.8, vein_orders=2,
                              vein_radius=.0002, relief=.00008),
        "perfurada": PalmateBlade(b.Field(.22), b.Field(.23), tips=5, cut_depth=.20,
                                  rounding=1.2, reticulate=True,
                                  holes=(Hole(-.10, .45, .027, .06),
                                         Hole(.10, .45, .027, .06)), vein_width=.001),
    }


def leaf_plant(spec, name="leaf"):
    return b.generate(b.Recipe(name, {"leaf": spec}, (b.Root("leaf"),)))
