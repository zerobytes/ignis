#!/usr/bin/env python3
"""Read-only comparison of global and regional projected IGC1 certificates."""
import struct,math,json
from pathlib import Path
p=Path(__file__).resolve().parents[1]/'artifacts/appearance-probe/external-recovery'
b=(p/'stable-100.igc').read_bytes()
assert b[:4]==b'IGC1' and struct.unpack_from('<I',b,4)[0]==1
size,expected=struct.unpack_from('<QQ',b,8);assert size==len(b)-24
h=0xcbf29ce484222325
for v in b[24:]:h=((h^v)*0x100000001b3)&((1<<64)-1)
assert h==expected
shared=struct.unpack_from('<I',b,32)[0];off=36
for _ in range(1 if shared else 2):
 v,t=struct.unpack_from('<II',b,off);off+=8+24*v+16*t
n,=struct.unpack_from('<I',b,off);off+=4
assert shared in [0,1] and off+n*48==len(b)
regions=[struct.unpack_from('<8f4I',b,off+i*48) for i in range(n)]
def bound(rs):return ([min(r[i] for r in rs) for i in range(3)],max(r[3] for r in rs),[max(r[4+i] for r in rs) for i in range(3)])
def projected(b,angle):
 lo,e,hi=b;eye=[math.cos(angle)*.85,math.sin(angle)*.85,.53];norm=math.sqrt(sum(x*x for x in eye));eye=[x/norm*2.7 for x in eye];fw=[-x/2.7 for x in eye];depth=math.inf;radius=0
 for bits in range(8):
  v=[(hi[i] if bits&(1<<i) else lo[i])-eye[i] for i in range(3)];z=sum(a*f for a,f in zip(v,fw));depth=min(depth,z);radius=max(radius,math.sqrt(sum((a-z*f)**2 for a,f in zip(v,fw))))
 return 2234/(2*math.tan(math.pi/8))*e*(1+radius/depth)/(depth-e)
whole=bound(regions);bs=[bound([r]) for r in regions];rows=[]
for frame in range(32):
 angle=frame*math.tau/32;loc=[projected(x,angle) for x in bs];rows.append({'frame':frame,'global_px':projected(whole,angle),'local_max_px':max(loc),'local_failures':sum(x>.15 for x in loc),'source_faces_local_fail':sum(r[8] for r,x in zip(regions,loc) if x>.15)})
out={'regions':n,'max_error':whole[1],'poses':rows,'note':'Read-only f64 audit of existing f32 certificates; not a measured runtime selector.'};(p/'certificate-audit.json').write_text(json.dumps(out,indent=2)+'\n');print('regions',n,'error',whole[1]);print('global fails',sum(r['global_px']>.15 for r in rows),'local fails',max(r['local_failures'] for r in rows),'maximums',max(r['global_px'] for r in rows),max(r['local_max_px'] for r in rows))
