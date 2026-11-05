"""Seed is geometry input: forty distinct native plateau bakes via MCP stdio."""
import argparse,copy,hashlib,json,random
from collections import Counter
from verify_lab import Client,ROOT
from open_earth_walk import init
from relief_compositions import ZERO,position
from directional_plateau import box
from patchy_plateau import joint,spall
from weathered_plateau import chip
from render_plateau_materials import grass
from render_plateau_amphitheatre import placements,GROUND_Y
from seeded_plateaus import controls,derive

OUT=ROOT/'artifacts/seeded-plateau-world';PREFIX='artifacts/seeded-plateau-world'
class Refused(Exception):pass

def fingerprint(g):
 return hashlib.sha256(json.dumps({k:g[k] for k in ['positions','triangles']},sort_keys=True,separators=(',',':')).encode()).hexdigest()

def normalized_footprint(g):
 # Read-only geometry diagnostic: remove translation/axis scale before hashing.
 edges=Counter(tuple(sorted([u,v])) for f in g['triangles'] for u,v in zip(f,f[1:]+f[:1]))
 points=[g['positions'][v] for edge,count in edges.items() if count==1 for v in edge]
 lo=[min(p[i] for p in points) for i in [0,2]];hi=[max(p[i] for p in points) for i in [0,2]]
 values=sorted({tuple(round((p[axis]-lo[j])/(hi[j]-lo[j]),6) for j,axis in enumerate([0,2])) for p in points})
 return hashlib.sha256(json.dumps(values).encode()).hexdigest()

def apply(c,name,operation):
 before=c.call('inspect_mesh',asset=name)['asset']['geometry']
 reference=c.call('inspect_mesh_topology',asset=name)['reference']
 r=c.request('tools/call',dict(name='engine_command',arguments=dict(command=dict(op='edit_mesh',edit=dict(kind='model_selection',asset=name,reference=reference,operation=operation)))))
 assert 'error' not in r,r;r=r['result']
 if r['isError']:
  assert c.call('inspect_mesh',asset=name)['asset']['geometry']==before
  return False,r.get('structuredContent',r)
 return True,None

def bake(c,world_seed,row,name):
 failures=[]
 for attempt in range(3):
  cfg=controls(world_seed,row,attempt);h=cfg['parameters']['height'];ops=[];refused=[]
  c.call('publish_landform',publication=dict(document=cfg['source'],asset=name,instance=name,frame_origin=ZERO,color=[1,1,1],shading='flat',collision=False))
  c.call('edit_mesh',edit=dict(kind='remove_instance',id=name))
  try:
   for points in [box([-220,h,-220],[220,220,220]),box([-220,-220,-220],[220,0,220])]:
    op=dict(kind='subtract_convex',points=points);ok,error=apply(c,name,op)
    if not ok:raise Refused(error)
    ops.append(op)
   rng=random.Random(derive(cfg['effective_seed'],'cuts'))
   for builder,count in [(joint,cfg['joint_count']),(spall,cfg['spall_count']),(chip,cfg['chip_count'])]:
    for _ in range(count):
     g=c.call('inspect_mesh',asset=name)['asset']['geometry'];op=dict(kind='subtract_convex',points=builder(g,rng));ok,error=apply(c,name,op)
     if ok:ops.append(op)
     else:refused.append(dict(operation=op,error=error))
   a=c.call('inspect_mesh',asset=name)['asset'];top=c.call('inspect_mesh_topology',asset=name)['topology']
   assert top['boundary_edges']==top['non_manifold_edges']==0
   assert len(c.call('inspect_mesh_islands',asset=name)['topology']['islands'])==1
   return a,dict(configuration=cfg,operations=ops,rejected_cuts=refused,rejected_candidates=failures,geometry_sha256=fingerprint(a['geometry']))
  except Refused as e:
   failures.append(dict(attempt=attempt,effective_seed=cfg['effective_seed'],error=e.args[0]))
   c.call('edit_mesh',edit=dict(kind='remove_asset',id=name))
 raise RuntimeError(f'All deterministic candidates refused: {row["id"]}: {failures}')

def extract_used(c,source,target):
 info=c.call('inspect_mesh_islands',asset=source)
 c.call('edit_mesh',edit=dict(kind='extract_islands',asset=source,id=target,reference=info['reference'],islands=[v['id'] for v in info['topology']['islands']],mode='copy'))
 c.call('edit_mesh',edit=dict(kind='remove_asset',id=source))
 return c.call('inspect_mesh',asset=target)['asset']

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--seed',type=int,default=81477);parser.add_argument('--count',type=int,default=40);args=parser.parse_args()
 if not 1<=args.count<=40 or not 0<=args.seed<2**64:parser.error('seed u64; count1..40')
 OUT.mkdir(parents=True,exist_ok=True);rows=placements(args.seed)[:args.count]
 for row in rows:row['scale']=1;row['position'][1]=GROUND_Y+row['support_depth']
 assert [controls(args.seed,r) for r in rows]==[controls(args.seed,r) for r in rows]
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log');records=[];stats=[];collision=[]
 approved=ROOT/'artifacts/patchy-plateau/plateau.scene.json';baseline=hashlib.sha256(approved.read_bytes()).hexdigest() if approved.exists() else None
 try:
  init(c);c.call('generate',recipe=dict(width=20,depth=20,seed=221,obstacles=0));c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in c.call('inspect')['scene']['entities']])
  floor=dict(version=1,space_version=1,id='floor',revision=1,datum=ZERO,base=dict(kind='constant',height_meters=GROUND_Y),operations=[])
  c.call('publish_terrain',publication=dict(document=floor,patch=dict(origin=position(-2048,0,-2048),cells=[8,8],spacing_meters=512),stride=1,asset='floor',instance='floor',frame_origin=ZERO,color=[.24,.27,.19],collision=True))
  material=json.loads((ROOT/'assets/materials/rock/alpine.ignis-material').read_text())['material'];material['procedural']['recipe']['tile_size']=[12,12]
  material['procedural']['recipe']['rock'].update(dark=[.14,.16,.19],light=[.43,.46,.49],contrast=.65,joint_depth=.035,grain=.4,grain_frequency=320,detail_height=.12)
  for row in rows:
   name=row['id'];work=name+'_work';a,record=bake(c,args.seed,row,work);h=record['configuration']['parameters']['height'];depth=row['support_depth']
   if depth:
    faces=[i for i,f in enumerate(a['geometry']['triangles']) if all(abs(a['geometry']['positions'][v][1])<.001 for v in f)]
    c.call('edit_mesh',edit=dict(kind='model_selection',asset=work,reference=c.call('inspect_mesh_topology',asset=work)['reference'],operation=dict(kind='extrude',faces=faces,offset=[0,-depth,0])))
   g=c.call('inspect_mesh',asset=work)['asset']['geometry'];roof=[i for i,f in enumerate(g['triangles']) if all(abs(g['positions'][v][1]-h)<.001 for v in f)];assert roof
   cap=c.call('inspect_mesh',asset=work)['asset'];cap['id']=name+'_cap';c.call('edit_mesh',edit=dict(kind='put_asset',asset=cap))
   c.call('edit_mesh',edit=dict(kind='remove_faces',asset=cap['id'],reference=c.call('inspect_mesh_topology',asset=cap['id'])['reference'],faces=[i for i in range(len(g['triangles'])) if i not in roof]))
   turf=extract_used(c,cap['id'],name+'_turf')
   c.call('edit_mesh',edit=dict(kind='set_material',asset=turf['id'],material=grass(record['configuration']['requested_seed']%65536)))
   c.call('edit_mesh',edit=dict(kind='remove_faces',asset=work,reference=c.call('inspect_mesh_topology',asset=work)['reference'],faces=roof))
   rock=extract_used(c,work,name+'_rock');g=rock['geometry']
   assert abs(min(p[1] for p in g['positions'])+depth)<.001
   edges=Counter()
   for geom in [g,turf['geometry']]:
    for f in geom['triangles']:
     points=[tuple(geom['positions'][v]) for v in f]
     for u,v in zip(points,points[1:]+points[:1]):edges[tuple(sorted([u,v]))]+=1
   assert all(n==2 for n in edges.values())
   c.call('edit_mesh',edit=dict(kind='set_material',asset=rock['id'],material=material))
   nx,_,nz=record['configuration']['source']['pieces'][0]['axis'];ny=record['configuration']['source']['pieces'][0]['axis'][1]
   c.call('edit_mesh',edit=dict(kind='set_material_coordinates',asset=rock['id'],coordinates=[[x+.4*z,y+(nx*x+nz*z)/ny,z] for x,y,z in g['positions']]))
   for suffix,asset in [('',rock['id']),('_turf',turf['id'])]:
    id=name+suffix;c.call('edit_mesh',edit=dict(kind='put_instance',instance=dict(id=id,asset=asset,position=row['position'],rotation=row['rotation'],scale=1,color=[1,1,1],playback=None)))
    collision.append(dict(op='set_mesh_collision',instance=id,enabled=True))
   c.call('edit_mesh',edit=dict(kind='put_object',object=dict(id=name+'_group',members=[name,name+'_turf'])))
   record['placement']=row;record['normalized_footprint_sha256']=normalized_footprint(turf['geometry']);records.append(record);stats.append(dict(id=name,seed=record['configuration']['requested_seed'],height=h,width=record['configuration']['parameters']['width'],depth=record['configuration']['parameters']['depth'],vertices=len(g['positions']),triangles=len(g['triangles']),turf_vertices=len(turf['geometry']['positions']),geometry_sha256=record['geometry_sha256']))
   print(json.dumps(dict(completed=len(stats),id=name,attempt=record['configuration']['attempt'],vertices=len(g['positions']))),flush=True)
  c.call('edit_scene',edits=collision)
  assert len({r['geometry_sha256'] for r in records})==len(rows)
  assert len({r['normalized_footprint_sha256'] for r in records})==len(rows)
  scene=c.call('inspect')['scene'];assert len(scene['meshes']['instances'])==len(rows)*2+1
  assert len(scene['collision_meshes'])==len(rows)*2+1
  light=dict(direction=[.4,-.75,.55],ambient=.25,direct_intensity=1.15,direct_color=[1,.96,.9],atmosphere=dict(density=.00008,color=[.61,.74,.84],reduce_details=False,sun_color=[1,.95,.85],sun_strength=.05));c.call('set_light',light=light)
  views=dict(overview=dict(position=[590,450,-640],target=[0,40,100],fov_degrees=54),valley=dict(position=[155,160,-540],target=[10,60,135],fov_degrees=65),reverse=dict(position=[-690,600,910],target=[0,40,75],fov_degrees=50))
  for name,view in views.items():c.call('set_camera',camera=view);c.call('capture',path=f'{PREFIX}/{name}.png',width=1600,height=1000)
  c.call('set_camera',camera=views['overview']);c.call('save_scene',path=f'{PREFIX}/world.scene.json');c.call('load_scene',path=f'{PREFIX}/world.scene.json');c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=1000);assert (OUT/'reload.png').read_bytes()==(OUT/'overview.png').read_bytes()
  c.call('step',ticks=2);c.call('save_replay',path=f'{PREFIX}/world.replay.json');replay=c.call('run_replay',path=f'{PREFIX}/world.replay.json');assert replay['matched'] and replay['meshes_matched']
  (OUT/'bake-records.json').write_text(json.dumps(dict(version=1,world_seed=args.seed,records=records),indent=2)+'\n')
  inputs=dict(version=1,world_seed=args.seed,count=len(rows),expected_geometry=[dict(id=r['placement']['id'],seed=r['configuration']['requested_seed'],effective_seed=r['configuration']['effective_seed'],geometry_sha256=r['geometry_sha256']) for r in records],light=light,views=views)
  (ROOT/'examples/seeded-plateau-world.recipe.json').write_text(json.dumps(inputs,indent=2)+'\n')
  report=dict(build=c.call('inspect')['build_id'],world_seed=args.seed,count=len(rows),unique_native_geometry=len(rows),unique_normalized_footprints=len(rows),stats=stats,paired_edges=True,reload_pixels_equal=True,replay_matched=True,human_acceptance=False)
 finally:c.close()
 # Native regeneration proof in an independent session; full saved world unchanged.
 probe=Client(ROOT/'target/release/ignis',log_path=OUT/'seed-proof.log')
 try:
  init(probe);report['proof_build']=probe.call('inspect')['build_id'];probe.call('generate',recipe=dict(width=20,depth=20,seed=221,obstacles=0))
  _,same=bake(probe,args.seed,rows[0],'same_probe');_,other=bake(probe,(args.seed+1)%2**64,rows[0],'other_probe')
  assert same['geometry_sha256']==records[0]['geometry_sha256'] and same['operations']==records[0]['operations']
  assert other['geometry_sha256']!=same['geometry_sha256']
  report.update(same_seed_native_exact=True,changed_seed_native_different=True)
 finally:probe.close()
 if baseline:assert hashlib.sha256(approved.read_bytes()).hexdigest()==baseline
 (OUT/'verification.json').write_text(json.dumps(report,indent=2));print(json.dumps({k:v for k,v in report.items() if k!='stats'}))
if __name__=='__main__':main()
