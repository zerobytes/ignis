"""Cold composition of the same native Surface program for plane and sphere.

Only locations and authored paths are prepared here. Rust evaluates every height,
normal, mesh, collision and frame. No terrain sampling or renderer in Python.
"""
import math
import random


def translate(origin, delta):
    cells, offsets = [], []
    for cell, offset, change in zip(origin['cell'], origin['offset'], delta):
        value = offset + change
        carry = math.floor(value / 1024)
        cells.append(str(int(cell) + carry))
        offsets.append(value - carry * 1024)
    return {'cell': cells, 'offset': offsets}


def direction(radius, x, z):
    distance = math.hypot(x, z)
    if not distance:
        return [0, 1, 0]
    angle = distance / radius
    return [x / distance * math.sin(angle), math.cos(angle), z / distance * math.sin(angle)]


def path_points(controls, count):
    """Sample the authored Catmull-Rom path into editable bounded native waypoints."""
    if not 2 <= len(controls) <= 64 or not 2 <= count <= 64:
        raise ValueError('Path needs 2..64 controls and output points')
    result = []
    for i in range(count):
        u = i * (len(controls) - 1) / (count - 1)
        j = min(int(u), len(controls) - 2)
        t = u - j
        a, b, c, d = [controls[min(max(k, 0), len(controls) - 1)] for k in (j-1, j, j+1, j+2)]
        result.append([.5 * (2*b[k] + (-a[k]+c[k])*t + (2*a[k]-5*b[k]+4*c[k]-d[k])*t*t
                             + (-a[k]+3*b[k]-3*c[k]+d[k])*t*t*t) for k in (0, 1)])
    return result


def draft_for(recipe, seed=None, domain='sphere'):
    seed = recipe['seed'] if seed is None else seed
    if type(seed) is not int or not 0 <= seed <= 0xffffffff or domain not in ('plane', 'sphere'):
        raise ValueError('A u32 seed and plane/sphere domain are required')
    if not 1 <= len(recipe['ranges']) <= 8:
        raise ValueError('This bounded authoring recipe needs 1..8 mountain ranges')
    rng = random.Random(seed)
    radius = recipe['diameter_meters'] / 2
    origin = recipe['center']
    def location(x, z):
        delta = [x, 0, z] if domain == 'plane' else [v*radius for v in direction(radius, x, z)]
        return translate(origin, delta)
    nodes = []
    def add(kind, **kw):
        nodes.append(dict(kind=kind, **kw))
        return len(nodes)-1
    def noise(salt, period, octaves, amplitude, style='smooth'):
        return add('noise', seed=(seed+salt*0x9e3779b9)&0xffffffff, period_meters=period,
                   octaves=octaves, persistence=.5, amplitude_meters=amplitude, style=style)
    def remap(slot, values):
        return add('remap', input=slot, knots=[dict(x=x, value=y, derivative=0) for x, y in values])
    def binary(kind, a, b, **kw):
        return add(kind, a=a, b=b, **kw)

    continental = remap(noise(1, 524288, 4, 1), [(-1, -1100), (-.2, -200), (.1, 70), (.65, 500), (1, 750)])
    regional = remap(noise(2, 131072, 3, 1), [(-1, 0), (-.15, .1), (.5, .8), (1, 1)])
    mountain_field = remap(noise(3, 32768, 5, 1, 'ridged'), [(-1, 0), (-.3, 0), (.25, 200), (.75, 1100), (1, 1700)])
    base = binary('add', continental, binary('multiply', regional, mountain_field))
    modulation = remap(noise(4, 8192, 4, 1),
                       [(-1, .15), (-.3, .25), (0, .7), (.3, 1.5), (1, 1.8)])
    # Ridge noise supplies intersecting secondary crests, rather than a constant
    # height rail along each authored range. Small octaves stay >=256 metres.
    dissection = remap(noise(6, 4096, 5, 1, 'ridged'),
                       [(-1, .1), (0, .25), (.35, .5), (.65, 1), (1, 1.35)])
    modulation = binary('multiply', modulation, dissection)
    combined = None
    for settings in recipe['ranges']:
        controls = [[x+rng.uniform(-180, 180), z+rng.uniform(-180, 180)] for x, z in settings['points_meters']]
        curve = add('curve', points=[location(x, z) for x, z in path_points(controls, settings['samples'])],
                    width_meters=settings['width_meters'], height_meters=settings['height_meters'],
                    left_exponent=settings['left_exponent'], right_exponent=settings['right_exponent'], roundness=1)
        envelope = add('raise', center=location(*settings['envelope_center']),
                       radius_meters=settings['envelope_radius_meters'], amount_meters=1)
        shaped = binary('multiply', binary('multiply', curve, envelope), modulation)
        combined = shaped if combined is None else binary('smooth_maximum', combined, shaped, transition_meters=120)
    terrain = binary('add', base, combined)
    terrain = binary('add', terrain, noise(5, 2048, 4, 65))
    valley = recipe['valley']
    terrain = add('channel', input=terrain, points=[location(x, z) for x, z in valley['points_meters']],
                  width_meters=valley['width_meters'], bed_start_meters=valley['bed_start_meters'],
                  bed_end_meters=valley['bed_end_meters'], blend=valley['blend'])
    if len(nodes) > 128:
        raise ValueError('Geographic composition exceeds native node budget')
    return {'nodes': nodes, 'output': terrain}
