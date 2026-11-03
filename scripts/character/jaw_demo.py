#!/usr/bin/env python3
"""Native mandibular-chain proof; never edits the human's running scene."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client
OUT=ROOT/'artifacts/character-jaw'

def main():
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'runtime.log');call=c.call;report={'checks':[],'captures':[],'metrics':{}}
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'jaw-chain','version':'1'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='artifacts/character-jaw/user-before-jaw.scene.json');call('open_character_panel',open=True,prefix='female');old=call('inspect_character_editor');assert old['ready']
  call('upgrade_character_source',prefix='female',path='artifacts/character-base/human-jaw.ignis-character',reference=old['reference']);state=call('inspect_character_editor');binding=state['binding'];assert len(binding['recipe']['configuration']['parameters'])==343 and len(state['handles'])==80
  assert binding['bake_sha256']==old['binding']['bake_sha256'];assert all(binding['recipe']['configuration']['parameters'][k]==v for k,v in old['binding']['recipe']['configuration']['parameters'].items())
  call('undo');assert call('inspect_character_editor')['binding']==old['binding'];call('redo');assert call('inspect_character_editor')['binding']==binding
  base={h['name']:h for h in call('inspect_character_editor')['handles']};new=[row['name'] for row in json.load(open(OUT/'source.json'))['fields']];assert len(new)==13
  report['checks'].append('all304 saved human values including green eyes and jaw angle retained; thirteen new XYZ handles atzero preserve complete bake exactly; upgrade undo/redo atomic')
  def move(name,offset,symmetry=False):return call('set_character_handle',prefix='female',handle=name,offset=offset,symmetry=symmetry,reference=call('inspect_character_editor')['reference'])
  protected=[n for n,h in base.items() if h.get('group') in ['Lábios','Orelhas']]+['nose_tip','nose_bridge','eye_L','eye_R','forehead_middle']
  maximum=0.
  for name in new:
   move(name,[0,.002,0],symmetry=not base[name]['midline']);current={h['name']:h for h in call('inspect_character_editor')['handles']}
   amount=math.dist(current[name]['position'],base[name]['position']);assert .0018<amount<.0022,(name,amount)
   for n in protected:
    delta=math.dist(current[n]['position'],base[n]['position']);maximum=max(maximum,delta);assert delta<3e-7,(name,n,delta)
   if name.endswith('_R'):
    mirror=name[:-1]+'L';assert .0018<math.dist(current[mirror]['position'],base[mirror]['position'])<.0022
   move(name,[0,0,0],symmetry=not base[name]['midline'])
  report['metrics']['max_protected_pin_displacement_m']=maximum
  report['checks'].append('all13 fields move their drawn anchor2mm; paired symmetry; all31lip/ear pins and nose/eyes/forehead remain invariant')
  for name in new:
   for axis in range(3):
    for sign in [-1,1]:
     offset=[0,0,0];offset[axis]=sign*base[name]['limits'][axis];move(name,offset)
   move(name,[0,0,0])
  before=call('inspect_character_editor')['binding'];ref=call('inspect_character_editor')['reference'];call('preview_character_handle',prefix='female',handle='jaw_body_R',offset=[.002,.001,0],symmetry=True,reference=ref);assert call('inspect_character_editor')['binding']==before
  call('set_character_handle',prefix='female',handle='jaw_body_R',offset=[.1,0,0],symmetry=True,reference=ref,expected_error=True);assert call('inspect_character_editor')['binding']==before
  move('jaw_body_R',[.002,0,0]);current={h['name']:h for h in call('inspect_character_editor')['handles']};assert math.dist(current['jaw_body_L']['position'],base['jaw_body_L']['position'])<3e-7;move('jaw_body_R',[0,0,0])
  assert call('inspect_character_editor')['binding']['bake_sha256']==old['binding']['bake_sha256']
  report['checks'].append('all78 XYZ endpoints compile; unilateral edit independent; preview disposable; bounds rejection atomic; returningzero restores exact prior bake')
  def camera(profile=False):
   chin=base['chin']['position'];angle=base['jaw_angle_R']['position'];center=[(a+b)*.5 for a,b in zip(chin,angle)];center[0]=0 if not profile else center[0]
   cam=dict(position=[center[0]+.25,center[1]+.015,center[2]-.025] if profile else [0,center[1]+.02,center[2]-.25],target=center,fov_degrees=32)
   call('set_camera',camera=cam);call('edit_editor',action={'action':'camera','camera':cam},expected_editor_revision=None)
  def capture(label,profile=False):camera(profile);call('capture',path='artifacts/character-jaw/'+label+'.png',width=1100,height=1000);report['captures'].append(label)
  capture('zero-front');capture('zero-profile',True)
  move('jaw_body_R',[.003,-.003,0],True);move('jaw_front_R',[.002,-.001,-.002],True);capture('wider-front');capture('wider-profile',True)
  call('save_character_configuration',prefix='female',path='artifacts/character-jaw/wider.ignis-character-config')
  move('jaw_body_R',[0,0,0],True);move('jaw_front_R',[0,0,0],True)
  call('save_scene',path='artifacts/character-jaw/zero.scene.json');call('save_character_configuration',prefix='female',path='artifacts/character-jaw/zero.ignis-character-config');durable=call('inspect_character_editor')['binding']
  call('save_portable_save',path='artifacts/character-jaw/body.ignis-portable');call('load_portable_save',path='artifacts/character-jaw/body.ignis-portable',mode='recorded');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==durable
  call('load_scene',path='artifacts/character-jaw/zero.scene.json');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==durable
  call('set_character_handle_view',view={'enabled':True,'symmetry':True,'selected':'jaw_body_R','group':'Mandíbula'});camera(True);call('edit_editor',action={'action':'lighting','light':None},expected_editor_revision=None);call('capture_editor',path='artifacts/character-jaw/points-editor.png',width=1440,height=1000)
  report['checks'].append('343values/configuration and source hash survive Scene/portable reload; generic native mandibular group and3Dpoints available')
  report.update(passed=True,build=call('inspect')['build_id'],source_sha256=binding['recipe']['source_sha256'],old_bake=old['binding']['bake_sha256'])
 except BaseException as e:report.update(passed=False,error=str(e));raise
 finally:(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');c.close()
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
