#!/usr/bin/env python3
"""Regenerate the same tree's three-port junction from canonical sources."""
import argparse,copy,hashlib,json,math,os,subprocess
from pathlib import Path
PROJECT=Path(__file__).resolve().parents[1]
OUTPUT=PROJECT/'artifacts/tree-assembly'
(OUTPUT/'examples').mkdir(parents=True,exist_ok=True);(OUTPUT/'artifacts').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT']=str(OUTPUT)
from verify_lab import Client
ID='branch-editable-1'
CAMS={'branch':{'position':[3.5,3.2,4.7],'target':[.55,2.95,.13],'fov_degrees':30},'profile':{'position':[4,3.5,-1.4],'target':[.4,2.95,.12],'fov_degrees':30},'back':{'position':[-3.5,3.5,-4.7],'target':[.35,2.95,.13],'fov_degrees':30}}
def run(frames=0):
 if frames:
  from PIL import Image
 snapshot=OUTPUT/'examples/input.scene.json'
 snapshot.write_bytes(subprocess.run(['git','show','1d36a65:examples/native-tree-study.scene.json'],cwd=PROJECT,check=True,stdout=subprocess.PIPE).stdout)
 c=Client(PROJECT/'target/release/ignis',False);call=c.call
 def capture(name,cam):
  call('set_camera',camera=cam);call('step',ticks=60);assert call('inspect')['tick']==60
  path=f'artifacts/{name}.png';meta=call('capture',path=path,width=1200,height=1100)
  return {'sha256':hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest(),'draw_triangles':meta['mesh_triangle_count']}
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-assembly','version':'1'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='examples/input.scene.json')
  highres=os.environ.get('IGNIS_TREE_HIGH_RES')=='1'
  if highres:
   d=copy.deepcopy(call('inspect_plant',id=ID)['plant']['definition']);d['segments']=[8,8,8]
   call('edit_plant',edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'})
  initial=call('inspect')['scene'];canonical=call('inspect_plant',id=ID)['plant']
  assets={a['id']:a for a in initial['meshes']['assets']};instance=next(i for i in initial['meshes']['instances'] if i['asset']==ID)
  wood=next(i for i in initial['meshes']['instances'] if i['asset']=='wood');assert wood['position']==[0,0,0] and wood['rotation']==[0,0,0,1] and wood['scale']==1
  assert instance['rotation']==[0,0,0,1] and instance['scale']==1
  poses={(i['id'],tick):call('inspect_mesh_pose',instance=i['id'],tick=tick)['pose'] for i in initial['meshes']['instances'] for tick in [0,60]}
  cameras=dict(CAMS,whole=initial['camera']);views={name:{'before':capture('before-'+name,cam)} for name,cam in cameras.items()}
  info=call('inspect_mesh_islands',asset='wood');assert info['topology']['islands'][0]['vertices']==2127
  extract={'kind':'extract_islands','asset':'wood','reference':info['reference'],'islands':[0],'id':'central-context','mode':'move'}
  state=call('inspect');preview=call('preview_mesh_edit',edit=extract);assert call('inspect')==state
  context=next(a for a in preview['meshes']['assets'] if a['id']=='central-context');world_context=copy.deepcopy(context)
  for p in context['geometry']['positions']:
   for k in range(3):p[k]-=instance['position'][k]
  g=context['geometry'];tile=set(range(943,1103));faces=[i for i,t in enumerate(g['triangles']) if set(t)<=tile];assert len(faces)==256
  options=json.loads((PROJECT/'examples/mesh-junction-union.input.json').read_text())['options']
  if highres:options['cell_size']=.0375
  options.update(seam_relaxation_steps=4,seam_max_displacement=.0125 if highres else .025,material_relaxation_steps=64)
  assembly={'context':context,'context_faces':faces,'context_loops':[list(range(943,975)),list(range(1071,1103))],'station':'branch1-030','context_bone':'base','options':options}
  edit={'kind':'set_assembly','id':ID,'assembly':assembly}
  batch=[{'op':'mesh','edit':extract},{'op':'mesh','edit':{'kind':'remove_asset','id':'central-context'}},{'op':'plant','edit':edit}]
  before=call('inspect');predicted_scene=call('preview_scene_edits',edits=batch)['scene'];assert call('inspect')==before
  predicted=next(a for a in predicted_scene['meshes']['assets'] if a['id']==ID)
  call('edit_scene',edits=batch);composed=call('inspect')['scene'];assert composed==predicted_scene;actual=call('inspect_plant',id=ID);out=next(a for a in composed['meshes']['assets'] if a['id']==ID)
  assert out==predicted and actual['plant']['source']==canonical['source'] and actual['plant']['reference']==canonical['reference']
  assert composed['meshes']['instances']==initial['meshes']['instances']
  for a in composed['meshes']['assets']:
   if a['id'] not in ['wood',ID]:assert a==assets[a['id']]
  assert next(a for a in composed['meshes']['assets'] if a['id']=='wood')==next(a for a in preview['meshes']['assets'] if a['id']=='wood')
  call('undo');assert call('inspect')['scene']==before['scene'];call('redo');assert call('inspect')['scene']==composed
  assert out['rig']['bones']==assets[ID]['rig']['bones'] and out['clips']==assets[ID]['clips']
  pose_error=0.
  for (name,tick),old in poses.items():
   if name in [instance['id'],wood['id']]:continue
   new=call('inspect_mesh_pose',instance=name,tick=tick)['pose'];assert new==old,(name,tick)
  kept_context=sorted({i for f,t in enumerate(g['triangles']) if f not in set(faces) for i in t})
  for tick in [0,60]:
   new=call('inspect_mesh_pose',instance=instance['id'],tick=tick)['pose']
   error=max(math.dist(p,world_context['geometry']['positions'][i]) for p,i in zip(new['positions'],kept_context));assert error<1e-6,error;pose_error=max(pose_error,error)
  topo=call('inspect_mesh_topology',asset=ID)['topology'];assert topo['boundary_edges']==topo['non_manifold_edges']==0 and not topo['isolated_vertices']
  assert len(call('inspect_mesh_islands',asset=ID)['topology']['islands'])==1
  for _ in range(2):
   d=copy.deepcopy(canonical['definition']);d['trajectory']['control_points_xz'][3][0]+=.001
   call('edit_plant',edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'})
   call('edit_plant',edit={'kind':'put','definition':canonical['definition'],'detail_policy':'physical','animation_translation_policy':'physical'})
   assert call('inspect_mesh',asset=ID)['asset']==out and call('inspect_plant',id=ID)['plant']['source']==canonical['source']
  # Invalid source/section/options and geometry rebases must leave everything intact.
  bad=copy.deepcopy(assembly);bad['station']='branch1-044-missing';before=call('inspect');call('edit_plant',edit=dict(edit,assembly=bad),expected_error=True);assert call('inspect')==before
  call('set_camera',camera=initial['camera']);call('save_scene',path='examples/candidate.scene.json');saved=call('inspect')['scene'];assert saved['version']==69
  for name,cam in cameras.items():views[name]['after']=capture('after-'+name,cam)
  call('load_scene',path='examples/candidate.scene.json')
  for name,cam in CAMS.items():
   crown=[i for i in saved['meshes']['instances'] if i['id'].startswith('crown-')]
   call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'remove_instance','id':i['id']}} for i in crown])
   capture('structure-'+name,cam);call('load_scene',path='examples/candidate.scene.json')
   call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'remove_instance','id':i['id']}} for i in crown]+[{'op':'mesh','edit':{'kind':'set_material','asset':ID,'material':{}}}])
   call('set_light',light={'direction':[-.8,-1,-.4],'ambient':.5})
   capture('neutral-'+name,cam);call('load_scene',path='examples/candidate.scene.json')
  call('capture',path='artifacts/start.png',width=1200,height=1100);call('step',ticks=60);call('capture',path='artifacts/live.png',width=1200,height=1100)
  call('save_replay',path='artifacts/tree.replay.json');assert json.loads((OUTPUT/'artifacts/tree.replay.json').read_text())['version']==73;assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
  for name,tick,source in [('replay',60,'live'),('prefix',0,'start')]:
   call('capture_replay',replay_path='artifacts/tree.replay.json',path=f'artifacts/{name}.png',tick=tick,width=1200,height=1100);assert (OUTPUT/f'artifacts/{name}.png').read_bytes()==(OUTPUT/f'artifacts/{source}.png').read_bytes()
  call('load_scene',path='examples/candidate.scene.json');assert call('inspect')['scene']==saved
  hashes=[]
  if frames:
   folder=OUTPUT/'artifacts/frames';folder.mkdir(exist_ok=True)
   images=[];loop_poses={i['id']:call('inspect_mesh_pose',instance=i['id'])['pose'] for i in saved['meshes']['instances']}
   for n in range(frames):
    path=f'artifacts/frames/frame-{n:03}.png';call('capture',path=path,width=840,height=770)
    hashes.append(hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest())
    with Image.open(OUTPUT/path) as png:images.append(png.convert('RGB').quantize(colors=192))
    call('step',ticks=5)
   durations=[80 if n%3!=2 else 90 for n in range(frames)]
   images[0].save(OUTPUT/'artifacts/tree.gif',save_all=True,append_images=images[1:],duration=durations,loop=0,disposal=2,optimize=False)
   assert len(set(hashes))==48 and sum(durations)==4000
   with Image.open(OUTPUT/'artifacts/tree.gif') as gif:
    assert gif.n_frames==frames and sum((gif.seek(n),gif.info['duration'])[1] for n in range(frames))==4000
   for name,pose in loop_poses.items():assert call('inspect_mesh_pose',instance=name)['pose']==pose
  sizes={a['id']:len(a['geometry']['triangles']) for a in saved['meshes']['assets']}
  report={'passed':True,'build_id':call('inspect')['build_id'],'report':actual['report'],'views':views,'source_reference_exact':True,'other_assets_instances_poses_0_60_exact':True,'context_world_error_m':pose_error,'edit_cycles_undo_save_load_replay_prefix_exact':True,'draw_triangles':sum(sizes[i['asset']] for i in saved['meshes']['instances']),'vertices':len(out['geometry']['positions']),'triangles':len(out['geometry']['triangles']),'whole_tree_converted':False,'animation_frames':frames,'unique_animation_frames':len(set(hashes))}
  (OUTPUT/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
 finally:c.close()
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--frames',type=int,choices=[0,48],default=0);a=p.parse_args();run(a.frames)
