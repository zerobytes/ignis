#!/usr/bin/env python3
"""Critical MCP integration: source/undo/replay/save, growing complete plant,
pruning ledgers, nonlinear bending and explicit contact. No GUI activation.
"""
import argparse,copy,json,math,time
from pathlib import Path
from verify_lab import Client,ROOT
from botany_exemplars import Builder

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');args=parser.parse_args()
    out=ROOT/'artifacts/botany-native';out.mkdir(parents=True,exist_ok=True);report={}
    c=Client(args.binary,log_path=out/'verification.log')
    def op(**kw):return c.call('botany',operation=kw)
    def load_recipe(r,name):
        path=f'artifacts/botany-native/{name}.ignis-botany';(ROOT/path).write_text(json.dumps(r,separators=(',',':')))
        return op(kind='load',path=path,publish=True)
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'botany-native-verifier','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='artifacts/botany-native/empty.scene.json');report['build_id']=c.call('inspect')['build_id']
        # A small source protects replay/portable contracts without expanding their envelopes.
        b=Builder('Integration fixture');root=b.axis('stem',None,0,(0,1,0),1.,.015);b.leaf('blade',root,.7,(1,.2,0),.3,.1,normal=(0,0,1))
        for name,side in [('left',-1),('right',1)]:
            b.axis(name,root,1.,(side*.7,.8,.1),.7,.005,normal=(0,0,1));b.organs[-1]['connection']='fork'
        r=b.recipe('fixture');r['version']=r['source']['version']=4;r['source']['development']={};r['settings']['age_years']=50;r['edits']={'stem':{'deformation':{'base':[.35,.15],'tip':[-.2,.6],'twist':.5,'sag':.15,'irregularity':.12}}};load_recipe(r,'fixture');before=c.call('inspect');revision=before['revision']
        preview=copy.deepcopy(r);preview['settings']['scale']=1.2
        op(kind='preview',recipe=preview);assert c.call('inspect')['scene']==before['scene'];assert c.call('inspect')['revision']==revision
        op(kind='cancel_preview');op(kind='set_age',prefix='fixture',age_years=20,preview=True);assert c.call('inspect')['scene']==before['scene'];op(kind='cancel_preview');op(kind='set_age',prefix='fixture',age_years=50);op(kind='save',prefix='fixture',path='artifacts/botany-native/saved-fixture.ignis-botany')
        c.call('step',ticks=2);c.call('save_replay',path='artifacts/botany-native/fixture.replay.json');assert c.call('run_replay',path='artifacts/botany-native/fixture.replay.json')['meshes_matched']
        c.call('save_portable_save',path='artifacts/botany-native/fixture.ignis-portable');c.call('load_portable_save',path='artifacts/botany-native/fixture.ignis-portable');assert c.call('inspect')['scene']['botany'][0]['source']==before['scene']['botany'][0]['source']
        report['authoring_preview_replay_portable']=True
        # Whole source and all descendants in the same solve; no species-specific physics path.
        c.call('load_scene',path='artifacts/botany-native/empty.scene.json');loaded=op(kind='load',path='assets/botany/figueira.ignis-botany',publish=True);report['complete_source']=loaded['compiled']
        c.call('set_camera',camera={'position':[5,5,8],'target':[0,2,0],'fov_degrees':38})
        initial=op(kind='physics',prefix='figueira',action={'kind':'start','settings':{'age':1.8,'gravity':[0,0,0],'wind':[1,0,0],'ground':None,'iterations':24}})
        initial_nodes=initial['node_identity'];start=time.monotonic()
        for age in [1.8+.05*i for i in range(1,33)]:
            op(kind='physics',prefix='figueira',action={'kind':'grow','age':age});grown=op(kind='physics',prefix='figueira',action={'kind':'step','ticks':4})
            assert grown['node_identity']==initial_nodes and grown['attachment_error_m']<1e-4
            if round((age-1.8)*20)%8==0:print(json.dumps({'phase':round(age,2),'nodes':grown['nodes'],'constraints':grown['constraints'],'attachment_error_m':grown['attachment_error_m']}),flush=True)
        report['complete_growth']={'before':initial,'after':grown,'seconds':time.monotonic()-start}
        c.call('capture',path='artifacts/botany-native/figueira-grown.png',width=1200,height=1000)
        before=op(kind='physics',prefix='figueira',action={'kind':'inspect'});pruned=op(kind='physics',prefix='figueira',action={'kind':'prune','organ':'tronco0/f0'})
        assert pruned['node_identity']==initial_nodes and pruned['removed_mass_kg']>0
        assert abs(pruned['mass_kg']+pruned['removed_mass_kg']-pruned['added_mass_kg']-pruned['initial_mass_kg'])<1e-8
        for k in range(3):assert abs(before['momentum'][k]-pruned['momentum'][k]-pruned['removed_momentum'][k])<1e-8
        report['pruning']=pruned;c.call('capture',path='artifacts/botany-native/figueira-pruned.png',width=1200,height=1000)
        # Large bending/ground: native renderer consumes the same deformed material lattice.
        c.call('load_scene',path='artifacts/botany-native/empty.scene.json');b=Builder('Large bending and ground');b.axis('stem',None,0,(1,0,0),1.,.02,position=(0,.4,0),young=1e5,normal=(0,1,0));r=b.recipe('bend');load_recipe(r,'bend')
        c.call('set_camera',camera={'position':[1.5,1.1,2.1],'target':[.4,.2,0],'fov_degrees':35})
        op(kind='physics',prefix='bend',action={'kind':'start','settings':{'iterations':32}});c.call('capture',path='artifacts/botany-native/bend-before.png',width=1000,height=800)
        bent=op(kind='physics',prefix='bend',action={'kind':'step','ticks':240});assert bent['max_frame_rotation_rad']>.65 and bent['contacts']>0 and bent['min_node_y']>=0
        c.call('capture',path='artifacts/botany-native/bend-after.png',width=1000,height=800);report['bending']=bent
        c.call('load_scene',path='artifacts/botany-native/empty.scene.json');b=Builder('Crossed branch proxies');b.axis('a',None,0,(1,0,0),1.,.02,position=(-.5,.5,0),young=1e5,normal=(0,1,0));b.axis('b',None,0,(0,0,1),1.,.02,position=(0,.5,-.5),young=1e5,normal=(0,1,0));load_recipe(b.recipe('contact'),'contact')
        c.call('set_camera',camera={'position':[1.4,1.4,1.8],'target':[0,.5,0],'fov_degrees':35})
        intersecting=op(kind='physics',prefix='contact',action={'kind':'start','settings':{'gravity':[0,0,0],'ground':None,'self_contact':True,'iterations':64}})
        separated=op(kind='physics',prefix='contact',action={'kind':'step','ticks':120});assert intersecting['contact_probe_complete'] and separated['contact_probe_complete'] and intersecting['max_proxy_overlap_m']>0 and separated['max_proxy_overlap_m']<1e-4
        c.call('capture',path='artifacts/botany-native/contact-after.png',width=1000,height=800);report['contact']={'before':intersecting,'after':separated}
        (out/'verification.json').write_text(json.dumps(report,indent=2));print(json.dumps({'build':report['build_id'],'checks':['preview/replay/portable','complete growth/momentum/pruning','large bending/ground','branch contact']},indent=2))
    finally:c.close()
if __name__=='__main__':main()
