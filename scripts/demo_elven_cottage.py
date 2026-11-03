#!/usr/bin/env python3
"""Reproducible architectural authoring through the native MCP dispatcher."""
import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import random

P = Path(__file__).resolve().parents[1]
R = P / 'artifacts/elven-cottage'
TAU = math.tau


def add(a, b): return [x+y for x, y in zip(a, b)]
def sub(a, b): return [x-y for x, y in zip(a, b)]
def mul(a, s): return [x*s for x in a]
def cross(a, b): return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
def dot(a, b): return sum(x*y for x, y in zip(a, b))
def unit(a): return mul(a, 1/math.sqrt(dot(a, a)))
def cubic(points,t):
    s=1-t
    return [s*s*s*a+3*s*s*t*b+3*s*t*t*c+t*t*t*d for a,b,c,d in zip(*points)]


def cubic_at_x(points,x):
    """Monotone X inversion; the surface and its native trim share controls."""
    lo,hi=0.,1.
    for _ in range(40):
        t=(lo+hi)/2
        if cubic(points,t)[0]<x:lo=t
        else:hi=t
    return cubic(points,(lo+hi)/2)[1]


def bezier(p, n=14):
    out = []
    for i in range(n):
        t = i/(n-1)
        q = p
        while len(q) > 1: q = [add(mul(a, 1-t), mul(b, t)) for a, b in zip(q, q[1:])]
        out.append(q[0])
    return out


def arch_top(x, width, height, spring=.59):
    """Invert the monotone half-ogive; the planks and spandrels share its outline."""
    w=width/2;x=abs(x);lo,hi=0.,1.
    for _ in range(30):
        t=(lo+hi)/2;s=1-t
        px=w*(s*s*s+3*s*s*t+3*s*t*t*.38)
        if px>x:lo=t
        else:hi=t
    t=(lo+hi)/2;s=1-t
    shoulder=spring+(1-spring)*.56;tip=spring+(1-spring)*.88
    return height*(spring*s*s*s+3*shoulder*s*s*t+3*tip*s*t*t+t*t*t)


class Mesh:
    def __init__(self,name=''):
        self.name=name;self.positions, self.triangles, self.sweeps, self.reliefs, self.boxes = [], [], [], [], []

    def chamfer_box(self,center,size,bevel,angle=0):
        self.boxes.append({'center':center,'size':size,'rotation':[0,math.sin(angle/2),0,math.cos(angle/2)],'bevel':bevel})

    def relief_leaf(self,points,width,depth,normal):
        """A ridged closed blade: native profile sweep with explicit stations."""
        assert self.name=='bronze' and width>0 and depth>0
        scales=[.18,.60,.88,1.,.92,.75,.53,.28,.045]
        self.reliefs.append(len(self.sweeps))
        self.sweeps.append({'profile':[[-depth,0],[0,-width],[depth,0],[0,width]],'curves':[points],'spans':8,'profile_x':normal,'sections':[{'at':j/8,'scale':[.25+.75*s,s]} for j,s in enumerate(scales)],'capped':True,'centerline_tolerance':.003})

    def native_curve(self,points,radius,n,end_radius,profile_x=None):
        if self.name not in ['carved-stone','bronze']:return False
        self.native_path([points],radius,n,end_radius,profile_x)
        return True

    def native_path(self,curves,radius,n,end_radius=None,profile_x=None):
        assert self.name in ['carved-stone','bronze']
        if profile_x is None:
            tangent=unit(sub(curves[0][1],curves[0][0]));profile_x=[0,0,1] if abs(tangent[2])<.95 else [1,0,0]
        if self.name=='carved-stone':
            d,w=radius*.72,radius
            profile=[[-d,-w*.7],[-d*.7,-w],[d*.7,-w],[d,-w*.7],[d,w*.7],[d*.7,w],[-d*.7,w],[-d,w*.7]]
        else:
            r=radius;profile=[[-r*.45,-r],[r*.45,-r],[r*.45,r],[-r*.45,r]]
        taper=1 if end_radius is None else end_radius/radius
        self.sweeps.append({'profile':profile,'curves':curves,'spans':max(1,n),'profile_x':profile_x,'sections':[{'at':0,'scale':[1,1]},{'at':1,'scale':[taper,taper]}],'capped':True,'centerline_tolerance':.012})

    def surface_shell(self,surface,umin,umax,vmax,rows,cols,reverse=False):
        """One indexed closed backing, no invisible internal cell walls."""
        start=len(self.positions);stride=cols+1;layer=(rows+1)*stride
        top=[surface(umin+(umax-umin)*i/cols,vmax*j/rows) for j in range(rows+1) for i in range(cols+1)]
        self.positions.extend(top+[add(q,[0,-.075,0]) for q in top])
        faces=[]
        for j in range(rows):
            for i in range(cols):
                a=start+j*stride+i;b=a+1;c=a+stride;d=c+1
                faces.extend([[a,b,c],[b,d,c]])
        if reverse:faces=[f[::-1] for f in faces]
        self.triangles.extend(faces+[[i+layer for i in f[::-1]] for f in faces])
        # Counter-clockwise boundary of the oriented top surface.
        boundary=[start+i for i in range(cols+1)]+[start+j*stride+cols for j in range(1,rows+1)]+[start+rows*stride+i for i in range(cols-1,-1,-1)]+[start+j*stride for j in range(rows-1,0,-1)]
        if reverse:boundary.reverse()
        for a,b in zip(boundary,boundary[1:]+boundary[:1]):
            self.triangles.extend([[b,a,a+layer],[b,a+layer,b+layer]])

    def poly(self, points):
        start = len(self.positions)
        self.positions.extend(points)
        self.triangles.extend([start, start+i, start+i+1] for i in range(1, len(points)-1))

    def prism(self, points, delta):
        n, start = len(points), len(self.positions)
        self.positions.extend(points + [add(p, delta) for p in points])
        self.triangles.extend([start, start+i+1, start+i] for i in range(1, n-1))
        self.triangles.extend([start+n, start+n+i, start+n+i+1] for i in range(1, n-1))
        for i in range(n):
            a, b = start+i, start+(i+1) % n
            self.triangles.extend([[a, b, b+n], [a, b+n, a+n]])

    def box(self, center, size, angle=0):
        x, y, z = center
        w, h, d = [v/2 for v in size]
        # Consistent outward normals on all six planes.
        pts = [[a, b, c] for a, b, c in [(-w,-h,-d),(w,-h,-d),(w,h,-d),(-w,h,-d),(-w,-h,d),(w,-h,d),(w,h,d),(-w,h,d)]]
        cs, sn = math.cos(angle), math.sin(angle)
        pts = [[x+cs*a+sn*c, y+b, z-sn*a+cs*c] for a,b,c in pts]
        start=len(self.positions);self.positions.extend(pts)
        for f in [[3,2,1,0],[4,5,6,7],[0,1,5,4],[7,6,2,3],[0,4,7,3],[2,6,5,1]]:
            self.triangles.extend([[start+f[0],start+f[1],start+f[2]],[start+f[0],start+f[2],start+f[3]]])

    def tube(self, path, radius, sides=6, end_radius=None, profile_x=None):
        if len(path)==2:
            a,b=path;delta=sub(b,a)
            if self.native_curve([a,add(a,mul(delta,1/3)),add(a,mul(delta,2/3)),b],radius,1,end_radius,profile_x):return
        sides=min(sides,4 if radius<.04 else 6)
        n, start, radial = len(path), len(self.positions), None
        for i, p in enumerate(path):
            t = unit(sub(path[min(i+1, n-1)], path[max(0, i-1)]))
            if radial is None:
                axis = min([[1,0,0],[0,1,0],[0,0,1]], key=lambda a: abs(dot(a,t)))
                radial = unit(cross(t, axis))
            else: radial = unit(sub(radial, mul(t, dot(radial,t))))
            other = cross(t, radial)
            r = radius if end_radius is None else radius+(end_radius-radius)*i/(n-1)
            for j in range(sides):
                a = j*TAU/sides
                self.positions.append(add(p, add(mul(radial,r*math.cos(a)),mul(other,r*math.sin(a)))))
        for i in range(n-1):
            for j in range(sides):
                a,b = start+i*sides+j, start+i*sides+(j+1)%sides
                self.triangles.extend([[a,b,a+sides],[b,b+sides,a+sides]])
        for ring, rev in [(start,True),(start+(n-1)*sides,False)]:
            for j in range(1,sides-1):
                t=[ring,ring+j,ring+j+1];self.triangles.append(t[::-1] if rev else t)

    def curve(self, points, radius, n=14, sides=6, end_radius=None, profile_x=None):
        if self.native_curve(points,radius,9 if radius>=.04 else 6,end_radius,profile_x):return
        n=min(n,8 if radius<.04 else 14)
        self.tube(bezier(points,n),radius,sides,end_radius)

    def lathe(self, center, rings, sides=12):
        start=len(self.positions)
        for y,r in rings:
            self.positions.extend([[center[0]+r*math.cos(i*TAU/sides),center[1]+y,center[2]+r*math.sin(i*TAU/sides)] for i in range(sides)])
        for j in range(len(rings)-1):
            for i in range(sides):
                a,b=start+j*sides+i,start+j*sides+(i+1)%sides
                self.triangles.extend([[a,a+sides,b],[b,a+sides,b+sides]])
        for ring,rev in [(start,False),(start+(len(rings)-1)*sides,True)]:
            for j in range(1,sides-1):
                t=[ring,ring+j,ring+j+1];self.triangles.append(t[::-1] if rev else t)

    def geometry(self):
        return {'positions':[[round(v,6) for v in p] for p in self.positions], 'triangles':self.triangles}


def material(seed, frequency=(8,8,8), bump=.004, low=(.7,.7,.68), high=(1.1,1.08,1), roughness=.9):
    return {'pattern':{'frequency':frequency,'low':low,'high':high,'bump':bump,'warp':.12,'octaves':2,'seed':seed,'contrast':1.1,'kind':'noise'},'reflectance':{'roughness':roughness,'f0':.04}}


def build(cfg):
    rng=random.Random(cfg['seed']);stone_rng=random.Random(cfg['seed']^0x81ad53);m=cfg['main'];p=cfg['porch'];b=cfg['bay'];roof=cfg['roof']
    names=['masonry','stone-blocks','carved-stone','roof-backing','slate-0','slate-1','slate-2','oak','bronze','window-infill','green-panels','ground','leaves','flowers','paving']
    meshes={k:Mesh(k) for k in names}
    stone,blocks,trim=meshes['masonry'],meshes['stone-blocks'],meshes['carved-stone']
    oak,gold,dark=meshes['oak'],meshes['bronze'],meshes['window-infill']
    fields=meshes['green-panels']
    shell=meshes['roof-backing'];foliage=meshes['leaves'];flowers=meshes['flowers'];ground=meshes['ground'];paving=meshes['paving']
    groups={}

    def mark(name, family, fn):
        before={k:(len(v.positions),len(v.triangles),len(v.sweeps),len(v.boxes)) for k,v in meshes.items()};fn()
        groups[name]={'family':family,'parts':{k:{'first_vertex':before[k][0],'vertices':len(v.positions)-before[k][0],'first_triangle':before[k][1],'triangles':len(v.triangles)-before[k][1],'first_native_sweep':before[k][2],'native_sweeps':len(v.sweeps)-before[k][2],'first_native_box':before[k][3],'native_boxes':len(v.boxes)-before[k][3]} for k,v in meshes.items() if len(v.positions)>before[k][0] or len(v.sweeps)>before[k][2] or len(v.boxes)>before[k][3]}}

    def wall(center,width,height,depth,angle=0):
        stone.box(center,[width,height,depth],angle)
        rows=max(1,math.ceil(height/.38));h=height/rows
        for row in range(rows):
            count=max(1,math.ceil(width/.7));step=width/count
            edges=[-width/2]+[-width/2+(j+.5*(row%2))*step for j in range(1,count)]+[width/2]
            for col,(edge0,edge1) in enumerate(zip(edges,edges[1:])):
                xx=(edge0+edge1)/2
                zz=depth/2+.008
                cc=[center[0]+math.cos(angle)*xx+math.sin(angle)*zz,center[1]-height/2+(row+.5)*h,center[2]-math.sin(angle)*xx+math.cos(angle)*zz]
                size=[edge1-edge0-.018,h-.018,.035+rng.random()*.008]
                if angle==0 and center[0]>2 and center[2]>1.5 and (row+col)%2==0:
                    blocks.chamfer_box(cc,size,stone_rng.uniform(.0045,.007),angle)
                else:blocks.box(cc,size,angle)

    def arch(mesh,cx,bottom,width,height,z,radius=.065,transform=None,spring_fraction=.59):
        def point(x,y):return transform(x,y,z) if transform else [cx+x,bottom+y,z]
        spring=height*spring_fraction;w=width/2
        normal=unit(cross(sub(point(1,0),point(0,0)),sub(point(0,1),point(0,0))))
        shoulder=spring+(height-spring)*.56;tip=spring+(height-spring)*.88
        mesh.tube([point(-w,0),point(-w,spring)],radius,8,profile_x=normal)
        mesh.curve([point(-w,spring),point(-w,shoulder),point(-w*.38,tip),point(0,height)],radius,n=18,sides=8,profile_x=normal)
        mesh.curve([point(0,height),point(w*.38,tip),point(w,shoulder),point(w,spring)],radius,n=18,sides=8,profile_x=normal)
        mesh.tube([point(w,spring),point(w,0)],radius,8,profile_x=normal)

    def tree_motif(cx,bottom,w,h,z,mesh=gold,transform=None,fine=False):
        def point(x,y):return transform(x,y,z) if transform else [cx+x,bottom+y,z]
        r=.019 if fine else (.045 if mesh.name=='carved-stone' else .029)
        normal=unit(cross(sub(point(1,0),point(0,0)),sub(point(0,1),point(0,0))))
        mesh.curve([point(0,0),point(-.06*w,.3*h),point(.06*w,.7*h),point(0,h)],r,n=18,sides=6,end_radius=.008,profile_x=normal)
        for s in [-1,1]:
            for t in [.28,.56]:
                endpoint=.43 if t<.5 else .27
                mesh.curve([point(0,t*h),point(s*w*.14,(t+.22)*h),point(s*w*.45,(t+.14)*h),point(s*w*endpoint,(t+.37)*h)],r*.8,n=12,sides=5,end_radius=.005,profile_x=normal)
            # Slender pointed leaf-shaped tracery surrounding the leader.
            mesh.curve([point(0,.55*h),point(s*.25*w,.76*h),point(s*.20*w,.84*h),point(0,h)],r*.65,n=12,sides=5,end_radius=.006,profile_x=normal)

    def foliate_spray(cx,bottom,w,h,z,transform=None,pairs=2,normal=None):
        def point(x,y):return transform(x,y,z) if transform else [cx+x,bottom+y,z]
        if normal is None:normal=unit(cross(sub(point(1,0),point(0,0)),sub(point(0,1),point(0,0))))
        r=min(.014,w*.025);depth=min(.014,w*.025)
        gold.curve([point(0,0),point(-.035*w,.20*h),point(.035*w,.52*h),point(0,.73*h)],r,end_radius=r*.45,profile_x=normal)
        leaves=[((0,.63),(0,1))]
        for j in range(pairs):
            root=.25+j*.24;tip=.73+j*.21
            for side in [-1,1]:
                leaves.append(((0,root),(side*(.55-j*.18),tip)))
        for (a,b0),(c,d) in leaves:
            a0=point(a*w,b0*h);a3=point(c*w,d*h);delta=sub(a3,a0)
            bulge=mul(normal,depth*.7)
            controls=[a0,add(add(a0,mul(delta,.30)),bulge),add(sub(a3,mul(delta,.28)),bulge),a3]
            gold.relief_leaf(controls,w*.06,depth,normal)

    def foundation():
        stone.box([-.3,.3,.1],[9.4,.6,6.1])
        # Real porch plinth and paved top; individual staggered ashlar fronts.
        wall([-2.35,.39,3.04],5.12,.68,.35)
        wall([-4.82,.39,.25],5.7,.68,.28,-math.pi/2)
        trim.box([-2.35,.72,2.0],[5.2,.14,2.6])
        trim.box([1.45,.72,2.0],[3.0,.14,2.6])
        for row in range(4):
            for col in range(8):paving.box([-4.7+(col+.5)*.62,.805,1.04+(row+.5)*.52],[.6,.04,.50])
        # Four treads, landing aligns with the front wall/door.
        for j in range(4):
            top=.18*(j+1);front=4.25-j*.36
            stone.box([1.45,top/2,(front+1.1)/2],[2.7,top,front-1.1])
            trim.box([1.45,top-.025,front-.15],[2.8,.08,.38])
        # Bay lower masonry, polygonal rather than a cylinder with decals.
        cx,cz=b['center'];rad=b['radius'];n=b['windows']
        for i in range(n):
            a=-math.pi/2+(i+.5)*math.pi/n;span=2*rad*math.sin(math.pi/n/2);rotation=math.pi/2-a
            center=[cx+rad*math.cos(a),.55,cz+rad*math.sin(a)]
            wall(center,span+.03,.9,.28,rotation)
            trim.box([center[0],1.02,center[2]],[span+.14,.13,.43],rotation)
        # Fill the raised bay base to its attachment plane, rather than floating
        # a polygonal shell beyond the rectangular building plinth.
        ring=[[cx,.1,cz]]+[[cx+rad*math.cos(-math.pi/2+j*math.pi/n),.1,cz+rad*math.sin(-math.pi/2+j*math.pi/n)] for j in range(n+1)]
        stone.prism(ring,[0,.72,0])
    mark('plinth-porch-stairs','structure',foundation)

    def walls():
        door_x=cfg['entrance']['door_x'];wall_z=cfg['entrance']['wall_z'];delta=wall_z-1.1
        wall([-.5,2.24,m['back']],8.2,3.05,.24,math.pi)
        wall([m['left'],2.24,-.65],3.9,3.05,.24,-math.pi/2)
        # The bay belongs to this end: leave the central end wall open.
        for z,width in [(m['back']+.15,.3),(1.6,1.3)]:wall([m['right'],2.24,z],width,3.05,.24,math.pi/2)
        # Rear porch wall and recessed entrance opening: no solid box over door.
        wall([-2.35,2.24,1.1],4.55,3.05,.22)
        wall([-.04,2.24,wall_z],.30,3.05,.30)
        wall([.325,2.24,wall_z],.60,3.05,.30)
        wall([2.8,2.24,wall_z],1.60,3.05,.3)
        # Door width1.35, tip3.65; solid upper spandrels follow the arch.
        for s in [-1,1]:
            pts=[[door_x+s*.76,2.65,1.25+delta],[door_x+s*.76,3.85,1.25+delta],[door_x,3.85,1.25+delta],[door_x,3.65,1.25+delta]]
            stone.prism(pts,[0,0,-.3])
        # Opaque oak leaf occupies only the opening; planks have real grooves.
        width=1.35;bot=.8;h=2.77
        for j in range(11):
            x0=j/11*width-width/2+.003;x1=(j+1)/11*width-width/2-.003
            points=[[door_x+x0,bot,1.265+delta],[door_x+x1,bot,1.265+delta]]
            points.extend([[door_x+x,bot+arch_top(x,width,h),1.265+delta] for x in [x1-(x1-x0)*k/3 for k in range(4)]])
            oak.prism(points,[0,0,-.1])
        arch(trim,door_x,.78,1.58,2.90,1.38+delta,.11)
        arch(trim,door_x,.78,1.76,3.0,1.31+delta,.065)
        arch(gold,door_x,.85,1.23,2.59,1.34+delta,.022)
        tree_motif(door_x,.85,1.15,2.48,1.34+delta)
        # Door handle and threshold.
        gold.tube([[door_x+.4,1.65,1.38+delta],[door_x+.4,1.82,1.38+delta]],.025,8)
        trim.box([door_x,.79,1.4+delta],[1.9,.10,.55])
    mark('walls-and-door','structure',walls)

    columns=[]
    def porch():
        step=(p['right']-p['left'])/p['bays']
        for j in range(p['bays']+1):
            x=p['left']+j*step;columns.append([x,2.17,p['front']])
            for y,w,h in [(.86,.47,.14),(1.0,.35,.12),(3.43,.39,.11),(3.55,.48,.13)]:trim.box([x,y,p['front']],[w,h,.45])
        columns.append([p['left'],2.17,1.1])
        columns.append([2.95,2.17,p['front']])
        for y,w,h in [(.86,.57,.14),(1.0,.44,.12),(3.43,.49,.11),(3.55,.57,.13)]:trim.box([2.95,y,p['front']],[w,h,.52])
        for j in range(p['bays']):
            x=p['left']+(j+.5)*step
            arch(trim,x,1.66,step-.25,1.88,p['front'],.08)
            arch(trim,x,1.66,step-.38,1.79,p['front']+.035,.026)
            arch(gold,x,1.66,step-.39,1.77,p['front']+.035,.02)
            tree_motif(x,1.66,step-.38,1.73,p['front']+.05,trim)
            trim.box([x,1.63,p['front']],[step-.05,.14,.39])
            trim.box([x,.88,p['front']],[step-.05,.1,.28])
            for k in range(3):
                xx=x+(k-1)*step/3
                arch(trim,xx,.94,step/3-.09,.63,p['front'],.033)
                tree_motif(xx,.94,step/3-.13,.61,p['front']+.04,gold,fine=True)
        for j in range(1,p['bays']):
            foliate_spray(p['left']+j*step,p.get('relief_bottom',3.43),.40,p.get('relief_height',.25),p['front']+p.get('relief_front_offset',.10),pairs=1)
        # West porch return is also an open arch with railing.
        def side(x,y,z):return [p['left'],1.42+y,2.125+x]
        arch(trim,0,0,1.7,2.1,0,.07,side)
        tree_motif(0,0,1.5,2,0,trim,side)
        trim.box([p['left'],1.38,2.1],[.35,.12,2.1])
        for j in range(4):
            def small(x,y,z,j=j):return [p['left'],.92+y,1.33+j*.49+x]
            arch(trim,0,0,.4,.42,0,.028,small)
        # Patio bench behind the tracery.
        oak.box([-2.8,1.12,1.8],[1.45,.13,.4]);oak.box([-2.8,1.48,1.65],[1.45,.6,.10])
        for x in [-3.4,-2.2]:oak.box([x,.96,1.8],[.1,.36,.36])
    mark('open-porch-and-balustrade','architecture',porch)

    def bay_windows():
        cx,cz=b['center'];rad=b['radius'];n=b['windows'];span=2*rad*math.sin(math.pi/n/2)
        for j in range(n):
            a=-math.pi/2+(j+.5)*math.pi/n;xx,zz=cx+rad*math.cos(a),cz+rad*math.sin(a);rotation=math.pi/2-a
            def frame(x,y,z):return [xx+math.sin(a)*x+math.cos(a)*z,1.08+y,zz-math.cos(a)*x+math.sin(a)*z]
            width=span-.08;h=2.4
            points=[frame(-width/2,0,-.08),frame(width/2,0,-.08)]
            points.extend(frame(x,arch_top(x,width,h),-.08) for x in [width/2-width*k/12 for k in range(13)])
            dark.prism(points,mul([math.cos(a),0,math.sin(a)],.05))
            for s in [-1,1]:
                points=[frame(0,2.48,-.05),frame(s*width/2,2.48,-.05)]
                points.extend(frame(x,arch_top(x,width,h),-.05) for x in [s*width/2*(1-k/6) for k in range(7)])
                stone.prism(points,mul([math.cos(a),0,math.sin(a)],.08))
            arch(trim,0,0,span-.08,2.4,.02,.066,frame)
            arch(gold,0,0,span-.23,2.26,.05,.025,frame)
            tree_motif(0,0,span-.25,2.24,.075,gold,frame)
            trim.box([xx,3.56,zz],[span+.12,.14,.38],rotation)
            # Continuous sill cornice with lower drip edge.
            trim.box([xx,1.02,zz],[span+.12,.16,.42],rotation)
        for j in range(n+1):
            a=-math.pi/2+j*math.pi/n;xx,zz=cx+rad*math.cos(a),cz+rad*math.sin(a)
            trim.tube([[xx,1.06,zz],[xx,3.56,zz]],.083,8)
    mark('polygonal-bay-tracery','architecture',bay_windows)

    left,right=m['left']-.36,m['right']+.29
    eave=roof['front_eave']
    assert eave[0][0]>left and eave[-1][0]==right and eave[0][1]==m['eave']
    assert all(a[0]<q[0] and a[1]<=q[1] for a,q in zip(eave,eave[1:]))
    assert eave[0][1]==eave[1][1] and eave[2][1]==eave[3][1]
    delta=m['ridge']-m['eave']
    roof_profile=[[m['ridge']],[m['ridge']-.55*delta],[m['eave']],[m['eave']]]
    def roof_y(s):return cubic(roof_profile,s)[0]
    def eave_lift(u):return 0 if u<=eave[0][0] else cubic_at_x(eave,u)-m['eave']
    def lift_weight(v):return cubic([[0],[.55],[1],[1]],v)[0]
    def front(u,v):
        return [u,roof_y(v)+eave_lift(u)*lift_weight(v),m['ridge_z']+v*(p['front']+.4-m['ridge_z'])]
    def back(u,v):return [u,roof_y(v),m['ridge_z']-v*(m['ridge_z']-m['back']+.4)]

    def eave_path(v=1,z=None):
        zz=front(left,v)[2] if z is None else z
        a=[left,roof_y(v),zz];end=[eave[0][0],roof_y(v),zz]
        d=sub(end,a)
        return [[a,add(a,mul(d,1/3)),add(a,mul(d,2/3)),end],[[x,roof_y(v)+(y-m['eave'])*lift_weight(v),zz] for x,y in eave]]

    def slope_curve(fn,x):
        a,z=fn(x,0),fn(x,1)
        lift=eave_lift(x) if fn==front else 0
        return [a,add(a,[0,-.55*(delta-lift),(z[2]-a[2])/3]),sub(z,[0,0,(z[2]-a[2])/3]),z]

    # Verify independent parameterizations agree before publishing either one.
    for x in [left,right,(left+right)/2]:
        for fn in [front,back]:
            curve=slope_curve(fn,x)
            samples=[fn(x,j/64) for j in range(65)]
            assert all(a[1]>=q[1]-1e-10 for a,q in zip(samples,samples[1:]))
            assert all(math.dist(cubic(curve,j/64),q)<1e-9 for j,q in enumerate(samples))
    for level in [0,.5,1,(p['front']-m['ridge_z'])/(p['front']+.4-m['ridge_z'])]:
        previous=-math.inf
        for curve in eave_path(level):
            for j in range(65):
                q=cubic(curve,j/64)
                assert q[1]>=previous-1e-10 and math.dist(front(q[0],level),q)<1e-9
                previous=q[1]

    def tiling(surface,umin,umax,vmax,rows,angular=False):
        cols=max(3,round((umax-umin)*(b['radius']+.25 if angular else 1)/roof['tile_width']))
        # Shared backing removes internal walls and retains a closed boundary.
        shell.surface_shell(surface,umin,umax,vmax,rows,12 if surface!=back else 4,reverse=surface==front)
        if surface==front:
            for row in range(rows):
                v0=vmax*row/rows;v1=vmax*(row+1)/rows
                if v0<=.70:continue
                for col in range(12):
                    u0=umin+(umax-umin)*col/12;u1=umin+(umax-umin)*(col+1)/12
                    if u1>-.05:
                        a,b0,c,d=[add(surface(u,v),[0,-.085,0]) for u,v in [(u0,v0),(u1,v0),(u0,v1),(u1,v1)]]
                        stone.poly([a,b0,c]);stone.poly([b0,d,c])
        for row in range(rows):
            v0=vmax*row/rows;v1=min(vmax,vmax*(row+1)/rows+vmax/rows*roof['tile_overlap'])
            count=cols if not angular else max(5,round(cols*(.28+.72*v0)))
            du=(umax-umin)/count
            for col in range(count+1):
                u0=max(umin,umin+(col-.5*(row%2))*du);u1=min(umax,umin+(col+1-.5*(row%2))*du-.009)
                if u1-u0<.025:continue
                v1a=v1-rng.uniform(0,.006)
                uv=[(u0,v0),(u1,v0),(u1,v1a-.015),(u1-(u1-u0)*.18,v1a),(u0+(u1-u0)*.18,v1a),(u0,v1a-.015)]
                pts=[add(surface(u,v),[0,.022+.002*(rows-row),0]) for u,v in uv]
                meshes['slate-'+str(rng.randrange(3))].prism(pts,[0,.032,0])
    def roofs():
        tiling(front,left,right,1,15)
        tiling(back,left,right,1,11)
        # Native profile sweeps use exactly the same cubic roof contours.
        for side in [left,right]:
            for fn in [front,back]:
                curve=slope_curve(fn,side)
                trim.native_path([curve],.085,10,profile_x=[1,0,0]);gold.native_path([[add(q,[0,.07,0]) for q in curve]],.023,10,profile_x=[1,0,0])
        for fn in [front,back]:
            curves=eave_path() if fn==front else [[back(left,1),back(left+(right-left)/3,1),back(left+2*(right-left)/3,1),back(right,1)]]
            trim.native_path(curves,.095,10,profile_x=[0,0,1]);gold.native_path([[add(q,[0,.07,0]) for q in c] for c in curves],.018,10,profile_x=[0,0,1])
        # Individual ridge caps retain a deliberate architectural edge.
        for i in range(25):
            x=left+(i+.5)*(right-left)/25
            trim.tube([[x-.18,m['ridge']+.085,m['ridge_z']],[x+.18,m['ridge']+.085,m['ridge_z']]],.095,8)
        cx,cz=b['center'];rad=b['radius']+.25
        bay_profile=[[roof['bay_apex']],[roof['bay_apex']-.08],[3.80],[3.70]]
        apex=right+.015
        def bayroof(a,v):
            t=.04+.96*v
            return [apex*(1-t)+t*(cx+rad*math.cos(a)),cubic(bay_profile,t)[0],cz+t*rad*math.sin(a)]
        tiling(bayroof,-math.pi/2,math.pi/2,1,9,True)
        # Two tangent quarter-circle Beziers; circle approximation is explicit.
        k=4*(math.sqrt(2)-1)/3
        rim=[[[cx,3.7,cz-rad],[cx+k*rad,3.7,cz-rad],[cx+rad,3.7,cz-k*rad],[cx+rad,3.7,cz]],[[cx+rad,3.7,cz],[cx+rad,3.7,cz+k*rad],[cx+k*rad,3.7,cz+rad],[cx,3.7,cz+rad]]]
        trim.native_path(rim,.09,10,profile_x=[0,1,0]);gold.native_path([[add(q,[0,.075,0]) for q in c] for c in rim],.018,10,profile_x=[0,1,0])
        for a in [-math.pi/2,0,math.pi/2]:
            pts=[bayroof(a,j/24) for j in range(25)];gold.tube([add(q,[0,.055,0]) for q in pts],.025,6)
        # Closed green gable field in the right end, ornament lies in its plane.
        pts=[back(right,j/16) for j in range(16,-1,-1)]+[front(right,j/16) for j in range(1,17)]
        fields.prism(pts,[.025,0,0])
        pts=[back(left,j/16) for j in range(16,-1,-1)]+[front(left,j/16) for j in range(1,17)]
        fields.prism(pts,[-.025,0,0])
        tree_motif(0,0,3.0,2.2,0,gold,lambda x,y,z:[right+.12,3.76+y,m['ridge_z']+x*(1-y/2.2)*1.6])
        # Gable leader/finial: a true solid ornament above the ridge.
        foliate_spray(0,0,.65,.85,0,lambda x,y,z:[right+.12,m['ridge']-.05+y,m['ridge_z']+x],normal=[1,0,0])
        # Exposed front entrance canopy, open beneath it.
        arch(trim,1.45,.82,3.0,3.2,p['front'],.12,spring_fraction=.82)
        arch(gold,1.45,.82,2.77,3.06,p['front']+.035,.026,spring_fraction=.82)
        # A real spandrel occupies the space between the outer arch and roof;
        # raising the eave alone exposed an unfinished black soffit.
        level=(p['front']-m['ridge_z'])/(p['front']+.4-m['ridge_z'])
        upper=[]
        for j in range(17):
            x=-.05+3*j/16;upper.append([x,front(x,level)[1]-.035,p['front']+.05])
        for a0,a1 in zip(upper,upper[1:]):
            lo0=[a0[0],.82+arch_top(a0[0]-1.45,3.,3.2,.82)+.08,a0[2]]
            lo1=[a1[0],.82+arch_top(a1[0]-1.45,3.,3.2,.82)+.08,a1[2]]
            assert lo0[1]<a0[1] and lo1[1]<a1[1], 'Portico arch crosses the roof field'
            fields.prism([lo0,lo1,a1,a0],[0,0,-.18])
        # Continue the cheek beyond the portico pier: the roof backing is not
        # a finished vertical facade and must not remain exposed here.
        for j in range(4):
            x0=2.95+(right-2.95)*j/4;x1=2.95+(right-2.95)*(j+1)/4
            pts=[[x0,3.56,p['front']+.05],[x1,3.56,p['front']+.05],[x1,front(x1,level)[1]-.035,p['front']+.05],[x0,front(x0,level)[1]-.035,p['front']+.05]]
            fields.prism(pts,[0,0,-.18])
        rib=[[2.95,3.57,p['front']+.10],[2.95,4.12,p['front']+.10],[3.25,4.58,p['front']+.10],[right+.04,front(right,level)[1]-.08,p['front']+.10]]
        trim.curve(rib,.055,end_radius=.018,profile_x=[0,0,1])
        # Stone rises out of the pier beneath the curved rib. Green remains a
        # bounded spandrel above it, rather than an unsupported rectangular flap.
        strip=[cubic(rib,j/12) for j in range(13)]
        trim.prism([[strip[0][0],3.56,strip[0][2]-.025],[strip[-1][0],3.56,strip[-1][2]-.025]]+[[q[0],q[1],q[2]-.025] for q in strip[::-1]],[0,0,-.20])
        foliate_spray(0,0,.65,.92,0,lambda x,y,z:[2.83-.42*y+x,3.73+y,p['front']+.105],normal=[0,0,1])
        # Same exact contour as the roof, tucked under the eave.
        trim.native_path([[add(q,[0,-.035,0]) for q in c] for c in eave_path(level,p['front']+.05)],.045,10,profile_x=[0,0,1])
        for root,tip in [([.08,3.61,p['front']+.08],[.65,3.89,p['front']+.08])]:
            middle=add(mul(root,.45),mul(tip,.55))
            gold.curve([root,add(middle,[0,.05,0]),add(tip,[.08,-.1,0]),tip],.018,n=8,sides=4,end_radius=.006,profile_x=[0,0,1])
    mark('curved-slate-roofs-and-finials','roof',roofs)

    def chimney():
        x,z=-2.7,.2
        wall([x,5.35,z],.65,1.85,.58)
        wall([x+.35,5.35,z-.06],.62,1.85,.12,math.pi/2)
        trim.box([x,6.27,z],[.88,.13,.79]);trim.box([x,6.4,z],[.96,.12,.85])
        arch(trim,x,5.2,.48,.92,z+.34,.032)
        foliate_spray(x,5.2,.38,.88,z+.38,pairs=1)
    mark('carved-chimney','architecture',chimney)

    def props():
        # Porch urns, natural flower crowns, lantern frame and mullions.
        for x,z in [(-.32,2.45),(-3.85,2.15),(2.45,2.65)]:
            trim.lathe([x,.8,z],[(0,.17),(.08,.21),(.12,.14),(.22,.19),(.42,.25),(.56,.23),(.60,.28),(.66,.28)],12)
            ground.lathe([x,1.40,z],[(0,.22),(.025,.22)],12)
        x,y,z=2.4,2.57,cfg['entrance']['wall_z']+.42
        gold.curve([[x,y+.12,z],[x,y+.63,z],[x-.16,y+.7,z],[x-.2,y+.49,z]],.018,16,6)
        for dx in [-.12,.12]:
            for dz in [-.10,.10]:gold.tube([[x+dx,y-.25,z+dz],[x+dx,y+.15,z+dz]],.015,5)
        gold.box([x,y-.25,z],[.28,.035,.25]);gold.box([x,y+.16,z],[.28,.045,.25])
        # Warm opaque core: lighting approximation, not a point-light feature.
        flowers.box([x,y-.03,z],[.16,.26,.13])
        for s in [-1,1]:
            gold.curve([[x+s*.16,y+.17,z],[x+s*.12,y+.3,z],[x+s*.03,y+.32,z],[x,y+.34,z]],.016,8,5)
    mark('urns-bench-and-lantern','props',props)

    def leaf(center,length,width,angle,tilt=0):
        u=[math.cos(angle),tilt,math.sin(angle)];v=[-math.sin(angle),.25,math.cos(angle)]
        points=[center,add(center,add(mul(u,length*.45),mul(v,width))),add(center,mul(u,length)),add(center,sub(mul(u,length*.45),mul(v,width))),add(center,add(mul(u,length*.48),[0,.035,0]))]
        start=len(foliage.positions);foliage.positions.extend(points)
        foliage.triangles.extend([[start,start+1,start+4],[start+1,start+2,start+4],[start+2,start+3,start+4],[start+3,start,start+4]])

    def gardens():
        ground.box([0,-.08,.5],[200,.12,200])
        for k in range(cfg['garden']['leaf_clusters']):
            if k<3:cx,cz=[(-.32,2.45),(-3.85,2.15),(2.45,2.65)][k];cy=1.46
            else:
                side=rng.randrange(3)
                if side==0:cx=rng.uniform(-5.3,-.4);cz=rng.uniform(3.45,3.95)
                elif side==1:cx=rng.uniform(2.35,4.7);cz=rng.uniform(2.7,3.8)
                else:cx=rng.choice([-5.0,4.6]);cz=rng.uniform(-2.1,2.6)
                cy=.1
            for j in range(cfg['garden']['leaves_per_cluster']):
                a=rng.uniform(0,TAU);height=rng.uniform(.13,.52)
                pos=[cx+rng.uniform(-.25,.25),cy+height*.65,cz+rng.uniform(-.25,.25)]
                foliage.tube([[cx,cy,cz],pos],.008,4,end_radius=.004)
                leaf(pos,rng.uniform(.12,.25),rng.uniform(.035,.075),a,rng.uniform(-.3,.7))
                if j%4==0:
                    for petal in range(5):
                        q=petal*TAU/5;flowers.poly([pos,add(pos,[.06*math.cos(q),.025,.06*math.sin(q)]),add(pos,[.06*math.cos(q+.9),.025,.06*math.sin(q+.9)])])
        # Climbing vines: continuous stems with distributed individual leaves.
        for cx,cz,height in [(-4.65,3.3,2.8),(-1.6,3.26,2.6),(-.22,1.48,3.1),(2.2,cfg['entrance']['wall_z']+.38,2.0),(4.45,1.4,2.7),(-2.85,-.5,5.6)]:
            path=[[cx+.08*math.sin(i*.7),.2+height*i/23,cz+.06*math.cos(i*.8)] for i in range(24)]
            foliage.tube(path,.013,5,end_radius=.005)
            for i in range(1,24):
                s=1 if i%2 else -1
                leaf(path[i],.14,.055,(0 if s==1 else math.pi)+rng.uniform(-.5,.5),rng.uniform(-.1,.3))
        # Irregular paving apron, with a clear central path to the steps.
        for row in range(3):
            for j in range(17):
                x=-5.55+(j+.5)*.62+rng.uniform(-.03,.03)+.31*(row%2);z=3.9+(row+.5)*.53;angle=rng.uniform(-.03,.03)
                if (row+j)%2==0:paving.chamfer_box([x,.025,z],[stone_rng.uniform(.54,.59),.06,stone_rng.uniform(.44,.50)],stone_rng.uniform(.006,.009),angle)
                else:paving.box([x,.025,z],[.59,.06,.50],angle)
        for row in range(10):
            paving.box([-5.2,.02,-2.7+row*.65],[.6,.06,.61],rng.uniform(-.02,.02))
    mark('garden-and-paving','landscape',gardens)

    mats={
        'masonry':([.43,.40,.32],material(51,low=(.78,.76,.7),bump=.006),'flat'),
        'stone-blocks':([.73,.68,.53],material(9,bump=.008),'flat'),
        'carved-stone':([.82,.77,.61],material(3,bump=.003),'smooth'),
        'roof-backing':([.055,.075,.074],{},'flat'),
        'slate-0':([.16,.24,.245],material(11,bump=.004,roughness=.8),'flat'),
        'slate-1':([.22,.29,.28],material(21,bump=.004,roughness=.8),'flat'),
        'slate-2':([.27,.32,.29],material(31,bump=.004,roughness=.8),'flat'),
        'oak':([.27,.14,.055],material(7,(27,1.7,27),.005),'flat'),
        'bronze':([.60,.36,.105],{'reflectance':{'roughness':.4,'f0':.12}},'smooth'),
        'window-infill':([.025,.065,.055],{'reflectance':{'roughness':.25,'f0':.10}},'flat'),
        'green-panels':([.15,.24,.105],material(95,(4,4,4),.002),'flat'),
        'ground':([.44,.40,.31],material(71,(3,3,3),.006),'flat'),
        'leaves':([.12,.26,.045],{**material(87,(7,7,7),.001,low=(.65,.8,.6),high=(1.2,1.2,.8)),'transmission':.15},'smooth'),
        'flowers':([.91,.79,.47],{},'flat'),
        'paving':([.60,.57,.45],material(33,bump=.007),'flat')}
    assets=[{'id':k,'geometry':v.geometry(),'material':mats[k][1],'shading':mats[k][2],'_sweeps':v.sweeps,'_reliefs':v.reliefs,'_boxes':v.boxes} for k,v in meshes.items() if v.positions or v.sweeps or v.boxes]
    instances=[{'id':'cottage-'+a['id'],'asset':a['id'],'position':[0,0,0],'rotation':[0,0,0,1],'scale':1,'color':mats[a['id']][0]} for a in assets]
    h=cfg['column']['height'];w=cfg['column']['width']*1.4
    stations=[('foot',0,1.35),('base',.06,1.2),('shaft-low',.14,.83),('shaft-high',.83,.76),('capital',.94,1.3),('abacus',1,1.4)]
    column={'id':'porch-column','profile':{'id':'column-elevation','width':w,'height':h},'depth':w,'segments':[1,6,1],'sampling_tolerance':.001,'sections':{'id':'column-sections','cross_section':'rectangle','interpolation':'pchip','stations':[{'id':k,'at':t,'width':r/1.4,'depth':r/1.4} for k,t,r in stations]}}
    for i,pos in enumerate(columns):instances.append({'id':f'column-{i:02}','asset':'porch-column','position':pos,'rotation':[0,0,0,1],'scale':1,'color':[.8,.74,.58]})
    return assets,instances,column,groups


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--recipe',type=Path,default=P/'examples/elven-cottage.recipe.json');ap.add_argument('--skip-validation',action='store_true');args=ap.parse_args()
    cfg=json.loads(args.recipe.read_text())
    if cfg['version']!=6:raise ValueError('Cottage recipe version6 required; reproduce older recipes with their original authoring revision')
    assert 0<=cfg['porch'].get('relief_front_offset',.10)<=1
    assert 0<=cfg['porch'].get('relief_bottom',3.43)<=6 and .05<=cfg['porch'].get('relief_height',.25)<=1
    assert 1<=cfg['porch']['bays']<=6 and 3<=cfg['bay']['windows']<=8
    assert .3<=cfg['roof']['tile_width']<=1 and 4.5<=cfg['roof']['bay_apex']<=6
    assert 0<=cfg['garden']['leaf_clusters']<=32 and 1<=cfg['garden']['leaves_per_cluster']<=16
    assets,instances,column,groups=build(cfg)
    counts={a['id']:{'vertices':len(a['geometry']['positions'])+sum((len(s['curves'])*s['spans']+1)*len(s['profile']) for s in a['_sweeps']),
                      'triangles':len(a['geometry']['triangles'])+sum(2*len(s['curves'])*s['spans']*len(s['profile'])+2*(len(s['profile'])-2) for s in a['_sweeps'])} for a in assets}
    for a in assets:
        counts[a['id']]['vertices']+=sum(24 if b['bevel'] else 8 for b in a['_boxes'])
        counts[a['id']]['triangles']+=sum(44 if b['bevel'] else 12 for b in a['_boxes'])
    assert all(c['vertices']<=16384 and c['triangles']<=32768 for c in counts.values()),counts
    assert sum(c['vertices'] for c in counts.values())<32768,counts
    assert sum(c['triangles'] for c in counts.values())<65000,counts
    assert len(assets)+1<=16 and len(instances)<=32
    for d in ['examples','artifacts']:(R/d).mkdir(parents=True,exist_ok=True)
    os.environ['IGNIS_VERIFY_ROOT']=str(R)
    from verify_lab import Client
    c=Client(P/'target/release/ignis');call=c.call
    def capture(name,width=1536,height=1024):
        path=f'artifacts/{name}.png';meta=call('capture',path=path,width=width,height=height);return (R/path).read_bytes(),meta
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'cottage-authoring','version':'1'}});c.request('notifications/initialized',notification=True)
        call('generate',recipe={'width':24,'depth':24,'seed':cfg['seed'],'obstacles':0})
        scene=call('inspect')['scene']
        invisible=[]
        for entity in scene['entities']:
            e=copy.deepcopy(entity);e['visible']=False;invisible.append({'op':'upsert','entity':e})
        call('edit_scene',edits=invisible)
        native_plans={};reliefs={};native_boxes={}
        for a in assets:
            if a['_sweeps']:
                before=call('inspect');planned=call('plan_mesh_sweep',sweeps=a['_sweeps'])['plan'];assert call('inspect')==before
                native_plans[a['id']]=planned
                for s,prepared in zip(a['_sweeps'],planned['pieces']):s['spans']=prepared['spans']
                counts[a['id']]={'vertices':len(a['geometry']['positions'])+planned['vertices']+sum(24 if b['bevel'] else 8 for b in a['_boxes']),'triangles':len(a['geometry']['triangles'])+planned['triangles']+sum(44 if b['bevel'] else 12 for b in a['_boxes'])}
        assert sum(c['vertices'] for c in counts.values())<32768,counts
        for a in assets:
            sweeps=a.pop('_sweeps')
            leaves=a.pop('_reliefs')
            boxes=a.pop('_boxes')
            if leaves:
                planned=native_plans[a['id']]['pieces'];v=len(a['geometry']['positions']);t=len(a['geometry']['triangles']);parts=[]
                for i,piece in enumerate(planned):
                    if i in leaves:parts.append({'sweep_index':i,'first_vertex':v,'vertices':piece['vertices'],'first_triangle':t,'triangles':piece['triangles']})
                    v+=piece['vertices'];t+=piece['triangles']
                reliefs[a['id']]=parts
            if a['geometry']['positions']:
                edit={'kind':'put_asset','asset':a};assert len(json.dumps(edit).encode())<1_900_000
                call('edit_mesh',edit=edit)
            if sweeps:
                edit={'kind':'sweep','id':a['id'],'sweeps':sweeps,'append':bool(a['geometry']['positions'])}
                assert len(json.dumps(edit).encode())<1_900_000
                before=call('inspect');preview=call('preview_mesh_edit',edit=edit);assert call('inspect')==before
                call('edit_mesh',edit=edit);assert call('inspect')['scene']['meshes']==preview['meshes']
                call('edit_mesh',edit={'kind':'set_material','asset':a['id'],'material':a['material']})
                call('edit_mesh',edit={'kind':'set_shading','asset':a['id'],'shading':a['shading']})
            if boxes:
                edit={'kind':'chamfer_boxes','id':a['id'],'boxes':boxes,'append':bool(a['geometry']['positions'] or sweeps)}
                before=call('inspect');preview=call('preview_mesh_edit',edit=edit);assert call('inspect')==before
                call('edit_mesh',edit=edit);assert call('inspect')['scene']['meshes']==preview['meshes']
                native_boxes[a['id']]=boxes
                if not a['geometry']['positions'] and not sweeps:
                    call('edit_mesh',edit={'kind':'set_material','asset':a['id'],'material':a['material']})
                    call('edit_mesh',edit={'kind':'set_shading','asset':a['id'],'shading':a['shading']})
        column_edit={'kind':'put','definition':column,'detail_policy':'physical','animation_translation_policy':'physical'}
        unchanged=call('inspect');call('preview_plant',edit=column_edit);assert call('inspect')==unchanged
        call('edit_plant',edit=column_edit)
        call('edit_mesh',edit={'kind':'set_material','asset':'porch-column','material':material(3,bump=.003)})
        call('edit_mesh',edit={'kind':'set_shading','asset':'porch-column','shading':'smooth'})
        edits=[{'op':'mesh','edit':{'kind':'put_instance','instance':i}} for i in instances]
        edits.extend([{'op':'set_camera','camera':cfg['camera']},{'op':'set_light','light':cfg['light']}])
        predicted=call('preview_scene_edits',edits=edits)['scene'];call('edit_scene',edits=edits);assert call('inspect')['scene']==predicted
        from cottage_composition import adjust_lantern
        composition=adjust_lantern(call,cfg['entrance']['wall_z'],cfg['entrance']['lantern_offset'])
        backing=call('inspect_mesh_topology',asset='roof-backing')['topology']
        assert backing['boundary_edges']==backing['non_manifold_edges']==0 and not backing['isolated_vertices']
        islands=call('inspect_mesh_islands',asset='roof-backing')['topology']['islands'];assert len(islands)==3
        # Manifold count alone does not prove consistent triangle orientation.
        for edge in backing['edges']:
            signs=[]
            for face in edge['faces']:
                f=backing['faces'][face];pairs=list(zip(f,f[1:]+f[:1]))
                signs.append(1 if tuple(edge['vertices']) in pairs else -1)
            assert sum(signs)==0
        relief_count=0
        for family,parts in reliefs.items():
            topo=call('inspect_mesh_topology',asset=family)['topology']
            assert topo['boundary_edges']==topo['non_manifold_edges']==0 and not topo['isolated_vertices']
            positions=call('inspect_mesh',asset=family)['asset']['geometry']['positions']
            def area(ring):
                result=[0,0,0]
                for a,b0 in zip(ring,ring[1:]+ring[:1]):result=add(result,cross(a,b0))
                return math.sqrt(dot(result,result))*.5
            for piece in parts:
                start=piece['first_vertex'];n=piece['vertices']//4
                a0=area(positions[start:start+4]);mid=start+(n//2)*4;am=area(positions[mid:mid+4]);end=start+(n-1)*4;ae=area(positions[end:end+4])
                assert 0<ae<a0<am
                relief_count+=1
        for family in native_boxes:
            top=call('inspect_mesh_topology',asset=family)['topology']
            assert top['boundary_edges']==top['non_manifold_edges']==0 and not top['isolated_vertices']
        call('save_scene',path='examples/elven-cottage.scene.json');hero,meta=capture('hero')
        print(json.dumps({'stage':'authored','assets':len(assets)+1,'instances':len(instances),'vertices':sum(c['vertices'] for c in counts.values()),'triangles':meta['mesh_triangle_count']}),flush=True)
        # Other views reveal depth, openings, back and sampled roof profile.
        for name,cam in {
            'front':{'position':[0,5,19],'target':[0,2.7,.5],'fov_degrees':37},
            'west':{'position':[-12,7,13],'target':[-.5,2.6,.5],'fov_degrees':38},
            'rear':{'position':[-10,8,-15],'target':[0,2.7,0],'fov_degrees':38},
            'detail':{'position':[4.9,4.1,8.5],'target':[1.3,2.6,1.8],'fov_degrees':36},
        }.items():call('set_camera',camera=cam);capture(name)
        call('set_camera',camera=cfg['camera'])
        if not args.skip_validation:
            assert capture('repeat')[0]==hero
            baseline=call('inspect')['scene']
            # Reject a malformed mesh before any publication.
            bad={'id':'invalid','geometry':{'positions':[[0,0,0]],'triangles':[[0,1,2]]}}
            before=call('inspect');call('edit_mesh',edit={'kind':'put_asset','asset':bad},expected_error=True);assert call('inspect')==before
            definition=copy.deepcopy(column);definition['profile']['width']*=1.05
            call('edit_plant',edit={'kind':'put','definition':definition,'detail_policy':'physical','animation_translation_policy':'physical'})
            assert capture('column-edited')[0]!=hero
            call('undo');assert call('inspect')['scene']==baseline and capture('undo')[0]==hero
            call('redo');call('undo');assert call('inspect')['scene']==baseline
            call('save_scene',path='examples/elven-cottage.scene.json')
            call('step',ticks=60);assert capture('tick60')[0]==hero
            call('save_replay',path='artifacts/cottage.replay.json');result=call('run_replay',path='artifacts/cottage.replay.json');assert result['meshes_matched']
            for label,t in [('replay',60),('prefix',0)]:
                path=f'artifacts/{label}.png';call('capture_replay',replay_path='artifacts/cottage.replay.json',path=path,tick=t,width=1536,height=1024);assert (R/path).read_bytes()==hero
            call('load_scene',path='examples/elven-cottage.scene.json');assert call('inspect')['scene']==baseline and capture('loaded')[0]==hero
            call('reset');assert capture('reset')[0]==hero
            capture('viewport',800,600)
        state=call('inspect');cg=state['scene']['meshes']['assets'][-1]['geometry'];counts['porch-column']={'vertices':len(cg['positions']),'triangles':len(cg['triangles'])}
        report={'passed':not args.skip_validation,'build_id':state['build_id'],'recipe_sha256':hashlib.sha256(args.recipe.read_bytes()).hexdigest(),'assets':len(state['scene']['meshes']['assets']),'instances':len(instances),'vertices':sum(v['vertices'] for v in counts.values()),'triangles':sum(v['triangles'] for v in counts.values()),'geometry':counts,'draw_triangles':meta['mesh_triangle_count'],'parts':groups,'composition':composition,'native_sweep_plans':native_plans,'native_reliefs':reliefs,'relief_leaf_count':relief_count,'native_chamfer_boxes':native_boxes,'native_box_count':sum(len(v) for v in native_boxes.values()),'roof_checks':{'cubic_surface_contours_match':True,'monotone_eave_and_fall':True,'portico_below_roof_field':True,'backing_closed_oriented_islands':len(islands)},'checks':[] if args.skip_validation else ['native_part_transform_preview','plant_preview_pure','preview_matches_commit','roof_shared_contours_and_clearance','roof_backing_manifold_orientation','relief_closed_positive_taper','chamfered_stone_closed_manifold','malformed_atomic','column_edit_undo_redo','repeat','tick60_static','replay','prefix','load','reset','viewport'],'limitations':['Native columns/profile sweeps/chamfer boxes; other surfaces computed in authoring client','No authored interior or collision','Hidden surfaces inferred','Artistic acceptance separate']}
        (R/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({k:v for k,v in report.items() if k not in ['geometry','parts','composition','native_sweep_plans','native_reliefs','native_chamfer_boxes']}),flush=True)
    finally:c.close()


if __name__=='__main__':main()
