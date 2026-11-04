"""Reference reconstruction using native profile/section tools through MCP.
Cold bounded authorship; no mesh triangulation in Python, no frame-time generation.
"""
import copy,json,random
from verify_lab import Client,ROOT
from open_earth_walk import init
from relief_compositions import ZERO
from render_plateau_materials import grass
OUT=ROOT/'artifacts/plateau-reconstruction';P='artifacts/plateau-reconstruction'
def piece(name,axis,px,anchor,profile,rows):
 if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(profile,profile[1:]+profile[:1]))<0:profile=list(reversed(profile))
 return dict(id=name,axis=axis,profile_x=px,anchor=anchor,profile=profile,inner_scale=None,sections=[dict(at=a,scale=s,offset=o) for a,s,o in rows])
def main():
 OUT.mkdir(parents=True,exist_ok=True);c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log')
 try:
  init(c);assert 'publish_landform' in json.dumps(c.request('tools/list')['result'])
  c.call('generate',recipe=dict(width=20,depth=20,seed=221,obstacles=0));c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in c.call('inspect')['scene']['entities']])
  outline=[[-1,-1],[1,-1],[1,1],[-1,1]]
  pieces=[]
  # X along wall; YZ profiles form genuinely solid inclined slabs, not image sheets.
  pieces.append(piece('P1_wall',[1,0,0],[0,1,0],[0,0,12],[[0,-8],[0,16],[34,16],[34,1],[25,-4],[14,-7]],[(-76,1,[0,0]),(-35,1,[0,-2]),(20,1,[0,1]),(76,.94,[1,0])]))
  pieces.append(piece('P3_slab',[1,0,0],[0,1,0],[0,25,0],[[12,-19],[12,-11],[7,-9],[-9,-1],[-13,-4],[-3,-11]],[(-37,.68,[-7,3]),(-13,.87,[-2,1]),(23,1,[7.3,0]),(52,1.05,[15.6,0]),(61,1.08,[18,0])]))
  pieces.append(piece('P4_lower_wedge',[1,0,0],[0,1,0],[0,15,-1],[[5,-14],[9,-6],[2,2],[-4,3],[-7,-6]],[(-22,.45,[0,0]),(-5,1,[0,0]),(15,.9,[-1,0]),(32,0,[-4,2])]))
  for idx,(x,w,h,z) in enumerate([(-55,10,29,-2),(-40,8,22,-5),(-27,10,18,-1),(28,10,32,2),(46,12,38,4),(62,9,40,6)]):
   profile=[[-w,-4],[-w*.75,-10],[0,-13],[w*.7,-8],[w,0],[w*.7,9],[-w,8]]
   pieces.append(piece(f'P5_6_support{idx}',[0,1,0],[1,0,0],[x,0,z],profile,[(0,1.12,[0,0]),(h*.2,.91,[-1,-1]),(h*.65,.85,[1,1]),(h,.53,[0,4])]))
  for idx,(x,y,z,s) in enumerate([(-57,0,-10,5),(-44,0,-13,6),(-30,0,-9,4),(-17,0,-12,5),(10,0,-7,7),(26,0,-8,4),(44,0,-5,5)]):
   pieces.append(piece(f'P7_foot{idx}',[0,1,0],[1,0,0],[x,y,z],[[-s,-s],[s*.6,-s],[s,s*.5],[-s*.7,s]],[(0,1,[0,0]),(s*.7,.95,[1,-1]),(s*1.2,.2,[0,0])]))
  terrace=piece('P8_terrace',[0,1,0],[1,0,0],[0,-8,-25],[[-83,-45],[-66,-42],[-27,-40],[8,-42],[40,-46],[78,-30],[81,12],[-82,10]],[(0,1,[0,0]),(7,.98,[0,0]),(8,1,[0,0])]);pieces.append(terrace)
  slab=next(p for p in pieces if p['id']=='P3_slab')
  stamps=[]
  for k,x in enumerate([-22,-3,17,36,52]):
   for j in [1,2]:
    stamps.append(dict(center=[x,37.3+.306*x-10*j,-19+5.3*j],axis=[0,0,-1],profile_x=[1,0,0],half_extents=[8,6,7],displacement=[.25*(-1 if k%2 else 1),-.1,-1.3],flat_core=.3,falloff='linear'))
  slab['shaping']=dict(subdivisions=2,stamps=stamps,creases=[])
  mat=copy.deepcopy(json.loads((ROOT/'assets/materials/rock/alpine.ignis-material').read_text())['material']);mat['procedural']['recipe']['tile_size']=[22,22]
  mat['procedural']['recipe']['rock'].update(dark=[.16,.19,.23],light=[.49,.53,.57],contrast=.6,joint_depth=.045,detail_height=.12,grain=.35)
  stats=[]
  for p in pieces:
   name=p['id'];doc=dict(version=3 if p.get('shaping') else 1,space_version=1,id=name,revision=1,origin=ZERO,pieces=[p]);c.call('publish_landform',publication=dict(document=doc,asset=name,instance=name,frame_origin=ZERO,color=[1,1,1],shading='flat',collision=True))
   a=c.call('inspect_mesh',asset=name)['asset'];g=a['geometry'];t=c.call('inspect_mesh_topology',asset=name)['topology'];assert t['boundary_edges']==t['non_manifold_edges']==0
   c.call('edit_mesh',edit=dict(kind='set_material',asset=name,material=mat));c.call('edit_mesh',edit=dict(kind='set_material_coordinates',asset=name,coordinates=[[x+.5*z,y+.2*z,z] for x,y,z in g['positions']]))
   if name in ['P1_wall','P8_terrace']:
    roof=[i for i,f in enumerate(g['triangles']) if max(g['positions'][v][1] for v in f)-min(g['positions'][v][1] for v in f)<.01 and min(g['positions'][v][1] for v in f)>=(-.01 if name=='P8_terrace' else 30)]
    if roof:
     turf=copy.deepcopy(a);turf['id']=name+'_turf';turf['material']=grass(221);c.call('edit_mesh',edit=dict(kind='put_asset',asset=turf));c.call('edit_mesh',edit=dict(kind='remove_faces',asset=turf['id'],reference=c.call('inspect_mesh_topology',asset=turf['id'])['reference'],faces=[i for i in range(len(g['triangles'])) if i not in roof]));inst=copy.deepcopy(next(i for i in c.call('inspect_meshes')['instances'] if i['id']==name));inst.update(id=turf['id'],asset=turf['id'],position=[0,.25,0]);c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
   stats.append(dict(id=name,vertices=len(g['positions']),triangles=len(g['triangles'])))
  c.call('set_light',light=dict(direction=[.35,-.85,.45],ambient=.24,direct_intensity=1.1,direct_color=[1,.97,.9],shadow=dict(center=[0,16,0],extent=180,resolution=2048,bias=.0004,receiver_plane_bias=True),atmosphere=dict(density=.0006,color=[.58,.73,.84],reduce_details=False,sun_color=[1,.95,.85],sun_strength=.05)))
  views=dict(reference=dict(position=[60,85,-142],target=[0,20,-3],fov_degrees=45),front=dict(position=[0,40,-210],target=[0,14,-2],fov_degrees=45),side=dict(position=[175,55,-30],target=[0,16,0],fov_degrees=48),rear=dict(position=[100,90,180],target=[0,16,0],fov_degrees=48))
  for n,cam in views.items():c.call('set_camera',camera=cam);c.call('capture',path=f'{P}/{n}.png',width=1323,height=708)
  c.call('set_camera',camera=views['reference']);c.call('save_scene',path=f'{P}/crop.scene.json');c.call('load_scene',path=f'{P}/crop.scene.json');c.call('capture',path=f'{P}/reload.png',width=1323,height=708);assert (OUT/'reload.png').read_bytes()==(OUT/'reference.png').read_bytes()
  (ROOT/'examples/plateau-reconstruction.recipe.json').write_text(json.dumps(dict(pieces=pieces,views=views),indent=2)+'\n');(OUT/'verification.json').write_text(json.dumps(dict(stats=stats,views=views,build=c.call('inspect')['build_id'],closed_per_piece=True,boolean_union=False,reload_equal=True,reference_match=False),indent=2));print('captured',len(pieces),'native solids')
 finally:c.close()
if __name__=='__main__':main()
