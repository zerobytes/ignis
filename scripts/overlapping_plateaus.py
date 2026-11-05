"""Seeded overlapping outcrops rooted in one native continuous basal formation."""
import math,random
from relief_compositions import ZERO
from render_plateau_amphitheatre import GROUND_Y
from seeded_plateaus import derive,region,controls
from patchy_plateau import bell

def placements(seed):
 rng=random.Random(derive(seed,'overlapping-layout'));rows=[]
 for level in range(5):
  for col in range(8):
   x=(col-3.5)*42+rng.uniform(-5,5)+7*math.sin(level*.9)
   z=-64+level*32+rng.uniform(-5,5)+.0006*x*x
   angle=rng.uniform(-.3,.3)
   rows.append(dict(id=f'q{level}_{col}',level=level,support_depth=0,position=[x,GROUND_Y,z],scale=1,
     rotation=[0,math.sin(angle/2),0,math.cos(angle/2)],target_height=30+level*22+col*.85+rng.uniform(0,.2),
     lateral_frequency_multiplier=1.10,lateral_relief_multiplier=1.10,full_height=True))
 return rows

def foundation_config(seed):
 rng=random.Random(derive(seed,'shared-foundation'));phase=[rng.uniform(-math.pi,math.pi) for _ in range(4)];n=96;h=27.;regional=region(seed)
 profile=[[265*math.cos(j*math.tau/n),170*math.sin(j*math.tau/n)] for j in range(n)];rows=[];nx,_,nz=regional['bedding']
 for k in range(25):
  y=h*k/24;window=math.sin(math.pi*k/24);factors=[]
  for j in range(n):
   a=j*math.tau/n;base=1+.055*math.sin(3*a+phase[0])+.04*math.sin(5*a+phase[1])+.025*math.sin(8*a+phase[2])
   x,z=profile[j][0]*base,-profile[j][1]*base
   mask=min(1,bell(a,phase[3],.7)+.6*bell(a,phase[3]+2.7,.8))
   notch=max(0,math.cos(math.tau*(y+nx*x+nz*z)/(8/1.10)))**6
   factors.append(base*(1-window*.018*1.10*notch*mask))
  rows.append(dict(at=y,scale=1,offset=[0,0],radial_scales=factors))
 source=dict(version=2,space_version=1,id='shared_foundation',revision=1,origin=ZERO,pieces=[dict(id='basal_mass',anchor=[0,0,0],axis=[0,1,0],profile_x=[1,0,0],profile=profile,sections=rows,inner_scale=None)])
 return dict(version=1,requested_seed=derive(seed,'shared-foundation'),effective_seed=derive(seed,'shared-foundation'),attempt=0,
   parameters=dict(id='foundation',height=h,width=530,depth=340),regional=regional,source=source,joint_count=0,spall_count=0,chip_count=0)

def configuration(seed,row,attempt=0):
 return foundation_config(seed) if row.get('foundation') else controls(seed,row,attempt)

def vertical_inside(g,x,y,z):
 """Read-only vertical parity diagnostic on native triangles; no generated topology."""
 hits=[]
 for f in g['triangles']:
  a,b,c=[g['positions'][i] for i in f];v0=[b[0]-a[0],b[2]-a[2]];v1=[c[0]-a[0],c[2]-a[2]];p=[x-a[0],z-a[2]]
  den=v0[0]*v1[1]-v1[0]*v0[1]
  if abs(den)<1e-9:continue
  u=(p[0]*v1[1]-v1[0]*p[1])/den;v=(v0[0]*p[1]-p[0]*v0[1])/den
  if u>=-1e-7 and v>=-1e-7 and u+v<=1+1e-7:hits.append(a[1]+u*(b[1]-a[1])+v*(c[1]-a[1]))
 unique=[]
 for at in sorted(hits):
  if not unique or at-unique[-1]>1e-4:unique.append(at)
 return sum(at>y+1e-4 for at in unique)%2==1

def boundary(g,row):
 from collections import Counter
 edges=Counter(tuple(sorted([u,v])) for f in g['triangles'] for u,v in zip(f,f[1:]+f[:1]));angle=2*math.atan2(row['rotation'][1],row['rotation'][3]);co,si=math.cos(angle),math.sin(angle)
 def point(i):
  x,_,z=g['positions'][i];return [co*x+si*z+row['position'][0],-si*x+co*z+row['position'][2]]
 return [(point(a),point(b)) for (a,b),count in edges.items() if count==1]

def overlaps(a,b):
 def inside(p,edges):
  n=0
  for u,v in edges:
   if (u[1]>p[1])!=(v[1]>p[1]) and p[0]<(v[0]-u[0])*(p[1]-u[1])/(v[1]-u[1])+u[0]:n+=1
  return n%2==1
 def cross(a,b,c):return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
 if any(inside(p,b) for p,_ in a) or any(inside(p,a) for p,_ in b):return True
 return any(cross(p,q,r)*cross(p,q,s)<0 and cross(r,s,p)*cross(r,s,q)<0 for p,q in a for r,s in b)
