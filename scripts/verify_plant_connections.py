#!/usr/bin/env python3
"""Regenerable paired plants through the shared dispatcher and real renderer."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import argparse,copy,json,math
from pathlib import Path
from verify_lab import Client,ROOT

def verify(binary,window=False):
 c=Client(binary,window);call=c.call;checks=[]
 def mesh(edit,**kw):return call('edit_mesh',edit=edit,**kw)
 def plant(edit,**kw):return call('edit_plant',edit=edit,**kw)
 def put(d):return {'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'}
 def asset(id):return call('inspect_mesh',asset=id)['asset']
 def pose(id,t):return call('inspect_mesh_pose',instance=id,tick=t)['pose']
 def capture(name):
  path=f'artifacts/connections-{name}.png';r=call('capture',path=path,width=720,height=540);return (ROOT/path).read_bytes(),r
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'plant-connections','version':'1'}});c.request('notifications/initialized',notification=True)
  schema=json.dumps(c.request('tools/list')['result']);assert all(s in schema for s in ['put_connection','remove_connection','inspect_plant_connections'])
  call('generate',recipe={'seed':93,'width':12,'depth':12,'obstacles':0})
  inp=json.loads((ROOT/'examples/plant-connection.input.json').read_text());recipe=inp['connection']
  for d in inp['definitions']:
   id=d['id'];plant(put(d));a=asset(id)
   bones=[{'name':'root','parent':None,'translation':[0,0,0],'rotation':[0,0,0,1]}]
   if id=='parent':bones.append({'name':'socket','parent':0,'translation':[0,-.12,0],'rotation':[0,0,0,1]})
   mesh({'kind':'set_rig','asset':id,'rig':{'bones':bones,'weights':[{'joints':[0,0,0,0],'weights':[1,0,0,0]}]*len(a['geometry']['positions'])}})
  mesh({'kind':'put_clip','asset':'parent','clip':{'id':'wind','duration_ticks':120,'tracks':[{'bone':1,'keys':[{'tick':t,'translation':[0,-.12,0],'rotation':[0,0,math.sin(a/2),math.cos(a/2)]} for t,a in [(0,0),(60,.1),(120,0)]]}]}})
  instances=[{'id':'parent-instance','asset':'parent','position':[0,2,0],'rotation':[0,0,0,1],'scale':1,'color':[.5,.3,.1],'playback':{'clip':'wind','offset_ticks':0,'looping':True}}, {'id':'child-instance','asset':'child','position':[.38971144,.225,0],'rotation':[0,0,-.5,.8660254],'scale':1,'color':[.5,.3,.1],'attachment':{'parent':'parent-instance','bone':'socket'}}]
  for i in instances:mesh({'kind':'put_instance','instance':i})
  mesh({'kind':'grid','id':'leaf','segments':1,'size':.2})
  mesh({'kind':'put_instance','instance':{'id':'leaf-instance','asset':'leaf','position':[0,.45,0],'rotation':[0,0,0,1],'scale':1,'color':[.1,.7,.1],'attachment':{'parent':'child-instance','bone':'root'}}})
  call('set_camera',camera={'position':[3,3,4],'target':[.2,2,0],'fov_degrees':35})
  original=call('inspect')['scene'];child=asset('child');oldposes={(id,t):pose(id,t) for id in ['child-instance','leaf-instance'] for t in [0,60,240]}
  edit={'kind':'put_connection','connection':recipe};state=call('inspect');pred=call('preview_scene_edits',edits=[{'op':'plant','edit':edit}])['scene'];assert call('inspect')==state
  preview=call('preview_plant',edit=edit);assert call('inspect')==state
  plant(edit);connected=call('inspect')['scene'];assert connected==pred and preview['assets']==connected['meshes']['assets']
  assert connected['plants']['entries']==original['plants']['entries'] and connected['meshes']['instances']==original['meshes']['instances'] and asset('child')==child
  for k,p in oldposes.items():assert pose(*k)==p
  report=call('inspect_plant_connections');assert report['reports'][0]['skin']['residual_max']<=1e-6
  topo=call('inspect_mesh_topology',asset='parent')['topology'];assert topo['boundary_edges']==topo['non_manifold_edges']==0 and not topo['isolated_vertices']
  assert len(call('inspect_mesh_islands',asset='parent')['topology']['islands'])==1
  drawn=sum(len(a['geometry']['triangles'])*sum(i['asset']==a['id'] and i['id']!='child-instance' for i in connected['meshes']['instances']) for a in connected['meshes']['assets'])
  start,meta=capture('start');assert meta['mesh_triangle_count']==drawn
  checks.append('Tiny native recipe, pure previews, closed connected parent, independent sources/rig/instances and exact child/descendant poses; renderer consumes only child body')
  out=asset('parent')
  for id in ['parent','child']:
   entry=call('inspect_plant',id=id)['plant'];d=copy.deepcopy(entry['definition']);d['trajectory']['control_points_xz'][1][0]+=.004
   for _ in range(2):
    plant(put(d));assert asset('parent')!=out and call('inspect_plant',id=id)['plant']['source']==entry['source'];plant(put(entry['definition']));assert call('inspect')['scene']==connected
  d=copy.deepcopy(inp['definitions'][1]);d['segments'][1]+=2;plant(put(d));assert asset('parent')!=out;plant(put(inp['definitions'][1]));assert call('inspect')['scene']==connected
  moved=copy.deepcopy(instances[1]);moved['position'][0]+=.001;state=call('inspect')
  pred=call('preview_mesh_edit',edit={'kind':'put_instance','instance':moved});assert call('inspect')==state
  mesh({'kind':'put_instance','instance':moved});assert asset('parent')!=out and call('inspect')['scene']['meshes']==pred['meshes']
  mesh({'kind':'put_instance','instance':instances[1]});assert call('inspect')['scene']==connected
  checks.append('Both trajectories, child longitudinal density and instance transform regenerate assembly; pure mesh preview and repeated A-B-A equality preserve canonical source')
  call('step',ticks=13)
  for case in ['missing','cycle','shared','station','solver','grid','refine','rebase','detach','instance','stale']:
   state=call('inspect');r=copy.deepcopy(recipe)
   if case=='missing':r['child_instance']='missing'
   elif case=='cycle':r['child_instance']=r['parent_instance']
   elif case=='shared':r['id']='other'
   elif case=='station':r['parent_sections'].reverse()
   elif case=='solver':r['skin']['iterations']=1
   elif case=='grid':r['shape']['cell_size']=.001
   if case in ['missing','cycle','shared','station','solver','grid']:plant({'kind':'put_connection','connection':r},expected_error=True)
   elif case=='refine':
    d=copy.deepcopy(inp['definitions'][1]);d['subdivision_levels']=2;plant(put(d),expected_error=True)
   elif case=='rebase':mesh({'kind':'set_rig','asset':'child','rig':child['rig']},expected_error=True)
   elif case=='detach':plant({'kind':'detach','id':'child'},expected_error=True)
   elif case=='instance':mesh({'kind':'remove_instance','id':'child-instance'},expected_error=True)
   else:plant(edit,expected_revision=state['revision']-1,expected_error=True)
   assert call('inspect')==state
  ref=call('inspect_mesh_topology',asset='child')['reference'];state=call('inspect')
  call('edit_scene',edits=[{'op':'plant','edit':{'kind':'remove_connection','id':'join'}},{'op':'mesh','edit':{'kind':'subdivide','asset':'child','reference':ref,'components':None}}],expected_error=True);assert call('inspect')==state
  checks.append('Missing/shared/cyclic dependencies, section order, solver/grid/refinement limits, rebase/detach/removal, stale revision and component reference after unlink reject atomically')
  call('reset');plant({'kind':'remove_connection','id':'join'});assert call('inspect')['scene']==original
  plain,meta=capture('removed');assert meta['mesh_triangle_count']==sum(len(a['geometry']['triangles'])*sum(i['asset']==a['id'] for i in original['meshes']['instances']) for a in original['meshes']['assets'])
  call('undo');assert call('inspect')['scene']==connected and capture('undo')[0]==start
  call('redo');assert call('inspect')['scene']==original and capture('redo')[0]==plain
  call('undo');call('save_scene',path='examples/plant-connections.scene.json');saved=call('inspect')['scene'];assert saved['version']==SCENE_VERSION
  for case in ['version','hidden','cache']:
   bad=copy.deepcopy(saved)
   if case=='cache':bad['meshes']['assets'][0]['geometry']['positions'][0][0]+=.001
   else:
    bad['version']=71
    if case=='hidden':bad['plants']['entries']=[]
   path='examples/connections-invalid.scene.json';(ROOT/path).write_text(json.dumps(bad));state=call('inspect');call('load_scene',path=path,expected_error=True);assert call('inspect')==state
  call('load_scene',path='examples/plant-connections.scene.json');assert call('inspect')['scene']==saved and capture('loaded')[0]==start
  checks.append('Unlink restores base asset/draw; undo/redo/load pixels exact; scene72 and cache integrity gates include hidden recipes')
  call('step',ticks=60);live=capture('live')[0];assert live!=start and capture('repeat')[0]==live
  call('save_replay',path='artifacts/connections.replay.json');assert json.loads((ROOT/'artifacts/connections.replay.json').read_text())['version']==REPLAY_VERSION
  assert call('run_replay',path='artifacts/connections.replay.json')['meshes_matched']
  for name,t,pixels in [('replay',60,live),('prefix',0,start)]:
   path=f'artifacts/connections-{name}.png';call('capture_replay',replay_path='artifacts/connections.replay.json',path=path,tick=t,width=720,height=540);assert (ROOT/path).read_bytes()==pixels
  call('reset');assert capture('reset')[0]==start
  checks.append('Animated live/repeat/replay76/prefix/reset reproduce native pixels with consumed draw mask')
  result={'passed':True,'checks':checks,'native_window':window,'build_id':call('inspect')['build_id']};(ROOT/'artifacts/plant-connections-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
 finally:c.close()
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary,a.window)
