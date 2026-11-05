"""Full plateau, layered volumes and geometric weathering via native MCP tools.
Cold authoring: capped additions/cuts, no frame-time generation or new passes.
"""
import copy,hashlib,json,random
from verify_lab import Client,ROOT
from open_earth_walk import init
from relief_compositions import ZERO,position
from render_plateau_materials import grass
from weathered_plateau import chip,edge_chip
from directional_plateau import formation,box,fractures
OUT=ROOT/'artifacts/directional-plateau';PREFIX='artifacts/directional-plateau'
def main():
 OUT.mkdir(parents=True,exist_ok=True);c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log');operations=[];rejections=[]
 try:
  init(c);assert 'union_convex' in json.dumps(c.request('tools/list')['result'])
  c.call('generate',recipe=dict(width=20,depth=20,seed=221,obstacles=0));c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in c.call('inspect')['scene']['entities']])
  params=dict(id='directional',seed=4721,width=104,depth=78,height=34,contour_points=20,rock_count=1)
  source=formation(params)
  c.call('publish_landform',publication=dict(document=source,asset='plateau',instance='plateau',frame_origin=ZERO,color=[.65,.65,.65],shading='flat',collision=True))
  def asset():return c.call('inspect_mesh',asset='plateau')['asset']
  def apply(kind,points):
   before=asset()['geometry'];ref=c.call('inspect_mesh_topology',asset='plateau')['reference'];op=dict(kind=kind,points=points)
   response=c.request('tools/call',dict(name='engine_command',arguments=dict(command=dict(op='edit_mesh',edit=dict(kind='model_selection',asset='plateau',reference=ref,operation=op)))))
   assert 'error' not in response,response
   result=response['result']
   if result['isError']:
    assert asset()['geometry']==before
    rejections.append(dict(operation=op,error=result));print(json.dumps(result),flush=True);return False
   operations.append(op);return True
  assert apply('subtract_convex',box([-180,34,-180],[180,160,180]))
  assert apply('subtract_convex',box([-180,-160,-180],[180,0,180]))
  for pts in fractures():apply('subtract_convex',pts)
  rng=random.Random(27319)
  for i in range(12):apply('subtract_convex',edge_chip(asset()['geometry'],rng))
  for i in range(14):apply('subtract_convex',chip(asset()['geometry'],rng))
  topology=c.call('inspect_mesh_topology',asset='plateau')['topology'];assert topology['boundary_edges']==topology['non_manifold_edges']==0
  islands=c.call('inspect_mesh_islands',asset='plateau')['topology']['islands'];assert len(islands)==1
  a=asset();g=a['geometry']
  views=dict(overview=dict(position=[114,80,-130],target=[0,16,0],fov_degrees=45),detail=dict(position=[87,37,-100],target=[25,18,-18],fov_degrees=43),rear=dict(position=[-110,75,130],target=[0,16,0],fov_degrees=47),top=dict(position=[0,210,3],target=[0,0,0],fov_degrees=44))
  c.call('set_light',light=dict(direction=[.45,-.75,.55],ambient=.26,direct_intensity=1.15,direct_color=[1,.96,.88],shadow=dict(center=[0,17,0],extent=160,resolution=2048,bias=.0004,receiver_plane_bias=True),atmosphere=dict(density=.0002,color=[.58,.71,.82],reduce_details=False,sun_color=[1,.95,.85],sun_strength=.05)))
  c.call('set_camera',camera=views['overview']);c.call('capture',path=f'{PREFIX}/neutral.png',width=1440,height=960)
  material=copy.deepcopy(json.loads((ROOT/'assets/materials/rock/alpine.ignis-material').read_text())['material']);material['procedural']['recipe']['tile_size']=[12,12];material['procedural']['recipe']['rock'].update(dark=[.14,.16,.19],light=[.43,.46,.49],contrast=.65,joint_depth=.035,grain=.4,grain_frequency=320,detail_height=.12)
  c.call('edit_mesh',edit=dict(kind='set_material',asset='plateau',material=material));c.call('edit_mesh',edit=dict(kind='set_material_coordinates',asset='plateau',coordinates=[[x+.4*z, y+.34*x+.08*z,z] for x,y,z in g['positions']]))
  inst=copy.deepcopy(next(i for i in c.call('inspect_meshes')['instances'] if i['id']=='plateau'));inst['color']=[1,1,1];c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
  roof=[i for i,f in enumerate(g['triangles']) if all(abs(g['positions'][v][1]-34)<.001 for v in f)]
  if roof:
   turf=copy.deepcopy(a);turf['id']='turf';turf['material']=grass(4721);turf['material']['pattern'].update(low=[.09,.18,.035],high=[.2,.3,.065],bump=.002)
   c.call('edit_mesh',edit=dict(kind='put_asset',asset=turf));c.call('edit_mesh',edit=dict(kind='remove_faces',asset='turf',reference=c.call('inspect_mesh_topology',asset='turf')['reference'],faces=[i for i in range(len(g['triangles'])) if i not in roof]));inst=copy.deepcopy(next(i for i in c.call('inspect_meshes')['instances'] if i['id']=='plateau'));inst.update(id='turf',asset='turf',position=[0,.15,0]);c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
  floor=dict(version=1,space_version=1,id='floor',revision=1,datum=ZERO,base=dict(kind='constant',height_meters=-1.5),operations=[])
  c.call('publish_terrain',publication=dict(document=floor,patch=dict(origin=position(-4096,0,-4096),cells=[8,8],spacing_meters=1024),stride=1,asset='floor',instance='floor',frame_origin=ZERO,color=[.24,.27,.19],collision=True))
  for name,view in views.items():c.call('set_camera',camera=view);c.call('capture',path=f'{PREFIX}/{name}.png',width=1440,height=960)
  c.call('set_camera',camera=views['overview']);c.call('save_scene',path=f'{PREFIX}/plateau.scene.json');c.call('load_scene',path=f'{PREFIX}/plateau.scene.json');c.call('capture',path=f'{PREFIX}/reload.png',width=1440,height=960);assert (OUT/'reload.png').read_bytes()==(OUT/'overview.png').read_bytes()
  c.call('step',ticks=2);c.call('save_replay',path=f'{PREFIX}/plateau.replay.json');r=c.call('run_replay',path=f'{PREFIX}/plateau.replay.json');assert r['matched'] and r['meshes_matched']
  report=dict(build=c.call('inspect')['build_id'],vertices=len(g['positions']),triangles=len(g['triangles']),operations=len(operations),rejections=len(rejections),closed=True,islands=1,roof_faces=len(roof),reload_pixels_equal=True,replay_matched=True,physical_erosion=False,human_acceptance=False)
  (OUT/'verification.json').write_text(json.dumps(report,indent=2));(OUT/'rejections.json').write_text(json.dumps(rejections,indent=2));(ROOT/'examples/directional-plateau.recipe.json').write_text(json.dumps(dict(parameters=params,base=source,operations=operations,views=views),indent=2)+'\n');print(json.dumps(report))
 finally:c.close()
if __name__=='__main__':main()
