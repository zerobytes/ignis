#!/usr/bin/env python3
"""Native root-system review, isolated from the human scene/window."""
import argparse,copy,hashlib,json,subprocess,time
from pathlib import Path
from verify_lab import Client,ROOT
from demo_botany_age import fit
import botany_math as bm
OUT=ROOT/'artifacts/botany-buttress'
BASELINE='5e37102'
SOURCE_OVERRIDE=None
DOC='botany-buttress'
ANALYSIS='buttress-analysis'
PRESERVE_ALL_ORGANS=False
REFERENCE_IMAGES=('reference-wall.png','reference-ridge.png')
COMPARE_VIEW='front'
COMPARE_REGIONS=[dict(id='collar',x=300,y=250,width=600,height=600),dict(id='left_roots',x=0,y=400,width=600,height=500)]
DEFAULT=dict(kind='radial',count=7,seed=0,spread=9.,depth=1.2,radius_ratio=.55,neck_height=2.5,exposure=.55,soil_height=0.,soil_slope=[0.,0.],orders=2,length_ratio=.65,branch_angle=.60,irregularity=.30,anchored_fraction=.3,buttress=dict(height=2.6,reach=5.5,wall_width=.28,crest_width=.65,meander=.18,fluting=1.2))

def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--appearance-only',action='store_true');p.add_argument('--reuse-before',action='store_true');p.add_argument('--settings',type=Path);p.add_argument('--publish-source',action='store_true');a=p.parse_args();relative=OUT.relative_to(ROOT).as_posix();OUT.mkdir(parents=True,exist_ok=True)
    report=json.loads((OUT/'report.json').read_text()) if a.appearance_only else {'baseline':BASELINE}
    source_bytes=SOURCE_OVERRIDE.read_bytes() if SOURCE_OVERRIDE else subprocess.run(['git','show',f'{BASELINE}:assets/botany/figueira_rs.ignis-botany'],cwd=ROOT,check=True,capture_output=True).stdout;base=json.loads(source_bytes)
    settings=json.loads(a.settings.read_text()) if a.settings else DEFAULT
    beards=[o['id'] for o in base['source']['organs'] if '/epifita' in o['id'] and '/fio' not in o['id']]
    c=Client(a.binary,log_path=OUT/'runtime.log')
    def publish(recipe):
        path=OUT/'candidate.ignis-botany';path.write_text(json.dumps(recipe,separators=(',',':')))
        started=time.monotonic();r=c.call('botany',operation={'kind':'load','path':str(path.relative_to(ROOT)),'publish':True});r['seconds']=time.monotonic()-started;return r
    def capture(name,camera):
        c.call('set_camera',camera=camera);c.call('capture',path=f'{relative}/{name}.png',width=1200,height=1000)
    ground=dict(id='ground',geometry=dict(positions=[[-150,-.07,-150],[150,-.07,-150],[-150,-.07,150],[150,-.07,150]],triangles=[[0,2,1],[1,2,3]]))
    instance=dict(id='ground',asset='ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.22,.26,.11])
    def ground_visible(visible):c.call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_instance','instance':instance} if visible else {'kind':'remove_instance','id':'ground'}}])
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'roots-review','version':'1'}});c.request('notifications/initialized',notification=True)
        schema=c.request('tools/list')['result'];encoded=json.dumps(schema)
        for name in ['set_roots','root_systems','basal','buttress','crest_width']:assert name in encoded,name
        (OUT/'native-schema.json').write_text(json.dumps(schema,indent=2))
        c.call('generate',recipe={'width':60,'depth':60,'seed':198,'obstacles':0});state=c.call('inspect');report['build_id']=state['build_id']
        c.call('edit_scene',edits=[{'op':'remove','id':e['id']} for e in state['scene']['entities']])
        c.call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_asset','asset':ground}}]);ground_visible(True)
        c.call('set_light',light=dict(direction=[.5,-.8,.3],ambient=.32,direct_intensity=1.3,environment={'sky':[.52,.64,.78],'horizon':[.65,.67,.53],'ground':[.10,.16,.04],'axis':[0,1,0],'intensity':.5},shadow={'center':[0,3,0],'extent':45,'resolution':2048,'bias':.0005},atmosphere={'density':0.,'reduce_details':False,'color':[.5,.6,.7],'sun_color':[1.,.9,.7],'sun_strength':0.,'layers':{'sky_color':[.43,.60,.83],'mountain_color':[.43,.60,.83],'detail_start':10000.,'detail_end':20000.,'influence':0.}}))
        cameras=dict(front=dict(position=[4.8,2.4,8.],target=[0,.8,0],fov_degrees=48),side=dict(position=[9.,3.,-.8],target=[0,.6,0],fov_degrees=55),back=dict(position=[-5.,3.,-9.],target=[0,.7,0],fov_degrees=55),top=dict(position=[.2,22.,.6],target=[0,0,0],fov_degrees=55),network=dict(position=[10.,5.,15.],target=[0,-.3,0],fov_degrees=60))
        if not a.appearance_only:
            bare=copy.deepcopy(base);bare['settings'].update(leaf_density=0.,bark_color=[.43,.43,.43]);bare['pruned']=beards
            before=publish(bare);report['before']=before
            for view,camera in cameras.items():
                if not (a.reuse_before and (OUT/f'before-{view}.png').exists()):capture('before-'+view,camera)
            # Preview must not change the world; the next operation publishes it.
            preview=c.call('botany',operation={'kind':'set_roots','prefix':'figueira_rs','stem':'tronco0','settings':settings,'preview':True})
            assert not preview['world_changed']
            after=c.call('botany',operation={'kind':'set_roots','prefix':'figueira_rs','stem':'tronco0','settings':settings})
            report.update(after=after,settings=settings,before_sha256=hashlib.sha256(source_bytes).hexdigest(),cameras=cameras)
            lo=[min(before['compiled']['bounds_min'][k],after['compiled']['bounds_min'][k]) for k in range(3)];hi=[max(before['compiled']['bounds_max'][k],after['compiled']['bounds_max'][k]) for k in range(3)];cameras['tree']=fit(lo,hi,1200,1000)
            for view,camera in cameras.items():capture('after-'+view,camera)
            ground_visible(False)
            for view in ['front','side','top','network']:capture('underground-'+view,cameras[view])
            ground_visible(True)
            c.call('save_scene',path=f'{relative}/structure.scene.json')
            c.call('botany',operation={'kind':'save','prefix':'figueira_rs','path':f'{relative}/structure.ignis-botany'})
            full=json.loads((OUT/'structure.ignis-botany').read_text());full['settings']=base['settings'];full['pruned']=base.get('pruned',[])
            def normalized(o):return {'twist':0.,'taper_power':1.,'connection':'graft',**o}
            assert [normalized(o) for o in full['source']['organs'] if '/root' not in o['id']]==[normalized(o) for o in base['source']['organs'] if '/root' not in o['id']]
            if PRESERVE_ALL_ORGANS:assert [normalized(o) for o in full['source']['organs']]==[normalized(o) for o in base['source']['organs']]
            report['approved_organs_unchanged']=True
            bm.atomic_json(OUT/'rooted.ignis-botany',full)
            report['rooted_sha256']=hashlib.sha256((OUT/'rooted.ignis-botany').read_bytes()).hexdigest()
            lower=copy.deepcopy(settings);lower['buttress'].update(height=1.3,wall_width=.4,crest_width=.65,meander=.12,fluting=.5)
            variant=c.call('botany',operation={'kind':'set_roots','prefix':'figueira_rs','stem':'tronco0','settings':lower})
            report['lower_wall_variant']=dict(settings=lower,compiled=variant['compiled'])
            for view in ['front','side']:capture('lower-'+view,cameras[view])
            # Same framing, including a before image of the whole approved tree.
            publish(bare);capture('before-tree',cameras['tree'])
            report['appearance_pending']=True
            report['native_change']=c.call('compare_images',reference_path=f'{relative}/before-{COMPARE_VIEW}.png',candidate_path=f'{relative}/after-{COMPARE_VIEW}.png',regions=COMPARE_REGIONS,difference_path=f'{relative}/change.png')
        else:
            full=json.loads((OUT/'rooted.ignis-botany').read_bytes());cameras=report['cameras']
            assert report['rooted_sha256']==hashlib.sha256((OUT/'rooted.ignis-botany').read_bytes()).hexdigest()
            adult=publish(full);report['adult']=adult
            for view in ['front','side','tree']:capture('adult-'+view,cameras[view])
            rest=c.call('botany',operation={'kind':'physics','prefix':'figueira_rs','action':{'kind':'start','settings':{'gravity':[0,0,0],'ground':0.,'wind':[.5,0,0],'iterations':4}}})
            assert rest['root_anchor_nodes']>0
            step=c.call('botany',operation={'kind':'physics','prefix':'figueira_rs','action':{'kind':'step','ticks':4}})
            assert step['attachment_error_m']<1e-4 and step['node_identity']==rest['node_identity'] and step['min_node_y']<0
            prune=c.call('botany',operation={'kind':'physics','prefix':'figueira_rs','action':{'kind':'prune','organ':'tronco0/root0'}})
            assert prune['removed_mass_kg']>0 and prune['node_identity']==rest['node_identity']
            assert abs(prune['mass_kg']+prune['removed_mass_kg']-prune['initial_mass_kg']-prune['added_mass_kg'])<1e-8
            report['physics']=dict(rest=rest,step=step,prune=prune)
            c.call('botany',operation={'kind':'physics','prefix':'figueira_rs','action':{'kind':'reset'}})
            c.call('set_camera',camera=cameras['front']);c.call('save_scene',path=f'{relative}/figueira.scene.json');c.call('load_scene',path=f'{relative}/figueira.scene.json');capture('roundtrip',cameras['front'])
            assert (OUT/'adult-front.png').read_bytes()==(OUT/'roundtrip.png').read_bytes();report['roundtrip_pixels_equal']=True;report['appearance_pending']=False
            if a.publish_source:
                c.call('botany',operation={'kind':'save','prefix':'figueira_rs','path':'assets/botany/figueira_rs.ignis-botany'});report['canonical_sha256']=hashlib.sha256((ROOT/'assets/botany/figueira_rs.ignis-botany').read_bytes()).hexdigest()
        (OUT/'report.json').write_text(json.dumps(report,indent=2));review(report)
        print(json.dumps({k:v for k,v in report.items() if k in ['build_id','approved_organs_unchanged','roundtrip_pixels_equal','canonical_sha256']},indent=2),flush=True)
    finally:c.close();(OUT/'candidate.ignis-botany').unlink(missing_ok=True)

def review(report):
    html='''<!doctype html><meta charset="utf-8"><title>Raízes nativas · Ignis</title><style>body{font:16px system-ui;background:#172126;color:#edf2e7;max-width:1450px;margin:32px auto;padding:0 24px;line-height:1.5}a{color:#9bd8b9}.row{display:grid;grid-template-columns:1fr 1fr;gap:16px}img{width:100%;display:block}figure{margin:0}summary{cursor:pointer;margin:16px 0}@media(max-width:800px){.row{grid-template-columns:1fr}}</style><h1>Raízes em paredes e cristas</h1><p>Seções volumétricas com paredes altas, cristas arredondadas e nervuras no colo, geradas nativamente. Câmeras manuais; fotos sem escala calibrada. O solo é um plano autoral. Ramos/folhas aprovados mantidos na fonte; a extensão do colo pode elevar a copa.</p><p><a href="../../docs/botany-buttress.md">Matemática, parâmetros e limites</a> · <a href="../../docs/references/botany/buttress-analysis.md">Lista de construção</a></p>'''
    for view,label in [('front','Base: antes e depois'),('side','Lateral'),('back','Verso hipotético'),('top','Superior'),('tree','Árvore inteira')]:
        if (OUT/f'after-{view}.png').exists():html+=f'<h2>{label}</h2><div class="row"><img loading="lazy" src="before-{view}.png"><img loading="lazy" src="after-{view}.png"></div>'
    html+='<h2>Outra configuração: paredes mais baixas e largas</h2><div class="row"><img src="lower-front.png"><img src="lower-side.png"></div>'
    for view in ['front','side','top','network']:
        if (OUT/f'underground-{view}.png').exists():html+=f'<details><summary>Rede sem solo · {view}</summary><img loading="lazy" src="underground-{view}.png"></details>'
    for view in ['front','side','tree']:
        if (OUT/f'adult-{view}.png').exists():html+=f'<h2>Com folhas · {view}</h2><img loading="lazy" src="adult-{view}.png">'
    html+='<h2>Recortes da referência</h2><div class="row"><img src="reference-wall.png"><img src="reference-ridge.png"></div><p>Partes subterrâneas e o verso são hipóteses. Continuidade técnica não aprova semelhança botânica/artística; raízes não simulam interação constitutiva com solo ou fusão universal.</p>'
    html=html.replace('docs/botany-buttress.md',f'docs/{DOC}.md').replace('botany/buttress-analysis.md',f'botany/{ANALYSIS}.md').replace('reference-wall.png',REFERENCE_IMAGES[0]).replace('reference-ridge.png',REFERENCE_IMAGES[1])
    (OUT/'review.html').write_text(html)
if __name__=='__main__':main()
