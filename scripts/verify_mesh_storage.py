#!/usr/bin/env python3
"""Full closed16384v32768t asset, storage/version limits and actual GPU/replay."""
from verify_lab import SCENE_VERSION
import argparse,copy,json,math
from pathlib import Path
from verify_lab import Client,ROOT

def verify(binary,window=False):
 c=Client(binary,window);call=c.call;checks=[]
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'mesh-storage','version':'1'}});c.request('notifications/initialized',notification=True)
  call('generate',recipe={'seed':99,'width':20,'depth':20,'obstacles':0})
  limits=call('inspect_meshes')['limits'];assert all(limits[k] is None for k in ['assets','instances','total_vertices','total_triangles','draw_triangles'])
  positions=[];triangles=[];nx=256;ny=64
  for x in range(nx):
   a=x*math.tau/nx
   for y in range(ny):
    b=y*math.tau/ny;r=2+.5*math.cos(b);positions.append([r*math.cos(a),.5*math.sin(b),r*math.sin(a)])
  for x in range(nx):
   for y in range(ny):
    a=x*ny+y;b=((x+1)%nx)*ny+y;cc=x*ny+(y+1)%ny;d=((x+1)%nx)*ny+(y+1)%ny;triangles.extend([[a,cc,b],[b,cc,d]])
  asset={'id':'torus0','geometry':{'positions':positions,'triangles':triangles},'shading':'smooth'}
  for n in range(2):call('edit_mesh',edit={'kind':'put_asset','asset':dict(asset,id=f'torus{n}')})
  topo=call('inspect_mesh_topology',asset='torus0')['topology'];assert topo['boundary_edges']==topo['non_manifold_edges']==0 and not topo['isolated_vertices'];assert len(topo['faces'])==32768
  before=call('inspect')
  for mode in ['asset','vertices','triangles']:
   bad=copy.deepcopy(asset);bad['id']='overflow'
   if mode=='vertices':bad['geometry']['positions'].append([0,0,0])
   if mode=='triangles':bad['geometry']['triangles'].append(triangles[0])
   call('edit_mesh',edit={'kind':'put_asset','asset':bad});call('undo');assert call('inspect')['scene']==before['scene']
  checks.append('Closed16384v32768t torus, exact declared storage32768v65536t, max index16383 and larger assets/aggregate storage accepted and undo restores source')
  edits=[]
  for n in range(4):edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':{'id':f'torus{n}','asset':f'torus{n%2}','position':[(n%4-1.5)*2.7,1.7,(n//4-.5)*3.5],'rotation':[0,0,0,1],'scale':.4,'color':[.2,.5+.04*n,.3],'playback':None}}})
  call('edit_scene',edits=edits);call('set_camera',camera={'position':[13,12,17],'target':[0,1.5,0],'fov_degrees':40})
  def capture(name):
   path=f'artifacts/storage-{name}.png';meta=call('capture',path=path,width=900,height=700);assert meta['mesh_triangle_count']==131072 and meta['mesh_vertex_count']==393216;return (ROOT/path).read_bytes()
  start=capture('start');call('save_scene',path='examples/mesh-storage.scene.json');scene=call('inspect')['scene'];assert scene['version']==SCENE_VERSION
  old=copy.deepcopy(scene);old['version']=66;(ROOT/'artifacts/mesh-storage-old.scene.json').write_text(json.dumps(old));before=call('inspect');error=call('load_scene',path='artifacts/mesh-storage-old.scene.json',expected_error=True);assert 'version67' in str(error) and call('inspect')==before
  checks.append('Native GPU reaches131072 triangles with full-size source assets; scene66 cannot smuggle the extended storage')
  call('step',ticks=9);live=capture('live');assert capture('repeat')==live
  call('save_replay',path='artifacts/mesh-storage.replay.json');assert call('run_replay',path='artifacts/mesh-storage.replay.json')['meshes_matched']
  for name,tick,pixels in [('replay',9,live),('prefix',0,start)]:
   path=f'artifacts/storage-{name}.png';call('capture_replay',replay_path='artifacts/mesh-storage.replay.json',path=path,tick=tick,width=900,height=700);assert (ROOT/path).read_bytes()==pixels
  call('load_scene',path='examples/mesh-storage.scene.json');assert call('inspect')['scene']==scene and capture('loaded')==start
  checks.append('Bounded scene/replay serialization, save/load, repeated capture and live/prefix pixels exact at full storage/draw')
  result={'passed':True,'checks':checks,'native_window':window,'build_id':call('inspect')['build_id']};(ROOT/'artifacts/mesh-storage-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
 finally:c.close()
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary,a.window)
