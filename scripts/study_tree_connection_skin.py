"""Explicit harmonic skinning of the junction study; does not install a live plant link."""
import copy
import json
import math
import os
import sys
from pathlib import Path
P = Path(__file__).resolve().parents[1]
R = P / 'artifacts/live-plant-junction'
sys.path.insert(0, str(P / 'scripts'))
os.environ['IGNIS_VERIFY_ROOT'] = str(R)
from verify_lab import Client
from tree_study_geometry import add, cross, dot, rotate, sub


def normal(positions, triangle):
    a, b, c = [positions[i] for i in triangle]
    return cross(sub(b, a), sub(c, a))

def skin(asset, initial, parts):
    """Keep old weights, pin exact guards, solve a two-bone harmonic field on iso."""
    parent = next((a for a in initial['meshes']['assets'] if a['id'] == 'branch-editable-1'))
    child = next((a for a in initial['meshes']['assets'] if a['id'] == 'secondary-06'))
    inst = next((i for i in initial['meshes']['instances'] if i['id'] == 'tree-secondary-06'))
    root = next((i for i, b in enumerate(parent['rig']['bones']) if b['name'] == 'base'))
    anchor = next((i for i, b in enumerate(parent['rig']['bones']) if b['name'] == 'secondary-06'))
    assert root != anchor
    assert all((all((j == 0 or w == 0 for j, w in zip(v['joints'], v['weights']))) for v in child['rig']['weights'])) and (not child.get('clips'))
    rig = copy.deepcopy(parent['rig'])
    rig['weights'] = [rig['weights'][i] for i in parts['pids']] + [{'joints': [anchor, 0, 0, 0], 'weights': [1, 0, 0, 0]} for _ in parts['cids']]
    d = add(child['rig']['bones'][0]['translation'], child['rig']['bones'][1]['translation'])
    assert all((b['rotation'] == [0, 0, 0, 1] for b in child['rig']['bones']))
    rig['bones'].append({'name': 'secondary06-tip', 'parent': anchor, 'translation': add(inst['position'], rotate(inst['rotation'], d)), 'rotation': inst['rotation']})
    plan = asset['material_plan']
    out = asset['asset']
    g = out['geometry']
    nv = plan['retained_vertices']
    end = nv + plan['iso_vertices']
    assert nv == len(rig['weights'])
    known = [0.0] * len(g['positions'])
    fixed = [False] * len(known)
    for i, w in enumerate(rig['weights']):
        fixed[i] = True
        known[i] = sum((a for j, a in zip(w['joints'], w['weights']) if j == anchor))
    for source, guard in plan['pairs']:
        known[guard] = known[source]
        fixed[guard] = True
    for port in parts['ports']:
        assert len({known[i] for i in port}) == 1
    assert [known[r[0]] for r in parts['ports']] == [0, 1, 0]
    adj = [set() for _ in known]
    for t in g['triangles']:
        for a, b in zip(t, t[1:] + t[:1]):
            adj[a].add(b)
            adj[b].add(a)
    unknown = list(range(nv, end))
    mapping = {v: i for i, v in enumerate(unknown)}
    rows = [[mapping[j] for j in sorted(adj[i]) if not fixed[j]] for i in unknown]
    diag = [len(adj[i]) for i in unknown]
    rhs = [sum((known[j] for j in adj[i] if fixed[j])) for i in unknown]

    def apply(x):
        return [degree * x[i] - sum((x[j] for j in neighbors)) for i, (degree, neighbors) in enumerate(zip(diag, rows))]
    # Jacobi-preconditioned conjugate gradients; no nearest-bone skin inference.
    # Only the new iso vertices are unknown. Source and guard values stay fixed.
    x = [0.5] * len(unknown)
    r = [b - a for a, b in zip(apply(x), rhs)]
    z = [a / d for a, d in zip(r, diag)]
    p = z[:]
    rz = sum((a * b for a, b in zip(r, z)))
    iterations = 0
    for iterations in range(1, 257):
        ap = apply(p)
        den = sum((a * b for a, b in zip(p, ap)))
        assert den > 0
        alpha = rz / den
        x = [a + alpha * b for a, b in zip(x, p)]
        r = [a - alpha * b for a, b in zip(r, ap)]
        if max(map(abs, r)) < 1e-06:
            break
        z = [a / d for a, d in zip(r, diag)]
        new = sum((a * b for a, b in zip(r, z)))
        p = [a + new / rz * b for a, b in zip(z, p)]
        rz = new
    residual = max((abs(a - b) for a, b in zip(apply(x), rhs)))
    assert residual < 1e-06, residual
    assert min(x) > -1e-05 and max(x) < 1 + 1e-05
    for i, w in zip(unknown, x):
        known[i] = max(0, min(1, w))
    rig['weights'] += [{'joints': [root, anchor, 0, 0], 'weights': [1 - w, w, 0, 0]} for w in known[nv:]]
    out['id'] = 'branch-editable-1'
    out['rig'] = rig
    out['clips'] = parent['clips']
    return (out, {'iterations': iterations, 'max_laplacian_residual': residual, 'range': [min(x), max(x)], 'new_bone': 'secondary06-tip'})

def main():
    fitted = json.loads((R / 'artifacts/fit.json').read_text())
    parts = json.loads((R / 'artifacts/parts.json').read_text())
    c = Client(P / 'target/release/ignis')
    call = c.call
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'live-junction-pose-probe', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/input.scene.json')
        initial = call('inspect')['scene']
        out, report = skin(fitted, initial, parts)
        poses = {(i['id'], t): call('inspect_mesh_pose', instance=i['id'], tick=t)['pose'] for i in initial['meshes']['instances'] for t in [0, 60, 240]}
        batch = [{'op': 'plant', 'edit': {'kind': 'detach', 'id': 'branch-editable-1'}}, {'op': 'mesh', 'edit': {'kind': 'put_asset', 'asset': out}}, {'op': 'mesh', 'edit': {'kind': 'remove_instance', 'id': 'tree-secondary-06'}}]
        for i in initial['meshes']['instances']:
            if i['id'] in ['crown-06', 'tree-secondary-twigs']:
                i = copy.deepcopy(i)
                i['attachment'] = {'parent': 'tree-branch-editable-1', 'bone': 'secondary06-tip'}
                batch.append({'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': i}})
        candidate = copy.deepcopy(initial)
        candidate['plants']['entries'] = [e for e in candidate['plants']['entries'] if e['definition']['id'] != 'branch-editable-1']
        candidate['meshes']['assets'] = [out if a['id'] == 'branch-editable-1' else a for a in candidate['meshes']['assets']]
        candidate['meshes']['instances'] = [i for i in candidate['meshes']['instances'] if i['id'] != 'tree-secondary-06']
        for item in batch[3:]:
            inst = item['edit']['instance']
            candidate['meshes']['instances'] = [inst if i['id'] == inst['id'] else i for i in candidate['meshes']['instances']]
        (R / 'examples/proposal.scene.json').write_text(json.dumps(candidate, separators=(',', ':')))
        call('load_scene', path='examples/proposal.scene.json')
        published = call('inspect')['scene']
        call('undo')
        assert call('inspect')['scene'] == initial
        call('redo')
        assert call('inspect')['scene'] == published
        topology = call('inspect_mesh_topology', asset='branch-editable-1')['topology']
        assert topology['boundary_edges'] == topology['non_manifold_edges'] == 0
        assert not topology['isolated_vertices']
        assert len(call('inspect_mesh_islands', asset='branch-editable-1')['topology']['islands']) == 1
        errors = {}
        for (id, t), old in poses.items():
            if id == 'tree-secondary-06':
                continue
            new = call('inspect_mesh_pose', instance=id, tick=t)['pose']
            expected = old['positions']
            if id == 'tree-branch-editable-1':
                expected = [old['positions'][i] for i in parts['pids']] + [poses['tree-secondary-06', t]['positions'][i] for i in parts['cids']]
                positions = new['positions'][:len(expected)]
            else:
                positions = new['positions']
            assert len(positions) == len(expected)
            error = max((math.dist(a, b) for a, b in zip(positions, expected)))
            errors[f'{id}:{t}'] = error
            assert error < 3e-06, (id, t, error)
        rest = call('inspect_mesh_pose', instance='tree-branch-editable-1', tick=0)['pose']['positions']
        minimum_cosine = 1.
        for tick in [30, 60, 90, 120, 150, 180, 210, 240]:
            posed = call('inspect_mesh_pose', instance='tree-branch-editable-1', tick=tick)['pose']['positions']
            for triangle in out['geometry']['triangles'][fitted['material_plan']['retained_faces']:]:
                a, b = normal(rest, triangle), normal(posed, triangle)
                cosine = dot(a, b) / math.sqrt(dot(a, a) * dot(b, b))
                minimum_cosine = min(minimum_cosine, cosine)
        assert minimum_cosine > 0, minimum_cosine
        call('save_scene', path='examples/shape-study.scene.json')
        report.update(retained_pose_errors_m=errors, union=fitted['report'], native_regeneration=False, baked_study_only=True,
                      closed_connected=True, minimum_new_face_pose_cosine=minimum_cosine,
                      build_id=call('inspect')['build_id'], native_window=False)
        cam = {'position': [3.0, 4.8, 4.8], 'target': [0.65, 3.35, 0.63], 'fov_degrees': 22}
        for label, path in [('before', 'examples/input.scene.json'), ('after', 'examples/shape-study.scene.json')]:
            call('load_scene', path=path)
            scene = call('inspect')['scene']
            edits = [{'op': 'mesh', 'edit': {'kind': 'remove_instance', 'id': i['id']}} for i in scene['meshes']['instances'] if i['id'].startswith('crown-')]
            edits += [{'op': 'mesh', 'edit': {'kind': 'set_material', 'asset': a['id'], 'material': {}}} for a in scene['meshes']['assets'] if a['id'] in ['wood', 'branch-editable-1', 'secondary-06', 'secondary-twigs']]
            call('edit_scene', edits=edits)
            call('set_light', light={'direction': [-0.8, -1, -0.4], 'ambient': 0.45})
            call('set_camera', camera=cam)
            call('step', ticks=60)
            call('capture', path=f'artifacts/{label}-neutral.png', width=1100, height=1000)
        call('load_scene', path='examples/shape-study.scene.json')
        call('set_camera', camera=cam)
        call('capture', path='artifacts/start.png', width=1100, height=1000)
        call('step', ticks=60)
        call('capture', path='artifacts/live.png', width=1100, height=1000)
        call('capture', path='artifacts/repeat.png', width=1100, height=1000)
        live = (R/'artifacts/live.png').read_bytes()
        assert live == (R/'artifacts/repeat.png').read_bytes()
        call('save_replay', path='artifacts/study.replay.json')
        assert call('run_replay', path='artifacts/study.replay.json')['meshes_matched']
        for label, tick in [('replay', 60), ('replay-prefix', 0)]:
            call('capture_replay', replay_path='artifacts/study.replay.json', path=f'artifacts/{label}.png',
                 tick=tick, width=1100, height=1000)
            expected = live if tick else (R/'artifacts/start.png').read_bytes()
            assert (R/f'artifacts/{label}.png').read_bytes() == expected
        call('reset')
        call('capture', path='artifacts/reset.png', width=1100, height=1000)
        assert (R/'artifacts/reset.png').read_bytes() == (R/'artifacts/start.png').read_bytes()
        report['passed'] = True
        report['repeat_replay_prefix_reset_exact'] = True
        (R / 'artifacts/pose-report.json').write_text(json.dumps(report, indent=2))
        print(json.dumps({k:v for k,v in report.items() if k not in {'retained_pose_errors_m', 'union'}}))
    finally:
        c.close()
if __name__ == '__main__':
    main()
