#!/usr/bin/env python3
"""Native neutral root morphology and persistent woody history studies."""
import copy,json,math,time,hashlib
from collections import Counter
from verify_lab import ROOT,Client
OUT=ROOT/'artifacts/wood-details'
ANCESTRAL=ROOT/'games/land-of-survivors/artifacts/ancient-tree/current.scene.json'
def main():
 OUT.mkdir(parents=True,exist_ok=True)
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log')
 report={}
 def bot(**op):return c.call('botany',operation=op)
 def recipe(prefix):return next(r for r in bot(kind='inspect')['recipes'] if r['prefix']==prefix)
 def snap(name):c.call('capture',path=f'artifacts/wood-details/{name}.png',width=1500,height=1300)
 def camera(p,t,f=42):c.call('set_camera',camera=dict(position=p,target=t,fov_degrees=f))
 def light():c.call('set_light',light=dict(direction=[.5,-.75,-.5],ambient=.28,direct_intensity=1.25,environment=dict(sky=[.55,.6,.65],horizon=[.5]*3,ground=[.2]*3,axis=[0,1,0],intensity=.35)))
 def integrity(prefix):
  scene=c.call('inspect')['scene'];g=next(a['geometry'] for a in scene['meshes']['assets'] if a['id']==prefix+'_wood');edges=Counter();sign=Counter();v=0.
  for a,b,d in g['triangles']:
   p,q,r=[g['positions'][i] for i in (a,b,d)];cross=[q[1]*r[2]-q[2]*r[1],q[2]*r[0]-q[0]*r[2],q[0]*r[1]-q[1]*r[0]];v+=sum(x*y for x,y in zip(p,cross))/6
   for x,y in [(a,b),(b,d),(d,a)]:key=(min(x,y),max(x,y));edges[key]+=1;sign[key]+=1 if x<y else -1
  assert v>0 and all(n==2 and sign[e]==0 for e,n in edges.items())
  return dict(vertices=len(g['positions']),triangles=len(g['triangles']),closed_oriented=True,positive_volume=v,material_coordinates=bool(g.get('material_coordinates')))
 try:
  c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='wood-details',version='1')));c.request('notifications/initialized',notification=True)
  if ANCESTRAL.exists():c.call('load_scene',path=str(ANCESTRAL.relative_to(ROOT)));prefix='elven_ancestral';report['root_input']='preserved ancestral experiment'
  else:
   c.call('load_scene',path='artifacts/sequoia-realistic/empty.scene.json');bot(kind='load',path='assets/botany/wood-authoring-with-roots.ignis-botany',publish=True);prefix='woodlab';report['root_input']='tracked woody fixture'
  old=recipe(prefix);bot(kind='publish',recipe=old);light();old=recipe(prefix)
  views=dict(roots=([18,7,22],[0,1.4,0],44),root_side=([-19,5,16],[0,1,0],44),whole=([3,14,54],[0,13.5,0],36))
  report['roots_before']=integrity(prefix)
  for name,(p,t,f) in views.items():camera(p,t,f);snap(name+'-before')
  details=dict(seed=8821,age_years=180,healing_years=30,roots=dict(bulges=9,swelling=.35,flattening=.2,bend=.22,twist=.10,grain=.10),scars=[])
  camera(*views['roots']);bot(kind='set_wood_details',prefix=prefix,details=details,preview=True);snap('root-preview');assert recipe(prefix)==old
  bot(kind='cancel_preview');snap('root-cancel');assert (OUT/'root-cancel.png').read_bytes()==(OUT/'roots-before.png').read_bytes()
  start=time.monotonic();bot(kind='set_wood_details',prefix=prefix,details=details);report['root_apply_seconds']=time.monotonic()-start
  snap('root-published');assert (OUT/'root-published.png').read_bytes()==(OUT/'root-preview.png').read_bytes()
  assert recipe(prefix)['source']==old['source'];report['roots_after']=integrity(prefix)
  for name,(p,t,f) in views.items():camera(p,t,f);snap(name+'-after')
  c.call('save_scene',path='artifacts/wood-details/roots.scene.json');snap('roots-saved');c.call('load_scene',path='artifacts/wood-details/roots.scene.json');snap('roots-reopened');assert (OUT/'roots-saved.png').read_bytes()==(OUT/'roots-reopened.png').read_bytes()
  camera(*views['roots']);c.call('material',operation=dict(kind='load',asset=prefix+'_wood',path='assets/materials/sequoia/bark.ignis-material'));snap('roots-bark')
  # History starts at birth, so scheduled losses are actual chronological events.
  c.call('load_scene',path='artifacts/sequoia-realistic/empty.scene.json');light()
  definition=json.loads((ROOT/'examples/sequoia-volume.recipe.json').read_text());definition['prefix']='wood_history'
  bot(kind='create_living',definition=definition,born=0);prefix='wood_history'
  r=recipe(prefix);r['settings']['leaf_density']=0;r['settings']['sides']=16;bot(kind='publish',recipe=r)
  history=dict(seed=881,age_years=0,healing_years=22,natural_loss=dict(first_year=30,interval_years=25,maximum_events=5),scars=[])
  bot(kind='set_wood_details',prefix=prefix,details=history);bot(kind='save',prefix=prefix,path='assets/botany/wood-history.ignis-botany')
  bot(kind='living',prefix=prefix,action=dict(kind='advance',time=200*365));r=recipe(prefix);report['natural_events']=r['wood_details']['scars'];assert len(report['natural_events'])==5
  # Add two explicit historical knots to inspect metre-scale geometry and healing.
  history=copy.deepcopy(r['wood_details'])
  history['scars']+= [dict(id='old_knot',parent='stem',distance_m=4,angle=.3,radius_m=.8,raised_m=.45,depth_m=.24,year=40,closing_years=90,cause='historical'),dict(id='younger_knot',parent='stem',distance_m=9,angle=.8,radius_m=.65,raised_m=.4,depth_m=.22,year=150,closing_years=45,cause='historical')]
  bot(kind='set_wood_details',prefix=prefix,details=history)
  camera([8,5,13],[0,5,0],44);snap('knots-gray');report['history_geometry']=integrity(prefix)
  canonical=recipe(prefix)
  for name,year in [('young',39),('wounded',41),('healing',60),('adult',200),('young-again',39)]:
   bot(kind='preview_living_age',prefix=prefix,time=year*365);snap('age-'+name);assert recipe(prefix)==canonical
  assert (OUT/'age-young.png').read_bytes()==(OUT/'age-young-again.png').read_bytes();bot(kind='cancel_preview');snap('history-cancel');assert (OUT/'history-cancel.png').read_bytes()==(OUT/'knots-gray.png').read_bytes()
  # True recorded pruning on a currently present branch, with exact undo/redo.
  branch=next(o['id'] for o in canonical['source']['organs'] if o['id'].startswith('node') and o.get('parent')=='stem' and o['shape']['kind']=='axis')
  bot(kind='lose_branch',prefix=prefix,organ=branch,cause='pruning');snap('pruned');event=recipe(prefix)['wood_details']['scars'][-1];assert event['branch']==branch and event['year']==200;report['pruning_event']=event
  c.call('undo');snap('undo');assert (OUT/'undo.png').read_bytes()==(OUT/'knots-gray.png').read_bytes();c.call('redo');snap('redo');assert (OUT/'redo.png').read_bytes()==(OUT/'pruned.png').read_bytes()
  saved=recipe(prefix);bad=copy.deepcopy(saved['wood_details']);bad['scars'][-1]['depth_m']=-1
  c.call('botany',expected_error=True,operation=dict(kind='set_wood_details',prefix=prefix,details=bad));assert recipe(prefix)==saved
  c.call('save_scene',path='artifacts/wood-details/history.scene.json');snap('history-saved');c.call('load_scene',path='artifacts/wood-details/history.scene.json');snap('history-reopened');assert (OUT/'history-saved.png').read_bytes()==(OUT/'history-reopened.png').read_bytes()
  # Existing native bark generator asset; geometry is already reviewable above.
  c.call('material',operation=dict(kind='load',asset=prefix+'_wood',path='assets/materials/sequoia/bark.ignis-material'));appearance=c.call('material',operation=dict(kind='inspect',asset=prefix+'_wood'));snap('knots-bark')
  bot(kind='preview_living_age',prefix=prefix,time=230*365);assert c.call('material',operation=dict(kind='inspect',asset=prefix+'_wood'))==appearance;bot(kind='cancel_preview');snap('material-cancel');assert (OUT/'material-cancel.png').read_bytes()==(OUT/'knots-bark.png').read_bytes()
  bot(kind='open',open=True,prefix=prefix);c.call('capture_editor',path='artifacts/wood-details/editor.png',width=1700,height=1200)
  report.update(build=c.call('inspect')['build_id'],root_preview_publish_png_exact=True,root_save_load_png_exact=True,history_preview_back_forth_png_exact=True,history_save_load_png_exact=True,undo_redo_png_exact=True,invalid_atomic=True,material_preserved=True,installed_in_game=False,artistic_acceptance=False)
  (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
  cards=''.join(f'<h2>{v}</h2><div><figure><img src="{v}-before.png"><figcaption>Before</figcaption></figure><figure><img src="{v}-after.png"><figcaption>Root morphology</figcaption></figure></div>' for v in views)
  (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Roots and woody history</title><style>body{background:#20252b;color:#eee;font:16px system-ui;margin:24px}div{display:flex}figure{width:50%;margin:8px}img{width:100%}</style><h1>Irregular roots and persistent scars</h1><p>Native geometry. Authorial age/healing rules, no fracture or soil simulation. Existing collar/field-transport limitations remain; artistic acceptance pending.</p>'+cards+''.join(f'<h2>{name}</h2><img src="{name}.png">' for name in ['roots-bark','knots-gray','age-young','age-wounded','age-healing','age-adult','pruned','knots-bark','editor']))
  print(json.dumps(report))
 except BaseException:
  print(json.dumps(dict(native_exit=c.process.poll(),slow_requests=c.slow_requests)),flush=True);raise
 finally:c.close()
if __name__=='__main__':main()
