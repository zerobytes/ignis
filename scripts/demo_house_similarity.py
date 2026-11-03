#!/usr/bin/env python3
"""A coarse authoring hypothesis and native comparison against the supplied house."""
import argparse
import json
import math
import shutil
from pathlib import Path
from verify_lab import Client

ROOT = Path(__file__).resolve().parents[1]


def box():
    p = [[x,y,z] for x in [-.5,.5] for y in [-.5,.5] for z in [-.5,.5]]
    t = [[0,1,3],[0,3,2],[4,6,7],[4,7,5],[0,4,5],[0,5,1],
         [2,3,7],[2,7,6],[0,2,6],[0,6,4],[1,5,7],[1,7,3]]
    return dict(positions=p,triangles=t)


def roof():
    # One general sampled surface, authored in metres. Curvature is a hypothesis.
    p=[]; t=[]; steps=24
    for x in [-5.1,3.5]:
        for j in range(steps+1):
            z=-2.9+5.8*j/steps
            height=3.65+2.25*(1-abs(z/2.9))**1.65+.16*abs(z/2.9)**8
            p.append([x,height,z])
    for j in range(steps):
        t.extend([[j,j+1,steps+1+j],[j+1,steps+2+j,steps+1+j]])
    return dict(positions=p,triangles=t)


def radial(radius,bottom,top,sides=10):
    p=[];t=[]
    for y in [bottom,top]:
        for j in range(sides):
            a=math.tau*j/sides;p.append([radius*math.cos(a),y,radius*math.sin(a)])
    for j in range(sides):
        k=(j+1)%sides;t.extend([[j,j+sides,k],[k,j+sides,k+sides]])
    for j in range(1,sides-1):
        t.extend([[0,j,j+1],[sides,sides+j+1,sides+j]])
    return dict(positions=p,triangles=t)


def bay_roof():
    p=[[3.05,4.7,1.75]];t=[];sides=16
    for j in range(sides):
        a=math.tau*j/sides;p.append([3.05+1.65*math.cos(a),3.55,1.75+1.65*math.sin(a)])
    for j in range(sides):t.append([0,(j+1)%sides+1,j+1])
    return dict(positions=p,triangles=t)


def run(binary):
    out=ROOT/'artifacts/house-similarity';(out/'artifacts').mkdir(parents=True,exist_ok=True)
    (out/'examples').mkdir(exist_ok=True)
    shutil.copyfile(ROOT/'docs/references/image-to-3d/elven-house.png',out/'artifacts/reference.png')
    # Client root is explicit and isolated, as in the regression runner.
    import verify_lab
    verify_lab.ROOT=out
    c=Client(binary)
    call=c.call
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'house-similarity','version':'1'}})
        c.request('notifications/initialized',notification=True)
        call('generate',recipe={'seed':7,'width':20,'depth':20,'obstacles':0})
        scene=call('inspect')['scene']
        edits=[{'op':'remove','id':e['id']} for e in scene['entities'] if e['id']!='player']
        player=next(e for e in scene['entities'] if e['id']=='player');player['visible']=False;player['transform']['position']=[0,-100,0]
        edits.append({'op':'upsert','entity':player})
        call('edit_scene',edits=edits)
        def asset(id,geometry):call('edit_mesh',edit={'kind':'put_asset','asset':{'id':id,'geometry':geometry}})
        def instance(id,asset,position,color):
            call('edit_mesh',edit={'kind':'put_instance','instance':{'id':id,'asset':asset,'position':position,'rotation':[0,0,0,1],'scale':1,'color':color}})
        stone=[.65,.59,.46]; roof_color=[.14,.19,.18]
        geometry={'positions':[],'triangles':[]}
        def component(pos,size):
            nonlocal geometry
            # All masonry uses a single asset; independent roof/door frames remain distinct.
            base=len(geometry['positions']);g=box()
            geometry['positions'].extend([[p[i]*size[i]+pos[i] for i in range(3)] for p in g['positions']])
            geometry['triangles'].extend([[base+i for i in tri] for tri in g['triangles']])
        component([-.15,.25,.1],[10,.5,6.2])
        component([1.2,2.,-.2],[4.5,3.,4.7])
        component([-3.1,.6,.25],[4.4,.3,5.7])
        for x in [-4.8,-3.55,-2.3,-1.05]:
            component([x,2.15,2.65],[.26,2.8,.26])
            component([x,.82,2.65],[.45,.18,.45])
            component([x,3.55,2.65],[.45,.18,.45])
        component([-2.95,1.2,2.7],[4.3,.14,.22])
        for x in [-4.65,-4.15,-3.65,-3.15,-2.65,-2.15,-1.65,-1.15]:
            component([x,.98,2.7],[.08,.45,.12])
        for j in range(3):component([.2,.15+.1*j,3.0-.3*j],[2.5,.3+.2*j,.8])
        component([-3.1,5.5,-.3],[.65,1.8,.65])
        for j in range(8):
            angle=math.tau*j/8
            component([3.05+1.48*math.cos(angle),2.2,1.75+1.48*math.sin(angle)],[.16,2.5,.16])
        asset('masonry',geometry);instance('masonry','masonry',[0,0,0],stone)
        asset('roof',roof());instance('roof','roof',[0,0,0],roof_color)
        gable={'positions':[],'triangles':[]}
        for j in range(25):
            z=-2.9+5.8*j/24
            y=3.65+2.25*(1-abs(z/2.9))**1.65+.16*abs(z/2.9)**8-.03
            gable['positions'].extend([[3.48,3.5,z],[3.48,y,z]])
            if j<24:
                k=j*2;gable['triangles'].extend([[k,k+1,k+2],[k+1,k+3,k+2]])
        asset('gable',gable);instance('gable','gable',[0,0,0],stone)
        asset('bay',radial(1.45,.5,3.45));instance('bay','bay',[3.05,0,1.75],stone)
        asset('bay-roof',bay_roof());instance('bay-roof','bay-roof',[0,0,0],roof_color)
        # Door and glass are only positioning cues at this stage.
        asset('door',{'positions':[[-.65,.6,0],[.65,.6,0],[.65,2.55,0],[0,3.2,0],[-.65,2.55,0]],'triangles':[[0,1,2],[0,2,3],[0,3,4]]})
        instance('door','door',[.15,0,2.165],[.2,.13,.065])
        asset('glass',radial(1.47,1.15,3.1));instance('glass','glass',[3.05,0,1.75],[.18,.22,.18])
        floor=box();floor['positions']=[[p[0]*180,p[1]*.1-.1,p[2]*180] for p in floor['positions']]
        asset('ground',floor);instance('ground','ground',[0,0,0],[.64,.59,.51])
        # Large upright plane is an authored studio background, not a composited image.
        asset('backdrop',{'positions':[[-1000,-10,-45],[1000,-10,-45],[1000,1000,-45],[-1000,1000,-45]],'triangles':[[0,1,2],[0,2,3]]})
        instance('backdrop','backdrop',[0,0,0],[.67,.62,.54])
        light={'direction':[-.5,-1,-.4],'ambient':.4,'shadow':{'center':[0,2,0],'extent':12,'resolution':1024,'bias':.002}}
        call('set_light',light=light)
        camera={'position':[13,8,16],'target':[-.1,2.0,.25],'fov_degrees':27}
        call('set_camera',camera=camera)
        call('save_scene',path='examples/house-blockout.scene.json')
        capture=call('capture',path='artifacts/blockout.png',width=1536,height=1024)
        regions=[{'id':'roof','x':75,'y':65,'width':1400,'height':410},
                 {'id':'veranda','x':100,'y':425,'width':620,'height':435},
                 {'id':'door','x':715,'y':415,'width':225,'height':410},
                 {'id':'bay','x':980,'y':440,'width':485,'height':425}]
        initial=call('compare_images',reference_path='artifacts/reference.png',candidate_path='artifacts/blockout.png',regions=regions,difference_path='artifacts/difference.png')
        call('set_camera',camera={**camera,'position':[15,8,16]})
        shifted_capture=call('capture',path='artifacts/camera-shift.png',width=1536,height=1024)
        shifted=call('compare_images',reference_path='artifacts/reference.png',candidate_path='artifacts/camera-shift.png',regions=regions)
        call('set_camera',camera=camera)
        call('save_replay',path='artifacts/house-blockout.replay.json')
        report={'scope':'Comparator exercise with structurally insufficient geometry; not a faithful reconstruction candidate',
                'reference':'Original supplied image, no transformation','regions':'Manually annotated reference rectangles, not automatic segmentation',
                'capture':capture,'camera_shift_capture':shifted_capture,'comparison':initial,'camera_shift_comparison':shifted,
                'hypotheses':['Metre scale chosen, not measured','Perspective camera chosen by inspection','Body/roof/bay approximations overlap; not a finished closed building',
                              'No tracery/tiles/vegetation; studio light/background differ from reference'],
                'automated_alignment':False,'geometry_reconstructed_automatically':False}
        (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
        shutil.copyfile(out/'examples/house-blockout.scene.json',ROOT/'examples/house-blockout.scene.json')
        reference_src='artifacts/reference.png';candidate_src='artifacts/blockout.png'
        def panel(title,src):
            rects=''.join(f'<rect x="{r["x"]}" y="{r["y"]}" width="{r["width"]}" height="{r["height"]}"/><text x="{r["x"]+8}" y="{r["y"]+26}">{r["id"]}</text>' for r in regions)
            return f'<section><h2>{title}</h2><div class="image"><img src="{src}"><svg viewBox="0 0 1536 1024">{rects}</svg></div></section>'
        rows=''.join(f'<tr><td>{r["region"]["id"]}</td><td>{r["metrics"]["rgb_mae"]:.4f}</td><td>{r["metrics"]["ssim_luma"]:.4f}</td></tr>' for r in initial['report']['regions'])
        html=f'''<!doctype html><meta charset="utf-8"><title>Casa — primeiro ensaio de comparação</title>
<style>body{{background:#171a1e;color:#eee;font:16px system-ui;margin:32px;max-width:1500px}}h1{{font-size:28px}}.grid{{display:grid;grid-template-columns:1fr 1fr;gap:20px}}img{{width:100%;display:block}}.image{{position:relative}}svg{{position:absolute;inset:0;width:100%;height:100%;pointer-events:none}}rect{{fill:none;stroke:#00dfd7;stroke-width:2}}text{{fill:#002c2a;stroke:white;stroke-width:.8;font:24px system-ui}}table{{border-collapse:collapse}}td,th{{padding:10px 20px;border-bottom:1px solid #555;text-align:left}}.note{{background:#292e35;padding:16px}}h2{{font-size:18px}}</style>
<h1>Casa élfica — ensaio do comparador</h1><p class="note"><strong>Geometria insuficiente como reconstrução.</strong> Faltam formas, arcos, vazios e relações dominantes, mesmo ignorando textura e cor. A câmera é manual e não foi calibrada. Este ensaio verifica o fluxo de comparação; as métricas não são porcentagens de fidelidade 3D.</p>
<div class="grid">{panel('Referência original',reference_src)}{panel('Volumes na Ignis',candidate_src)}</div>
<h2>Regiões anotadas na referência</h2><table><tr><th>Região</th><th>Erro RGB médio ↓</th><th>SSIM de luminância ↑</th></tr>{rows}</table>
<p>O erro RGB vai de0 a1. SSIM vai de-1 a1. As medidas incluem diferenças de luz, cor e câmera; não identificam a causa de uma diferença.</p>
<details><summary>Mapa de erro — escala fixa0 preto/255 branco</summary><img src="artifacts/difference.png"></details>
<h2>Controle: outra câmera, mesma geometria</h2><table><tr><th>Câmera</th><th>Erro RGB médio ↓</th><th>SSIM ↑</th></tr><tr><td>Base manual</td><td>{initial['report']['metrics']['rgb_mae']:.4f}</td><td>{initial['report']['metrics']['ssim_luma']:.4f}</td></tr><tr><td>Deslocada em X</td><td>{shifted['report']['metrics']['rgb_mae']:.4f}</td><td>{shifted['report']['metrics']['ssim_luma']:.4f}</td></tr></table><p>O ranking pode discordar entre métricas. Esse ensaio não calibra a câmera automaticamente.</p><p><a href="report.json">Relatório completo, câmera e hipóteses</a></p>'''
        (out/'index.html').write_text(html)
        print(json.dumps({'report':str(out/'index.html'),'metrics':initial['report']['metrics'],'regions':initial['report']['regions']},indent=2))
    finally:c.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',type=Path,required=True)
    run(parser.parse_args().binary)
