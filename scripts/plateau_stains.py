"""Seeded authoring of angular stain contours and rooted triangular rock prisms.
Native Landform compiles all geometry. Cold bounded recipe data only.
"""
import math,random
from relief_compositions import ZERO

def document(r):
 rng=random.Random(r['seed']);n=r.get('contour_points',28);count=r.get('rock_count',24)
 if not 12<=n<=64 or not 1<=count<=30:raise ValueError('Use12..64 contour points and1..30 rocks')
 w,d,h=[r[k] for k in ['width','depth','height']]
 if not all(math.isfinite(x) and 5<=x<=500 for x in [w,d,h]):raise ValueError('Dimensions5..500m')
 phases=[rng.uniform(0,math.tau) for _ in range(3)]; contour=[]
 for i in range(n):
  a=i*math.tau/n
  radius=1+.22*math.sin(3*a+phases[0])+.14*math.sin(5*a+phases[1])+.07*math.cos(7*a+phases[2])
  contour.append([math.cos(a)*w*.5*radius,math.sin(a)*d*.5*radius])
 profile=[[x,-z] for x,z in reversed(contour)]
 def row(at,scale,offset=(0,0)):return dict(at=at,scale=scale,offset=list(offset))
 body=dict(id='stain',anchor=[0,0,0],axis=[0,1,0],profile_x=[1,0,0],profile=profile,inner_scale=None,sections=[row(0,1.045),row(h*.45,.98),row(h,1)])
 pieces=[body];family=r.get('fracture_direction',[.22,1,-.28]); lengths=[]
 edges=[math.dist(a,b) for a,b in zip(contour,contour[1:]+contour[:1])];perimeter=sum(edges)
 for k in range(count):
  station=perimeter*(k+rng.uniform(.12,.88))/count;edge=0
  while station>edges[edge] and edge<n-1:station-=edges[edge];edge+=1
  t=station/edges[edge];a=contour[edge];b=contour[(edge+1)%n]
  x=a[0]*(1-t)+b[0]*t;z=a[1]*(1-t)+b[1]*t
  tx=(b[0]-a[0])/edges[edge];tz=(b[1]-a[1])/edges[edge];nx,nz=tz,-tx
  width=rng.uniform(*r.get('rock_width',[9,22]));depth=rng.uniform(*r.get('rock_depth',[8,16]));length=rng.uniform(h*.65,h*1.05)
  root_y=-rng.uniform(.3,2);length=min(length,h-root_y-rng.uniform(.3,2))
  axis=[family[0]+nx*.08+rng.uniform(-.05,.05),family[1],family[2]+nz*.08+rng.uniform(-.05,.05)]
  norm=math.sqrt(sum(v*v for v in axis));axis=[v/norm for v in axis]
  # Embed the root and the middle of the tilted prism in the same wall.
  embed=depth*.45+max(0,(axis[0]*nx+axis[2]*nz)*length*.5)
  anchor=[x-nx*embed,root_y,z-nz*embed]
  triangle=[[-width*rng.uniform(.35,.65),-depth*.55],[width*rng.uniform(.35,.65),-depth*rng.uniform(.25,.6)],[width*rng.uniform(-.8,.8),depth*rng.uniform(.6,1.05)]]
  centroid=[sum(p[i] for p in triangle)/3 for i in range(2)]
  triangle=[[p[i]-centroid[i] for i in range(2)] for p in triangle]
  pieces.append(dict(id=f'rock{k}',anchor=anchor,axis=axis,profile_x=[tx,0,tz],profile=triangle,inner_scale=None,sections=[row(0,rng.uniform(.8,1.2)),row(length, rng.uniform(.55,1.15),[rng.uniform(-.8,.8),rng.uniform(-.4,.4)])]))
  lengths.append(length)
 return dict(version=1,space_version=1,id=r['id'],revision=1,origin=ZERO,pieces=pieces)
