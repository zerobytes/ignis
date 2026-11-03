#!/usr/bin/env python3
"""Native chronological snapshots. Same camera/seed for8,20and mature years."""
import argparse,copy,itertools,json,math,time
from pathlib import Path
from verify_lab import Client,ROOT
import botany_math as bm
OUT=ROOT/'artifacts/botany-age'
NAMES={'palmeirinha':'Palmeirinha','platano':'Plátano','eucalipto':'Eucalipto','figueira':'Figueira ornamental','figueira_rs':'Figueira campestre · RS'}
def fit(lo,hi,width=1200,height=1100,direction=(.53,.12,.83)):
    center=[(a+b)/2 for a,b in zip(lo,hi)];direction=bm.unit(direction);right=bm.unit(bm.cross(bm.mul(direction,-1),bm.UP));up=bm.cross(right,bm.mul(direction,-1));tan=math.tan(math.radians(35/2))
    distance=max(bm.dot(bm.sub(p,center),direction)+1.30*max(abs(bm.dot(bm.sub(p,center),right))/(tan*width/height),abs(bm.dot(bm.sub(p,center),up))/tan) for p in itertools.product(*zip(lo,hi)))
    return dict(position=bm.add(center,bm.mul(direction,distance)),target=center,fov_degrees=35)
def main():
    global OUT
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--only',choices=list(NAMES));p.add_argument('--output',type=Path,default=OUT);a=p.parse_args();OUT=a.output.resolve();relative=OUT.relative_to(ROOT).as_posix()
    OUT.mkdir(parents=True,exist_ok=True);c=Client(a.binary,log_path=OUT/'runtime.log');report={};gallery=[]
    def publish(recipe):
        path=f'{relative}/candidate.ignis-botany';(ROOT/path).write_text(json.dumps(recipe,separators=(',',':')))
        start=time.monotonic();r=c.call('botany',operation={'kind':'load','path':path,'publish':True});r['publish_seconds']=time.monotonic()-start;return r
    def capture(name,w=1200,h=1100):c.call('capture',path=f'{relative}/{name}.png',width=w,height=h)
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'botany-age','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':40,'depth':40,'seed':193,'obstacles':0});initial=c.call('inspect');report['build_id']=initial['build_id']
        c.call('edit_scene',edits=[{'op':'remove','id':e['id']} for e in initial['scene']['entities']])
        ground=dict(id='ground',geometry=dict(positions=[[-150,-.045,-150],[150,-.045,-150],[-150,-.045,150],[150,-.045,150]],triangles=[[0,2,1],[1,2,3]]))
        instance=dict(id='ground',asset='ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.24,.27,.21])
        c.call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_asset','asset':ground}},{'op':'mesh','edit':{'kind':'put_instance','instance':instance}}])
        c.call('set_light',light={'direction':[.5,-.8,-.3],'ambient':.27,'direct_intensity':1.2,'environment':{'sky':[.5,.6,.7],'horizon':[.5,.5,.4],'ground':[.1,.14,.06],'axis':[0,1,0],'intensity':.5},'shadow':{'center':[0,10,0],'extent':60,'resolution':2048,'bias':.0005}})
        c.call('save_scene',path=f'{relative}/empty.scene.json')
        for key in ([a.only] if a.only else NAMES):
            c.call('load_scene',path=f'{relative}/empty.scene.json')
            base=json.loads((ROOT/f'assets/botany/{key}.ignis-botany').read_text());mature=base['settings']['age_years'];entry={}
            entry['mature_years']=mature;adult=publish(base);lo,hi=adult['compiled']['bounds_min'],adult['compiled']['bounds_max'];camera=fit(lo,hi)
            c.call('set_camera',camera=camera);capture(f'{key}-adult');entry[str(int(mature))]=adult
            c.call('set_camera',camera=fit(lo,hi,direction=(-.83,.18,.53)));capture(f'{key}-alternate');c.call('set_camera',camera=camera)
            # No time stepping is needed to define a complete chronological snapshot.
            physical=c.call('botany',operation={'kind':'physics','prefix':key,'action':{'kind':'start','settings':{'gravity':[0,0,0],'ground':None,'iterations':4}}})
            assert physical['attachment_error_m']<1e-4;entry['physical_rest']=physical
            c.call('botany',operation={'kind':'physics','prefix':key,'action':{'kind':'reset'}})
            for years,label in [(8,'young'),(20,'middle')]:
                start=time.monotonic();entry[str(years)]=c.call('botany',operation={'kind':'set_age','prefix':key,'age_years':years});entry[str(years)]['publish_seconds']=time.monotonic()-start;capture(f'{key}-{label}')
            c.call('botany',operation={'kind':'set_age','prefix':key,'age_years':mature});bare=copy.copy(base);bare['settings']={**base['settings'],'leaf_density':0};publish(bare);capture(f'{key}-structure')
            c.call('set_camera',camera=fit(lo,hi,direction=(-.83,.18,.53)));capture(f'{key}-structure-alternate');c.call('set_camera',camera=camera)
            report[key]=entry
            gallery.append((base,lo,hi));print(key,{age:entry[age]['compiled']['organs'] for age in ['8','20',str(int(mature))]},flush=True)
        if not a.only:
            c.call('load_scene',path=f'{relative}/empty.scene.json');x=0.;bounds=[]
            for base,lo,hi in gallery:
                offset=x-lo[0];r=copy.copy(base);r['settings']={**base['settings'],'position':[offset,0,0]};publish(r)
                bounds.append(([lo[0]+offset,lo[1],lo[2]],[hi[0]+offset,hi[1],hi[2]]));x+=hi[0]-lo[0]+2.
            lo=[min(b[0][k] for b in bounds) for k in range(3)];hi=[max(b[1][k] for b in bounds) for k in range(3)]
            c.call('set_camera',camera=fit(lo,hi,1920,1000));capture('gallery',1920,1000)
            c.call('save_scene',path=f'{relative}/gallery.scene.json');c.call('load_scene',path=f'{relative}/gallery.scene.json');capture('roundtrip',1920,1000)
            assert (OUT/'gallery.png').read_bytes()==(OUT/'roundtrip.png').read_bytes();report['roundtrip_pixels_equal']=True
        (OUT/('report.json' if not a.only else f'{a.only}-report.json')).write_text(json.dumps(report,indent=2))
        review(report,a.only)
    finally:c.close()
def review(report,only=None):
    import os
    html='''<!doctype html><meta charset="utf-8"><title>Estrutura e copa procedural</title>
<style>body{font:16px system-ui;background:#18202b;color:#edf1e9;max-width:1600px;margin:32px auto;padding:0 24px}a{color:#a3d8be}.row{display:grid;grid-template-columns:repeat(3,1fr);gap:12px}img{width:100%;display:block}figure{margin:0}p{max-width:1100px;line-height:1.5}h2{margin-top:48px}summary{cursor:pointer;margin:16px 0}figcaption{margin:10px 0}@media(max-width:900px){.row{grid-template-columns:1fr}}</style>
<h1>Estrutura e copa · exemplares procedurais</h1>
<p>Fotos para comparar silhueta, forquilhas, espessuras e ocupação da copa. Geometria gerada por regras, seed e curvas; capturas nativas do Ignis. Quantidade de folhas ou testes aprovados não prova semelhança. Os anos são perfis autorais, sem calibração para datar as árvores fotografadas.</p>
<p>Referências: <a href="https://www.windyridgegardencentre.ie/products/dypsis-lutescens-areca-160-cm">Areca, Windyridge</a>; <a href="https://trees.stanford.edu/ENCYC/PLATace.htm">Plátano, Sairus Patel / Stanford</a>; <a href="https://www.pinterest.com/pin/683139837211649809/">eucalipto, foto de estudo sem identificação confirmada</a>; <a href="https://www.gardenershq.com/Ficus-benjamina.php">Ficus, Forest e Kim Starr</a>.</p>'''
    refs={'palmeirinha':'palm.jpg','platano':'platano-stanford.jpg','eucalipto':'eucalipto.jpg','figueira':'figueira.jpg','figueira_rs':'../botany-rs/reference.png'}
    def picture(src,label):
        return f'<figure><a href="{src}"><img src="{src}" loading="lazy"></a><figcaption>{label}</figcaption></figure>'
    for key in ([only] if only else NAMES):
        entry=report[key];old=int(entry['mature_years'])
        ref=os.path.relpath(ROOT/'artifacts/botany-rs/reference.png' if key=='figueira_rs' else ROOT/'artifacts/botany-native/references'/refs[key],OUT)
        html+=f'<h2>{NAMES[key]}</h2><div class="row">'
        html+=picture(ref,'Foto de referência')+picture(f'{key}-adult.png',f'Render procedural · perfil de {old}anos')+picture(f'{key}-structure.png','Mesma planta sem folhas')+'</div>'
        html+='<details><summary>Segundo ângulo</summary><div class="row">'+picture(f'{key}-alternate.png','Copa')+picture(f'{key}-structure-alternate.png','Estrutura')+'</div></details>'
        before=ROOT/f'artifacts/botany-age/{key}-adult.png'
        if before.exists() and before.parent.resolve()!=OUT.resolve():
            html+='<details><summary>Antes desta revisão</summary><p>Enquadramentos individuais; comparação qualitativa da arquitetura.</p><div class="row">'+picture(os.path.relpath(before,OUT),'Versão anterior')+picture(f'{key}-adult.png','Versão corrente')+'</div></details>'
        html+='<details><summary>Desenvolvimento: mesma seed, escala e câmera</summary><div class="row">'
        for age,label in [(8,'young'),(20,'middle'),(old,'adult')]:
            c=entry[str(age)]['compiled'];height=c['bounds_max'][1]-c['bounds_min'][1]
            html+=picture(f'{key}-{label}.png',f'{age}anos · {height:.1f}m · {c["organs"]}órgãos presentes')
        html+='</div></details>'
    html+='<p>Folhas representam a folhagem corrente. A física é um ensaio autoral local; idade em anos e fase mecânica são controles diferentes. Anos aqui não simulam uma retrospectiva biológica de décadas. Relevo de troncos, raízes fundidas e semelhança artística fina continuam abertos.</p>'
    (OUT/'review.html').write_text(html)
if __name__=='__main__':main()
