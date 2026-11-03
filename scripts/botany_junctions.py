"""Local implicit junctions; bounded marching tetrahedra, shared edge identity."""
from collections import Counter
from dataclasses import dataclass
from itertools import permutations
import math

import botany_math as b


@dataclass(frozen=True)
class Tube:
    a: b.Vec
    c: b.Vec
    ra: float
    rc: float

    def value(self, point):
        d = b.sub(self.c, self.a)
        t = max(0., min(1., b.dot(b.sub(point, self.a), d)/b.dot(d, d)))
        return b.norm(b.sub(point, b.lerp(self.a, self.c, t)))-(self.ra*(1-t)+self.rc*t)


@dataclass(frozen=True)
class Ellipsoid:
    center: b.Vec
    radii: b.Vec

    def value(self, point):
        local = b.sub(point, self.center)
        return (math.sqrt(sum((x/r)**2 for x, r in zip(local, self.radii)))-1)*min(self.radii)


@dataclass(frozen=True)
class Junction:
    volumes: tuple[Tube | Ellipsoid, ...]
    blend: float = .04

    def validate(self):
        b.require(1 <= len(self.volumes) <= 16 and math.isfinite(self.blend)
                  and self.blend > 0, "invalid junction")
        for v in self.volumes:
            values = (*v.a, *v.c, v.ra, v.rc) if isinstance(v, Tube) else (*v.center, *v.radii)
            b.require(all(math.isfinite(x) for x in values), "nonfinite junction")
            if isinstance(v, Tube):
                b.require(min(v.ra, v.rc) > 0 and b.norm(b.sub(v.a, v.c)) > 1e-9, "invalid tube")
            else:
                b.require(min(v.radii) > 0, "invalid ellipsoid")

    def value(self, point):
        value = self.volumes[0].value(point)
        for primitive in self.volumes[1:]:
            other = primitive.value(point)
            h = max(0., min(1., .5+.5*(other-value)/self.blend))
            value = other*(1-h)+value*h-self.blend*h*(1-h)
        return value

    def gradient(self, point, epsilon=1e-5):
        result = []
        for i in range(3):
            d = tuple(epsilon if j == i else 0. for j in range(3))
            result.append((self.value(b.add(point, d))-self.value(b.sub(point, d)))/(2*epsilon))
        return tuple(result)


@dataclass(frozen=True)
class SurfaceMesh:
    vertices: tuple[b.Vec, ...]
    triangles: tuple[tuple[int, int, int], ...]
    iso_level_m: float = 0.

    def inspect(self):
        edges = Counter()
        orientation = Counter()
        volume = area = 0.
        for a, c, d in self.triangles:
            b.require(a != c and a != d and c != d, "degenerate triangle indices")
            p, q, r = self.vertices[a], self.vertices[c], self.vertices[d]
            normal = b.cross(b.sub(q, p), b.sub(r, p))
            b.require(b.norm(normal) > 1e-14, "degenerate triangle area")
            area += b.norm(normal)/2
            volume += b.dot(p, b.cross(q, r))/6
            for edge in ((a, c), (c, d), (d, a)):
                edges[tuple(sorted(edge))] += 1
                orientation[tuple(sorted(edge))] += 1 if edge[0] < edge[1] else -1
        return dict(vertices=len(self.vertices), triangles=len(self.triangles),
                    boundary_or_nonmanifold_edges=sum(n != 2 for n in edges.values()),
                    orientation_conflicts=sum(value != 0 for value in orientation.values()),
                    area_m2=area, signed_volume_m3=volume, iso_level_m=self.iso_level_m)


def extract(junction: Junction, minimum: b.Vec, maximum: b.Vec, resolution=24,
            max_cells=100000, max_triangles=100000, iso_offset_fraction=.001):
    junction.validate()
    b.require(type(resolution) is int and resolution >= 4 and resolution**3 <= max_cells,
              "junction grid budget exceeded")
    b.require(all(math.isfinite(a) and math.isfinite(c) and c > a
                  for a, c in zip(minimum, maximum)), "invalid extraction box")
    n = resolution
    b.require(1e-6 <= iso_offset_fraction <= .01, "invalid isolevel regularization")
    level = iso_offset_fraction*min(c-a for a, c in zip(minimum, maximum))/n
    nodes, values = [], []
    for z in range(n+1):
        for y in range(n+1):
            for x in range(n+1):
                point = tuple(a+(c-a)*v/n for a, c, v in zip(minimum, maximum, (x, y, z)))
                value = junction.value(point)-level
                b.require(abs(value) > 1e-14, "isolevel touches grid node; change extraction bounds")
                if x in (0, n) or y in (0, n) or z in (0, n):
                    b.require(value > 0, "extraction box clips junction surface")
                nodes.append(point)
                values.append(value)

    def index(x, y, z):
        return x+(n+1)*(y+(n+1)*z)

    vertices, triangles, edge_vertices = [], [], {}

    def intersection(a, c):
        key = tuple(sorted((a, c)))
        if key not in edge_vertices:
            t = values[a]/(values[a]-values[c])
            edge_vertices[key] = len(vertices)
            vertices.append(b.lerp(nodes[a], nodes[c], t))
        return edge_vertices[key]

    def emit(a, c, d):
        b.require(len(triangles) < max_triangles, "junction triangle budget exceeded")
        p, q, r = vertices[a], vertices[c], vertices[d]
        center = b.mul(b.add(b.add(p, q), r), 1/3)
        if b.dot(b.cross(b.sub(q, p), b.sub(r, p)), junction.gradient(center)) < 0:
            c, d = d, c
        triangles.append((a, c, d))

    orders = tuple(permutations(range(3)))
    for z in range(n):
        for y in range(n):
            for x in range(n):
                origin = (x, y, z)
                for order in orders:
                    cursor = list(origin)
                    ids = [index(*cursor)]
                    for dimension in order:
                        cursor[dimension] += 1
                        ids.append(index(*cursor))
                    inside = [i for i in ids if values[i] < 0]
                    outside = [i for i in ids if values[i] >= 0]
                    if len(inside) in (1, 3):
                        single, rest = (inside[0], outside) if len(inside) == 1 else (outside[0], inside)
                        emit(*(intersection(single, i) for i in rest))
                    elif len(inside) == 2:
                        a, c = inside
                        d, e = outside
                        ad, ae, cd, ce = (intersection(a, d), intersection(a, e),
                                          intersection(c, d), intersection(c, e))
                        emit(ad, ae, ce)
                        emit(ad, ce, cd)
    result = SurfaceMesh(tuple(vertices), tuple(triangles), level)
    info = result.inspect()
    b.require(bool(triangles) and info["boundary_or_nonmanifold_edges"] == 0
              and info["orientation_conflicts"] == 0 and info["signed_volume_m3"] > 0,
              "junction extraction is empty or not closed")
    return result


def branch_junction():
    return Junction((Tube((0., -.3, 0.), (0., .45, 0.), .10, .075),
                     Tube((0., .05, 0.), (.35, .30, .04), .07, .035)), .065)
