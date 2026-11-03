#!/usr/bin/env python3
"""Regenerable native side-branch attachment on the maintained tree; no baked context."""
import copy, hashlib, json, os, subprocess
from pathlib import Path
PROJECT=Path(__file__).resolve().parents[1]
OUTPUT=PROJECT/'artifacts/tree-graft'
(OUTPUT/'examples').mkdir(parents=True,exist_ok=True);(OUTPUT/'artifacts').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT']=str(OUTPUT)
from verify_lab import Client
ID='branch-editable-1'
CAMS={'neck':{'position':[1.8,2.9,4.0],'target':[-.57,2.65,.30],'fov_degrees':19},
      'profile':{'position':[-3,3.2,-3.5],'target':[-.67,2.85,.25],'fov_degrees':20},
      'fork':{'position':[3.5,3.2,4.7],'target':[.55,2.95,.13],'fov_degrees':25}}

def prepare(call, initial):
    """Native loose-part extraction provides exact source channels and remaining wood."""
    inspected=call('inspect_mesh_islands',asset='wood')
    g=next(a['geometry'] for a in initial['meshes']['assets'] if a['id']=='wood')
    # The minimum face of the component incident on canonical vertex0 is an
    # explicit authored identity, checked by the native island partition.
    island=min(i for i,t in enumerate(g['triangles']) if 0 in t)
    assert next(p for p in inspected['topology']['islands'] if p['id']==island)['vertices']==119
    extract={'kind':'extract_islands','asset':'wood','id':'secondary-00-source','reference':inspected['reference'],'islands':[island],'mode':'move'}
    before=call('inspect');parts=call('preview_mesh_edit',edit=extract)['meshes']['assets'];assert call('inspect')==before
    source=next(a for a in parts if a['id']=='secondary-00-source');remaining=next(a for a in parts if a['id']=='wood')
    assert len(source['geometry']['positions'])==119
    root=set(range(7))|set(range(35,56))
    faces=[i for i,t in enumerate(source['geometry']['triangles']) if any(v in root for v in t)]
    plant=call('inspect_plant',id=ID)['plant'];recipe=copy.deepcopy(plant['assembly'])
    rings=recipe['context_lofts'][0]['rings'];lo=next(i for i,r in enumerate(rings) if 1903 in r);hi=next(i for i,r in enumerate(rings) if 1999 in r)
    offset=next(i['position'] for i in initial['meshes']['instances'] if i['asset']==ID)
    assert next(i for i in initial['meshes']['instances'] if i['asset']==ID)['rotation']==[0,0,0,1]
    assert next(i for i in initial['meshes']['instances'] if i['asset']==ID)['scale']==1
    recipe['context_grafts']=[{'source':source,'source_translation':[-v for v in offset],'source_faces':faces,'source_loop':list(range(7,14)),
        'context_vertices':[v for r in rings[lo:hi+1] for v in r], 'context_loops':[rings[lo],rings[hi]],'surface':{'iterations':16,'strength':.3,'max_displacement':.02},
        'options':{'hub':[-.80,-.69,.18],'hub_radius':.145,'blend':.035,'cell_size':.032,'guard_length':.008,'collar_length':.03,
                   'relaxation_steps':3,'material_offset':offset,'seam_relaxation_steps':4,'seam_max_displacement':.008,'material_relaxation_steps':64}}]
    return plant, recipe, [{'op':'mesh','edit':{'kind':'put_asset','asset':remaining}}, {'op':'plant','edit':{'kind':'set_assembly','id':ID,'assembly':recipe}}]

def run():
    (OUTPUT/'examples/input.scene.json').write_bytes(subprocess.run(['git','show','7ee07c6:examples/native-tree-study.scene.json'],cwd=PROJECT,check=True,stdout=subprocess.PIPE).stdout)
    c=Client(PROJECT/'target/release/ignis',False);call=c.call
    def views(scene,label):
        for name,cam in dict(CAMS,whole=scene['camera']).items():
            for neutral in [False,True]:
                call('load_scene',path=f'examples/{label}.scene.json')
                if neutral:
                    edits=[{'op':'mesh','edit':{'kind':'remove_instance','id':i['id']}} for i in scene['meshes']['instances'] if i['id'].startswith('crown-')]
                    edits.extend({'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':{}}} for a in scene['meshes']['assets'] if a['id'] in ['wood',ID,'secondary-06','secondary-twigs'])
                    call('edit_scene',edits=edits);call('set_light',light={'direction':[-.8,-1,-.4],'ambient':.35})
                call('set_camera',camera=cam);call('step',ticks=60)
                call('capture',path=f'artifacts/{label}-'+('neutral-' if neutral else '')+name+'.png',width=1100,height=1100)
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-graft','version':'1'}});c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json');initial=call('inspect')['scene'];call('save_scene',path='examples/before.scene.json')
        poses={(i['id'],t):call('inspect_mesh_pose',instance=i['id'],tick=t)['pose'] for i in initial['meshes']['instances'] for t in [0,60,240]}
        old,recipe,batch=prepare(call,initial)
        state=call('inspect');predicted=call('preview_scene_edits',edits=batch)['scene'];assert call('inspect')==state
        call('edit_scene',edits=batch);actual=call('inspect')['scene'];assert actual==predicted
        current=call('inspect_plant',id=ID)
        for key in ['source','reference','definition']:assert current['plant'][key]==old[key]
        assert current['plant']['assembly']['context']==old['assembly']['context']
        assert current['plant']['assembly']['context_grafts'][0]['source']==recipe['context_grafts'][0]['source']
        before_assets={a['id']:a for a in initial['meshes']['assets']};after_assets={a['id']:a for a in actual['meshes']['assets']}
        assert actual['meshes']['instances']==initial['meshes']['instances']
        assert after_assets[ID]['rig']['bones']==before_assets[ID]['rig']['bones'] and after_assets[ID]['clips']==before_assets[ID]['clips']
        assert len(call('inspect_mesh_islands',asset='wood')['topology']['islands'])==176
        assert call('inspect_mesh_boundaries',asset=ID)['boundaries']==[]
        assert len(call('inspect_mesh_islands',asset=ID)['topology']['islands'])==1
        for id,a in after_assets.items():
            if id not in ['wood',ID]:assert a==before_assets[id]
        for (id,t),pose in poses.items():
            if next(i['asset'] for i in initial['meshes']['instances'] if i['id']==id) not in ['wood',ID]:assert call('inspect_mesh_pose',instance=id,tick=t)['pose']==pose
        call('undo');assert call('inspect')['scene']==initial;call('redo');assert call('inspect')['scene']==actual
        for _ in range(2):
            d=copy.deepcopy(old['definition']);d['trajectory']['control_points_xz'][3][0]+=.001
            call('edit_plant',edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'})
            call('edit_plant',edit={'kind':'put','definition':old['definition'],'detail_policy':'physical','animation_translation_policy':'physical'});assert call('inspect')['scene']==actual
        call('save_scene',path='examples/after.scene.json');views(initial,'before');views(actual,'after')
        call('load_scene',path='examples/after.scene.json');call('step',ticks=60);call('save_replay',path='artifacts/tree.replay.json');assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
        data=(OUTPUT/'examples/after.scene.json').read_bytes()
        report={'passed':True,'build_id':call('inspect')['build_id'],'source_sha256':hashlib.sha256(data).hexdigest(),'source_bytes':len(data),'source_reference_context_exact':True,
                'bones_clips_other_assets_instances_poses_exact':True,'closed_connected_graft':True,'wood_islands_before':177,'wood_islands_after':176,'all_junctions_connected':False,
                'assembly':current['report']['assembly'],'vertices':sum(len(a['geometry']['positions']) for a in actual['meshes']['assets']),
                'triangles':sum(len(a['geometry']['triangles']) for a in actual['meshes']['assets']),
                'draw_triangles':sum(len(after_assets[i['asset']]['geometry']['triangles']) for i in actual['meshes']['instances'])}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='assembly'}))
    finally:c.close()
if __name__=='__main__':run()
