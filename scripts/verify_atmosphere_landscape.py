#!/usr/bin/env python3
"""Real metre-scale native landscape, same surface in both paths."""
import argparse
import json
import math
from pathlib import Path
from png_pixels import rgba8
from verify_lab import Client, ROOT
from verify_materials import f32
from mountain_recipe import ridge

OUT = ROOT / 'artifacts/atmosphere-landscape'
PREFIX = 'artifacts/atmosphere-landscape/'


def write_viewer(report):
    # Only UI layout; the displayed pixels are untouched native PNGs.
    data = json.dumps(report, ensure_ascii=False).replace('</','<\\/')
    html = '''<!doctype html><meta charset="utf-8"><title>Ignis · Paisagem em quilômetros</title>
<style>body{margin:0;background:#10171d;color:#e6edf2;font:16px system-ui}main{max-width:1450px;margin:auto;padding:28px}h1{font-size:28px}p{line-height:1.5;color:#b8c7d2}button,select{font:inherit;padding:9px 14px;background:#253642;color:#eef5fa;border:1px solid #516574;border-radius:6px;margin:4px}#stage{overflow:auto;border:1px solid #516574;margin-top:16px}img{display:block;width:100%;height:auto}img.native{width:auto;max-width:none}.metrics{padding:14px;background:#1a2832}small{display:block;color:#b8c7d2;margin:12px 0}a{color:#95d5fa}</style>
<main><h1>Uma paisagem de 18 quilômetros</h1><p>Cinco cadeias a 1, 3, 6, 10 e 18 km em profundidade, medidas a partir da câmera inicial. Câmera a 1.200 m; dimensões e alturas em metros reais. Relevo sintético, atmosfera homogênea artística, sem curvatura terrestre ou streaming.</p>
<select id="cases"></select><button id="full">Geometria completa</button><button id="selected">Seleção automática</button><button id="zoom">Pixels 1:1</button>
<div id="metrics" class="metrics"></div><div id="stage"><img id="image"></div><small id="caption"></small>
<p>A referência usa a mesma atmosfera da seleção. Ar mais limpo pode exigir todos os detalhes. Resolução das capturas: 1728 × 1117 pixels físicos; a opção 1:1 evita redimensionamento CSS. 40 e 100 km não foram testados: o renderer atual tem plano distante de 30 km.</p>
<a href="landscape.scene.json">Cena nativa compilada</a> · <a href="verification.json">Métricas completas</a> · <a href="cost.json">Tempos de desenho</a>
<script>const report=REPORT;const cases=document.querySelector('#cases'),im=document.querySelector('#image');let optimized=true;
report.cases.forEach((c,i)=>{let o=document.createElement('option');o.value=i;o.textContent=c.label;cases.appendChild(o)});cases.value=1;
function draw(){const c=report.cases[+cases.value],s=optimized?c.selected:c.full;im.src=c.id+(optimized?'-selected.png':'-full.png');im.alt=c.label+' — '+(optimized?'seleção automática':'referência completa');document.querySelector('#metrics').textContent=(optimized?'Seleção automática':'Referência completa')+' · '+(s.detailed_triangles+s.cheap_triangles).toLocaleString('pt-BR')+' triângulos · '+((s.selected_geometry_bytes??s.selected_vertex_bytes)/1048576).toFixed(2)+' MiB de geometria (vértices + índices) · '+s.substituted_instances+' instâncias substituídas · diferença máxima '+c.max_rgb8+'/255';document.querySelector('#caption').textContent=c.label+' · densidade '+c.density+' /m · metade do contraste a '+(Math.log(2)/c.density/1000).toFixed(2)+' km';}
cases.onchange=draw;document.querySelector('#full').onclick=()=>{optimized=false;draw()};document.querySelector('#selected').onclick=()=>{optimized=true;draw()};document.querySelector('#zoom').onclick=()=>im.classList.toggle('native');draw();</script></main>'''
    (OUT/'index.html').write_text(html.replace('REPORT',data))
    static = '# Paisagem em quilômetros\n\nCâmera inicial a 1.200 m. Centros das cadeias a 1, 3, 6, 10 e 18 km em profundidade horizontal. Metros reais, relevo sintético e atmosfera homogênea artística.\n\n'
    for case in report['cases']:
        static += f"## {case['label']}\n\n![{case['label']}]({OUT / (case['id']+'-selected.png')})\n\n"
        stats = case['selected']
        static += f"Geometria GPU alocada: {stats.get('allocated_geometry_bytes',stats['allocated_vertex_bytes'])/1048576:.2f} MiB. Diferença máxima: {case['max_rgb8']}/255. {stats['substituted_instances']} instâncias com preenchimento barato; {stats['geometric_lod_instances']} com geometria reduzida.\n\n"
    static += 'Resolução1728×1117pixels físicos. Modelo plano, sem curvatura terrestre ou streaming.40/100km não foram testados; o renderer atual termina em30km.\n'
    (OUT/'comparison.md').write_text(static)


def verify(binary, draft=False):
    OUT.mkdir(parents=True,exist_ok=True)
    c=Client(binary)
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='landscape-scale',version='1')))
        c.request('notifications/initialized',notification=True)
        c.call('generate',recipe=dict(width=16,depth=14,seed=17,obstacles=0))
        entities=c.call('inspect')['scene']['entities']
        c.call('edit_scene',edits=[dict(op='upsert',entity=dict(e,visible=False)) for e in entities])
        assets=[ridge('range-a',.3),ridge('range-b',1.7)]
        for a in assets:c.call('edit_mesh',edit=dict(kind='put_asset',asset=a))
        chains=[]
        for i,(distance,scale) in enumerate(zip([1000,3000,6000,10000,18000],[1,1.4,2.2,3.3,5])):
            a=assets[i%2]
            instance=f32(dict(id=f'range-{i}',asset=a['id'],position=[0,0,9000-distance],
                rotation=[0,math.sin((i-2)*.025),0,math.cos((i-2)*.025)],scale=scale,
                color=[.24+i*.025,.31+i*.005,.18+i*.018],playback=None))
            c.call('edit_mesh',edit=dict(kind='put_instance',instance=instance))
            chains.append(dict(id=instance['id'],depth_m=distance,base_distance_m=math.hypot(distance,1200),
                local_width_scaled_m=4000*scale,local_depth_scaled_m=1600*scale,
                summit_height_m=max(p[1] for p in a['geometry']['positions'])*scale,instance=instance))
        c.call('edit_mesh',edit=dict(kind='put_asset',asset=dict(id='ground',geometry=dict(
            positions=[[-10000,-.1,-10000],[10000,-.1,-10000],[10000,-.1,10000],[-10000,-.1,10000]],
            triangles=[[0,2,1],[0,3,2]]))))
        c.call('edit_mesh',edit=dict(kind='put_instance',instance=dict(id='ground',asset='ground',
            position=[0,0,0],rotation=[0,0,0,1],scale=3,color=[.17,.23,.12],playback=None)))
        camera=dict(position=[0,1200,9000],target=[0,1300,0],fov_degrees=55)
        light=dict(direction=[.65,-1,-.25],ambient=.25,direct_intensity=1,
            atmosphere=dict(density=.00006,color=[.44,.59,.75],sun_color=[.95,.73,.40],sun_strength=.15,
                layers=dict(sky_color=[.73,.82,.94],mountain_color=[.008,.025,.055],detail_start=800,detail_end=3200,influence=1),
                reduce_details=True,silhouette_error_px=.5,color_error=.002))
        c.call('set_camera',camera=camera);c.call('set_light',light=f32(light))
        c.call('edit_scene',edits=[dict(op='compile_distant_meshes')])
        c.call('save_scene',path=PREFIX+'landscape.scene.json')
        variants=[('clear','Pouca influência · mais detalhes',.000025,camera,[.65,-1,-.25]),
            ('haze','Céu claro · silhuetas azuis',.00006,camera,[.65,-1,-.25]),
            ('mist','Maior influência · detalhes somem antes',.0001,camera,[.65,-1,-.25]),
            ('move','Câmera lateral e sol mudando',.00006,
                dict(position=[250,1000,8500],target=[0,1300,0],fov_degrees=55),[-.6,-.8,.35]),
            ('high','Vista elevada',.00006,
                dict(position=[0,2000,9000],target=[0,1300,0],fov_degrees=55),[.65,-1,-.25]),
            ('sunset','Pôr do sol · montanhas douradas',.00006,camera,[-.3,-.15,1]),
            ('purple','Mundo roxo',.00006,camera,[.65,-1,-.25])]
        cases=[]
        for id,label,density,cam,direction in variants:
            if draft and id!='haze':continue
            c.call('set_camera',camera=cam)
            case_light=f32(dict(light,direction=direction,atmosphere=dict(light['atmosphere'],density=density)))
            if id=='clear':
                case_light['atmosphere']['layers'].update(influence=.25)
            if id=='mist':
                case_light['atmosphere']['layers'].update(detail_start=400,detail_end=2000)
            if id=='sunset':
                case_light['atmosphere'].update(color=[.60,.35,.13],sun_color=[1,.72,.28],sun_strength=.55)
                case_light['atmosphere']['layers'].update(sky_color=[.97,.82,.52],mountain_color=[.045,.024,.012])
            if id=='purple':
                case_light['atmosphere'].update(color=[.40,.20,.53],sun_color=[.9,.55,.78],sun_strength=.25)
                case_light['atmosphere']['layers'].update(sky_color=[.78,.64,.90],mountain_color=[.03,.008,.055])
            metadata=[]
            for enabled in [False,True]:
                case_light['atmosphere']['reduce_details']=enabled
                c.call('set_light',light=case_light)
                suffix='selected' if enabled else 'full'
                capture=c.call('capture',path=PREFIX+id+'-'+suffix+'.png',width=1728,height=1117)
                metadata.append(capture['adapter']['distant_selection'])
            a=(OUT/(id+'-full.png')).read_bytes();b=(OUT/(id+'-selected.png')).read_bytes()
            if a==b:maximum=mean=changed=0
            else:
                aa=rgba8(a)[2];bb=rgba8(b)[2]
                differences=[abs(x-y) for i,(x,y) in enumerate(zip(aa,bb)) if i%4!=3]
                maximum=max(differences);mean=sum(differences)/len(differences)
                changed=sum(x>0 for x in differences)/len(differences)
            cases.append(dict(id=id,label=label,density=density,camera=cam,light=case_light,
                full=metadata[0],selected=metadata[1],max_rgb8=maximum,mae_rgb8=mean,changed_channels_fraction=changed))
            print(json.dumps(dict(case=id,max_rgb8=maximum,substituted=metadata[1]['substituted_instances'],
                geometric_lods=metadata[1]['geometric_lod_instances'])),flush=True)
            assert maximum<=3,(id,maximum)
        motion=[]
        if not draft:
            # Moving camera/light through the transition: overwrite current images,
            # retain measurements and the worst pair, not a history of captures.
            worst=-1
            for i in range(12):
                t=i/11*2*math.pi
                cam=dict(position=[400*math.sin(t),1200+150*math.sin(t),7000+2000*math.cos(t)],target=[0,1300,0],fov_degrees=55)
                ml=f32(dict(light,direction=[.65*math.cos(t),-1,-.25+.3*math.sin(t)]))
                c.call('set_camera',camera=f32(cam))
                stats=[]
                for enabled in [False,True]:
                    ml['atmosphere']['reduce_details']=enabled
                    c.call('set_light',light=ml)
                    suffix='selected' if enabled else 'full'
                    stats.append(c.call('capture',path=PREFIX+'motion-'+suffix+'.png',width=1728,height=1117)['adapter']['distant_selection'])
                a=(OUT/'motion-full.png').read_bytes();b=(OUT/'motion-selected.png').read_bytes()
                aa=rgba8(a)[2];bb=rgba8(b)[2]
                differences=[abs(x-y) for j,(x,y) in enumerate(zip(aa,bb)) if j%4!=3]
                maximum=max(differences);mean=sum(differences)/len(differences)
                motion.append(dict(frame=i,camera=cam,max_rgb8=maximum,mae_rgb8=mean,selected=stats[1]))
                if maximum>worst:
                    (OUT/'motion-worst-full.png').write_bytes(a);(OUT/'motion-worst-selected.png').write_bytes(b);worst=maximum
                assert maximum<=3,('motion',i,maximum)
            print(json.dumps(dict(motion_frames=len(motion),max_rgb8=worst)),flush=True)
        report=dict(build_id=c.call('inspect')['build_id'],width=1728,height=1117,chains=chains,cases=cases,motion=motion,
            camera=camera,scene=PREFIX+'landscape.scene.json',passed=True,renderer_far_m=30000,
            limitations=['synthetic relief','homogeneous artistic atmosphere','flat world/no Earth curvature',
                'finite local scene, no streaming','source CPU retained','40/100km not tested'])
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        if not draft:write_viewer(report)
    finally:c.close()


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--draft',action='store_true')
    args=p.parse_args();verify(args.binary.resolve(),args.draft)
