#!/usr/bin/env python3
"""Native MCP loft/fairing publication, rig, references, cache and replay."""
import argparse,copy,json,math
from pathlib import Path
from verify_lab import Client,ROOT

def verify(binary,window=False):
 c=Client(binary,window);call=c.call;checks=[]
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'surface-tools','version':'1'}});c.request('notifications/initialized',notification=True)
  schema=json.dumps(c.request('tools/list')['result']);assert all(k in schema for k in ['preview_mesh_ring_loft','preview_mesh_surface','smooth_region'])
  call('generate',recipe={'seed':54,'width':20,'depth':20,'obstacles':0});positions=[];triangles=[]
  for y,x in [(0,0),(1,.1),(2,.55),(3,.6)]:
   for j in range(8):
    a=j*math.tau/8;positions.append([x+.08*math.cos(a),y,.08*math.sin(a)])
  for i in range(3):
   for j in range(8):
    a=i*8+j;b=i*8+(j+1)%8;triangles.extend([[a,b,a+8],[b,b+8,a+8]])
  for j in range(1,7):triangles.extend([[0,j+1,j],[24,24+j,25+j]])
  bones=[{'name':'base','parent':None,'translation':[0,0,0],'rotation':[0,0,0,1]},{'name':'socket','parent':0,'translation':[.1,1,0],'rotation':[0,0,0,1]}]
  weights=[{'joints':[0,1,0,0],'weights':[1-i/31,i/31,0,0]} for i in range(32)]
  source={'id':'tube','geometry':{'positions':positions,'triangles':triangles,'material_coordinates':copy.deepcopy(positions)},'rig':{'bones':bones,'weights':weights},'shading':'smooth','clips':[{'id':'wind','duration_ticks':240,'tracks':[{'bone':1,'keys':[{'tick':t,'translation':[.1,1,0],'rotation':[0,0,math.sin(a/2),math.cos(a/2)]} for t,a in [(0,0),(60,.05),(120,0),(180,-.05),(240,0)]]}]}]}
  call('edit_mesh',edit={'kind':'put_asset','asset':source})
  for i in range(8,24):
   angle=(i%8)*math.tau/8;strength=.008*math.cos(angle*3)
   call('edit_mesh',edit={'kind':'sculpt','asset':'tube','stroke':{'kind':'grab','centers':[positions[i]],'radius':.005,'strength':strength,'direction':[math.cos(angle),0,math.sin(angle)]}})
  source=call('inspect_mesh',asset='tube')['asset']
  inst={'id':'tube','asset':'tube','position':[0,0,0],'rotation':[0,0,0,1],'scale':1,'color':[.45,.4,.3],'playback':{'clip':'wind','offset_ticks':0,'looping':True}}
  call('edit_mesh',edit={'kind':'put_instance','instance':inst})
  poses={t:call('inspect_mesh_pose',instance='tube',tick=t)['pose'] for t in [0,60]}
  loft={'rings':[list(range(i*8,(i+1)*8)) for i in range(4)],'spans':12,'interpolation':'cubic','max_deviation':.2}
  before=call('inspect');result=call('preview_mesh_ring_loft',source=source,lofts=[loft]);assert call('inspect')==before and call('preview_mesh_ring_loft',source=source,lofts=[loft])==result
  refined=result['asset'];assert refined['geometry']['positions'][:32]==source['geometry']['positions'];assert refined['rig']['weights'][:32]==source['rig']['weights'];assert refined['rig']['bones']==source['rig']['bones'] and refined['clips']==source['clips'];assert result['report']['curve_deviation_bound']>=result['report']['sampled_deviation']-1e-6
  linear=call('preview_mesh_ring_loft',source=source,lofts=[dict(loft,interpolation='linear')]);assert linear['asset']['geometry']['positions']!=refined['geometry']['positions'] and linear['report']['sampled_deviation']<1e-6
  for case in range(4):
   bad=copy.deepcopy(loft)
   if case==0:bad['rings'][1][0]=bad['rings'][0][0]
   elif case==1:bad['rings'][0].reverse()
   elif case==2:bad['max_deviation']=0
   else:bad['spans']=17
   call('preview_mesh_ring_loft',source=source,lofts=[bad],expected_error=True);assert call('inspect')==before
  call('edit_mesh',edit={'kind':'put_asset','asset':refined})
  for t,p in poses.items():
   new=call('inspect_mesh_pose',instance='tube',tick=t)['pose'];assert new['positions'][:32]==p['positions'] and new['joints']==p['joints']
  topo=call('inspect_mesh_topology',asset='tube')['topology'];assert topo['boundary_edges']==topo['non_manifold_edges']==0 and not topo['isolated_vertices']
  checks.append('Typed pure deterministic cubic/linear intent, explicit-ring refusals, retained rig/domain/source poses0/60 and closed topology')
  # A selected peak is an accepted fairing case; the curved tube remains a
  # separate preserved rigged loft. A generic filter may reject tube masks that
  # reduce the peak but increase mean bend, so do not weaken that guard here.
  fan=copy.deepcopy(source);fan['id']='surface';fan['geometry']={'positions':[[.3,1.7,0],[.7,1.5,0],[.3,1.5,.4],[-.1,1.5,0],[.3,1.5,-.4]],'triangles':[[0,2,1],[0,3,2],[0,4,3],[0,1,4]],'material_coordinates':[[0,1,0],[1,0,0],[0,0,1],[-1,0,0],[0,0,-1]]};fan['rig']['weights']=[{'joints':[0,1,0,0],'weights':[1-i/4,i/4,0,0]} for i in range(5)]
  call('edit_mesh',edit={'kind':'put_asset','asset':fan});call('edit_mesh',edit={'kind':'put_instance','instance':dict(inst,id='surface',asset='surface',position=[1,0,0],color=[.2,.65,.7])});refined=call('inspect_mesh',asset='surface')['asset']
  call('set_camera',camera={'position':[3,2.5,6],'target':[.8,1.5,0],'fov_degrees':32})
  call('capture',path='artifacts/surface-tools-before.png',width=700,height=700);pixels_before=(ROOT/'artifacts/surface-tools-before.png').read_bytes()
  vertices=[0,1];anchors=[1];options={'iterations':4,'strength':.1,'max_displacement':.01}
  before=call('inspect');smooth=call('preview_mesh_surface',source=refined,vertices=vertices,anchors=anchors,options=options);assert call('inspect')==before
  fixed=set(range(len(refined['geometry']['positions'])))-set(vertices)|set(anchors)
  for i in fixed:assert smooth['asset']['geometry']['positions'][i]==refined['geometry']['positions'][i]
  assert smooth['asset']['rig']==refined['rig'] and smooth['asset']['clips']==refined['clips'] and smooth['asset']['geometry']['material_coordinates']==refined['geometry']['material_coordinates']
  reference=call('inspect_mesh_topology',asset='surface')['reference'];edit={'kind':'smooth_region','asset':'surface','reference':reference,'vertices':vertices,'anchors':anchors,'options':options}
  predicted=call('preview_mesh_edit',edit=edit);assert call('inspect')==before
  call('edit_mesh',edit=edit);assert call('inspect_mesh',asset='surface')['asset']==smooth['asset'] and call('inspect')['scene']['meshes']==predicted['meshes']
  call('capture',path='artifacts/surface-tools-after.png',width=700,height=700);assert (ROOT/'artifacts/surface-tools-after.png').read_bytes()!=pixels_before
  after=call('inspect');call('edit_mesh',edit=edit,expected_error=True);assert call('inspect')==after
  call('undo');assert call('inspect')['scene']==before['scene'];call('redo');assert call('inspect')['scene']==after['scene']
  checks.append('Fairing mask/anchors/rig exact, pure preview equals publication, stale reference rejects and native undo/redo restore')
  call('set_camera',camera={'position':[3,2.5,6],'target':[.3,1.5,0],'fov_degrees':32});call('save_scene',path='examples/surface-tools.scene.json');saved=call('inspect')['scene']
  def capture(name):
   path=f'artifacts/surface-tools-{name}.png';call('capture',path=path,width=700,height=700);return (ROOT/path).read_bytes()
  start=capture('start');call('step',ticks=60);live=capture('live');assert live!=start and capture('repeat')==live
  call('save_replay',path='artifacts/surface-tools.replay.json');assert call('run_replay',path='artifacts/surface-tools.replay.json')['meshes_matched']
  for name,tick,pixels in [('replay',60,live),('prefix',0,start)]:
   path=f'artifacts/surface-tools-{name}.png';call('capture_replay',replay_path='artifacts/surface-tools.replay.json',path=path,tick=tick,width=700,height=700);assert (ROOT/path).read_bytes()==pixels
  call('load_scene',path='examples/surface-tools.scene.json');assert call('inspect')['scene']==saved and capture('loaded')==start
  checks.append('Native skinning and render cache, repeat/live/replay/prefix pixels, save/load exact without window')
  report={'passed':True,'checks':checks,'native_window':window,'build_id':call('inspect')['build_id']};(ROOT/'artifacts/mesh-surface-tools-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':report['build_id']}))
 finally:c.close()
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary,a.window)
