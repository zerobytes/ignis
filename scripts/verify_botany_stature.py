#!/usr/bin/env python3
"""Native numeric/authoring checks for botanical width; no ancestral art claim."""
import copy,json,time
from pathlib import Path
from verify_lab import Client,ROOT
from demo_groundcover import setup
OUT=ROOT/'artifacts/botany-stature'

def main():
 OUT.mkdir(parents=True,exist_ok=True);c=Client(ROOT/'target/release/ignis',log_path=OUT/'runtime.log');report={}
 def botany(**operation):return c.call('botany',operation=operation)
 def recipe(prefix):return next(r for r in botany(kind='inspect')['recipes'] if r['prefix']==prefix)
 def diameter(r):return r['source']['organs'][0]['shape']['radius']*2
 try:
  c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='stature-check',version='1')));c.request('notifications/initialized',notification=True)
  report['build']=setup(c)
  d=json.loads((ROOT/'examples/living-tree.recipe.json').read_text());d['version']=3;d['prefix']='stature';d['stature']=dict(adult_diameter_m=5,seedling_diameter_m=.01,maturity_years=50)
  botany(kind='create_living',definition=d,born=0);rows=[]
  for year in [0,5,20,50]:
   start=time.monotonic();result=botany(kind='living',prefix='stature',action=dict(kind='advance',time=year*365));r=recipe('stature')
   rows.append(dict(year=year,diameter_m=diameter(r),organs=len(r['source']['organs']),wall_seconds=time.monotonic()-start,compiled=result['compiled']))
  assert abs(rows[0]['diameter_m']-.01)<1e-10 and abs(rows[-1]['diameter_m']-5)<1e-10
  assert all(a['diameter_m']<b['diameter_m'] for a,b in zip(rows,rows[1:]));report['growth']=rows
  original=recipe('stature');c.call('save_scene',path='artifacts/botany-stature/current.scene.json');c.call('load_scene',path='artifacts/botany-stature/current.scene.json');assert recipe('stature')==original;report['scene_roundtrip']=True
  legacy=copy.deepcopy(d);legacy['version']=2;legacy['prefix']='legacy';legacy.pop('stature')
  botany(kind='create_living',definition=legacy,born=0);botany(kind='living',prefix='legacy',action=dict(kind='advance',time=50*365))
  old=recipe('legacy');leaves={o['id']:o['shape'] for o in old['source']['organs'] if o['shape']['kind']=='blade'}
  assert all(o['shape']==leaves[o['id']] for o in original['source']['organs'] if o['shape']['kind']=='blade');report['leaf_dimensions_preserved']=True
  wide=json.loads((ROOT/'assets/botany/wood-authoring.ignis-botany').read_text());wide['prefix']='wide'
  for organ in wide['source']['organs']:
   organ['path']=[[x*100 for x in p] for p in organ['path']];organ['shape']['radius']*=100
  surface=wide['edits']['trunk']['wood_surface']
  for section in surface['sections']:section['radii']=[x*100 for x in section['radii']];section['offset']=[x*100 for x in section['offset']]
  for relief in surface['relief']:relief['height']*=100;relief['width']*=100
  report['large_volume']=botany(kind='publish',recipe=wide)['compiled'];assert wide['source']['organs'][0]['shape']['radius']==250
  state=c.call('inspect')['revision'];before=recipe('stature')
  c.call('botany',expected_error=True,operation=dict(kind='living',prefix='stature',action=dict(kind='set_stature',stature=dict(adult_diameter_m=1.7976931348623157e308,seedling_diameter_m=.01,maturity_years=50))))
  assert c.call('inspect')['revision']==state and recipe('stature')==before;report['unrepresentable_rejected_atomically']=True
  (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
 finally:c.close()
if __name__=='__main__':main()
