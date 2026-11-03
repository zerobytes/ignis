#!/usr/bin/env python3
"""Critical integration of KCC/dynamic handoff on the same actor via real MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client,ROOT
from verify_geometry import box


def fixture(c,fly=True,local=False):
    c.call('generate',recipe={'width':30,'depth':20,'seed':42,'obstacles':0})
    p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[-10,0.8,7];p.pop('footsteps',None)
    n=box('hybrid',[-5,0.81,0],[0.6,1.6,0.6],body='npc',color=(0.7,0.2,0.8));n['character_controller']={'snap_distance':0.3,'step':{'max_height':0.3,'min_width':0.4}}
    n['npc']={'surface':'floor','speed':2,'stop_distance':0.2,'enabled':True,'mobility':{'walk':True},'mind':{'intelligence':1,'personality':{'courage':1,'fear':0.2,'caution':0.5}},'objectives':[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[5,2.8,0],'arrival_distance':0.2}}]}
    if fly:n['npc']['mobility']['fly_speed']=3
    if local:n['npc']['local_connections']={'walk':True,'fly':True,'max_distance':8}
    net={'nodes':[[-5,0,0],[-3,0,0],[-3,3,0],[3,3,0],[3,2,0],[5,2,0]],'links':[{'from':a,'to':b,'mode':mode} for a,b,mode in [(0,1,'walk'),(1,2,'fly'),(2,3,'fly'),(3,4,'fly'),(4,5,'walk')]]}
    c.call('edit_scene',edits=[{'op':'set_spatial_network','network':net}]+[{'op':'upsert','entity':e} for e in [p,n,box('platform',[4.5,1,0],[5,2,4],color=(0.2,0.7,0.5))]]+[{'op':'set_camera','camera':{'position':[17,15,23],'target':[0,1,0],'fov_degrees':48}}])
    return n


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-npc-hybrid','version':'1'}});c.request('notifications/initialized',notification=True)
        fixture(c);c.call('set_debug_view',npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False);c.call('save_scene',path='examples/npc-hybrid-mobility.scene.json');c.call('load_scene',path='examples/npc-hybrid-mobility.scene.json');identity=c.call('inspect_character_controllers')['controllers']['hybrid']['instance']
        seen=set()
        for _ in range(60):
            c.call('step',ticks=10);s=c.call('inspect');motor=s['mobility']['hybrid']['motor_mode'];seen.add(motor);controller=s['character_controllers']['hybrid'];assert controller['instance']==identity
            if controller['airborne']:assert not controller['grounded']
            if s['tick']==150:prefix=s;c.call('capture',path='artifacts/npc-hybrid-prefix.png',width=1000,height=700)
        assert {'walk','fly'}<=seen and s['mobility']['hybrid']['phase']=='arrived' and not controller['airborne'] and controller['grounded'],s['mobility']
        path='artifacts/npc-hybrid.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched'];recovered=c.call('inspect_replay',path=path,tick=150)['snapshot']
        for key in ['mobility','runtime_positions','character_controllers','npcs']:assert recovered[key]==prefix[key]
        c.call('capture_replay',replay_path=path,path='artifacts/npc-hybrid-prefix-replay.png',tick=150,width=1000,height=700);assert (ROOT/'artifacts/npc-hybrid-prefix.png').read_bytes()==(ROOT/'artifacts/npc-hybrid-prefix-replay.png').read_bytes()
        checks.append('One actor walks, takes off, flies, lands and resumes KCC walking with stable identity, physical contact, replay state and identical offscreen prefix pixels')
        fixture(c,fly=False);c.call('step',ticks=300);assert c.call('inspect_mobility')['states']['hybrid']['phase']!='arrived';assert not c.call('inspect_character_controllers')['controllers']['hybrid']['airborne']
        fixture(c,local=True);c.call('step',ticks=600);s=c.call('inspect_mobility')['states']['hybrid'];assert s['phase']=='arrived',s
        c.call('reset');assert c.call('inspect_mobility')['states']['hybrid']['motor_mode'] is None
        checks.append('Absent physical capability cannot traverse an aerial route; hybrid local walk/fly connectors choose valid modes and reset removes motor state')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/npc-hybrid-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
