#!/usr/bin/env python3
"""Partial paths through the same MCP contract used by the native demo."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client,ROOT

def verify(binary,window):
    c=Client(binary,window);checks=[];samples=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-partial-route-verifier','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/npc-mobility.scene.json');scene=c.call('inspect')['scene'];walker=next(e for e in scene['entities'] if e['id']=='walker');network=scene['spatial_network']
        base=[-2.5,0,-7]
        if base not in network['nodes']:network['nodes'].append(base)
        node=network['nodes'].index(base)
        for link in [{'from':9,'to':node,'mode':'walk'},{'from':node,'to':10,'mode':'jump'}]:
            if link not in network['links']:network['links'].append(link)
        walker['npc'].pop('unreachable_policy',None)
        c.call('edit_scene',edits=[{'op':'set_spatial_network','network':network},{'op':'upsert','entity':walker}]);c.call('step',ticks=180);stop=c.call('inspect');assert stop['mobility']['walker']['completed']==0 and abs(stop['runtime_positions']['walker'][0]+5)<0.05
        walker['npc']['unreachable_policy']={'mode':'approach_reachable','min_progress':0.25,'max_path_ticks':300};c.call('upsert',entity=walker);c.call('set_camera',camera={'position':[-14,16,-18],'target':[0,2,0],'fov_degrees':48});c.call('set_debug_view',npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False);c.call('select_entity',id='walker');c.call('save_scene',path='examples/npc-mobility.scene.json')
        c.call('step',ticks=1);initial=c.call('inspect_mobility');assert initial['states']['walker']['partial_target']==node;assert all(abs(a-b)<1e-6 for a,b in zip(initial['states']['walker']['destination'],[-1,1,-7])),initial['states']['walker'];samples.append(initial['states']['walker'])
        c.call('step',ticks=59);reached=c.call('inspect');s=reached['mobility']['walker'];assert s['phase']=='partial_reached' and s['completed']==1 and s['plans']==1,s
        assert -2.65<reached['runtime_positions']['walker'][0]<-2.4 and reached['runtime_positions']['walker'][1]<0.85
        assert reached['navigation']['walker']['status']=='unreachable' and reached['navigation']['walker']['reason']=='partial_route_end'
        c.call('capture',path='artifacts/partial-route.png',width=1280,height=900);c.call('save_replay',path='artifacts/partial-route.replay.json');assert c.call('run_replay',path='artifacts/partial-route.replay.json')['matched'];c.call('capture_replay',replay_path='artifacts/partial-route.replay.json',path='artifacts/partial-route-replay.png',width=1280,height=900);assert (ROOT/'artifacts/partial-route.png').read_bytes()==(ROOT/'artifacts/partial-route-replay.png').read_bytes()
        assert c.call('inspect_replay',path='artifacts/partial-route.replay.json',tick=1)['snapshot']['mobility']['walker']==initial['states']['walker']
        c.call('step',ticks=600);end=c.call('inspect');assert end['mobility']['walker']['plans']==1 and end['mobility']['walker']['completed']==1 and end['mobility']['walker']['phase']=='partial_reached';samples.append(end['mobility']['walker'])
        checks.append('Default stop versus physical partial approach; original destination preserved, terminal state is not success, no stationary replanning, exact replay/prefix/PNG')
        blocked=copy.deepcopy(walker);blocked['npc']['unreachable_policy']['max_path_ticks']=1;c.call('upsert',entity=blocked);c.call('step',ticks=1);assert c.call('inspect')['mobility']['walker']['reason']=='no_partial_progress'
        blocked['npc']['unreachable_policy'].update(max_path_ticks=300,min_progress=5);c.call('upsert',entity=blocked);c.call('step',ticks=1);assert c.call('inspect')['mobility']['walker']['reason']=='no_partial_progress'
        before=c.call('inspect');bad=copy.deepcopy(walker);bad['npc']['unreachable_policy']['min_progress']=0;c.call('upsert',entity=bad,expected_error=True);assert c.call('inspect')==before
        c.call('load_scene',path='examples/npc-mobility.scene.json');c.call('undo');c.call('redo');assert c.call('inspect_mobility')['policies']['walker']['mode']=='approach_reachable';assert c.call('inspect')['mobility']['walker']['plans']==0
        checks.append('MCP effort/progress limits, atomic invalid edits and policy save/load/undo/redo/reset')
        c.call('step',ticks=60);before=c.call('inspect');c.call('set_debug_view',npc_routes=False);c.call('capture',path='artifacts/partial-route-no-guides.png',width=1280,height=900);c.call('set_debug_view',npc_routes=True);after=c.call('inspect');assert before['mobility']==after['mobility'] and before['runtime_positions']==after['runtime_positions'];assert (ROOT/'artifacts/partial-route-no-guides.png').read_bytes()!=(ROOT/'artifacts/partial-route.png').read_bytes();c.call('reset')
        checks.append('Partial endpoint marker and original goal share existing MCP guides without affecting simulation')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':c.call('inspect')['build_id'],'slow_requests':c.slow_requests};(ROOT/'artifacts/partial-route-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
