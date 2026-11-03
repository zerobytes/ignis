"""E02 authorship from the accepted inventory and image observations, via public MCP."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import random
import shutil

import numpy as np
from reconstruction_structure import building_core_preflight

ROOT = Path(__file__).resolve().parents[1] / 'artifacts/house-e02-rebuild'
PROJECT = Path(__file__).resolve().parents[1]
BINARY = '/Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis'
SOURCE = Path('/Volumes/Zero/EpicGames/Unreal Projects/LandOfChaos/docs/art/settlements/residential-concepts-v2/E02.png')
CAMERA = {'position':[12,11,20], 'target':[0,3.5,0], 'fov_degrees':34}
WIDTH, HEIGHT = 1402, 1122


def save(name, value):
    ROOT.mkdir(parents=True, exist_ok=True)
    (ROOT/name).write_text(json.dumps(value, indent=2, ensure_ascii=False, allow_nan=False)+'\n')


class Engine:
    def __init__(self):
        ROOT.mkdir(parents=True, exist_ok=True)
        self.log = (ROOT/'stderr.log').open('w')
        self.p = subprocess.Popen([BINARY, '--root', str(ROOT), '--mcp'],
             stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.log, text=True)
        self.seq = 0
        self.revision = None
        self.init = self.rpc('initialize', {'protocolVersion':'2024-11-05', 'capabilities':{},
                'clientInfo':{'name':'e02-inventory-build','version':'1'}})
        self.p.stdin.write(json.dumps({'jsonrpc':'2.0','method':'notifications/initialized'})+'\n')
        self.p.stdin.flush()

    def rpc(self, method, params):
        self.seq += 1
        self.p.stdin.write(json.dumps({'jsonrpc':'2.0','id':self.seq,'method':method,'params':params},
                                     allow_nan=False)+'\n')
        self.p.stdin.flush()
        while True:
            line = self.p.stdout.readline()
            if not line: raise RuntimeError('Ignis closed; see stderr.log')
            r = json.loads(line)
            if r.get('id') != self.seq: continue
            if 'error' in r: raise RuntimeError(r['error'])
            return r['result']

    def command(self, op, **kw):
        args = {'command':{'op':op, **kw}}
        if self.revision is not None: args['expected_revision'] = self.revision
        r = self.rpc('tools/call', {'name':'engine_command','arguments':args})
        if r.get('isError'): raise RuntimeError(r)
        data = r.get('structuredContent')
        if data is None: data = json.loads(next(c['text'] for c in r['content'] if c['type']=='text'))
        if 'revision' in data and op not in ['capture_replay','run_replay','inspect_replay']:
            self.revision = data['revision']
        return data

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=30)
        self.log.close()


def probe():
    e = Engine()
    try:
        save('schema.json', e.rpc('tools/list', {}))
        save('initialize.json', e.init)
        r = e.command('inspect_meshes'); save('limits.json', r['limits']); print(json.dumps(r))
        schema = json.loads((ROOT/'schema.json').read_text())['tools'][0]['inputSchema']['$defs']
        for key in ['Material','Light','DirectionalShadow','Pattern','Detail','Definition','Station']:
            print(key, json.dumps(schema.get(key)))
    finally:e.close()


def unit(v):
    v=np.array(v,dtype=float);return v/np.linalg.norm(v)


class Camera:
    def __init__(self):
        self.eye=np.array(CAMERA['position'],float)
        self.forward=unit(np.array(CAMERA['target'])-self.eye)
        self.right=unit(np.cross(self.forward,[0,1,0]))
        self.up=unit(np.cross(self.right,self.forward))
        self.focal=HEIGHT/(2*math.tan(math.radians(CAMERA['fov_degrees']/2)))

    def ray(self,p):
        return self.forward+self.right*((p[0]-WIDTH/2)/self.focal)+self.up*((HEIGHT/2-p[1])/self.focal)

    def on_plane(self,p,origin,normal):
        r=self.ray(p);normal=np.array(normal)
        t=np.dot(np.array(origin)-self.eye,normal)/np.dot(r,normal)
        if t<=0:raise ValueError('Plane intersection behind camera')
        return self.eye+t*r

    def floor(self,p,y=.55):return self.on_plane(p,[0,y,0],[0,1,0])

    def project(self,p):
        d=np.array(p)-self.eye;z=np.dot(d,self.forward)
        return np.array([WIDTH/2+self.focal*np.dot(d,self.right)/z,
                         HEIGHT/2-self.focal*np.dot(d,self.up)/z])


CAM=Camera()


class Plane:
    def __init__(self,a,b,y=.55):
        self.origin=CAM.floor(a,y);self.axis=unit(CAM.floor(b,y)-self.origin)
        self.normal=unit(np.cross(self.axis,[0,1,0]))
        if np.dot(self.normal,CAM.eye-self.origin)<0:self.normal=-self.normal

    def __call__(self,p,depth=0):return CAM.on_plane(p,self.origin+self.normal*depth,self.normal)

    def uv(self,p):
        v=np.array(p)-self.origin
        return [float(np.dot(v,self.axis)),float(v[1])]


def cubic(points,count=12):
    a,b,c,d=map(np.array,points)
    return [(1-t)**3*a+3*(1-t)**2*t*b+3*(1-t)*t*t*c+t**3*d for t in np.linspace(0,1,count+1)]


def signed_area(p):return sum(p[i][0]*p[(i+1)%len(p)][1]-p[(i+1)%len(p)][0]*p[i][1] for i in range(len(p)))/2


def ears(points):
    p=[np.array(x,float) for x in points]
    ids=list(range(len(p)))
    if signed_area(p)<0:ids.reverse()
    triangles=[]
    def cross(a,b,c):return float((b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]))
    while len(ids)>3:
        found=False
        for j,b in enumerate(ids):
            a,c=ids[j-1],ids[(j+1)%len(ids)]
            if cross(p[a],p[b],p[c])<1e-8:continue
            blocked=False
            for k in ids:
                if k in [a,b,c] or any(np.linalg.norm(p[k]-p[v])<1e-7 for v in [a,b,c]):continue
                if min(cross(p[a],p[b],p[k]),cross(p[b],p[c],p[k]),cross(p[c],p[a],p[k]))>1e-8:
                    blocked=True;break
            if not blocked:
                triangles.append([a,b,c]);ids.pop(j);found=True;break
        if not found:
            # Remove a collinear bridge vertex; do not fan across a hole.
            for j,b in enumerate(ids):
                if abs(cross(p[ids[j-1]],p[b],p[ids[(j+1)%len(ids)]]))<1e-7:
                    ids.pop(j);found=True;break
            if not found:raise ValueError('Polygon needs a valid hole bridge')
    if len(ids)==3 and abs(cross(*[p[i] for i in ids]))>1e-8:triangles.append(ids)
    return triangles


def holes_polygon(outer,holes):
    out=[list(p) for p in outer]
    if signed_area(out)<0:out.reverse()
    # Bridge each rightmost hole vertex to the nearest outer boundary on its right.
    for hole in sorted(holes,key=lambda h:max(p[0] for p in h),reverse=True):
        h=[list(p) for p in hole]
        if signed_area(h)>0:h.reverse()
        k=max(range(len(h)),key=lambda i:h[i][0]);x,y=h[k]
        hits=[]
        for i,a in enumerate(out):
            b=out[(i+1)%len(out)]
            if (a[1]<=y<b[1]) or (b[1]<=y<a[1]):
                xx=a[0]+(y-a[1])*(b[0]-a[0])/(b[1]-a[1])
                if xx>=x:hits.append((xx,i))
        if not hits:raise ValueError('Opening outside wall boundary')
        xx,i=min(hits);q=[xx,y]
        out=out[:i+1]+[q]+h[k:]+h[:k+1]+[q]+out[i+1:]
    return out


class Mesh:
    def __init__(self,name,color):self.name=name;self.color=color;self.p=[];self.t=[];self.parts=[]
    def polygon(self,pts):
        pts=[np.array(x,float) for x in pts]
        if len(pts)<3:return
        n=unit(np.cross(pts[1]-pts[0],pts[2]-pts[0]))
        if np.dot(n,CAM.eye-np.mean(pts,axis=0))<0:
            pts.reverse();n=unit(np.cross(pts[1]-pts[0],pts[2]-pts[0]))
        u=unit(pts[1]-pts[0]);v=np.cross(n,u)
        projected=[[np.dot(p-pts[0],u),np.dot(p-pts[0],v)] for p in pts]
        start=len(self.p);self.p.extend([p.tolist() for p in pts])
        self.t.extend([[start+i for i in tri] for tri in ears(projected)])
    def poly2(self,points,plane,depth=0,holes=None):
        if holes:
            # Horizontal trapezoid decomposition keeps each opening genuinely empty.
            # Slabs are bounded at every contour vertex, so edge order is constant.
            contours=[points,*holes];ys=sorted(set(float(p[1]) for c in contours for p in c))
            def edges_at(c,y):
                edges=[]
                for a,b in zip(c,list(c[1:])+[c[0]]):
                    if min(a[1],b[1])<y<max(a[1],b[1]):
                        edges.append((float(a[0]+(y-a[1])*(b[0]-a[0])/(b[1]-a[1])),a,b))
                return sorted(edges,key=lambda e:e[0])
            def x_at(e,y):
                _,a,b=e;return float(a[0]+(y-a[1])*(b[0]-a[0])/(b[1]-a[1]))
            for y0,y1 in zip(ys,ys[1:]):
                if y1-y0<1e-7:continue
                ym=(y0+y1)/2;edges=edges_at(points,ym)
                spans=list(zip(edges[::2],edges[1::2]))
                for hole in holes:
                    he=edges_at(hole,ym)
                    for left,right in zip(he[::2],he[1::2]):
                        nxt=[]
                        for a,b in spans:
                            if right[0]<=a[0] or left[0]>=b[0]:nxt.append((a,b));continue
                            if left[0]>a[0]:nxt.append((a,left))
                            if right[0]<b[0]:nxt.append((right,b))
                        spans=nxt
                for a,b in spans:
                    ps=[[x_at(a,y0),y0],[x_at(b,y0),y0],[x_at(b,y1),y1],[x_at(a,y1),y1]]
                    start=len(self.p);self.p.extend([plane(p,depth).tolist() for p in ps])
                    self.t.extend([[start+2,start+1,start],[start+3,start+2,start]])
            return
        points=holes_polygon(points,holes or [])
        start=len(self.p);self.p.extend([plane(p,depth).tolist() for p in points])
        self.t.extend([[start+i for i in reversed(tri)] for tri in ears(points)])
    def box(self,center,size):
        c=np.array(center);s=np.array(size)/2
        vs=[c+s*np.array([x,y,z]) for x,y,z in [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]]
        n=len(self.p);self.p.extend([p.tolist() for p in vs])
        for a,b,c,d in [(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(0,1,5,4),(3,7,6,2)]:
            self.t.extend([[n+a,n+b,n+c],[n+a,n+c,n+d]])
    def tube(self,pts,radius,sides=4,taper=1):
        sides=min(sides,3)
        pts=[np.array(p,float) for p in pts];start=len(self.p)
        for i,p in enumerate(pts):
            tangent=unit(pts[min(i+1,len(pts)-1)]-pts[max(0,i-1)])
            a=unit(np.cross(tangent,[0,0,1] if abs(tangent[2])<.8 else [0,1,0]));b=np.cross(tangent,a)
            rr=radius*(1+(taper-1)*i/max(1,len(pts)-1))
            for j in range(sides):self.p.append((p+rr*(a*math.cos(j*2*math.pi/sides)+b*math.sin(j*2*math.pi/sides))).tolist())
        for i in range(len(pts)-1):
            for j in range(sides):
                a=start+i*sides+j;b=start+i*sides+(j+1)%sides;c=b+sides;d=a+sides
                self.t.extend([[a,b,c],[a,c,d]])
    def stroke(self,pts,plane,width=3,depth=.025,sides=4,taper=1):
        # Simplify curve tessellation in reference pixels, preserving endpoints and bends.
        def simplify(ps,tolerance=.9):
            if len(ps)<3:return ps
            a,b=np.array(ps[0]),np.array(ps[-1]);d=b-a
            ds=[np.linalg.norm(np.array(p)-a-d*np.clip(np.dot(np.array(p)-a,d)/max(np.dot(d,d),1e-12),0,1)) for p in ps]
            i=int(np.argmax(ds))
            return simplify(ps[:i+1])+simplify(ps[i:])[1:] if ds[i]>tolerance else [ps[0],ps[-1]]
        pts=simplify(pts)
        mid=pts[len(pts)//2]
        # A circular stroke uses camera depth for its projected radius. Differencing
        # two ray/plane intersections explodes at grazing hosts (curved rail ends).
        radius=width*float(np.dot(plane(mid,depth)-CAM.eye,CAM.forward))/CAM.focal
        self.tube([plane(p,depth) for p in pts],radius,sides,taper)
    def mark(self,start,ids,name):
        self.parts.append({'name':name,'items':['E02-'+i if not i.startswith('E02-') else i for i in ids],
                           'vertices':[start,len(self.p)]})
    def weld(self):
        unique={};ps=[];mapping=[]
        for p in self.p:
            key=tuple(round(float(x),7) for x in p)
            if key not in unique:unique[key]=len(ps);ps.append(list(key))
            mapping.append(unique[key])
        self.t=[[mapping[i] for i in t] for t in self.t]
        self.t=[t for t in self.t if len(set(t))==3 and np.linalg.norm(np.cross(np.array(ps[t[1]])-ps[t[0]],np.array(ps[t[2]])-ps[t[0]]))>1e-9]
        for part in self.parts:
            a,b=part.pop('vertices');part['vertex_indices']=sorted(set(mapping[a:b]))
        self.p=ps


PALETTE={'stone':[.64,.57,.46],'trim':[.79,.72,.59], 'gold':[.49,.32,.13],
         'wood':[.24,.12,.045], 'shadow':[.055,.04,.022], 'roof':[.20,.25,.22],
         'roof-var':[.28,.31,.26], 'inlay':[.11,.26,.22], 'leaf':[.065,.17,.026],
         'leaf-var':[.22,.32,.055], 'white':[.92,.86,.68], 'purple':[.43,.23,.48],
         'light':[1,.63,.22], 'soil':[.19,.16,.09]}


def arch(left,bottom,right,spring,apex,count=10):
    return [[left,bottom]]+cubic([[left,spring],[left,spring-(spring-apex[1])*.45],
                  [apex[0]-(apex[0]-left)*.45,apex[1]+10],apex],count)+cubic([apex,
                  [apex[0]+(right-apex[0])*.45,apex[1]+10],[right,spring-(spring-apex[1])*.45],[right,spring]],count)[1:]+[[right,bottom]]


def build(stage='structure'):
    m={k:Mesh('rebuild-'+k,c) for k,c in PALETTE.items()};stone=m['stone'];trim=m['trim'];gold=m['gold'];wood=m['wood']
    main=Plane([860,750],[1150,758]);side=Plane([1150,758],[1310,710])
    wing=Plane([212,705],[795,685]);porch=Plane([170,733],[575,697])
    wing.origin-=wing.normal*.65  # rear wall sits behind the independent front support line
    roof_depth=abs(float(np.dot(CAM.floor([1300,710])-main.origin,main.normal)))
    plan={'camera':CAMERA,'main':{'origin':main.origin.tolist(),'normal':main.normal.tolist()},
          'side':{'origin':side.origin.tolist(),'normal':side.normal.tolist()},
          'wing':{'origin':wing.origin.tolist(),'normal':wing.normal.tolist()},'roof_depth':roof_depth}
    save('spatial-plan.json',plan)
    # Real openings are removed from host-plane wall polygons.
    great=arch(907,703,1106,513,[1008,393],12)
    tall=arch(821,665,876,395,[853,278],8)
    wall=[[798,755],[1159,770],[1168,443]]+list(reversed(cubic([[705,80],[780,130],[1030,390],[1168,443]],16)[:-1]))+[[750,295],[798,450]]
    start=len(stone.p);stone.poly2(wall,main,holes=[great,tall]);stone.mark(start,['A01','F01','F02','F10','L17'],'main-facade-openings')
    sw1=arch(1152,697,1220,556,[1187,493],8);sw2=arch(1241,677,1286,546,[1262,514],8)
    start=len(stone.p);stone.poly2([[1143,773],[1308,725],[1308,482],[1143,453]],side,holes=[sw1,sw2]);stone.mark(start,['M01','M03','M04','M09','M11'],'right-facade')
    door=arch(686,675,783,570,[735,511],10);leftdoor=arch(219,697,282,596,[250,554],8)
    low_windows=[arch(346,655,390,584,[369,545],8),arch(438,643,482,568,[460,531],8),arch(595,625,644,556,[620,518],8)]
    start=len(stone.p);stone.poly2([[171,715],[818,706],[818,485],[170,520]],wing,holes=[door,leftdoor,*low_windows]);stone.mark(start,['A02','I01','I02','I07','J01','K01'],'lower-host-wall')
    # Gabled fields and window sills are actual separate layers.
    fields=[[[911,247],[1006,352],[928,370],[912,347]],[[1033,355],[1109,408],[1050,397]]]
    for p in fields:m['inlay'].poly2(p,main,.015)
    for outline,host,tag in [(great,main,'great'),(tall,main,'tall'),(sw1,side,'side1'),(sw2,side,'side2'),(door,wing,'door'),(leftdoor,wing,'left-door'),*[(p,wing,'low-window') for p in low_windows]]:
        s=len(trim.p);trim.stroke(outline,host,8 if tag=='great' else 4,.025,4)
        trim.stroke(outline,host,4 if tag=='great' else 1.6,.09,4)
        trim.mark(s,{'great':['L02','L03','L04','L05'],'tall':['F03','F04'],
                      'side1':['M03','M05'],'side2':['M04','M05'],'door':['J02','J03'],
                      'left-door':['K02'],'low-window':['I03','I04']}[tag],tag+'-mouldings')
        if tag in ['door','left-door']:
            s=len(wood.p);wood.poly2(outline,host,-.055);wood.mark(s,['J04','J05'] if tag=='door' else ['K03','K05'],tag+'-wood')
    # Gallery modules are hosted on one plane, independently of the high tall opening.
    gallery_plane=Plane([510,430],[810,457],3.7)
    gallery=[]
    for cx,base,tip in [(514,414,378),(570,429,367),(634,440,355),(698,448,343),(765,455,332)]:
        p=arch(cx-23,base,cx+23,tip+24,[cx,tip],6);gallery.append(p)
    s=len(stone.p);stone.poly2([[485,422],[814,468],[816,320],[485,353]],gallery_plane,holes=gallery);stone.mark(s,['G01','G02','G04','G09'],'gallery-wall')
    s=len(trim.p)
    for p in gallery:trim.stroke(p,gallery_plane,3,.03,3)
    trim.mark(s,['G03'],'gallery-mouldings')
    # Roofs are swept between observed front/back profiles on parallel depth planes.
    roofs=[]
    def roof(name,top,bottom,left,right,host,depth,nu,nv,ids):
        nu,nv=max(5,nu//2),max(5,nv//2)
        a,b=np.array(top[0]),np.array(top[-1]);d,c=np.array(bottom[0]),np.array(bottom[-1])
        def bez(q,t):
            q=np.array(q);return (1-t)**3*q[0]+3*(1-t)**2*t*q[1]+3*(1-t)*t*t*q[2]+t**3*q[3]
        def sample(u,v):
            p=(1-v)*bez(top,u)+v*bez(bottom,u)+(1-u)*bez(left,v)+u*bez(right,v)-((1-u)*(1-v)*a+u*(1-v)*b+(1-u)*v*d+u*v*c)
            return host(p,.10-depth*u)
        base=m['roof'];start=len(base.p)
        for i in range(nu+1):
            for j in range(nv+1):base.p.append((sample(i/nu,j/nv)-[0,.20,0]).tolist())
        for i in range(nu):
            for j in range(nv):
                k=start+i*(nv+1)+j;base.t.extend([[k,k+1,k+nv+2],[k,k+nv+2,k+nv+1]])
        base.mark(start,ids,name)
        # Solid roof-edge strips, not a flat image panel.
        for pts in [[sample(t,0) for t in np.linspace(0,1,15)], [sample(t,1) for t in np.linspace(0,1,15)],
                    [sample(0,t) for t in np.linspace(0,1,19)], [sample(1,t) for t in np.linspace(0,1,15)]]:
            trim.tube(pts,.060,4);gold.tube([p+np.array([0,.044,0]) for p in pts],.012,3)
        roofs.append((name,sample,nu,nv))
    roof('main',[[705,80],[820,128],[1030,150],[1168,146]],[[1168,443],[1235,464],[1318,465],[1358,454]],
         [[705,80],[780,130],[1030,390],[1168,443]],[[1168,146],[1250,257],[1335,455],[1358,454]],main,roof_depth,15,15,['B01','B02','B03','B04','B05','B06','B07'])
    roof('upper-wing',[[207,292],[340,350],[513,285],[561,220]],[[820,343],[824,337],[819,306],[813,272]],
         [[207,292],[330,369],[650,369],[820,343]],[[561,220],[645,249],[759,259],[813,272]],wing,1.9,10,9,['C01','C02','C03','C04','C05'])
    roof('porch',[[210,310],[320,385],[590,438],[818,463]],[[95,490],[200,620],[500,422],[817,493]],
         [[210,310],[201,392],[65,455],[95,490]],[[818,463],[815,473],[815,483],[817,493]],porch,-1.55,17,10,['D01','D02','D03','D04','D05','D06','D07','D08'])
    # Courtyard footprint and separate stair stacks, all horizontal and extruded.
    ground=lambda p,depth=0:CAM.floor(p,.55+depth)
    patio=[[145,746],[245,708],[540,680],[794,680],[1020,708],[1094,760],[1050,820],[865,875],[730,902],[550,879],[310,812],[161,786]]
    s=len(stone.p);stone.poly2(patio,ground);stone.mark(s,['A04','Q01','Q14'],'patio-platform')
    for p,q in zip(patio,patio[1:]+patio[:1]):
        a,b=ground(p),ground(q);stone.polygon([a,b,b-[0,.5,0],a-[0,.5,0]])
    def steps(center,spans,y0,count,tag):
        # Pixel coordinates specify each projected horizontal tread boundary.
        for j in range(count):
            cy=center[1]-j*spans[2];rx=spans[0]-j*spans[3];ry=spans[1]-j*spans[4]
            outer=[[center[0]+rx*math.cos(t),cy+ry*math.sin(t)] for t in np.linspace(0,math.pi,19)]
            inner=[[center[0]+(rx-spans[3])*math.cos(t),cy-spans[2]+(ry-spans[4])*math.sin(t)] for t in np.linspace(math.pi,0,19)]
            plane=lambda p,depth=0,j=j:CAM.floor(p,y0+j*.145+depth)
            start=len(stone.p);stone.poly2(outer+inner,plane)
            for a,b in zip(outer,outer[1:]):
                aa,bb=plane(a),plane(b);stone.polygon([aa,bb,bb-[0,.145,0],aa-[0,.145,0]])
            stone.mark(start,tag,'stair-tread-'+str(j))
    steps([705,936],[185,43,22,16,2],.11,3,['Q10','Q11'])
    steps([733,675],[99,28,9,11,2],.65,3,['J10','J11','J12'])
    # Native profile columns are derived from their observed feet/top projections.
    columns=[]
    for p,top in [([171,729],541),([294,739],522),([430,719],503),([575,682],486)]:
        foot=CAM.floor(p);lo,hi=0,8
        for _ in range(35):
            mid=(lo+hi)/2
            if CAM.project(foot+[0,mid,0])[1]>top:lo=mid
            else:hi=mid
        height=(lo+hi)/2;columns.append({'foot':foot.tolist(),'height':height})
    save('columns.json',columns)
    # Close non-visible sides with an explicit hypothesis behind the visible hosts.
    for plane_,outline in [(main,wall),(wing,[[171,715],[818,706],[818,485],[170,520]])]:
        for a,b in zip(outline,outline[1:]+outline[:1]):
            pa,pb=plane_(a,-.18),plane_(b,-.18)
            # No caps across window openings: only outer perimeter thickness.
            stone.polygon([plane_(a),plane_(b),pb,pa])
    hosts={'main':main,'side':side,'wing':wing,'porch':porch,'gallery':gallery_plane}
    openings={'great':great,'tall':tall,'side1':sw1,'side2':sw2,'door':door,'leftdoor':leftdoor,'low':low_windows,'gallery':gallery}
    if stage=='detail':details(m,hosts,roofs,openings)
    return m,columns,hosts,roofs,openings


def details(m,h,roofs,o):
    """Observed component families; dimensions are image anchors, depth is a hypothesis."""
    rng=random.Random(20261027)
    main,side,wing,porch,gallery=[h[k] for k in ['main','side','wing','porch','gallery']]
    stone,trim,gold,wood=[m[k] for k in ['stone','trim','gold','wood']]
    floor=lambda p,depth=0:CAM.floor(p,.55+depth)
    def record(name,ids,fn):
        starts={k:len(v.p) for k,v in m.items()};fn()
        for k,v in m.items():
            if len(v.p)>starts[k]:v.mark(starts[k],ids,name)
    def curve(mesh,points,host,width=2,depth=.08,n=5):
        mesh.stroke(cubic(points,n),host,width,depth,3)
    def line(mesh,points,host,width=2,depth=.08):mesh.stroke(points,host,width,depth,3)
    def ring(mesh,c,rx,ry,host,width=2,depth=.03,n=28):
        line(mesh,[[c[0]+rx*math.cos(t),c[1]+ry*math.sin(t)] for t in np.linspace(0,2*math.pi,n+1)],host,width,depth)
    def tree(host,cx,base,top,half,mesh,width=1.5,depth=.10,levels=2,fine=False):
        # Curved central stem, paired branches and terminal forks. Each host varies.
        height=base-top
        curve(mesh,[[cx,base],[cx-2,base-height*.35],[cx+2,top+height*.2],[cx,top]],host,width,depth,6)
        for level in range(levels):
            y=base-height*(.23+.27*level)
            for sign in [-1,1]:
                end=[cx+sign*half*(1-.17*level),top+height*(.30-.12*level)]
                curve(mesh,[[cx,y],[cx+sign*half*.12,y-height*.18],[end[0],end[1]+height*.15],end],host,width*.76,depth,5)
                if fine:
                    fork=[end[0]-sign*half*.27,end[1]-height*.13]
                    curve(mesh,[[cx+sign*half*.65,y-height*.2],[end[0],end[1]+height*.12],fork,fork],host,width*.43,depth,3)
        return None
    # Each tile is a separate shallow quadrilateral on the curved 3D support.
    def tiles():
        for name,sample,_,_ in roofs:
            nu,nv={'main':(20,12),'upper-wing':(17,6),'porch':(23,6)}[name]
            for j in range(nv):
                for i in range(nu):
                    u0=max(0,(i-(j%2)*.5)/nu);u1=min(1,(i+1-(j%2)*.5)/nu)
                    v0=j/nv+.002;v1=min(1,(j+1.12)/nv)
                    pts=[sample(u0+.002,v0),sample(u1-.002,v0),sample(u1-.002,v1),sample(u0+.002,v1)]
                    pts=[p+np.array([0,.012+(k>1)*.012,0]) for k,p in enumerate(pts)]
                    target=m['roof-var'] if rng.random()<.55 else m['roof']
                    target.polygon(pts)
    record('staggered-curved-roof-tiles',['C05','D10','E01','E02','E03','E04'],tiles)
    # Fascia, eave thickness and rafters are distinct from the tiles.
    def roof_edges():
        for name,sample,_,_ in roofs:
            for boundary in [lambda t:sample(0,t),lambda t:sample(t,1)]:
                pts=[boundary(t) for t in np.linspace(0,1,20)]
                trim.tube([p-[0,.075,0] for p in pts],.055,4)
                wood.tube([p-[0,.13,0] for p in pts],.044,3)
            # Under the visible front edge: a continuous narrow underside strip.
            pts=[sample(0,t) for t in np.linspace(0,1,20)]
            for a,b in zip(pts,pts[1:]):wood.polygon([a,b,b-[0,.16,0],a-[0,.16,0]])
        for i in range(7):
            x=727+i*13;y=124+i*25
            line(wood,[[x,y],[x+17,y+49]],main,3,-.12)
        for p in [[[758,186],[794,286]],[[778,223],[820,326]]]:line(wood,p,main,4,.03)
        wood.poly2([[715,102],[789,212],[805,316],[764,233]],main,.15)
    record('eaves-undersides-rafters',['B07','B08','B09','B10','C04','C06','D06','D08','D12'],roof_edges)
    def finials():
        for x,y,hh,host in [(707,83,63,main),(1168,148,42,main),(561,223,43,wing),(208,299,67,porch),(1205,267,42,main)]:
            line(gold,[[x,y+8],[x-5,y-hh]],host,2,.15)
            for sign in [-1,1]:
                curve(gold,[[x,y],[x+sign*12,y-12],[x+sign*16,y-hh*.6],[x-3,y-hh*.78]],host,2.2,.15,4)
                m['inlay'].poly2([[x,y-7],[x+sign*10,y-24],[x-2,y-hh*.7]],host,.14)
    record('five-individual-finials',['E05','E06','E07','E08','N06'],finials)
    # Recessed interior surfaces do not close the actual masonry openings.
    def interiors():
        for key,host,depth in [('great',main,-.75),('tall',main,-.6),('side1',side,-.45),('side2',side,-.45)]:
            m['shadow'].poly2(o[key],host,depth)
        for p in o['low']:m['shadow'].poly2(p,wing,-.4)
        for p in o['gallery']:m['shadow'].poly2(p,gallery,-.35)
        for name in ['great','tall','side1','side2','door','leftdoor']:
            host=main if name in ['great','tall'] else side if name.startswith('side') else wing
            p=o[name]
            for a,b in zip(p,p[1:]+p[:1]):stone.polygon([host(a),host(b),host(b,-.2),host(a,-.2)])
        # Main-window shelving and visible rectangular volumes, at separate depths.
        for y in [604,644,682]:line(wood,[[922,y],[1085,y]],main,3,-.48)
        for x in [932,962,1044,1077]:line(wood,[[x,560],[x,685]],main,2,-.5)
        for i in range(12):
            x=947+i*10;y=630 if i<6 else 594
            m['trim' if i%3==0 else 'wood'].poly2([[x,y],[x+6,y],[x+6,y-15-i%4*3],[x,y-15-i%4*3]],main,-.43)
        curve(wood,[[925,660],[909,615],[954,620],[957,661]],main,4,-.4,6)
    record('opening-reveals-and-interior-layers',['I06','I08','G07','L18','R09','R10','R12','R13'],interiors)
    def window_trees():
        # The large pale network has its own graph, independent of the small-window grammar.
        curve(trim,[[1008,701],[1007,614],[1008,509],[1008,410]],main,4,.13,9)
        for sign in [-1,1]:
            curve(trim,[[1008,586],[1008+sign*20,545],[1008+sign*79,535],[1008+sign*64,493]],main,3.8,.14,7)
            curve(trim,[[1008+sign*64,493],[1008+sign*57,475],[1008+sign*43,477],[1008+sign*45,459]],main,2.5,.14,5)
            curve(trim,[[1008,529],[1008+sign*31,497],[1008+sign*43,481],[1008+sign*28,451]],main,2.8,.14,7)
            curve(trim,[[1008,478],[1008+sign*33,454],[1008+sign*19,433],[1008,410]],main,2.1,.14,6)
        for cx in [946,975,1039,1070]:tree(main,cx,691,479 if cx in [946,1070] else 437,17,gold,.9,.065,3,True)
        tree(main,852,657,286,20,gold,1.8,.07,3,True)
        for cx,bot,top,hw in [(1185,690,502,28),(1263,671,520,17)]:tree(side,cx,bot,top,hw,gold,1.7,.08,3)
        for p in o['low']:
            p=np.array(p);tree(wing,float((p[:,0].min()+p[:,0].max())/2),float(p[:,1].max()),float(p[:,1].min()+5),17,gold,1,.07,2)
        for p in o['gallery']:
            p=np.array(p);tree(gallery,float((p[:,0].min()+p[:,0].max())/2),float(p[:,1].max()),float(p[:,1].min()+3),16,gold,.9,.07,2)
        for x,y in [(939,292),(1069,373)]:tree(main,x,y+42,y-22,18,gold,1.4,.08,2,True)
    record('distinct-window-ornament-graphs',['F05','F07','F08','G06','I05','L06','L07','L08','L09','L10','L11','L12','L13','M06'],window_trees)
    def doors():
        for cx,base,top,hw in [(735,672,518,43),(250,694,560,26)]:
            line(wood,[[cx,base],[cx,top]],wing,1.8,.10)
            for dx in [-hw*.43,hw*.43]:
                tree(wing,cx+dx*.6,base-6,top+8,hw*.45,gold,.9,.11,2,True)
                x=cx+dx
                line(gold,[[x-hw*.23,base-7],[x-hw*.23,base-37],[x+hw*.23,base-37],[x+hw*.23,base-7],[x-hw*.23,base-7]],wing,.75,.09)
            for dx in [-4,4]:line(gold,[[cx+dx,base-61],[cx+dx,base-53]],wing,1.1,.13)
    record('double-door-panels-forks-hardware',['J06','J07','J08','J09','K04','K05'],doors)
    def porch_arcs():
        for left,right,bottom,spring,tip in [(177,292,690,605,[235,550]),(300,427,696,572,[363,515]),(437,571,670,551,[508,502]),(581,810,650,543,[700,495])]:
            p=arch(left,bottom,right,spring,tip,7)[1:-1]
            line(trim,p,porch,5,.08);line(gold,p,porch,1,.13)
            for sx in [left,right]:
                curve(trim,[[sx,spring+15],[sx+(-16 if sx==left else 16),spring-34],[sx,tip[1]-5],[sx+(22 if sx==left else -22),tip[1]-15]],porch,2.4,.07,5)
        # Moulded corner pillar and sills, still maintaining clear openings.
        for host,x,y0,y1 in [(main,1134,752,474),(main,885,748,349),(side,1230,720,506),(side,1300,715,490)]:
            line(trim,[[x,y0],[x,y1]],host,7,.09)
            for y in [y0-3,y0-15,y1+14]:line(trim,[[x-10,y],[x+10,y]],host,4,.10)
        for host,p in [(main,o['great']),(side,o['side1']),(side,o['side2'])]:
            a,b=p[0],p[-1];line(trim,[a,b],host,7,.13)
            line(trim,[[a[0],a[1]+12],[b[0],b[1]+12]],host,3,.10)
    record('porch-arcs-ribs-and-profiled-piers',['A03','H07','H08','H09','H10','H12','H13','H14','L05','M02','M08','M09'],porch_arcs)
    def dormer():
        host=lambda p,d=0:main(p,-.45+d)
        p=arch(1162,374,1231,319,[1205,278],7)
        stone.poly2([[1147,385],[1245,396],[1240,302],[1205,252],[1151,307]],host,holes=[p])
        m['shadow'].poly2(p,host,-.3);line(gold,p,host,3,.08)
        tree(host,1202,374,283,24,gold,1.6,.1,2)
        # Two actual roof facets with a shared raised ridge, not a window painted on the big roof.
        for poly in [[[1102,236],[1208,255],[1150,310],[1135,327]],[[1208,255],[1241,305],[1249,391],[1228,367]]]:
            m['roof-var'].poly2(poly,host,.13);line(trim,poly,host,3,.17)
        for j in range(5):line(gold,[[1110+j*8,245+j*12],[1198,266+j*6]],host,.6,.18)
    record('dormer-volume-and-opening',['N01','N02','N03','N04','N05','N07','N08','N09','N10'],dormer)
    # Curved rail footprint is a horizontal curve; each panel gets its own local host.
    def rail_run(points,n):
        foot=cubic(points,n)
        for i,(a,b) in enumerate(zip(foot,foot[1:])):
            aa,bb=floor(a),floor(b);axis=unit(bb-aa)
            # y projected from actual horizontal base/cap heights, no screen-space panel billboard.
            local=Plane(a,b)
            base=(aa+bb)/2;length=np.linalg.norm(bb-aa)
            for yy,r in [(.03,.12),(.15,.065),(.92,.10),(1,.12)]:trim.tube([aa+[0,yy,0],bb+[0,yy,0]],r,4)
            footp=CAM.project(base+[0,.16,0]);topp=CAM.project(base+[0,.93,0]);lp=CAM.project(aa+[0,.18,0]);rp=CAM.project(bb+[0,.18,0])
            midx=float((lp[0]+rp[0])/2);bot=float(footp[1]);top=float(topp[1]);half=abs(float(rp[0]-lp[0]))*.43
            outline=arch(midx-half,bot,midx+half,top+(bot-top)*.36,[midx,top],5)
            line(trim,outline,local,2.8,.05);tree(local,midx,bot,top+3,half*.87,trim,1.6,.06,2)
            trim.tube([aa+[0,.14,0],aa+[0,.95,0]],.045,4)
    record('both-pierced-curved-rails',['O01','O02','O03','O04','O05','O06','O07','O08','O09','O10','O11','O12','O13','O14'],lambda:(rail_run([[149,802],[222,846],[387,883],[498,932]],7),rail_run([[933,963],[1008,956],[1117,903],[1091,850]],3)))
    def pot(center,host,width,height,depth=.15):
        # Revolved rounded hollow bowl, with an inset soil disk; open mouth remains visible.
        c=host(center,depth);px=np.linalg.norm(host([center[0]+width,center[1]],depth)-c)
        ph=np.linalg.norm(host([center[0],center[1]-height],depth)-c)
        rings=[]
        for yy,r in [(0,.35),(.10,.38),(.25,.65),(.78,1),(.94,1.02),(1,.90),(.86,.83)]:
            rings.append([c+np.array([px*r*math.cos(t),ph*yy,px*r*math.sin(t)]) for t in np.linspace(0,2*math.pi,9)[:-1]])
        start=len(stone.p)
        for rr in rings:stone.p.extend([p.tolist() for p in rr])
        for j in range(len(rings)-1):
            for k in range(8):
                a=start+j*8+k;b=start+j*8+(k+1)%8;stone.t.extend([[a,b,b+8],[a,b+8,a+8]])
        m['soil'].polygon(rings[-1]);return c+[0,ph,0]
    pot_centers=[]
    def posts():
        for pixel,top in [([502,954],793),([925,985],814)]:
            foot=floor(pixel);topworld=CAM.on_plane([pixel[0],top],foot,[CAM.forward[0],0,CAM.forward[2]])
            height=topworld[1]-foot[1];width=.54
            for frac,hh,ww in [(0,.14,.83),(.14,.13,.71),(.27,height-.5,.54),(height-.23,.13,.71),(height-.10,.10,.88)]:
                stone.box(foot+[0,frac+hh/2,0],[ww,hh,ww])
            face=Plane([pixel[0]-22,pixel[1]],[pixel[0]+22,pixel[1]])
            panel=arch(pixel[0]-13,pixel[1]-34,pixel[0]+13,top+57,[pixel[0],top+29],5)
            m['inlay'].poly2(panel,face,.35);line(gold,panel,face,1,.38)
            tree(face,pixel[0],pixel[1]-34,top+33,10,gold,.85,.40,2)
            # Pale side relief uses a separate small face, unlike the green front panel.
            sf=Plane([pixel[0]+23,pixel[1]],[pixel[0]+38,pixel[1]-13])
            tree(sf,pixel[0]+28,pixel[1]-34,top+29,7,trim,1.2,.15,2)
            pot_centers.append(pot([pixel[0],top-12],face,24,24,.18))
    record('gateposts-profiled-faces-and-bowls',['P01','P02','P03','P04','P05','P06','P07','P08','P09','P10','P11','P12','P13'],posts)
    def paving():
        # Voronoi cells clipped in image space, then mapped to horizontal planes.
        def cells(bounds,n,host,boundary=None):
            x0,y0,x1,y1=bounds;seeds=[np.array([rng.uniform(x0,x1),rng.uniform(y0,y1)]) for _ in range(n)]
            for i,a in enumerate(seeds):
                cell=[np.array(p,float) for p in boundary] if boundary else [np.array([x0,y0]),np.array([x1,y0]),np.array([x1,y1]),np.array([x0,y1])]
                for j,b in enumerate(seeds):
                    if i==j:continue
                    normal=b-a;mid=(a+b)/2;new=[]
                    for p,q in zip(cell,cell[1:]+cell[:1]):
                        dp=np.dot(p-mid,normal);dq=np.dot(q-mid,normal)
                        if dp<=0:new.append(p)
                        if (dp<=0)!=(dq<=0):new.append(p+(q-p)*dp/(dp-dq))
                    cell=new
                    if not cell:break
                if cell:
                    center=np.mean(cell,axis=0);cell=[center+(p-center)*.965 for p in cell]
                    m['stone' if i%4 else 'trim'].poly2(cell,host,.006)
        # Exterior cells are a deliberately bounded paving apron, not an infinite floor.
        cells([285,969,1110,1032],45,lambda p,d=0:CAM.floor(p,.025+d),[[285,991],[389,964],[925,964],[1110,1007],[1050,1040],[500,1041]])
        cells([160,700,1090,900],62,floor,[[160,752],[545,690],[795,690],[1090,760],[1040,820],[855,875],[730,900],[550,877],[310,812]])
        def ribbon(points,width):
            ps=cubic(points,18);left=[];right=[]
            for j,p in enumerate(ps):
                tangent=unit(ps[min(j+1,len(ps)-1)]-ps[max(0,j-1)]);normal=np.array([-tangent[1],tangent[0]])
                left.append(p+normal*width/2);right.append(p-normal*width/2)
            m['inlay'].poly2(left+list(reversed(right)),floor,.018)
        ribbon([[568,723],[505,757],[716,758],[605,800]],14)
        curve(gold,[[568,723],[505,757],[716,758],[605,800]],floor,1,.025,14)
        ribbon([[546,846],[668,913],[836,873],[897,810]],13)
        curve(gold,[[546,846],[668,913],[836,873],[897,810]],floor,1,.025,18)
        med=[[679+72*math.cos(t),779+31*math.sin(t)] for t in np.linspace(0,2*math.pi,33)[:-1]]
        m['inlay'].poly2(med,floor,.02);ring(trim,[679,779],76,34,floor,2,.03)
        ring(gold,[679,779],66,28,floor,1,.035)
        for t in np.linspace(0,2*math.pi,9)[:-1]:
            end=[679+61*math.cos(t),779+25*math.sin(t)]
            curve(trim,[[679,779],[679+25*math.cos(t+.3),779+10*math.sin(t+.3)],end,end],floor,1.4,.04,4)
    record('paving-inlays-radial-medallion',['Q02','Q03','Q04','Q05','Q06','Q07','Q08','Q09','Q12','Q13'],paving)
    def furniture():
        for poly in [[[353,659],[509,647],[518,660],[359,674]],[[357,626],[499,615],[498,622],[356,634]]]:wood.poly2(poly,wing,.45)
        for x in range(365,498,13):line(wood,[[x,629-(x-365)*.08],[x+3,653-(x-365)*.08]],wing,2,.46)
        for x,y in [(363,672),(504,660)]:line(wood,[[x,y],[x-5,y+25]],wing,3,.48)
        for x in [356,508]:curve(wood,[[x,659],[x-4,635],[x+13,637],[x+16,653]],wing,2.5,.53,5)
        # Separate chair in front of the long bench.
        wood.poly2([[321,677],[367,677],[379,690],[330,695]],wing,.9)
        line(wood,[[322,679],[316,646],[353,643],[366,681]],wing,3,.91)
        for x in [326,339,351]:line(wood,[[x,650],[x+4,677]],wing,1.5,.92)
        for x,y in [(330,693),(370,690)]:line(wood,[[x,y],[x-4,y+23]],wing,2.5,.95)
        curve(wood,[[320,677],[313,665],[333,660],[341,678]],wing,2,.98,4)
        for c,host,w,hh,d in [([202,713],wing,17,27,.3),([654,658],wing,13,22,.2),([798,660],wing,13,20,.2),([355,650],wing,9,14,.65),([489,639],wing,10,14,.5),([968,692],main,12,25,-.30),([1063,691],main,13,27,-.30),([1085,665],main,9,16,-.30)]:
            pot_centers.append(pot(c,host,w,hh,d))
    record('bench-chair-and-supported-pots',['J14','J15','K08','R01','R02','R03','R04','R05','R06','R07','R11'],furniture)
    def lanterns():
        for cx,cy,ss in [(230,620,.8),(378,583,.85),(664,552,1),(796,560,.85),(1064,550,.7)]:
            host=main if cx>900 else wing
            rect=[[cx-5*ss,cy-13*ss],[cx+5*ss,cy-13*ss],[cx+5*ss,cy+13*ss],[cx-5*ss,cy+13*ss]]
            m['light'].poly2(rect,host,.24)
            line(gold,rect+[rect[0]],host,1.1,.27)
            line(gold,[[cx,cy-18*ss],[cx-9*ss,cy-13*ss],[cx+9*ss,cy-13*ss],[cx,cy-18*ss],[cx,cy-25*ss]],host,1.4,.27)
            line(gold,[[cx-5*ss,cy+13*ss],[cx,cy+20*ss],[cx+5*ss,cy+13*ss]],host,1.4,.27)
            line(gold,[[cx,cy-13*ss],[cx,cy+13*ss]],host,.7,.29)
            curve(gold,[[cx,cy-23*ss],[cx-6,cy-34*ss],[cx-15,cy-31*ss],[cx-13,cy-20*ss]],host,.9,.18,4)
    record('four-exterior-and-interior-lantern-bodies',['S01','S02','S03','S04','S05','S06','S08'],lanterns)
    # Foliage is volumetric crossed leaves, not uniformly scaled cones.
    def leaf(p,size,angle,mesh):
        size*=1.9
        a=np.array([math.cos(angle),.35,math.sin(angle)])*size;b=np.array([-math.sin(angle),.2,math.cos(angle)])*size*.42
        mesh.polygon([p-a*.5+b,p+a,p-a*.5-b])
    def cluster(c,radius,height,n,flowers=False,shared=False):
        c=np.array(c)
        for i in range(n):
            angle=rng.random()*math.tau;r=radius*math.sqrt(rng.random());yy=rng.random()*height
            p=c+[r*math.cos(angle)*(1-.55*yy/max(height,.01)),yy,r*math.sin(angle)*(1-.55*yy/max(height,.01))]
            if not shared:leaf(p,rng.uniform(.045,.10),angle,m['leaf' if i%3 else 'leaf-var'])
            if flowers and i%4==0:
                rr=.055;petals=[p+np.array([rr*math.cos(t),.025*(j%2),rr*math.sin(t)]) for j,t in enumerate(np.linspace(0,2*math.pi,4)[:-1])]
                m['white'].polygon(petals)
    def greenery():
        for c in pot_centers:cluster(c,.22,.24,16,True)
        for px,py,r,hh,n in [(91,757,.28,1.5,60),(1320,755,.25,1.7,62),(1259,870,.24,1.2,48),(1350,815,.22,1.5,48)]:cluster(CAM.floor([px,py],.03),r,hh,n)
        for px,py,r,n in [(62,788,.6,50),(135,827,.45,35),(262,871,.42,32),(399,930,.4,30),(564,950,.35,30),(876,982,.35,30),(1111,919,.45,38),(1194,910,.5,42),(1289,858,.45,38),(1009,783,.25,20),(593,710,.22,20),(826,708,.28,20)]:
            c=CAM.floor([px,py],.04)
            shared.instances.append({'position':c.tolist(),'scale':r/.4,'color':PALETTE['leaf' if n%2 else 'leaf-var']})
            cluster(c,r,.35,n,True,True)
        # Planting strips behind the real pierced rail panels.
        for px,py in [(226,791),(306,814),(400,850),(1003,878),(1047,851)]:cluster(CAM.floor([px,py],.58),.27,.36,20,True)
        for px,py in [(1192,897),(1243,872),(604,714)]:
            c=CAM.floor([px,py],.03)
            for i in range(7):
                p=c+[rng.uniform(-.17,.17),0,rng.uniform(-.17,.17)];tip=p+[0,rng.uniform(.22,.55),0]
                m['leaf'].tube([p,tip],.007,3)
                for j in range(3):leaf(tip-[0,j*.05,0],.04,i,m['purple'])
        # The long climbing vine follows a recorded irregular path on the high pier.
        for path,host in [([[824,704],[847,548],[826,427],[867,300],[855,192]],main),([[528,641],[544,572],[525,520]],wing),([[1192,525],[1170,476],[1194,479]],side)]:
            # piecewise line to keep known bends, with small individual leaves around it.
            line(m['leaf'],path,host,1.1,.21)
            for a,b in zip(path,path[1:]):
                for t in np.linspace(0,1,19):
                    p=np.array(a)*(1-t)+np.array(b)*t+[rng.uniform(-10,10),rng.uniform(-5,5)]
                    leaf(host(p,.23),rng.uniform(.035,.07),rng.random()*math.tau,m['leaf-var' if rng.random()<.4 else 'leaf'])
    shared=Mesh('rebuild-leaf-cluster',PALETTE['leaf']);shared.instances=[]
    for i in range(42):
        a=rng.random()*math.tau;r=.4*math.sqrt(rng.random());y=rng.random()*.32
        leaf(np.array([r*math.cos(a)*(1-y),y,r*math.sin(a)*(1-y)]),rng.uniform(.05,.10),a,shared)
    m['leaf-cluster']=shared
    record('located-vegetation-pots-vines-flowers',['T01','T02','T03','T04','T05','T06','T07','T08','T09','T10','T11','T12','T13','T14','T15','T16','T17','T18','T19'],greenery)


def publish(stage='structure'):
    # A front-view match cannot authorize publishing a building with no body.
    meshes,columns,hosts,roofs,openings=build('structure')
    structural=building_core_preflight(meshes)
    save('structural-preflight.json',structural)
    if not structural['solid_preflight_passed']:
        raise RuntimeError('Building structure rejected before MCP publication: '+', '.join(structural['defects']))
    if stage=='detail':
        meshes,columns,hosts,roofs,openings=build(stage)
        structural=building_core_preflight(meshes)
        save('structural-preflight.json',structural)
        if not structural['solid_preflight_passed']:
            raise RuntimeError('Detailed building structure rejected before MCP publication')
    for mesh in meshes.values():mesh.weld()
    meshes={k:v for k,v in meshes.items() if v.p}
    structural=building_core_preflight(meshes)
    save('structural-preflight.json',structural)
    if not structural['solid_preflight_passed']:
        raise RuntimeError('Building structure rejected after vertex remapping')
    counts={'vertices':sum(len(x.p) for x in meshes.values()),'triangles':sum(len(x.t) for x in meshes.values()),'assets':len(meshes)+1,
            'instances':sum(len(getattr(x,'instances',[None])) for x in meshes.values())+len(columns)}
    save('preflight.json',counts);print(json.dumps(counts),flush=True)
    e=Engine()
    try:
        limits=e.command('inspect_meshes')['limits']
        assert counts['vertices']+160<=limits['total_vertices']
        assert counts['triangles']+160<=limits['total_triangles']
        assert counts['assets']<=limits['assets'] and counts['instances']<=limits['instances']
        assert all(len(x.p)<=limits['asset_vertices'] and len(x.t)<=limits['asset_triangles'] for x in meshes.values())
        state=e.command('inspect')
        edits=[{'op':'remove','id':v['id']} for v in state['scene']['entities'] if v['id']!='player']
        edits.append({'op':'upsert','entity':{'id':'player','name':'outside study','visible':False,'body':'player',
                      'transform':{'position':[40,1,40],'size':[.7,1.6,.7]},'color':[.5,.5,.5]}})
        edits.extend([{'op':'upsert','entity':{'id':'studio-ground','name':'studio ground',
              'transform':{'position':[0,-.075,0],'size':[120,.2,120]},'color':[.57,.52,.45]}},
              {'op':'upsert','entity':{'id':'studio-background','name':'studio background',
              'transform':{'position':[0,22,-18],'size':[120,50,.1]},'color':[.62,.57,.49]}}])
        e.command('edit_scene',edits=edits)
        for name,mesh in meshes.items():
            material={'pattern':{'frequency':[3,4,3],'low':[.85,.85,.82],'high':[1.08,1.06,1.01],
                'bump':.012,'warp':.3,'octaves':3,'seed':42,'contrast':.55}}
            if name=='stone':
                segments=[]
                for j in range(20):
                    yy=.3+j*.42
                    segments.append({'start':[-10,yy],'end':[8,yy], 'widths':[.012,.012]})
                for j in range(12):
                    xx=-7+j*1.05;yy=.3+(j%5)*.42
                    segments.append({'start':[xx,yy],'end':[xx,yy+.42], 'widths':[.012,.012]})
                material['detail']={'plane':'xy','segments':segments,'tint':[.36,.31,.24],'opacity':.5,'bump':.025}
            if name in ['gold','roof','roof-var']:
                material['reflectance']={'roughness':.7,'f0':.04}
            e.command('edit_mesh',edit={'kind':'put_asset','asset':{'id':mesh.name,
                'geometry':{'positions':mesh.p,'triangles':mesh.t},'shading':'smooth','material':material}})
            for i,inst in enumerate(getattr(mesh,'instances',[{'position':[0,0,0],'scale':1,'color':mesh.color}])):
                e.command('edit_mesh',edit={'kind':'put_instance','instance':{'id':mesh.name+'-'+str(i),'asset':mesh.name,
                    'rotation':[0,0,0,1],**inst}})
        definition={'id':'rebuild-column','profile':{'id':'profile-column','width':.4,'height':2.8},'depth':.4,'segments':[1,1,1],
            'sections':{'id':'stations-column','cross_section':'rectangle','stations':[
                {'id':'s'+str(i),'at':a,'width':w,'depth':w} for i,(a,w) in enumerate(
                    [(0,1),(.045,1),(.06,.85),(.085,.85),(.1,.95),(.125,.95),(.145,.53),(.82,.53),(.86,.66),(.89,.75),(.94,1),(1,1)])]}}
        edit={'kind':'put','definition':definition,'detail_policy':'physical','animation_translation_policy':'physical'}
        save('column-preview.json',e.command('preview_plant',edit=edit));e.command('edit_plant',edit=edit)
        for i,c in enumerate(columns):
            scale=c['height']/2.8
            e.command('edit_mesh',edit={'kind':'put_instance','instance':{'id':'column-'+str(i),'asset':'rebuild-column',
                'position':[c['foot'][0],c['foot'][1]+c['height']/2,c['foot'][2]],'rotation':[0,0,0,1],
                'scale':scale,'color':PALETTE['trim']}})
        e.command('set_camera',camera=CAMERA)
        e.command('set_light',light={'direction':[-.65,-1,-.65],'ambient':.35,
              'shadow':{'center':[0,3,0],'extent':18,'resolution':2048,'bias':.001}})
        save('capture.json',e.command('capture',path=stage+'.png',width=WIDTH,height=HEIGHT))
        e.command('save_scene',path='house.scene.json')
        e.command('set_camera',camera={'position':[17,17,17],'target':[0,3,0],'fov_degrees':48})
        e.command('capture',path='auxiliary.png',width=WIDTH,height=HEIGHT)
        e.command('set_camera',camera=CAMERA)
        save('counts.json',e.command('inspect_meshes'))
        save('components.json',{k:v.parts for k,v in meshes.items()})
        save('plant.json',e.command('inspect_plant',id='rebuild-column'))
        save('column-mesh.json',e.command('inspect_mesh',asset='rebuild-column'))
        shutil.copyfile(SOURCE,ROOT/'reference.png')
    finally:e.close()


def verify():
    """Integrity is a separate result from visual acceptance."""
    meshes,_,_,_,_=build('detail')
    for mesh in meshes.values():
        mesh.weld();p=np.array(mesh.p)
        assert np.isfinite(p).all()
        assert all(len(set(t))==3 and min(t)>=0 and max(t)<len(p) for t in mesh.t)
        assert all(max(part['vertex_indices'],default=-1)<len(p) for part in mesh.parts)
    e=Engine();checks={}
    try:
        e.command('load_scene',path='house.scene.json')
        e.command('capture',path='repeat.png',width=WIDTH,height=HEIGHT)
        checks['fresh_process_load_exact']=hashlib.sha256((ROOT/'repeat.png').read_bytes()).digest()==hashlib.sha256((ROOT/'detail.png').read_bytes()).digest()
        e.command('capture',path='repeat.png',width=WIDTH,height=HEIGHT)
        checks['repeated_capture_exact']=hashlib.sha256((ROOT/'repeat.png').read_bytes()).digest()==hashlib.sha256((ROOT/'detail.png').read_bytes()).digest()
        e.command('set_camera',camera={**CAMERA,'fov_degrees':40})
        e.command('undo')
        e.command('capture',path='undo.png',width=WIDTH,height=HEIGHT)
        checks['undo_exact']=hashlib.sha256((ROOT/'undo.png').read_bytes()).digest()==hashlib.sha256((ROOT/'detail.png').read_bytes()).digest()
        e.command('save_replay',path='static.replay.json')
        replay=e.command('run_replay',path='static.replay.json');save('replay-report.json',replay)
        checks['static_replay_matched']=replay['matched']
        e.command('capture_replay',replay_path='static.replay.json',path='replay.png',width=WIDTH,height=HEIGHT,tick=0)
        checks['static_replay_exact']=hashlib.sha256((ROOT/'replay.png').read_bytes()).digest()==hashlib.sha256((ROOT/'detail.png').read_bytes()).digest()
        crops=json.loads((PROJECT/'artifacts/house-e02-analysis/crops.json').read_text())['crops']
        rois=[{'id':c['id'],'x':c['box'][0],'y':c['box'][1],'width':c['box'][2],'height':c['box'][3]} for c in crops if c['kind']!='coverage']
        comparisons=[]
        for i in range(0,len(rois),16):
            comparisons.append(e.command('compare_images',reference_path='reference.png',candidate_path='detail.png',regions=rois[i:i+16],difference_path='difference.png'))
        save('comparison.json',comparisons)
        checks['geometry_finite_indices_and_semantic_ranges']=True
        checks['reference_unchanged']=hashlib.sha256(SOURCE.read_bytes()).hexdigest()=='4153a456f9fdf2e9e6f43328171692156db7563c2062654199e66c0c540882ad'
        save('verification.json',{'checks':checks,'render_sha256':hashlib.sha256((ROOT/'detail.png').read_bytes()).hexdigest(),'fidelity_accepted':False,'note':'Replay estático tick0; não valida simulação, colisão, fechamento ou fidelidade.'})
        assert all(checks.values()),checks
        print(json.dumps(checks),flush=True)
    finally:e.close()


if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--probe',action='store_true');parser.add_argument('--verify',action='store_true');parser.add_argument('--stage',choices=['structure','detail'])
    args=parser.parse_args()
    if args.probe:probe()
    elif args.verify:verify()
    elif args.stage:publish(args.stage)
