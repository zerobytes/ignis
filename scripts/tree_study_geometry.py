"""Explicit geometry helpers for the native MCP tree authoring experiment.

These are author-side calculations, not new engine modeling commands.
"""
import math


def add(a, b):
    return [x+y for x, y in zip(a, b)]


def sub(a, b):
    return [x-y for x, y in zip(a, b)]


def mul(a, s):
    return [x*s for x in a]


def dot(a, b):
    return sum(x*y for x, y in zip(a, b))


def cross(a, b):
    return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]


def unit(a):
    return mul(a, 1/math.sqrt(dot(a, a)))


def quat(axis, angle):
    return mul(axis, math.sin(angle/2))+[math.cos(angle/2)]


def qmul(a, b):
    x, y, z, w = a
    X, Y, Z, W = b
    return [w*X+x*W+y*Z-z*Y, w*Y-x*Z+y*W+z*X,
            w*Z+x*Y-y*X+z*W, w*W-x*X-y*Y-z*Z]


def rotate(q, p):
    return qmul(qmul(q, list(p)+[0]), [-q[0], -q[1], -q[2], q[3]])[:3]


def transform(p, position, rotation, scale=1):
    return add(position, rotate(rotation, mul(p, scale)))


def geometry():
    return {'positions': [], 'triangles': []}


def append(dst, src, position=(0, 0, 0), rotation=(0, 0, 0, 1), scale=1):
    n = len(dst['positions'])
    if 'material_coordinates' in src or 'material_coordinates' in dst:
        if 'material_coordinates' not in dst:
            dst['material_coordinates'] = list(dst['positions'])
        dst['material_coordinates'].extend(mul(p, scale) for p in src.get('material_coordinates', src['positions']))
    dst['positions'].extend(transform(p, position, rotation, scale) for p in src['positions'])
    dst['triangles'].extend([i+n for i in t] for t in src['triangles'])


def bezier(points, count):
    out = []
    for i in range(count):
        t = i/(count-1)
        p = points
        while len(p) > 1:
            p = [add(mul(a, 1-t), mul(b, t)) for a, b in zip(p, p[1:])]
        out.append(p[0])
    return out


def sweep(path, radii, sides, phase=0, roughness=0.03, buttress=0,
          start_frame=None, cap_start=True, cap_end=True, tangents=None,
          roughness_coordinates=None, roughness_weights=None):
    """Transport a radial frame along a sampled curve, avoiding helper-axis flips."""
    g = geometry()
    g['material_coordinates'] = []
    previous = None
    distance = 0
    roots = [(0.10, 1.0), (1.31, 0.72), (2.65, 0.90), (3.92, 0.65), (5.20, 0.84)]
    for i, (p, r) in enumerate(zip(path, radii)):
        if i:
            delta = sub(p, path[i-1])
            distance += math.sqrt(dot(delta, delta))
        tangent = unit(tangents[i]) if tangents is not None else unit(sub(path[min(i+1, len(path)-1)], path[max(0, i-1)]))
        if i == 0 and start_frame is not None:
            tangent, radial = start_frame
        elif previous is None:
            axis = min([[1, 0, 0], [0, 1, 0], [0, 0, 1]], key=lambda a: abs(dot(a, tangent)))
            radial = unit(cross(tangent, axis))
        else:
            radial = unit(sub(previous, mul(tangent, dot(previous, tangent))))
        other = cross(tangent, radial)
        previous = radial
        for j in range(sides):
            theta = 2*math.pi*j/sides
            coordinate = i if roughness_coordinates is None else roughness_coordinates[i]
            weight = 1 if roughness_weights is None else roughness_weights[i]
            ridge = 1+roughness*weight*(0.65*math.sin(theta*7+phase+coordinate*0.19)+
                                         0.35*math.sin(theta*13-phase+coordinate*0.11))
            # Broad flutes flow into a buried root collar on the SAME surface.
            # No independent spikes/caps are attached to the soil-level ring.
            if buttress:
                flutes = 0
                for angle, amplitude in roots:
                    delta = math.atan2(math.sin(theta-angle-0.2*max(0, p[1])),
                                       math.cos(theta-angle-0.2*max(0, p[1])))
                    flutes += amplitude*math.exp(-0.5*(delta/0.25)**2)
                ridge += buttress*math.exp(-max(0, p[1])*4.5)*flutes
            g['positions'].append(add(p, add(mul(radial, r*ridge*math.cos(theta)),
                                             mul(other, r*ridge*math.sin(theta)))))
            # Hold the angular material domain across taper/flare; otherwise
            # radius changes turn longitudinal grain into isolated spots.
            g['material_coordinates'].append([radii[0]*math.cos(theta), distance, radii[0]*math.sin(theta)])
    for i in range(len(path)-1):
        for j in range(sides):
            a, b = i*sides+j, i*sides+(j+1) % sides
            g['triangles'].extend([[a, b, a+sides], [b, b+sides, a+sides]])
    caps = ([(0, True)] if cap_start else []) + ([((len(path)-1)*sides, False)] if cap_end else [])
    for start, reverse in caps:
        for j in range(1, sides-1):
            t = [start, start+j, start+j+1]
            g['triangles'].append(t[::-1] if reverse else t)
    return g


def blade():
    """Small curved broadleaf: explicit low-budget working topology, metres."""
    g = geometry()
    g['positions'].append([0, 0, -0.012])
    for z, width in [(0.020, 0.020), (0.110, 0.064), (0.200, 0.040)]:
        for u in [-1, 0, 1]:
            y = 0.010*(1-u*u)*math.sin(max(0, z)*math.pi/0.25)+0.11*z*z
            g['positions'].append([u*width, y, z])
    g['triangles'].extend([[0, 1, 2], [0, 2, 3]])
    for row in range(2):
        for x in range(2):
            a = 1+row*3+x
            g['triangles'].extend([[a, a+3, a+1], [a+1, a+3, a+4]])
    tip = len(g['positions'])
    g['positions'].append([0, 0.015, 0.25])
    g['triangles'].extend([[7, tip, 8], [8, tip, 9]])
    return g


def detailed_blade():
    """Broadleaf with eight edge stations, slight camber and an asymmetric curl."""
    g = geometry()
    g['positions'].append([0, 0, -0.012])
    stations = [(0.015, 0.007), (0.045, 0.035), (0.075, 0.056), (0.110, 0.066),
                (0.145, 0.064), (0.180, 0.051), (0.212, 0.031), (0.234, 0.012)]
    for z, width in stations:
        for u in [-1, 0, 1]:
            y = 0.004*(1-u*u)*math.sin(z*math.pi/0.25)
            y += 0.008*(z/0.25)**2+u*0.003*math.sin(z*math.pi*5)
            g['positions'].append([u*width*(1+u*0.03), y, z])
    g['triangles'].extend([[0, 1, 2], [0, 2, 3]])
    for row in range(len(stations)-1):
        for x in range(2):
            a = 1+row*3+x
            g['triangles'].extend([[a, a+3, a+1], [a+1, a+3, a+4]])
    tip = len(g['positions'])
    g['positions'].append([0.001, 0.011, 0.25])
    a = 1+(len(stations)-1)*3
    g['triangles'].extend([[a, tip, a+1], [a+1, tip, a+2]])
    return g
