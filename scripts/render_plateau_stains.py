"""Cold authored plateau landscape: native Landform and MCP cuts; no Python meshes.
Performance review: bounded three compound artifacts, no per-frame generation or new passes.
"""
import copy,json,math,random
from verify_lab import Client,ROOT
from open_earth_walk import init
from relief_compositions import ZERO
from render_plateau_materials import grass
from plateau_stains import document
OUT=ROOT/'artifacts/plateau-stains'; PREFIX='artifacts/plateau-stains'
def main():
 OUT.mkdir(parents=True,exist_ok=True)
 c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log')
 try:
  init(c);c.call('generate',recipe=dict(width=20,depth=20,obstacles=0,seed=221))
  c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in c.call('inspect')['scene']['entities']])
  material=copy.deepcopy(json.loads((ROOT/'assets/materials/rock/alpine.ignis-material').read_text())['material'])
  material['procedural']['recipe']['tile_size']=[22,22]
  placements=[(-62,-15,90,70,33),(42,22,110,78,48),(-10,108,126,88,64)]
  recipes=[]; stats=[]
  for idx,(x,z,w,d,h) in enumerate(placements):
   params=dict(id=f'plateau{idx}',seed=731+idx,width=w,depth=d,height=h,contour_points=28,rock_count=26,fracture_direction=[.38,1,-.42])
   doc=document(params)
   assert doc==document(params)
   probe=copy.deepcopy(doc);probe['pieces']=probe['pieces'][:1]
   for attempt in range(5):
    rays=[]
    for rock in doc['pieces'][1:]:
     for t in [.25,.5,.75]:
      length=rock['sections'][-1]['at'];axis=rock['axis'];anchor=rock['anchor']
      rays.append(dict(origin=[anchor[i]+axis[i]*length*t for i in range(3)],direction=[1,0,0],max_distance_meters=1000))
    hits=c.call('query_landform',query=dict(kind='raycast',document=probe,rays=rays))['hits']
    rooted=[any(len(q)%2==1 for q in hits[i:i+3]) for i in range(0,len(hits),3)]
    if all(rooted):break
    for k,ok in enumerate(rooted):
     if not ok:
      doc['pieces'][k+1]['anchor'][0]*=.9;doc['pieces'][k+1]['anchor'][2]*=.9
   assert all(rooted),rooted
   name=f'plateau{idx}'
   c.call('publish_landform',publication=dict(document=doc,asset=name,instance=name,frame_origin=ZERO,color=[1,1,1],shading='flat',collision=True))
   cuts=[]
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
   inst.update(id=turf['id'],asset=turf['id'],position=[x,.75,z]);c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
   recipes.append(dict(parameters=params,source=doc,cuts=cuts,placement=[x,0,z]));stats.append(dict(id=name,vertices=len(geom['positions']),triangles=len(geom['triangles'])))
  floor=dict(version=1,space_version=1,id='floor',revision=1,datum=ZERO,base=dict(kind='constant',height_meters=-.1),operations=[])
  c.call('publish_terrain',publication=dict(document=floor,patch=dict(origin={'cell':['-1','0','-1'],'offset':[624,0,624]},cells=[8,8],spacing_meters=100),stride=1,asset='floor',instance='floor',frame_origin=ZERO,color=[1,1,1],collision=True))
  c.call('edit_mesh',edit=dict(kind='set_material',asset='floor',material=grass(223,True)))
  c.call('set_light',light=dict(direction=[.4,-.7,.55],ambient=.28,direct_intensity=1.1,direct_color=[1,.95,.85],shadow=dict(center=[0,30,40],extent=250,resolution=2048,bias=.0005,receiver_plane_bias=True),atmosphere=dict(density=.0003,color=[.57,.7,.8],reduce_details=False,sun_color=[1,.9,.7],sun_strength=.1)))
  views=dict(overview=dict(position=[165,145,-190],target=[0,26,40],fov_degrees=48),near=dict(position=[110,56,-105],target=[35,25,22],fov_degrees=52))
  views['top']=dict(position=[0,330,47],target=[0,0,40],fov_degrees=48)
  for name,camera in views.items():
   c.call('set_camera',camera=camera);c.call('capture',path=f'{PREFIX}/{name}.png',width=1600,height=1000)
  c.call('set_camera',camera=views['overview']);c.call('save_scene',path=f'{PREFIX}/valley.scene.json')
  c.call('load_scene',path=f'{PREFIX}/valley.scene.json');c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=1000)
  assert (OUT/'overview.png').read_bytes()==(OUT/'reload.png').read_bytes()
  (ROOT/'examples/plateau-stains.recipe.json').write_text(json.dumps(dict(plateaus=recipes,views=views),indent=2)+'\n')
  (OUT/'verification.json').write_text(json.dumps(dict(build=c.call('inspect')['build_id'],stats=stats,closed=True,root_probes_per_prism=3,all_prisms_rooted=True,reload_pixels_equal=True,native_window=False,human_acceptance=False),indent=2))
  print(json.dumps(stats))
 finally:c.close()
if __name__=='__main__':main()
