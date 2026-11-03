#!/usr/bin/env python3
"""Connect the two native tree plants without detaching either source."""
import argparse,copy,json,math,os
from pathlib import Path
P=Path(__file__).resolve().parents[1]
R=P/'artifacts/plant-connections'

def main():
 p=argparse.ArgumentParser();p.add_argument('--input',type=Path,default=P/'artifacts/tree-secondary-plant/examples/after.scene.json');p.add_argument('--recipe',type=Path,default=P/'examples/native-tree.connection.json');p.add_argument('--compare-recipe',type=Path);a=p.parse_args()
 for d in ['examples','artifacts']:(R/d).mkdir(parents=True,exist_ok=True)
 (R/'examples/input.scene.json').write_bytes(a.input.read_bytes())
 os.environ['IGNIS_VERIFY_ROOT']=str(R)
 from verify_lab import Client
 c=Client(P/'target/release/ignis');call=c.call
 def asset(id):return call('inspect_mesh',asset=id)['asset']
 def entry(id):return call('inspect_plant',id=id)['plant']
 def pose(id,t):return call('inspect_mesh_pose',instance=id,tick=t)['pose']
 def cap(name):
  path=f'artifacts/{name}.png';meta=call('capture',path=path,width=1100,height=1000);return (R/path).read_bytes(),meta
 def put(d):return {'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'}
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-plant-connections','version':'1'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='examples/input.scene.json');call('set_camera',camera={'position':[3,3.4,3.3],'target':[.75,3.25,.4],'fov_degrees':25})
  before=call('inspect')['scene'];old=asset('branch-editable-1');child=asset('secondary-06');entries=before['plants']['entries'];assert len(entries)==2
  poses={(i['id'],t):pose(i['id'],t) for i in before['meshes']['instances'] for t in [0,60,240]}
  cap('before');recipe=json.loads(a.recipe.read_text());edit={'kind':'put_connection','connection':recipe}
  state=call('inspect');pred=call('preview_scene_edits',edits=[{'op':'plant','edit':edit}])['scene'];assert call('inspect')==state
  call('edit_plant',edit=edit);after=call('inspect')['scene'];assert after==pred and after['plants']['entries']==entries and after['meshes']['instances']==before['meshes']['instances']
  out=asset('branch-editable-1');report=call('inspect_plant_connections')['reports'][0]
  n=report['retained_parent_vertices'];nc=report['retained_child_vertices']
  # Source compaction keeps original order. Match exact retained triples/weights,
  # not a nearest-point projection, then compare their world-space animation.
  pids=[];cursor=0
  for i,pnt in enumerate(out['geometry']['positions'][:n]):
   while old['geometry']['positions'][cursor]!=pnt:cursor+=1
   assert out['rig']['weights'][i]==old['rig']['weights'][cursor];pids.append(cursor);cursor+=1
  assert out['rig']['bones']==old['rig']['bones'] and out['clips']==old['clips'] and asset('secondary-06')==child
  sample=call('inspect_plant_sampling',id='secondary-06')['sampling'];m=sample['material'];tris=[t for t in sample['geometry']['triangles'] if all(m[v][1]>=-.25-1e-6 for v in t)];cids=sorted({v for t in tris for v in t});assert len(cids)==nc
  errors={}
  for (id,t),baseline in poses.items():
   current=pose(id,t)
   if id=='tree-branch-editable-1':
    error=max(math.dist(current['positions'][i],baseline['positions'][j]) for i,j in enumerate(pids))
    cp=poses['tree-secondary-06',t]['positions'];error=max(error,max(math.dist(current['positions'][n+i],cp[j]) for i,j in enumerate(cids)))
    assert error<2e-6
   else:assert current==baseline;error=0.
   errors[f'{id}:{t}']=error
  # Check the newly generated faces through eight native animated poses.
  from tree_study_geometry import cross,dot,sub
  def normal(points,tri):
   a,b,c=[points[i] for i in tri];return cross(sub(b,a),sub(c,a))
  rest=pose('tree-branch-editable-1',0)['positions'];first_new=len(out['geometry']['triangles'])-report['junction']['added_triangles'];minimum_cosine=1.
  for tick in [30,60,90,120,150,180,210,240]:
   points=pose('tree-branch-editable-1',tick)['positions']
   for tri in out['geometry']['triangles'][first_new:]:
    n,q=normal(rest,tri),normal(points,tri);denominator=math.sqrt(dot(n,n)*dot(q,q));assert denominator>0
    minimum_cosine=min(minimum_cosine,dot(n,q)/denominator)
  assert minimum_cosine>0
  for id,delta in [('branch-editable-1',.001),('secondary-06',.004)]:
   e=entry(id);d=copy.deepcopy(e['definition']);d['trajectory']['control_points_xz'][1][0]+=delta
   call('edit_plant',edit=put(d));assert asset('branch-editable-1')!=out and entry(id)['source']==e['source'];cap('edited-'+id)
   call('edit_plant',edit=put(e['definition']));assert call('inspect')['scene']==after
  topo=call('inspect_mesh_topology',asset='branch-editable-1')['topology'];assert topo['boundary_edges']==topo['non_manifold_edges']==0 and len(call('inspect_mesh_islands',asset='branch-editable-1')['topology']['islands'])==1
  call('save_scene',path='examples/connected.scene.json');start,meta=cap('connected')
  call('edit_plant',edit={'kind':'remove_connection','id':recipe['id']});assert call('inspect')['scene']==before;call('undo');assert call('inspect')['scene']==after and cap('undo')[0]==start
  call('step',ticks=60);live=cap('live')[0];assert live!=start and cap('repeat')[0]==live
  call('save_replay',path='artifacts/tree.replay.json');assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
  for name,t,pixels in [('replay',60,live),('prefix',0,start)]:
   path=f'artifacts/{name}.png';call('capture_replay',replay_path='artifacts/tree.replay.json',path=path,tick=t,width=1100,height=1000);assert (R/path).read_bytes()==pixels
  call('load_scene',path='examples/connected.scene.json');assert call('inspect')['scene']==after and cap('loaded')[0]==start
  result={'passed':True,'build_id':call('inspect')['build_id'],'recipe_bytes':len(json.dumps(edit).encode()),'report':report,'pose_errors_m':errors,'max_pose_error_m':max(errors.values()),'draw_triangles':meta['mesh_triangle_count'],'two_live_plants':True,'minimum_new_face_pose_cosine':minimum_cosine,'recipe':recipe,'pixel_checks':['undo','repeat','replay','prefix','load']}
  # Diagnostic clay views expose geometry, separate from bark appearance.
  views=[('before-clay','examples/input.scene.json',None),('connected-clay','examples/connected.scene.json',None)]
  if a.compare_recipe:views.append(('previous-clay','examples/input.scene.json',json.loads(a.compare_recipe.read_text())))
  for name,path,comparison in views:
   call('load_scene',path=path)
   if comparison:call('edit_plant',edit={'kind':'put_connection','connection':comparison})
   call('set_camera',camera={'position':[2.1,3.4,3.5],'target':[.75,3.25,.36],'fov_degrees':19})
   call('set_light',light={'direction':[-.7,-.3,-1],'ambient':.18})
   edits=[{'op':'mesh','edit':{'kind':'set_material','asset':id,'material':{}}} for id in ['wood','branch-editable-1','secondary-06']]
   for i in call('inspect')['scene']['meshes']['instances']:
    if i['asset'] in ['wood','branch-editable-1','secondary-06']:
     i=copy.deepcopy(i);i['color']=[.65,.65,.65];edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':i}})
   call('edit_scene',edits=edits);cap(name)
   call('set_camera',camera={'position':[3.2,3.6,-.8],'target':[.75,3.25,.36],'fov_degrees':19});cap(name+'-side')
  call('load_scene',path='examples/connected.scene.json');assert call('inspect')['scene']==after
  (R/'artifacts/report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['report','pose_errors_m']}))
 finally:c.close()
if __name__=='__main__':main()
