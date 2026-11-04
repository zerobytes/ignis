#!/usr/bin/env python3
"""Neutral native review of editable woody volumes; never installs the elven tree."""
import argparse,copy,json,time,hashlib
from collections import Counter
from pathlib import Path
from verify_lab import Client,ROOT
from demo_groundcover import setup,fit,relative
OUT=ROOT/'artifacts/wood-authoring'
SOURCE=ROOT/'assets/botany/wood-authoring.ignis-botany'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');args=p.parse_args();OUT.mkdir(parents=True,exist_ok=True)
 c=Client(args.binary,log_path=OUT/'runtime.log');report={}
 def botany(**operation):return c.call('botany',operation=operation)
 try:
  c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='wood-authoring-review',version='1')));c.request('notifications/initialized',notification=True)
  report['build']=setup(c);report['binary_sha256']=hashlib.sha256(args.binary.read_bytes()).hexdigest()
  c.call('set_light',light=dict(direction=[.45,-.82,-.35],ambient=.3,direct_intensity=1.1,environment=dict(sky=[.5,.5,.5],horizon=[.5,.5,.5],ground=[.3,.3,.3],axis=[0,1,0],intensity=.35),shadow=dict(center=[0,7,0],extent=27,resolution=2048,bias=.001)))
  started=time.monotonic();result=botany(kind='load',path=relative(SOURCE),publish=True);report['initial_compile']=dict(seconds=time.monotonic()-started,result=result)
  # Existing native radial roots, round rather than wall-shaped. Editable curves are retained.
  settings=dict(kind='radial',count=4,seed=91,spread=10,depth=.8,radius_ratio=.5,neck_height=1.25,exposure=1.,orders=0,irregularity=.13)
  started=time.monotonic();result=botany(kind='set_roots',prefix='woodlab',stem='trunk',settings=settings);report['root_compile']=dict(seconds=time.monotonic()-started,result=result)
  r=next(r for r in botany(kind='inspect')['recipes'] if r['prefix']=='woodlab')
  for index,o in enumerate(o for o in r['source']['organs'] if '/root' in o['id']):
   # Native curvature fields use the existing organ controls.
   r['edits'][o['id']]=dict(deformation=dict(base=[0,.025*(-1 if index%2 else 1)],tip=[0,-.04*(-1 if index%2 else 1)]))
  result=botany(kind='publish',recipe=r)
  report['root_curve_compile']=result
  botany(kind='save',prefix='woodlab',path='assets/botany/wood-authoring-with-roots.ignis-botany')
  scene=c.call('inspect')['scene'];wood=next(a for a in scene['meshes']['assets'] if a['id']=='woodlab_wood')['geometry'];positions=wood['positions'];lo=[min(v[i] for v in positions) for i in range(3)];hi=[max(v[i] for v in positions) for i in range(3)]
  edges=Counter();orientation=Counter()
  for face in wood['triangles']:
   for a,b in zip(face,face[1:]+face[:1]):
    edge=(min(a,b),max(a,b));edges[edge]+=1;orientation[edge]+=1 if a<b else -1
  assert all(count==2 and orientation[edge]==0 for edge,count in edges.items())
  report['geometry']=dict(vertices=len(positions),triangles=len(wood['triangles']),bounds=[lo,hi],closed_oriented=True,geometry_bytes_f32_u32=12*(len(positions)+len(wood['triangles'])),source_bytes=(ROOT/'assets/botany/wood-authoring-with-roots.ignis-botany').stat().st_size)
  # Ground below exposed roots; geometry is visible without leaf/texture camouflage.
  ground=dict(id='wood_ground',geometry=dict(positions=[[-40,-.04,-40],[40,-.04,-40],[-40,-.04,40],[40,-.04,40]],triangles=[[0,2,1],[1,2,3]]))
  c.call('edit_scene',edits=[dict(op='mesh',edit=dict(kind='put_asset',asset=ground)),dict(op='mesh',edit=dict(kind='put_instance',instance=dict(id='wood_ground',asset='wood_ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.32,.32,.32])))])
  for name,direction in [('front',(0,.15,1)),('side',(1,.15,0)),('back',(0,.15,-1)),('top',(.015,1,.03))]:fit(c,lo,hi,OUT/f'{name}.png',direction=direction,width=1400,height=1200)
  for name,pos,target in [('trunk-detail',[8,6,10],[0,5,0]),('base-detail',[9,4,10],[0,1.5,0]),('fork-detail',[-9,11,10],[0,8,0])]:
   c.call('set_camera',camera=dict(position=pos,target=target,fov_degrees=42));c.call('capture',path=relative(OUT/f'{name}.png'),width=1400,height=1200)
  c.call('save_scene',path=relative(OUT/'wood.scene.json'));c.call('capture',path=relative(OUT/'before.png'),width=1400,height=1200);c.call('load_scene',path=relative(OUT/'wood.scene.json'));c.call('capture',path=relative(OUT/'after.png'),width=1400,height=1200)
  report['scene_pixels_equal']=(OUT/'before.png').read_bytes()==(OUT/'after.png').read_bytes();assert report['scene_pixels_equal']
  botany(kind='open',open=True,prefix='woodlab');botany(kind='select_organ',prefix='woodlab',organ='trunk');c.call('capture_editor',path=relative(OUT/'editor.png'),width=1600,height=1100)
  report['views']=['front','side','back','top','trunk-detail','base-detail','fork-detail','editor'];(OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
  (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Wood volume review</title><style>body{background:#222;color:#ddd;font:16px sans-serif}img{max-width:100%}</style><h1>Neutral woody volume authoring</h1><p>Native geometry. Artistic acceptance pending; no final elven tree.</p>'+''.join(f'<h2>{name}</h2><img src="{name}.png">' for name in report['views']))
  print(json.dumps(report['geometry']))
 finally:c.close()
if __name__=='__main__':main()
