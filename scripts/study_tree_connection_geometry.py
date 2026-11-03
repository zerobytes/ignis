"""Geometric proof on two live inputs; output is a baked study, not a persisted link."""
import argparse
import copy
import json
import os
import sys
from pathlib import Path
P = Path(__file__).resolve().parents[1]
R = P / 'artifacts/live-plant-junction'
sys.path.insert(0, str(P / 'scripts'))
os.environ['IGNIS_VERIFY_ROOT'] = str(R)
from verify_lab import Client
from tree_study_geometry import add, rotate
parent = 'branch-editable-1'
child = 'secondary-06'

def subset(a, remove):
    """Retain exact source indices/channels while compacting an authored face cut."""
    a = copy.deepcopy(a)
    g = a['geometry']
    tris = [t for i, t in enumerate(g['triangles']) if i not in remove]
    ids = sorted({v for t in tris for v in t})
    mapping = {v: i for i, v in enumerate(ids)}
    a['geometry'] = {'positions': [g['positions'][i] for i in ids], 'triangles': [[mapping[v] for v in t] for t in tris], 'material_coordinates': [g['material_coordinates'][i] for i in ids]}
    if a.get('rig'):
        a['rig']['weights'] = [a['rig']['weights'][i] for i in ids]
    return (a, ids, mapping)

def boundaries(g):
    """Order real directed boundary edges by topology, without spatial fitting."""
    edges = {}
    for t in g['triangles']:
        for a, b in zip(t, t[1:] + t[:1]):
            edges.setdefault(tuple(sorted((a, b))), []).append((a, b))
    nxt = {pairs[0][0]: pairs[0][1] for pairs in edges.values() if len(pairs) == 1}
    out = []
    while nxt:
        start = next(iter(nxt))
        ring = []
        v = start
        while True:
            ring.append(v)
            v = nxt.pop(v)
            if v == start:
                break
        out.append(ring)
    return out

def prepare(call):
    """Locate section cells through native material identities and assembly prefixes."""
    initial = call('inspect')['scene']
    entries = {e['definition']['id']: e for e in initial['plants']['entries']}
    assets = {a['id']: a for a in initial['meshes']['assets']}
    samples = {id: call('inspect_plant_sampling', id=id)['sampling'] for id in (parent, child)}
    p = entries[parent]
    sample = samples[parent]
    m = sample['material']
    cut = p['assembly']['station']
    stations = p['definition']['sections']['stations']
    cutidx = next((i for i, s in enumerate(stations) if s['id'] == cut))
    lo = cutidx / (len(stations) - 1) - 0.5
    hi = (cutidx + 1) / (len(stations) - 1) - 0.5
    basefaces = [t for t in sample['geometry']['triangles'] if all((m[v][1] >= lo - 1e-06 for v in t))]
    retained = sorted({v for t in basefaces for v in t})
    n = call('inspect_plant', id=parent)['report']['assembly']['retained_context_vertices']
    mapping = {v: i + n for i, v in enumerate(retained)}
    face_lookup = {tuple(t): i for i, t in enumerate(assets[parent]['geometry']['triangles'])}
    removed = set()
    for t in basefaces:
        ys = [m[v][1] for v in t]
        if min(ys) >= lo - 1e-06 and max(ys) <= hi + 1e-06 and (max(ys) - min(ys) > 1e-06):
            removed.add(face_lookup[tuple((mapping[v] for v in t))])
    assert removed
    pa, pids, _ = subset(assets[parent], removed)
    ca, cids, _ = subset(assets[child], {i for i, t in enumerate(samples[child]['geometry']['triangles']) if any((samples[child]['material'][v][1] < -0.25 - 1e-06 for v in t))})
    pr = boundaries(pa['geometry'])
    cr = boundaries(ca['geometry'])
    assert len(pr) == 2 and len(cr) == 1, (len(pr), len(cr))
    inst = next((i for i in initial['meshes']['instances'] if i['id'] == 'tree-secondary-06'))
    root = assets[parent]['rig']['bones'][0]['translation']
    socket = assets[parent]['rig']['bones'][1]['translation']
    anchor = add(root, socket)
    for i, pnt in enumerate(ca['geometry']['positions']):
        ca['geometry']['positions'][i] = add(anchor, add(inst['position'], rotate(inst['rotation'], pnt)))
    out = copy.deepcopy(pa)
    out['id'] = 'junction-study'
    out['rig'] = None
    out['clips'] = []
    g = out['geometry']
    nc = len(g['positions'])
    g['positions'] += ca['geometry']['positions']
    g['material_coordinates'] += ca['geometry']['material_coordinates']
    g['triangles'] += [[v + nc for v in t] for t in ca['geometry']['triangles']]
    center = lambda ids: [sum((g['positions'][v][k] for v in ids)) / len(ids) for k in range(3)]
    pr.sort(key=lambda r: center(r)[1])
    ports = [pr[0], [v + nc for v in cr[0]], pr[1]]
    centers = [center(r) for r in ports]
    return (initial, out, ports, centers, (pa, pids, ca, cids, nc))

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', type=Path, required=True)
    args = parser.parse_args()
    (R / 'examples').mkdir(parents=True, exist_ok=True)
    (R / 'artifacts').mkdir(exist_ok=True)
    (R / 'examples/input.scene.json').write_bytes(args.input.read_bytes())
    c = Client(P / 'target/release/ignis')
    call = c.call
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'live-junction-study', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/input.scene.json')
        initial, out, ports, centers, parts = prepare(call)
        print('centers', centers, 'retained', len(out['geometry']['positions']), flush=True)
        options = {'hub': [(a + b) / 2 for a, b in zip(centers[0], centers[2])], 'hub_radius': 0.09, 'blend': 0.025, 'cell_size': 0.028, 'guard_length': 0.004, 'collar_length': 0.018, 'relaxation_steps': 3, 'material_offset': [0, 0, 0], 'seam_relaxation_steps': 4, 'seam_max_displacement': 0.004, 'material_relaxation_steps': 32}
        options['seam_relaxation_steps'] = 0
        before = call('inspect')
        result = call('preview_mesh_junction_union', source=out, ports=ports, root=None, options=options)
        assert call('inspect') == before
        (R / 'artifacts/fit.json').write_text(json.dumps(result))
        (R / 'artifacts/parts.json').write_text(json.dumps({'ports': ports, 'source': out, 'centers': centers, 'options': options, 'pids': parts[1], 'cids': parts[3], 'nc': parts[4]}))
        print(result['report'], flush=True)
    finally:
        c.close()
if __name__ == '__main__':
    main()
