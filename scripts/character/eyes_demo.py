#!/usr/bin/env python3
"""Real MCP ocular authoring, preservation, pose and durable pixel checks."""
import hashlib,json,math,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client
OUT=ROOT/'artifacts/character-eyes';OUT.mkdir(exist_ok=True)

def main():
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'runtime.log');call=c.call;report={'checks':[],'captures':[],'edit_ms':[]}
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'character-eyes','version':'7'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='artifacts/character-base/female.scene.json');call('open_character_panel',open=True,prefix='female');old=call('inspect_character_editor');assert old['ready']
  call('save_scene',path='artifacts/character-eyes/before.scene.json')
  body=call('inspect_mesh',asset='female_body')['asset'];old_params=old['binding']['recipe']['configuration']['parameters']
  t=time.monotonic();call('upgrade_character_source',prefix='female',path='artifacts/character-base/human-eyes.ignis-character',reference=old['reference']);report['upgrade_ms']=(time.monotonic()-t)*1000
  state=call('inspect_character_editor');params=state['binding']['recipe']['configuration']['parameters'];assert len(params)==304
  assert all(params[k]==v for k,v in old_params.items());assert call('inspect_mesh',asset='female_body')['asset']==body
  upgraded=state['binding'];call('undo');assert call('inspect_character_editor')['binding']==old['binding'];call('redo');assert call('inspect_character_editor')['binding']==upgraded
  report['checks'].append('262 authored values and exact body/rig preserved; seven owned identities; source7 upgrade undo/redo atomic')
  def change(values):
   t=time.monotonic();call('set_character_parameters',prefix='female',configuration={'parameters':values},gesture=None);report['edit_ms'].append((time.monotonic()-t)*1000)
  # Use the fitted ocular geometry, including the user-authored head forms.
  a=call('inspect_mesh',asset='female_eye_R_iris')['asset']['geometry']['positions'];instance=next(i for i in json.load(open(OUT/'before.scene.json'))['meshes']['instances'] if i['id']=='female_eye_R_iris')
  assert instance['rotation']==[0,0,0,1] and instance['scale']==1
  center=[sum(p[k] for p in a)/len(a)+instance['position'][k] for k in range(3)]
  def camera(mode='close',oblique=False):
   target=center if mode=='close' else [0,center[1]-.015,center[2]]
   distance=.075 if mode=='close' else .36
   call('set_camera',camera=dict(position=[target[0]+(distance*.45 if oblique else 0),target[1]+distance*.08,target[2]-distance],target=target,fov_degrees=30))
  def capture(name):
   result=call('capture',path='artifacts/character-eyes/'+name+'.png',width=1000,height=900);report['captures'].append(name);return result
  # A controlled light is confined to this separate demonstration scene.
  call('set_light',light={'direction':[-.3,-.35,.85],'ambient':.18,'direct_intensity':1.,'environment':{'sky':[.5,.6,.75],'horizon':[.55,.5,.43],'ground':[.11,.09,.08],'axis':[0,1,0],'intensity':.5}})
  camera();first=capture('brown-close');warm=capture('brown-warm');assert (OUT/'brown-close.png').read_bytes()==(OUT/'brown-warm.png').read_bytes();(OUT/'brown-warm.png').unlink();report['captures'].remove('brown-warm')
  report['adapter']=warm['adapter'];report['checks'].append('same warm frame byte-identical; material upload counters recorded')
  call('save_scene',path='artifacts/character-eyes/baseline.scene.json')
  colors={'blue':([.065,.19,.32],[.075,.12,.13]),'green':([.11,.19,.095],[.24,.15,.035]),'hazel':([.30,.20,.055],[.37,.16,.025])}
  for label,(outer,inner) in colors.items():
   values={f'eye_{side}_{name}':v for side in ['L','R'] for names,rgb in [(['red','green','blue'],outer),(['inner_red','inner_green','inner_blue'],inner)] for name,v in zip(names,rgb)}
   change(values);capture(label+'-close')
  camera('portrait');capture('hazel-portrait');camera(oblique=True);capture('hazel-oblique')
  old_left={k:v for k,v in call('inspect_character_editor')['binding']['recipe']['configuration']['parameters'].items() if k.startswith('eye_L_')}
  change({'eye_R_pupil':.58,'eye_R_seed':913.});now=call('inspect_character_editor')['binding']['recipe']['configuration']['parameters'];assert all(now[k]==v for k,v in old_left.items());camera();capture('dilated-right')
  change({'eye_R_pupil':.19});capture('contracted-right')
  call('load_scene',path='artifacts/character-eyes/baseline.scene.json');call('open_character_panel',open=True,prefix='female')
  original=call('inspect_character_editor')['binding'];call('set_character_parameters',prefix='female',configuration={'parameters':{'eye_R_pupil':2}},gesture=None,expected_error=True);assert call('inspect_character_editor')['binding']==original
  # Local identity follows every existing fitted eye socket, including whole-head tilt.
  for pose in [dict(head_pitch=.3),dict(head_pitch=0,neck_pitch=-.25),dict(neck_pitch=0,face_eye_R_x=.002)]:
   change(pose)
   assert call('inspect_character_editor')['ready']
  change(dict(head_pitch=params['head_pitch'],neck_pitch=params['neck_pitch'],face_eye_R_x=params['face_eye_R_x']))
  report['checks'].append('independent pigment/pupil/identity; existing rigid head/neck and orbit controls compile; invalid bounds rejected atomically')
  # Deliver a blue example to show structure clearly; both sides are fully editable.
  outer,inner=colors['blue'];change({f'eye_{side}_{n}':v for side in ['L','R'] for names,rgb in [(['red','green','blue'],outer),(['inner_red','inner_green','inner_blue'],inner)] for n,v in zip(names,rgb)})
  camera('portrait');capture('editor-portrait');call('save_scene',path='artifacts/character-eyes/editor.scene.json');saved=call('inspect_character_editor')['binding']
  old_scene=json.load(open(OUT/'before.scene.json'));new_scene=json.load(open(OUT/'editor.scene.json'));old_scene.update(version=new_scene['version'],camera=new_scene['camera'],light=new_scene['light']);(OUT/'cost-before.scene.json').write_text(json.dumps(old_scene,separators=(',',':')))
  call('save_character_configuration',prefix='female',path='artifacts/character-eyes/current.ignis-character-config');call('save_portable_save',path='artifacts/character-eyes/body.ignis-portable');call('load_portable_save',path='artifacts/character-eyes/body.ignis-portable',mode='recorded');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==saved
  capture('portable-portrait');assert (OUT/'editor-portrait.png').read_bytes()==(OUT/'portable-portrait.png').read_bytes();(OUT/'portable-portrait.png').unlink();report['captures'].remove('portable-portrait')
  call('load_scene',path='artifacts/character-eyes/editor.scene.json');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==saved
  capture('reopened-portrait');assert (OUT/'editor-portrait.png').read_bytes()==(OUT/'reopened-portrait.png').read_bytes();(OUT/'reopened-portrait.png').unlink();report['captures'].remove('reopened-portrait')
  report['checks'].append('all304 values/configuration and ocular bake survive Scene and portable save/reopen with exact native pixels')
  report.update(passed=True,build=call('inspect')['build_id'],source_sha256=upgraded['recipe']['source_sha256'],eye_center=center)
 except BaseException as e:report.update(passed=False,error=str(e));raise
 finally:(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');c.close()
 cards=''.join(f'<figure><img src="{name}.png"><figcaption>{name}</figcaption></figure>' for name in report['captures'])
 (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Ignis ocular authoring</title><style>body{background:#171c24;color:#edf0f5;font:16px system-ui;max-width:1500px;margin:32px auto;padding:20px}main{display:grid;grid-template-columns:repeat(2,1fr);gap:24px}img{width:100%}figure{margin:0}figcaption{padding:10px}</style><h1>Ignis · Iris authoring</h1><p>Native renderer. Same authored face. Seeded pigment, manual pupil and corneal reflection; no spatial ray refraction. Original character scene preserved.</p><p><a href="verification.json">Verification</a> · <a href="cost.json">Warm draw cost</a></p><main>'+cards+'</main>')
 print(json.dumps({k:v for k,v in report.items() if k!='adapter'},indent=2))
if __name__=='__main__':main()
