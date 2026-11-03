"""Cold authoring of connected mountain chains using native spherical operations.

Polyline stations describe the authored alignment; widths/heights and shoulders
vary by seed. The output is a small editable Planet1 document. Rust owns all
surface sampling, meshing, normals, physics and residency; no Python per frame.
"""
import bisect
import copy
import math
import random
from build_earth_procedural import direction


def stations(points, count):
    if len(points) < 2 or count < 2:
        raise ValueError('A mountain chain requires at least two points/stations')
    prefix = [0.]
    for a, b in zip(points, points[1:]):
        length = math.dist(a, b)
        if not math.isfinite(length) or length <= 0:
            raise ValueError('Chain segments must have positive finite length')
        prefix.append(prefix[-1] + length)
    result = []
    for i in range(count):
        distance = prefix[-1] * i / (count - 1)
        segment = min(bisect.bisect_right(prefix, distance) - 1, len(points) - 2)
        t = (distance - prefix[segment]) / (prefix[segment + 1] - prefix[segment])
        a, b = points[segment:segment + 2]
        result.append([a[c] + (b[c] - a[c]) * t for c in range(2)])
    return result


def document_for(recipe, seed):
    if type(seed) is not int or not 0 <= seed <= 0xffffffff:
        raise ValueError('Seed must be a u32 integer')
    # Reject an expanded recipe before allocating station lists or source ops.
    work = len(recipe['noise_layers']) + len(recipe['valley_floor'])
    for chain in recipe['chains']:
        if type(chain['stations']) is not int or not 2 <= chain['stations'] <= 32:
            raise ValueError('Chain station count must be an integer in2..32')
        work += chain['stations'] + len(chain['crest_peaks']['stations'])
    if work > 32:
        raise ValueError('Mountain recipe exceeds the native operation budget')
    rng = random.Random(seed)
    radius = recipe['diameter_meters'] / 2
    document = {'version': 1, 'space_version': 1, 'id': f'earth_mountain_chains_{seed}',
                'revision': 1, 'center': copy.deepcopy(recipe['center']),
                'diameter_meters': recipe['diameter_meters'],
                'surface_gravity_mps2': recipe['gravity_mps2'],
                'base_height_meters': 0, 'operations': []}
    for i, layer in enumerate(recipe['noise_layers']):
        document['operations'].append(dict(kind='noise', seed=(seed + 0x9e3779b9 * (i + 1)) & 0xffffffff, **layer))

    def cap(point, width, height):
        x, z = point
        if width <= 0 or math.hypot(x, z) >= radius or width < math.radians(.001) * radius:
            raise ValueError('Chain cap is outside the native spherical support')
        document['operations'].append({'kind': 'raise', 'direction': direction(radius, x, z),
                                       'angular_radius_degrees': math.degrees(width / radius),
                                       'height_meters': height})

    for chain in recipe['chains']:
        points = stations(chain['points_meters'], chain['stations'])
        # Broad overlapping shoulders join consecutive stations; smaller cores
        # add an irregular summit rhythm without changing the alignment.
        for point in points:
            cap(point, rng.uniform(*chain['radius_meters']), rng.uniform(*chain['height_meters']))
        peaks = chain['crest_peaks']
        for index in peaks['stations']:
            cap(points[index], rng.uniform(*peaks['radius_meters']), rng.uniform(*peaks['height_meters']))
    for basin in recipe['valley_floor']:
        cap(basin['center_meters'], basin['radius_meters'], -basin['depth_meters'])
    if len(document['operations']) > 32:
        raise ValueError('Mountain recipe exceeds the native operation budget')
    return document
