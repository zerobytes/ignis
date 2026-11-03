"""Camera-independent surface diagnostics, not semantic reconstruction approval."""
import argparse
from collections import defaultdict
import hashlib
import json
import math
from pathlib import Path

MAX_VERTICES = 8192
MAX_TRIANGLES = 16384


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def cross(a, b):
    return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])


def dot(a, b):
    return math.fsum(x*y for x, y in zip(a, b))


def inspect_surface(geometry):
    """Inspect exact-position seams without changing input or filling any gap."""
    positions, triangles = geometry['positions'], geometry['triangles']
    if not isinstance(positions, list) or not 1 <= len(positions) <= MAX_VERTICES:
        raise ValueError('positions must contain 1..8192 vertices')
    if not isinstance(triangles, list) or not 1 <= len(triangles) <= MAX_TRIANGLES:
        raise ValueError('triangles must contain 1..16384 faces')
    lookup, points, remap = {}, [], []
    for p in positions:
        if (not isinstance(p, list) or len(p) != 3 or
                any(type(v) not in (int, float) or not math.isfinite(v) or abs(v) > 1e6 for v in p)):
            raise ValueError('positions must be finite numeric triples within +/-1e6')
        key = tuple(p)
        if key not in lookup:
            lookup[key] = len(points)
            points.append(key)
        remap.append(lookup[key])
    edges, faces, invalid_faces = defaultdict(list), [], []
    used = set()
    for index, t in enumerate(triangles):
        if (not isinstance(t, list) or len(t) != 3 or
                any(type(v) is not int or not 0 <= v < len(positions) for v in t)):
            raise ValueError('triangles must contain valid integer index triples')
        face = tuple(remap[v] for v in t)
        faces.append(face)
        used.update(face)
        a, b, c = (points[v] for v in face)
        area_normal = cross(sub(b, a), sub(c, a))
        if len(set(face)) != 3 or dot(area_normal, area_normal) == 0:
            invalid_faces.append(index)
        for a, b in zip(face, face[1:]+face[:1]):
            edges[tuple(sorted((a, b)))].append((index, 1 if a < b else -1))
    adjacency = [[] for _ in faces]
    boundary = non_manifold = orientation = 0
    for entries in edges.values():
        boundary += len(entries) == 1
        non_manifold += len(entries) > 2
        orientation += len(entries) == 2 and entries[0][1] == entries[1][1]
        first = entries[0][0]
        for index, _ in entries[1:]:
            adjacency[first].append(index)
            adjacency[index].append(first)
    seen, components = set(), []
    for seed in range(len(faces)):
        if seed in seen:
            continue
        stack, members = [seed], []
        seen.add(seed)
        while stack:
            index = stack.pop()
            members.append(index)
            for other in adjacency[index]:
                if other not in seen:
                    seen.add(other)
                    stack.append(other)
        origin = points[faces[seed][0]]
        volume = math.fsum(dot(sub(points[a], origin), cross(sub(points[b], origin), sub(points[c], origin)))/6
                           for a, b, c in (faces[index] for index in members))
        components.append({'faces': len(members), 'signed_volume_diagnostic': volume})
    low = [min(p[k] for p in points) for k in range(3)]
    high = [max(p[k] for p in points) for k in range(3)]
    diagonal = math.sqrt(math.fsum((b-a)**2 for a, b in zip(low, high)))
    tolerance = diagonal**3 * 1e-12
    defects = []
    for count, code in [(boundary, 'open_boundary'), (non_manifold, 'non_manifold_edge'),
                        (orientation, 'inconsistent_orientation'), (len(invalid_faces), 'degenerate_face'),
                        (len(points)-len(used), 'isolated_vertex')]:
        if count:
            defects.append(code)
    if len(components) != 1:
        defects.append('multiple_components')
    if any(c['signed_volume_diagnostic'] <= tolerance for c in components):
        defects.append('zero_or_inward_volume')
    return {'vertices': len(positions), 'coincident_positions': len(positions)-len(points),
            'faces': len(faces), 'boundary_edges': boundary, 'non_manifold_edges': non_manifold,
            'inconsistent_edges': orientation, 'degenerate_faces': len(invalid_faces),
            'isolated_positions': len(points)-len(used), 'components': components,
            'bounds': {'min': low, 'max': high}, 'volume_tolerance': tolerance,
            'solid_preflight_passed': not defects, 'defects': defects}


def building_core_preflight(meshes):
    """E02 publisher convention; optional ornaments are not required to be solids."""
    core = meshes.get('building-core')
    if core is None:
        return {'solid_preflight_passed': False, 'defects': ['missing_building_core'],
                'note': 'Author the actual building body before projecting facade details.'}
    result = inspect_surface({'positions': core.p, 'triangles': core.t})
    result['note'] = 'Topology only; correspondence, intersections, supports and thickness remain unverified.'
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('--require-solid', metavar='ASSET')
    args = parser.parse_args()
    try:
        if args.input.stat().st_size > 32*1024*1024:
            raise ValueError('input exceeds 32MiB')
        raw = args.input.read_bytes()
        data = json.loads(raw)
        assets = data['meshes']['assets'] if 'meshes' in data else [{'id': 'geometry', 'geometry': data}]
        if not isinstance(assets, list) or not 1 <= len(assets) <= 16:
            raise ValueError('expected 1..16 assets')
        reports = {}
        for asset in assets:
            key = asset['id']
            if not isinstance(key, str) or key in reports:
                raise ValueError('asset IDs must be unique strings')
            reports[key] = inspect_surface(asset['geometry'])
        result = {'algorithm': 'surface-structure-v1', 'input_sha256': hashlib.sha256(raw).hexdigest(),
                  'coordinate_policy': 'asset rest positions; exact coincidence; no repair',
                  'semantic_acceptance': 'not_evaluated', 'assets': reports}
        passed = True
        if args.require_solid:
            passed = reports.get(args.require_solid, {}).get('solid_preflight_passed', False)
            result['required_solid'] = {'asset': args.require_solid, 'passed': passed}
        print(json.dumps(result, indent=2, allow_nan=False))
        return 0 if passed else 1
    except (OSError, ValueError, KeyError, TypeError, OverflowError) as error:
        print(json.dumps({'error': str(error)}))
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
