#!/usr/bin/env python3
"""Prepare canonical organ sources with the mathematical frame integrator.

The files contain curves and sections; native Rust alone compiles/edit/renders them.
No triangles, runtime dependency or species-specific renderer is exported.
"""
import math,argparse,json
from dataclasses import dataclass
from pathlib import Path
import botany_math as b

ROOT=Path(__file__).resolve().parents[1]
class Builder:
    def __init__(self,name,seed=42):self.name=name;self.seed=seed;self.organs=[];self.by_id={};self.samples={};self.axis_samples=18
    def sample(self,id,u):
        rows=self.samples[id];x=u*(len(rows)-1);i=min(int(x),len(rows)-2);f=x-i;a,c=rows[i:i+2]
        t=b.unit(b.lerp(a.t,c.t,f));return b.Sample(b.lerp(a.p,c.p,f),t,b.normal_to(t,b.lerp(a.n,c.n,f)))
    def organ(self,id,parent,u,direction,length,shape,*,position=(0.,0.,0.),bend=0.,target=b.UP,attraction=0.,normal=None,birth=0.,young=7e8):
        start=self.sample(parent,u).p if parent else position
        direction=b.unit(direction);normal=b.normal_to(direction,normal or (0.,1.,0.))
        axis=shape['kind']=='axis';radius=shape.get('radius',.001)
        growth=b.Growth(bend_n=bend if isinstance(bend,b.Field) else b.Field(bend),attraction=b.Field(attraction),direction=target)
        rows=b.path(length,b.Sample(start,direction,normal),growth,dict(u=0.,height=start[1],radius=radius,level=0,parent_length=length),self.seed,id,b.Limits(segments=32),b.Budget(10000),None,radius)
        self.samples[id]=rows
        self.organs.append(dict(id=id,parent=parent,insertion=u,path=[r.p for r in rows[::2 if axis else 8]],normal=normal,shape=shape,birth=birth,duration=3. if axis else .8,density=650. if axis else 850.,young=young if axis else 2e7,shear=young*.2 if axis else 8e6))
        self.by_id[id]=self.organs[-1]
        return id
    def axis(self,id,parent,u,direction,length,radius,*,tip=.38,**kw):return self.organ(id,parent,u,direction,length,dict(kind='axis',radius=radius,tip_ratio=tip,ellipticity=1.),**kw)
    def leaf(self,id,parent,u,direction,length,width,*,palm=False,**kw):
        outline=dict(kind='palmate',tips=5,spread=1.65,depth=.50,rounding=.65,valley_rounding=1.8,lengths=[.64,.91,1.,.91,.64]) if palm else dict(kind='strip',base_power=.45,tip_power=1.1)
        return self.organ(id,parent,u,direction,length,dict(kind='blade',width=width,thickness=.00025,fold=.10,cup=.12,outline=outline),**kw)
    def recipe(self,prefix,**settings):return dict(version=2,prefix=prefix,source=dict(version=2,name=self.name,seed=self.seed,organs=self.organs),settings=dict(axis_samples=self.axis_samples,sides=8,**settings))

def radial(angle,h):return (math.cos(angle),h,math.sin(angle))
def palm(seed=42):
    x=Builder('Palmeirinha · touceira pinada',seed)
    for stem in range(12):
        a=stem*2.4;h=.55+1.45*b.uniform(x.seed,str(stem),'cane_height')**.85
        root=x.axis(f'caule{stem}',None,0.,radial(a,4.5),h,.036,position=((.22+.025*math.sqrt(stem))*math.cos(a),0.,(.22+.025*math.sqrt(stem))*math.sin(a)),bend=-.18,normal=b.UP,young=3e8)
        for j in range(7):
            az=j*2.4+stem+.25*b.uniform(x.seed,f'{root}/{j}','frond_azimuth');u=.95-.045*j
            length=2.0*(.84+.30*b.uniform(x.seed,f'{root}/{j}','frond_length'))
            # Inner fronds rise; outer fronds open and recurve at the tip.
            elevation=math.radians(83.-2.*j)
            turn=math.radians(100.+6.*j+6.*b.uniform(x.seed,f'{root}/{j}','frond_arch'))
            curvature=b.Field(-turn/length,drivers=(('u',b.Curve(((0.,1.45),(1.,.55)))),))
            frond=x.axis(f'{root}/raquis{j}',root,u,radial(az,math.tan(elevation)),length,.0045,bend=curvature,normal=b.UP,young=8e7)
            x.organs[-1]['connection']='sheath'
            for k in range(24):
                v=.28+.69*k/23;frame=x.sample(frond,v)
                long=.50*b.LEAFLET_LENGTH_PROFILE.at(v);width=.045*b.LEAFLET_WIDTH_PROFILE.at(v)
                for side in [-1,1]:
                    lateral=b.mul(frame.b,side);direction=b.add(b.add(b.mul(lateral,.92),b.mul(frame.t,.26)),(0.,.17,0.))
                    x.leaf(f'{frond}/foliolo{k}_{side}',frond,v,direction,long,width,bend=-.15,normal=frame.t,attraction=1.5,target=(0,-1,0),birth=1.+v)
    return chronological(x.recipe('palmeirinha',bark_color=[.33,.36,.10],leaf_color=[.075,.22,.015]),'palmeirinha')

def spline(points,u):
    x=u*(len(points)-1);i=min(int(x),len(points)-2);t=x-i;a,c=points[i:i+2]
    m0=b.sub(c,a) if i==0 else b.mul(b.sub(c,points[i-1]),.5)
    m1=b.sub(c,a) if i+2==len(points) else b.mul(b.sub(points[i+2],a),.5)
    p=b.add(b.add(b.mul(a,2*t**3-3*t*t+1),b.mul(m0,t**3-2*t*t+t)),b.add(b.mul(c,-2*t**3+3*t*t),b.mul(m1,t**3-t*t)))
    d=b.add(b.add(b.mul(a,6*t*t-6*t),b.mul(m0,3*t*t-4*t+1)),b.add(b.mul(c,-6*t*t+6*t),b.mul(m1,3*t*t-2*t)))
    return p,b.unit(d)

def curved(x,id,parent,u,end,direction,radius,*,end_direction=None,position=(0.,0.,0.),arch=.2,noise=.05,tip=.07,taper=1.3,young=3e9,handle=.37):
    radius=max(.0012,radius);tip=max(tip,.0011/radius)
    start=x.sample(parent,u).p if parent else position;delta=b.sub(end,start);length=b.norm(delta);t=b.unit(direction);last=b.unit(end_direction or delta)
    c1=b.add(start,b.mul(t,min(length*.37,max(length*handle,3*radius))));c2=b.sub(end,b.mul(last,min(length*.32,max(length*min(.32,handle),3*radius*tip))));points=[]
    phase=math.tau*b.uniform(x.seed,id,'irregularity')
    n=b.normal_to(t,(0.,1.,0.));side=b.cross(t,n)
    for k in range(17):
        q=k/16;v=1-q
        p=b.add(b.add(b.mul(start,v**3),b.mul(c1,3*v*v*q)),b.add(b.mul(c2,3*v*q*q),b.mul(end,q**3)))
        envelope=math.sin(math.pi*q)**2
        p=b.add(p,b.add((0.,arch*envelope,0.),b.mul(side,noise*envelope*math.sin(math.tau*q+phase))))
        points.append(p)
    old=None;rows=[]
    for k in range(x.axis_samples+1):
        p,t=spline(points,k/x.axis_samples);n=b.normal_to(t,(0,1,0)) if old is None else b.transport(old.n,old.t,t)
        old=b.Sample(p,t,n);rows.append(old)
    x.samples[id]=rows
    x.organs.append(dict(id=id,parent=parent,insertion=u,path=points,normal=rows[0].n,shape=dict(kind='axis',radius=radius,tip_ratio=tip,ellipticity=1.),taper_power=taper,birth=0.,duration=3.,density=650.,young=young,shear=young*.22))
    x.by_id[id]=x.organs[-1]
    return id

@dataclass(frozen=True)
class Crown:
    height:float;spread:float;first_fork:float;crown_height:float;shoots:int;leaves:int
    inner_sites:int;uplift:float
    radius:float;leaf_length:float;leaf_width:float;droop:float;leaf_type:str
    bark:tuple;green:tuple
    attractors:int=64
    pipe_exponent:float=2.3
    fork_tip:float=.82
    fork_forward:float=.60
    tip_segment_ratio:float=10.
    blade_samples:int=12
    tiers:int=0
    tier_thickness:float=.10
    early_vertical:float=1.
    trunk_lean:tuple=(.08,-.05)
    min_segment_ratio:float=10.
    shoot_min:float=.65
    shoot_span:float=.55
    beard_groups:int=0
    beard_length:float=1.4
    age_years:float=50.
    curve_handle:float=.37
    forward_min:float=.18
    root_fork_forward:float|None=None
    shoot_vertical_scale:float=1.
    beard_strands:int=4
    beard_radius:float=.005
    beard_radius_span:float=.003
    shoot_orders:int=0
    shoot_length_ratio:float=.68
    shoot_fork_forward:float=.62
    structural_step:float=.40
    structural_decay:float=.70
    early_length_ratio:float=1.
    target_bias:float=.20
    crown_floor:float=.30
    reiteration_sites:int=0
    reiteration_orders:int=1
    reiteration_length:float=.9
    reiteration_min_spacing:float=12.

def pattern(freq,low,high,bump=.0005,kind='noise'):
    return dict(frequency=freq,low=low,high=high,bump=bump,warp=.6,octaves=3,seed=129,contrast=1.5,kind=kind)

CROWNS={
    'platano':Crown(height=4.,spread=3.3,first_fork=.28,crown_height=.48,
        shoots=3,leaves=8,inner_sites=16,uplift=.35,radius=.22,
        leaf_length=.39,leaf_width=.40,droop=.22,leaf_type='palmate',
        bark=(.42,.40,.31),green=(.11,.25,.035),attractors=96,shoot_orders=1,reiteration_sites=12),
    'eucalipto':Crown(height=5.8,spread=3.6,first_fork=.47,crown_height=.53,
        shoots=3,leaves=15,inner_sites=0,uplift=.55,radius=.21,
        leaf_length=.36,leaf_width=.09,droop=.70,leaf_type='lanceolate',
        bark=(.48,.46,.37),green=(.105,.20,.12),attractors=192,tip_segment_ratio=26.,blade_samples=6,shoot_orders=1,crown_floor=.47,reiteration_sites=16,reiteration_length=1.2,structural_step=.48),
    'figueira_rs':Crown(height=2.8,spread=3.7,first_fork=.24,crown_height=.40,
        shoots=1,leaves=12,inner_sites=16,uplift=.02,radius=.31,
        leaf_length=.30,leaf_width=.18,droop=.22,leaf_type='ovate',
        bark=(.13,.12,.075),green=(.10,.16,.03),attractors=512,fork_tip=.88,shoot_orders=2,reiteration_sites=48,reiteration_orders=2,reiteration_min_spacing=9.,structural_step=.38,early_length_ratio=.60,
        tip_segment_ratio=10.,blade_samples=4,tiers=5,early_vertical=.32,
        trunk_lean=(.24,.04),min_segment_ratio=5.,fork_forward=.70,root_fork_forward=.32,
        shoot_min=.35,shoot_span=.40,beard_groups=120,age_years=100.,curve_handle=.18,forward_min=.18,shoot_vertical_scale=.20,
        beard_strands=8,beard_radius=.008,beard_radius_span=.004),
    'figueira':Crown(height=2.9,spread=2.8,first_fork=.30,crown_height=.40,
        shoots=2,leaves=20,inner_sites=32,uplift=.12,radius=.24,
        leaf_length=.31,leaf_width=.20,droop=1.2,leaf_type='ovate',
        bark=(.30,.31,.23),green=(.055,.18,.028),attractors=384,tip_segment_ratio=26.,blade_samples=4,shoot_orders=1,reiteration_sites=32),
}

def radius_at(x,id,u):
    o=x.by_id[id];shape=o['shape']
    return shape['radius']*(shape['tip_ratio']+(1-shape['tip_ratio'])*(1-u)**o.get('taper_power',1.))

def diverge(direction,t):
    d=b.unit(direction);c=max(-.25,min(.55,b.dot(d,t)));radial=b.normal_to(t,d)
    return b.add(b.mul(t,c),b.mul(radial,math.sqrt(1-c*c)))

def mean(points):
    return tuple(sum(p[k] for p in points)/len(points) for k in range(3))

def partition(points,fraction=.5):
    # Spatial median, with a seed-stable oblique plane to avoid planar fans.
    spread=[max(p[k] for p in points)-min(p[k] for p in points) for k in range(3)]
    k=max(range(3),key=lambda k:spread[k]);order=sorted(points,key=lambda p:p[k]+.17*p[(k+1)%3])
    split=max(1,min(len(order)-1,int(len(order)*fraction)))
    return order[:split],order[split:]

def tree(kind,seed=129):
    cfg=CROWNS[kind];x=Builder({'platano':'Plátano · forquilhas e copa arredondada','eucalipto':'Eucalipto · fustes ascendentes e copa aberta','figueira':'Figueira ornamental · copa ampla e raízes aparentes','figueira_rs':'Figueira campestre · referência RS'}[kind],seed)
    if not all(0<=v<=3 for v in (cfg.shoot_orders,cfg.reiteration_orders)):raise ValueError('Shoot orders must be in0..3')
    # Attractors describe a volume, never a traced photo or a stored mesh.
    targets=[]
    for i in range(cfg.attractors):
        y=1.-2.*(i+.5)/cfg.attractors;a=i*2.399963;disk=math.sqrt(1-y*y)
        jitter=.78+.22*b.uniform(x.seed,str(i),'crown_shell')
        if cfg.tiers:
            layer=i%cfg.tiers;v=layer/max(1,cfg.tiers-1)
            disk=(1.-.38*v)*(.87+.13*math.cos(3*a+layer*.7))
            y=.50+.80*v+cfg.tier_thickness*(b.uniform(x.seed,str(i),'tier_height')-.5)
            targets.append((cfg.spread*disk*math.cos(a)*jitter,cfg.height*y,cfg.spread*disk*math.sin(a)*jitter))
        else:
            targets.append((cfg.spread*disk*math.cos(a)*jitter,cfg.height*(.96+cfg.crown_height*y),cfg.spread*disk*math.sin(a)*jitter))
    first=(cfg.trunk_lean[0],cfg.height*cfg.first_fork,cfg.trunk_lean[1])
    exponent=cfg.pipe_exponent;terminal=[];interior=[];limbs=[];scaffold=[]
    def grow(id,parent,points,start,end,radius,depth,takeoff=None):
        direction=b.sub(end,start)
        if takeoff is not None:direction=takeoff
        # Larger structural axes keep most of their section up to the fork.
        tip=cfg.fork_tip if len(points)>1 else .24
        axis=curved(x,id,parent,1. if parent else 0.,end,direction,radius,position=start,tip=tip,taper=1.,arch=min(.08 if depth else .02,.07*b.norm(b.sub(end,start))),noise=.020*min(1.,b.norm(b.sub(end,start))),end_direction=b.sub(end,start),handle=cfg.curve_handle if depth<4 else .37)
        if parent:x.organs[-1]['connection']='fork'
        if depth in (2,3):limbs.append(axis)
        if 1<=depth<=6:scaffold.append((axis,depth))
        if 1<len(points)<=4:interior.append(axis)
        if len(points)==1:
            terminal.append(axis);return
        groups=partition(points,.38+.24*b.uniform(x.seed,axis,'fork_balance'))
        incoming=x.sample(axis,1.).t
        splitting=b.normal_to(incoming,b.sub(mean(groups[1]),mean(groups[0])))
        for j,group in enumerate(groups):
            target=mean(group)
            incoming=x.sample(axis,1.).t
            child_radius=radius*tip*(len(group)/len(points))**(1/exponent)
            forward=cfg.root_fork_forward if depth==0 and cfg.root_fork_forward is not None else cfg.fork_forward
            takeoff=b.unit(b.add(b.add(b.mul(incoming,forward),b.mul(splitting,math.sqrt(1-forward**2)*(1 if j else -1))),(0,cfg.uplift if depth<4 else cfg.uplift*.3,0)))
            # A target biases the heading; it is not an endpoint to return to.
            # Decreasing internodes fill the crown without stretching late forks
            # past close targets and making hairpin loops.
            takeoff=b.unit(b.lerp(takeoff,b.unit(b.sub(target,end)),cfg.target_bias))
            if depth<2:takeoff=b.unit((takeoff[0],takeoff[1]*cfg.early_vertical,takeoff[2]))
            if b.dot(takeoff,incoming)<cfg.forward_min:
                side=b.normal_to(incoming,takeoff)
                takeoff=b.add(b.mul(incoming,cfg.forward_min),b.mul(side,math.sqrt(1-cfg.forward_min**2)))
            nominal=cfg.spread*cfg.structural_step*cfg.structural_decay**depth
            if depth<2:nominal*=cfg.early_length_ratio
            nominal*=.90+.20*b.uniform(x.seed,f'{axis}/{j}','internode_length')
            length=max(nominal,child_radius*(cfg.tip_segment_ratio if depth>=4 else cfg.min_segment_ratio))
            floor=(cfg.height*cfg.crown_floor-end[1])/length
            if takeoff[1]<floor:
                y=min(.95,floor);horizontal=b.normal_to(b.UP,takeoff)
                takeoff=b.add(b.mul(horizontal,math.sqrt(1-y*y)),(0,y,0))
            finish=b.add(end,b.mul(takeoff,length))
            grow(f'{axis}/f{j}',axis,group,end,finish,child_radius,depth+1,takeoff)
    grow('tronco0',None,targets,(0.,0.,0.),first,cfg.radius,0)
    # Leaf-bearing shoots belong to the outer axes. Their spread, droop and
    # phyllotaxis are data; the native compiler knows no species names.
    shoot_count=cfg.shoots;leaf_count=cfg.leaves;shoot_hosts=[]
    def foliate(shoot,leaves,phase):
        shoot_hosts.append(shoot)
        for leaf in range(leaves):
            u=.10+.85*leaf/(leaves-1);frame=x.sample(shoot,u);az=leaf*2.39996+phase
            outward=b.add(b.add(b.mul(frame.n,math.cos(az)),b.mul(frame.b,math.sin(az))),b.mul(frame.t,.35))
            outward=b.add(outward,(0,-cfg.droop,0));factor=.72+.35*b.uniform(x.seed,f'{shoot}/{leaf}','leaf_size')
            normal=b.cross(b.UP,outward);normal=b.rotate(normal,b.mul(b.unit(outward),.65*math.sin(leaf*5.3)))
            x.leaf(f'{shoot}/folha{leaf}',shoot,u,outward,cfg.leaf_length*factor,cfg.leaf_width*factor,palm=cfg.leaf_type=='palmate',normal=normal,attraction=.4 if kind=='platano' else 1.4,target=(0,-1,0),birth=1.+u)
            blade=x.organs[-1]['shape'];blade.update(fold=.035,cup=.055)
            if cfg.leaf_type=='palmate':blade['outline'].update(depth=.30,rounding=1.05)
            elif cfg.leaf_type=='ovate':blade['outline'].update(base_power=.65,tip_power=.85)
    def branchlet(id,parent,u,direction,length,radius,orders,phase):
        f=x.sample(parent,u);end=b.add(f.p,b.mul(b.unit(direction),length));end=b.add(end,(0,-cfg.droop*.18,0))
        axis=curved(x,id,parent,u,end,direction,radius,arch=min(.06,length*.08),noise=min(.025,length*.04),end_direction=b.add(direction,(0,-cfg.droop,0)),tip=.90 if orders else .16,young=6e8)
        if u==1.:x.by_id[axis]['connection']='fork'
        if not orders:return [axis]
        f=x.sample(axis,1.);az=phase+math.tau*b.uniform(x.seed,axis,'twig_split')
        lateral=b.add(b.mul(f.n,math.cos(az)),b.mul(f.b,math.sin(az)))
        children=[]
        for j in range(2):
            d=b.add(b.mul(f.t,cfg.shoot_fork_forward),b.mul(lateral,math.sqrt(1-cfg.shoot_fork_forward**2)*(1 if j else -1)))
            child_radius=radius_at(x,axis,1.)*(.5**(1/exponent))
            scale=cfg.shoot_length_ratio*(.85+.30*b.uniform(x.seed,f'{axis}/{j}','twig_length'))
            children.extend(branchlet(f'{axis}/ramo{j}',axis,1.,d,length*scale,child_radius,orders-1,az+j*2.4))
        return children
    # Reiterations occupy the interior rather than only the outermost tips.
    candidates=[]
    occupied=set(interior)
    if kind=='figueira':occupied.update(limbs) # aerial-root collars already occupy these axes
    for axis,depth in scaffold:
        if axis in occupied:continue
        rows=x.samples[axis];length=sum(b.norm(b.sub(c.p,a.p)) for a,c in zip(rows,rows[1:]))
        if length/x.by_id[axis]['shape']['radius']>=cfg.reiteration_min_spacing:candidates.append((axis,depth))
    candidates.sort(key=lambda row:(row[1],b.uniform(x.seed,row[0],'reiteration_site')))
    for axis,depth in candidates[:cfg.reiteration_sites]:
        u=.50;f=x.sample(axis,u);phase=math.tau*b.uniform(x.seed,axis,'reiteration_phase')
        inward=b.add(b.mul(f.p,-.30),(0,cfg.height*.65,0))
        direction=diverge(b.add(inward,b.mul(f.b,.25*math.sin(phase))),f.t)
        length=cfg.reiteration_length*(.85+.30*b.uniform(x.seed,axis,'reiteration_length'))
        hosts=branchlet(f'{axis}/broto_interior',axis,u,direction,length,min(.045,radius_at(x,axis,u)*.40),cfg.reiteration_orders,phase)
        for shoot in hosts:foliate(shoot,leaf_count,phase)
    sites=terminal+sorted(interior,key=lambda id:b.uniform(x.seed,id,'foliage_site'))[:cfg.inner_sites]
    for branch in sites:
        inner=branch in interior
        section=x.by_id[branch]['shape']['radius']
        rows=x.samples[branch];length=sum(b.norm(b.sub(c.p,a.p)) for a,c in zip(rows,rows[1:]))
        count=(2 if length/section>22 else 1) if inner else shoot_count
        leaves=leaf_count*2 if inner else leaf_count
        for k in range(count):
            q=(.53 if count==1 else .42+.28*k) if inner else (.62 if count==1 else .26+.65*k/(count-1));f=x.sample(branch,q)
            a=k*2.39996+math.tau*b.uniform(x.seed,branch,'shoot_phase')
            direction=diverge(b.add(b.add(b.mul(f.n,math.cos(a)),b.mul(f.b,math.sin(a))),b.mul(f.t,.5)),f.t)
            direction=b.add(direction,(0,.25 if kind=='platano' else -.25*cfg.droop,0))
            if cfg.shoot_vertical_scale!=1.:direction=diverge((direction[0],direction[1]*cfg.shoot_vertical_scale,direction[2]),f.t)
            length=cfg.shoot_min+cfg.shoot_span*b.uniform(x.seed,f'{branch}/{k}','shoot_length')
            hosts=branchlet(f'{branch}/broto{k}',branch,q,direction,length,min(.0045,radius_at(x,branch,q)*.55),cfg.shoot_orders,a)
            for shoot in hosts:foliate(shoot,leaves,a)
    if cfg.beard_groups:
        hosts=sorted(shoot_hosts,key=lambda id:b.uniform(x.seed,id,'beard_host'))[:cfg.beard_groups]
        for group,host in enumerate(hosts):
            cluster=.42+.35*b.uniform(x.seed,host,'beard_cluster')
            for strand in range(cfg.beard_strands):
                u=cluster+.08*(b.uniform(x.seed,f'{host}/{strand}','beard_site')-.5);f=x.sample(host,u)
                key=f'{host}/epifita{strand}';length=cfg.beard_length*(.55+.75*b.uniform(x.seed,key,'length'))
                sway=.08+.14*b.uniform(x.seed,key,'sway');az=group*2.4+strand
                end=b.add(f.p,(sway*math.cos(az),-length,sway*math.sin(az)))
                beard=curved(x,key,host,u,end,(.12*math.cos(az),-1.,.12*math.sin(az)),cfg.beard_radius+cfg.beard_radius_span*b.uniform(x.seed,key,'radius'),arch=0.,noise=.025,tip=.18,young=6e5)
                x.organs[-1].update(connection='sheath',density=50.)
                for k,v in enumerate([.36,.68]):
                    f=x.sample(beard,v);side=.07+.10*b.uniform(x.seed,f'{key}/{k}','fray')
                    end=b.add(f.p,(side*math.cos(az+k),-length*.28,side*math.sin(az+k)))
                    curved(x,f'{key}/fio{k}',beard,v,end,(.3*math.cos(az+k),-1.,.3*math.sin(az+k)),.0025,arch=0.,noise=.008,tip=.45,young=4e5)
                    x.organs[-1].update(connection='sheath',density=50.)
    if kind=='figueira':
        for i,limb in enumerate(limbs):
            u=.48;f=x.sample(limb,u)
            curved(x,f'{limb}/raiz',limb,u,(f.p[0]+.10,0.,f.p[2]-.08),diverge((.10,-1,0),f.t),min(.060,radius_at(x,limb,u)*.70),arch=0.,noise=.025,end_direction=(0,-1,0),tip=.85,taper=.8)
    bark=pattern([2.,.65,2.],[.64,.68,.58],[1.5,1.42,1.22],.0008,'cellular') if kind=='platano' else pattern([5.,.45,5.],[.75,.78,.72],[1.18,1.15,1.05],.001)
    recipe=chronological(x.recipe(kind,bark_color=cfg.bark,leaf_color=cfg.green,bark_pattern=bark,leaf_pattern=pattern([7,7,7],[.90,.94,.86],[1.10,1.06,1.03],.00002)),kind)
    recipe['version']=recipe['source']['version']=4
    if cfg.blade_samples!=12:recipe['settings']['blade_samples']=cfg.blade_samples
    recipe['settings']['age_years']=cfg.age_years
    # Preserve the authored area allocation through structural forks. Radius
    # increases with age; terminal shoots remain slender by their source size.
    recipe['source']['development'].update(branch_thickening_decay=1.,branch_length_decay=1.)
    return recipe

def chronological(recipe,kind,age=None):
    """Potential sites are stable; Rust alone evaluates their age and geometry."""
    recipe['version']=recipe['source']['version']=3
    profile=dict(reference_age_years=8.,height_time_years=25.,crown_time_years=14.,radial_time_years=14.,branch_thickening_decay=.8,branch_length_decay=.85,basal_flare=.45,bark_gain=1.5,senescence_start_years=140.,senescence_time_years=45.,emergence_fraction=.65)
    if kind in ('figueira','figueira_rs'):profile.update(height_time_years=18.,crown_time_years=24.,radial_time_years=14.,basal_flare=.65,senescence_start_years=180.)
    if kind=='figueira_rs':profile.update(basal_flare=2.,height_time_years=24.)
    if kind=='eucalipto':profile.update(height_time_years=18.,crown_time_years=12.,radial_time_years=12.,basal_flare=.25,senescence_start_years=100.)
    if kind=='palmeirinha':profile.update(model='palm',height_time_years=20.,crown_time_years=4.,palm_establishment_years=4.,basal_flare=0.,bark_gain=.15,senescence_start_years=65.)
    recipe['settings']['sides']=12
    recipe['source']['development']=profile;recipe['settings']['age_years']=float(age if age is not None else 30 if kind=='palmeirinha' else 50)
    births={}
    for o in recipe['source']['organs']:
        parent=o['parent'];key=o['id'];role='axis';duration=3.
        if parent is None:
            born=int(key.removeprefix('caule'))*.85 if kind=='palmeirinha' else 0.
        elif o.get('connection')=='fork':
            born=births[parent]+1.8;duration=2.
        elif o['shape']['kind']=='blade':
            born=births[parent]+(.2+.8*b.uniform(recipe['source']['seed'],key,'site_birth') if kind=='palmeirinha' else .5+3.*b.uniform(recipe['source']['seed'],key,'site_birth')**2)
            duration=1.
        elif key.endswith('/raiz'):
            born=births[parent]+9.+8.*b.uniform(recipe['source']['seed'],key,'root_birth');role='aerial_root';duration=8.
        elif '/epifita' in key:
            born=births[parent]+(8. if '/epifita' not in parent else .4);role='frond';duration=2.
        elif '/raquis' in key:
            born=births[parent]+1.5;role='frond';duration=2.
        else:
            born=births[parent]+1.0+o['insertion']*2.0
        births[key]=born;o['life']=dict(born_years=born,expansion_years=duration,role=role)
    return recipe

def main():
    dest=ROOT/'assets/botany';dest.mkdir(parents=True,exist_ok=True)
    parser=argparse.ArgumentParser();parser.add_argument('--only',choices=['palmeirinha',*CROWNS]);parser.add_argument('--age',type=float);parser.add_argument('--seed',type=int);parser.add_argument('--without-roots',action='store_true');parser.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');args=parser.parse_args()
    for name in ([args.only] if args.only else ['palmeirinha',*CROWNS]):
        r=palm(args.seed if args.seed is not None else 42) if name=='palmeirinha' else tree(name,args.seed if args.seed is not None else 129)
        if args.age is not None:r['settings']['age_years']=args.age
        path=dest/(r['prefix']+'.ignis-botany');systems=[]
        if not args.without_roots and path.exists():systems=json.loads(path.read_bytes())['source'].get('root_systems',[])
        count=len(r['source']['organs'])
        if systems:
            from apply_botany_roots import generate
            count=generate(r,path,systems,args.binary)
        else:b.atomic_json(path,r)
        print(r['prefix'],count)
if __name__=='__main__':main()
