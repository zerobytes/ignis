#!/usr/bin/env python3
"""Tall grasses and branching herbs authored as editable native botanical curves.

Generic silhouettes, not botanically calibrated species. Uses the same compiler
and shared-instance asset workflow as the approved short groundcover collection.
"""
import argparse
import math
from pathlib import Path

from groundcover_assets import Plant, ROOT, TAU, add, atomic_json, curve, library as build_library, scale

DEST=ROOT/'assets/botany/tallgrass'
FAMILIES=[
    dict(id='capim_muralha',name='Capim alto denso',form='dense',height=2.0,reach=.65,width=.055,blades=25,green=[.13,.23,.035]),
    dict(id='capim_longo',name='Capim alto recurvado',form='arch',height=1.85,reach=1.05,width=.045,blades=26,green=[.16,.27,.045]),
    dict(id='canico',name='Caniço de folhas largas',form='cane',height=2.65,reach=.65,width=.10,blades=9,green=[.13,.24,.055]),
    dict(id='junco',name='Junco de espiga cilíndrica',form='reed',height=2.10,reach=.44,width=.038,blades=20,green=[.14,.25,.065],accent=[.19,.095,.035]),
    dict(id='pluma_alta',name='Capim de plumas altas',form='plume',height=2.05,reach=.90,width=.028,blades=22,green=[.23,.29,.085],accent=[.64,.55,.37]),
    dict(id='macega_seca',name='Macega alta seca',form='dry',height=1.70,reach=.80,width=.022,blades=27,green=[.36,.28,.11],accent=[.43,.31,.13]),
    dict(id='mato_ramos',name='Mato alto ramificado',form='herb',height=1.75,reach=.64,width=.085,blades=7,green=[.075,.18,.035]),
    dict(id='flores_altas',name='Mato alto florido',form='flower',height=1.65,reach=.64,width=.07,blades=6,green=[.10,.22,.045],accent=[.71,.44,.08],center=[.24,.115,.035]),
]


class TallPlant(Plant):
    def point(self,path,u):
        x=u*(len(path)-1);i=min(int(x),len(path)-2);t=x-i
        return [(1-t)*a+t*b for a,b in zip(path[i],path[i+1])]

    def blade(self,start,a,length,reach,width,drop=.12):
        end=add(start,[reach*math.cos(a),length*(1-drop),reach*math.sin(a)])
        control=add(start,[reach*.15*math.cos(a),length*1.16,reach*.15*math.sin(a)])
        self.leaf('green',start,control,end,width,[-math.sin(a),0,math.cos(a)],base=.12,tip=1.,fold=.24)

    def seedhead(self,base,a,height,form):
        r=self.rng
        if form=='reed':
            # Solid round head; bounded tube, not a flat brown card.
            end=add(base,[.035*math.cos(a),height,.035*math.sin(a)])
            self.organ('seed',curve(base,add(base,[0,height*.6,0]),end),
                       dict(kind='axis',radius=.026*r.uniform(.85,1.15),tip_ratio=.82,ellipticity=1.),[1,0,0])
            return
        for j in range(30 if form=='plume' else 14):
            u=j/(29 if form=='plume' else 13);q=a+j*2.39996
            root=add(base,[0,u*height,0]);span=(.12 if form=='plume' else .09)*math.sin(math.pi*(.08+.88*u))
            tip=add(root,[span*math.cos(q),height*.22,span*math.sin(q)])
            if form=='dry':
                self.axis('seed',root,add(root,[span*.55*math.cos(q),height*.15,span*.55*math.sin(q)]),tip,.00085)
                for side in [-1,1]:
                    end=add(tip,[side*.008*math.sin(q),.028,side*.008*math.cos(q)])
                    self.leaf('seed',tip,add(tip,[0,.018,0]),end,.004,[math.sin(q),0,math.cos(q)],tip=.65)
            else:
                self.leaf('seed',root,add(root,[span*.6*math.cos(q),height*.23,span*.6*math.sin(q)]),tip,.0075,[-math.sin(q),0,math.cos(q)],tip=.7,fold=.05)

    def flower(self,center,a):
        for j in range(8):
            q=a+j*TAU/8
            base=add(center,[.009*math.cos(q),0,.009*math.sin(q)])
            tip=add(center,[.060*math.cos(q),-.006+.009*math.cos(q),.060*math.sin(q)])
            self.leaf('petal',base,add(center,[.035*math.cos(q),.020,.035*math.sin(q)]),tip,.022,[-math.sin(q),0,math.cos(q)],base=.3,tip=.5,fold=.02)
        self.organ('center',curve(add(center,[0,-.004,0]),add(center,[0,.010,0]),add(center,[0,.020,0])),
                   dict(kind='axis',radius=.014,tip_ratio=.6,ellipticity=1.),[1,0,0])

    def build(self):
        f,r=self.f,self.rng;h=f['height']*r.uniform(.88,1.12);reach=f['reach']*r.uniform(.88,1.1);form=f['form']
        if form in ('cane','herb','flower'):
            for k in range(5 if form=='cane' else 4):
                a=k*2.39996+r.uniform(-.3,.3)
                root=[.17*math.cos(a),-.008,.17*math.sin(a)]
                top=[reach*.38*math.cos(a),h*r.uniform(.78,1.10),reach*.38*math.sin(a)]
                control=[top[0]*.20,top[1]*.70,top[2]*.20];path=curve(root,control,top)
                self.axis('green',root,control,top,.009 if form=='cane' else .004)
                for j in range(f['blades']):
                    u=.14+.75*j/(f['blades']-1);base=self.point(path,u);q=a+j*(math.pi if form=='cane' else 2.39996)
                    envelope=math.sin(math.pi*(.15+.75*u))
                    if form=='cane':
                        self.blade(base,q,.42*envelope,.72*envelope,f['width']*r.uniform(.75,1.12),drop=.85)
                    else:
                        # Alternating genuine lateral stalks with their own leaves.
                        endpoint=add(base,[reach*.63*envelope*math.cos(q),h*.12,reach*.63*envelope*math.sin(q)])
                        branch_control=add(base,[reach*.38*envelope*math.cos(q),h*.11,reach*.38*envelope*math.sin(q)])
                        branch=curve(base,branch_control,endpoint)
                        self.axis('green',base,branch_control,endpoint,.0018)
                        for node in range(3):
                            v=.28+.31*node;origin=self.point(branch,v);angle=q+(-1 if node%2 else 1)*1.1
                            length=.15*envelope*r.uniform(.8,1.15)
                            tip=add(origin,[length*math.cos(angle),.055,length*math.sin(angle)])
                            self.leaf('green',origin,add(origin,[length*.55*math.cos(angle),.06,length*.55*math.sin(angle)]),tip,
                                      f['width']*envelope,[-math.sin(angle),0,math.cos(angle)],base=.42,tip=.75,fold=.06)
                        if form=='flower' and j%2==0:self.flower(endpoint,q)
                if form=='flower':self.flower(top,a)
        else:
            for i in range(f['blades']):
                a=i*2.39996+r.uniform(-.28,.28);radius=reach*r.uniform(.04,.25)
                root=[radius*math.cos(a),-.008,radius*math.sin(a)]
                vig=r.uniform(.60,1.12);drop=.62 if form=='arch' else .22 if form in ('dry','plume') else .10
                self.blade(root,a,h*vig,reach*r.uniform(.6,1.15),f['width']*r.uniform(.72,1.15),drop=drop)
            if form in ('reed','plume','dry'):
                for k in range(4):
                    a=k*2.39996+r.uniform(-.4,.4);root=[.08*math.cos(a),-.008,.08*math.sin(a)]
                    top=[reach*.30*math.cos(a),h*r.uniform(.98,1.15),reach*.30*math.sin(a)]
                    self.axis('green',root,[top[0]*.25,top[1]*.7,top[2]*.25],top,.004 if form=='reed' else .002)
                    self.seedhead(top,a,.24 if form=='reed' else .36 if form=='plume' else .24,form)
        return self.layers


def library(seed=8401,variants=3):
    return build_library(seed,variants,families=FAMILIES,plant_class=TallPlant,prefix_tag='tg')


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--seed',type=int,default=8401)
    p.add_argument('--variants',type=int,default=3);p.add_argument('--output',type=Path,default=DEST)
    args=p.parse_args();manifest,files=library(args.seed,args.variants)
    for path,value in files.items():atomic_json(args.output/path,value)
    atomic_json(args.output/'library.json',manifest)
    print(f"{len(FAMILIES)} types / {len(manifest['entries'])} variants / {len(files)} components: {args.output}")


if __name__=='__main__':main()
