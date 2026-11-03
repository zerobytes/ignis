#!/usr/bin/env python3
"""Native proof for lower nasal ala landmarks and expanded authoring recipes."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client

def main():
 out=ROOT/'artifacts/character-nose';c=Client(ROOT/'target/release/ignis',log_path=out/'runtime.log');call=c.call;report={'checks':[],'captures':[],'metrics':{}}
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'character-nose','version':'6'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='artifacts/character-nose/user-before-nose.scene.json');call('open_character_panel',open=True,prefix='female');old=call('inspect_character_editor');assert old['ready']
  call('upgrade_character_source',prefix='female',path='artifacts/character-base/human-nose.ignis-character',reference=old['reference']);state=call('inspect_character_editor');binding=state['binding'];assert binding['recipe']['version']==2 and len(binding['recipe']['configuration']['parameters'])==262
  assert binding['bake_sha256']==old['binding']['bake_sha256'];assert all(binding['recipe']['configuration']['parameters'][k]==v for k,v in old['binding']['recipe']['configuration']['parameters'].items());assert len(state['handles'])==67
  call('undo');assert call('inspect_character_editor')['binding']==old['binding'];call('redo');assert call('inspect_character_editor')['binding']==binding
  call('save_scene',path='artifacts/character-nose/baseline.scene.json');assert json.load(open(out/'baseline.scene.json'))['version']==109
  base={h['name']:h for h in call('inspect_character_editor')['handles']}
  report['checks'].append('source5/recipe1→source6/recipe2/scene109 preserves256 old values and exact bake at zero;262channels/67landmarks; undo-redo atomic')
  def move(offset,side='R',symmetry=True):return call('set_character_handle',prefix='female',handle='nose_alar_base_'+side,offset=offset,symmetry=symmetry,reference=call('inspect_character_editor')['reference'])
  def capture(name,side=False):
   p=base['nose_alar_base_R']['position'];center=[0,p[1]+.004,p[2]];camera=dict(position=[center[0]+.19,center[1]+.006,center[2]-.025] if side else [0,center[1],center[2]-.19],target=center,fov_degrees=32)
   call('set_camera',camera=camera);call('capture',path='artifacts/character-nose/'+name+'.png',width=1100,height=1000);report['captures'].append(name)
  capture('zero-front');capture('zero-profile',True)
  for y,label in [(-.002,'lower'),(.002,'raise')]:
   move([0,y,0]);state=call('inspect_character_editor');handles={h['name']:h for h in state['handles']}
   for side in ['R','L']:
    name='nose_alar_base_'+side;d=math.dist(handles[name]['position'],base[name]['position']);assert abs(d-abs(y))<3e-7,(name,d);assert (handles[name]['position'][1]-base[name]['position'][1])*y>0
    wing='nose_wing_'+side;amount=math.dist(handles[wing]['position'],base[wing]['position']);assert amount<.0005,(wing,amount);report['metrics']['max_upper_wing_movement_m']=max(amount,report['metrics'].get('max_upper_wing_movement_m',0))
   for name in ['upper_lip','lower_lip','cupid_valley','mouth_corner_L','mouth_corner_R','nose_tip','chin','ear_lobe_R']:
    assert math.dist(handles[name]['position'],base[name]['position'])<3e-7,name
   capture(label+'-front');capture(label+'-profile',True);move([0,0,0])
  move([0,.002,0],symmetry=False);state=call('inspect_character_editor');assert math.dist(next(h for h in state['handles'] if h['name']=='nose_alar_base_L')['position'],base['nose_alar_base_L']['position'])<3e-7;move([0,0,0],symmetry=False)
  report['checks'].append('both alar bases raise/lower independently from the upper wing; optional symmetry and unilateral edit; lips/Cupid/nose tip/chin/ear remain unchanged')
  for side in ['L','R']:
   h=base['nose_alar_base_'+side]
   for axis in range(3):
    for sign in [-1,1]:
     offset=[0,0,0];offset[axis]=sign*h['limits'][axis];move(offset,side)
   move([0,0,0],side)
  before=call('inspect_character_editor')['binding'];ref=call('inspect_character_editor')['reference'];call('preview_character_handle',prefix='female',handle='nose_alar_base_R',offset=[0,.001,0],symmetry=True,reference=ref);assert call('inspect_character_editor')['binding']==before
  call('set_character_handle',prefix='female',handle='nose_alar_base_R',offset=[0,.006,0],symmetry=True,reference=ref,expected_error=True);assert call('inspect_character_editor')['binding']==before
  report['checks'].append('all12 XYZ endpoints compile; preview disposable and out-of-range update rejected without mutation')
  move([0,.0015,0]);durable=call('inspect_character_editor')['binding'];call('save_character_configuration',prefix='female',path='artifacts/character-nose/raised.ignis-character-config');call('save_scene',path='artifacts/character-nose/raised.scene.json');call('save_portable_save',path='artifacts/character-nose/body.ignis-portable');call('load_portable_save',path='artifacts/character-nose/body.ignis-portable',mode='recorded');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==durable
  call('load_scene',path='artifacts/character-nose/raised.scene.json');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==durable
  report['checks'].append('262 values and recipe2 survive configuration, scene and portable save/load; scene109/replay119 source contracts exercised by critical Rust fixtures')
  report.update(passed=True,build=call('inspect')['build_id'],source_sha256=binding['recipe']['source_sha256'])
 except BaseException as e:report.update(passed=False,error=str(e));raise
 finally:(out/'verification.json').write_text(json.dumps(report,indent=2)+'\n');c.close()
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
