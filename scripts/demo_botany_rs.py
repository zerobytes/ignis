#!/usr/bin/env python3
"""Isolated native review of the user's RS fig reference. No GUI/world writes."""
import argparse,copy,hashlib,json,math,time
from pathlib import Path
from verify_lab import Client,ROOT
from demo_botany_age import fit
import botany_math as bm

OUT=ROOT/'artifacts/botany-rs'
REGIONS=dict(trunk=(280,480,370,310),layers=(340,120,620,490),hanging=(630,390,300,230),top=(380,50,400,230))

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    parser.add_argument('--structure-only',action='store_true')
    args=parser.parse_args();OUT.mkdir(parents=True,exist_ok=True)
    source_bytes=(ROOT/'assets/botany/figueira_rs.ignis-botany').read_bytes();base=json.loads(source_bytes)
    stem_beards=[o['id'] for o in base['source']['organs'] if '/epifita' in o['id'] and '/fio' not in o['id']]
    c=Client(args.binary,log_path=OUT/'runtime.log');report={'source_sha256':hashlib.sha256(source_bytes).hexdigest()}
    def publish(r):
        path='artifacts/botany-rs/candidate.ignis-botany';(ROOT/path).write_text(json.dumps(r,separators=(',',':')))
        start=time.monotonic();v=c.call('botany',operation={'kind':'load','path':path,'publish':True});v['seconds']=time.monotonic()-start;return v
    def capture(name,camera):
        c.call('set_camera',camera=camera)
        c.call('capture',path=f'artifacts/botany-rs/{name}.png',width=1000,height=1000)
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'botany-rs','version':'1'}})
        c.request('notifications/initialized',notification=True)
        schema=c.request('tools/list')['result'];encoded=json.dumps(schema)
        for name in ['fork','sheath','blade_samples','compare_images']:assert name in encoded,name
        (OUT/'native-schema.json').write_text(json.dumps(schema,indent=2))
        c.call('generate',recipe={'width':60,'depth':60,'seed':195,'obstacles':0})
        state=c.call('inspect');report['build_id']=state['build_id']
        c.call('edit_scene',edits=[{'op':'remove','id':e['id']} for e in state['scene']['entities']])
        ground=dict(id='ground',geometry=dict(positions=[[-150,-.07,-150],[150,-.07,-150],[-150,-.07,150],[150,-.07,150]],triangles=[[0,2,1],[1,2,3]]))
        instance=dict(id='ground',asset='ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.20,.29,.09])
        c.call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_asset','asset':ground}},{'op':'mesh','edit':{'kind':'put_instance','instance':instance}}])
        light=dict(direction=[.4,-.85,.25],ambient=.26,direct_intensity=1.35,
            environment={'sky':[.52,.64,.78],'horizon':[.65,.67,.53],'ground':[.10,.16,.04],'axis':[0,1,0],'intensity':.5},
            shadow={'center':[0,7,0],'extent':60,'resolution':2048,'bias':.0005},
            atmosphere={'density':0.,'reduce_details':False,'color':[.5,.6,.7],'sun_color':[1.,.9,.7],'sun_strength':0.,
                'layers':{'sky_color':[.43,.60,.83],'mountain_color':[.43,.60,.83],'detail_start':10000.,'detail_end':20000.,'influence':0.}})
        c.call('set_light',light=light)
        c.call('save_scene',path='artifacts/botany-rs/empty.scene.json')
        structural=copy.deepcopy(base);structural['settings']['leaf_density']=0
        structural['settings']['bark_color']=[.43,.43,.43];structural['pruned']=stem_beards
        bare=publish(structural);lo=bare['compiled']['bounds_min'];hi=bare['compiled']['bounds_max']
        whole=fit(lo,hi,1000,1000,direction=(.08,-.08,1.))
        center=[(a+b)*.5 for a,b in zip(lo,hi)];center[1]=lo[1]+.42*(hi[1]-lo[1]);d=bm.unit((.08,-.08,1.));distance=(hi[1]-lo[1])/(2*math.tan(math.radians(17.5)))*1.50
        reference_camera=dict(position=bm.add(center,bm.mul(d,distance)),target=center,fov_degrees=35)
        cameras={'reference':reference_camera,'whole':whole,
            'side':fit(lo,hi,1000,1000,direction=(1.,.12,.10)),
            'back':fit(lo,hi,1000,1000,direction=(-.1,.12,-1.)),
            'top':fit(lo,hi,1000,1000,direction=(.03,1.,.06))}
        report.update(structure=bare,cameras=cameras,view_policy='manual, uncalibrated; reference view fits height, can crop sides like the supplied photo; auxiliary views show the full3D hypothesis')
        for name,camera in cameras.items():capture('structure-'+name,camera)
        c.call('save_scene',path='artifacts/botany-rs/structure.scene.json')
        (OUT/'structure-report.json').write_text(json.dumps(report,indent=2))
        if args.structure_only:print(json.dumps(bare['compiled'],indent=2));return
        adult=publish(base);report['adult']=adult
        for name,camera in cameras.items():capture('adult-'+name,camera)
        # Isolate the pendant bodies without mistaking them for structural wood.
        absent=copy.deepcopy(base);absent['pruned']=stem_beards;publish(absent);capture('without-pendants',reference_camera)
        publish(base)
        rest=c.call('botany',operation={'kind':'physics','prefix':'figueira_rs','action':{'kind':'start','settings':{'gravity':[0,0,0],'ground':None,'wind':[.5,0,0],'iterations':4}}})
        stepped=c.call('botany',operation={'kind':'physics','prefix':'figueira_rs','action':{'kind':'step','ticks':4}})
        assert stepped['attachment_error_m']<1e-4 and stepped['node_identity']==rest['node_identity']
        pruned=c.call('botany',operation={'kind':'physics','prefix':'figueira_rs','action':{'kind':'prune','organ':stem_beards[0]}})
        assert pruned['removed_mass_kg']>0 and pruned['node_identity']==rest['node_identity']
        assert abs(pruned['mass_kg']+pruned['removed_mass_kg']-pruned['initial_mass_kg']-pruned['added_mass_kg'])<1e-8
        report['physics']={'rest':rest,'step':stepped,'prune':pruned}
        c.call('botany',operation={'kind':'physics','prefix':'figueira_rs','action':{'kind':'reset'}})
        c.call('set_camera',camera=reference_camera);c.call('save_scene',path='artifacts/botany-rs/figueira.scene.json')
        c.call('load_scene',path='artifacts/botany-rs/figueira.scene.json');capture('roundtrip',reference_camera)
        assert (OUT/'adult-reference.png').read_bytes()==(OUT/'roundtrip.png').read_bytes();report['roundtrip_pixels_equal']=True
        regions=[dict(id=name,x=x,y=y,width=w,height=h) for name,(x,y,w,h) in REGIONS.items()]
        report['appearance_difference']=c.call('compare_images',reference_path='artifacts/botany-rs/reference.png',candidate_path='artifacts/botany-rs/adult-reference.png',regions=regions,difference_path='artifacts/botany-rs/difference.png')
        (OUT/'report.json').write_text(json.dumps(report,indent=2))
        review(report)
        print(json.dumps({'compiled':adult['compiled'],'nodes':rest['nodes'],'constraints':rest['constraints'],'roundtrip':True},indent=2))
    finally:c.close()

def review(report):
    c=report['adult']['compiled'];physics=report['physics']['rest']
    metrics=report['appearance_difference']['report']['metrics']
    html='''<!doctype html><meta charset="utf-8"><title>Figueira campestre · RS</title>
<style>body{font:16px system-ui;background:#172126;color:#edf2e7;max-width:1450px;margin:32px auto;padding:0 24px;line-height:1.5}a{color:#9bd8b9}.row{display:grid;grid-template-columns:repeat(2,1fr);gap:16px}img,svg{width:100%;display:block}figure{margin:0}figcaption{margin:10px 0}summary{cursor:pointer;margin:18px 0}h2{margin-top:40px}@media(max-width:800px){.row{grid-template-columns:1fr}}</style>
<h1>Figueira campestre · referência RS</h1>
<p>Exemplar adicional, gerado por parâmetros e seed no mesmo sistema. A foto fornecida pelo usuário orienta a arquitetura; não é usada como textura ou geometria. Espécie, escala e idade reais não foram confirmadas. O perfil de100anos é uma escolha autoral.</p>
<div class="row"><figure><a href="reference.png"><img src="reference.png"></a><figcaption>Referência enviada pelo usuário</figcaption></figure><figure><a href="adult-reference.png"><img src="adult-reference.png"></a><figcaption>Render nativo · vista aproximada, não calibrada</figcaption></figure></div>
<p>Há tronco curto/inclinado, braços laterais, massas em alturas distintas e filamentos pendentes. Permanecem divergentes a seção basal sulcada/fundida, a naturalidade de algumas curvas, a compactação dos patamares e a aparência felpuda dos pendentes. A comparação não é uma aprovação de réplica exata.</p>'''
    html+=f'<p>{c["organs"]:,}órgãos · {c["vertices"]:,}vértices · {c["triangles"]:,}triângulos. Física autoral: {physics["nodes"]:,}pontos e {physics["constraints"]:,}restrições; não é medida de FPS.</p>'
    for name,label in [('whole','Árvore inteira'),('side','Lateral'),('back','Verso hipotético'),('top','Superior')]:
        html+=f'<details><summary>{label}: corpo e copa</summary><div class="row"><img loading="lazy" src="structure-{name}.png"><img loading="lazy" src="adult-{name}.png"></div></details>'
    html+='<details><summary>Efeito dos pendentes</summary><div class="row"><figure><img loading="lazy" src="without-pendants.png"><figcaption>Sem pendentes</figcaption></figure><figure><img loading="lazy" src="adult-reference.png"><figcaption>Com pendentes</figcaption></figure></div></details>'
    for name,label in [('trunk','Base e tronco'),('layers','Patamares e vãos'),('hanging','Formações pendentes'),('top','Topo')]:
        # Read directly from the current captures; no stale, manually cut images.
        box=' '.join(str(v) for v in REGIONS[name])
        html+=f'<h2>{label}</h2><div class="row">'
        for source in ['reference.png','adult-reference.png']:
            html+=f'<svg viewBox="{box}" role="img" aria-label="{label}"><image href="{source}" width="1000" height="1000"/></svg>'
        html+='</div>'
    html+=f'<details><summary>Diferença de aparência RGB</summary><p>MAE normalizado {metrics["rgb_mae"]:.4f}; SSIM luminância {metrics["ssim_luma"]:.4f}. Sem alinhamento/segmentação, mesmas caixas1000×1000; inclui luz, fundo, perspectiva e elementos não reconstruídos. Não é porcentagem de fidelidade.</p><img loading="lazy" src="difference.png"></details>'
    html+='<p><a href="../../docs/botany-rs.md">Contrato, análise e validação</a> · <a href="../botany-architecture/review.html">Exemplares anteriores</a></p>'
    (OUT/'review.html').write_text(html)

if __name__=='__main__':main()
