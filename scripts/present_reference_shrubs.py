#!/usr/bin/env python3
"""Native close views and separate desert/topiary demonstrations of the library.

Run demo_groundcover.py --collection reference-shrubs first. No human scene or
window is touched; all outputs replace stable paths in artifacts/reference-shrubs.
"""
import copy
import hashlib
import html
import json
import subprocess

from demo_groundcover import field_scene,fit,ground,place,relative,scene_stats,setup
from groundcover_assets import ROOT,atomic_json,source_fingerprint
from reference_shrub_assets import DEST,FAMILIES,library
from shrub_assets import unit
from verify_lab import Client

OUT=ROOT/'artifacts/reference-shrubs'


def small_scene(compiled,templates,entries):
    ids={i['asset'] for e in entries for i in templates[e['id']]}
    scene={k:v for k,v in compiled.items() if k not in ['meshes','botany']};scene['botany']=[]
    scene['meshes']=dict(assets=[a for a in compiled['meshes']['assets'] if a['id'] in ids],instances=[],objects=[])
    ground(scene)
    return scene


def main():
    manifest=json.loads((DEST/'library.json').read_text());expected,files=library(manifest['seed'],manifest['variants'])
    assert manifest==expected and all(json.loads((DEST/path).read_text())==v for path,v in files.items()),'Canonical sources changed; regenerate explicitly'
    built=json.loads((OUT/'library-build.json').read_text())
    assert built['source_sha256']==source_fingerprint(DEST,manifest),'Stale compiled library; run demo_groundcover.py --collection reference-shrubs'
    for path,key in [('library.scene.json','scene_sha256'),('templates.json','templates_sha256')]:
        assert hashlib.sha256((OUT/path).read_bytes()).hexdigest()==built[key],'Compiled library changed'
    compiled=json.loads((OUT/'library.scene.json').read_text());templates=json.loads((OUT/'templates.json').read_text())
    native=json.loads((OUT/'report.json').read_text());report=dict(source_sha256=built['source_sha256'],details={},crops={},scenes={})
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'presentation-runtime.log')
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='reference-shrub-presentation',version='1')));c.request('notifications/initialized',notification=True)
        report['build_id']=c.call('inspect')['build_id']
        for f in FAMILIES:
            entry=next(e for e in manifest['entries'] if e['family']==f['id']);scene=small_scene(compiled,templates,[entry])
            place(scene,templates,entry,'detail',[0,0,0],0)
            atomic_json(OUT/'individual.scene.json',scene);c.call('load_scene',path=relative(OUT/'individual.scene.json'))
            parts=[native['components'][p['prefix']] for p in entry['components']]
            lo=[min(p['bounds_min'][k] for p in parts) for k in range(3)];hi=[max(p['bounds_max'][k] for p in parts) for k in range(3)]
            hi[1]-=.025;lo[1]-=.025
            detail=OUT/f"{f['id']}-detail.png";fit(c,lo,hi,detail,direction=(.42,.25,1),width=1100,height=1000)
            crop=OUT/f"{f['id']}-crop.png";box=[300,270,500,500]
            subprocess.run(['sips','-c','500','500','--cropOffset','270','300',str(detail),'--out',str(crop)],check=True,capture_output=True)
            report['crops'][f['id']]=dict(source=detail.name,box=box,resize=False,rotation=0)
            report['details'][f['id']]=dict(bounds=[lo,hi],direction=[.42,.25,1],width=1100,height=1000)
            setup(c)
            if f.get('heads'):
                part=next(p for p in entry['components'] if p['layer']=='head');recipe=copy.deepcopy(files[part['path']])
                organs=recipe['source']['organs'][:3];origin=organs[0]['path'][0]
                for o in organs:o['path']=[[x-y for x,y in zip(p,origin)] for p in o['path']]
                recipe['source']['organs']=organs;recipe['prefix']='macro_heads';recipe['settings']['position']=[0,.08,0]
                c.call('botany',operation=dict(kind='publish',recipe=recipe))
                fit(c,[-.045,.065,-.045],[.045,.13,.045],OUT/f"{f['id']}-macro.png",direction=(.5,.65,1),width=1000,height=750)
            else:
                extent=f['length']*1.4
                for index,layer in enumerate(['green','flush']):
                    part=next(p for p in entry['components'] if p['layer']==layer);recipe=copy.deepcopy(files[part['path']])
                    organ=copy.deepcopy(next(o for o in recipe['source']['organs'] if o['shape']['kind']=='blade'))
                    origin=organ['path'][0];t=unit([y-x for x,y in zip(organ['path'][0],organ['path'][1])]);n=unit(organ['normal'])
                    projection=sum(a*b for a,b in zip(t,n));n=unit([a-projection*b for a,b in zip(n,t)])
                    b=[t[1]*n[2]-t[2]*n[1],t[2]*n[0]-t[0]*n[2],t[0]*n[1]-t[1]*n[0]]
                    organ['path']=[[sum((x-y)*d for x,y,d in zip(point,origin,axis)) for axis in (n,b,t)] for point in organ['path']]
                    organ['normal']=[1,0,0];recipe['source']['organs']=[organ];recipe['prefix']=f'macro_{layer}';recipe['settings']['position']=[(index-.5)*extent,.08,0]
                    c.call('botany',operation=dict(kind='publish',recipe=recipe))
                fit(c,[-extent,.07,0],[extent,.12,extent],OUT/f"{f['id']}-macro.png",direction=(.05,1,.12),width=1000,height=750)
        recipe=json.loads((ROOT/'examples/reference-shrubs-desert.recipe.json').read_text())
        desert,counts,_=field_scene(compiled,templates,manifest,recipe)
        next(i for i in desert['meshes']['instances'] if i['id']=='gc_ground')['color']=[.46,.30,.12]
        atomic_json(OUT/'desert.scene.json',desert);c.call('load_scene',path=relative(OUT/'desert.scene.json'))
        c.call('set_camera',camera=dict(position=[8,5,10],target=[0,.3,0],fov_degrees=50))
        c.call('capture',path=relative(OUT/'desert.png'),width=1800,height=1200);c.call('save_scene',path=relative(OUT/'desert.scene.json'))
        report['scenes']['desert']=dict(counts=counts,stats=scene_stats(desert))
        entries=[e for e in manifest['entries'] if e['family']=='esfera_miuda'];garden=small_scene(compiled,templates,entries)
        next(i for i in garden['meshes']['instances'] if i['id']=='gc_ground')['color']=[.12,.20,.035]
        for j in range(6):place(garden,templates,entries[j%3],f'topiary_{j}',[(j-2.5)*1.28,0,0],j*.7)
        atomic_json(OUT/'topiary-row.scene.json',garden);c.call('load_scene',path=relative(OUT/'topiary-row.scene.json'))
        fit(c,[-4,-.03,-.7],[4,1.3,.7],OUT/'topiary-row.png',direction=(1,.22,1.2),width=1800,height=1000)
        c.call('save_scene',path=relative(OUT/'topiary-row.scene.json'));report['scenes']['topiary_row']=scene_stats(garden)
        report['macro_policy']='leaves rigidly oriented in local frame, mature left/flush right, no scaling; heads translated as a three-organ group'
        atomic_json(OUT/'presentation-report.json',report)
    finally:c.close()
    cards=''
    for f in FAMILIES:
        ref=f['reference'];name=html.escape(f['name']);id=f['id']
        cards+=f'<h2>{name}</h2><div class="pair"><img src="references/{ref}.png"><img src="{id}-detail.png"></div><div class="pair"><img src="references/{ref}-0.png"><img src="{id}-crop.png"></div><p>Órgão nativo isolado; enquadramento próprio, sem redimensionar a geometria:</p><img class="macro" src="{id}-macro.png">'
    (OUT/'references.html').write_text('<!doctype html><meta charset="utf-8"><title>Ignis · referência e candidato</title><style>body{background:#202921;color:#e4ecdd;font:16px system-ui;max-width:1500px;margin:auto;padding:30px}img{max-width:100%;display:block}.pair{display:grid;grid-template-columns:1fr 1fr;align-items:center;gap:20px;margin:20px 0}.macro{max-width:700px}a{color:#c7db9f}</style><h1>Sete referências · oito perfis procedurais</h1><p>Fotos fornecidas pelo usuário, mantidas com marcas/créditos. Comparação qualitativa: câmeras, escala e luz independentes; sem percentual de fidelidade. Nomes descritivos, espécies não confirmadas. Recortes sem resize/rotação, caixas nos manifestos.</p><p><a href="review.html">Galeria</a> · <a href="presentation-report.json">Câmeras, crops e contagens</a></p><h2>Plantios nativos</h2><img src="desert.png"><img src="topiary-row.png">'+cards)
    print(json.dumps({k:v for k,v in report.items() if k not in ['details','crops']},ensure_ascii=False,indent=2))


if __name__=='__main__':main()
