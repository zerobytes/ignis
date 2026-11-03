#!/usr/bin/env python3
"""Author and exercise physical 3D NPC capabilities through MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT

def verify(binary,window):
    c=Client(binary,window);checks=[];samples=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-spatial-verifier','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':20,'depth':18,'seed':42,'obstacles':0})
        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');player['transform']['position']=[-8,0.85,7];player.pop('footsteps',None)
        nodes=[[-5,0,-4],[-1,1,-4],[3,2,-4],[-5,0,0],[-5,5,0],[4,5,0],[4,0,0],[-5,6,4],[4,0,4],[-5,0,-7],[-1,1,-7]]
        links=[(0,1,'jump'),(1,2,'jump'),(3,4,'fly'),(4,5,'fly'),(5,6,'fly'),(7,8,'glide'),(9,10,'jump')]
        network={'nodes':nodes,'links':[{'from':a,'to':b,'mode':m} for a,b,m in links]}
        def npc(id,start,end,color,**cap):
            p=nodes[start];q=nodes[end]
            return {'id':id,'name':id,'body':'npc','color':color,'transform':{'position':[p[0],p[1]+0.8,p[2]],'size':[0.6,1.6,0.6]},'npc':{'surface':'floor','speed':4,'stop_distance':0.2,'enabled':True,'mobility':{'walk':True,'jump_speed':None,'fly_speed':None,'glide':None,**cap},'objectives':[{'priority':1,'when':'always','action':{'kind':'guard_3d','point':[q[0],q[1]+0.8,q[2]],'arrival_distance':0.2}}]}}
        actors=[npc('jumper',0,2,[0.75,0.15,0.85],jump_speed=7),npc('flyer',3,6,[0.1,0.6,1],fly_speed=3),npc('glider',7,8,[0.15,0.9,0.35],glide={'speed':3,'sink_speed':1}),npc('walker',9,10,[0.7,0.7,0.7])]
        def box(id,p,size,color):return {'id':id,'name':id,'body':'fixed','color':color,'transform':{'position':p,'size':size}}
        geometry=[box('step_one',[-0.5,0.5,-5.5],[3,1,6],[0.45,0.38,0.25]),box('step_two',[3.5,1,-4],[3,2,3],[0.5,0.4,0.25]),box('flight_wall',[0,1.5,0],[1,3,2.5],[0.2,0.6,0.65]),box('glider_perch',[-6.5,3,4],[2,6,2],[0.2,0.45,0.3])]
        edits=[{'op':'set_spatial_network','network':network}]+[{'op':'upsert','entity':e} for e in [player,*geometry,*actors]]+[{'op':'set_camera','camera':{'position':[14,14,17],'target':[0,2,0],'fov_degrees':48}}]
        c.call('edit_scene',edits=edits);c.call('set_debug_view',npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False);c.call('select_entity',id='jumper');c.call('save_scene',path='examples/npc-mobility.scene.json')
        c.call('step',ticks=1);initial=c.call('inspect_mobility');samples.append(initial)
        for id in ['jumper','flyer','glider']:assert initial['states'][id]['phase']=='traversing',(id,initial)
        assert initial['states']['walker']['phase']=='unreachable' and initial['rejection_reasons']['walker'][6]=='missing_capability'
        c.call('step',ticks=39);air=c.call('inspect');samples.append(air['mobility']);assert air['runtime_positions']['jumper'][1]>2.5 and air['runtime_positions']['flyer'][1]>2.5 and air['runtime_positions']['glider'][1]<6.3
        c.call('capture',path='artifacts/mobility-airborne.png',width=1280,height=900);c.call('save_replay',path='artifacts/mobility-airborne.replay.json');assert c.call('run_replay',path='artifacts/mobility-airborne.replay.json')['matched'];c.call('capture_replay',replay_path='artifacts/mobility-airborne.replay.json',path='artifacts/mobility-airborne-replay.png',width=1280,height=900);assert (ROOT/'artifacts/mobility-airborne.png').read_bytes()==(ROOT/'artifacts/mobility-airborne-replay.png').read_bytes()
        c.call('step',ticks=560);end=c.call('inspect');samples.append(end['mobility'])
        for id,count in [('jumper',2),('flyer',3),('glider',1)]:assert end['mobility'][id]['phase']=='arrived' and end['mobility'][id]['completed']==count,(id,end)
        assert end['runtime_positions']['jumper'][1]>2.7 and abs(end['runtime_positions']['walker'][0]+5)<0.05
        c.call('save_replay',path='artifacts/mobility.replay.json');assert c.call('run_replay',path='artifacts/mobility.replay.json')['matched'];assert c.call('inspect_replay',path='artifacts/mobility.replay.json',tick=40)['snapshot']['mobility']==air['mobility']
        checks.append('Physical jump/platforms, flight above obstacle, descending glide, missing capability, replay prefix and exact GPU PNG')
        before=c.call('inspect');bad=copy.deepcopy(network);bad['links'][0]['to']=63;c.call('set_spatial_network',network=bad,expected_error=True);assert c.call('inspect')==before
        c.call('set_spatial_network',network={**network,'links':[]});c.call('step',ticks=1);assert all(s['phase']=='unreachable' for s in c.call('inspect_mobility')['states'].values());c.call('undo');c.call('redo');c.call('load_scene',path='examples/npc-mobility.scene.json');assert c.call('inspect')['scene']['version']==SCENE_VERSION
        checks.append('Atomic network/capability authoring, invalid references, save/load/undo/redo and legacy normalization')
        c.call('step',ticks=40);before=c.call('inspect');c.call('set_debug_view',npc_routes=False);c.call('capture',path='artifacts/mobility-no-guides.png',width=1280,height=900);c.call('set_debug_view',npc_routes=True);after=c.call('inspect');assert before['mobility']==after['mobility'] and before['runtime_positions']==after['runtime_positions'];assert (ROOT/'artifacts/mobility-no-guides.png').read_bytes()!=(ROOT/'artifacts/mobility-airborne.png').read_bytes();c.call('reset')
        checks.append('XYZ route and ballistic arc guides share MCP control without changing physics or decisions; reset clears runtime')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':c.call('inspect')['build_id'],'slow_requests':c.slow_requests};(ROOT/'artifacts/mobility-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
