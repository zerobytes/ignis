#!/usr/bin/env python3
"""Branch-supported desert and clipped shrub profiles from seven user images.

Authored silhouettes, not species identifications or calibrated growth. See the
preconstruction inventory in docs/references/botany/reference-shrubs-analysis.md.
"""
import argparse
import json
import math
from pathlib import Path

from groundcover_assets import Plant,ROOT,TAU,add,atomic_json,curve,library as build_library,scale
from shrub_assets import unit

DEST=ROOT/'assets/botany/reference-shrubs'
FAMILIES=[
    dict(id='xerico_cinza',name='Moita desértica de galhos expostos · R1',reference=1,height=.85,reach=.64,
         stems=12,shoots=7,twigs=5,sites=4,occupancy=.25,length=.035,width=.005,leaf_start=.68,up=.65,
         green=[.19,.23,.12],accent=[.29,.31,.16],wood=[.27,.22,.145],lobes=1,broom=True,fine_forks=2,blade_samples=4),
    dict(id='xerico_baixo',name='Moita baixa verde-acinzentada · R1',reference=1,height=.42,reach=.79,
         stems=4,shoots=7,twigs=5,sites=6,occupancy=.90,length=.043,width=.011,leaf_start=.30,up=.35,
         green=[.20,.26,.105],accent=[.29,.33,.14],wood=[.29,.23,.14],lobes=3,fine_forks=1,blade_samples=4),
    dict(id='almofada_clara',name='Almofada lobada de cabeças claras · R2',reference=2,height=.67,reach=.94,
         stems=4,shoots=8,twigs=5,sites=4,occupancy=.55,length=.024,width=.0045,leaf_start=.56,up=.45,
         green=[.23,.28,.065],accent=[.51,.47,.15],wood=[.31,.25,.115],lobes=3,heads=True,fine_forks=3,blade_samples=4),
    dict(id='duna_oval',name='Arbusto de duna com folhas ovais · R3',reference=3,height=.95,reach=.76,
         stems=7,shoots=8,twigs=5,sites=5,occupancy=.94,length=.098,width=.033,leaf_start=.48,up=.65,
         green=[.065,.18,.025],accent=[.23,.34,.065],wood=[.24,.18,.105],lobes=1,cap_twigs=14,leaf_roll=1.3,blade_samples=6),
    dict(id='duna_agulha',name='Arbusto de duna com folhas estreitas · R4',reference=4,height=.72,reach=.60,
         stems=9,shoots=7,twigs=5,sites=5,occupancy=.94,length=.115,width=.0075,leaf_start=.30,up=1.6,
         green=[.085,.18,.065],accent=[.22,.30,.085],wood=[.20,.18,.11],lobes=1,leaf_roll=.35,blade_samples=4),
    dict(id='esfera_miuda',name='Topiaria esférica de folha miúda · R5',reference=5,height=1.10,reach=.55,
         stems=8,shoots=12,twigs=10,sites=6,occupancy=.98,length=.052,width=.027,leaf_start=.60,up=.3,
         green=[.035,.115,.018],accent=[.20,.31,.045],wood=[.16,.12,.065],lobes=1,clipped=True,round_leaf=True,cap_twigs=20,leaf_roll=1.5,blade_samples=4),
    dict(id='esfera_folhosa',name='Esfera folhosa irregular · R6',reference=6,height=1.06,reach=.66,
         stems=6,shoots=8,twigs=8,sites=5,occupancy=.98,length=.105,width=.052,leaf_start=.55,up=.45,
         green=[.055,.15,.025],accent=[.25,.37,.065],wood=[.18,.14,.07],lobes=1,cap_twigs=14,leaf_roll=1.4,blade_samples=6),
    dict(id='vertical_denso',name='Arbusto vertical de folhas pequenas · R7',reference=7,height=1.18,reach=.40,
         stems=10,shoots=10,twigs=7,sites=7,occupancy=.96,length=.046,width=.022,leaf_start=.50,up=1.0,
         green=[.07,.18,.028],accent=[.24,.35,.065],wood=[.19,.15,.08],lobes=1,vertical=True,round_leaf=True,cap_twigs=6,leaf_roll=1.4,blade_samples=4),
]


class ReferenceShrub(Plant):
    @staticmethod
    def at(path,u):
        x=max(0.,min(1.,u))*(len(path)-1);i=min(int(x),len(path)-2);t=x-i
        return [(1-t)*a+t*b for a,b in zip(path[i],path[i+1])]

    def wood(self,start,control,end,radius):
        self.axis('wood',start,control,end,radius)
        return curve(start,control,end)

    def foliage(self,path,phase,center,h,reach):
        f,r=self.f,self.rng
        for site in range(f['sites']):
            u=f['leaf_start']+(1-f['leaf_start'])*site/(f['sites']-1)
            start=self.at(path,min(.99,u))
            tangent=unit([b-a for a,b in zip(self.at(path,max(0,u-.04)),self.at(path,min(1,u+.04)))])
            lateral=unit([tangent[2],0,-tangent[0]])
            for side in [-1,1]:
                if r.random()>f['occupancy']:continue
                angle=phase+site*2.39996
                direction=unit(add(add(scale(tangent,.65),scale(lateral,side*.75)),
                                   [.16*math.cos(angle),f['up'],.16*math.sin(angle)]))
                flush=site==f['sites']-1 and not f.get('heads')
                length=f['length']*r.uniform(.78,1.18)*(.64 if flush else 1.)
                end=add(start,scale(direction,length))
                control=add(add(start,scale(direction,length*.55)),[0,length*.12,0])
                if f.get('clipped'):
                    # Grow inside an authored ellipsoid; this is neither mesh
                    # clipping nor a hidden sphere behind the actual blades.
                    def bound(p):
                        d=[p[0]-center[0],p[1]-h*.51,p[2]-center[2]]
                        norm=math.sqrt((d[0]/reach)**2+(d[1]/(h*.49))**2+(d[2]/reach)**2)
                        return add([center[0],h*.51,center[2]],scale(d,min(1.,1./max(norm,1e-9))))
                    control=bound(control);end=bound(end)
                normal=unit([direction[2],0,-direction[0]])
                roll=f.get('leaf_roll',0.)*math.sin(phase+site*1.618+side*2.1)
                binormal=unit([direction[1]*normal[2]-direction[2]*normal[1],direction[2]*normal[0]-direction[0]*normal[2],direction[0]*normal[1]-direction[1]*normal[0]])
                normal=add(scale(normal,math.cos(roll)),scale(binormal,math.sin(roll)))
                self.leaf('flush' if flush else 'green',start,control,end,
                          f['width']*r.uniform(.85,1.12)*(.64 if flush else 1.),normal,
                          base=.40,tip=.36 if f.get('round_leaf') else .82,fold=.025)

    def head(self,tip,angle):
        # The image resolves rounded heads, not petal anatomy. Tiny short axes
        # are closed native volumes, on real stalks, without invented corollas.
        for j in range(3):
            q=angle+j*TAU/3;end=add(tip,[.014*math.cos(q),.014+.006*j,.014*math.sin(q)])
            self.wood(tip,add(tip,[.007*math.cos(q),.012,.007*math.sin(q)]),end,.00048)
            self.organ('head',curve(add(end,[0,-.004,0]),end,add(end,[0,.004,0])),
                       dict(kind='axis',radius=.0048,tip_ratio=.80,ellipticity=1.),[1,0,0])

    def build(self):
        f,r=self.f,self.rng;h=f['height']*r.uniform(.94,1.06);reach=f['reach']*r.uniform(.94,1.06)
        phase=r.uniform(0,TAU)
        for lobe in range(f['lobes']):
            q=phase+lobe*TAU/f['lobes']
            center=[0,0,0] if f['lobes']==1 else [reach*.42*math.cos(q),0,reach*.42*math.sin(q)]
            local_h=h if f['lobes']==1 else h*(1. if lobe==0 else r.uniform(.63,.90))
            local_reach=reach if f['lobes']==1 else reach*.64
            def crown(angle,z,radial=1.):
                z=max(-.985,min(.998,z));radius=local_reach*math.sqrt(1-z*z)*radial
                irregular=1. if f.get('clipped') else 1+.06*math.sin(3*angle+phase)+.035*math.cos(7*angle+phase)
                return add(center,[radius*math.cos(angle)*irregular,local_h*(.51+.48*z),radius*math.sin(angle)*irregular])
            for k in range(f['stems']):
                a=phase+k*2.39996+r.uniform(-.18,.18)
                spread=.16*local_reach if f['id']=='xerico_cinza' else .055
                root=add(center,[spread*math.cos(a),-.009,spread*math.sin(a)])
                stem_tip=crown(a,.78,r.uniform(.65,.95)) if f.get('broom') else crown(a,.48,.32)
                control=add(center,[stem_tip[0]*.05-center[0]*.05,local_h*.40,stem_tip[2]*.05-center[2]*.05])
                stem=self.wood(root,control,stem_tip,.0035 if f.get('broom') else .009 if f['height']<.9 else .014)
                for j in range(f['shoots']):
                    bottom=-.96 if f.get('clipped') or f.get('vertical') or f['id']=='esfera_folhosa' else -.74
                    z=bottom+(.88-bottom)*(j+.5)/f['shoots'];angle=a+j*2.39996+r.uniform(-.18,.18)
                    start=self.at(stem,.16+.76*(z+.8)/1.7)
                    if f.get('broom'):
                        angle=a+r.uniform(-.60,.60);start=self.at(stem,.10+.55*(j+.5)/f['shoots'])
                        z=r.uniform(-.05,.88)
                    end=crown(angle,z,r.uniform(.76,.93));control=add(scale(add(start,end),.5),[0,local_h*.06,0])
                    branch=self.wood(start,control,end,.0024 if f['height']<.9 else .0035)
                    for twig in range(f['twigs']):
                        v=(twig+.5)/f['twigs'];origin=self.at(branch,.28+.65*v)
                        az=angle+r.uniform(-.46,.46);tz=z+r.uniform(-.20,.20)
                        if f.get('vertical'):tz=z+.20+r.uniform(0,.12)
                        if j==f['shoots']-1:
                            # Real terminal growth closes the polar gap left by
                            # lateral shoots; no cap mesh or opaque proxy.
                            tz=.50+.498*(1-(1-v)**2)
                        tip=crown(az,tz,r.uniform(.86,.98))
                        control=add(scale(add(origin,tip),.5),[0,local_h*.04,0])
                        path=self.wood(origin,control,tip,.00075 if f['height']<.9 else .0011)
                        self.foliage(path,az,center,local_h,local_reach)
                        for fork in range(f.get('fine_forks',0)):
                            root=self.at(path,.28+.17*fork)
                            fq=az+(-1 if fork%2 else 1)*r.uniform(.12,.38)
                            target=crown(fq,tz+r.uniform(-.08,.10),r.uniform(.74,.97))
                            fine=self.wood(root,add(scale(add(root,target),.5),[0,local_h*.04,0]),target,.0010)
                            if f.get('heads') and fork==1:self.head(fine[-1],fq)
                        if f.get('heads'):
                            self.head(tip,az)
                            self.head(self.at(path,.74),az+.8)
                for j in range(f.get('cap_twigs',0)):
                    # Upper shoots occupy the polar disk as well as its rim.
                    # Each remains a real branch rooted on an existing stem.
                    angle=a+j*2.39996;disk=.68*math.sqrt((j+.5)/f['cap_twigs'])
                    tip=crown(angle,math.sqrt(1-disk*disk),r.uniform(.94,1.))
                    start=self.at(stem,r.uniform(.70,.96))
                    path=self.wood(start,add(scale(add(start,tip),.5),[0,local_h*.045,0]),tip,.0010)
                    self.foliage(path,angle,center,local_h,local_reach)
        # Preserve unique IDs within their original layers, then partition the
        # complete source into the two native material channels and extra parts.
        self.layers['green']=self.layers.pop('wood')+self.layers.get('green',[])
        return self.layers


def library(seed=11701,variants=3):
    manifest,files=build_library(seed,variants,families=FAMILIES,plant_class=ReferenceShrub,prefix_tag='rs')
    for entry in manifest['entries']:
        family=next(f for f in FAMILIES if f['id']==entry['family'])
        for part in entry['components']:
            recipe=files[part['path']];recipe['settings']['blade_samples']=family['blade_samples']
            if part['layer']=='green':recipe['settings']['bark_color']=family['wood']
    return manifest,files


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--seed',type=int,default=11701)
    p.add_argument('--variants',type=int,default=3);p.add_argument('--output',type=Path,default=DEST)
    args=p.parse_args();manifest,files=library(args.seed,args.variants)
    for value in [manifest,*files.values()]:json.dumps(value,allow_nan=False)
    for path,value in files.items():atomic_json(args.output/path,value)
    atomic_json(args.output/'library.json',manifest)
    print(f"{len(FAMILIES)} profiles / {len(manifest['entries'])} variants / {len(files)} sources: {args.output}")


if __name__=='__main__':main()
