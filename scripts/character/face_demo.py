#!/usr/bin/env python3
"""Socket-contact and local facial-authoring checks in an isolated MCP process."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client
OUT=ROOT/'artifacts/character-face'

def main():
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'runtime.log');call=c.call;report={'checks':[],'captures':[]}
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'face-contact','version':'8'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='artifacts/character-face/user-before.scene.json');call('open_character_panel',open=True,prefix='female');old=call('inspect_character_editor');assert old['ready']
  old_body=call('inspect_mesh',asset='female_body')['asset'];old_config=old['binding']['recipe']['configuration']['parameters']
  src=json.load(open(ROOT/'artifacts/character-base/human-face.ignis-character'));fields=json.load(open(OUT/'source.json'))['fields'];new=[f['name'] for f in fields]
  call('upgrade_character_source',prefix='female',path='artifacts/character-base/human-face.ignis-character',reference=old['reference']);state=call('inspect_character_editor');baseline=state['binding'];params=baseline['recipe']['configuration']['parameters'];assert len(params)==392 and len(state['handles'])==95
  assert all(params[k]==v for k,v in old_config.items());call('undo');assert call('inspect_character_editor')['binding']==old['binding'];call('redo');assert call('inspect_character_editor')['binding']==baseline
  def change(values):return call('set_character_parameters',prefix='female',configuration={'parameters':values},gesture=None)
  change({'eye_L_socket_fit':0,'eye_R_socket_fit':0});assert call('inspect_character_editor')['binding']['bake_sha256']==old['binding']['bake_sha256']
  change({'eye_L_socket_fit':1,'eye_R_socket_fit':1});assert call('inspect_character_editor')['binding']==baseline
  report['checks'].append('343 authored values retained; source8 upgrade undo/redo atomic; disabling contact returns exact source7 bake')
  body=call('inspect_mesh',asset='female_body')['asset']
  assert body['geometry']['positions']==old_body['geometry']['positions']
  def gaps(body,eyes):
   result={}
   for e in src['eyes']:
    mesh=eyes[e['prefix']];points=mesh['positions'];covered=0;behind=[]
    for i in e['fit']['rim']:
     p=body['geometry']['positions'][i];hits=[]
     for ia,ib,ic in mesh['triangles']:
      a,b,c=points[ia],points[ib],points[ic];ab=[b[k]-a[k] for k in range(3)];ac=[c[k]-a[k] for k in range(3)];ap=[p[k]-a[k] for k in range(3)];d=ab[0]*ac[1]-ab[1]*ac[0]
      if abs(d)<1e-12:continue
      u=(ap[0]*ac[1]-ap[1]*ac[0])/d;v=(ab[0]*ap[1]-ab[1]*ap[0])/d
      if u>=-1e-5 and v>=-1e-5 and u+v<=1.00001:hits.append(a[2]+u*ab[2]+v*ac[2])
     if hits:covered+=1;behind.append(min(hits)-p[2])
    result[e['prefix']]=dict(center=[sum(p[k] for p in points)/len(points) for k in range(3)],covered=covered,total=len(e['fit']['rim']),maximum_front_depth_behind_rim=max(behind) if behind else None)
   return result
  def eyeballs():return {e['prefix']:call('inspect_mesh',asset='female_'+e['sclera'])['asset']['geometry'] for e in src['eyes']}
  fitted=eyeballs();report['contact']=gaps(body,fitted)
  assert all(v['covered']==v['total'] for v in report['contact'].values()),report['contact']
  call('save_scene',path='artifacts/character-face/baseline.scene.json')
  # Capture the actual authoring shape at matching front and three-quarter views.
  center=report['contact']['eye_R']['center'];world=[center[0],center[1]+.85,center[2]]
  def camera(label,oblique=False,close=False):
   target=world[:] if close else [0,world[1]-.02,world[2]];distance=.09 if close else .33
   cam=dict(position=[target[0]+(distance*.65 if oblique else 0),target[1]+distance*.04,target[2]-distance],target=target,fov_degrees=30)
   call('set_camera',camera=cam);call('edit_editor',action={'action':'camera','camera':cam},expected_editor_revision=None)
   call('capture',path='artifacts/character-face/'+label+'.png',width=1100,height=1000);report['captures'].append(label)
  camera('fitted-front');camera('fitted-eye',True,True)
  change({'eye_L_socket_fit':0,'eye_R_socket_fit':0});camera('legacy-eye',True,True);legacy_body=call('inspect_mesh',asset='female_body')['asset'];report['before_contact']=gaps(legacy_body,eyeballs());change({'eye_L_socket_fit':1,'eye_R_socket_fit':1})
  base={h['name']:h for h in call('inspect_character_editor')['handles']};protected=[n for n,h in base.items() if h.get('group') in ['Lábios','Orelhas']]+['nose_tip','nose_bridge']
  def move(name,offset,symmetry=False):return call('set_character_handle',prefix='female',handle=name,offset=offset,symmetry=symmetry,reference=call('inspect_character_editor')['reference'])
  maximum=0.
  for name in new:
   axis=0 if name.startswith('nose_') else 1 if name.startswith('nostril_') else 2;offset=[0,0,0];offset[axis]=.001
   move(name,offset,name.endswith('_R'));current={h['name']:h for h in call('inspect_character_editor')['handles']};delta=math.dist(current[name]['position'],base[name]['position']);assert .00085<delta<.00115,(name,delta)
   for n in protected:
    d=math.dist(current[n]['position'],base[n]['position']);maximum=max(maximum,d);assert d<3e-7,(name,n,d)
   isolated=['eye_inner_L','eye_inner_R','eye_outer_L','eye_outer_R','upper_lid_L','upper_lid_R','lower_lid_L','lower_lid_R'] if name.startswith(('malar_','orbital_')) else ['nose_wing_L','nose_wing_R','nose_alar_base_L','nose_alar_base_R'] if name.startswith('nostril_') else []
   for n in isolated:assert math.dist(current[n]['position'],base[n]['position'])<3e-7,(name,n)
   if name.startswith('nose_'):
    for n in new:
     if n.startswith('nose_') and n.rsplit('_',1)[0]!=name.rsplit('_',1)[0]:assert math.dist(current[n]['position'],base[n]['position'])<3e-7,(name,n)
   move(name,[0,0,0],name.endswith('_R'))
  report['max_protected_displacement']=maximum
  report['checks'].append('15 local XYZ anchors move1mm; lips/ears/nasal midline protected; three nasal width levels independently editable')
  # Limits are compilation checks, not universal guarantees against crossing surfaces.
  for name in new:
   for axis in range(3):
    for sign in [-1,1]:
     offset=[0,0,0];offset[axis]=sign*base[name]['limits'][axis];move(name,offset)
   move(name,[0,0,0])
  assert call('inspect_character_editor')['binding']==baseline
  for depth in [-.003,.008]:
   change({'eye_R_globe_depth':depth});current_body=call('inspect_mesh',asset='female_body')['asset'];eye=call('inspect_mesh',asset='female_eye_R_sclera')['asset'];metrics=gaps(current_body,{**fitted,'eye_R':eye['geometry']});assert metrics['eye_R']['covered']==metrics['eye_R']['total']
   assert call('inspect_mesh',asset='female_eye_L_sclera')['asset']['geometry']==fitted['eye_L']
   camera('eye-depth-'+('back' if depth>0 else 'forward'),True,True)
  change({'eye_R_globe_depth':0})
  before=call('inspect_character_editor')['binding'];ref=call('inspect_character_editor')['reference'];call('preview_character_handle',prefix='female',handle='nostril_roof_R',offset=[0,-.001,0],symmetry=True,reference=ref);assert call('inspect_character_editor')['binding']==before
  call('set_character_parameters',prefix='female',configuration={'parameters':{'eye_R_globe_depth':.1}},gesture=None,expected_error=True);assert call('inspect_character_editor')['binding']==before
  report['checks'].append('90 XYZ endpoints and ocular depth endpoints -3/+8mm compile; contact maintained; opposite eye independent; preview and bounds errors atomic')
  move('nostril_roof_R',[0,-.001,0],True);camera('lower-nostril');move('nostril_roof_R',[0,0,0],True)
  for name in ['nose_upper_side_R','nose_middle_side_R','nose_tip_side_R']:move(name,[-.001,0,0],True)
  camera('narrow-nose')
  for name in ['nose_upper_side_R','nose_middle_side_R','nose_tip_side_R']:move(name,[0,0,0],True)
  move('submental',[0,.006,0]);camera('raised-submental',True);move('submental',[0,0,0])
  move('malar_upper_R',[0,0,.003],True);move('orbital_lateral_R',[0,0,.003],True);camera('flatter-malar');move('malar_upper_R',[0,0,0],True);move('orbital_lateral_R',[0,0,0],True)
  camera('fitted-front');call('save_scene',path='artifacts/character-face/editor.scene.json');saved=call('inspect_character_editor')['binding'];call('save_character_configuration',prefix='female',path='artifacts/character-face/current.ignis-character-config');call('save_portable_save',path='artifacts/character-face/body.ignis-portable');call('load_portable_save',path='artifacts/character-face/body.ignis-portable',mode='recorded');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==saved
  call('load_scene',path='artifacts/character-face/editor.scene.json');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==saved
  call('capture',path='artifacts/character-face/reopened.png',width=1100,height=1000);assert (OUT/'reopened.png').read_bytes()==(OUT/'fitted-front.png').read_bytes();(OUT/'reopened.png').unlink()
  report['checks'].append('392 coefficients/source8 contact bake preserved by configuration, Scene and portable reload; native pixels exact after reopen')
  report.update(passed=True,build=call('inspect')['build_id'],source_sha256=baseline['recipe']['source_sha256'])
 except BaseException as e:report.update(passed=False,error=str(e));raise
 finally:(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');c.close()
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
