#!/usr/bin/env python3
"""Independent sphere/convexity checks plus real native face and isolated-eye views."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client
OUT=ROOT/'artifacts/character-sphere'
def sphere(points):
 p0=points[-1];a=[[0.]*4 for _ in range(3)]
 for p in points[::8]:
  row=[2*(p[i]-p0[i]) for i in range(3)];b=sum(x*x for x in p)-sum(x*x for x in p0)
  for i in range(3):
   for k in range(3):a[i][k]+=row[i]*row[k]
   a[i][3]+=row[i]*b
 for k in range(3):
  at=max(range(k,3),key=lambda i:abs(a[i][k]));a[k],a[at]=a[at],a[k];scale=a[k][k];a[k]=[v/scale for v in a[k]]
  for i in range(3):
   if i!=k:
    scale=a[i][k];a[i]=[x-scale*y for x,y in zip(a[i],a[k])]
 center=[a[i][3] for i in range(3)];radius=math.dist(p0,center);return center,radius,max(abs(math.dist(p,center)-radius) for p in points)
def winding(g,center):
 minimum=math.inf
 for t in g['triangles']:
  a,b,c=[g['positions'][i] for i in t];u=[b[i]-a[i] for i in range(3)];v=[c[i]-a[i] for i in range(3)];normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];out=[(a[i]+b[i]+c[i])/3-center[i] for i in range(3)];minimum=min(minimum,sum(x*y for x,y in zip(normal,out)))
 return minimum
def main():
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'runtime.log');call=c.call;report={'checks':[],'captures':[]}
 try:
  c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'sphere-regression','version':'9'}});c.request('notifications/initialized',notification=True)
  call('load_scene',path='artifacts/character-sphere/user-before.scene.json');call('open_character_panel',open=True,prefix='female');old=call('inspect_character_editor');assert old['ready']
  before={s:call('inspect_mesh',asset=f'female_eye_{s}_sclera')['asset']['geometry'] for s in ['L','R']};report['before']={s:dict(zip(['center','radius','max_radius_error'],sphere(g['positions']))) for s,g in before.items()}
  old_body=call('inspect_mesh',asset='female_body')['asset']['geometry']['positions'];old_material=call('inspect_mesh',asset='female_eye_R_iris')['asset']['material']
  call('upgrade_character_source',prefix='female',path='artifacts/character-base/human-spherical.ignis-character',reference=old['reference']);state=call('inspect_character_editor');binding=state['binding'];params=binding['recipe']['configuration']['parameters'];assert len(params)==392 and len(state['handles'])==95
  assert params==old['binding']['recipe']['configuration']['parameters'];call('undo');assert call('inspect_character_editor')['binding']==old['binding'];call('redo');assert call('inspect_character_editor')['binding']==binding
  assert call('inspect_mesh',asset='female_eye_R_iris')['asset']['material']==old_material
  src=json.load(open(ROOT/'artifacts/character-base/human-spherical.ignis-character'));region={w['vertex'] for e in src['eyes'] for w in e['fit']['lid']};body=call('inspect_mesh',asset='female_body')['asset']['geometry']['positions'];assert all(p==q for i,(p,q) in enumerate(zip(body,old_body)) if i not in region)
  def check():
   result={};skin=call('inspect_mesh',asset='female_body')['asset']['geometry']['positions']
   for e in src['eyes']:
    g=call('inspect_mesh',asset='female_'+e['sclera'])['asset']['geometry'];center,radius,error=sphere(g['positions']);convex=winding(g,center)
    assert error<3e-7,(e['prefix'],error);assert convex>0,(e['prefix'],convex)
    radial=[abs(math.dist(skin[i],center)-radius) for i in e['fit']['rim']]
    result[e['prefix']]=dict(center=center,radius=radius,max_radius_error=error,min_outward_winding=convex,max_rim_separation=max(radial))
   return result
  report['spheres']=check();assert all(v['max_rim_separation']<.0001 for v in report['spheres'].values())
  report['checks'].append('all392 user values and iris material retained; source8 undo/redo exact; source9 sclera has one radius and outward winding; only bounded lid regions change')
  call('save_scene',path='artifacts/character-sphere/baseline.scene.json');base=params.copy()
  def change(values):call('set_character_parameters',prefix='female',configuration={'parameters':values},gesture=None)
  for amount in [0,.5,1]:
   for depth in [-.003,.008]:change({'eye_R_socket_fit':amount,'eye_R_globe_depth':depth});check()
  change({'eye_R_socket_fit':base['eye_R_socket_fit'],'eye_R_globe_depth':base['eye_R_globe_depth']})
  for pose in [dict(head_pitch=.2),dict(head_pitch=base['head_pitch'],neck_pitch=-.2)]:change(pose);check()
  change({'neck_pitch':base['neck_pitch']});assert call('inspect_character_editor')['binding']==binding
  ref=call('inspect_character_editor')['reference'];call('set_character_parameters',prefix='female',configuration={'parameters':{'eye_R_globe_depth':-.004}},gesture=None,expected_error=True);assert call('inspect_character_editor')['binding']==binding
  report['checks'].append('globe remains spherical through fit strengths, depth endpoints, whole-head and neck transforms; bounds rejection atomic')
  center=report['spheres']['eye_R']['center'];world=[center[0],center[1]+.85,center[2]]
  def capture(name,target,distance=.09,angle=0):
   cam=dict(position=[target[0]+distance*math.sin(angle),target[1]+distance*.03,target[2]-distance*math.cos(angle)],target=target,fov_degrees=30)
   call('set_camera',camera=cam);call('capture',path='artifacts/character-sphere/'+name+'.png',width=1100,height=1000);report['captures'].append(name)
  capture('face-front',[0,world[1]-.02,world[2]],.33);capture('face-eye',world,.09,.6);capture('face-profile',[0,world[1]-.02,world[2]],.33,math.pi/2)
  call('save_scene',path='artifacts/character-sphere/face.scene.json');call('save_character_configuration',prefix='female',path='artifacts/character-sphere/current.ignis-character-config');call('save_portable_save',path='artifacts/character-sphere/body.ignis-portable');call('load_portable_save',path='artifacts/character-sphere/body.ignis-portable',mode='recorded');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==binding
  call('capture',path='artifacts/character-sphere/reopened.png',width=1100,height=1000);assert (OUT/'reopened.png').read_bytes()==(OUT/'face-profile.png').read_bytes();(OUT/'reopened.png').unlink()
  j=json.load(open(OUT/'face.scene.json'));j['characters']=[];j['meshes']['instances']=[i for i in j['meshes']['instances'] if i['id'].startswith('female_eye_R_')];(OUT/'isolated.scene.json').write_text(json.dumps(j,separators=(',',':')));call('load_scene',path='artifacts/character-sphere/isolated.scene.json')
  for label,angle in [('front',0),('oblique',.7),('profile',math.pi/2)]:capture('globe-'+label,world,.07,angle)
  report['checks'].append('native front/oblique/profile face and isolated-globe views; Scene/configuration/portable bake retained and reopened pixels exact')
  report.update(passed=True,build=call('inspect')['build_id'],source_sha256=binding['recipe']['source_sha256'])
 except BaseException as e:report.update(passed=False,error=str(e));raise
 finally:(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');c.close()
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()
