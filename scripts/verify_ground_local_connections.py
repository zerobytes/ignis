#!/usr/bin/env python3
"""Bounded KCC connectors via MCP, without desktop windows or physical input."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_geometry import box
from verify_ground_navigation import npc


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-ground-local-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        def fixture(ramp=False,detour=False,speed=2,budget=False,pursuit=False):
            c.call('generate',recipe={'width':30,'depth':20,'seed':42,'obstacles':0})
            n=npc('guard',[-4,0.81,0] if detour else [-3,0.81,0],[3,2.8 if ramp else 0.8,0])
            n['npc']['local_connections']={'walk':True,'fly':False,'max_distance':2.85 if detour else 8}
            n['npc']['speed']=speed
            p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[5,2.81 if ramp else 0.8,0] if pursuit else [-9,0.8,7];p.pop('footsteps',None)
            if ramp:p['character_controller']={'snap_distance':0.3,'step':{'max_height':0.3,'min_width':0.4}}
            if pursuit:
                n['npc'].pop('objectives');n['npc']['target']='player'
                n['npc']['stop_distance']=0.9
                n['npc']['awareness']={'mode':'perceptive','vision_range':25,'fov_degrees':360,'initial_facing_degrees':0,'memory_ticks':120,'search_ticks':60,'search_turn_degrees_per_second':60,'hearing':{'range':12}}
            net={'nodes':[],'links':[]};terrain=[]
            if ramp:
                terrain=[box('ramp',[0,1,0],[6,2,5],geometry={'kind':'wedge','rise':'positive_x'},color=(0.2,0.7,0.4)),box('landing',[6,1,0],[6,2,5],color=(0.25,0.55,0.65))]
                n['transform']['position']=[-4,0.81,0];n['npc']['objectives']=[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[3.5,2.8,0],'arrival_distance':0.2}}] if not pursuit else n['npc'].get('objectives')
                if pursuit:n['npc'].pop('objectives',None)
                n['npc']['local_connections']['max_distance']=8
                if pursuit:net={'nodes':[[-3.8,0,1.5],[4.5,2,1.5]],'links':[{'from':0,'to':1,'mode':'walk'}]}
            if detour:
                net={'nodes':[[-2,0,-2],[2,0,-2]],'links':[{'from':0,'to':1,'mode':'walk'}]}
                terrain=[box('wall',[0,1.5,0],[1,3,2.5])]
            if budget:net['nodes']=[[-2+i*0.2,0,1] for i in range(12)]
            c.call('edit_scene',edits=[{'op':'set_spatial_network','network':net}]+[{'op':'upsert','entity':e} for e in [p,n]+terrain]+[{'op':'set_camera','camera':{'position':[16,15,22],'target':[1,1,0],'fov_degrees':48}}])
            return n
        fixture(detour=True)
        kinds=set();edges=False
        for _ in range(60):
            c.call('step',ticks=10);s=c.call('inspect_mobility')['states']['guard'];edges|=s['edge'] is not None
            if s['connector']:kinds.add(s['connector']['kind'])
            assert s['ground_planning']['probes']<=8 and s['ground_planning']['steps']<=2400
        assert s['phase']=='arrived' and {'entry','exit'}<=kinds and edges,s
        checks.append('Physical entry, authored wall detour and exit reach an off-node target without zero-length anchor loops; diagnostics remain bounded')
        fixture(speed=0.1,budget=True);c.call('step',ticks=1);s=c.call('inspect_mobility')['states']['guard'];assert s['reason']=='ground_local_budget' and s['ground_planning']['limited'];plans=s['plans'];c.call('step',ticks=20);assert c.call('inspect_mobility')['states']['guard']['plans']==plans
        path='artifacts/ground-local-budget.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        checks.append('Probe/candidate exhaustion is observable and replayable; stable goals do not rescan while waiting for retry')
        n=fixture(ramp=True);before=c.call('inspect');bad=copy.deepcopy(n);bad['npc']['local_connections']['fly']=True;c.call('upsert',entity=bad,expected_error=True);assert c.call('inspect')==before
        c.call('step',ticks=400);assert c.call('inspect_mobility')['states']['guard']['phase']=='arrived'
        c.call('reset');wall=box('ceiling',[0,2.1,0],[6,0.3,5]);c.call('upsert',entity=wall);c.call('step',ticks=400);assert c.call('inspect_mobility')['states']['guard']['phase']!='arrived';c.call('undo');c.call('step',ticks=400);assert c.call('inspect_mobility')['states']['guard']['phase']=='arrived'
        checks.append('Direct ramp walking uses controller geometry and headroom; invalid combinations are atomic and undo rebuilds admissions')
        fixture(pursuit=True);c.call('step',ticks=2);original=c.call('inspect_mobility')['states']['guard']['connector']['endpoint'];c.call('set_input',input={'x':0,'z':1});c.call('step',ticks=25);s=c.call('inspect_mobility')['states']['guard'];assert s['committed'] and s['connector']['endpoint']==original;c.call('set_input',input={'x':0,'z':0});c.call('step',ticks=500);assert c.call('inspect_mobility')['states']['guard']['phase']=='arrived'
        checks.append('Perceptive pursuit freezes the active endpoint, then follows updated visual evidence outside graph nodes')
        fixture(ramp=True,pursuit=True)
        c.call('set_debug_view',npc_routes=True,npc_vision=True,npc_hearing=True,noise_events=True)
        c.call('save_scene',path='examples/npc-ground-local.scene.json');c.call('load_scene',path='examples/npc-ground-local.scene.json');initial=c.call('inspect');assert initial['scene']['version']==SCENE_VERSION
        c.call('step',ticks=100);prefix=c.call('inspect');c.call('capture',path='artifacts/ground-local-prefix.png',width=1000,height=700)
        c.call('step',ticks=500);final=c.call('inspect');assert final['mobility']['guard']['phase']=='arrived',final['mobility']
        path='artifacts/ground-local.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        recovered=c.call('inspect_replay',path=path,tick=100)['snapshot']
        for key in ['runtime_positions','mobility','character_controllers']:assert recovered[key]==prefix[key]
        c.call('capture_replay',replay_path=path,path='artifacts/ground-local-prefix-replay.png',tick=100,width=1000,height=700)
        assert (ROOT/'artifacts/ground-local-prefix.png').read_bytes()==(ROOT/'artifacts/ground-local-prefix-replay.png').read_bytes()
        c.call('capture',path='artifacts/ground-local-final.png',width=1000,height=700);c.call('capture_replay',replay_path=path,path='artifacts/ground-local-final-replay.png',width=1000,height=700)
        assert (ROOT/'artifacts/ground-local-final.png').read_bytes()==(ROOT/'artifacts/ground-local-final-replay.png').read_bytes()
        data=json.loads((ROOT/path).read_text());data['expected_mobility']['guard']['ground_planning']['probes']=99;tampered='artifacts/ground-local-tampered.replay.json';(ROOT/tampered).write_text(json.dumps(data));assert not c.call('run_replay',path=tampered)['matched'];assert c.call('inspect')==final
        c.call('reset');assert c.call('inspect')['mobility']==initial['mobility'];c.call('load_scene',path='examples/npc-ground-local.scene.json');assert c.call('inspect')['character_controllers']==initial['character_controllers']
        checks.append('MCP-authored perceptive ramp pursuit demo, same-controller player, replay/prefix/tamper, identical offscreen GPU pixels and reset/load; no desktop focus')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/ground-local-connections-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
