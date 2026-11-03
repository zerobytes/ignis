#!/usr/bin/env python3
"""Territory constraints, committed return and visual guides over real MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_targeting import actor
from verify_objectives import objective


def verify(binary,window):
    (ROOT/'artifacts').mkdir(exist_ok=True);c=Client(binary,window);checks=[];samples=[]
    def territory():return c.call('inspect_territory')['npcs']['guard']
    def emit(x):return c.call('emit_noise',origin={'kind':'position','position':[x,0.85,0]},radius=12)['queued']
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-territory-verifier','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':24,'depth':24,'seed':42,'obstacles':0})
        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');player['transform']['position']=[2,0.85,0];player.pop('footsteps',None)
        guard=actor('guard','guards',[1,0.85,0],[0.7,0.2,0.8]);guard['npc'].update(enabled=True,target='player',speed=2,
            awareness={'mode':'perceptive','vision_range':12,'fov_degrees':360,'initial_facing_degrees':0,'memory_ticks':180,'search_ticks':90,'search_turn_degrees_per_second':90,'hearing':{'range':12}},
            territory={'home':[0,0],'radius':5,'arrival_distance':0.3,'rearm_ticks':30},
            patrol={'route':{'kind':'points','points':[[0,2],[0,-2]]},'traversal':'loop','wait_ticks':10,'travel_ticks':600,'retry_ticks':60,'arrival_distance':0.3},
            objectives=[objective(10,'respond','evidence'),objective(1,'patrol')])
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [player,guard]])
        c.call('set_debug_view',npc_vision=False,npc_hearing=False,noise_events=True,npc_routes=True);c.call('select_entity',id='guard')
        c.call('step',ticks=1);assert c.call('inspect')['npcs']['guard']['memory'] is not None
        c.call('set_input',input={'x':1,'z':0});c.call('step',ticks=60);c.call('set_input',input={'x':0,'z':0});returning=territory();assert returning['territory']['phase']=='returning' and returning['territory']['last_trigger']=='target_left',returning;samples.append(returning)
        emit(1);c.call('step',ticks=1);assert c.call('inspect')['npcs']['guard']['memory'] is None
        c.call('capture',path='artifacts/territory-returning.png',width=1280,height=900)
        c.call('save_replay',path='artifacts/territory-returning.replay.json');assert c.call('run_replay',path='artifacts/territory-returning.replay.json')['matched']
        c.call('capture_replay',replay_path='artifacts/territory-returning.replay.json',path='artifacts/territory-returning-replay.png',width=1280,height=900)
        assert (ROOT/'artifacts/territory-returning.png').read_bytes()==(ROOT/'artifacts/territory-returning-replay.png').read_bytes()
        c.call('step',ticks=180);home=territory();assert home['territory']['phase']=='active' and home['territory']['returns']==1,home
        assert c.call('inspect')['npcs']['guard']['memory'] is None and c.call('inspect_objectives')['npcs']['guard']['selection']['action']=='patrol'
        emit(6);c.call('step',ticks=1);assert c.call('inspect')['npcs']['guard']['memory'] is None and territory()['territory']['rejected_sound']==1
        c.call('save_replay',path='artifacts/territory.replay.json');assert c.call('run_replay',path='artifacts/territory.replay.json')['matched']
        live=c.call('inspect');prefix=c.call('inspect_replay',path='artifacts/territory.replay.json',tick=62);assert prefix['snapshot']['npcs']['guard']['territory']['phase']=='returning' and c.call('inspect')==live
        checks.append('Observed departure, committed return despite new sound, actual arrival/cooldown/patrol, outside reacquisition rejected and exact replay/prefix/PNG')

        listener=copy.deepcopy(guard);listener['npc'].pop('target');listener['transform']['position']=[4,0.85,0]
        c.call('upsert',entity=listener);inside=emit(-2);emit(5.1);c.call('step',ticks=1);state=c.call('inspect')['npcs']['guard'];assert state['memory']['noise_id']==inside['id'] and state['territory']['rejected_sound']==1
        outside=copy.deepcopy(listener);outside['transform']['position']=[6,0.85,0];c.call('upsert',entity=outside);c.call('step',ticks=1);assert territory()['territory']['last_trigger']=='own_outside'
        wall={'id':'block','name':'Blocked origin','body':'fixed','color':[0.2,0.65,0.45],'transform':{'position':[0,1,0],'size':[2,2,2]}}
        c.call('upsert',entity=wall);c.call('step',ticks=30);blocked=territory();assert blocked['territory']['phase']=='returning' and blocked['navigation']['status']=='unreachable' and blocked['position'][0]>5.9;samples.append(blocked);c.call('remove',id='block')
        checks.append('Stronger outside sound cannot mask internal cue, initial displacement and unreachable home without teleport/fallback')

        c.call('upsert',entity=guard);c.call('step',ticks=3);live=c.call('inspect');bad=copy.deepcopy(guard);bad['npc']['territory']['radius']=1
        c.call('upsert',entity=bad,expected_error=True);after=c.call('inspect');assert after==live,{k:(live.get(k),after.get(k)) for k in live if live.get(k)!=after.get(k)}
        saved=territory()['policy'];c.call('save_scene',path='artifacts/territory.scene.json');c.call('load_scene',path='artifacts/territory.scene.json');assert territory()['policy']==saved and territory()['territory']['returns']==0
        c.call('undo');c.call('redo');assert territory()['policy']==saved and territory()['territory']['returns']==0
        before=c.call('inspect');c.call('set_debug_view',npc_routes=False);c.call('capture',path='artifacts/territory-no-guides.png',width=1280,height=900);c.call('set_debug_view',npc_routes=True);c.call('capture',path='artifacts/territory-guides.png',width=1280,height=900);after=c.call('inspect');assert before['npcs']==after['npcs'] and before['runtime_positions']==after['runtime_positions'];assert (ROOT/'artifacts/territory-no-guides.png').read_bytes()!=(ROOT/'artifacts/territory-guides.png').read_bytes()
        checks.append('Atomic bounds/route validation, persistence/undo/redo reset and visual boundary toggles without simulation changes')

        demo=copy.deepcopy(guard);demo['transform']['position']=[-3,0.85,0];demo['npc']['speed']=1.5;demo['npc']['territory'].update(home=[-3,0],radius=5,arrival_distance=0.4,rearm_ticks=120);demo['npc']['awareness']['hearing']['range']=10;demo['npc']['patrol']['route']['points']=[[-5,-1],[-1,-1]]
        player['transform']['position']=[0,0.85,-2]
        screen={'id':'screen','name':'Wall','body':'fixed','color':[0.2,0.65,0.45],'transform':{'position':[-4,1.5,2],'size':[4,3,0.4]}}
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [demo,player,screen]]+[{'op':'set_camera','camera':{'position':[11,15,18],'target':[-3,0,0],'fov_degrees':48}}]);c.call('save_scene',path='examples/npc-territory.scene.json');c.call('step',ticks=1);assert c.call('inspect')['npcs']['guard']['observed_this_tick'];c.call('capture',path='artifacts/territory-demo.png',width=1280,height=900)
        c.call('save_replay',path='artifacts/territory-demo.replay.json');assert c.call('run_replay',path='artifacts/territory-demo.replay.json')['matched'];c.call('reset');assert territory()['territory']['returns']==0
        checks.append('MCP-authored playable home-radius demo, guide/home marker, normal acquisition and reset')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':c.call('inspect')['build_id'],'slow_requests':c.slow_requests};(ROOT/'artifacts/territory-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
