"""Shared bedding/fault controls. Native Landform and CSG own all mesh topology."""
import math
from plateau_stains import document
from weathered_plateau import norm,add,mul

def formation(params):
 s=document(params);s['version']=2;s['pieces']=s['pieces'][:1];p=s['pieces'][0]
 p['axis']=norm([.34,1,.08]);p['profile_x']=norm([1,-.34,0]);p['anchor']=[0,0,0]
 n=len(p['profile']);rows=[];at=-28
 # Same boundary indices and broad fracture valleys persist through every stratum.
 for k,thickness in enumerate([7,4,9,3,6,8,4,7,5,9,6,8,7]):
  resistance=[1.025,.975,1.01,.96,1.018,.982][k%6]
  for t,retreat in [(0,1),(.7,.986)]:
   h=at+thickness*t
   factors=[(1-.048*math.sin(j*math.tau/n*3+.4)**8)*(1+.035*math.sin(j*math.tau/n*2+h*.035)+.024*math.sin(j*math.tau/n*3+k*.72)) for j in range(n)]
   rows.append(dict(at=h,scale=resistance*retreat,offset=[.7*math.sin(h*.035),.35*math.sin(h*.06)],radial_scales=factors))
  at+=thickness
 rows.append(dict(at=at,scale=1,offset=[0,0],radial_scales=[1]*n));p['sections']=rows
 return s

def box(lo,hi):return [[hi[d] if i&(1<<d) else lo[d] for d in range(3)] for i in range(8)]

def fractures():
 # Three related steep joints; finite mouth opens on front, root ends in the mass.
 result=[]
 for x,z,depth,length in [(-29,-32,10,31),(-5,-35,8,44),(23,-29,11,34)]:
  axis=norm([.21,1,.04]);origin=[x,-3,z]
  triangle=[[-1.25,0,-8],[1.15,0,-8],[.1,0,depth]]
  result.append([add(add(origin,mul(axis,h)),q) for h in [0,length] for q in triangle])
 return result
