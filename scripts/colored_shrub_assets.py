#!/usr/bin/env python3
"""Reference-guided ornamental foliage; native curves, mature leaves and flush.

Photographs guide authored profiles only, never mesh coordinates or textures.
See docs/references/botany/colored-shrubs-analysis.md before adjusting profiles.
"""
import argparse
import json
from pathlib import Path

from groundcover_assets import ROOT, atomic_json, library as build_library
from shrub_assets import Shrub

DEST=ROOT/'assets/botany/colored-shrubs'
FAMILIES=[
    dict(id='pingo_ouro',name='Pingo-de-ouro · referência Gold Mound',height=.72,reach=.46,stems=7,shoots=6,sites=10,occupancy=.97,
         length=.066,width=.032,green=[.46,.37,.025],accent=[.67,.53,.055],colors=dict(wood=[.25,.19,.10]),
         leaf_base=.42,leaf_tip=.74,flush_scale=.62,flush_sites=3,blade_samples=6,reference='duranta'),
    dict(id='berberis_vinho',name='Berberis · referência Crimson Pygmy',height=.68,reach=.62,stems=6,shoots=5,sites=9,occupancy=.94,
         length=.045,width=.029,green=[.145,.018,.045],accent=[.37,.055,.08],colors=dict(wood=[.25,.09,.055]),
         leaf_base=.74,leaf_tip=.36,flush_scale=.54,flush_sites=3,blade_samples=6,reference='berberis'),
    dict(id='loropetalo_roxo',name='Loropétalo · referência rubrum',height=1.10,reach=.92,stems=6,shoots=5,sites=7,occupancy=.90,
         length=.095,width=.050,green=[.085,.025,.070],accent=[.24,.06,.12],colors=dict(wood=[.20,.14,.09]),
         leaf_base=.44,leaf_tip=.70,flush_scale=.67,flush_sites=2,blade_samples=6,alternate=True,reference='loropetalum'),
    dict(id='cotinus_roxo',name='Cotinus · referência Royal Purple',height=1.65,reach=1.05,stems=5,shoots=4,sites=5,occupancy=.79,
         length=.16,width=.145,green=[.085,.036,.105],accent=[.21,.065,.11],colors=dict(wood=[.22,.14,.09]),
         leaf_base=.46,leaf_tip=.32,flush_scale=.70,flush_sites=2,blade_samples=8,alternate=True,reference='cotinus'),
]


class ColoredShrub(Shrub):
    def __init__(self,family,seed):
        super().__init__(family,seed)
        self.flush_ids=set()

    def leaf_cluster(self,path,phase):
        start=len(self.layers.get('green',[]))
        super().leaf_cluster(path,phase)
        new=self.layers['green'][start:]
        for index,organ in enumerate(new):
            outline=organ['shape']['outline'];outline['base_power']=self.f['leaf_base'];outline['tip_power']=self.f['leaf_tip']
            organ['shape']['fold']=.035
            if self.f.get('alternate'):
                origin=organ['path'][0];best=(float('inf'),0.)
                for segment,(a,b) in enumerate(zip(path,path[1:])):
                    d=[y-x for x,y in zip(a,b)];length=sum(x*x for x in d)
                    t=max(0.,min(1.,sum((p-x)*y for p,x,y in zip(origin,a,d))/length))
                    point=[x+t*y for x,y in zip(a,d)]
                    candidate=(sum((x-y)**2 for x,y in zip(origin,point)),(segment+t)/(len(path)-1))
                    best=min(best,candidate)
                u=max(.05,min(.99,best[1]+(.30 if index%2 else -.30)/(self.f['sites']-1)))
                target=self.at(path,u);delta=[x-y for x,y in zip(target,origin)]
                organ['path']=[[x+y for x,y in zip(p,delta)] for p in organ['path']]
        for organ in new[-self.f['flush_sites']:]:
            self.flush_ids.add(organ['id'])

    def build(self):
        layers=super().build();mature=[];flush=[]
        for organ in layers['green']:
            if organ['id'] not in self.flush_ids:
                mature.append(organ);continue
            # Keep the insertion point; smaller terminal blades are a separate
            # shared colour component, without a new native per-organ material.
            root=organ['path'][0];factor=self.f['flush_scale']
            organ['path']=[[a+(b-a)*factor for a,b in zip(root,p)] for p in organ['path']]
            organ['shape']['width']*=factor
            flush.append(organ)
        return dict(green=mature,flush=flush)


def library(seed=10601,variants=3):
    manifest,files=build_library(seed,variants,families=FAMILIES,plant_class=ColoredShrub,prefix_tag='cs')
    for entry in manifest['entries']:
        f=next(f for f in FAMILIES if f['id']==entry['family'])
        for part in entry['components']:
            r=files[part['path']]
            r['settings']['blade_samples']=f['blade_samples']
            if part['layer']=='green':r['settings']['bark_color']=f['colors']['wood']
    return manifest,files


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--seed',type=int,default=10601)
    p.add_argument('--variants',type=int,default=3);p.add_argument('--output',type=Path,default=DEST)
    args=p.parse_args();manifest,files=library(args.seed,args.variants)
    for value in [manifest,*files.values()]:json.dumps(value,allow_nan=False)
    for path,value in files.items():atomic_json(args.output/path,value)
    atomic_json(args.output/'library.json',manifest)
    print(f"{len(FAMILIES)} references / {len(manifest['entries'])} variants / {len(files)} sources: {args.output}")


if __name__=='__main__':main()
