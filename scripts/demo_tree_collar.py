#!/usr/bin/env python3
"""Native collar finishing on the same connected tree, preserving canonical sources."""
import copy,hashlib,json,os,subprocess
from pathlib import Path
PROJECT=Path(__file__).resolve().parents[1];OUTPUT=PROJECT/'artifacts/tree-collar'
(OUTPUT/'examples').mkdir(parents=True,exist_ok=True);(OUTPUT/'artifacts').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT']=str(OUTPUT)
from verify_lab import Client
ID='branch-editable-1'
CAMS={'neck':{'position':[1.8,2.9,4.0],'target':[-.57,2.65,.30],'fov_degrees':19},
      'profile':{'position':[-3,3.2,-3.5],'target':[-.67,2.85,.25],'fov_degrees':20},
      'fork':{'position':[3.5,3.2,4.7],'target':[.55,2.95,.13],'fov_degrees':25}}
COLLAR={'ports':[0,1],'rings':6,'fixed_rings':2,'options':{'iterations':4,'strength':.3,'max_displacement':.01}}
def run():
 (OUTPUT/'examples/input.scene.json').write_bytes(subprocess.run(['git','show','b1058b5:examples/native-tree-study.scene.json'],cwd=PROJECT,check=True,stdout=subprocess.PIPE).stdout)
 c=Client(PROJECT/'target/release/ignis',False);call=c.call
 def views(scene,label):
  for name,cam in dict(CAMS,whole=scene['camera']).items():
   for neutral in [False,True]:
    call('load_scene',path=f'examples/{label}.scene.json')
    if neutral:
     edits=[{'op':'mesh','edit':{'kind':'remove_instance','id':i['id']}} for i in scene['meshes']['instances'] if i['id'].startswith('crown-')]
     edits.extend({'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':{}}} for a in scene['meshes']['assets'] if a['id'] in ['wood',ID,'secondary-06','secondary-twigs'])
     call('edit_scene',edits=edits);call('set_light',light={'direction':[-.8,-1,-.4],'ambient':.35})
    call('set_camera',camera=cam);call('step',ticks=60);call('capture',path=f'artifacts/{label}-'+('neutral-' if neutral else '')+name+'.png',width=1100,height=1100)
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-collar','version':'1'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='examples/input.scene.json');initial=call('inspect')['scene'];call('save_scene',path='examples/before.scene.json');old=call('inspect_plant',id=ID)['plant']
  poses={(i['id'],t):call('inspect_mesh_pose',instance=i['id'],tick=t)['pose'] for i in initial['meshes']['instances'] for t in [0,60,240]}
  recipe=copy.deepcopy(old['assembly']);recipe['context_grafts'][0]['collar_surface']=COLLAR
  edit={'kind':'set_assembly','id':ID,'assembly':recipe};before=call('inspect');pred=call('preview_plant',edit=edit);assert call('inspect')==before
  call('edit_plant',edit=edit);actual=call('inspect')['scene'];after=call('inspect_plant',id=ID)
  aa={a['id']:a for a in actual['meshes']['assets']};ba={a['id']:a for a in initial['meshes']['assets']};assert aa[ID]==next(a for a in pred['assets'] if a['id']==ID)
  copy_recipe=copy.deepcopy(after['plant']['assembly']);copy_recipe['context_grafts'][0].pop('collar_surface');assert copy_recipe==old['assembly']
  for key in ['definition','reference','source']:assert after['plant'][key]==old[key]
  for id,a in aa.items():
   if id!=ID:assert a==ba[id]
  for key in ['rig','clips','shading','material']:assert aa[ID][key]==ba[ID][key]
  for key in ['triangles','material_coordinates']:assert aa[ID]['geometry'][key]==ba[ID]['geometry'][key]
  assert actual['meshes']['instances']==initial['meshes']['instances']
  collar=after['report']['assembly']['context_grafts'][0]['collar_surface'];moved=sum(p!=q for p,q in zip(aa[ID]['geometry']['positions'],ba[ID]['geometry']['positions']));assert moved==collar['filter']['moved_vertices']
  assert collar['moved_retained_source']==0 and collar['moved_retained_context']>0 and collar['moved_generated']>0
  for (id,t),pose in poses.items():
   current_pose=call('inspect_mesh_pose',instance=id,tick=t)['pose']
   if next(i['asset'] for i in initial['meshes']['instances'] if i['id']==id)!=ID:assert current_pose==pose
   else:
    for key in ['joints','world_transform','clip_tick']:assert current_pose[key]==pose[key]
  call('undo');assert call('inspect')['scene']==initial;call('redo');assert call('inspect')['scene']==actual
  for _ in range(2):
   d=copy.deepcopy(old['definition']);d['trajectory']['control_points_xz'][3][0]+=.001
   call('edit_plant',edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'})
   call('edit_plant',edit={'kind':'put','definition':old['definition'],'detail_policy':'physical','animation_translation_policy':'physical'});assert call('inspect')['scene']==actual
  changed=copy.deepcopy(recipe);changed['context_lofts'][0]['spans']=4
  call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':changed});call('edit_plant',edit=edit);assert call('inspect')['scene']==actual
  call('save_scene',path='examples/after.scene.json');views(initial,'before');views(actual,'after')
  for name in ['neck','profile']:assert (OUTPUT/f'artifacts/before-neutral-{name}.png').read_bytes()!=(OUTPUT/f'artifacts/after-neutral-{name}.png').read_bytes()
  call('load_scene',path='examples/after.scene.json');assert call('inspect')['scene']==actual;call('step',ticks=60)
  call('capture',path='artifacts/live.png',width=1100,height=1100);call('capture',path='artifacts/repeat.png',width=1100,height=1100);assert (OUTPUT/'artifacts/live.png').read_bytes()==(OUTPUT/'artifacts/repeat.png').read_bytes()
  call('save_replay',path='artifacts/tree.replay.json');assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
  call('capture_replay',replay_path='artifacts/tree.replay.json',path='artifacts/replay.png',tick=60,width=1100,height=1100);assert (OUTPUT/'artifacts/live.png').read_bytes()==(OUTPUT/'artifacts/replay.png').read_bytes()
  data=(OUTPUT/'examples/after.scene.json').read_bytes();report={'passed':True,'build_id':call('inspect')['build_id'],'source_sha256':hashlib.sha256(data).hexdigest(),'source_bytes':len(data),'source_reference_context_child_exact':True,
   'topology_domains_rig_clips_instances_other_assets_poses_exact':True,'collar':collar,'moved_final_vertices':moved,'vertices':sum(len(a['geometry']['positions']) for a in aa.values()),'triangles':sum(len(a['geometry']['triangles']) for a in aa.values()),
   'draw_triangles':sum(len(aa[i['asset']]['geometry']['triangles']) for i in actual['meshes']['instances']),'all_junctions_connected':False}
  (OUTPUT/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
 finally:c.close()
if __name__=='__main__':run()
