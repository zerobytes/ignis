#!/usr/bin/env python3
"""Add a second static context graft while preserving the live plant pair."""
import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
P=Path(__file__).resolve().parents[1]
R=P/'artifacts/tree-second-graft'
ID='branch-editable-1'

def prepare(call,scene):
    inspected=call('inspect_mesh_islands',asset='wood')
    g=next(a['geometry'] for a in scene['meshes']['assets'] if a['id']=='wood')
    island=min(i for i,t in enumerate(g['triangles']) if 35 in t)
    part=next(p for p in inspected['topology']['islands'] if p['id']==island)
    assert part['vertices']==119 and part['triangles']==234
    extract={'kind':'extract_islands','asset':'wood','id':'secondary-02-source','reference':inspected['reference'],'islands':[island],'mode':'move'}
    state=call('inspect');preview=call('preview_mesh_edit',edit=extract);assert call('inspect')==state
    source=next(a for a in preview['meshes']['assets'] if a['id']=='secondary-02-source')
    assert len(source['geometry']['positions'])==119
    root=set(range(7))|set(range(35,56))
    faces=[i for i,t in enumerate(source['geometry']['triangles']) if any(v in root for v in t)]
    entry=call('inspect_plant',id=ID)['plant'];assembly=copy.deepcopy(entry['assembly'])
    rings=assembly['context_lofts'][0]['rings'];lo=next(i for i,r in enumerate(rings) if 1647 in r);hi=next(i for i,r in enumerate(rings) if 1711 in r);assert lo<hi
    selected=[v for r in rings[lo:hi+1] for v in r]
    assert not set(selected).intersection(assembly['context_grafts'][0]['context_vertices'])
    placement=next(i for i in scene['meshes']['instances'] if i['asset']==ID)
    assert placement['rotation']==[0,0,0,1] and placement['scale']==1
    offset=placement['position'];cg=assembly['context']['geometry']
    center=lambda vertices:[[cg['positions'][v][k] for v in vertices] for k in range(3)]
    centers=[[sum(xs)/len(xs) for xs in center(r)] for r in [rings[lo],rings[hi]]]
    hub=[(a+b)/2 for a,b in zip(*centers)]
    root_center=[sum(p[k] for p in source['geometry']['positions'][:7])/7 for k in range(3)]
    affine=lambda delta:{'columns':[[1,0,0],[0,1,0],[0,0,1]],'offset':delta}
    graft={'source':source,'source_translation':[-v for v in offset],'source_faces':faces,'source_loop':list(range(7,14)),'context_vertices':selected,'context_loops':[rings[lo],rings[hi]],
           'source_material_frame':affine(root_center),
           'material_finish':{'reference':affine(offset),'iterations':64,'tolerance':.002,'max_change':5,'max_anisotropy':10000},
           'options':{'hub':hub,'hub_radius':.07,'blend':.035,'cell_size':.026,'guard_length':.004,'collar_length':.02,'relaxation_steps':3,'material_offset':offset,'seam_relaxation_steps':0,'seam_max_displacement':.004,'material_relaxation_steps':0}}
    return entry,assembly,graft,extract

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--input',type=Path,default=P/'artifacts/plant-connections/examples/connected.scene.json');ap.add_argument('--study',action='store_true');ap.add_argument('--refine',action='store_true');ap.add_argument('--recipe',type=Path,default=P/'examples/native-tree-secondary02.graft.json');args=ap.parse_args()
    for d in ['examples','artifacts']:(R/d).mkdir(parents=True,exist_ok=True)
    (R/'examples/input.scene.json').write_bytes(args.input.read_bytes())
    os.environ['IGNIS_VERIFY_ROOT']=str(R)
    from verify_lab import Client
    c=Client(P/'target/release/ignis');call=c.call;results=[]
    def mesh(id):return call('inspect_mesh',asset=id)['asset']
    def edit(recipe,extraction):
        return [{'op':'mesh','edit':extraction},{'op':'mesh','edit':{'kind':'remove_asset','id':'secondary-02-source'}},{'op':'plant','edit':{'kind':'set_assembly','id':ID,'assembly':recipe}}]
    def capture(name):
        path=f'artifacts/{name}.png';meta=call('capture',path=path,width=1100,height=1000);return (R/path).read_bytes(),meta
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-second-graft','version':'1'}});c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json');initial=call('inspect')['scene']
        old,assembly,graft,extraction=prepare(call,initial)
        if args.study:
            variants=[('base',{}),('soft065',{'hub_radius':.065,'blend':.04,'cell_size':.024}),('wide080',{'hub_radius':.08,'blend':.04,'cell_size':.024}),('soft070',{'hub_radius':.07,'blend':.05,'cell_size':.026}),('coarse065',{'hub_radius':.065,'blend':.03,'cell_size':.028}),('soft060',{'hub_radius':.06,'blend':.05,'cell_size':.025})]
            if args.refine:variants=[('round070',{'hub_radius':.07,'blend':.05,'cell_size':.032}),('round065',{'hub_radius':.065,'blend':.065,'cell_size':.032,'relaxation_steps':5})]
            for name,values in variants:
                g=copy.deepcopy(graft);g['options'].update(values);recipe=copy.deepcopy(assembly);recipe['context_grafts'].append(g);batch=edit(recipe,extraction);state=call('inspect')
                res=c.request('tools/call',{'name':'engine_command','arguments':{'command':{'op':'preview_scene_edits','edits':batch}}})['result'];assert call('inspect')==state
                if res['isError']:
                    results.append({'name':name,'passed':False,'error':res.get('structuredContent',res['content'])});print(name,'rejected',results[-1]['error'],flush=True);continue
                call('edit_scene',edits=batch);actual=call('inspect')['scene'];assert actual==res['structuredContent']['scene']
                report=call('inspect_plant',id=ID)['report']['assembly']['context_grafts'][-1]
                call('save_scene',path='examples/candidate.scene.json')
                # Clay view only, leaving authored appearance saved above.
                edits=[{'op':'mesh','edit':{'kind':'set_material','asset':id,'material':{}}} for id in ['wood',ID,'secondary-06','secondary-twigs']]
                for i in actual['meshes']['instances']:
                    if i['asset'] in ['wood',ID,'secondary-06','secondary-twigs']:
                        i=copy.deepcopy(i);i['color']=[.65,.65,.65];edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':i}})
                call('edit_scene',edits=edits);call('set_light',light={'direction':[.7,-.3,-1],'ambient':.2});call('set_camera',camera={'position':[-3.5,4.4,3.5],'target':[-1.2,3.85,.5],'fov_degrees':21});capture(name)
                results.append({'name':name,'passed':True,'graft':g,'report':report,'vertices':len(mesh(ID)['geometry']['positions'])});print(name,json.dumps({'vertices':results[-1]['vertices'],'junction':report['junction']}),flush=True)
                call('load_scene',path='examples/input.scene.json');extraction['reference']=call('inspect_mesh_islands',asset='wood')['reference']
            (R/('artifacts/study-refined.json' if args.refine else 'artifacts/study.json')).write_text(json.dumps({'build_id':call('inspect')['build_id'],'results':results},indent=2)+'\n')
            return
        from collections import Counter,defaultdict
        selected=json.loads(args.recipe.read_text())
        assert {k:v for k,v in selected['source'].items() if k!='material'}=={k:v for k,v in graft['source'].items() if k!='material'}
        # Use the exact extracted source, including the current authored material.
        selected['source']=graft['source']
        assert selected['source_faces']==graft['source_faces'] and selected['source_loop']==graft['source_loop']
        assert selected['context_vertices']==graft['context_vertices'] and selected['context_loops']==graft['context_loops']
        assembly['context_grafts'].append(selected)
        before_assets={a['id']:a for a in initial['meshes']['assets']}
        poses={(i['id'],t):call('inspect_mesh_pose',instance=i['id'],tick=t)['pose'] for i in initial['meshes']['instances'] for t in [0,60,240]}
        approved=call('inspect_plant_connections')['reports'][0]
        first_approved=approved['retained_parent_vertices']
        batch=edit(assembly,extraction);state=call('inspect')
        predicted=call('preview_scene_edits',edits=batch)['scene'];assert call('inspect')==state
        call('edit_scene',edits=batch);actual=call('inspect')['scene'];assert actual==predicted
        after_assets={a['id']:a for a in actual['meshes']['assets']}
        new_entry=call('inspect_plant',id=ID)['plant']
        for key in ['source','reference','definition']:assert new_entry[key]==old[key]
        assert new_entry['assembly']['context']==old['assembly']['context']
        assert new_entry['assembly']['context_grafts'][:-1]==old['assembly']['context_grafts']
        assert actual['plants']['entries'][1]==initial['plants']['entries'][1]
        assert actual['plants']['connections']==initial['plants']['connections']
        assert actual['meshes']['instances']==initial['meshes']['instances']
        assert after_assets[ID]['rig']['bones']==before_assets[ID]['rig']['bones'] and after_assets[ID]['clips']==before_assets[ID]['clips']
        for id,a in after_assets.items():
            if id not in ['wood',ID]:assert a==before_assets[id]
        # Exact extraction conservation in geometric/material domains.
        def points(g):return Counter((tuple(p),tuple(m)) for p,m in zip(g['positions'],g['material_coordinates']))
        def faces(g):return Counter(tuple(tuple(g['positions'][v]) for v in t) for t in g['triangles'])
        for count in [points,faces]:assert count(before_assets['wood']['geometry'])==count(after_assets['wood']['geometry'])+count(selected['source']['geometry'])
        topology=call('inspect_mesh_topology',asset=ID)['topology'];assert topology['boundary_edges']==topology['non_manifold_edges']==0 and not topology['isolated_vertices']
        assert len(call('inspect_mesh_islands',asset=ID)['topology']['islands'])==1
        # Identify old retained points by exact data, not nearest projection.
        parent=after_assets[ID];previous=before_assets[ID]
        lookup=defaultdict(list)
        for i,p in enumerate(parent['geometry']['positions']):lookup[tuple(p)].append(i)
        matching={}
        for i,p in enumerate(previous['geometry']['positions']):
            candidates=lookup[tuple(p)]
            chosen=next((j for j in candidates if parent['rig']['weights'][j]==previous['rig']['weights'][i]),None)
            if chosen is not None:matching[i]=chosen
        assert all(i in matching for i in range(first_approved,len(previous['geometry']['positions'])))
        assert len(previous['geometry']['positions'])-len(matching)==160
        # Match retained loose-branch vertices through the exact native f32 frame.
        import struct
        f32=lambda x:struct.unpack('f',struct.pack('f',x))[0]
        removed=set(selected['source_faces']);source=selected['source']['geometry']
        source_ids=sorted({v for i,t in enumerate(source['triangles']) if i not in removed for v in t})
        assert len(source_ids)==91
        source_map={}
        for i in source_ids:
            local=tuple(f32(p+f32(d)) for p,d in zip(source['positions'][i],selected['source_translation']))
            source_map[i]=lookup[local][0]
        wood_instance=next(i for i in initial['meshes']['instances'] if i['asset']=='wood')
        assert wood_instance['position']==[0,0,0] and wood_instance['rotation']==[0,0,0,1] and wood_instance['scale']==1
        static_error=0.

        old_faces={tuple(tuple(previous['geometry']['positions'][v]) for v in t) for t in previous['geometry']['triangles'] if any(v>=first_approved for v in t)}
        new_faces=faces(parent['geometry']);assert old_faces<=new_faces.keys()
        for i in range(first_approved,len(previous['geometry']['positions'])):
            assert previous['geometry']['material_coordinates'][i]==parent['geometry']['material_coordinates'][matching[i]]
        errors={}
        for (id,t),baseline in poses.items():
            if id=='tree-wood':continue
            current=call('inspect_mesh_pose',instance=id,tick=t)['pose']
            if id=='tree-branch-editable-1':
                error=max(math.dist(baseline['positions'][i],current['positions'][j]) for i,j in matching.items());assert error<2e-6
                static_error=max(static_error,max(math.dist(source['positions'][i],current['positions'][j]) for i,j in source_map.items()));assert static_error<2e-6
            else:assert current==baseline;error=0.
            errors[f'{id}:{t}']=error
        call('undo');assert call('inspect')['scene']==initial;call('redo');assert call('inspect')['scene']==actual
        # Existing plants still regenerate around the static mounted context.
        for id,delta in [(ID,.001),('secondary-06',.004)]:
            entry=call('inspect_plant',id=id)['plant'];d=copy.deepcopy(entry['definition']);d['trajectory']['control_points_xz'][1][0]+=delta
            call('edit_plant',edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'})
            call('edit_plant',edit={'kind':'put','definition':entry['definition'],'detail_policy':'physical','animation_translation_policy':'physical'});assert call('inspect')['scene']==actual
        adjusted=copy.deepcopy(assembly);adjusted['context_grafts'][-1]['options']['hub_radius']+=.001
        call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':adjusted});assert mesh(ID)!=parent
        call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':assembly});assert call('inspect')['scene']==actual
        # The new static patch is root-bound; check all new faces in animation.
        from tree_study_geometry import cross,dot,sub
        def normal(p,t):
            a,b,c=[p[v] for v in t];return cross(sub(b,a),sub(c,a))
        original_faces=faces(previous['geometry']);new_triangles=[t for t in parent['geometry']['triangles'] if tuple(tuple(parent['geometry']['positions'][v]) for v in t) not in original_faces]
        rest=call('inspect_mesh_pose',instance='tree-branch-editable-1',tick=0)['pose']['positions'];minimum_cosine=1.
        for tick in [30,60,90,120,150,180,210,240]:
            posed=call('inspect_mesh_pose',instance='tree-branch-editable-1',tick=tick)['pose']['positions']
            for t in new_triangles:
                a,b=normal(rest,t),normal(posed,t);den=math.sqrt(dot(a,a)*dot(b,b));assert den>0;minimum_cosine=min(minimum_cosine,dot(a,b)/den)
        assert minimum_cosine>0
        call('save_scene',path='examples/after.scene.json')
        # Remove just this graft and restore its original loose part as one edit.
        restore=[{'op':'mesh','edit':{'kind':'put_asset','asset':before_assets['wood']}},{'op':'plant','edit':{'kind':'set_assembly','id':ID,'assembly':old['assembly']}}]
        call('edit_scene',edits=restore);assert call('inspect')['scene']==initial;call('undo');assert call('inspect')['scene']==actual
        call('set_camera',camera={'position':[-3.5,4.4,3.5],'target':[-1.2,3.85,.5],'fov_degrees':21})
        call('save_scene',path='examples/after.scene.json');saved=call('inspect')['scene'];start,meta=capture('start')
        call('step',ticks=60);live=capture('live')[0];assert live!=start and capture('repeat')[0]==live
        call('save_replay',path='artifacts/tree.replay.json');assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
        for name,t,pixels in [('replay',60,live),('prefix',0,start)]:
            path=f'artifacts/{name}.png';call('capture_replay',replay_path='artifacts/tree.replay.json',path=path,tick=t,width=1100,height=1000);assert (R/path).read_bytes()==pixels
        call('load_scene',path='examples/after.scene.json');assert call('inspect')['scene']==saved and capture('loaded')[0]==start
        report=call('inspect_plant',id=ID)['report']['assembly']['context_grafts'][-1]
        def views(path,label):
            call('load_scene',path=path);scene=call('inspect')['scene']
            edits=[{'op':'mesh','edit':{'kind':'set_material','asset':id,'material':{}}} for id in ['wood',ID,'secondary-06','secondary-twigs']]
            for i in scene['meshes']['instances']:
                if i['asset'] in ['wood',ID,'secondary-06','secondary-twigs']:
                    i=copy.deepcopy(i);i['color']=[.65,.65,.65];edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':i}})
            call('edit_scene',edits=edits);call('set_light',light={'direction':[.7,-.3,-1],'ambient':.2})
            cams={'new':{'position':[-3.5,4.4,3.5],'target':[-1.2,3.85,.5],'fov_degrees':21},'side':{'position':[-3.8,4,-2.5],'target':[-1.2,3.85,.5],'fov_degrees':21},'approved':{'position':[2.1,3.4,3.5],'target':[.75,3.25,.36],'fov_degrees':19},'whole':{'position':[8,7,11],'target':[0,3.5,0],'fov_degrees':43}}
            for name,cam in cams.items():call('set_camera',camera=cam);capture(label+'-'+name)
        views('examples/input.scene.json','before');views('examples/after.scene.json','after')
        assert (R/'artifacts/before-approved.png').read_bytes()==(R/'artifacts/after-approved.png').read_bytes()
        call('load_scene',path='examples/after.scene.json');assert call('inspect')['scene']==saved
        result={'passed':True,'build_id':call('inspect')['build_id'],'new_static_graft':True,'report':report,'retained_old_parent_points':len(matching),'removed_old_parent_points':len(previous['geometry']['positions'])-len(matching),'approved_joint_points_exact':len(previous['geometry']['positions'])-first_approved,'retained_pose_errors_m':errors,'max_pose_error_m':max(errors.values()),'minimum_new_face_pose_cosine':minimum_cosine,'max_static_source_pose_error_m':static_error,'vertices':len(parent['geometry']['positions']),'triangles':len(parent['geometry']['triangles']),'draw_triangles':meta['mesh_triangle_count'],'instances':len(actual['meshes']['instances']),'approved_pixels_exact':True,'input_sha256':hashlib.sha256(args.input.read_bytes()).hexdigest()}
        (R/'artifacts/report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['report','retained_pose_errors_m']}))
    finally:c.close()
if __name__=='__main__':main()
