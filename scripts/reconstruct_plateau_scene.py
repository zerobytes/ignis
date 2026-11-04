"""Layer-mapped reference landscape; native geometry through MCP only.
Static performance review: cold source authorship, bounded region meshes and one
96x48 background height patch, no generation in frames or new rendering passes.
"""
import copy,json,math,random
from verify_lab import Client,ROOT
from open_earth_walk import init
from relief_compositions import ZERO,position
from render_plateau_materials import grass
OUT=ROOT/'artifacts/plateau-scene';PREFIX='artifacts/plateau-scene'
REGIONS=[
 ('L1_plaza',0,-100,130,86,6,[[-.55,-.4],[-.3,-.55],[.14,-.5],[.42,-.34],[.5,.13],[.3,.44],[-.15,.5],[-.5,.28]]),
 ('L2_right_low',104,-68,58,46,9,[[-.5,-.3],[-.16,-.5],[.4,-.46],[.56,-.1],[.36,.32],[.1,.5],[-.46,.28]]),
 ('L3_right_mid',111,-26,78,48,16,[[-.52,-.38],[-.18,-.47],[.11,-.35],[.49,-.42],[.5,.25],[.25,.48],[-.4,.42]]),
 ('L4_center',-19,0,155,60,24,[[-.54,-.28],[-.38,-.51],[-.1,-.46],[.08,-.52],[.32,-.39],[.54,-.25],[.48,.4],[.2,.47],[-.45,.42]]),
 ('L4_right_ledge',103,15,86,55,25,[[-.5,-.36],[-.2,-.52],[.2,-.38],[.55,-.1],[.36,.4],[-.25,.49],[-.48,.2]]),
 ('L4_left_low',-112,-38,75,45,12,[[-.5,-.28],[-.22,-.48],[.25,-.45],[.5,-.04],[.38,.4],[-.3,.47]]),
 ('L4_left_mid',-143,9,86,66,24.7,[[-.5,-.3],[-.18,-.5],[.3,-.44],[.53,-.05],[.38,.42],[-.33,.49]]),
 ('L5_left_back',-149,75,90,90,34,[[-.5,-.4],[-.17,-.52],[.4,-.43],[.52,.2],[.16,.5],[-.45,.4]]),
 ('L5_upper',-19,88,184,116,43,[[-.51,-.4],[-.23,-.5],[.1,-.44],[.5,-.27],[.52,.35],[.18,.49],[-.44,.4]]),
 ('L5_right_upper',133,90,119,116,44,[[-.5,-.45],[-.14,-.5],[.47,-.27],[.5,.35],[.1,.51],[-.46,.33]])]
shore=[(-230,-175),(-100,-175),(-70,-147),(-40,-126),(20,-129),(65,-146),(105,-183),(230,-185),(230,240),(-230,240)]
REGIONS.insert(0,('shore_bank',0,30,460,440,2.5,[[x/460,(z-30)/440] for x,z in shore]))
def main():
 OUT.mkdir(parents=True,exist_ok=True);c=Client(ROOT/'target/release/ignis',log_path=OUT/'native.log');sources=[];stats=[]
 try:
  init(c);assert 'publish_landform' in json.dumps(c.request('tools/list')['result']);c.call('generate',recipe=dict(width=20,depth=20,seed=221,obstacles=0));c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in c.call('inspect')['scene']['entities']])
  rock=copy.deepcopy(json.loads((ROOT/'assets/materials/rock/alpine.ignis-material').read_text())['material']);rock['procedural']['recipe']['tile_size']=[18,18];rock['procedural']['recipe']['rock'].update(dark=[.17,.2,.24],light=[.51,.55,.58],contrast=.7,joint_depth=.07,grain=.35,detail_height=.2)
  def publish(name,doc,roof_y=None,green=True):
   c.call('publish_landform',publication=dict(document=doc,asset=name,instance=name,frame_origin=ZERO,color=[1,1,1],shading='flat',collision=True))
   a=c.call('inspect_mesh',asset=name)['asset'];g=a['geometry'];t=c.call('inspect_mesh_topology',asset=name)['topology'];assert t['boundary_edges']==t['non_manifold_edges']==0
   c.call('edit_mesh',edit=dict(kind='set_material',asset=name,material=rock));c.call('edit_mesh',edit=dict(kind='set_material_coordinates',asset=name,coordinates=[[x+.45*z,y+.25*z,z] for x,y,z in g['positions']]))
   if roof_y is not None:
    roof=[i for i,f in enumerate(g['triangles']) if all(abs(g['positions'][v][1]-roof_y)<.01 for v in f)]
    if roof:
     turf=copy.deepcopy(a);turf['id']=name+'_top';turf['material']=grass(221)
     if not green:turf['material']['pattern'].update(low=[.34,.3,.21],high=[.48,.44,.31],bump=.002)
     else:turf['material']['pattern'].update(low=[.12,.27,.055],high=[.25,.4,.1],bump=.002)
     c.call('edit_mesh',edit=dict(kind='put_asset',asset=turf));c.call('edit_mesh',edit=dict(kind='remove_faces',asset=turf['id'],reference=c.call('inspect_mesh_topology',asset=turf['id'])['reference'],faces=[i for i in range(len(g['triangles'])) if i not in roof]));inst=copy.deepcopy(next(i for i in c.call('inspect_meshes')['instances'] if i['id']==name));inst.update(id=turf['id'],asset=turf['id'],position=[0,.6,0]);c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
   sources.append(doc);stats.append(dict(id=name,vertices=len(g['positions']),triangles=len(g['triangles'])))
  for idx,(name,x,z,w,d,h,contour) in enumerate(REGIONS):
   rng=random.Random(221+idx); profile=[]
   # Keep authored corners; add a shoulder on each boundary segment, not circular pillars.
   for a,b in zip(contour,contour[1:]+contour[:1]):
    profile.extend([[a[0]*w,-a[1]*d],[((a[0]+b[0])*.5)*w*.97,-((a[1]+b[1])*.5)*d*.97]])
   if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(profile,profile[1:]+profile[:1]))<0:profile.reverse()
   n=len(profile);joints=[rng.uniform(.83,1.03) for _ in profile]
   rows=[dict(at=a,scale=s,offset=o,radial_scales=r) for a,s,o,r in [(0,1.05,[0,0],[1]*n),(h*.32,1,[-1,1],joints),(h*.72,.98,[1,0],[.96+.025*math.sin(i*1.7+idx) for i in range(n)]),(h,1,[0,0],[1]*n)]]
   doc=dict(version=2,space_version=1,id=name,revision=1,origin=ZERO,pieces=[dict(id='body',axis=[0,1,0],profile_x=[1,0,0],anchor=[-x,-4,z],profile=profile,inner_scale=None,sections=rows)])
   publish(name,doc,h-4,name!='L1_plaza')
  # Existing fully volumetric landform source for the main projecting escarpment.
  hero=json.loads((ROOT/'examples/plateau-wall.recipe.json').read_text());hero['id']='main_escarpment'
  for p in hero['pieces']:p['anchor'][1]+=20;p['anchor'][2]+=40
  # Shaping fields are in document space and must follow the translated source.
  for p in hero['pieces']:
   if p.get('shaping'):
    for s in p['shaping']['stamps']:s['center'][1]+=20;s['center'][2]+=40
    for plane in p['shaping'].get('creases',[]):
     if 'point' in plane:plane['point'][1]+=20;plane['point'][2]+=40
  publish('main_escarpment',hero,60)
  water=dict(version=1,space_version=1,id='water',revision=1,datum=ZERO,base=dict(kind='constant',height_meters=-3),operations=[])
  c.call('publish_terrain',publication=dict(document=water,patch=dict(origin=position(-1280,0,-1280),cells=[16,16],spacing_meters=160),stride=1,asset='water',instance='water',frame_origin=ZERO,color=[.07,.53,.6],collision=False));c.call('edit_mesh',edit=dict(kind='set_material',asset='water',material=dict(reflectance=dict(roughness=.22,f0=.025),transmission=0)))
  bg=dict(version=2,space_version=1,id='mountains',revision=1,datum=ZERO,base=dict(kind='constant',height_meters=-8),operations=[])
  for x,z,width,height,az in [(-150,260,210,120,-15),(40,310,180,100,23),(180,270,160,90,-25)]:
   bg['operations'].append(dict(kind='ridge',center=position(x,0,z),azimuth_degrees=az,width_meters=width,depth_meters=150,height_meters=height,phase=.3,crest_wander=.16,slope_exponent=1.35,folds_meters=5,end_fade=.22))
  c.call('publish_terrain',publication=dict(document=bg,patch=dict(origin=position(-360,0,160),cells=[96,48],spacing_meters=8),stride=1,asset='mountains',instance='mountains',frame_origin=ZERO,color=[.39,.49,.57],collision=False))
  c.call('set_light',light=dict(direction=[.4,-.8,.6],ambient=.3,direct_intensity=1.1,direct_color=[1,.97,.9],shadow=dict(center=[0,30,0],extent=250,resolution=2048,bias=.0004,receiver_plane_bias=True),atmosphere=dict(density=.00035,color=[.55,.75,.87],reduce_details=False,sun_color=[1,.95,.85],sun_strength=.04)))
  views=dict(overview=dict(position=[0,190,-300],target=[0,23,15],fov_degrees=56),near=dict(position=[25,112,-174],target=[0,27,25],fov_degrees=55),side=dict(position=[275,175,-180],target=[0,20,20],fov_degrees=55))
  for name,cam in views.items():c.call('set_camera',camera=cam);c.call('capture',path=f'{PREFIX}/{name}.png',width=1600,height=900)
  c.call('set_camera',camera=views['overview']);c.call('save_scene',path=f'{PREFIX}/scene.scene.json');c.call('load_scene',path=f'{PREFIX}/scene.scene.json');c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=900);assert (OUT/'overview.png').read_bytes()==(OUT/'reload.png').read_bytes()
  (ROOT/'examples/plateau-scene.recipe.json').write_text(json.dumps(dict(regions=REGIONS,sources=sources,background=bg,views=views),indent=2)+'\n');(OUT/'verification.json').write_text(json.dumps(dict(build=c.call('inspect')['build_id'],stats=stats,closed_per_asset=True,reload_png_equal=True,boolean_union=False,reference_fidelity=False),indent=2));print('published',len(stats),'landforms')
 finally:c.close()
if __name__=='__main__':main()
