#!/usr/bin/env python3
"""Native boundary bridge and bounded brush study; upper bridges remain rejected."""
import argparse,hashlib,json,math,os,shutil,sys
from pathlib import Path
PROJECT=Path(__file__).resolve().parents[1];OUTPUT=PROJECT/('artifacts/tree-bridge-upper-probe' if '--include-upper' in sys.argv else 'artifacts/tree-bridges')
BASELINE=PROJECT/'artifacts/tree-bridges/examples/input.scene.json'
(OUTPUT/'artifacts').mkdir(parents=True,exist_ok=True);(OUTPUT/'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT']=str(OUTPUT)
from verify_lab import Client
CAMERA={'position':[11,8,15],'target':[0,3.65,0],'fov_degrees':34}
LIGHT={'direction':[-.8,-1,-.4],'ambient':.28,'shadow':{'center':[0,3.1,0],'extent':5.8,'resolution':2048,'bias':.0008}}

def centre(points):return [sum(p[k] for p in points)/len(points) for k in range(3)]
def unit(p):
    d=math.sqrt(sum(x*x for x in p));return [x/d for x in p]
def dihedrals(g,loops):
    edges={}
    for tri in g['triangles']:
        a,b,c=[g['positions'][i] for i in tri];u=[b[k]-a[k] for k in range(3)];v=[c[k]-a[k] for k in range(3)];n=unit([u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]])
        for a,b in zip(tri,tri[1:]+tri[:1]):edges.setdefault(tuple(sorted([a,b])),[]).append(n)
    angles=[]
    for loop in loops:
        for a,b in zip(loop,loop[1:]+loop[:1]):
            ns=edges[tuple(sorted([a,b]))]
            if len(ns)==2:angles.append(math.degrees(math.acos(max(-1,min(1,sum(x*y for x,y in zip(*ns)))))))
    return {'max_degrees':max(angles),'mean_degrees':sum(angles)/len(angles)} if angles else None

def region_dihedrals(g, triangles):
    edges={tuple(sorted((a,b))) for t in triangles for a,b in zip(t,t[1:]+t[:1])}
    return dihedrals(g,[list(e) for e in sorted(edges)])

def volume(g):
    total=0.
    for tri in g['triangles']:
        a,b,c=[g['positions'][i] for i in tri]
        total+=a[0]*(b[1]*c[2]-b[2]*c[1])+a[1]*(b[2]*c[0]-b[0]*c[2])+a[2]*(b[0]*c[1]-b[1]*c[0])
    return abs(total)/6

def run(input_path, include_upper=False):
    source=json.loads(input_path.read_text())
    assert len(next(a['geometry']['positions'] for a in source['meshes']['assets'] if a['id']=='wood'))==3808,'Use the pre-bridge snapshot; refusing to overwrite it with the current tree'
    snapshot=OUTPUT/'examples/input.scene.json'
    if input_path.resolve()!=snapshot.resolve():shutil.copyfile(input_path,snapshot)
    c=Client(PROJECT/'target/release/ignis',False);call=c.call
    def asset():return call('inspect_mesh',asset='wood')['asset']
    def edit(edit):return call('edit_mesh',edit=edit)
    def capture(name,camera):
        call('set_camera',camera=camera);call('step',ticks=60);assert call('inspect')['tick']==60
        path=f'artifacts/{name}.png';meta=call('capture',path=path,width=1100,height=1000)
        return {'sha256':hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest(),'draw_triangles':meta['mesh_triangle_count']}
    cameras={'whole':CAMERA,'upper':{'position':[2.9,4,5.5],'target':[.15,3.5,.1],'fov_degrees':35},'back':{'position':[-3.5,4.2,-5],'target':[.1,3.7,0],'fov_degrees':35},'bark':{'position':[1.3,2.3,2],'target':[.1,2,.1],'fov_degrees':32}}
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-tree-bridges','version':'1'}});c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json');call('set_light',light=LIGHT)
        initial=call('inspect')['scene'];original=asset();g=original['geometry'];assert len(g['positions'])==3808,'Use the pre-bridge saved tree'
        assert call('inspect_mesh_boundaries',asset='wood')['boundaries']==[]
        count_before=len(call('inspect_mesh_islands',asset='wood')['topology']['islands'])
        poses={t:{i['id']:call('inspect_mesh_pose',instance=i['id'],tick=t)['pose'] for i in initial['meshes']['instances'] if i['asset']!='wood'} for t in [0,60]}
        views={name:{'before':capture('before-'+name,cam)} for name,cam in cameras.items()}
        back_tile={r+j for r in [1167,1199,1231] for j in range(14,19)}
        front_tile={r+j for r in [1231,1263,1295] for j in [30,31,0,1,2]}
        cuts=[]
        for tile in ([back_tile,front_tile] if include_upper else []):
            chosen=[i for i,t in enumerate(g['triangles']) if set(t)<=tile];assert len(chosen)==16;cuts+=chosen
        removed_vertices=set(range(1647,1775))
        if include_upper:removed_vertices |= set(range(2282,2318))|set(range(2577,2613))
        cuts+= [i for i,t in enumerate(g['triangles']) if any(v in removed_vertices for v in t)]
        cuts=sorted(set(cuts));assert len(cuts)==(516 if include_upper else 320),len(cuts)
        used=sorted({v for i,t in enumerate(g['triangles']) if i not in set(cuts) for v in t});mapping={old:i for i,old in enumerate(used)};reverse={i:old for old,i in mapping.items()}
        before=call('inspect');reference=call('inspect_mesh_boundaries',asset='wood')['reference']
        cut={'kind':'remove_faces','asset':'wood','reference':reference,'faces':cuts}
        predicted=call('preview_mesh_edit',edit=cut);assert call('inspect')==before
        edit(cut);assert call('inspect')['scene']['meshes']==predicted['meshes']
        cut_asset=asset();assert cut_asset['geometry']['positions']==[g['positions'][i] for i in used]
        assert cut_asset['geometry']['material_coordinates']==[g['material_coordinates'][i] for i in used]
        info=call('inspect_mesh_boundaries',asset='wood');assert len(info['boundaries'])==(6 if include_upper else 2)
        def find(old_set):
            found=[b['vertices'] for b in info['boundaries'] if {reverse[i] for i in b['vertices']}<=old_set];assert len(found)==1;return found[0]
        jobs=[('neck',find(set(range(1615,1647))),find(set(range(1775,1807))),10)]
        if include_upper:jobs=[('back',find(back_tile),find(set(range(2318,2330))),6),('front',find(front_tile),find(set(range(2613,2625))),6)]+jobs
        reports=[]
        for name,a,b,spans in jobs:
            a=list(reversed(a));reference=call('inspect_mesh_boundaries',asset='wood')['reference'];current=asset();points=current['geometry']['positions']
            ca=centre([points[i] for i in a]);cb=centre([points[i] for i in b]);n=len(a)
            phases=sorted(range(n),key=lambda phase:sum(sum((x-y)**2 for x,y in zip(unit([points[a[j]][k]-ca[k] for k in range(3)]),unit([points[b[(j+phase)%n]][k]-cb[k] for k in range(3)]))) for j in range(n)))
            failures=[];chosen=None;best=float("inf")
            for phase in phases:
                second=b[phase:]+b[:phase]
                for handles in ([[.015,.12],[.03,.15],[.06,.2],[.1,.25],[.15,.2],[.25,.3],[.4,.4]] if name!='neck' else ([[h,h] for h in [.15,.25,.35,.45,.6]]+[[.2,.4],[.4,.2],[.3,.5],[.5,.3]])):
                    operation={'kind':'bridge_boundary_loops','asset':'wood','reference':reference,'loops':[a,second],'spans':spans,'handles':handles}
                    state=call('inspect')
                    raw=c.request('tools/call',{'name':'engine_command','arguments':{'command':{'op':'preview_mesh_edit','edit':operation}}})['result']
                    assert call('inspect')==state
                    if raw['isError']:failures.append(raw['structuredContent']['message']);continue
                    preview=raw['structuredContent'];pg=next(a['geometry'] for a in preview['meshes']['assets'] if a['id']=='wood')
                    metric=region_dihedrals(pg,pg['triangles'][-2*n*spans:]);score=metric['max_degrees']+metric['mean_degrees']*.1
                    if score<best:best=score;chosen=(operation,preview)
                if name=='neck':break
            assert chosen,(name,failures[:2])
            operation,preview=chosen
            accepted_geometry=next(a['geometry'] for a in preview['meshes']['assets'] if a['id']=='wood')
            metric=region_dihedrals(accepted_geometry,accepted_geometry['triangles'][-2*n*spans:])
            assert metric['max_degrees'] < 85, (name,metric)
            edit(operation);assert call('inspect')['scene']['meshes']==preview['meshes']
            reports.append({'name':name,'operation':operation,'failed_previews':len(failures),'join_dihedrals':dihedrals(asset()['geometry'],operation['loops']),'region_dihedrals':metric})
        unpolished=asset()
        call('undo');assert len(call('inspect_mesh_boundaries',asset='wood')['boundaries'])==2
        call('redo');assert asset()==unpolished
        old_points=unpolished['geometry']['positions'][:len(used)]
        strokes=[]
        for iteration in range(2):
            points=asset()['geometry']['positions']
            centres=[p for p in points[len(used):] if min(math.dist(p,q) for q in old_points)>.005]
            for offset in range(0,len(centres),128):
                operation={'kind':'sculpt','asset':'wood','stroke':{'kind':'smooth','centers':centres[offset:offset+128],
                    'radius':.002,'strength':.3,'direction':[0,0,0]}}
                before=call('inspect');preview=call('preview_mesh_edit',edit=operation);assert call('inspect')==before
                edit(operation);assert call('inspect')['scene']['meshes']==preview['meshes'];strokes.append(operation)
        final=call('inspect')['scene'];actual=asset();after=actual['geometry']
        polished_metric=region_dihedrals(after,after['triangles'][-640:])
        assert polished_metric['max_degrees']<24.247,polished_metric
        assert max(math.dist(p,q) for p,q in zip(after['positions'],unpolished['geometry']['positions']))<.025
        polish_volume_change=abs(volume(after)/volume(unpolished['geometry'])-1)
        assert polish_volume_change<.02

        assert final['plants']==initial['plants'] and final['meshes']['instances']==initial['meshes']['instances']
        assert actual['material']==original['material'] and actual.get('rig')==original.get('rig') and actual.get('clips')==original.get('clips')
        assert [a for a in final['meshes']['assets'] if a['id']!='wood']==[a for a in initial['meshes']['assets'] if a['id']!='wood']
        assert after['positions'][:len(used)]==[g['positions'][i] for i in used]
        assert after['material_coordinates'][:len(used)]==[g['material_coordinates'][i] for i in used]
        for t,ps in poses.items():
            for id,p in ps.items():assert call('inspect_mesh_pose',instance=id,tick=t)['pose']==p
        topology=call('inspect_mesh_topology',asset='wood')['topology'];assert topology['boundary_edges']==topology['non_manifold_edges']==0 and not topology['isolated_vertices']
        assert call('inspect_mesh_boundaries',asset='wood')['boundaries']==[]
        count_after=len(call('inspect_mesh_islands',asset='wood')['topology']['islands']);assert count_after==count_before-(2 if include_upper else 0)
        for name,cam in cameras.items():views[name]['after']=capture('after-'+name,cam)
        call('set_camera',camera=CAMERA);call('save_scene',path='examples/candidate.scene.json')
        report={'passed':True,'build_id':call('inspect')['build_id'],'input_sha256':hashlib.sha256(snapshot.read_bytes()).hexdigest(),'cuts':len(cuts),'polishing':{'strokes':len(strokes),'global_wood_volume_change_fraction':polish_volume_change,'max_displacement':max(math.dist(p,q) for p,q in zip(after['positions'],unpolished['geometry']['positions'])),'region_dihedrals':polished_metric},'neck_before':dihedrals(g,[list(range(1615,1647)),list(range(1775,1807))]),'neck_region_before':region_dihedrals(g,[t for t in g['triangles'] if any(1647<=v<1775 for v in t)]),'bridges':reports,'views':views,'vertices':len(after['positions']),'triangles':len(after['triangles']),
            'components_before':count_before,'components_after':count_after,'retained_vertices_and_channels_exact':True,'other_assets_materials_plants_instances_poses_exact':True,'native_window':False,
            'remaining':['Upper static bridges rejected by internal triangle bend; retained original components','Opposite lower parametric insertion is preserved, not silently detached','Smaller secondary branch intersections','Finite angular resolution and material grain domain']}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['views','bridges']},indent=2))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--input',type=Path,default=BASELINE if BASELINE.exists() else PROJECT/'examples/native-tree-study.scene.json');p.add_argument('--include-upper',action='store_true',help='Rejected upper-junction experiment; never promotes the central source');a=p.parse_args();run(a.input.resolve(),a.include_upper)
