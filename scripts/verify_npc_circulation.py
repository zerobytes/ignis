#!/usr/bin/env python3
"""Critical physical/knowledge boundary of anonymous circulation through the MCP."""
import argparse
import json
from pathlib import Path
from verify_lab import Client,ROOT
from verify_geometry import box

def actor(name,x,goal):
    e=box(name,[x,0.81,0],[0.6,1.6,0.6],body='npc',color=(0.7,0.2,0.8) if x<0 else (0.2,0.6,0.9))
    e['character_controller']={'snap_distance':0.3}
    e['npc']={'surface':'floor','speed':2,'stop_distance':0.2,'enabled':True,'mobility':{'walk':True},'local_connections':{'walk':True,'fly':False,'max_distance':3},'mind':{'intelligence':1},'circulation':{},'objectives':[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[goal,0.8,0],'arrival_distance':0.2}}]}
    return e

def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-circulation','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':24,'depth':20,'seed':42,'obstacles':0})
        p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[-8,0.8,7];p.pop('footsteps',None)
        a,b=actor('a',-4,4),actor('b',4,-4)
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [p,a,b]])
        c.call('set_spatial_network',network={'nodes':[[-4,0,0],[4,0,0]],'links':[{'from':0,'to':1,'mode':'walk'},{'from':1,'to':0,'mode':'walk'}]})
        c.call('set_debug_view',npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call('save_scene',path='examples/npc-circulation.scene.json')
        avoiding=False
        for _ in range(8):
            c.call('step',ticks=30);states=c.call('inspect_npc_circulation')['npcs'];avoiding |= any(v['state']['reason']=='avoiding' for v in states.values())
        prefix=c.call('inspect');c.call('capture',path='artifacts/npc-circulation-prefix.png',width=1000,height=700)
        c.call('step',ticks=240);now=c.call('inspect')['runtime_positions'];assert avoiding and now['a'][0]>2 and now['b'][0]<-2,now
        assert all(v['memory'] is None for v in c.call('inspect_npcs')['npcs'].values())
        path='artifacts/npc-circulation.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        recovered=c.call('inspect_replay',path=path,tick=240)['snapshot'];assert recovered['runtime_positions']==prefix['runtime_positions'] and recovered['npcs']==prefix['npcs']
        c.call('capture_replay',replay_path=path,path='artifacts/npc-circulation-prefix-replay.png',tick=240,width=1000,height=700)
        assert (ROOT/'artifacts/npc-circulation-prefix.png').read_bytes()==(ROOT/'artifacts/npc-circulation-prefix-replay.png').read_bytes()
        checks.append('Opposing grounded NPCs pass physically with bounded velocities, anonymous knowledge, same individual and exact replay/prefix/offscreen capture')
        a['transform']['position'][0]=-0.8;b['transform']['position'][0]=0.8
        wall=box('wall',[0,2,0],[0.4,4,8])
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [a,b,wall]]);c.call('step',ticks=1)
        assert all(v['state']['neighbors']==0 for v in c.call('inspect_npc_circulation')['npcs'].values())
        before=c.call('inspect');a['npc']['circulation']['radius']=0;c.call('upsert',entity=a,expected_error=True);assert c.call('inspect')==before
        checks.append('Fixed wall occludes anonymous proximity and invalid configuration preserves the entire running state')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/npc-circulation-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
