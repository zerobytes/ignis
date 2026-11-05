"""Locally interrupted strata exposure; native tools compile/cut all surfaces."""
import math,random
from directional_plateau import formation as directional
from weathered_plateau import add,mul,norm,cross,surface_face,dot

def bell(angle,center,width):
 d=math.atan2(math.sin(angle-center),math.cos(angle-center))
 return math.exp(-(d/width)**2)

def formation(params):
 s=directional(params);p=s['pieces'][0];n=len(p['profile'])
 # No global alternating section scale. Exposure is a local field, so a layer
 # remains inside the mass even where its ledge disappears at the surface.
 rng=random.Random(params['seed']+935);phase=rng.uniform(-math.pi,math.pi)
 for i,row in enumerate(p['sections']):
  k=i//2;h=row['at'];erosion=(i%2)*[.065,.025,.085,.012,.047,.09,.018][k%7]
  factors=[]
  for j in range(n):
   a=j*math.tau/n
   preserved=.035*math.sin(2*a+.022*h)+.04*math.sin(3*a-.035*h+phase)
   preserved+=.035*math.sin(7*a+.09*h)*bell(a,-.75,.9)
   exposure=min(1,bell(a,-1.1+.48*math.sin(k*.83),.35)+.8*bell(a,1.8+.42*math.cos(k*.65),.55))
   # Broad localized lost masses; shared by adjacent heights, not white noise.
   scar=.10*bell(a,-2.5+.008*h,.52)*math.exp(-((h-16)/15)**2)
   scar+=.065*bell(a,.25-.004*h,.7)*math.exp(-((h-30)/11)**2)
   factors.append(1+preserved-erosion*exposure**3-scar)
  row['scale']=1;row['radial_scales']=factors
  row['offset']=[1.15*math.sin(h*.03),.65*math.sin(h*.055)]
 return s

def joint(g,rng):
 p,n=surface_face(g,rng);u=norm(cross([0,1,0],n));v=norm(cross(n,u))
 # Shared steep family with modest angular scatter; finite pointed endpoints.
 axis=norm(add(v,mul(u,rng.uniform(.08,.3))))
 length=rng.uniform(5,15);width=rng.uniform(.25,.8);depth=rng.uniform(.8,2.6)
 result=[]
 for at in [-length*.28,length*.28]:
  c=add(p,mul(axis,at))
  result.extend([add(c,add(mul(u,-width),mul(n,depth))),add(c,add(mul(u,width),mul(n,depth))),add(c,mul(n,-depth))])
 result.extend([add(add(p,mul(axis,s*length*.5)),mul(n,depth*.6)) for s in [-1,1]])
 return result

def spall(g,rng):
 normals=[];edges={}
 for face,tri in enumerate(g['triangles']):
  a,b,c=[g['positions'][i] for i in tri]
  normals.append(norm(cross(add(b,mul(a,-1)),add(c,mul(a,-1)))))
  for i,j in zip(tri,tri[1:]+tri[:1]):edges.setdefault(tuple(sorted([i,j])),[]).append(face)
 choices=[]
 for (i,j),faces in edges.items():
  if len(faces)!=2:continue
  a,b=[g['positions'][v] for v in [i,j]];delta=add(b,mul(a,-1));length=math.sqrt(dot(delta,delta))
  n1,n2=[normals[f] for f in faces]
  if length<5 or dot(n1,n2)>.85:continue
  opposite=next(v for v in g['triangles'][faces[1]] if v not in [i,j])
  if dot(n1,add(g['positions'][opposite],mul(a,-1)))>=-.02:continue
  t=rng.uniform(.25,.75);p=add(mul(a,1-t),mul(b,t))
  if 3<p[1]<31 and dot(add(n1,n2),add(n1,n2))>.05:
   choices.append((p,norm(add(n1,n2)),norm(delta),length))
 if not choices:return joint(g,rng)
 p,n,u,_=rng.choices(choices,weights=[v[3]**1.5 for v in choices],k=1)[0];v=norm(cross(n,u))
 width=rng.uniform(5,11);height=rng.uniform(2,4);depth=rng.uniform(.8,2.7)
 # Cut across an existing convex edge, avoiding isolated triangular wall holes.
 return [add(p,add(mul(u,x*width),add(mul(v,y*height),mul(n,z*depth))))
         for x in [-1,1] for y in [-1,1] for z in [-rng.uniform(.65,1.1),2]]

if __name__=='__main__':
 from render_directional_plateau import main
 main(formation_builder=formation,fracture_builder=lambda:[],output='patchy-plateau',configuration=dict(id='patchy',contour_points=32),joint_builder=joint,spall_builder=spall)
