#!/usr/bin/env python3
"""Broader facial handle fields and bounded overlap on the connected cage.

No mesh smoothing is applied to the character itself. Only displacement fields
are replaced; body proportions, detail, topology and spherical eyes are retained.
The source-owned blend can recover every old field at zero, including its bake.
"""
import hashlib
import heapq
import json
import math
from collections import defaultdict

from extend_source import OUT, ROOT, neutral

CHANNEL = 'face_softness'
RADII = {
    'forehead_upper': .090, 'forehead_middle': .065, 'forehead_side': .060,
    'glabella': .040, 'temple': .060, 'brow_inner': .036,
    'brow_arch': .036, 'brow_outer': .040,
    'nose_bridge': .035, 'nose_tip': .028, 'nose_wing': .022,
    'nose_alar_base': .014, 'nose_upper_side': .016,
    'nose_middle_side': .016, 'nose_tip_side': .016, 'nostril_roof': .012,
    'upper_lid': .026, 'lower_lid': .026, 'eye_inner': .022, 'eye_outer': .026,
    'cheekbone': .060, 'cheek': .065, 'malar_upper': .026,
    'orbital_lateral': .026, 'chin': .060, 'jaw_angle': .045,
    'chin_base': .035, 'chin_side': .030, 'chin_corner': .035,
    'jaw_front': .040, 'jaw_body': .040, 'jaw_back': .040, 'jaw_ramus': .035,
    'submental': .040, 'submental_side': .032,
    'upper_lip': .030, 'lower_lip': .030, 'mouth_corner': .025,
    'cupid_peak': .014, 'cupid_valley': .012, 'upper_tubercle': .014,
    'upper_lip_lobe': .018, 'upper_lip_rim': .018, 'upper_lip_seam': .016,
    'lower_lip_lobe': .018, 'lower_lip_rim': .018, 'lower_lip_seam': .016,
    'lower_lip_border': .018,
    'ear_upper': .024, 'ear_lobe': .020, 'ear_tip': .020,
    'ear_rim': .022, 'ear_concha': .020,
}


def topology(source):
    body = next(p for p in source['parts'] if p['name'] == 'body')
    ids = body['vertices']
    faces = source['refinements'][0]['faces']
    neighbors = defaultdict(set)
    edges = defaultdict(list)
    incident = defaultdict(list)
    for fi, face in enumerate(faces):
        for a, b in zip(face, face[1:] + face[:1]):
            edges[tuple(sorted((a, b)))].append(fi)
            neighbors[a].add(b)
            neighbors[b].add(a)
            incident[a].append(fi)
    rows = []
    for i in range(len(ids)):
        row = defaultdict(float)
        boundary = [j for j in neighbors[i] if len(edges[tuple(sorted((i, j)))]) == 1]
        if boundary:
            assert len(boundary) == 2
            row[i] = .75
            for j in boundary:
                row[j] += .125
        else:
            n = len(neighbors[i])
            row[i] = (n - 3) / n
            for fi in incident[i]:
                for j in faces[fi]:
                    row[j] += 1 / (n * n * len(faces[fi]))
            for j in neighbors[i]:
                row[i] += 1 / (n * n)
                row[j] += 1 / (n * n)
        rows.append(dict(row))
    return ids, neighbors, rows


def distances(graph, seeds, radius):
    result = {i: 0. for i in seeds}
    queue = [(0., i) for i in seeds]
    heapq.heapify(queue)
    while queue:
        distance, i = heapq.heappop(queue)
        if distance != result[i]:
            continue
        for j, length in graph[i]:
            candidate = distance + length
            if candidate < radius and candidate < result.get(j, math.inf):
                result[j] = candidate
                heapq.heappush(queue, (candidate, j))
    return result


def field(graph, neighbors, stencils, anchors, radius):
    # A small plateau makes the dragged point and its immediate neighborhood
    # travel together. The quintic falloff has zero first/second derivatives
    # at both ends. Graph distance cannot jump across a lip/eye opening.
    distance = distances(graph, anchors, radius)
    weights = {}
    for i, d in distance.items():
        t = max(0., (d / radius - .20) / .80)
        weights[i] = 1 - t**3 * (10 + t * (-15 + 6*t))
    # Relax graph-distance kinks; outside support is a fixed zero boundary.
    for _ in range(3):
        weights = {i: .5*w + .5*sum(weights.get(j, 0.) for j in neighbors[i])/len(neighbors[i])
                   for i, w in weights.items()}
    response = sum(sum(w * weights.get(i, 0.) for i, w in stencils[a].items())
                   for a in anchors) / len(anchors)
    assert .25 < response <= 1.000001
    return {i: w/response for i, w in weights.items() if w > 1e-7}, response


def build():
    parent = OUT/'human-spherical.ignis-character'
    raw = parent.read_bytes()
    source = json.loads(raw)
    assert source['version'] == 9 and len(source['parameters']) == 392
    source['id'] = 'human_adult_soft_v1'
    source['version'] = 10
    source['field_blends'] = [dict(strength=CHANNEL, fields=[])]
    source['provenance'] += ' Ignis source10 soft fields1: connected-surface proportional facial handles with bounded overlap; source9 spherical eyes retained; face_softness0 reproduces parent fields.'
    points = neutral(source)
    ids, neighbors, stencils = topology(source)
    local = {v: i for i, v in enumerate(ids)}
    graph = {i: [(j, math.dist(points[ids[i]], points[ids[j]])) for j in sorted(ns)]
             for i, ns in neighbors.items()}
    targets = {t['name']: t for t in source['targets']}
    report = []
    additions = []
    for handle in source['handles']:
        if handle['part'] != 'body':
            continue  # Rigid eye controls already transport their whole insert.
        name = handle['name']
        stem = name[:-2] if name.endswith(('_L', '_R')) else name
        radius = RADII[stem]
        anchors = [local[v] for v in handle['anchor']['vertices']]
        before = targets['handle_'+name+'_0']
        old = {local[d['vertex']]: d['offset'][0]/handle['limits'][0]
               for d in before['deltas'] if d['vertex'] in local}
        adjacent = set().union(*(neighbors[i] for i in anchors))
        def variation(values):
            return max(abs(values.get(i, 0.)-values.get(j, 0.))
                       for i in values for j in neighbors[i])
        old_jump = variation(old)
        # Sparse cage spacing differs between lips, nose, ears and jaw. Expand
        # the authored radius until the first ring follows and the worst edge
        # transition improves, rather than assuming a fixed millimetre size is
        # equally smooth everywhere. This is bounded offline authoring work.
        for _ in range(9):
            weights, response = field(graph, neighbors, stencils, anchors, radius)
            if (variation(weights) <= old_jump*.9
                    and min(weights.get(i, 0.) for i in adjacent) >= .85):
                break
            radius *= 1.12
        else:
            raise ValueError('Cannot smooth field within calibration budget: '+name)
        assert radius <= .15
        report.append(dict(name=name, radius=radius, response=response,
                           before=len(old), after=len(weights),
                           before_peak=max(old.values()), after_peak=max(weights.values()),
                           before_edge_jump=old_jump, after_edge_jump=variation(weights),
                           before_neighbor_min=min(old.get(i, 0.) for i in adjacent),
                           after_neighbor_min=min(weights.get(i, 0.) for i in adjacent)))
        for axis, limit in enumerate(handle['limits']):
            target = targets['handle_'+name+'_'+str(axis)]
            # Keep any nondrawn helper deltas unchanged for fitted joints.
            helpers = [d for d in target['deltas'] if d['vertex'] not in local]
            deltas = [dict(vertex=ids[i], offset=[round(limit*w, 9) if k == axis else 0. for k in range(3)])
                      for i, w in weights.items()] + helpers
            additions.append(dict(name='soft_'+name+'_'+str(axis), gain=target['gain'],
                                  factors=target['factors'] + [dict(parameter=CHANNEL, curve=[[0, 0], [1, 1]])],
                                  deltas=sorted(deltas, key=lambda d: d['vertex'])))
            target['factors'].append(dict(parameter=CHANNEL, curve=[[0, 1], [1, 0]]))
        source['field_blends'][0]['fields'].append(dict(parameters=handle['parameters'], targets=['soft_'+name+'_'+str(a) for a in range(3)]))
    source['targets'].extend(additions)
    source['parameters'].append(dict(name=CHANNEL, minimum=0, maximum=1, default=1))
    source['parameters'].sort(key=lambda p: p['name'])
    encoded = json.dumps(source, separators=(',', ':')).encode()
    assert len(encoded) <= 96*1024*1024 and len(source['targets']) <= 1024
    path = OUT/'human-soft.ignis-character'
    path.write_bytes(encoded)
    out = ROOT/'artifacts/character-soft'
    out.mkdir(exist_ok=True)
    result = dict(path=str(path), version=source['version'], bytes=len(encoded),
                  sha256=hashlib.sha256(encoded).hexdigest(), parent_sha256=hashlib.sha256(raw).hexdigest(),
                  parameters=len(source['parameters']), handles=len(source['handles']),
                  targets=len(source['targets']), fields=report)
    (out/'source.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({k: v for k, v in result.items() if k != 'fields'}, indent=2))


if __name__ == '__main__':
    build()
