#!/usr/bin/env python3
"""Author seeded groundcover as native botanical curves, never stored triangles.

Components separate colours using the existing native two-material compiler.
This is a build-time asset library, not a runtime growth/physics implementation.
"""
import argparse
import hashlib
import json
import math
import os
import random
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'assets/botany/groundcover'
TAU = math.tau

# Generic authored silhouettes; names do not claim calibrated botanical species.
FAMILIES = [
    dict(id='grama_baixa', name='Grama baixa', form='grass', height=.16, reach=.20, width=.012, blades=22, green=[.16,.28,.035]),
    dict(id='grama_fina', name='Grama fina', form='grass', height=.30, reach=.12, width=.006, blades=25, green=[.10,.23,.035]),
    dict(id='folha_larga', name='Touceira de folha larga', form='grass', height=.32, reach=.27, width=.034, blades=13, green=[.075,.19,.045]),
    dict(id='capim_arqueado', name='Capim arqueado', form='grass', height=.55, reach=.48, width=.017, blades=18, green=[.17,.29,.045]),
    dict(id='capim_alto', name='Capim alto', form='grass', height=.95, reach=.32, width=.019, blades=15, green=[.14,.25,.055]),
    dict(id='capim_seco', name='Capim seco', form='grass', height=.46, reach=.34, width=.011, blades=19, green=[.40,.31,.12]),
    dict(id='capim_espiga', name='Capim com espigas', form='spike', height=.60, reach=.29, width=.012, blades=13, green=[.18,.27,.055], accent=[.47,.34,.12]),
    dict(id='capim_panicula', name='Capim de panículas abertas', form='panicle', height=.79, reach=.31, width=.011, blades=12, green=[.15,.25,.045], accent=[.39,.25,.12]),
    dict(id='capim_pluma', name='Capim de plumas', form='plume', height=.88, reach=.40, width=.015, blades=14, green=[.21,.29,.075], accent=[.58,.49,.30]),
    dict(id='mato_trifolio', name='Mato rasteiro trifoliado', form='trifoliate', height=.14, reach=.22, width=.050, blades=7, green=[.075,.22,.035]),
    dict(id='flores_brancas', name='Mato de flores brancas', form='flower', height=.34, reach=.18, width=.033, blades=9, green=[.085,.22,.04], accent=[.84,.81,.66], center=[.67,.38,.035], petals=9),
    dict(id='flores_roxas', name='Mato de flores roxas', form='flower', height=.48, reach=.21, width=.029, blades=10, green=[.10,.23,.055], accent=[.46,.095,.42], center=[.76,.46,.055], petals=5),
]


def atomic_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + '.candidate')
    try:
        temporary.write_text(json.dumps(value, ensure_ascii=False, separators=(',', ':'), allow_nan=False) + '\n')
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def add(a, b):
    return [x+y for x, y in zip(a, b)]


def scale(v, f):
    return [x*f for x in v]


def curve(start, control, end):
    return [[(1-t)**2*a+2*(1-t)*t*b+t*t*c for a,b,c in zip(start,control,end)] for t in [i/6 for i in range(7)]]


class Plant:
    def __init__(self, family, seed):
        self.f = family
        self.rng = random.Random(seed)
        self.seed = seed
        self.layers = {}

    def organ(self, layer, path, shape, normal):
        organs = self.layers.setdefault(layer, [])
        organs.append(dict(id=f'{layer}_{len(organs):03}', parent=None, insertion=0., path=path, normal=normal,
                           shape=shape, birth=0., duration=1., density=600., young=2e7, shear=8e6))

    def leaf(self, layer, start, control, end, width, normal, base=.30, tip=1.0, fold=.12):
        self.organ(layer, curve(start, control, end), dict(kind='blade', width=width, thickness=.00018,
                   fold=fold, cup=.035, outline=dict(kind='strip', base_power=base, tip_power=tip)), normal)

    def axis(self, layer, start, control, end, radius=.0013):
        self.organ(layer, curve(start,control,end), dict(kind='axis', radius=radius, tip_ratio=.40, ellipticity=1.), [1,0,0])

    def build(self):
        f,r=self.f,self.rng
        h,reach=f['height']*r.uniform(.85,1.15),f['reach']*r.uniform(.88,1.12)
        if f['form']=='trifoliate':
            for i in range(f['blades']):
                a=i*2.39996+r.uniform(-.25,.25)
                root=[reach*.23*math.cos(a),-.005,reach*.23*math.sin(a)]
                end=[reach*math.cos(a),h*r.uniform(.7,1.15),reach*math.sin(a)]
                self.axis('green',root,[end[0]*.6,end[1]*.7,end[2]*.6],end,.0009)
                for j in range(3):
                    q=a+TAU*j/3
                    d=[math.cos(q),.12,math.sin(q)]
                    mid=add(end,scale(d,.030));tip=add(end,scale(d,.062*r.uniform(.85,1.1)))
                    self.leaf('green',end,mid,tip,f['width'],[math.sin(q),0,-math.cos(q)],base=.48,tip=.55,fold=.02)
        else:
            for i in range(f['blades']):
                a=i*2.39996+r.uniform(-.35,.35)
                radial=r.uniform(.04,.28)*reach
                root=[radial*math.cos(a),-.008,radial*math.sin(a)]
                vig=r.uniform(.62,1.20)
                length=h*vig*(.48 if f['form']=='flower' else 1.)
                spread=reach*r.uniform(.55,1.15)
                drop=.55 if f['id']=='capim_arqueado' else .17
                end=[root[0]+spread*math.cos(a),length*(1-drop),root[2]+spread*math.sin(a)]
                control=[root[0]+spread*.16*math.cos(a),length*1.15,root[2]+spread*.16*math.sin(a)]
                self.leaf('green',root,control,end,f['width']*r.uniform(.65,1.25),[-math.sin(a),0,math.cos(a)],base=.12,tip=.9,fold=.22)
        if f['form'] in ('spike','panicle','plume'):
            for k in range(3):
                a=k*2.39996+r.uniform(-.5,.5)
                start=[.025*math.cos(a),-.005,.025*math.sin(a)]
                top=[reach*.45*math.cos(a),h*r.uniform(1.15,1.45),reach*.45*math.sin(a)]
                self.axis('green',start,[top[0]*.2,top[1]*.7,top[2]*.2],top)
                if f['form']=='panicle':
                    for j in range(9):
                        u=j/8;q=a+j*2.39996
                        base=add(top,[0,-h*.23*(1-u),0])
                        span=.095*(1-u*.65)
                        end=add(base,[span*math.cos(q),.035,span*math.sin(q)])
                        self.axis('seed',base,add(base,scale([math.cos(q),.25,math.sin(q)],span*.6)),end,.00065)
                        for side in [-1,1]:
                            p=add(end,[.004*side*math.sin(q),.012,.004*side*math.cos(q)])
                            self.leaf('seed',end,add(end,[0,.008,0]),p,.003,[math.sin(q),0,math.cos(q)],tip=.65)
                else:
                    for j in range(30 if f['form']=='plume' else 12):
                        u=j/(29 if f['form']=='plume' else 11);q=a+j*2.39996
                        base=add(top,[0,-h*.19*(1-u),0])
                        span=(.040 if f['form']=='plume' else .018)*math.sin(math.pi*(.12+.8*u))
                        end=add(base,[span*math.cos(q),.055 if f['form']=='plume' else .014,span*math.sin(q)])
                        self.leaf('seed',base,add(base,[span*.7*math.cos(q),.032,span*.7*math.sin(q)]),end,.0025 if f['form']=='plume' else .011,[-math.sin(q),0,math.cos(q)],tip=.6,fold=.02)
        if f['form']=='flower':
            for k in range(3):
                a=k*2.39996+r.uniform(-.3,.3)
                start=[reach*.16*math.cos(a),-.005,reach*.16*math.sin(a)]
                top=[reach*.65*math.cos(a),h*r.uniform(1.10,1.65),reach*.65*math.sin(a)]
                self.axis('green',start,[top[0]*.35,top[1]*.70,top[2]*.35],top)
                for node in range(2):
                    origin=curve(start,[top[0]*.35,top[1]*.70,top[2]*.35],top)[2+node]
                    q=a+node*math.pi
                    end=add(origin,[.07*math.cos(q),.025,.07*math.sin(q)])
                    self.leaf('green',origin,add(origin,[.04*math.cos(q),.03,.04*math.sin(q)]),end,.025,[-math.sin(q),0,math.cos(q)],base=.4,tip=.8)
                # Discrete petals in a slightly tilted radial corolla, no billboard.
                for j in range(f['petals']):
                    q=a+TAU*j/f['petals'];size=.035 if f['petals']==9 else .031
                    base=add(top,[.005*math.cos(q),0,.005*math.sin(q)])
                    end=add(top,[size*math.cos(q),.003+math.cos(q)*.006,size*math.sin(q)])
                    control=add(top,[size*.6*math.cos(q),.012,size*.6*math.sin(q)])
                    self.leaf('petal',base,control,end,.014 if f['petals']==9 else .025,[-math.sin(q),0,math.cos(q)],base=.25,tip=.40,fold=.01)
                self.axis('center',add(top,[0,-.002,0]),add(top,[0,.003,0]),add(top,[0,.009,0]),.008 if f['petals']==9 else .005)
        return self.layers


def library(seed=7301, variants=3, *, families=FAMILIES, plant_class=Plant, prefix_tag='gc'):
    if not 1<=variants<=8 or not 0<=seed<2**32:
        raise ValueError('variants requires1..8; seed requires0..2**32-1')
    files={};entries=[]
    for f in families:
        for v in range(variants):
            identity=f"{f['id']}_{v}"
            s=int.from_bytes(hashlib.sha256(f'{seed}:{identity}'.encode()).digest()[:4], 'little')
            plant=plant_class(f,s);layers=plant.build();components=[]
            for layer,organs in layers.items():
                prefix=f'{prefix_tag}_{identity}_{layer}'
                color=f['green'] if layer=='green' else f.get('center',f.get('accent')) if layer=='center' else f['accent']
                relative=f'{identity}-{layer}.ignis-botany'
                files[relative]=dict(version=7,prefix=prefix,source=dict(version=7,name=f"{f['name']} / {v+1} / {layer}",seed=s,organs=organs),
                                     settings=dict(axis_samples=4,blade_samples=4,sides=6,bark_color=color,leaf_color=color))
                components.append(dict(path=relative,prefix=prefix,layer=layer))
            entries.append(dict(id=identity,family=f['id'],name=f['name'],variant=v,seed=s,components=components))
    return dict(version=1,seed=seed,variants=variants,units='metres',families=families,entries=entries),files


def source_fingerprint(destination, manifest):
    digest=hashlib.sha256((destination/'library.json').read_bytes())
    for entry in manifest['entries']:
        for part in entry['components']:
            digest.update(part['path'].encode())
            digest.update((destination/part['path']).read_bytes())
    return digest.hexdigest()


def write_library(destination=DEST, seed=7301, variants=3):
    manifest,files=library(seed,variants)
    # Construct and validate JSON in memory before publishing any generated file.
    for value in [manifest,*files.values()]:
        json.dumps(value,allow_nan=False)
    for relative,value in files.items():
        atomic_json(destination/relative,value)
    atomic_json(destination/'library.json',manifest)
    return manifest


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--seed',type=int,default=7301);p.add_argument('--variants',type=int,default=3)
    p.add_argument('--output',type=Path,default=DEST)
    args=p.parse_args();m=write_library(args.output,args.seed,args.variants)
    print(json.dumps(dict(types=len(m['families']),variants=len(m['entries']),components=sum(len(e['components']) for e in m['entries']),path=str(args.output)),ensure_ascii=False))


if __name__=='__main__':
    main()
