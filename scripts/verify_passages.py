#!/usr/bin/env python3
"""Cooperative explicit passages over real MCP, physical motion and replay."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_targeting import actor

def verify(binary,window):
    c=Client(binary,window);checks=[];samples=[]
    def state():return c.call('inspect_passages')
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-passages-verifier','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':18,'depth':14,'seed':42,'obstacles':0})
        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');player['transform']['position']=[-6,0.85,4];player.pop('footsteps',None)
        actors=[]
        for id,x,goal,priority,color in [('alpha',-4,6,10,[0.7,0.2,0.8]),('beta',4,-6,0,[0.15,0.5,0.95])]:
            e=actor(id,'cooperative',[x,0.85,0],color);e['npc'].update(enabled=True,speed=2,stop_distance=0.3,passage_priority=priority,navigation_recovery={'retry_ticks':15,'max_retry_ticks':60,'stalled_ticks':60,'min_progress':0.1},objectives=[{'priority':1,'when':'always','action':{'kind':'guard','point':[goal,0],'arrival_distance':0.3}}]);actors.append(e)
        walls=[{'id':id,'name':id,'body':'fixed','color':[0.2,0.65,0.45],'transform':{'position':[0,1.5,z],'size':[0.4,3,6.9]}} for id,z in [('north',4.15),('south',-4.15)]]
        passage={'id':'door','surface':'floor','a':[-1,0],'b':[1,0],'width':1.4,'wait_a':[[-3,2],[-3,-2]],'wait_b':[[3,2],[3,-2]],'lease_ticks':600,'retry_ticks':60,'aging_ticks':60}
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [player,*actors,*walls]]+[{'op':'set_passages','passages':[passage]},{'op':'set_camera','camera':{'position':[12,18,19],'target':[0,0,0],'fov_degrees':48}}])
        c.call('set_debug_view',npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False);c.call('select_entity',id='alpha');c.call('save_scene',path='examples/npc-passages.scene.json')
        c.call('step',ticks=1);first=state();samples.append(first);assert first['owners']==['alpha'] and first['runtime']['agents']['beta']['phase']=='waiting',first
        c.call('step',ticks=59);assert state()['owners']==['alpha'];c.call('capture',path='artifacts/passages-waiting.png',width=1280,height=900)
        c.call('save_replay',path='artifacts/passages-waiting.replay.json');assert c.call('run_replay',path='artifacts/passages-waiting.replay.json')['matched']
        c.call('capture_replay',replay_path='artifacts/passages-waiting.replay.json',path='artifacts/passages-waiting-replay.png',width=1280,height=900);assert (ROOT/'artifacts/passages-waiting.png').read_bytes()==(ROOT/'artifacts/passages-waiting-replay.png').read_bytes()
        c.call('step',ticks=840);end=c.call('inspect');samples.append(state());assert end['runtime_positions']['alpha'][0]>5.5 and end['runtime_positions']['beta'][0]<-5.5,end
        assert all(a['completions']==1 for a in state()['runtime']['agents'].values());assert state()['owners']==[None]
        c.call('save_replay',path='artifacts/passages.replay.json');assert c.call('run_replay',path='artifacts/passages.replay.json')['matched'];live=c.call('inspect');prefix=c.call('inspect_replay',path='artifacts/passages.replay.json',tick=1);assert prefix['snapshot']['passages']==first['runtime'] and c.call('inspect')==live
        checks.append('Exclusive opposite crossings, non-preemption, original goal resumption, replay prefix and exact PNG')
        short=copy.deepcopy(passage);short['lease_ticks']=30;c.call('set_passages',passages=[short]);c.call('step',ticks=31);timed=state();assert timed['owners']==['beta'] and timed['runtime']['agents']['alpha']['phase']=='cooldown';samples.append(timed)
        player_block=copy.deepcopy(player);player_block['transform']['position']=[0,0.85,0];c.call('edit_scene',edits=[{'op':'set_passages','passages':[passage]},{'op':'upsert','entity':player_block}]);c.call('step',ticks=40);occupied=state();assert occupied['owners']==[None] and occupied['runtime']['slots'][0]['occupied']
        c.call('set_input',input={'x':1,'z':1});c.call('step',ticks=60);c.call('set_input',input={'x':0,'z':0});assert state()['runtime']['slots'][0]['grants']>0
        checks.append('Lease expiration/cooldown transfers next turn, physical nonparticipant occupancy prevents grants until player leaves')
        before=c.call('inspect');bad=copy.deepcopy(passage);bad['wait_a']=[];c.call('set_passages',passages=[bad],expected_error=True);assert c.call('inspect')==before
        c.call('load_scene',path='examples/npc-passages.scene.json');assert state()['runtime']['slots'][0]['grants']==0;c.call('undo');c.call('redo');assert state()['runtime']['slots'][0]['grants']==0
        c.call('step',ticks=1);before=c.call('inspect');c.call('set_debug_view',npc_routes=False);c.call('capture',path='artifacts/passages-no-guides.png',width=1280,height=900);c.call('set_debug_view',npc_routes=True);c.call('capture',path='artifacts/passages-guides.png',width=1280,height=900);after=c.call('inspect');assert before['passages']==after['passages'] and before['runtime_positions']==after['runtime_positions'];assert (ROOT/'artifacts/passages-no-guides.png').read_bytes()!=(ROOT/'artifacts/passages-guides.png').read_bytes();c.call('reset')
        checks.append('MCP atomic authoring, validation, save/load/undo/redo/reset and passage/waiting guides without simulation effects')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':c.call('inspect')['build_id'],'slow_requests':c.slow_requests};(ROOT/'artifacts/passages-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
