#!/usr/bin/env python3
"""Diagnose pre-material normals at cottage contacts without editing the source."""
import hashlib
import copy
import json
import math
import os
from pathlib import Path
P = Path(__file__).resolve().parents[1]
R = P/'artifacts/cottage-normals'


def main():
    source = P/'artifacts/cottage-lookdev/examples/cottage-lookdev.scene.json'
    source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
    canonical = P/'examples/elven-cottage.scene.json'
    canonical_hash = hashlib.sha256(canonical.read_bytes()).hexdigest()
    for name in ['examples', 'artifacts']:
        (R/name).mkdir(parents=True, exist_ok=True)
    (R/'examples/input.scene.json').write_bytes(source.read_bytes())
    os.environ['IGNIS_VERIFY_ROOT'] = str(R)
    from verify_lab import Client
    c = Client(P/'target/release/ignis')
    call = c.call
    checks = []
    def capture(name):
        path = 'artifacts/'+name+'.png'
        call('capture', path=path, width=1536, height=1024)
        return (R/path).read_bytes()
    def angle(a, b):
        dot = sum(x*y for x,y in zip(a,b))
        norm = math.sqrt(sum(x*x for x in a)*sum(x*x for x in b))
        return math.degrees(math.acos(max(-1,min(1,dot/norm))))
    try:
        c.request('initialize', {'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'cottage-normal-study','version':'1'}})
        c.request('notifications/initialized', notification=True)
        call('load_scene', path='examples/input.scene.json')
        recipe = json.loads((P/'examples/cottage-lookdev.recipe.json').read_text())
        assert canonical_hash == recipe['source_sha256']
        call('set_camera', camera=recipe['cameras']['entry'])
        baseline = capture('baseline-entry')
        assert baseline == (P/'artifacts/cottage-lookdev/artifacts/after-entry.png').read_bytes()
        before = call('inspect')
        asset = call('inspect_mesh', asset='carved-stone')['asset']
        pose = call('inspect_mesh_pose', instance='cottage-carved-stone')['pose']
        regions = {'door-left-spring':([.51,2.491,2.38],.16),
                   'door-crown':([1.3,3.68,2.38],.18),
                   'portico-crown':([1.45,4.02,3.15],.2)}
        reports = {}
        for name,(center,radius) in regions.items():
            selected = []
            for face,t in enumerate(asset['geometry']['triangles']):
                if any(sum((pose['positions'][v][k]-center[k])**2 for k in range(3)) <= radius**2 for v in t):
                    selected.append(face)
            assert selected, name
            samples = []
            for start in range(0,len(selected),32):
                q = call('inspect_mesh_normals', instance='cottage-carved-stone', faces=selected[start:start+32], sample_limit=32)['report']
                assert q['selected_faces'] == len(q['samples'])
                assert q['geometric_fallback_faces'] == q['smooth_fallback_corners'] == 0
                samples.extend(q['samples'])
            # Candidate contacts only; positional coincidence does not weld indices.
            corners = {}
            for s in samples:
                for index,p,n in zip(s['vertices'],s['positions'],s['corner_normals']):
                    if sum((p[k]-center[k])**2 for k in range(3)) <= radius**2:
                        corners[index] = (p,n)
            pairs = []
            items = sorted(corners.items())
            for j,(a,(p,na)) in enumerate(items):
                for b,(q,nb) in items[j+1:]:
                    distance = math.dist(p,q)
                    if distance <= 1e-5:
                        pairs.append(dict(vertices=[a,b], distance=distance, normal_degrees=angle(na,nb),position=p))
            pairs.sort(key=lambda p:(-p['normal_degrees'],p['vertices']))
            reports[name] = dict(center=center,radius=radius,selected_faces=selected,samples=samples,
                                 coincident_distinct_index_pairs=len(pairs),maximum_contact_degrees=pairs[0]['normal_degrees'] if pairs else None,worst_contacts=pairs[:8])
        assert call('inspect') == before and capture('after-query') == baseline
        checks.append('Combined house baseline exact on new renderer helper; native posed-normal queries preserve all scene/runtime/history and pixels')
        call('edit_mesh', edit={'kind':'set_shading','asset':'carved-stone','shading':'flat'})
        flat = capture('flat-entry'); assert flat != baseline
        q = call('inspect_mesh_normals',instance='cottage-carved-stone',sample_limit=32)['report']
        assert q['maximum_degrees'] == 0
        call('undo'); assert capture('smooth-restored') == baseline
        checks.append('Region queries at door spring/crown and portico crown expose actual normal angles; flat ablation and undo separate shading from geometry without source edits')
        call('edit_mesh',edit={'kind':'set_material','asset':'carved-stone','material':{}})
        capture('no-procedural-entry'); call('undo'); assert capture('material-restored') == baseline
        checks.append('Neutral material ablation preserves geometry; source file unchanged and exact restoration after diagnostics')
        # Reauthor one tangent contact with existing native Sweep; preserve canonical source.
        before = call('inspect')
        from demo_elven_cottage import build
        cfg = json.loads((P/'examples/elven-cottage.recipe.json').read_text())
        raw = next(a for a in build(cfg)[0] if a['id'] == 'carved-stone')
        sweeps = raw.pop('_sweeps')
        planned = call('plan_mesh_sweep', sweeps=sweeps)['plan']
        for sweep, plan in zip(sweeps, planned['pieces']):
            sweep['spans'] = plan['spans']
        prefix = {**asset, 'geometry':raw['geometry']}
        def edits(parts):
            return [dict(op='mesh',edit=dict(kind='put_asset',asset=prefix)),
                    dict(op='mesh',edit=dict(kind='sweep',id='carved-stone',sweeps=parts,append=True))]
        restored = call('preview_scene_edits',edits=edits(sweeps))['scene']
        regenerated = next(a for a in restored['meshes']['assets'] if a['id']=='carved-stone')
        assert regenerated == asset and call('inspect') == before
        merged = copy.deepcopy(sweeps[0])
        assert len(sweeps[0]['curves']) == len(sweeps[1]['curves']) == 1
        assert sweeps[0]['curves'][0][-1] == sweeps[1]['curves'][0][0]
        assert sweeps[0]['profile'] == sweeps[1]['profile']
        merged['curves'] += copy.deepcopy(sweeps[1]['curves'])
        merged['spans'] = (sweeps[0]['spans'] + sweeps[1]['spans']) // 2
        uniform_plan = call('plan_mesh_sweep',sweeps=[merged])['plan']['pieces'][0]
        merged['spans'] = uniform_plan['spans']
        uniform_preview = call('preview_scene_edits',edits=edits([merged]+sweeps[2:]))['scene']
        assert call('inspect') == before
        call('edit_scene',edits=edits([merged]+sweeps[2:]))
        assert call('inspect')['scene'] == uniform_preview
        capture('uniform-entry'); call('undo'); assert capture('uniform-undo') == baseline
        before = call('inspect')
        merged['spans'] = [sweeps[0]['spans'],sweeps[1]['spans']]
        plan = call('plan_mesh_sweep',sweeps=[merged])['plan']['pieces'][0]
        assert plan['spans'][0] == 1 and plan['vertices'] < uniform_plan['vertices']
        merged['spans'] = plan['spans']
        revised = [merged] + sweeps[2:]
        preview = call('preview_scene_edits',edits=edits(revised))['scene']
        assert call('inspect') == before
        def invariant(scene):
            s=copy.deepcopy(scene)
            s['meshes']['assets']=[a for a in s['meshes']['assets'] if a['id']!='carved-stone']
            return s
        assert invariant(preview) == invariant(before['scene'])
        call('edit_scene',edits=edits(revised)); assert call('inspect')['scene'] == preview
        improved = capture('joined-entry'); assert improved != baseline
        new_asset = call('inspect_mesh',asset='carved-stone')['asset']
        old_count = sum(planned['pieces'][i]['vertices'] for i in [0,1])
        old_triangles = sum(planned['pieces'][i]['triangles'] for i in [0,1])
        start = len(prefix['geometry']['positions']); start_face=len(prefix['geometry']['triangles'])
        delta = plan['vertices'] - old_count; delta_faces=plan['triangles']-old_triangles
        sides = len(merged['profile'])
        first_piece = planned['pieces'][0]['vertices']
        expected_joined = asset['geometry']['positions'][start:start+first_piece] + asset['geometry']['positions'][start+first_piece+sides:start+old_count]
        assert new_asset['geometry']['positions'][start:start+plan['vertices']] == expected_joined
        assert asset['geometry']['positions'][:start] == new_asset['geometry']['positions'][:start]
        assert asset['geometry']['positions'][start+old_count:] == new_asset['geometry']['positions'][start+plan['vertices']:]
        for old,new in zip(asset['geometry']['triangles'][start_face+old_triangles:],new_asset['geometry']['triangles'][start_face+plan['triangles']:]):
            assert [i+delta for i in old] == new
        assert asset['geometry']['material_coordinates'][start+old_count:] == new_asset['geometry']['material_coordinates'][start+plan['vertices']:]
        target = reports['door-left-spring']['worst_contacts'][0]['position']
        def at_target(geometry):
            return [i for i,p in enumerate(geometry['positions']) if math.dist(p,target)<=1e-5]
        old_indices=at_target(asset['geometry']); new_indices=at_target(new_asset['geometry'])
        assert len(old_indices)==2 and len(new_indices)==1
        incident=[i for i,t in enumerate(new_asset['geometry']['triangles']) if new_indices[0] in t]
        q=call('inspect_mesh_normals',instance='cottage-carved-stone',faces=incident,sample_limit=32)['report']
        target_normals=[n for s in q['samples'] for i,n in zip(s['vertices'],s['corner_normals']) if i==new_indices[0]]
        assert len(target_normals)>1 and all(n==target_normals[0] for n in target_normals)
        call('undo'); assert capture('joined-undo') == baseline
        call('redo'); assert capture('joined-redo') == improved
        call('save_scene',path='examples/cottage-spring.scene.json')
        call('step',ticks=30); assert capture('joined-tick30') == improved
        call('save_replay',path='artifacts/joined.replay.json')
        assert call('run_replay',path='artifacts/joined.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/joined.replay.json',path='artifacts/joined-replay.png',width=1536,height=1024)
        assert (R/'artifacts/joined-replay.png').read_bytes()==improved
        call('load_scene',path='examples/cottage-spring.scene.json'); assert capture('joined-load')==improved
        correction=dict(parts=[0,1],native_plan=plan,uniform_plan=uniform_plan,vertices_saved_against_uniform=uniform_plan['vertices']-plan['vertices'],triangles_saved_against_uniform=uniform_plan['triangles']-plan['triangles'],vertices_delta=delta,triangles_delta=delta_faces,
                        old_coincident_vertices=old_indices,new_shared_vertex=new_indices[0],
                        before_normal_jump=reports['door-left-spring']['maximum_contact_degrees'],after_normal_jump=0,
                        remaining_spans_exact=True,joined_positions_equal_source_without_duplicate_ring=True,canonical_source_unchanged=True)
        checks.append('Original asset regeneration exact; one tangent span contact reauthored by native independently sampled chained Sweep, fewer vertices than uniform, other spans/channels/components exact, one shared index, normal jump0, undo/redo/save/load/replay exact')
        hero_camera = json.loads(source.read_text())['camera']
        call('set_camera',camera=hero_camera); hero = capture('joined-hero')
        call('save_scene',path='examples/cottage-spring.scene.json')
        call('load_scene',path='examples/cottage-spring.scene.json'); assert capture('joined-hero-loaded') == hero
        detail_camera = dict(position=[2.5,3.2,6.5],target=[.62,2.5,2.35],fov_degrees=20)
        call('set_camera',camera=detail_camera); capture('joined-spring-close')
        call('load_scene',path='examples/input.scene.json'); call('set_camera',camera=detail_camera); capture('baseline-spring-close')
        assert hashlib.sha256(source.read_bytes()).hexdigest() == source_hash
        assert hashlib.sha256(canonical.read_bytes()).hexdigest() == canonical_hash
        report = dict(passed=True,checks=checks,build_id=call('inspect')['build_id'],source_sha256=source_hash,canonical_sha256=canonical_hash,baseline_sha256=hashlib.sha256(baseline).hexdigest(),regions=reports,correction=correction,
                      limitations=['Pre-material world normals only; bump, face orientation in shader and lighting excluded','Coincident indices are candidates, not a proof of faulty shading or welded topology','No automatic normal inversion, gap repair or architectural acceptance'])
        (R/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({**{k:v for k,v in report.items() if k!='regions'},'regions':{k:{x:y for x,y in v.items() if x not in ['samples','selected_faces']} for k,v in reports.items()}},indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    main()
