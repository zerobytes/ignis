#!/usr/bin/env python3
"""Explicit native trajectory adoption without rebasing a decorated legacy plant."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import argparse,copy,json,math
from pathlib import Path
from verify_lab import Client,ROOT

def verify(binary,window=False):
 c=Client(binary,window);call=c.call;checks=[]
 def asset():return call('inspect_mesh',asset='legacy')['asset']
 def entry():return call('inspect_plant',id='legacy')['plant']
 def put(d,kind='put',op='edit_plant'):return call(op,edit={'kind':kind,'definition':d,'detail_policy':'physical','animation_translation_policy':'physical'})
 def reject(d,kind='put'):
  before=call('inspect');call('edit_plant',edit={'kind':kind,'definition':d,'detail_policy':'physical','animation_translation_policy':'physical'},expected_error=True);assert call('inspect')==before
 def capture(name):
  path=f'artifacts/adoption-{name}.png';call('capture',path=path,width=900,height=1000);return (ROOT/path).read_bytes()
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'plant-adoption','version':'1'}});c.request('notifications/initialized',notification=True)
  call('generate',recipe={'seed':98,'width':12,'depth':12,'obstacles':0});entities=call('inspect')['scene']['entities']
  for e in entities:e['visible']=False
  call('edit_scene',edits=[{'op':'upsert','entity':e} for e in entities])
  d={'id':'legacy','profile':{'id':'profile','width':.04,'height':.3},'depth':.04,'sections':{'id':'sections','cross_section':'ellipse','stations':[{'id':'start','at':0,'width':1,'depth':1},{'id':'end','at':1,'width':1,'depth':1}]},'segments':[4,16,4],'sampling_tolerance':.001}
  put(d);base=asset()['geometry'];point=next(p for p in base['positions'] if abs(p[1])<1e-6 and p[2]>.019)
  for edit in [
   {'kind':'set_material_coordinates','asset':'legacy','coordinates':base['positions']},
   {'kind':'set_rig','asset':'legacy','rig':{'bones':[{'name':'root','parent':None,'translation':[0,-.12,0],'rotation':[0,0,0,1]},{'name':'tip','parent':0,'translation':[0,.135,0],'rotation':[0,0,0,1]}],'weights':[{'joints':[0,1,0,0],'weights':[.25,.75,0,0]} for _ in base['positions']]}},
   {'kind':'put_clip','asset':'legacy','clip':{'id':'bend','duration_ticks':60,'tracks':[{'bone':1,'keys':[{'tick':0,'translation':[0,.135,0],'rotation':[0,0,0,1]},{'tick':60,'translation':[0,.135,0],'rotation':[0,0,math.sin(.04),math.cos(.04)]}]}]}},
   {'kind':'sculpt','asset':'legacy','stroke':{'kind':'grab','centers':[point],'radius':.003,'strength':.0007,'direction':[0,0,1]}}
  ]:call('edit_mesh',edit=edit)
  mat=copy.deepcopy(asset()['material']);mat['reflectance']={'roughness':.7,'f0':.035};call('edit_mesh',edit={'kind':'set_material','asset':'legacy','material':mat})
  canonical=entry();original=asset();target=copy.deepcopy(d);target['trajectory']={'kind':'cubic_bezier','id':'axis','control_points_xz':[[0,0],[0,0],[-.1,0],[-.1,0]],'surface_tolerance':.001}
  reject(target);before=call('inspect');preview=put(target,'adopt_trajectory','preview_plant');assert call('inspect')==before
  put(target,'adopt_trajectory');bent=asset();assert bent==next(a for a in preview['assets'] if a['id']=='legacy');assert entry()['source']==canonical['source'] and entry()['reference']==canonical['reference']
  assert bent['geometry']['positions']!=original['geometry']['positions'] and bent['geometry']['material_coordinates']==original['geometry']['material_coordinates'];assert bent['material']==original['material'] and bent['rig']['weights']==original['rig']['weights'];assert bent['clips'][0]['tracks'][0]['keys'][0]['translation']==bent['rig']['bones'][1]['translation']
  call('undo');assert asset()==original and entry()==canonical;call('redo');assert asset()==bent
  checks.append('explicit preview/adoption retains legacy source/reference, sculpt domain, material, weights, rig and adapted clip; undo/redo exact')
  reject(target,'adopt_trajectory');reject(d)
  wrong=copy.deepcopy(target);wrong['trajectory']['id']='other-axis';reject(wrong)
  straight=copy.deepcopy(target);straight['trajectory']['control_points_xz']=[[0,0]]*4
  dense=copy.deepcopy(target);dense['segments']=[6,24,6]
  for _ in range(2):
   put(straight);put(dense);assert len(asset()['geometry']['positions'])>len(bent['geometry']['positions']);put(target);assert asset()==bent and entry()['source']==canonical['source'] and entry()['reference']==canonical['reference']
  assert call('inspect_plant',id='legacy')['report']['max_detail_displacement']>.00069
  checks.append('stable axis identity, invalid adoption, density changes and bend cycles stay atomic without canonical drift')
  call('edit_mesh',edit={'kind':'put_instance','instance':{'id':'legacy','asset':'legacy','position':[0,1,0],'rotation':[0,0,0,1],'scale':1,'color':[.6,.65,.7],'playback':{'clip':'bend','offset_ticks':0,'looping':False}}})
  call('set_camera',camera={'position':[.45,1.1,1.],'target':[-.05,1,0],'fov_degrees':22});start=capture('start');call('step',ticks=60);assert call('inspect')['tick']==60;live=capture('live');assert live!=start
  call('save_scene',path='examples/plant-adoption.scene.json');scene=call('inspect')['scene'];assert scene['version']==SCENE_VERSION
  call('save_replay',path='artifacts/plant-adoption.replay.json');assert json.loads((ROOT/'artifacts/plant-adoption.replay.json').read_text())['version']==REPLAY_VERSION
  assert call('run_replay',path='artifacts/plant-adoption.replay.json')['meshes_matched']
  for name,tick,pixels in [('replay',None,live),('prefix',0,start)]:
   args={} if tick is None else {'tick':tick};path=f'artifacts/adoption-{name}.png';call('capture_replay',replay_path='artifacts/plant-adoption.replay.json',path=path,width=900,height=1000,**args);assert (ROOT/path).read_bytes()==pixels
  old=copy.deepcopy(scene);old['version']=64;(ROOT/'examples/plant-adoption-old.scene.json').write_text(json.dumps(old));before=call('inspect');call('load_scene',path='examples/plant-adoption-old.scene.json',expected_error=True);assert call('inspect')==before
  call('load_scene',path='examples/plant-adoption.scene.json');assert call('inspect')['scene']==scene and capture('loaded')==start
  checks.append('version65 adoption guard, replay71, save/load and real animated tick60/prefix pixels exact headless')
  result={'passed':True,'checks':checks,'native_window':window,'build_id':call('inspect')['build_id']};(ROOT/'artifacts/plant-adoption-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
 finally:c.close()
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary,a.window)
