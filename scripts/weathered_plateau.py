"""Author convex volume controls; all hulls, unions and cuts execute natively via MCP."""
import math,random

def add(a,b):return [x+y for x,y in zip(a,b)]
def mul(a,s):return [x*s for x in a]
def dot(a,b):return sum(x*y for x,y in zip(a,b))
def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
def norm(a):return mul(a,1/math.sqrt(dot(a,a)))

def chamfered_cell(anchor,u,v,axis,width,depth,length,rng,wear=.12):
 corners=[]
 skew=rng.uniform(-.2,.2)*width;taper=rng.uniform(.65,1.15)
 for i in range(8):
  end=bool(i&4);x=(.5 if i&1 else -.5)*width*(taper if end else 1)+(skew if end else 0)
  y=(.5 if i&2 else -.5)*depth*(rng.uniform(.8,1.1) if end else 1)
  corners.append(add(anchor,add(mul(u,x),add(mul(v,y),mul(axis,length if end else 0)))))
 # Unequal corner truncations retain large planes; native convex hull closes them.
 points=[]
 for i,p in enumerate(corners):
  amount=rng.uniform(.035,wear)
  for bit in [1,2,4]:points.append(add(mul(p,1-amount),mul(corners[i^bit],amount)))
 return points

def macro_cells(contour,height,seed,count=20):
 rng=random.Random(seed);edges=[math.dist(a,b) for a,b in zip(contour,contour[1:]+contour[:1])];total=sum(edges)
 signed=sum(a[0]*b[1]-a[1]*b[0] for a,b in zip(contour,contour[1:]+contour[:1]))
 result=[]
 for k in range(count):
  at=total*(k+rng.uniform(.15,.85))/count;i=0
  while at>edges[i] and i<len(edges)-1:at-=edges[i];i+=1
  t=at/edges[i];a,b=contour[i],contour[(i+1)%len(contour)]
  x,z=a[0]*(1-t)+b[0]*t,a[1]*(1-t)+b[1]*t
  tangent=norm([b[0]-a[0],0,b[1]-a[1]])
  outward=[tangent[2],0,-tangent[0]]
  if signed<0:outward=mul(outward,-1)
  family=[.35,1,-.28] if x>0 else [-.45,1,.15]
  axis=norm(add(family,[rng.uniform(-.08,.08),0,rng.uniform(-.08,.08)]))
  u=norm(add(tangent,mul(axis,-dot(tangent,axis))));v=norm(cross(axis,u))
  if dot(v,outward)<0:u=mul(u,-1);v=mul(v,-1)
  w=rng.uniform(19,34);d=rng.uniform(11,19);length=rng.uniform(height*.7,height*.98)
  anchor=[x*.86-axis[0]*length*.18,-1,z*.86-axis[2]*length*.18]
  result.append(chamfered_cell(anchor,u,v,axis,w,d,length,rng,.2))
 return result

def surface_face(g,rng):
 candidates=[]
 for f in g['triangles']:
  a,b,c=[g['positions'][i] for i in f];n=cross(add(b,mul(a,-1)),add(c,mul(a,-1)));area=math.sqrt(dot(n,n))
  if area<.1:continue
  n=mul(n,1/area);p=mul(add(a,add(b,c)),1/3)
  if abs(n[1])<.72 and 2<p[1]<31:candidates.append((p,n,area))
 return rng.choices(candidates,weights=[x[2] for x in candidates],k=1)[0][:2]

def thin_cell(g,rng):
 p,n=surface_face(g,rng);u=norm(cross([0,1,0],n));v=norm(cross(n,u))
 # Tilt within the wall plane as a correlated sheet, rather than a small pillar.
 angle=rng.uniform(-.45,.15);u2=add(mul(u,math.cos(angle)),mul(v,math.sin(angle)));v2=norm(cross(n,u2))
 width=rng.uniform(10,24);height=rng.uniform(5,13);thickness=rng.uniform(.8,2.4)
 center=add(p,mul(n,-thickness*.25));anchor=add(center,mul(v2,-height*.5))
 return chamfered_cell(anchor,u2,n,v2,width,thickness,height,rng,.23)

def chip(g,rng):
 p,n=surface_face(g,rng);u=norm(cross([0,1,0],n));v=norm(cross(n,u));radius=rng.uniform(.6,2.2);depth=rng.uniform(.3,1.2)
 points=[]
 for angle in [0,2.2,4.3]:
  points.append(add(p,add(mul(n,depth*1.7),add(mul(u,math.cos(angle)*radius),mul(v,math.sin(angle)*radius)))))
 points.append(add(p,mul(n,-depth)))
 return points

def edge_chip(g,rng):
 normals=[];edges={}
 for f,tri in enumerate(g['triangles']):
  a,b,c=[g['positions'][i] for i in tri];normals.append(norm(cross(add(b,mul(a,-1)),add(c,mul(a,-1)))))
  for i,j in zip(tri,tri[1:]+tri[:1]):edges.setdefault(tuple(sorted([i,j])),[]).append(f)
 choices=[]
 for (i,j),faces in edges.items():
  if len(faces)!=2:continue
  n1,n2=[normals[f] for f in faces];a,b=g['positions'][i],g['positions'][j];length=math.dist(a,b)
  if dot(n1,n2)>.75 or length<4 or dot(add(n1,n2),add(n1,n2))<.01:continue
  t=rng.uniform(.2,.8);p=add(mul(a,1-t),mul(b,t))
  if not 1<p[1]<33:continue
  choices.append((p,norm(add(n1,n2)),norm(add(b,mul(a,-1))),length))
 if not choices:return chip(g,rng)
 p,n,u,_=rng.choices(choices,weights=[v[3] for v in choices],k=1)[0];v=norm(cross(n,u));radius=rng.uniform(2,5);depth=rng.uniform(.8,2.4)
 pts=[add(p,add(mul(n,depth*2.5),add(mul(u,math.cos(a)*radius),mul(v,math.sin(a)*radius)))) for a in [0,2.2,4.3]]
 pts.append(add(p,mul(n,-depth)));return pts
