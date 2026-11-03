#!/usr/bin/env python3
"""Seeded woody shrub silhouettes, foliage occupancy and flowers as native curves.

Generic authoring profiles. Static closed components overlap at attachments;
this asset workflow does not imply welded vascular skin or gameplay physics.
"""
import argparse
import json
import math
from pathlib import Path

from groundcover_assets import Plant, ROOT, TAU, add, atomic_json, curve, library as build_library, scale

DEST=ROOT/'assets/botany/shrubs'
WOOD=[.25,.17,.085]
FAMILIES=[
    dict(id='compacto',name='Arbusto compacto denso',height=.85,reach=.68,stems=7,shoots=5,sites=7,occupancy=.96,length=.125,width=.070,green=[.075,.18,.025],colors=dict(wood=WOOD)),
    dict(id='aberto',name='Arbusto aberto',height=1.10,reach=.85,stems=5,shoots=4,sites=5,occupancy=.52,length=.11,width=.055,green=[.13,.23,.055],colors=dict(wood=WOOD)),
    dict(id='espalhado',name='Arbusto baixo espalhado',height=.65,reach=1.15,stems=7,shoots=5,sites=6,occupancy=.90,length=.14,width=.075,green=[.12,.22,.035],colors=dict(wood=WOOD)),
    dict(id='vertical',name='Arbusto vertical',height=1.85,reach=.48,stems=6,shoots=5,sites=7,occupancy=.90,length=.15,width=.048,green=[.095,.19,.035],colors=dict(wood=WOOD)),
    dict(id='folha_grande',name='Arbusto de folhas grandes',height=1.10,reach=.90,stems=5,shoots=4,sites=5,occupancy=.92,length=.24,width=.16,green=[.065,.18,.055],colors=dict(wood=WOOD)),
    dict(id='flores_brancas',name='Arbusto de flores brancas',height=1.10,reach=.86,stems=6,shoots=5,sites=6,occupancy=.93,length=.13,width=.070,green=[.075,.20,.035],colors=dict(wood=WOOD),accent=[.83,.80,.68],center=[.68,.40,.045],flowers='cluster'),
    dict(id='flores_rosas',name='Arbusto de flores rosas',height=.95,reach=.78,stems=6,shoots=5,sites=6,occupancy=.95,length=.135,width=.080,green=[.08,.18,.035],colors=dict(wood=WOOD),accent=[.65,.10,.30],center=[.76,.52,.09],flowers='large'),
    dict(id='flores_amarelas',name='Arbusto de flores amarelas',height=1.35,reach=.90,stems=5,shoots=5,sites=6,occupancy=.73,length=.13,width=.040,green=[.15,.23,.045],colors=dict(wood=WOOD),accent=[.78,.48,.035],center=[.29,.14,.035],flowers='star'),
]


def unit(v):
    length=math.sqrt(sum(x*x for x in v))
    return [x/length for x in v] if length>1e-12 else [1,0,0]


class Shrub(Plant):
    def at(self,path,u):
        x=u*(len(path)-1);i=min(int(x),len(path)-2);t=x-i
        return [(1-t)*a+t*b for a,b in zip(path[i],path[i+1])]

    def wood(self,start,control,end,radius):
        self.axis('wood',start,control,end,radius)
        return curve(start,control,end)

    def blossom(self,center,a,kind):
        size=.027 if kind=='cluster' else .060 if kind=='large' else .043
        petals=5 if kind!='star' else 8
        for j in range(petals):
            q=a+j*TAU/petals
            base=add(center,[size*.12*math.cos(q),0,size*.12*math.sin(q)])
            tip=add(center,[size*math.cos(q),.004+size*.06*math.sin(q),size*math.sin(q)])
            control=add(center,[size*.60*math.cos(q),size*.36,size*.60*math.sin(q)])
            self.leaf('petal',base,control,tip,size*(.65 if kind!='star' else .32),[-math.sin(q),0,math.cos(q)],base=.42,tip=.45,fold=.02)
        radius=size*.16
        self.organ('center',curve(add(center,[0,-.003,0]),add(center,[0,.003,0]),add(center,[0,.008,0])),
                   dict(kind='axis',radius=radius,tip_ratio=.70,ellipticity=1.),[1,0,0])

    def leaf_cluster(self,path,phase):
        f,r=self.f,self.rng
        for site in range(f['sites']):
            u=.20+.76*site/(f['sites']-1)
            for side in [-1,1]:
                if r.random()>f['occupancy']:continue
                v=max(.05,min(.99,u+r.uniform(-.025,.025)))
                start=self.at(path,v)
                before=self.at(path,max(0,v-.025));after=self.at(path,min(1,v+.025))
                tangent=unit([b-a for a,b in zip(before,after)])
                lateral=unit([tangent[2],0,-tangent[0]])
                normal_axis=unit([tangent[1]*lateral[2]-tangent[2]*lateral[1],tangent[2]*lateral[0]-tangent[0]*lateral[2],tangent[0]*lateral[1]-tangent[1]*lateral[0]])
                rotation=.55*math.sin(phase+site*2.39996)
                around=add(scale(lateral,math.cos(rotation)),scale(normal_axis,math.sin(rotation)))
                direction=unit(add(add(scale(tangent,.35),scale(around,side*.85)),[0,.20+r.uniform(-.15,.25),0]))
                length=f['length']*r.uniform(.72,1.18)
                end=add(start,scale(direction,length))
                control=add(add(start,scale(direction,length*.55)),[0,length*.22,0])
                normal=unit([direction[2],0,-direction[0]])
                self.leaf('green',start,control,end,f['width']*r.uniform(.78,1.15),normal,base=.46,tip=.66,fold=.045)

    def build(self):
        f,r=self.f,self.rng;h=f['height']*r.uniform(.90,1.10);reach=f['reach']*r.uniform(.9,1.1)
        phase=r.uniform(0,TAU)
        for k in range(f['stems']):
            a=k*2.39996+phase+r.uniform(-.23,.23)
            root=[.095*math.cos(a),-.008,.095*math.sin(a)]
            tip=[reach*r.uniform(.18,.42)*math.cos(a),h*r.uniform(.72,.98),reach*r.uniform(.18,.42)*math.sin(a)]
            stem=self.wood(root,[tip[0]*.12,h*.53,tip[2]*.12],tip,.015 if h<1.2 else .020)
            for j in range(f['shoots']):
                v=(j+.5)/f['shoots'];u=.20+.72*v
                start=self.at(stem,u);q=a+j*2.39996+r.uniform(-.35,.35)
                y=h*(.17+.78*v)*r.uniform(.94,1.06)
                section=math.sqrt(max(.12,1-((y/h-.48)/.57)**2))
                radius=reach*section*(.88+.10*math.sin(3*q+phase))*r.uniform(.85,1.10)
                end=[radius*math.cos(q),y,radius*math.sin(q)]
                control=[start[0]*.50+end[0]*.50,max(start[1],end[1])+.10*h,start[2]*.50+end[2]*.50]
                branch=self.wood(start,control,end,.0048)
                # A terminal twig broadens each local leaf group without filling
                # a sphere with unrelated leaves. Its root is on the real branch.
                origin=self.at(branch,.60);angle=q+(-1 if (k+j)%2 else 1)*.65
                twig_end=add(end,[reach*.12*math.cos(angle),h*.09,reach*.12*math.sin(angle)])
                twig=self.wood(origin,add(origin,[reach*.15*math.cos(angle),h*.13,reach*.15*math.sin(angle)]),twig_end,.0020)
                self.leaf_cluster(branch,q)
                self.leaf_cluster(twig,q+.8)
                if f.get('flowers') and (k+j)%2==0:
                    kind=f['flowers']
                    center=add(twig_end,[.055*math.cos(q),.070,.055*math.sin(q)])
                    self.wood(twig_end,add(twig_end,[.025*math.cos(q),.05,.025*math.sin(q)]),center,.0012)
                    offsets=[[0,.015,0]]
                    if kind=='cluster':offsets=[[.038*math.cos(t),.022+.008*math.sin(t),.038*math.sin(t)] for t in [q,q+TAU/3,q+2*TAU/3]]
                    for offset in offsets:self.blossom(add(center,offset),q,kind)
        # Wood and foliage share one source and the compiler's two native
        # material channels, keeping the collection below the64-recipe quota.
        self.layers['green']=self.layers.pop('wood')+self.layers['green']
        return self.layers


def library(seed=9501,variants=3):
    manifest,files=build_library(seed,variants,families=FAMILIES,plant_class=Shrub,prefix_tag='sh')
    for entry in manifest['entries']:
        family=next(f for f in FAMILIES if f['id']==entry['family'])
        for part in entry['components']:
            if part['layer']=='green':files[part['path']]['settings']['bark_color']=family['colors']['wood']
    return manifest,files


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--seed',type=int,default=9501)
    p.add_argument('--variants',type=int,default=3);p.add_argument('--output',type=Path,default=DEST)
    args=p.parse_args();manifest,files=library(args.seed,args.variants)
    for value in [manifest,*files.values()]:json.dumps(value,allow_nan=False)
    for path,value in files.items():atomic_json(args.output/path,value)
    atomic_json(args.output/'library.json',manifest)
    print(f"{len(FAMILIES)} types / {len(manifest['entries'])} variants / {len(files)} components: {args.output}")


if __name__=='__main__':main()
