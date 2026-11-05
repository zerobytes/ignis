#!/usr/bin/env python3
"""Editable ancestral-tree study. Author source graphs; all meshing/rendering is native Ignis."""
import argparse,hashlib,json,math,time
from collections import Counter
from verify_lab import Client,ROOT
import botany_math as b
from botany_exemplars import Builder,curved,spline,partition,mean,chronological,radius_at
OUT=ROOT/'artifacts/ancestral-regeneration'
CONFIG=ROOT/'examples/ancestral-study.recipe.json'

def integrity(scene,prefix):
 g=next(a['geometry'] for a in scene['meshes']['assets'] if a['id']==prefix+'_wood')
 edges=Counter();sign=Counter();volume=0.;minimum_area=math.inf
 for a,b_,c in g['triangles']:
  p,q,r=[g['positions'][i] for i in (a,b_,c)]
  cross=b.cross(q,r);volume+=b.dot(p,cross)/6
  minimum_area=min(minimum_area,b.norm(b.cross(b.sub(q,p),b.sub(r,p)))/2)
  for i,j in [(a,b_),(b_,c),(c,a)]:
   key=(min(i,j),max(i,j));edges[key]+=1;sign[key]+=1 if i<j else -1
 assert volume>0 and minimum_area>0 and all(n==2 and sign[e]==0 for e,n in edges.items())
 return dict(vertices=len(g['positions']),triangles=len(g['triangles']),closed_oriented=True,signed_volume_m3=volume,minimum_triangle_area_m2=minimum_area,global_self_intersection_tested=False)

def bark_recipe():
 def sample(color,height):return dict(color_roughness=color+[.96],response=[0,height,1,1],emission=[0,0,0,0])
 layers=[]
 for seed,field,freq,color,height,strength in [(12,'noise',[8,4],[.23,.205,.17],.25,.8),(33,'ridged',[32,5],[.40,.36,.29],.75,.85),(51,'noise',[64,16],[.5,.46,.38],.6,.28)]:
  layers.append(dict(field=field,frequency=freq,seed=seed,octaves=4,warp=.35,threshold=[.2,.85],strength=strength,invert=False,target=sample(color,height)))
 return dict(seed=912,unique_instances=False,plane='xy',tile_size=[4,12],base=sample([.12,.105,.085],.15),layers=layers,height_m=.008,relief='bump',normal_filter='pixel',view_steps=16,shadow_steps=8)

def review():
 cards=''.join(f'<section><h2>{name}</h2><div><figure><img src="{name}-structure.png"><figcaption>Native geometry, no leaves or bark material</figcaption></figure><figure><img src="{name}-full.png"><figcaption>Same scaffold with individual closed leaves</figcaption></figure></div></section>' for name in ['front','side','back','top','base','arms'])
 (OUT/'review.html').write_text('''<!doctype html><meta charset="utf-8"><title>Ancestral tree study</title><style>body{background:#20252b;color:#eee;font:16px system-ui;max-width:1600px;margin:24px auto;padding:20px}img{width:100%;height:auto}div{display:flex;gap:16px}figure{margin:0;flex:1;min-width:0}figcaption{padding:12px}a{color:#abd9ff}section{margin:40px 0}@media(max-width:800px){div{display:block}}</style><h1>Ancestral tree · native Ignis study</h1><p>Editable seeded graph, native geometry, material and renderer. Authorial mature snapshot at 180 years; no live growth grammar or performance claim. Review study, not an accepted or installed game asset.</p><img src="hero.png"><p>Large junctions remain angular, roots need stronger structural relief, and canopy continuity still differs from the reference. Sheath-connected auxiliary arms are closed overlapping components, not a welded union. Bark is bump material, not geometric fibres.</p><p><a href="ancestral-study.ignis-botany">Editable source</a> · <a href="current.scene.json">Complete scene</a> · <a href="bark.ignis-material">Native bark material</a> · <a href="bark.recipe.json">Material recipe</a> · <a href="full-verification.json">Validation</a></p><h2>Approved concept reference</h2><img src="../../games/land-of-survivors/references/ancient-white-elven-tree.png"><p>The concept image above is a reference, not an Ignis render. The captures below are actual native renders.</p>'''+cards)

def graph(cfg,foliage):
 x=Builder('Árvore ancestral branca · estudo revisado',cfg['seed']);x.axis_samples=24;edits={}
 def cache(ident):
  o=x.by_id[ident];rows=[];old=None
  for i in range(25):
   p,t=spline(o['path'],i/24);n=b.normal_to(t,o['normal']) if old is None else b.transport(old.n,old.t,t)
   old=b.Sample(p,t,n);rows.append(old)
  x.samples[ident]=rows
 for index,row in enumerate(cfg['scaffold']):
  points=row['path'];parent=row['parent'];start=points[0];d=b.sub(points[1],start)
  curved(x,row['id'],parent,row.get('insertion',1) if parent else 0,points[-1],d,row['radius'],tip=row['tip'],taper=1.1,arch=0,noise=0)
  o=x.by_id[row['id']];o['path']=points
  if parent:o['connection']=row.get('connection','fork')
  cache(row['id'])
  phase=index*.63;sections=[];strength=row.get('shape_strength',1.)
  for k,(u,gain) in enumerate(zip([0,.16,.4,.7,1],row.get('gains',[1.05,1.03,1.04,1.,1.]))):
   r=row['radius']*(row['tip']+(1-row['tip'])*(1-u)**1.1)*gain
   sections.append(dict(id=f's{k}',u=u,angle=.25*u,offset=[.03*strength*row['radius']*math.sin(u*math.pi),-.025*strength*row['radius']*math.sin(u*math.pi)],radii=[(r*(1+.09*math.cos(3*a+phase+.5*u)+.04*math.sin(5*a-phase))*strength+row['radius']*(row['tip']+(1-row['tip'])*(1-u)**1.1)*(1-strength)) for a in [math.tau*j/16 for j in range(16)]]))
  relief=[]
  for j in range(7 if index==0 else 4):
   a=j*math.tau/(7 if index==0 else 4)+phase
   relief.append(dict(id=f'cord{j}',enabled=True,path=[dict(u=u,angle=a+.65*u+.17*math.sin(6*u+j)) for u in [0,.2,.45,.75,1]],width=row['radius']*.26,height=row['cord_height']*strength,end_fade=.1,follow_taper=True))
  if index==0:
   for j in range(4):relief.append(dict(id=f'valley{j}',enabled=True,path=[dict(u=u,angle=j*1.55+.45+.65*u) for u in [0,.4,.7,1]],width=.62,height=-.40*strength,end_fade=.12,follow_taper=True))
  edits[row['id']]=dict(wood_surface=dict(sections=sections,relief=relief,sides=32 if index<7 else 24,samples=96 if index==0 else 48),junction_refinement=2)
  if not row.get('surface',True):edits[row['id']].pop('wood_surface')
 tips=[]
 def grow(parent,points,depth=0):
  if len(points)==1:tips.append(parent);return
  groups=partition(points,.43+.14*b.uniform(x.seed,parent,'partition'));f=x.sample(parent,1)
  split=b.normal_to(f.t,b.sub(mean(groups[1]),mean(groups[0])))
  for j,group in enumerate(groups):
   key=f'{parent}/b{j}';goal=mean(group);aim=b.unit(b.sub(goal,f.p))
   d=b.unit(b.lerp(b.add(b.mul(f.t,.65),b.mul(split,(1 if j else -1)*.8)),aim,.72))
   if b.dot(d,f.t)<.12:d=b.unit(b.add(b.mul(f.t,.15),b.normal_to(f.t,d)))
   r=radius_at(x,parent,1)*(len(group)/len(points))**(1/2.3)
   distance=b.norm(b.sub(goal,f.p));length=max(r*8,min(3.5,max(.8,distance*.62)))
   end=b.add(f.p,b.mul(d,length))
   curved(x,key,parent,1,end,d,r,tip=.78 if len(group)>1 else .27,taper=1.1,arch=.12,noise=.10,handle=.29)
   x.by_id[key]['connection']='fork';grow(key,group,depth+1)
 for i,(host,lobe) in enumerate(zip(cfg['hosts'],cfg['lobes'])):
  center,radii=lobe;points=[]
  for k in range(cfg['attractors_per_lobe']):
   z=1-2*(k+.5)/cfg['attractors_per_lobe'];az=k*2.399963+i*.8;disk=math.sqrt(1-z*z);q=.7+.3*b.uniform(x.seed,f'{host}/{k}','cloud')
   points.append([center[0]+radii[0]*disk*math.cos(az)*q,center[1]+radii[1]*z*q,center[2]+radii[2]*disk*math.sin(az)*q])
  grow(host,points)
 def foliate(shoot,count,phase):
  for k in range(count):
   t=.05+.9*(k+.5)/count;sf=x.sample(shoot,t);a=k*2.399963+phase
   direction=b.unit(b.add(b.add(b.mul(sf.n,math.cos(a)),b.mul(sf.b,math.sin(a))),b.mul(sf.t,.25)))
   direction=b.unit(b.add(direction,(0,.2-.25*b.uniform(x.seed,f'{shoot}/{k}','tilt'),0)))
   size=.75+.5*b.uniform(x.seed,f'{shoot}/{k}','leaf_size');length=cfg['leaf_length_m']*size
   normal=b.normal_to(direction,b.add(b.UP,b.mul(sf.t,.3)))
   x.organs.append(dict(id=f'{shoot}/l{k}',parent=shoot,insertion=t,path=[sf.p,b.add(sf.p,b.mul(direction,length))],normal=normal,shape=dict(kind='blade',width=cfg['leaf_width_m']*size,thickness=.00025,fold=.05,cup=.06,outline=dict(kind='strip',base_power=.7,tip_power=1.1)),birth=0,duration=1,density=850,young=2e7,shear=8e6))
 # Terminal shoots keep a complete native branch hierarchy.
 for parent in tips:
  phase=math.tau*b.uniform(x.seed,parent,'shoot_phase')
  for j in range(2):
   u=.48+.32*j;f=x.sample(parent,u);az=phase+j*2.4;d=b.unit(b.add(b.add(b.mul(f.n,math.cos(az)),b.mul(f.b,math.sin(az))),b.mul(f.t,.35)))
   length=.85+.45*b.uniform(x.seed,f'{parent}/{j}','shoot_length');shoot=f'{parent}/shoot{j}'
   curved(x,shoot,parent,u,b.add(f.p,b.mul(d,length)),d,min(.018,radius_at(x,parent,u)*.42),tip=.18,arch=.08,noise=.04)
   if foliage:foliate(shoot,cfg['leaves_per_shoot'],phase)
 # Epicormic shoots distributed along old arms fill the crown's interior.
 # Their basal closed shells overlap the support inside its bark, using the
 # native sheath attachment; this is not an anastomosed boolean union.
 for row in cfg['scaffold'][1:] if cfg['interior_leaves'] else []:
  parent=row['id']
  for k in range(8):
   u=.22+.10*(k//2)*2;f=x.sample(parent,u);a=math.tau*b.uniform(x.seed,f'{parent}/{k}','interior_azimuth')
   outward=b.unit(b.add(b.mul(f.n,math.cos(a)),b.mul(f.b,math.sin(a))))
   d=b.unit(b.add(b.mul(outward,.65),b.add((0,1.15,0),b.mul(f.t,.15))))
   length=2.3+2.7*b.uniform(x.seed,f'{parent}/{k}','interior_length');ident=f'{parent}/inner{k}'
   curved(x,ident,parent,u,b.add(f.p,b.mul(d,length)),d,.045,tip=.15,arch=.24,noise=.12)
   x.by_id[ident]['connection']='sheath'
   # Place the base slightly inside the source bark; retain the authored offset.
   delta=b.mul(outward,radius_at(x,parent,u)*.9)
   x.by_id[ident]['path']=[b.add(p,delta) for p in x.by_id[ident]['path']];cache(ident)
   if foliage and cfg['interior_leaves']:foliate(ident,cfg['interior_leaves'],a)
 r=x.recipe(cfg['prefix'],bark_color=[.37,.37,.37],leaf_color=[.86,.88,.83])
 r=chronological(r,'figueira',cfg['age_years']);r['source']['development'].update(reference_age_years=cfg['age_years'],height_time_years=30,crown_time_years=40,radial_time_years=70,basal_flare=0,senescent_foliage_loss=0)
 r.update(version=12,edits=edits);r['source']['version']=9;r['settings'].update(axis_samples=12,sides=12,blade_samples=4)
 return r

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--structure',action='store_true');p.add_argument('--open',action='store_true');args=p.parse_args();OUT.mkdir(parents=True,exist_ok=True)
 if args.open:
  c=Client(ROOT/'target/release/ignis',window=True,detached=True,log_path=OUT/'window.log',startup_args=('--editor','--scene','artifacts/ancestral-regeneration/current.scene.json'))
  try:
   c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='ancestral-study',version='1')));c.request('notifications/initialized',notification=True)
   c.call('botany',operation=dict(kind='open',open=True,prefix='ancestral_study'));c.call('botany',operation=dict(kind='select_organ',prefix='ancestral_study',organ='trunk'))
   out=dict(pid=c.process.pid,build=c.call('inspect')['build_id']);(OUT/'window.json').write_text(json.dumps(out,indent=2));print(json.dumps(out))
  except BaseException:c.close();raise
  else:c.process.stdin.close();c.process.stdout.close();c.log.close()
  return
 cfg=json.loads(CONFIG.read_text());start=time.monotonic();r=graph(cfg,not args.structure);print(json.dumps(dict(authored_organs=len(r['source']['organs']),seconds=time.monotonic()-start)),flush=True)
 (OUT/'candidate.ignis-botany').write_text(json.dumps(r,separators=(',',':'))+'\n')
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'runtime.log');report=dict(stage='structure' if args.structure else 'foliage',reference='games/land-of-survivors/references/ancient-white-elven-tree.png',installed_in_game=False)
 def bot(**op):
  t=time.monotonic();result=c.call('botany',operation=op);print(json.dumps(dict(operation=op['kind'],seconds=time.monotonic()-t)),flush=True);return result
 def capture(name,eye,target,fov=40):
  c.call('set_camera',camera=dict(position=eye,target=target,fov_degrees=fov));c.call('capture',path=f'artifacts/ancestral-regeneration/{name}.png',width=1800,height=1200)
 try:
  c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='ancestral-study',version='1')));c.request('notifications/initialized',notification=True)
  from demo_groundcover import setup
  report['build']=setup(c)
  report['scaffold']=bot(kind='load',path='artifacts/ancestral-regeneration/candidate.ignis-botany',publish=True)
  report['roots']=bot(kind='set_roots',prefix=cfg['prefix'],stem='trunk',settings=cfg['roots'])
  report['details']=bot(kind='set_wood_details',prefix=cfg['prefix'],details=dict(seed=911,age_years=cfg['age_years'],healing_years=30,roots=dict(bulges=7,swelling=.24,flattening=.14,bend=.16,twist=.1,grain=.06),scars=[]))
  c.call('set_light',light=dict(direction=[-.45,-.78,-.38],ambient=.30,direct_intensity=1.3,environment=dict(sky=[.54,.56,.58],horizon=[.5]*3,ground=[.22]*3,axis=[0,1,0],intensity=.35),shadow=dict(center=[0,9,0],extent=42,resolution=2048,bias=.0008)))
  ground=dict(id='ancestral_ground',geometry=dict(positions=[[-100,.035,-100],[100,.035,-100],[-100,.035,100],[100,.035,100]],triangles=[[0,2,1],[1,2,3]]))
  c.call('edit_scene',edits=[dict(op='mesh',edit=dict(kind='put_asset',asset=ground)),dict(op='mesh',edit=dict(kind='put_instance',instance=dict(id='ancestral_ground',asset='ancestral_ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.22,.23,.21])))])
  views=dict(front=([3,14,70],[0,13,0],40),side=([70,17,3],[0,13,0],42),back=([-3,14,-70],[0,13,0],42),top=([2,76,4],[0,5,0],43),base=([15,6,20],[0,3,0],43),arms=([8,13,24],[0,8,0],45))
  suffix='structure' if args.structure else 'full'
  for name,argsview in views.items():capture(f'{name}-{suffix}',*argsview)
  bot(kind='save',prefix=cfg['prefix'],path=f'artifacts/ancestral-regeneration/{suffix}.ignis-botany')
  scene_name='structure.scene.json' if args.structure else 'current.scene.json'
  capture('current',*views['front']);c.call('save_scene',path=f'artifacts/ancestral-regeneration/{scene_name}');c.call('load_scene',path=f'artifacts/ancestral-regeneration/{scene_name}');capture('reopened',*views['front']);assert (OUT/'current.png').read_bytes()==(OUT/'reopened.png').read_bytes()
  report.update(save_load_png_exact=True,cameras=views,source_bytes=(OUT/f'{suffix}.ignis-botany').stat().st_size)
  report['wood_integrity']=integrity(json.loads((OUT/scene_name).read_text()),cfg['prefix'])
  if not args.structure:
   appearance=json.loads((OUT/'full.ignis-botany').read_text());appearance['settings']['bark_color']=[1,1,1]
   # Full source exceeds the MCP command envelope; use the native local loader.
   (OUT/'appearance.ignis-botany').write_text(json.dumps(appearance,separators=(',',':'))+'\n')
   bot(kind='load',path='artifacts/ancestral-regeneration/appearance.ignis-botany',publish=True)
   material=bark_recipe();(OUT/'bark.recipe.json').write_text(json.dumps(material,indent=2)+'\n')
   c.call('material',operation=dict(kind='generate',asset=cfg['prefix']+'_wood',recipe=material,execution=dict(fixed=dict(resolution=512))))
   c.call('material',operation=dict(kind='save',asset=cfg['prefix']+'_wood',path='artifacts/ancestral-regeneration/bark.ignis-material'))
   bot(kind='save',prefix=cfg['prefix'],path='artifacts/ancestral-regeneration/ancestral-study.ignis-botany')
   c.call('set_light',light=dict(direction=[-.55,-.72,-.42],ambient=.20,direct_intensity=1.1,direct_color=[1,.95,.84],environment=dict(sky=[.48,.6,.77],horizon=[.56,.6,.63],ground=[.16,.20,.11],axis=[0,1,0],intensity=.45),atmosphere=dict(density=.0005,reduce_details=False,color=[.35,.47,.62],sun_color=[1,.92,.78],sun_strength=.1),shadow=dict(center=[0,10,0],extent=42,resolution=2048,bias=.0008)))
   c.call('edit_scene',edits=[dict(op='mesh',edit=dict(kind='put_instance',instance=dict(id='ancestral_ground',asset='ancestral_ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.12,.16,.08])))])
   c.call('set_camera',camera=dict(position=[7,12,65],target=[0,13,0],fov_degrees=42))
   c.call('capture',path='artifacts/ancestral-regeneration/hero.png',width=2048,height=1365)
   c.call('save_scene',path='artifacts/ancestral-regeneration/current.scene.json')
   c.call('load_scene',path='artifacts/ancestral-regeneration/current.scene.json')
   c.call('capture',path='artifacts/ancestral-regeneration/hero-reopened.png',width=2048,height=1365)
   assert (OUT/'hero.png').read_bytes()==(OUT/'hero-reopened.png').read_bytes()
   bot(kind='load',path='artifacts/ancestral-regeneration/ancestral-study.ignis-botany',publish=True)
   c.call('capture',path='artifacts/ancestral-regeneration/hero-regenerated.png',width=2048,height=1365)
   assert (OUT/'hero.png').read_bytes()==(OUT/'hero-regenerated.png').read_bytes()
   report.update(presentation_save_load_png_exact=True,source_sha256=hashlib.sha256((OUT/'ancestral-study.ignis-botany').read_bytes()).hexdigest())
   report['source_regeneration_png_exact']=True
   if (OUT/'structure.scene.json').exists():
    def wood_hash(path):
     scene=json.loads(path.read_text());g=next(a['geometry'] for a in scene['meshes']['assets'] if a['id']==cfg['prefix']+'_wood')
     return hashlib.sha256(json.dumps(g,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    neutral=wood_hash(OUT/'structure.scene.json');presented=wood_hash(OUT/'current.scene.json')
    assert neutral==presented,'Regenerate --structure after editing the configuration.'
    report.update(neutral_scaffold_geometry_exact=True,wood_geometry_sha256=presented)
  (OUT/f'{suffix}-verification.json').write_text(json.dumps(report,indent=2)+'\n')
  if not args.structure:review()
  for temporary in ['candidate.ignis-botany','appearance.ignis-botany']:(OUT/temporary).unlink(missing_ok=True)
  print(json.dumps(report['details']['compiled']),flush=True)
 finally:c.close()
if __name__=='__main__':main()
