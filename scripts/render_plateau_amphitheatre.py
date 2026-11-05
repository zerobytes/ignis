"""Forty instanced approved plateaus in five levels, authored solely through MCP."""
import copy,hashlib,json,math,random
from collections import Counter
from verify_lab import Client,ROOT
from open_earth_walk import init

OUT=ROOT/'artifacts/plateau-amphitheatre';PREFIX='artifacts/plateau-amphitheatre'
BASE='artifacts/patchy-plateau/plateau.scene.json'
GROUND_Y=-1.5

def placements(seed=81477):
 rng=random.Random(seed);result=[]
 for level,depth in enumerate([0,18,38,60,84]):
  z=-170+level*125;valley=30*math.sin(level*.85)
  for side in [-1,1]:
   for column in range(4):
    x=valley+side*(80+column*120)+rng.uniform(-25,25)+20*math.sin(level*1.3)
    zz=z+rng.uniform(-30,30)+.00035*x*x+18*math.sin(column*.9+level*.7)
    scale=rng.uniform(.88,1.12);angle=rng.uniform(-.45,.45)
    result.append(dict(id=f'p{level}_{side+1}_{column}',level=level,support_depth=depth,
      position=[x,GROUND_Y+depth*scale,zz],scale=scale,rotation=[0,math.sin(angle/2),0,math.cos(angle/2)]))
 return result

def main():
 OUT.mkdir(parents=True,exist_ok=True);source=ROOT/BASE
 if not source.exists():raise RuntimeError('Generate approved patchy-plateau scene first')
 before=hashlib.sha256(source.read_bytes()).hexdigest();rows=placements();assert rows==placements()
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log')
 try:
  init(c);c.call('load_scene',path=BASE)
  floor=c.call('inspect_mesh',asset='floor')['asset']['geometry']
  assert all(abs(p[1]-GROUND_Y)<1e-6 for p in floor['positions'])
  rock=c.call('inspect_mesh',asset='plateau')['asset'];turf=c.call('inspect_mesh',asset='turf')['asset']
  c.call('edit_scene',edits=[dict(op='set_mesh_collision',instance='plateau',enabled=False)])
  for name in ['plateau','turf']:c.call('edit_mesh',edit=dict(kind='remove_instance',id=name))
  assets=[]
  for level,depth in enumerate([0,18,38,60,84]):
   name=f'rock_level{level}';a=copy.deepcopy(rock);a['id']=name
   c.call('edit_mesh',edit=dict(kind='put_asset',asset=a))
   if depth:
    faces=[i for i,f in enumerate(a['geometry']['triangles']) if all(abs(a['geometry']['positions'][v][1])<.001 for v in f)]
    assert faces
    c.call('edit_mesh',edit=dict(kind='model_selection',asset=name,
      reference=c.call('inspect_mesh_topology',asset=name)['reference'],
      operation=dict(kind='extrude',faces=faces,offset=[0,-depth,0])))
    actual=c.call('inspect_mesh',asset=name)['asset']['geometry']
    upper={tuple(p) for p in actual['positions']}
    assert all(tuple(p) in upper for p in rock['geometry']['positions'] if p[1]>.001)
    def wall_faces(g):
     return {tuple(sorted(tuple(g['positions'][v]) for v in f)) for f in g['triangles']
             if any(g['positions'][v][1]>.001 for v in f)}
    assert wall_faces(rock['geometry'])<=wall_faces(actual)
    c.call('edit_mesh',edit=dict(kind='set_material_coordinates',asset=name,
      coordinates=[[x+.4*z,y+.34*x+.08*z,z] for x,y,z in actual['positions']]))
   top=c.call('inspect_mesh_topology',asset=name)['topology'];assert top['boundary_edges']==top['non_manifold_edges']==0
   # Cap is rendered exactly once, by the shared turf mesh; no coplanar overlay.
   built=c.call('inspect_mesh',asset=name)['asset']['geometry']
   roof=[i for i,f in enumerate(built['triangles']) if all(abs(built['positions'][v][1]-34)<.001 for v in f)]
   c.call('edit_mesh',edit=dict(kind='remove_faces',asset=name,reference=c.call('inspect_mesh_topology',asset=name)['reference'],faces=roof))
   built=c.call('inspect_mesh',asset=name)['asset']['geometry']
   edges=Counter()
   for geometry in [built,turf['geometry']]:
    for face in geometry['triangles']:
     points=[tuple(geometry['positions'][i]) for i in face]
     for u,v in zip(points,points[1:]+points[:1]):edges[tuple(sorted([u,v]))]+=1
   assert all(count==2 for count in edges.values()),name
   assets.append(dict(id=name,support_depth=depth,triangles=len(built['triangles']),vertices=len(built['positions'])))
  collision=[]
  for row in rows:
   name=row['id'];pos=row['position'];scale=row['scale']
   instance=dict(id=name,asset=f"rock_level{row['level']}",position=pos,rotation=row['rotation'],scale=scale,color=[1,1,1],playback=None)
   c.call('edit_mesh',edit=dict(kind='put_instance',instance=instance))
   green=copy.deepcopy(instance);green.update(id=name+'_turf',asset='turf',position=pos)
   c.call('edit_mesh',edit=dict(kind='put_instance',instance=green))
   c.call('edit_mesh',edit=dict(kind='put_object',object=dict(id=name+'_group',members=[name,green['id']])))
   collision.extend([dict(op='set_mesh_collision',instance=id,enabled=True) for id in [name,green['id']]])
   assert abs(pos[1]-row['support_depth']*scale-GROUND_Y)<1e-8
  c.call('edit_scene',edits=collision)
  # Drop unused original rock only after the forty new paired instances exist.
  c.call('edit_mesh',edit=dict(kind='remove_asset',id='plateau'))
  light=dict(direction=[.4,-.75,.55],ambient=.25,direct_intensity=1.15,direct_color=[1,.96,.9],
    atmosphere=dict(density=.00008,color=[.61,.74,.84],reduce_details=False,sun_color=[1,.95,.85],sun_strength=.05))
  c.call('set_light',light=light)
  views=dict(overview=dict(position=[590,450,-640],target=[0,40,100],fov_degrees=54),
    valley=dict(position=[155,160,-540],target=[10,60,135],fov_degrees=65),
    reverse=dict(position=[-690,600,910],target=[0,40,75],fov_degrees=50))
  for name,view in views.items():c.call('set_camera',camera=view);c.call('capture',path=f'{PREFIX}/{name}.png',width=1600,height=1000)
  c.call('set_camera',camera=views['overview']);c.call('save_scene',path=f'{PREFIX}/world.scene.json')
  snapshot=c.call('inspect_meshes');assert len(snapshot['instances'])==81
  assert len(c.call('inspect')['scene']['collision_meshes'])==81
  c.call('load_scene',path=f'{PREFIX}/world.scene.json');c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=1000)
  assert (OUT/'overview.png').read_bytes()==(OUT/'reload.png').read_bytes()
  c.call('step',ticks=2);c.call('save_replay',path=f'{PREFIX}/world.replay.json');r=c.call('run_replay',path=f'{PREFIX}/world.replay.json');assert r['matched'] and r['meshes_matched']
  assert hashlib.sha256(source.read_bytes()).hexdigest()==before
  recipe=dict(seed=81477,source_scene=BASE,source_sha256=before,placements=rows,light=light,views=views)
  (ROOT/'examples/plateau-amphitheatre.recipe.json').write_text(json.dumps(recipe,indent=2)+'\n')
  report=dict(build=c.call('inspect')['build_id'],plateaus=40,levels=5,per_level=8,assets=assets,instances=81,
    support_to_ground=True,upper_geometry_preserved=True,paired_edge_incidence_two=True,collision_instances=81,source_preserved=True,reload_pixels_equal=True,replay_matched=True,native_window=False,human_acceptance=False)
  (OUT/'verification.json').write_text(json.dumps(report,indent=2));print(json.dumps(report))
 finally:c.close()
if __name__=='__main__':main()
