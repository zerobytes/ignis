"""Seeded native Landform controls, not mesh triangulation or a mesh catalog."""
import hashlib,math,random
from plateau_stains import document
from patchy_plateau import bell
from weathered_plateau import norm

VERSION=1

def derive(seed,key):
 return int.from_bytes(hashlib.blake2b(f'plateau-v{VERSION}:{seed}:{key}'.encode(),digest_size=8).digest(),'little')

def region(seed):
 rng=random.Random(derive(seed,'geology'))
 return dict(bedding=[rng.uniform(.24,.42),1,rng.uniform(.035,.12)],exposure_axis=rng.uniform(-1.5,-.5))

def controls(world_seed,row,attempt=0):
 seed=derive(world_seed,row['id']);effective=derive(seed,f'candidate:{attempt}');rng=random.Random(effective)
 cfg=dict(id=row['id'],seed=effective,width=rng.uniform(72,158),depth=rng.uniform(52,112),
          height=rng.uniform(24,40),contour_points=rng.choice([20,24,28]),rock_count=1)
 source=document(cfg);source['version']=2;source['pieces']=source['pieces'][:1];p=source['pieces'][0]
 regional=region(world_seed);x,_,z=regional['bedding'];x+=rng.uniform(-.018,.018);z+=rng.uniform(-.018,.018)
 yaw=2*math.atan2(row['rotation'][1],row['rotation'][3]);co,si=math.cos(yaw),math.sin(yaw)
 axis=norm([co*x-si*z,1,si*x+co*z]);p['axis']=axis;p['profile_x']=norm([1,-axis[0]/axis[1],0]);p['anchor']=[0,0,0]
 # Inclined section range extends past both eventual horizontal truncation planes.
 pad=cfg['width']*.7*abs(axis[0])+cfg['depth']*.7*abs(axis[2])+12
 start=-pad/axis[1];end=(cfg['height']+pad)/axis[1]
 count=rng.randint(9,13);weights=[rng.uniform(3,10) for _ in range(count)];span=end-start
 thicknesses=[v/sum(weights)*span for v in weights]
 phase=rng.uniform(-math.pi,math.pi);exposure=regional['exposure_axis']+yaw+rng.uniform(-.6,.6)
 scar_center=rng.uniform(-math.pi,math.pi);scar_h=rng.uniform(0,cfg['height']);scar_width=rng.uniform(.4,.9)
 scars=[rng.uniform(.045,.13),rng.uniform(.025,.085)];rows=[];at=start;n=len(p['profile'])
 for k,thickness in enumerate(thicknesses):
  erosion=rng.uniform(.01,.095);fraction=rng.uniform(.55,.78);patch=rng.uniform(.25,.65)
  for t in [0,fraction]:
   h=at+thickness*t;factors=[]
   for j in range(n):
    a=j*math.tau/n
    preserved=.035*math.sin(2*a+.022*h+phase)+.035*math.sin(3*a-.035*h-phase)
    preserved+=.03*math.sin(7*a+.09*h+phase)*bell(a,exposure,.9)
    mask=min(1,bell(a,exposure+.5*math.sin(k*.8+phase),patch)+.65*bell(a,exposure+2.7+.4*math.cos(k*.65),patch*1.3))
    scar=scars[0]*bell(a,scar_center+.007*h,scar_width)*math.exp(-((h-scar_h)/15)**2)
    scar+=scars[1]*bell(a,scar_center+2.1-.004*h,.65)*math.exp(-((h-cfg['height']*.8)/11)**2)
    factors.append(1+preserved-(erosion if t else 0)*mask**3-scar)
   rows.append(dict(at=h,scale=1,offset=[1.1*math.sin(h*.03+phase),.7*math.sin(h*.055-phase)],radial_scales=factors))
  at+=thickness
 rows.append(dict(at=end,scale=1,offset=[0,0],radial_scales=[1]*n));p['sections']=rows
 return dict(version=VERSION,requested_seed=seed,effective_seed=effective,attempt=attempt,parameters=cfg,
             regional=regional,source=source,joint_count=rng.randint(4,7),spall_count=rng.randint(4,7),chip_count=rng.randint(2,4))
