"""Image E02 study: authored geometry published exclusively via public Ignis MCP."""

import argparse
import hashlib
import json
import math
from pathlib import Path
import random
import subprocess

BINARY = '/Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis'
SOURCE = '/Volumes/Zero/EpicGames/Unreal Projects/LandOfChaos/docs/art/settlements/residential-concepts-v2/E02.png'
ROOT = Path(__file__).resolve().parents[1] / 'artifacts/house-e02'


class Engine:
    def __init__(self, root=ROOT, binary=BINARY):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)
        (self.root/'native').mkdir(exist_ok=True)
        self.log = open(self.root / 'native/stderr.log', 'w')
        self.process = subprocess.Popen([binary, '--root', str(self.root), '--mcp'],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=self.log, text=True)
        self.sequence = 0
        self.revision = None
        self.initialization=self.rpc('initialize', {'protocolVersion': '2024-11-05', 'capabilities': {},
                               'clientInfo': {'name': 'e02-study', 'version': '1'}})
        self.process.stdin.write(json.dumps({'jsonrpc': '2.0', 'method': 'notifications/initialized'})+'\n')
        self.process.stdin.flush()

    def rpc(self, method, params):
        self.sequence += 1
        request = {'jsonrpc': '2.0', 'id': self.sequence, 'method': method, 'params': params}
        self.process.stdin.write(json.dumps(request, separators=(',', ':'),allow_nan=False)+'\n')
        self.process.stdin.flush()
        while True:
            line = self.process.stdout.readline()
            if not line:
                raise RuntimeError('Ignis MCP closed; inspect native/stderr.log')
            result = json.loads(line)
            if result.get('id') == self.sequence:
                if 'error' in result:
                    raise RuntimeError(result['error'])
                return result['result']

    def command(self, op, **kwargs):
        arguments = {'command': {'op': op, **kwargs}}
        if self.revision is not None:
            arguments['expected_revision'] = self.revision
        result = self.rpc('tools/call', {'name': 'engine_command', 'arguments': arguments})
        if result.get('isError'):
            raise RuntimeError(result)
        data = result.get('structuredContent')
        if data is None:
            data = json.loads(next(c['text'] for c in result['content'] if c['type'] == 'text'))
        if 'revision' in data and op not in {'capture_replay','run_replay','inspect_replay'}:
            self.revision = data['revision']
        return data

    def close(self):
        self.process.stdin.close()
        self.process.wait(timeout=20)
        self.log.close()


def dump(name, data):
    (ROOT / name).write_text(json.dumps(data, indent=2, ensure_ascii=False)+'\n')


def add(a, b):
    return [a[i]+b[i] for i in range(3)]


def sub(a, b):
    return [a[i]-b[i] for i in range(3)]


def mul(a, s):
    return [v*s for v in a]


def cross(a, b):
    return [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]


def unit(a):
    length = math.sqrt(sum(v*v for v in a))
    return mul(a, 1/length) if length > 1e-10 else [0, 1, 0]


def lerp(a, b, t):
    return [(1-t)*a[i]+t*b[i] for i in range(len(a))]


def bezier(a, b, c, count=7):
    return [[(1-t)**2*a[j]+2*t*(1-t)*b[j]+t*t*c[j] for j in range(len(a))]
            for t in [i/count for i in range(count+1)]]


class Mesh:
    def __init__(self, name, color):
        self.name, self.color = name, color
        self.positions, self.triangles = [], []

    def polygon(self, points):
        start = len(self.positions)
        self.positions.extend(points)
        for i in range(1, len(points)-1):
            self.triangles.append([start, start+i, start+i+1])

    def box(self, center, size):
        n = len(self.positions)
        self.positions.extend([[center[0]+sx*size[0]/2, center[1]+sy*size[1]/2,
                                center[2]+sz*size[2]/2]
                               for sx, sy, sz in [(-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),
                                                  (-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1)]])
        for a,b,c,d in [(0,3,2,1),(4,5,6,7),(0,4,7,3),(1,2,6,5),(0,1,5,4),(3,7,6,2)]:
            self.triangles.extend([[n+a,n+b,n+c],[n+a,n+c,n+d]])

    def tube(self, points, radius, sides=4, radii=None):
        n = len(self.positions)
        for i,p in enumerate(points):
            tangent = unit(sub(points[min(i+1,len(points)-1)], points[max(i-1,0)]))
            axis = [0,0,1] if abs(tangent[2]) < .8 else [1,0,0]
            v = unit(cross(tangent, axis)); w = cross(tangent,v)
            r = radius if radii is None else radii[i]
            for j in range(sides):
                a=2*math.pi*j/sides
                self.positions.append(add(p, add(mul(v,r*math.cos(a)),mul(w,r*math.sin(a)))))
        for i in range(len(points)-1):
            for j in range(sides):
                a=n+i*sides+j;b=n+i*sides+(j+1)%sides;c=b+sides;d=a+sides
                self.triangles.extend([[a,b,c],[a,c,d]])
        # End caps are separate from adjacent tubes; no implicit boolean union.
        for j in range(1,sides-1):
            self.triangles.append([n,n+j+1,n+j])
            e=n+(len(points)-1)*sides
            self.triangles.append([e,e+j,e+j+1])

    def beam(self, a, b, radius):
        self.tube([a,b],radius)

    def lathe(self, center, rings, sides=8):
        n=len(self.positions)
        for y,r in rings:
            for j in range(sides):
                a=2*math.pi*j/sides
                self.positions.append([center[0]+r*math.cos(a),center[1]+y,center[2]+r*math.sin(a)])
        for i in range(len(rings)-1):
            for j in range(sides):
                a=n+i*sides+j;b=n+i*sides+(j+1)%sides
                self.triangles.extend([[a,b,b+sides],[a,b+sides,a+sides]])

    def weld(self):
        vertices,lookup,indices=[],{},[]
        for p in self.positions:
            key=tuple(round(v,9) for v in p)
            if key not in lookup:
                lookup[key]=len(vertices);vertices.append(list(key))
            indices.append(lookup[key])
        self.positions=vertices
        self.triangles=[[indices[i] for i in tri] for tri in self.triangles]
        self.triangles=[t for t in self.triangles if len(set(t))==3]


def arch(w, h, count=8):
    spring=h*.58
    left=bezier([-w/2,spring],[-w*.48,h*.85],[0,h],count)
    right=[[ -p[0],p[1]] for p in reversed(left[:-1])]
    return [[-w/2,0]]+left+right+[[w/2,0]]


def plane(cx, y, z, normal='front'):
    if normal=='front': return lambda p:[cx+p[0],y+p[1],z+p[2] if len(p)>2 else z]
    return lambda p:[cx+(p[2] if len(p)>2 else 0),y+p[1],z-p[0]]


def frame(mesh, mapper, w, h, radius, count=7):
    mesh.tube([mapper(p) for p in arch(w,h,10 if w>1.8 else min(count,4))],radius)


def trace(mesh, mapper, points, radius, taper=True):
    """Raised planar tracery strip: retains open regions without hidden fill."""
    n=len(mesh.positions)
    for i,p in enumerate(points):
        a=points[max(0,i-1)];b=points[min(len(points)-1,i+1)]
        dx,dy=b[0]-a[0],b[1]-a[1];length=math.hypot(dx,dy) or 1
        r=radius*(1-.65*i/max(1,len(points)-1)) if taper else radius
        mesh.positions.extend([mapper([p[0]-dy/length*r,p[1]+dx/length*r]),
                               mapper([p[0]+dy/length*r,p[1]-dx/length*r])])
    for i in range(len(points)-1):
        a=n+i*2;mesh.triangles.extend([[a,a+1,a+3],[a,a+3,a+2]])


def tree(mesh, mapper, w, h, fine=False, count=5):
    # Shared branching grammar; frame-specific proportions and twig count remain explicit.
    trace(mesh,mapper,[[0,0],[0,h*.36],[0,h*.70],[0,h*.96]],w*(.018 if fine else .032))
    count=6 if w>1.8 and not fine else min(count,3)
    for level, span in [(0.32,.43),(.57,.31),(.76,.18)]:
        for side in [-1,1]:
            end=[w*span*side,h*(level+.24)]
            points=bezier([0,h*level],[w*.08*side,h*(level+.18)],end,count)
            trace(mesh,mapper,points,w*.022)
            if not fine and level<.6:
                for branch in [-1,1]:
                    start=points[int(count*.6)]
                    tip=[end[0]+w*.07*side*branch,end[1]+h*.1]
                    ps=bezier(start,[end[0],end[1]+h*.04],tip,2)
                    trace(mesh,mapper,ps,w*.010)


COLORS={'stone':[.62,.55,.43],'trim':[.76,.68,.53], 'wood':[.25,.14,.065],
        'gold':[.47,.32,.13],'roof-dark':[.13,.17,.135], 'roof-a':[.24,.29,.235],
        'roof-b':[.31,.33,.26],'glass':[.13,.19,.16], 'inlay':[.10,.26,.22],
        'foliage-dark':[.065,.15,.025],'foliage-light':[.20,.29,.055],
        'flowers-ivory':[.90,.81,.58],'flowers-violet':[.39,.20,.46], 'lantern':[1,.63,.16]}


def build_house(stage='detail', variant=2):
    groups={k:Mesh('e02-'+k,c) for k,c in COLORS.items()}
    st,tr,wd,go,rd,ra,rb,gl,il,fd,fl,fw,fv,la=[groups[k] for k in COLORS]
    detailed=stage!='mass'
    main_shift=-3.7 if variant>=2 else (-1.8 if variant else 0)
    rng=random.Random(20261027)
    mapping={}
    def mark(id, names): mapping[id]=['e02-'+n for n in names]
    def wing_point(p):
        if variant<2:return p
        return [-2.14+.728*p[0]+.731*(p[2]-2.75),p[1],-1.03-.779*p[0]+.683*(p[2]-2.75)]
    def wing_box(mesh,center,size):
        before=len(mesh.positions);mesh.box(center,size)
        if variant>=2:
            mesh.positions[before:]=[wing_point(p) for p in mesh.positions[before:]]
    def checkpoint():return {k:len(m.positions) for k,m in groups.items()}
    def wing_since(start):
        if variant>=2:
            for k,m in groups.items():m.positions[start[k]:]=[wing_point(p) for p in m.positions[start[k]:]]
    # Elevated courtyard, curved access steps and masonry plinths.
    def ellipse_ring(mesh, rx, rz, y, a=0, b=2*math.pi, thick=.14, cy=.0, count=40):
        for i in range(count):
            t0=a+(b-a)*i/count;t1=a+(b-a)*(i+1)/count
            p=lambda t,r:[rx*r*math.cos(t),y,cy+rz*r*math.sin(t)]
            mesh.polygon([p(t0,1),p(t1,1),p(t1,1-thick),p(t0,1-thick)])
    floor=[]
    patio_rx,patio_rz=(6.3,4.7) if variant>=2 else (5.8,4.0)
    gap_a,gap_b=(.75,1.45) if variant>=2 else (1.22,1.95)
    for i in range(49):
        a=2*math.pi*i/48;floor.append([patio_rx*math.cos(a),.55,.0+patio_rz*math.sin(a)])
    st.polygon([[0,.55,0]]+list(reversed(floor)))
    for i in range(48):
        p=floor[i];q=floor[i+1]
        st.polygon([p,[p[0],.06,p[2]],[q[0],.06,q[2]],q])
    for step in range(3):
        ellipse_ring(st,patio_rx+.27*(3-step),patio_rz+.30*(3-step),.16+.13*step,
                     a=gap_a,b=gap_b,thick=.16,count=24)
        rx=patio_rx+.27*(3-step);rz=patio_rz+.30*(3-step);top=.16+.13*step
        for i in range(24):
            a=gap_a+(gap_b-gap_a)*i/24;b=gap_a+(gap_b-gap_a)*(i+1)/24
            p=[rx*math.cos(a),top,rz*math.sin(a)];q=[rx*math.cos(b),top,rz*math.sin(b)]
            st.polygon([p,q,[q[0],top-.13,q[2]],[p[0],top-.13,p[2]]])
    st.box([3.45,1.0,-.7+main_shift],[3.7,.9,5.3])
    wing_box(st,[-2.0,.85,-1.2],[7.2,.6,3.0])
    # Building walls with real openings; back and occluded walls are a visual hypothesis.
    if variant<2:st.box([3.45,2.65,-3.3+main_shift],[3.7,3.2,.18])
    st.box([1.63,2.65,-.8+main_shift],[.18,3.2,5.0])
    wing_box(st,[-2,2.1,-2.55],[7.2,2.55,.20])
    wing_box(st,[-5.55,2.1,-1.2],[.18,2.55,2.7])
    # A façade is filled around each arched opening, instead of dark patches covering a solid wall.
    def opening(cx, bottom, depth, w, h, wall_top, normal='front', kind='window', motif=True):
        mapper=plane(cx,bottom,depth,normal)
        outline=arch(w,h,8)
        # Headers follow the pointed opening; piers between openings are authored separately.
        top=wall_top-bottom
        path=outline[1:-1]
        for p,q in zip(path,path[1:]):
            st.polygon([mapper(p),mapper(q),mapper([q[0],top]),mapper([p[0],top])])
        backing=wd if kind=='door' else gl
        backing.polygon([mapper([p[0],p[1],-.06]) for p in outline])
        frame(tr,mapper,w+.14,h+.10,.075,7)
        frame(go if kind=='door' else tr,mapper,w-.06,h-.02,.028,6)
        if detailed and motif:
            tree(go if kind=='door' else tr,lambda p:mapper([p[0],p[1],.045]),w*.87,h*.94,fine=kind=='door',count=4)
            if kind!='door':
                # Thin warm secondary net seen behind the central pale tree.
                tree(go,lambda p:mapper([p[0],p[1],.012]),w*.78,h*.89,True,3)
        if kind=='door':
            wd.beam(mapper([0,0,.02]),mapper([0,h*.92,.02]),.018)
            if detailed:
                for j in range(1,8):
                    x=-w/2+w*j/8
                    go.beam(mapper([x,.04,.01]),mapper([x,h*.54,.01]),.005)
                for x in [-.055,.055]:
                    go.box(mapper([x,.85,.045]),[.025,.12,.025])
        return mapper

    # Main front (+Z): very large pointed window and asymmetric high gable.
    front=1.9+main_shift
    window_width=2.65 if variant else 2.25
    main_bottom=1.55 if variant else 1.15
    main_h=3.50 if variant else 3.05
    main_top=5.35 if variant else 4.5
    opening(3.4,main_bottom,front,window_width,main_h,main_top)
    for x in [1.72,5.12]: st.box([x,(main_top+1)/2,front],[.32,main_top-1,.26])
    st.box([3.4,main_bottom-.12,front],[3.38,.22,.42]);tr.box([3.4,main_bottom-.01,front+.03],[3.50,.12,.52])
    gable_peak=(-.45 if variant>=2 else .82) if variant else 1.72
    gable_height=(8.95 if variant>=2 else 9.25) if variant else 7.75
    gable=[[1.57,main_top,front],[gable_peak,gable_height,front],[5.25,main_top,front]]
    if variant<2:st.polygon(gable)
    # Tall slender upper-light and green inset on gable.
    opening(2.05,4.2,front+.025,.58,2.12,6.6,motif=True)
    gl.polygon([[2.65,4.8,front+.025],[2.65,6.1,front+.025],[3.85,4.8,front+.025]])
    if detailed:
        tree(go,plane(3.05,4.78,front+.065),.75,1.15,True,4)
    # Right-facing wall +X with three smaller windows.
    def side_top(z):
        if variant<2:return 4.05
        v=max(0,min(1,(2.45+main_shift-z)/6.15))
        rx=-.72+3.02*v;u=(5.32-rx)/(5.93-rx)
        ridge=9.3-2.5*v+.32*math.sin(math.pi*v)**2
        edge=5.35-2.5*v-.27*math.sin(math.pi*v)+.35*v**5
        return min(4.05,edge+(ridge-edge)*(1-u)**1.55+.3*u**6-.10)
    for z in [v+main_shift for v in [.65,-.95,-2.55]]:
        top=side_top(z)
        opening(5.32,1.35,z,1.06,min(2.45,top-1.45),top,'side')
        st.box([5.30,1.19,z],[.32,.20,1.45])
    for z in [v+main_shift for v in [1.48,-.15,-1.75,-3.35]]:
        top=side_top(z)
        st.box([5.31,(top+1.05)/2,z],[.32,top-1.05,.24]);tr.box([5.34,1.12,z],[.46,.20,.40])
    # Low wing front wall hosts both doors and other windows, all facing courtyard.
    wing_z=.35
    wing_start=checkpoint()
    for cx,w,h,kind in [(-4.82,.88,1.95,'door'),(-3.45,.72,1.72,'window'),
                         (-2.15,.78,1.72,'window'),(-.95,.76,1.72,'window')]+([] if variant>=2 else [(.38,1.27,2.24,'door')]):
        bottom=.74 if kind=='door' else 1.22
        opening(cx,bottom,wing_z,w,h,3.38,kind=kind)
        if kind=='window': st.box([cx,1.1,wing_z],[w+.24,.22,.28])
    for x in [-5.45,-4.1,-2.83,-1.56,-.38,1.23]:
        st.box([x,2.02,wing_z-.015],[.32,2.76,.23])
    st.box([-2.0,1.0,wing_z-.07],[7.0,.64,.21])
    wing_since(wing_start)
    if variant>=2:
        opening(-.55,.82,-1.90,1.27,2.24,3.38,kind='door')
        st.box([-1.40,2,-1.94],[.30,2.78,.25]);st.box([.32,2,-1.94],[.30,2.78,.25])
    # Semicircular stair at the main door, separate from patio entrance stairs.
    door_x,door_z=(-.55,-1.90) if variant>=2 else (.38,wing_z)
    for j in range(3):
        rr=.87-.14*j;y=.65+.09*j
        for i in range(16):
            a=math.pi*i/16;b=math.pi*(i+1)/16
            st.polygon([[door_x,y,door_z+.08],[door_x+rr*math.cos(a),y,door_z+.08+rr*.52*math.sin(a)],
                        [door_x+rr*math.cos(b),y,door_z+.08+rr*.52*math.sin(b)]])
    # Clerestory / gallery between the two roof levels.
    gallery=Mesh('e02-gallery',COLORS['trim'])
    gallery.placements=[]
    wing_start=checkpoint()
    for i in range(6):
        x=(-1.7 if variant>=2 else -4.9)+i*1.0
        pv,pt=len(tr.positions),len(tr.triangles)
        opening(x,3.50,.24,.72,1.0,4.62)
        if i==0:
            gallery.positions=[[p[0]-x,p[1]-3.5,p[2]-.24] for p in tr.positions[pv:]]
            gallery.triangles=[[j-pv for j in t] for t in tr.triangles[pt:]]
        del tr.positions[pv:];del tr.triangles[pt:]
        gallery.placements.append(wing_point([x,3.5,.24]))
        st.box([x+.49,4.03,.20],[.18,1.15,.20])
    tr.box([-2.4,3.45,.25],[6.6,.17,.32])
    wing_since(wing_start)
    if variant>=2:gallery.rotation=[0,math.sin(math.radians(47)/2),0,math.cos(math.radians(47)/2)]
    # Porch arcade, with a native sectioned source for five supporting columns.
    porch_front=2.75 if variant else 2.15
    columns_local=[[-5.4,.6,porch_front],[-3.75,.6,porch_front],[-2.10,.6,porch_front],[-.45,.6,porch_front],[1.12,.6,porch_front]]
    columns=[wing_point(p) for p in columns_local]
    wing_start=checkpoint()
    for i in range(4):
        cx=(columns_local[i][0]+columns_local[i+1][0])/2
        mapper=plane(cx,.64,porch_front)
        frame(tr,lambda p:mapper([p[0],p[1]+1.45]),1.65,1.12,.075,8)
        if detailed:
            for side in [-1,1]:
                curve=bezier([side*.81,1.72],[side*.78,2.32],[side*.34,2.62],5)
                go.tube([mapper(p) for p in curve],.026)
    wing_since(wing_start)
    # Roof functions: main asymmetric gable swept along depth; high and low left roofs.
    def main_roof(u,v):
        z=2.45+main_shift-6.15*v
        ridge_x=(-.72+3.02*v if variant>=2 else .68) if variant else 1.58
        if variant>=2:z-=1.5*v*(1-u)
        ridge_y=(9.30-(2.50 if variant>=2 else 2.30)*v if variant else 7.80-.80*v)+.32*math.sin(math.pi*v)**2
        eave_y=(5.35-(2.50 if variant>=2 else 1.30)*v if variant else 4.0)-.27*math.sin(math.pi*v)+.35*(v**5)
        x=ridge_x+(5.93-ridge_x)*u
        y=eave_y+(ridge_y-eave_y)*(1-u)**1.55+.30*u**6
        return [x,y,z]
    def main_back(u,v):
        p=main_roof(0,v)
        return [p[0]-.83*u,p[1]-3.0*u+.20*u**5,p[2]]
    def upper_roof(u,v):
        x=-5.85+(6.55+2.50*v if variant>=2 else 7.15)*u
        z=-2.95+3.15*v
        sag=.43*math.sin(math.pi*u)
        y=4.95-sag*(1-.65*v)-.35*v+.58*(abs(2*u-1)**5)*(1-v*.35)
        return wing_point([x,y,z])
    def porch_roof(u,v):
        x=-5.86+(8.65 if variant>=2 else 7.28)*u
        if variant>=2:x+=1.2*(1-u)**4*(1-v)
        z=.08+2.56*v+.28*math.sin(math.pi*u)*v
        y=3.65-.60*v+.18*v**7+.66*(1-u)**6*(1-v)**2
        if variant: y+=.16*math.sin(2*math.pi*u)*v
        # Raised left hip transitioning into a rounded low eave.
        y+=.66*(1-u)**5*(1-v)**.8
        if variant>=2:y+=.55*(1-u)**5*(1-v)**2
        return wing_point([x,y,z])
    if variant>=2:
        # Front/back gables follow the curved roof; a straight triangle would penetrate its sagging surface.
        for wall_z,base in [(front,main_top),(-3.3+main_shift,1.05)]:
            profile=[]
            for i in range(25):
                u=i/24
                v=(2.45+main_shift-wall_z)/(6.15+1.5*(1-u))
                p=main_roof(u,v);profile.append([p[0],p[1]-.11,wall_z])
            for p,q in zip(profile,profile[1:]):
                def lower(point):
                    bottom=base
                    if wall_z==front and point[0]<1.57:
                        bottom=max(base,gable_height+(base-gable_height)*(point[0]-gable_peak)/(1.57-gable_peak))
                    return [point[0],min(point[1],bottom),wall_z]
                st.polygon([p,q,lower(q),lower(p)])
        # Timber members support the lifted front verge; no CSG/structural solver is implied.
        wd.beam([1.57,main_top,front],[gable_peak,gable_height,front],.07)
    def roof_surface(fn, nu, nv):
        start=len(rd.positions)
        for i in range(nu+1):
            for j in range(nv+1):rd.positions.append(fn(i/nu,j/nv))
        for i in range(nu):
            for j in range(nv):
                a=start+i*(nv+1)+j;b=a+nv+1
                rd.triangles.extend([[a,a+1,b+1],[a,b+1,b]])
        for edge in [[fn(i/12,0) for i in range(13)], [fn(i/12,1) for i in range(13)],
                     [fn(0,j/12) for j in range(13)], [fn(1,j/12) for j in range(13)]]:
            tr.tube(edge,.070);go.tube([[p[0],p[1]+.055,p[2]] for p in edge],.022)
        if detailed:
            # Discrete shingle faces follow the actual curved surface, with row overlap.
            for row in range(nv):
                for col in range(nu):
                    u0=col/nu;u1=(col+.94)/nu
                    v0=row/nv;v1=min(1,(row+1.08)/nv)
                    points=[fn(u0,v0),fn(u1,v0),fn(u1,v1),fn(u0,v1)]
                    points=[[p[0],p[1]+.025+rng.random()*.012,p[2]] for p in points]
                    (ra if rng.random()<.66 else rb).polygon(points)
    roof_surface(main_roof,17 if detailed else 10,15 if detailed else 10)
    roof_surface(main_back,3,12 if detailed else 8)
    roof_surface(upper_roof,18 if detailed else 10,6)
    roof_surface(porch_roof,18 if detailed else 10,7 if detailed else 6)
    # Roof finials at the lifted ends.
    for p in [main_roof(0,0),main_roof(0,1),upper_roof(0,0),upper_roof(1,0),porch_roof(0,0)]:
        go.tube([p,add(p,[.02,.25,0]),add(p,[-.04,.45,0])],.055,
                radii=[.075,.045,.008])
        go.tube([add(p,[-.04,.07,0]),add(p,[-.18,.19,0]),add(p,[-.08,.32,0])],.025)
    # A real dormer protrudes from right-facing main roof slope.
    dormer_z=-1.7+main_shift
    dormer=plane(4.70,4.9,dormer_z,'side')
    gl.polygon([dormer(p) for p in arch(1.1,1.45)])
    frame(go,dormer,1.15,1.5,.045)
    if detailed: tree(go,dormer,.94,1.35,True,4)
    for sign in [-1,1]:
        rd.polygon([[4.75,4.9,dormer_z+sign*.68],[4.70,6.5,dormer_z],[3.65,6.65,dormer_z],
                    [3.65,5.2,dormer_z+sign*.68]])
        tr.tube([[4.8,4.9,dormer_z+sign*.68],[4.75,5.85,dormer_z+sign*.35],[4.7,6.5,dormer_z]],.055)
    # Courtyard rail: curved caps and genuinely open small arches.
    def rail(a,b,panels):
        rx,rz=(6.0,4.45) if variant>=2 else (5.4,3.72)
        for base_y,radius in [(.64,.12),(.85,.10),(1.69,.10)]:
            pts=[[rx*math.cos(t),base_y,rz*math.sin(t)] for t in [a+(b-a)*i/18 for i in range(19)]]
            tr.tube(pts,radius)
        for i in range(panels):
            t=a+(b-a)*(i+.5)/panels
            x,z=rx*math.cos(t),rz*math.sin(t)
            tangent=unit([-rx*math.sin(t),0,rz*math.cos(t)])
            mapper=lambda p:[x+tangent[0]*p[0],.86+p[1],z+tangent[2]*p[0]]
            w=(b-a)*math.sqrt((rx*math.sin(t))**2+(rz*math.cos(t))**2)/panels
            frame(tr,mapper,w*.85,.78,.042,5)
            if detailed: tree(tr,mapper,w*.73,.71,True,3)
        for t in [a,b]:
            x,z=rx*math.cos(t),rz*math.sin(t)
            for y,s,h in [(.75,.58,.34),(1.21,.38,.72),(1.65,.61,.13),(1.78,.64,.13)]:
                tr.box([x,y,z],[s,h,s])
            il.box([x,1.27,z+.20],[.22,.49,.015])
            if detailed: tree(go,plane(x,1.04,z+.22),.20,.44,True,3)
        return [[rx*math.cos(t),1.86,rz*math.sin(t)] for t in [a,b]]
    rail_start=checkpoint()
    pots=rail(gap_b,math.pi*.98,7)+rail(.10,gap_a,5)
    if variant>=2:
        for key,m in groups.items():
            m.positions[rail_start[key]:]=[[p[0],.60+(p[1]-.60)*1.35,p[2]] for p in m.positions[rail_start[key]:]]
        pots=[[p[0],.60+(p[1]-.60)*1.35,p[2]] for p in pots]
    mark('F01',['stone','trim']);mark('F02',['roof-dark','roof-a','roof-b','gold']);mark('F03',['roof-dark','roof-a','roof-b'])
    mark('F04',['trim','wood']);mark('F05',['wood','gold','stone']);mark('F06',['glass','trim','gold']);mark('F07',['glass','trim'])
    mark('F08',['glass','gallery']);mark('F09',['stone','inlay']);mark('F10',['trim','inlay']);mark('F11',['trim','inlay'])
    mark('F12',['roof-dark','glass','gold']);mark('F13',['foliage-dark','foliage-light','flowers-ivory','flowers-violet'])
    mark('F14',['trim','gold']);mark('F15',['trim']);mark('F16',['wood','gold']);mark('F17',['trim']);mark('F18',['inlay','gold'])
    mapping['F04'].append('e02-column');mapping['F15'].append('e02-column')
    if detailed:
        # Irregular paving with actual seams, plus green circular / winding inlays.
        for i in range(23):
            for j in range(17):
                x=-6+i*.54+(j%2)*.17;z=-3.5+j*.48
                if (x/patio_rx)**2+(z/patio_rz)**2> .98 or z<(-1.35 if variant>=2 else .55):continue
                points=[[x+.025+rng.random()*.06,.563,z+.025],[x+.51,.563,z+.04],
                        [x+.48,.563,z+.36],[x+.39,.563,z+.44],[x+.02,.563,z+.45]]
                tr.polygon(points)
        ellipse_ring(il,3.7,2.8,.575,thick=.025,count=55)
        ellipse_ring(go,3.78,2.86,.58,thick=.009,count=55)
        # Floor medallion lies on floor; its ellipse is generated only by the camera.
        center=[.7,.585,1.1] if variant>=2 else [-.1,.585,1.6]
        il.polygon([center]+[[center[0]+.75*math.cos(a),.585,center[2]+.75*math.sin(a)]
                            for a in [2*math.pi*i/32 for i in range(33)]])
        for i in range(10):
            a=2*math.pi*i/10
            points=bezier([0,0],[.28*math.cos(a+.3),.28*math.sin(a+.3)],
                          [.68*math.cos(a),.68*math.sin(a)],5)
            go.tube([[center[0]+p[0],.60,center[2]+p[1]] for p in points],.015)
        for sign in [-1,1]:
            points=bezier([-.2,1.6],[sign*2.1,2.2],[sign*2.8,.5],18)
            il.tube([[p[0],.58,p[1]] for p in points],.09)
        # Benches under porch and visible internal shelves behind the great window.
        wing_start=checkpoint()
        for x in [-3.6,-2.0]:
            wd.box([x,.98,1.18],[1.28,.13,.47]);wd.box([x,1.38,.97],[1.28,.62,.10])
            for dx in [-.5,.5]:
                for z in [1.0,1.35]:wd.box([x+dx,.76,z],[.08,.45,.08])
                wd.box([x+dx,1.12,1.18],[.07,.34,.48])
        wing_since(wing_start)
        for y in [1.55,2.12,2.68]:wd.box([3.45,y,front-.30],[2.0,.065,.35])
        # Extra planters near door, wall and window.
        pots += [wing_point(p) for p in [[-.55,.58,.85],[1.18,.58,.8],[-4.3,.58,1.2],[-2.7,1.07,1.15]]]+[
                 [2.65,1.22,front-.2],[3.8,1.22,front-.2]]
        def leafy(center,radius,count,flower=False):
            for i in range(max(1,int(count*.60))):
                a=rng.random()*2*math.pi;r=radius*math.sqrt(rng.random())
                p=add(center,[r*math.cos(a),rng.random()*radius*.85,r*math.sin(a)])
                target=fw if flower else (fl if rng.random()<.4 else fd)
                if flower and i%4==0:target=fv
                size=.034 if flower else (.045 if radius<.2 else .10)
                angle=rng.random()*math.pi
                along=[math.cos(angle)*size,0,math.sin(angle)*size]
                across=[-math.sin(angle)*size*.45,size*.4,math.cos(angle)*size*.45]
                target.polygon([sub(p,along),add(p,across),add(p,along),sub(p,across)])
        for p in pots:
            tr.lathe(p,[(0,.14),(.07,.16),(.28,.24),(.32,.26)],8)
            leafy(add(p,[0,.31,0]),.34,15)
            leafy(add(p,[0,.44,0]),.31,9,True)
        # Dense planting is localized along plinths and curved enclosure.
        for i in range(18):
            a=.15+math.pi*i/17
            center=[5.72*math.cos(a),.12,4.02*math.sin(a)]
            if gap_a-.02<a<gap_b+.02:continue
            leafy(center,.48,18);leafy(add(center,[0,.12,0]),.43,8,True)
        for z in [v+main_shift for v in [-3.1,-1.8,-.4,1.0]]:
            center=[5.72,.05,z]
            fd.lathe(center,[(0,.32),(.5,.35),(1.2,.19),(1.9,.035)],7)
            leafy(add(center,[0,.60,0]),.33,18)
            leafy([5.85,.22,z+.4],.44,10,True)
        # Vine climbs beside high front opening, not uniformly over every face.
        for i in range(70):
            y=.8+i*.075;x=1.76+.1*math.sin(i*.45)
            leafy([x,y,front+.16],.14,2)
        wing_start=checkpoint()
        for x in [-5.35,-3.0,-.5,1.10]:
            y=2.55;z=.68
            go.beam([x,y+.22,z-.13],[x,y+.22,z+.12],.025)
            la.box([x,y,z+.12],[.105,.24,.10])
            go.lathe([x,y-.16,z+.12],[(0,.08),(.06,.11),(.38,.09),(.45,0)],6)
        wing_since(wing_start)
    groups['gallery']=gallery
    groups={k:m for k,m in groups.items() if m.positions}
    for mesh in groups.values():mesh.weld()
    return groups,columns,mapping


def noise_material(seed, bump=.03):
    return {'pattern':{'kind':'noise','frequency':[3,5,3],'low':[.77,.78,.73],
                       'high':[1.12,1.08,1.02],'bump':bump,'warp':.3,'octaves':3,
                       'seed':seed,'contrast':.65}}


CAMERA={'position':[13.4,15.0,20.6],'target':[0,3.25,0],'fov_degrees':29}


def publish(engine, groups, columns, camera=CAMERA, clear=False):
    state=engine.command('inspect')
    if clear:
        edits=[{'op':'remove','id':e['id']} for e in state['scene']['entities'] if e['id']!='player']
        edits += [{'op':'upsert','entity':{'id':'player','name':'observer outside study',
                                         'transform':{'position':[40,1,40],'size':[.7,1.6,.7]},
                                         'color':[.5,.5,.5],'body':'player','visible':False}},
                  {'op':'upsert','entity':{'id':'backdrop','name':'neutral studio ground',
                                         'transform':{'position':[0,-.16,0],'size':[90,.20,90]},
                                         'color':[.48,.44,.38]}},
                  {'op':'upsert','entity':{'id':'background-wall','name':'neutral studio back wall',
                                         'transform':{'position':[0,16,-17],'size':[90,40,.1]},
                                         'color':[.54,.49,.42]}}]
        engine.command('edit_scene',edits=edits)
    edits=[]
    for i,(name,m) in enumerate(groups.items()):
        material=noise_material(23+i,.018 if name.startswith('roof') else .025)
        if name=='stone':
            segments=[{'start':[-12,.85+i*.37],'end':[12,.85+i*.37],'widths':[.012,.012]} for i in range(17)]
            segments += [{'start':[-5.8+i*.8,1.02+(i%4)*.37],
                          'end':[-5.8+i*.8,1.22+(i%4)*.37],'widths':[.010,.010]} for i in range(15)]
            material['detail']={'plane':'xy','tint':[.44,.40,.33],'opacity':.65,'bump':-.009,'segments':segments}
        if name=='glass':material={'transmission':.48,'reflectance':{'roughness':.20,'f0':.04}}
        if name=='gold':material={'reflectance':{'roughness':.40,'f0':.08}}
        asset={'id':m.name,'geometry':{'positions':m.positions,'triangles':m.triangles},
               'material':material,'shading':'flat'}
        edits.append({'op':'mesh','edit':{'kind':'put_asset','asset':asset}})
        for j,position in enumerate(getattr(m,'placements',[[0,0,0]])):
            edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':{
                'id':m.name+(f'-{j}' if hasattr(m,'placements') else ''),'asset':m.name,
                'position':position,'rotation':getattr(m,'rotation',[0,0,0,1]),'scale':1,'color':m.color}}})
    total_v=sum(len(m.positions) for m in groups.values())
    total_t=sum(len(m.triangles) for m in groups.values())
    if total_v>16200 or total_t>32000:
        raise ValueError(f'Preflight exceeded conservative room for column: {total_v}v/{total_t}t')
    engine.command('edit_scene',edits=edits)
    # Native source and retained stations, shared by all five columns.
    definition={'id':'e02-column','profile':{'id':'column-profile','width':.48,'height':2.4},'depth':.48,
                'segments':[1,1,1],'sections':{'id':'column-stations','cross_section':'rectangle',
                'stations':[{'id':f's{i}','at':at,'width':w,'depth':w} for i,(at,w) in enumerate(
                    [(0,1.0),(.07,1.0),(.09,.83),(.13,.83),(.15,.56),(.82,.47),(.86,.68),(.91,.74),(1,1.0)])]}}
    engine.command('edit_plant',edit={'kind':'put','definition':definition,'detail_policy':'physical',
                                     'animation_translation_policy':'physical'})
    column_edits=[]
    for i,p in enumerate(columns):
        # Sectioned source is centered vertically in local coordinates.
        column_edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':{
            'id':f'e02-column-{i}','asset':'e02-column','position':[p[0],p[1]+1.20,p[2]],
            'rotation':[0,0,0,1],'scale':1,'color':COLORS['trim']}}})
    engine.command('edit_scene',edits=column_edits)
    engine.command('set_camera',camera=camera)
    engine.command('set_light',light={'direction':[-.6,-1,-.7],'ambient':.53,
                                     'shadow':{'center':[0,2,0],'extent':12,'resolution':2048,'bias':.0012}})
    return {'vertices_direct':total_v,'triangles_direct':total_t,'assets_direct':len(groups)}


def compare(engine, name):
    manifest=json.loads((ROOT/'evidence.json').read_text())
    regions=[]
    for item in manifest['regions']:
        x,y,w,h=item['box'];regions.append({'id':item['id'],'x':x,'y':y,'width':w,'height':h})
    result=engine.command('compare_images',reference_path='reference.png',candidate_path=f'{name}.png',
                          regions=regions[:16],difference_path='difference.png')
    if len(regions)>16:
        extra=engine.command('compare_images',reference_path='reference.png',candidate_path=f'{name}.png',regions=regions[16:])
        result['report']['regions']+=extra['report']['regions']
    result['region_batches']=[16,len(regions)-16]
    dump('comparison.json',result)
    return result


def render(stage='detail', variant=2, check=False):
    engine=Engine()
    try:
        dump('native/schema.json',{'result':engine.rpc('tools/list',{})})
        dump('native/initialize.json',engine.initialization)
        groups,columns,mapping=build_house(stage,variant)
        counts=publish(engine,groups,columns,clear=True)
        print(json.dumps({'preflight':counts}),flush=True)
        result=engine.command('capture',path='render.png',width=1402,height=1122)
        dump('capture.json',result)
        metrics=compare(engine,'render')
        engine.command('save_scene',path='native/house.scene.json')
        dump('mapping.json',mapping)
        actual=engine.command('inspect_meshes')
        dump('counts.json',actual)
        if check:
            checks=verify(engine,groups,mapping)
            dump('verification.json',checks)
        dump('parameters.json',{'stage':stage,'variant':variant,'camera':CAMERA,
                                'columns':columns,'direct_counts':counts,'source':SOURCE,
                                'scope':'client-authored geometry, public MCP publication; hidden structure hypothetical'})
        print(json.dumps({'capture':result,'global_metrics':metrics['report']['metrics']}),flush=True)
    finally:
        engine.close()
    if check:
        second=Engine()
        try:
            second.command('load_scene',path='native/house.scene.json')
            second.command('capture',path='native/fresh-process.png',width=1402,height=1122)
            same=hash_file('native/fresh-process.png')==hash_file('render.png')
            checks['fresh_process_load_pixels_equal']=same
            if not same:raise AssertionError('Fresh-process pixels differ')
            dump('verification.json',checks)
        finally:second.close()


def hash_file(name):
    return hashlib.sha256((ROOT/name).read_bytes()).hexdigest()


def verify(engine,groups,mapping):
    expected=hash_file('render.png')
    result={'scope':'static authored scene; no gameplay or manual fidelity acceptance',
            'render_sha256':expected}
    engine.command('capture',path='native/repeated.png',width=1402,height=1122)
    result['repeated_capture_pixels_equal']=hash_file('native/repeated.png')==expected
    # Remove appearance in one authoring transaction, then prove undo restores it.
    scene_meshes=engine.command('inspect_meshes')
    edits=[{'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':{}}}
           for a in scene_meshes['assets']]
    for item in scene_meshes['instances']:
        instance={key:item[key] for key in ['id','asset','position','rotation','scale','color']}
        instance['color']=[.62,.60,.55]
        edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':instance}})
    engine.command('edit_scene',edits=edits)
    engine.command('capture',path='clay.png',width=1402,height=1122)
    result['appearance_edit_changes_pixels']=hash_file('clay.png')!=expected
    engine.command('undo')
    engine.command('capture',path='native/undo.png',width=1402,height=1122)
    result['material_undo_pixels_equal']=hash_file('native/undo.png')==expected
    engine.command('save_replay',path='native/house.replay.json')
    replay=engine.command('run_replay',path='native/house.replay.json')
    result['static_replay']=replay
    engine.command('capture_replay',replay_path='native/house.replay.json',path='native/replay.png',
                   width=1402,height=1122,tick=0)
    result['static_replay_pixels_equal']=hash_file('native/replay.png')==expected
    engine.command('load_scene',path='native/house.scene.json')
    engine.command('capture',path='native/reloaded.png',width=1402,height=1122)
    result['save_load_pixels_equal']=hash_file('native/reloaded.png')==expected
    ids={a['id'] for a in scene_meshes['assets']}
    result['all_evidence_asset_ids_resolve']=all(set(v)<=ids for v in mapping.values())
    result['stored_vertices']=sum(a['vertices'] for a in scene_meshes['assets'])
    result['stored_triangles']=sum(a['triangles'] for a in scene_meshes['assets'])
    result['instances']=len(scene_meshes['instances']);result['assets']=len(ids)
    for name,camera in [('spatial',{'position':[13,24,19],'target':[0,2.8,0],'fov_degrees':38}),
                        ('side',{'position':[24,12,7],'target':[0,3,0],'fov_degrees':33})]:
        engine.command('set_camera',camera=camera)
        engine.command('capture',path=f'{name}.png',width=1402,height=1122)
    engine.command('set_camera',camera=CAMERA)
    engine.command('capture',path='native/restored.png',width=1402,height=1122)
    result['auxiliary_camera_restore_pixels_equal']=hash_file('native/restored.png')==expected
    for key,value in result.items():
        if key.endswith('equal') or key in ['appearance_edit_changes_pixels','all_evidence_asset_ids_resolve']:
            if value is not True:raise AssertionError(f'Verification failed: {key}')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--inspect', action='store_true')
    parser.add_argument('--binary',default=BINARY)
    parser.add_argument('--stage',choices=['mass','detail'],default='detail')
    parser.add_argument('--variant',type=int,default=2)
    parser.add_argument('--check',action='store_true')
    args = parser.parse_args()
    BINARY=args.binary
    if args.inspect:
        engine = Engine()
        try:
            state = engine.command('inspect'); meshes = engine.command('inspect_meshes')
            dump('native/initial-state.json', state); dump('native/limits.json', meshes)
            print(json.dumps({'state': state, 'meshes': meshes}))
        finally: engine.close()
    else:
        render(args.stage,args.variant,args.check)
