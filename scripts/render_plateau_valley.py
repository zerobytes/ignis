"""Cold authored plateau landscape: native Landform and MCP cuts; no Python meshes.
Performance review: bounded seven small artifacts, no per-frame generation or new passes.
"""
import copy,json,math,random
from verify_lab import Client,ROOT
from open_earth_walk import init
from relief_compositions import ZERO
from render_plateau_materials import grass
OUT=ROOT/'artifacts/plateau-valley'; PREFIX='artifacts/plateau-valley'
def main():
 OUT.mkdir(parents=True,exist_ok=True)
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log')
 try:
  init(c);c.call('generate',recipe=dict(width=20,depth=20,obstacles=0,seed=221))
  c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in c.call('inspect')['scene']['entities']])
  material=copy.deepcopy(json.loads((ROOT/'assets/materials/rock/alpine.ignis-material').read_text())['material'])
  material['procedural']['recipe']['tile_size']=[22,22]
  placements=[(-52,12,30,30,22),(-23,0,44,29,29),(30,12,42,32,38),(64,34,31,28,52),(-62,62,48,36,55),(-12,80,36,32,67),(38,87,44,35,75)]
  recipes=[]; stats=[]
  for idx,(x,z,w,d,h) in enumerate(placements):
   rng=random.Random(2200+idx); n=12
   profile=[[round(math.cos(i*math.tau/n)*w*.5*(1+rng.uniform(-.1,.1)),5),round(math.sin(i*math.tau/n)*d*.5*(1+rng.uniform(-.1,.1)),5)] for i in range(n)]
   rows=[dict(at=0,offset=[0,0],scale=1.08),dict(at=h*.35,offset=[1,-.5],scale=.93),dict(at=h*.64,offset=[-.7,.4],scale=1.02),dict(at=h,offset=[0,0],scale=1)]
   doc=dict(version=1,space_version=1,id=f'plateau{idx}',revision=1,origin=ZERO,pieces=[dict(id='body',anchor=[0,0,0],axis=[0,1,0],profile_x=[1,0,0],profile=profile,inner_scale=None,sections=rows)])
   name=f'plateau{idx}'
   c.call('publish_landform',publication=dict(document=doc,asset=name,instance=name,frame_origin=ZERO,color=[1,1,1],shading='flat',collision=True))
   cuts=[]
   # Finite oblique wedges opening on the visible wall; ends/width/depth vary.
   for j in range(4):
    cx=(-.3+j*.2)*w; yy=h*(.22+.16*j); front=-d*.5
    pts=[[cx-w*.22,yy-h*.14,front-9],[cx+w*.19,yy+h*.17,front-9],[cx+w*.24,yy+h*.05,front-9],[cx-w*.14,yy-h*.2,front-9],[cx,yy,front+d*.4]]
    op=dict(kind='subtract_convex',points=pts)
    ref=c.call('inspect_mesh_topology',asset=name)['reference']
    result=c.call('edit_mesh',edit=dict(kind='model_selection',asset=name,reference=ref,operation=op))
    cuts.append(op)
   original=c.call('inspect_mesh',asset=name)['asset']; geom=original['geometry']
   topology=c.call('inspect_mesh_topology',asset=name)['topology'];assert topology['boundary_edges']==topology['non_manifold_edges']==0
   c.call('edit_mesh',edit=dict(kind='set_material',asset=name,material=material))
   c.call('edit_mesh',edit=dict(kind='set_material_coordinates',asset=name,coordinates=[[a+.45*k,b+.25*k,k] for a,b,k in geom['positions']]))
   inst=copy.deepcopy(next(i for i in c.call('inspect_meshes')['instances'] if i['id']==name));inst['position']=[x,0,z]
   c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
   turf=copy.deepcopy(original);turf['id']=name+'grass';turf['material']=grass(221+idx)
   c.call('edit_mesh',edit=dict(kind='put_asset',asset=turf))
   remove=[i for i,f in enumerate(geom['triangles']) if not all(abs(geom['positions'][v][1]-h)<.001 for v in f)]
   c.call('edit_mesh',edit=dict(kind='remove_faces',asset=turf['id'],reference=c.call('inspect_mesh_topology',asset=turf['id'])['reference'],faces=remove))
   inst.update(id=turf['id'],asset=turf['id'],position=[x,.035,z]);c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
   recipes.append(dict(source=doc,cuts=cuts,placement=[x,0,z]));stats.append(dict(id=name,vertices=len(geom['positions']),triangles=len(geom['triangles'])))
  floor=dict(version=1,space_version=1,id='floor',revision=1,datum=ZERO,base=dict(kind='constant',height_meters=-.1),operations=[])
  c.call('publish_terrain',publication=dict(document=floor,patch=dict(origin={'cell':['-1','0','-1'],'offset':[624,0,624]},cells=[8,8],spacing_meters=100),stride=1,asset='floor',instance='floor',frame_origin=ZERO,color=[1,1,1],collision=True))
  c.call('edit_mesh',edit=dict(kind='set_material',asset='floor',material=grass(223,True)))
  c.call('set_light',light=dict(direction=[.4,-.7,.55],ambient=.28,direct_intensity=1.1,direct_color=[1,.95,.85],shadow=dict(center=[0,30,40],extent=250,resolution=2048,bias=.0005,receiver_plane_bias=True),atmosphere=dict(density=.0012,color=[.57,.7,.8],reduce_details=False,sun_color=[1,.9,.7],sun_strength=.1)))
  views=dict(overview=dict(position=[155,135,-185],target=[0,26,40],fov_degrees=48),near=dict(position=[22,36,-77],target=[18,23,18],fov_degrees=52))
  for name,camera in views.items():
   c.call('set_camera',camera=camera);c.call('capture',path=f'{PREFIX}/{name}.png',width=1600,height=1000)
  c.call('set_camera',camera=views['overview']);c.call('save_scene',path=f'{PREFIX}/valley.scene.json')
  c.call('load_scene',path=f'{PREFIX}/valley.scene.json');c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=1000)
  assert (OUT/'overview.png').read_bytes()==(OUT/'reload.png').read_bytes()
  (ROOT/'examples/plateau-valley.recipe.json').write_text(json.dumps(dict(plateaus=recipes,views=views),indent=2)+'\n')
  (OUT/'verification.json').write_text(json.dumps(dict(build=c.call('inspect')['build_id'],stats=stats,closed=True,reload_pixels_equal=True,native_window=False,human_acceptance=False),indent=2))
  print(json.dumps(stats))
 finally:c.close()
if __name__=='__main__':main()
