#!/usr/bin/env python3
"""Objective arbitration, native movement and authored multi-role demo through MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_targeting import actor


def objective(priority, kind, when='always', **parameters):
    return {'priority':priority,'when':when,'action':{'kind':kind,**parameters}}


def verify(binary,window):
    (ROOT/'artifacts').mkdir(exist_ok=True)
    c=Client(binary,window); checks=[]; samples=[]
    def inspect(id='guard'):return c.call('inspect_objectives')['npcs'][id]
    def emit(x):c.call('emit_noise',origin={'kind':'position','position':[x,0.85,0]},radius=10)
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-objectives-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':24,'depth':24,'seed':42,'obstacles':0})
        guard=actor('guard','guards',[0,0.85,0],[0.7,0.2,0.8])
        guard['npc'].update(enabled=True,speed=2,
            awareness={'mode':'perceptive','vision_range':12,'fov_degrees':120,'initial_facing_degrees':0,'memory_ticks':30,'search_ticks':10,'search_turn_degrees_per_second':90,'hearing':{'range':10}},
            patrol={'route':{'kind':'points','points':[[5,0],[-5,0]]},'traversal':'loop','wait_ticks':10,'travel_ticks':600,'retry_ticks':60,'arrival_distance':0.3},
            objectives=[objective(10,'respond','evidence'),objective(1,'patrol')])
        c.call('upsert',entity=guard);c.call('select_entity',id='guard');c.call('set_debug_view',npc_vision=True,npc_hearing=True,noise_events=True,npc_routes=True)
        c.call('step',ticks=5);assert inspect()['selection']['active']==1
        emit(-4);c.call('step',ticks=1);react=inspect();assert react['selection']['active']==0 and react['selection']['reason']=='higher_priority';samples.append(react)
        c.call('capture',path='artifacts/objectives-interrupted.png',width=1280,height=900)
        c.call('step',ticks=30);resumed=inspect();assert resumed['selection']['active']==1 and resumed['selection']['resumed'] and resumed['memory'] is None;samples.append(resumed)
        c.call('capture',path='artifacts/objectives-resumed.png',width=1280,height=900)
        c.call('save_replay',path='artifacts/objectives.replay.json');assert c.call('run_replay',path='artifacts/objectives.replay.json')['matched']
        live=c.call('inspect');prefix=c.call('inspect_replay',path='artifacts/objectives.replay.json',tick=6);assert prefix['snapshot']['npcs']['guard']['objectives']['active']==0 and c.call('inspect')==live
        checks.append('Patrol interruption, same-point resumption after memory expiry, capture, full replay and isolated prefix')

        bad=copy.deepcopy(guard);bad['npc']['objectives']=[];c.call('upsert',entity=bad,expected_error=True);after=c.call('inspect');assert after==live,{k:(live.get(k),after.get(k)) for k in live if live.get(k)!=after.get(k)}
        fixed=copy.deepcopy(guard);fixed['npc']['objectives']=[objective(20,'guard',point=[5,0],arrival_distance=0.3),objective(10,'respond','evidence')]
        c.call('upsert',entity=fixed);saved_policy=inspect()['policy'];emit(-4);c.call('step',ticks=12);held=inspect();assert held['selection']['active']==0 and held['memory']['source']=='sound';assert c.call('inspect')['npcs']['guard']['investigation_distance']==0
        c.call('step',ticks=20);assert inspect()['memory'] is None
        c.call('save_scene',path='artifacts/objectives.scene.json');c.call('load_scene',path='artifacts/objectives.scene.json');assert inspect()['selection']['active'] is None
        c.call('undo');assert inspect()['selection']['active'] is None;c.call('redo');assert inspect()['policy']==saved_policy
        checks.append('Guard keeps its post while hearing, no investigation travel charged, expiry, atomic error and save/load/history')

        flee=copy.deepcopy(guard);flee['npc']['objectives']=[objective(10,'flee','evidence',distance=4),objective(1,'wait')]
        c.call('upsert',entity=flee);emit(2);c.call('step',ticks=20);flee_state=inspect();assert flee_state['selection']['action']=='flee' and flee_state['selection']['destination'][0]==-2
        assert c.call('inspect')['runtime_positions']['guard'][0] < -0.2
        c.call('save_replay',path='artifacts/objectives-flee.replay.json');assert c.call('run_replay',path='artifacts/objectives-flee.replay.json')['matched'];samples.append(flee_state)
        c.call('step',ticks=11);assert inspect()['selection']['action']=='wait'
        blocked=copy.deepcopy(flee);blocked['npc']['objectives'][0]['action']['distance']=32
        c.call('upsert',entity=blocked);emit(2);c.call('step',ticks=2);assert inspect()['navigation']['status']=='unreachable' and inspect()['selection']['action']=='flee'
        checks.append('Flee moves away from anonymous cue, deterministic replay, wait after expiry and explicit unreachable destination')

        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');player['transform']['position']=[0,0.85,-2];player.pop('footsteps',None)
        demo=copy.deepcopy(guard);demo['transform']['position']=[-4,0.85,3];demo['npc']['speed']=1.5;demo['npc']['target']='player'
        demo['npc']['awareness'].update(memory_ticks=180,search_ticks=90)
        demo['npc']['patrol']['route']['points']=[[-4,3],[-4,6],[0,6]]
        civil=actor('civilian','civilians',[2,0.85,1],[0.1,0.75,0.85]);civil['npc'].update(enabled=True,target='player',speed=2,
            awareness={'mode':'perceptive','vision_range':8,'fov_degrees':360,'initial_facing_degrees':0,'memory_ticks':120,'search_ticks':60,'search_turn_degrees_per_second':90},
            objectives=[objective(10,'flee','visual_memory',distance=6),objective(1,'guard',point=[2,1],arrival_distance=0.3)])
        ally=actor('ally','friends',[-4,0.85,-4],[0.15,0.3,0.9]);ally['npc'].update(enabled=True,target='player',speed=1.5,stop_distance=2,
            awareness={'mode':'perceptive','vision_range':12,'fov_degrees':360,'initial_facing_degrees':0,'memory_ticks':180,'search_ticks':60,'search_turn_degrees_per_second':90},
            objectives=[objective(10,'respond','visual_memory'),objective(1,'wait')])
        wall={'id':'screen','name':'Wall','body':'fixed','color':[0.2,0.65,0.45],'transform':{'position':[-3,1.5,0],'size':[6,3,0.4]}}
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [player,demo,civil,ally,wall]])
        c.call('save_scene',path='examples/npc-objectives.scene.json');c.call('step',ticks=30)
        roles={id:inspect(id)['selection']['action'] for id in ['guard','civilian','ally']};assert roles=={'guard':'patrol','civilian':'flee','ally':'respond'},roles
        c.call('capture',path='artifacts/objectives-demo.png',width=1280,height=900)
        c.call('emit_noise',origin={'kind':'entity','id':'player'},radius=12);c.call('step',ticks=1);assert inspect()['selection']['action']=='respond'
        c.call('save_replay',path='artifacts/objectives-demo.replay.json');assert c.call('run_replay',path='artifacts/objectives-demo.replay.json')['matched']
        c.call('reset');assert all(v['selection']['active'] is None for v in c.call('inspect_objectives')['npcs'].values())
        checks.append('MCP-authored guard/civilian/ally roles, simultaneous patrol/flee/follow, noise interruption, replay and reset')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':c.call('inspect')['build_id'],'slow_requests':c.slow_requests}
        (ROOT/'artifacts/objectives-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
