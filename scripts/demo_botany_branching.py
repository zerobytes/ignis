#!/usr/bin/env python3
"""Native before/after branch review. All scene writes stay in an isolated MCP process."""
import argparse,copy,hashlib,json,math,subprocess,time
from pathlib import Path
from verify_lab import Client,ROOT
from demo_botany_age import fit
import botany_math as bm

OUT=ROOT/'artifacts/botany-branching'
BASELINE='099f5d6'
NAMES={'figueira_rs':'Figueira campestre · RS','figueira':'Figueira ornamental','platano':'Plátano','eucalipto':'Eucalipto'}

def stats(recipe):
    organs=recipe['source']['organs'];depth={};woody=[];leaves=[];pendants=[]
    for o in organs:
        depth[o['id']]=depth.get(o['parent'],-1)+1
        if o['shape']['kind']=='blade':leaves.append(o)
        elif '/epifita' in o['id']:pendants.append(o)
        else:woody.append(o)
    hosts={o['parent'] for o in leaves}
    structural=[o for o in woody if '/broto' not in o['id'] and not o['id'].endswith('/raiz')]
    returns=0
    for o in structural:
        path=o['path'];chord=bm.unit(bm.sub(path[-1],path[0]))
        returns+=any(bm.dot(bm.sub(c,a),chord)<-1e-8 for a,c in zip(path,path[1:]))
    trunk=organs[0]['shape']['radius']
    return dict(organs=len(organs),woody_axes=len(woody),structural_axes=sum('/broto' not in o['id'] and not o['id'].endswith('/raiz') for o in woody),
        shoot_axes=sum('/broto' in o['id'] for o in woody),pendants=len(pendants),leaves=len(leaves),foliage_hosts=len(hosts),
        mean_leaves_per_host=len(leaves)/len(hosts),max_woody_depth=max(depth[o['id']] for o in woody),
        shoot_orders=max(o['id'].count('/ramo') for o in woody),reiteration_hosts=sum(o['id'].endswith('/broto_interior') for o in woody),structural_source_returns=returns,
        radius_bands={str(r):sum(o['shape']['radius']>=trunk*r for o in woody) for r in [.1,.03,.01]})

def bare(recipe):
    r=copy.deepcopy(recipe);r['settings']['leaf_density']=0.;r['settings']['bark_color']=[.43,.43,.43]
    r['pruned']=[o['id'] for o in r['source']['organs'] if '/epifita' in o['id'] and '/fio' not in o['id']]
    return r

def main():
    p=argparse.ArgumentParser();p.add_argument('--phase',choices=['structure','appearance'],default='structure');p.add_argument('--only',choices=list(NAMES));p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');a=p.parse_args()
    OUT.mkdir(parents=True,exist_ok=True);report_path=OUT/'report.json';report=json.loads(report_path.read_text()) if report_path.exists() else {'baseline':BASELINE,'profiles':{}}
    c=Client(a.binary,log_path=OUT/'runtime.log')
    def publish(recipe):
        path=OUT/'candidate.ignis-botany';path.write_text(json.dumps(recipe,separators=(',',':')))
        started=time.monotonic();r=c.call('botany',operation={'kind':'load','path':str(path.relative_to(ROOT)),'publish':True});r['publish_seconds']=time.monotonic()-started;return r
    def capture(name,camera):
        c.call('set_camera',camera=camera);c.call('capture',path=f'artifacts/botany-branching/{name}.png',width=1100,height=1000)
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'branch-review','version':'1'}});c.request('notifications/initialized',notification=True)
        schema=c.request('tools/list')['result'];encoded=json.dumps(schema)
        for name in ['fork','sheath','blade_samples','compare_images']:assert name in encoded
        (OUT/'native-schema.json').write_text(json.dumps(schema,indent=2))
        c.call('generate',recipe={'width':60,'depth':60,'seed':197,'obstacles':0});state=c.call('inspect');report['build_id']=state['build_id']
        c.call('edit_scene',edits=[{'op':'remove','id':e['id']} for e in state['scene']['entities']])
        ground=dict(id='ground',geometry=dict(positions=[[-150,-.07,-150],[150,-.07,-150],[-150,-.07,150],[150,-.07,150]],triangles=[[0,2,1],[1,2,3]]))
        instance=dict(id='ground',asset='ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.20,.29,.09])
        c.call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_asset','asset':ground}},{'op':'mesh','edit':{'kind':'put_instance','instance':instance}}])
        c.call('set_light',light=dict(direction=[.4,-.85,.25],ambient=.26,direct_intensity=1.35,environment={'sky':[.52,.64,.78],'horizon':[.65,.67,.53],'ground':[.10,.16,.04],'axis':[0,1,0],'intensity':.5},shadow={'center':[0,7,0],'extent':60,'resolution':2048,'bias':.0005},atmosphere={'density':0.,'reduce_details':False,'color':[.5,.6,.7],'sun_color':[1.,.9,.7],'sun_strength':0.,'layers':{'sky_color':[.43,.60,.83],'mountain_color':[.43,.60,.83],'detail_start':10000.,'detail_end':20000.,'influence':0.}}))
        c.call('save_scene',path='artifacts/botany-branching/empty.scene.json')
        for name in ([a.only] if a.only else NAMES):
            c.call('load_scene',path='artifacts/botany-branching/empty.scene.json')
            old_bytes=subprocess.run(['git','show',f'{BASELINE}:assets/botany/{name}.ignis-botany'],cwd=ROOT,check=True,capture_output=True).stdout
            new_bytes=(ROOT/f'assets/botany/{name}.ignis-botany').read_bytes();old=json.loads(old_bytes);new=json.loads(new_bytes);fingerprint=hashlib.sha256(new_bytes).hexdigest()
            if a.phase=='structure':
                entry=dict(before_sha256=hashlib.sha256(old_bytes).hexdigest(),after_sha256=fingerprint,before=stats(old),after=stats(new))
                before=publish(bare(old));after=publish(bare(new));entry['before_bare']=before;entry['after_bare']=after
                lo=[min(before['compiled']['bounds_min'][k],after['compiled']['bounds_min'][k]) for k in range(3)];hi=[max(before['compiled']['bounds_max'][k],after['compiled']['bounds_max'][k]) for k in range(3)]
                h=hi[1]-lo[1];span=max(hi[0]-lo[0],hi[2]-lo[2]);cameras={v:fit(lo,hi,1100,1000,direction=d) for v,d in dict(whole=(.08,-.08,1.),side=(1.,.12,.1),back=(-.1,.12,-1.),top=(.03,1.,.06)).items()}
                cameras['inside']=dict(position=[.3,h*.10,span*.22],target=[0,h*.70,0],fov_degrees=65)
                center=[(a+b)*.5 for a,b in zip(lo,hi)];center[1]=lo[1]+.42*h
                cameras['presentation']=dict(position=bm.add(center,bm.mul(bm.unit((.08,-.08,1.)),h/(2*math.tan(math.radians(17.5)))*1.5)),target=center,fov_degrees=35)
                entry['cameras']=cameras
                for label,r in [('before',old),('after',new)]:
                    publish(bare(r))
                    for view,camera in cameras.items():capture(f'{name}-{label}-bare-{view}',camera)
                report['profiles'][name]=entry
            else:
                entry=report['profiles'][name];assert entry['after_sha256']==fingerprint,'Geometry changed: rerun structural review first'
                for label,r in [('before',old),('after',new)]:
                    result=publish(r);entry[label+'_full']=result
                    for view in ['whole','side','inside','presentation']:capture(f'{name}-{label}-full-{view}',entry['cameras'][view])
                if name=='figueira_rs':
                    rest=c.call('botany',operation={'kind':'physics','prefix':name,'action':{'kind':'start','settings':{'gravity':[0,0,0],'ground':None,'wind':[.5,0,0],'iterations':4}}})
                    step=c.call('botany',operation={'kind':'physics','prefix':name,'action':{'kind':'step','ticks':4}})
                    assert step['attachment_error_m']<1e-4 and step['node_identity']==rest['node_identity']
                    root=next(o['id'] for o in new['source']['organs'] if o['id'].endswith('/ramo0/ramo0') and o['shape']['kind']=='axis')
                    pruned=c.call('botany',operation={'kind':'physics','prefix':name,'action':{'kind':'prune','organ':root}})
                    assert pruned['removed_mass_kg']>0 and pruned['node_identity']==rest['node_identity']
                    assert abs(pruned['mass_kg']+pruned['removed_mass_kg']-pruned['initial_mass_kg']-pruned['added_mass_kg'])<1e-8
                    entry['physics']=dict(rest=rest,step=step,prune=pruned)
                    c.call('botany',operation={'kind':'physics','prefix':name,'action':{'kind':'reset'}})
                    c.call('set_camera',camera=entry['cameras']['whole']);c.call('save_scene',path='artifacts/botany-branching/figueira.scene.json');c.call('load_scene',path='artifacts/botany-branching/figueira.scene.json');capture('roundtrip',entry['cameras']['whole'])
                    assert (OUT/'roundtrip.png').read_bytes()==(OUT/f'{name}-after-full-whole.png').read_bytes();entry['roundtrip_pixels_equal']=True
            report_path.write_text(json.dumps(report,indent=2));print(name,a.phase,entry['after'],flush=True)
        review(report)
    finally:
        c.close();(OUT/'candidate.ignis-botany').unlink(missing_ok=True)

def review(report):
    html='''<!doctype html><meta charset="utf-8"><title>Mais ramificação · Ignis</title><style>body{font:16px system-ui;background:#172126;color:#edf2e7;max-width:1450px;margin:32px auto;padding:0 24px;line-height:1.5}a{color:#9bd8b9}.row{display:grid;grid-template-columns:1fr 1fr;gap:16px}img{width:100%;display:block}figure{margin:0}h2{margin-top:40px}summary{cursor:pointer;margin:16px 0}table{border-collapse:collapse}td,th{padding:6px 16px;text-align:left;border-bottom:1px solid #555}@media(max-width:800px){.row{grid-template-columns:1fr}}</style><h1>Ramificação: antes e depois</h1><p>Esquerda: versão099f5d6. Direita: nova ramificação. Mesma câmera/luz por par, folhas removidas nas vistas de estrutura; pendentes também removidos para não inflar a leitura de galhos. Câmeras manuais não calibradas. O modelo não reproduz exatamente as fotografias.</p><p><a href="../../docs/references/botany/figueira-house-analysis.md">Análise da nova foto/lista construtiva</a> · <a href="../../docs/botany-branching.md">Matemática, parâmetros e validação</a></p><details><summary>Referência interna enviada pelo usuário</summary><img src="../botany-rs/house-reference.png"><p>Outra árvore e outro enquadramento; a foto anterior continua orientando a silhueta externa. A rede interna de galhos é o critério desta rodada.</p></details>'''
    for name,entry in report['profiles'].items():
        html+=f'<h2>{NAMES[name]}</h2><table><tr><th>Fonte real</th><th>Antes</th><th>Depois</th></tr>'
        for key,label in [('structural_axes','Eixos estruturais'),('shoot_axes','Eixos de brotação'),('woody_axes','Total lenhoso, sem pendentes'),('foliage_hosts','Hospedeiros de folhas'),('leaves','Folhas'),('max_woody_depth','Profundidade lenhosa máxima')]:html+=f'<tr><td>{label}</td><td>{entry["before"][key]}</td><td>{entry["after"][key]}</td></tr>'
        html+='</table>'
        for mode,label in [('bare','Estrutura'),('full','Com folhas')]:
            for view,view_label in [('whole','inteira'),('presentation','próxima, bordas podem sair do quadro'),('inside','interior'),('side','lateral'),('back','verso'),('top','superior')]:
                left=f'{name}-before-{mode}-{view}.png';right=f'{name}-after-{mode}-{view}.png'
                if not (OUT/right).exists():continue
                html+=f'<details'+(' open' if mode=='bare' and view=='whole' else '')+f'><summary>{label} · {view_label}</summary><div class="row"><figure><a href="{left}"><img loading="lazy" src="{left}"></a><figcaption>Antes</figcaption></figure><figure><a href="{right}"><img loading="lazy" src="{right}"></a><figcaption>Depois</figcaption></figure></div></details>'
    html+='<p>Mais ramos é uma mudança de fonte e topologia; testes e estatísticas não aprovam fidelidade artística. Base sulcada/fusão, epífitas felpudas e autointerseção global continuam fora da validação desta rodada.</p>'
    (OUT/'review.html').write_text(html)
if __name__=='__main__':main()
