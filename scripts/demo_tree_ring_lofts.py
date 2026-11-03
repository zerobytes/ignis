#!/usr/bin/env python3
"""Native, source-preserving section interpolation on the same authored tree."""
import copy,hashlib,json,os,subprocess
from pathlib import Path
PROJECT=Path(__file__).resolve().parents[1]
OUTPUT=PROJECT/'artifacts/tree-surface'
(OUTPUT/'artifacts').mkdir(parents=True,exist_ok=True);(OUTPUT/'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT']=str(OUTPUT)
from verify_lab import Client
ID='branch-editable-1'
CAMS={'neck':{'position':[1.8,2.9,4.0],'target':[-.57,2.65,.30],'fov_degrees':19},
      'fork':{'position':[3.5,3.2,4.7],'target':[.55,2.95,.13],'fov_degrees':25},
      'back':{'position':[-3.5,3.5,-4.7],'target':[.35,2.95,.13],'fov_degrees':30}}
def align_rings(geometry,groups):
 """Recover column phase from exact source topology, never by distance fitting."""
 faces={tuple(sorted(t)) for t in geometry['triangles']};rings=[groups[0]]
 for group in groups[1:]:
  a=rings[-1];n=len(a);found=None
  for seq in [group,group[::-1]]:
   for off in range(n):
    b=seq[off:]+seq[:off]
    if all((tuple(sorted([a[j],a[(j+1)%n],b[j]])) in faces and tuple(sorted([a[(j+1)%n],b[(j+1)%n],b[j]])) in faces) or (tuple(sorted([a[j],a[(j+1)%n],b[(j+1)%n]])) in faces and tuple(sorted([a[j],b[(j+1)%n],b[j]])) in faces) for j in range(n)):
     found=b;break
   if found:break
  if found is None:raise ValueError(f'No exact source cell phase for ring {group[0]}')
  rings.append(found)
 return rings

def tube_lofts(g):
 adjacent=[set() for _ in g['positions']]
 for t in g['triangles']:
  for i in t:adjacent[i].update(t)
 seen=set();parts=[]
 for i in range(len(adjacent)):
  if i in seen:continue
  seen.add(i);q=[i];part=[]
  while q:
   k=q.pop();part.append(k)
   for x in adjacent[k]-seen:seen.add(x);q.append(x)
  parts.append(sorted(part))
 lofts=[]
 for part in parts:
  # Author's two explicit tube layouts, independently verified against cells.
  sides=7 if len(part)==35 else 12 if len(part)==120 else None
  if sides:lofts.append({'rings':align_rings(g,[part[j:j+sides] for j in range(0,len(part),sides)]),'spans':4,'interpolation':'cubic','max_deviation':.3})
 return lofts

def run():
 snapshot=OUTPUT/'examples/input.scene.json'
 snapshot.write_bytes(subprocess.run(['git','show','2ba75eb:examples/native-tree-study.scene.json'],cwd=PROJECT,check=True,stdout=subprocess.PIPE).stdout)
 c=Client(PROJECT/'target/release/ignis',False);call=c.call
 def capture(name,cam):
  call('set_camera',camera=cam);call('step',ticks=60);assert call('inspect')['tick']==60
  path=f'artifacts/{name}.png';meta=call('capture',path=path,width=1100,height=1100)
  return {'sha256':hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest(),'triangles':meta['mesh_triangle_count']}
 def views(scene,label):
  for name,cam in dict(CAMS,whole=scene['camera']).items():
   call('load_scene',path=f'examples/{label}.scene.json');capture(f'{label}-{name}',cam)
   call('load_scene',path=f'examples/{label}.scene.json')
   crown=[i for i in scene['meshes']['instances'] if i['id'].startswith('crown-')]
   edits=[{'op':'mesh','edit':{'kind':'remove_instance','id':i['id']}} for i in crown]
   edits.extend({'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':{}}} for a in scene['meshes']['assets'] if a['id'] in ['wood',ID,'secondary-06','secondary-twigs'])
   call('edit_scene',edits=edits);call('set_light',light={'direction':[-.8,-1,-.4],'ambient':.35});capture(f'{label}-neutral-{name}',cam)
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-surface','version':'1'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='examples/input.scene.json');initial=call('inspect')['scene'];call('save_scene',path='examples/before.scene.json')
  assets={a['id']:a for a in initial['meshes']['assets']};canonical=call('inspect_plant',id=ID)['plant']
  poses={(i['id'],t):call('inspect_mesh_pose',instance=i['id'],tick=t)['pose'] for i in initial['meshes']['instances'] for t in [0,60]}
  batch=[];reports={}
  for id in ['wood','secondary-06']:
   source=assets[id];lofts=tube_lofts(source['geometry']);before=call('inspect');result=call('preview_mesh_ring_loft',source=source,lofts=lofts);assert call('inspect')==before
   out=result['asset'];n=len(source['geometry']['positions']);assert out['geometry']['positions'][:n]==source['geometry']['positions'];assert out['geometry']['material_coordinates'][:n]==source['geometry']['material_coordinates']
   assert out.get('rig')==source.get('rig') and out.get('clips')==source.get('clips')
   batch.append({'op':'mesh','edit':{'kind':'put_asset','asset':out}});reports[id]=result['report']
  recipe=copy.deepcopy(canonical['assembly']);starts=[32,1487,1519,1551,1583,1615,*range(1839,2127,32),*range(1647,1839,32)]
  rings=align_rings(recipe['context']['geometry'],[list(range(x,x+32)) for x in starts])
  recipe['context_lofts']=[{'rings':rings,'spans':3,'interpolation':'cubic','max_deviation':.2}]
  if os.environ.get('IGNIS_TREE_CONTEXT_FINISH','1')!='0':
   recipe['context_surface']={'vertices':[v for r in rings for v in r],'anchors':rings[0]+rings[-2]+rings[-1],'options':{'iterations':12,'strength':.3,'max_displacement':.03}}
   recipe['junction_surface']={'iterations':4,'strength':.1,'max_displacement':.01}
  batch.append({'op':'plant','edit':{'kind':'set_assembly','id':ID,'assembly':recipe}})
  state=call('inspect');predicted=call('preview_scene_edits',edits=batch)['scene'];assert call('inspect')==state
  call('edit_scene',edits=batch);actual=call('inspect')['scene'];assert actual==predicted
  after=call('inspect_plant',id=ID);assert after['plant']['source']==canonical['source'] and after['plant']['reference']==canonical['reference'] and after['plant']['assembly']['context']==canonical['assembly']['context']
  assert actual['meshes']['instances']==initial['meshes']['instances']
  for a in actual['meshes']['assets']:
   if a['id'] not in ['wood','secondary-06',ID]:assert a==assets[a['id']]
  for (name,tick),pose in poses.items():
   if next(i['asset'] for i in initial['meshes']['instances'] if i['id']==name) not in ['wood','secondary-06',ID]:assert call('inspect_mesh_pose',instance=name,tick=tick)['pose']==pose,(name,tick)
  call('undo');assert call('inspect')['scene']==initial;call('redo');assert call('inspect')['scene']==actual
  for _ in range(2):
   d=copy.deepcopy(canonical['definition']);d['trajectory']['control_points_xz'][3][0]+=.001
   call('edit_plant',edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'})
   call('edit_plant',edit={'kind':'put','definition':canonical['definition'],'detail_policy':'physical','animation_translation_policy':'physical'});assert call('inspect')['scene']==actual
  call('save_scene',path='examples/after.scene.json');views(initial,'before');views(actual,'after')
  call('load_scene',path='examples/after.scene.json');call('step',ticks=60);call('save_replay',path='artifacts/tree.replay.json');assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
  report={'passed':True,'build_id':call('inspect')['build_id'],'reports':reports,'assembly':after['report']['assembly'],'source_reference_context_exact':True,'other_assets_instances_poses_exact':True,'edit_cycles_undo_save_load_replay':True,'vertices':sum(len(a['geometry']['positions']) for a in actual['meshes']['assets']),'triangles':sum(len(a['geometry']['triangles']) for a in actual['meshes']['assets']),'draw_triangles':sum(len(next(a for a in actual['meshes']['assets'] if a['id']==i['asset'])['geometry']['triangles']) for i in actual['meshes']['instances']),'all_junctions_connected':False}
  (OUTPUT/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k not in ['reports','assembly']}))
 finally:c.close()
if __name__=='__main__':run()
