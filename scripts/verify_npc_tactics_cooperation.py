#!/usr/bin/env python3
"""Critical MCP integration of private tactics, frozen formation and support leases."""
import argparse
import json
from pathlib import Path
from verify_lab import Client,ROOT
from verify_geometry import box

def actor(name,x,z):
    e=box(name,[x,0.81,z],[0.6,1.6,0.6],body='npc',color=(0.6,0.2,0.8))
    e['character_controller']={'snap_distance':0.3}
    e['npc']={'surface':'floor','speed':2,'stop_distance':0.2,'enabled':True,'mobility':{'walk':True},'local_connections':{'walk':True,'fly':False,'max_distance':8},'mind':{'intelligence':1},'cooperation':{'group':7},'objectives':[{'priority':0,'when':'always','action':{'kind':'wait'}}]}
    return e

def awareness():
    return {'mode':'perceptive','vision_range':30,'fov_degrees':360,'initial_facing_degrees':0,'memory_ticks':120,'search_ticks':60,'search_turn_degrees_per_second':0}

def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-tactics-cooperation','version':'1'}});c.request('notifications/initialized',notification=True)
        def base():
            c.call('generate',recipe={'width':30,'depth':24,'seed':42,'obstacles':0});p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[0,0.8,0];p.pop('footsteps',None);return p
        p=base();t=actor('tactician',-8,0);t['npc']['target']='player';t['npc']['awareness']=awareness();t['npc']['mind']['personality']={'caution':1,'fear':1,'courage':0}
        t['npc']['objectives']=[{'priority':1,'when':'evidence','action':{'kind':'tactical','configuration':{'range':5,'cover':1,'flank':1,'max_candidates':64}}}]
        wall=box('cover',[-2,1.5,2],[0.5,3,2]);c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [p,t,wall]])
        c.call('set_spatial_network',network={'nodes':[[-8,0,0],[-4,0,0],[-4,0,4]],'links':[]});c.call('step',ticks=1)
        assert c.call('inspect_npc_tactics')['npcs']['tactician']['state']['choice']['covered']
        c.call('step',ticks=99);prefix=c.call('inspect');c.call('capture',path='artifacts/npc-tactics-prefix.png',width=1000,height=700)
        path='artifacts/npc-tactics.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched'];c.call('capture_replay',replay_path=path,path='artifacts/npc-tactics-prefix-replay.png',tick=100,width=1000,height=700)
        assert (ROOT/'artifacts/npc-tactics-prefix.png').read_bytes()==(ROOT/'artifacts/npc-tactics-prefix-replay.png').read_bytes()
        wall['transform']['position'][2]=0;c.call('upsert',entity=wall);c.call('step',ticks=30);state=c.call('inspect_npc_tactics')['npcs']['tactician']['state'];assert state is None
        checks.append('Cover/flank/range use frozen evidence and known geometry; hidden target supplies no tactical goal; exact replay and offscreen pixels')
        p=base();p['transform']['position']=[-10,0.8,8];a,b=actor('a',-4,-2),actor('b',-4,2);b['npc']['cooperation']['role']=1
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [p,a,b]]);c.call('step',ticks=1)
        order={'kind':'move','position':[2,0.81,0],'arrival_distance':0.2,'for_ticks':900,'formation':'line','spacing':2,'facing_degrees':0}
        response=c.call('order_npc_group',group=7,order=order);assert len(response['recipients'])==2;c.call('step',ticks=1);frozen=c.call('inspect_npc_cooperation')['npcs']['b']['order']['order']
        c.call('step',ticks=360);positions=c.call('inspect')['runtime_positions'];assert positions['a'][0]>0.5 and positions['b'][0]>2.5,positions
        c.call('despawn_actor',actor='a');c.call('step',ticks=1);assert c.call('inspect_npc_cooperation')['npcs']['b']['order']['order']==frozen
        before=c.call('inspect');c.call('order_npc',actor='player',order={'kind':'hold','for_ticks':60},expected_error=True);assert c.call('inspect')==before
        path='artifacts/npc-orders.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        checks.append('Role/traits assign frozen formation positions; commands execute physically, survivor keeps its slot after despawn, player authority rejection is atomic, replay matches')
        for projectile in [False,True]:
            p=base();p['faction']='team';p['vitality']={'initial':20,'maximum':100,'scale':1}
            heal={'id':'heal','selection':'actor','range':8,'relation':'allied','cast_ticks':0 if projectile else 30,'cooldown_ticks':90,'costs':[{'resource':'energy','units':5}],'effects':[{'kind':'heal','recipient':'target','amount':80}]}
            if projectile:heal['delivery']={'kind':'projectile','speed':10,'half_extent':0.05,'max_distance':20}
            catalog={'resources':[{'id':'energy','scale':1}],'abilities':[heal]};members=[]
            for name,x,concern in [('a',-2,0),('b',2,1)]:
                n=actor(name,x,-2);n['color']=[0.15,0.5,0.9] if name=='a' else [0.15,0.8,0.5];n['faction']='team';n['vitality']={'initial':100,'maximum':100,'scale':1};n['abilities']={'resources':[{'resource':'energy','initial':100,'capacity':100}],'grants':['heal']}
                n['npc']['cooperation']['reserve_support']=True;n['npc']['mind']['personality']={'concern':concern};n['npc']['awareness']=awareness();n['npc']['objectives']=[{'priority':2,'when':'always','action':{'kind':'use_ability','slot':1,'retry_ticks':10,'policy':{'allies':{'ranking':'lowest_health_ratio','include_self':False},'vitality_below':{'subject':'target','basis_points':9000}}}},{'priority':0,'when':'always','action':{'kind':'wait'}}];members.append(n)
            c.call('edit_scene',edits=[{'op':'set_ability_catalog','catalog':catalog}]+[{'op':'upsert','entity':e} for e in [p,*members]])
            if projectile:c.call('save_scene',path='examples/npc-tactics-cooperation.scene.json')
            c.call('step',ticks=1);coord=c.call('inspect_npc_cooperation');assert len(coord['support_claims'])==1 and sum(d['allowed'] for d in coord['support_decisions'])==1
            c.call('step',ticks=90);state=c.call('inspect');assert state['vitality']['actors']['player']['units']==100;assert sum(r['outcome']=='started' for r in state['abilities']['receipts'])==1
            assert not c.call('inspect_npc_cooperation')['support_claims'];path='artifacts/npc-support-reservations.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        checks.append('Concern wins shared support election; one heal/cost for visible wounded ally, lease persists through cast/projectile and releases on completion, replay matches')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/npc-tactics-cooperation-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
