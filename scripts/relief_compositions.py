"""Cold authoring of editable Landform2 sources; no vertices/triangles sampled here."""
import copy
import math
import random

ZERO = {'cell': ['0', '0', '0'], 'offset': [0, 0, 0]}


def authored_precision(value):
    # Stable decimal controls at the authoring boundary, before any Rust geometry.
    if isinstance(value, float): return round(value, 9)
    if isinstance(value, list): return [authored_precision(v) for v in value]
    if isinstance(value, dict): return {k:authored_precision(v) for k,v in value.items()}
    return value


def position(x, y, z):
    cells = [math.floor(v/1024) for v in (x,y,z)]
    return {'cell': [str(c) for c in cells], 'offset': [v-c*1024 for v,c in zip((x,y,z),cells)]}


def section(at, scale, factors, offset=(0,0)):
    value = dict(at=at, scale=scale, offset=list(offset))
    if factors: value['radial_scales'] = factors
    return value


def footprint(n, rng, stretch=1):
    # Coherent angular structure: broad facets, not independent per-vertex jitter.
    phases = [rng.uniform(0, math.tau) for _ in range(3)]
    return [[math.copysign(abs(math.cos(a))**.55,math.cos(a))*stretch*(1+.05*math.sin(3*a+phases[0])),
             math.copysign(abs(math.sin(a))**.55,math.sin(a))*(1+.05*math.sin(2*a+phases[1]))]
            for a in (i*math.tau/n for i in range(n))]


def factors(n, rng):
    phase = rng.uniform(0, math.tau)
    return [1+.13*math.sin(i*math.tau/n*3+phase) for i in range(n)]


def detailed_sections(rows, n, phase):
    """Author coherent recesses along the walls, keeping all original ledges/endpoints."""
    result=[]
    for a,b in zip(rows,rows[1:]):
        result.append(a)
        if b['at']==a['at'] or b['scale']==0: continue
        for t in (1/3,2/3):
            # A fixed angular fault family persists through neighbouring heights.
            window=math.sin(math.pi*t)**2
            aa=a.get('radial_scales',[1]*n);bb=b.get('radial_scales',[1]*n)
            radii=[(aa[j]*(1-t)+bb[j]*t)*(1-.12*window*math.sin(j*math.tau/n*4+phase+.25*t)**4)
                   for j in range(n)]
            result.append(section(a['at']*(1-t)+b['at']*t,a['scale']*(1-t)+b['scale']*t,radii,
                                  [a['offset'][j]*(1-t)+b['offset'][j]*t for j in (0,1)]))
    return result+[rows[-1]]


def piece(name, anchor, profile, rows, angle=0):
    return dict(id=name, anchor=anchor, axis=[0,1,0],
                profile_x=[math.cos(angle),0,math.sin(angle)],
                profile=profile, sections=rows, inner_scale=None)


def cover(p):
    """A thin diagnostic turf volume authored with the same native sections."""
    last = p['sections'][-1]
    if last['scale'] == 0:
        return None
    q = copy.deepcopy(p); q['id'] += '_cap'
    x, _, z = p['profile_x']; a, b = last['offset']
    q['anchor'] = [p['anchor'][0]+x*a+z*b, p['anchor'][1]+last['at']+.015,
                   p['anchor'][2]+z*a-x*b]
    q['sections'] = [section(0,last['scale'],last['radial_scales']),
                     section(.18,last['scale']*.998,last['radial_scales'])]
    return q


def documents(recipe, family, sphere=False):
    salts = {'terraces':1, 'towers':2, 'massif':3}
    if family not in salts:
        raise ValueError('Unknown relief composition')
    rng = random.Random(recipe['seed']+salts[family]*7919)
    pieces = []
    if family == 'terraces':
        cfg=recipe[family]
        if not 1<=cfg['count']<=16: raise ValueError('Terrace count needs1..16')
        for i in range(cfg['count']):
            x=(i%3-1)*75; z=(i//3-1)*65
            r=cfg['radius_meters']*rng.uniform(.85,1.2)
            h=cfg['height_meters']*(1+i//3)*rng.uniform(.8,1.15)
            n=24; a=factors(n,rng); b=factors(n,rng)
            rows=[section(0,r*1.12,a), section(h*.18,r*.95,b),
                  section(h*.18,r*1.02,b), section(h*.88,r*.99,a),
                  section(h,r,b)]
            pieces.append(piece(f'terrace_{i}',[x,0,z],footprint(n,rng,1.05),detailed_sections(rows,n,rng.uniform(0,math.tau)),rng.uniform(-.4,.4)))
        # Tilted slab: arbitrary native cross-profile, then variable longitudinal sections.
        blade=piece('inclined_slab',[3,38,35],
            [[-30,-4],[28,-4],[22,10],[-32,28],[-36,23]],
            [section(-47,1,[1]*5),section(-5,1,[.95,1.1,1,.9,1],(2,-3)),
             section(42,.75,[1,.8,1.15,.8,1],(-4,2))])
        blade['axis']=[0,0,1];blade['profile_x']=[1,0,0]
        pieces.append(blade)
    elif family == 'towers':
        cfg=recipe[family]
        if not 1<=cfg['count']<=24: raise ValueError('Tower count needs1..24')
        for i in range(cfg['count']):
            angle=i*2.399963229728653
            distance=25+20*math.sqrt(i)
            n=24; r=cfg['radius_meters']*rng.uniform(.65,1.2)
            h=cfg['height_meters']*rng.uniform(.5,1.5)
            a=factors(n,rng); b=factors(n,rng)
            shelf=rng.uniform(.38,.72)
            rows=[section(0,r*1.3,a),section(h*.17,r,a),section(h*shelf,r*.85,b,(2,-1))]
            if i%3==0:rows.append(section(h*shelf,r*.98,b,(2,-1)))
            rows.extend([section(h*.9,r*.72,a,(3,-2)),section(h,r*rng.uniform(.45,.8),b,(3,-2))])
            pieces.append(piece(f'tower_{i}',[math.cos(angle)*distance,-12,math.sin(angle)*distance],
                                footprint(n,rng),detailed_sections(rows,n,rng.uniform(0,math.tau)),rng.uniform(0,math.tau)))
        pieces.append(piece('connected_plateau',[100,-12,-70],footprint(20,rng,1.3),
            [section(0,65,factors(20,rng)),section(15,58,factors(20,rng)),section(47,54,factors(20,rng))],.2))
    else:
        cfg=recipe[family];h=cfg['height_meters'];r=cfg['radius_meters']
        if not 1<=cfg['shoulders']<=10 or not 0<=cfg['blocks']<=20 or 1+cfg['shoulders']+cfg['blocks']>32:
            raise ValueError('Massif components exceed the native piece budget')
        for i in range(1+cfg['shoulders']):
            a=i*math.tau/cfg['shoulders']+.3
            centre=[0,0,0] if i==0 else [math.cos(a)*r*.85,0,math.sin(a)*r*.85]
            height=h if i==0 else h*rng.uniform(.38,.7)
            width=r if i==0 else r*rng.uniform(.28,.48)
            n=32; low=factors(n,rng); high=factors(n,rng)
            rows=[section(0,width*1.25,low), section(height*.18,width,high),
                  section(height*.4,width*.8,low,(width*.1,0)),
                  section(height*.66,width*.65,high,(width*.15,-width*.05)),
                  section(height*.85,width*.55,low,(width*.22,-width*.1)),
                  section(height,width*.18 if i%3==0 else 0,high if i%3==0 else [],(width*.28,-width*.13))]
            p=piece(f'crest_{i}',centre,footprint(n,rng,1.25),detailed_sections(rows,n,rng.uniform(0,math.tau)),a)
            if i:
                p['axis']=[-.4*math.cos(a),1,-.4*math.sin(a)]
                p['profile_x']=[-math.sin(a),0,math.cos(a)]
            pieces.append(p)
        # Dominant mass is a broad broken ridge with an independent longitudinal
        # silhouette. It is authored through the same profile/section controls.
        main=pieces[0]
        main.update(axis=[0,0,1],profile_x=[1,0,0],
            profile=[[x*r/100,y*h/170] for x,y in
                [[-100,-10],[95,-10],[78,45],[60,110],[10,175],[-30,150],[-45,90],[-105,30]]])
        main['sections']=detailed_sections([
            section(-.7*r,.8,[1,1,.9,.95,1.1,.9,1,1]),
            section(0,1,[1,1,1.1,.95,.9,1,1,1]),
            section(.7*r,.5,[1,.9,1.2,1,.9,1.1,.8,1],(.15*r,3*h/170))],8,.6)
        for i in range(cfg['blocks']):
            a=rng.uniform(0,math.tau);d=r*rng.uniform(1,1.75);w=rng.uniform(3,8)
            p=piece(f'talus_{i}',[math.cos(a)*d,0,math.sin(a)*d],footprint(5,rng),
                    [section(0,w,factors(5,rng)),section(w*1.1,w*.8,factors(5,rng),(w*.2,0)),
                     section(w*1.4,w*.4,factors(5,rng),(w*.3,0))],a)
            pieces.append(p)
    radius=recipe['sphere_radius_meters'];origin=position(0,radius,0) if sphere else copy.deepcopy(ZERO)
    def doc(name, elements):
        d=dict(version=2,space_version=1,id=f'{family}_{name}',revision=1,origin=origin,pieces=elements)
        if sphere:d['domain']=dict(kind='sphere',center=copy.deepcopy(ZERO),radius_meters=radius,forward=[0,0,1])
        return authored_precision(d)
    caps=[cover(p) for p in pieces if p['axis']==[0,1,0]]
    return doc('rock',pieces),doc('caps',[p for p in caps if p is not None])
