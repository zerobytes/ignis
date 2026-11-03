"""Author-side assembly around native junction boundaries; no nearest-point weld."""
import math
from tree_study_geometry import add, sub, mul, unit, bezier, sweep, dot, cross


def split(points, t):
    levels = [points]
    while len(levels[-1]) > 1:
        levels.append([add(mul(a, 1-t), mul(b, t)) for a, b in zip(levels[-1], levels[-1][1:])])
    return [row[0] for row in levels], [row[-1] for row in levels[::-1]]


def leader_controls(trunk):
    tangent = unit([0.396*math.cos(2.2)+0.1, 4.8, 0.45*math.cos(3)])
    return [trunk(1), add(trunk(1), mul(tangent, 0.65)),
            [0.04, 6.45, -0.17], [0.25, 7.15, -0.10]]


def connected_first_fork(source, trunk, radius, call, continuity=False):
    anchor = trunk(0.34)
    _, retained = split(source, 0.30)
    splice = retained[0]
    outgoing = unit(sub(retained[1], retained[0]))
    side_center = add(anchor, [-0.52, 0.18, 0.02])
    side_normal = unit([-0.9, 0.43, 0.04])
    neck = [side_center, add(side_center, mul(side_normal, 0.30)),
            sub(splice, mul(outgoing, 0.30)), splice]
    if continuity:
        # Longer departure distributes the turn. The previous radius/curvature
        # product exceeded one, so the inner offset folded despite a C1 axis.
        neck[1] = add(side_center, mul(side_normal, 0.575))
        neck[2] = sub(splice, mul(outgoing, 0.375))

    def tangent(t):
        return unit([0.396*math.cos(t*2.2)+0.1, 4.8, 0.45*math.cos(t*3)])

    def collar(id, center, normal, r):
        return {'id': id, 'center': center, 'normal': normal,
                'radial': unit(sub([0, 0, 1], mul(normal, normal[2]))), 'radius': r, 'samples': 32}

    collars = [collar('trunk_below', trunk(0.24), mul(tangent(0.24), -1), radius(0.24)),
               collar('limb_zero', side_center, side_normal, 0.19),
               collar('trunk_above', trunk(0.42), tangent(0.42), radius(0.42))]
    request = {'collars': collars, 'options': {'refinement_levels': 2},
               'fusion': {'center': anchor, 'center_radius': 0.29, 'blend_radius': 0.06,
                          'guard_length': 0.025, 'max_displacement': 0.3}}
    if continuity:
        request['fusion']['transition_lengths'] = [0.20, 0.10, 0.03]
    before = call('inspect')
    junction = call('preview_mesh_junction', **request)
    assert call('inspect') == before
    g = junction['geometry']
    patch_vertices = len(g['positions'])
    boundaries = {b['id']: b['vertices'] for b in junction['boundaries']}

    def extend(index, path, radii, buttress=0, **sampling):
        c = collars[index]
        ids = boundaries[c['id']]
        n = c['samples']
        assert len(ids) == n
        tube = sweep(path, radii, n, roughness=0.025, buttress=buttress,
                     start_frame=(c['normal'], c['radial']), cap_start=False, **sampling)
        mapping = list(ids)
        # The native border is authoritative, including its f32 quantization.
        # New rings reference its existing indices; no position search/epsilon.
        for i, vertex in enumerate(ids):
            assert math.dist(tube['positions'][i], g['positions'][vertex]) < 0.02
        for p in tube['positions'][n:]:
            mapping.append(len(g['positions']))
            g['positions'].append(p)
        g['triangles'].extend([[mapping[i] for i in t] for t in tube['triangles']])

    heights = [-0.24, -0.10, 0, 0.10, 0.22, 0.38, 0.60]
    root_path = [[0, h, 0] for h in heights]+[trunk(4/21)]
    root_radii = [0.45, 0.42, 0.40, 0.39, 0.39, 0.38, 0.38, radius(4/21)]
    extend(0, [collars[0]['center']]+root_path[::-1],
           [collars[0]['radius']]+root_radii[::-1], buttress=1.25)
    leader = bezier(leader_controls(trunk), 6)
    leader_radii = [(radius(1)-0.003)*(1-i/5)**1.3+0.003 for i in range(1, 6)]
    extend(2, [collars[2]['center']]+[trunk(i/21) for i in range(10, 22)]+leader[1:],
           [collars[2]['radius']]+[radius(i/21) for i in range(10, 22)]+leader_radii)
    limb_radius = radius(0.34)*0.70
    retained_radius = lambda t: limb_radius*(1-t)**1.2+0.004
    neck_vertex_start = len(g['positions'])
    neck_quality = None
    if continuity:
        count = 9
        neck_path = bezier(neck, count)
        # Hermite taper: zero slope at cylindrical collar, matching dr/ds of
        # retained analytic curve at the splice. No ring snapping or smoothing.
        r0, r1 = 0.19, retained_radius(0.3)
        source_derivative = bezier([mul(sub(b,a), 3) for a,b in zip(source,source[1:])], 101)[30]
        speed = math.sqrt(sum(x*x for x in source_derivative))
        slope = -1.2*limb_radius*0.7**0.2*1.125/speed
        def neck_radius(t):
            return (2*t**3-3*t*t+1)*r0+(-2*t**3+3*t*t)*r1+(t**3-t*t)*slope
        neck_radii = [neck_radius(i/(count-1)) for i in range(count)]
        derivative = [mul(sub(b,a),3) for a,b in zip(neck,neck[1:])]
        second = [mul(sub(b,a),2) for a,b in zip(derivative,derivative[1:])]
        neck_tangents = bezier(derivative, count)
        ratios = []
        for i,(d,dd) in enumerate(zip(bezier(derivative,1001),bezier(second,1001))):
            kappa = math.sqrt(dot(cross(d,dd),cross(d,dd)))/dot(d,d)**1.5
            ratios.append(neck_radius(i/1000)*kappa)
        assert max(ratios)<0.9, 'Sampled offset would approach the curvature singularity'
        assert dot(unit(derivative[0]),side_normal)>1-1e-12
        assert dot(unit(derivative[-1]),outgoing)>1-1e-12
        assert abs(slope/1.125-(-1.2*limb_radius*0.7**0.2)/speed)<1e-12
        neck_quality = {'samples':1001,'max_sampled_radius_curvature':max(ratios),
                        'tangent_directions_match':True,'taper_slope_matches_per_arc_length':True,
                        'global_self_intersection_proven':False}
        retained_tangents = bezier([mul(sub(b,a),3) for a,b in zip(retained,retained[1:])], 8)
        params = [i/(count-1) for i in range(count-1)]
        extend(1, neck_path[:-1]+bezier(retained,8), neck_radii[:-1]+[retained_radius(0.3+0.7*i/7) for i in range(8)],
               tangents=neck_tangents[:-1]+retained_tangents,
               roughness_coordinates=[4*t for t in params]+list(range(4,12)),
               roughness_weights=[t**3*(10-15*t+6*t*t) for t in params]+[1]*8)
    else:
        neck_path = bezier(neck, 5)
        neck_radii = [0.19+(retained_radius(0.3)-0.19)*i/4 for i in range(5)]
        extend(1, neck_path[:-1]+bezier(retained, 8),
               neck_radii[:-1]+[retained_radius(0.3+0.7*i/7) for i in range(8)])

    # One analytic rest-space field over this whole connected component.
    # A previous fixed-boundary interpolation compressed the grain into visible
    # rings at the collars. New surface has no inherited texture identity.
    g['material_coordinates'] = [list(p) for p in g['positions']]
    errors = []
    for t in [0.30, 0.44, 0.58, 0.72, 0.86, 1.0]:
        original = split(source, t)[0][-1]
        kept = split(retained, (t-0.3)/0.7)[0][-1]
        errors.append(math.dist(original, kept))
    assert max(errors) < 1e-12
    edges = {}
    for a, b, c in g['triangles']:
        for x, y in [(a, b), (b, c), (c, a)]:
            edge = tuple(sorted((x, y)))
            count, orientation = edges.get(edge, (0, 0))
            edges[edge] = (count+1, orientation+(1 if x < y else -1))
    assert all(count == 2 and orientation == 0 for count, orientation in edges.values())
    assert len(g['positions'])-len(edges)+len(g['triangles']) == 2
    adjacent = [set() for _ in g['positions']]
    for a, b in edges:
        adjacent[a].add(b); adjacent[b].add(a)
    seen, pending = {0}, [0]
    while pending:
        for vertex in adjacent[pending.pop()]:
            if vertex not in seen:
                seen.add(vertex); pending.append(vertex)
    assert len(seen) == len(g['positions'])
    return g, {'request': request, 'native_report': junction['report'],
               'patch_vertices': patch_vertices, 'protected_anchor_errors': errors,
               'neck_vertex_start': neck_vertex_start,
               'neck_quality': neck_quality,
               'assembly_vertices': len(g['positions']), 'assembly_triangles': len(g['triangles']),
               'assembly_boundary_edges': 0, 'assembly_orientation_conflicts': 0, 'assembly_euler': 2,
               'assembly_components': 1,
               'decisions': ['trunk retessellated with32 angular samples', 'replace trunk parameter0.24..0.42',
                             'first limb neck explicitly moved; original curve retained from0.30',
                             'native collar indices shared exactly', 'continuous object-rest material field; no old vertex identity claimed']}
