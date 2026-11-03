#!/usr/bin/env python3
"""Critical integration: traits influence decisions without bypassing evidence/replay/identity."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT, SCENE_VERSION
from verify_geometry import box


def actor(name, z, brave, intelligence=0.5):
    e=box(name,[-3,0.8,z],[0.6,1.6,0.6],body='npc',color=(0.2,0.6,0.9) if brave else (0.85,0.4,0.65))
    e['npc']={'surface':'floor','speed':2,'stop_distance':0.8,'enabled':True,'target':'player',
        'awareness':{'mode':'perceptive','vision_range':20,'fov_degrees':360,'initial_facing_degrees':0,'memory_ticks':120,'search_ticks':60,'search_turn_degrees_per_second':60},
        'mind':{'intelligence':intelligence,'seed':42,'personality':{'courage':1 if brave else 0.1,'fear':0.1 if brave else 1,'anxiety':0.2 if brave else 0.9,'caution':0.7 if brave else 0.9,'aggression':0.7 if brave else 0.1}},
        'objectives':[{'priority':1,'when':'evidence','action':{'kind':'respond'}},{'priority':1,'when':'evidence','action':{'kind':'flee','distance':4}}]}
    return e


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-npc-mind','version':'1'}});c.request('notifications/initialized',notification=True)
        def fixture(hidden=False):
            c.call('generate',recipe={'width':24,'depth':20,'seed':42,'obstacles':0})
            p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[3,0.8,0];p.pop('footsteps',None)
            brave=actor('brave',-2,True,0.9);fearful=actor('fearful',2,False,0.2)
            entities=[p,brave,fearful]
            if hidden:entities.append(box('occluder',[0,2,0],[1,4,16]))
            c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in entities])
            return brave,fearful
        brave,fearful=fixture();c.call('step',ticks=2);view=c.call('inspect_npc_mind');assert view['npcs']['brave']['decision']['chosen']==0 and view['npcs']['fearful']['decision']['chosen']==1,view
        before=c.call('inspect');bad=copy.deepcopy(brave);bad['npc']['mind']['intelligence']=2;c.call('upsert',entity=bad,expected_error=True);assert c.call('inspect')==before
        path='artifacts/npc-mind.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched'];data=json.loads((ROOT/path).read_text());data['expected_npcs']['brave']['mind_profile']['intelligence']=0;(ROOT/'artifacts/npc-mind-tampered.replay.json').write_text(json.dumps(data));assert not c.call('run_replay',path='artifacts/npc-mind-tampered.replay.json')['matched'];assert c.call('inspect')==before
        fixture(hidden=True);c.call('step',ticks=30);s=c.call('inspect_npcs')['npcs'];assert all(v['memory'] is None for v in s.values())
        checks.append('Different trait profiles select approach versus retreat; invalid edits are atomic, hidden target remains unknown and personality state is replay-checked')
        fixture();n=copy.deepcopy(brave);n['npc']['mind']['variation']=0.25;n['npc']['enabled']=False;c.call('set_npc_templates',templates=[{**n,'id':'template'}]);c.call('spawn_actor',template='template',position=[-6,0.8,4],target='player');c.call('step',ticks=1);s=c.call('inspect_npc_mind')['npcs'];born=next(k for k in s if k not in ['brave','fearful']);assert s[born]['profile'] is None
        # Enable after authoring reset cannot preserve the individual; template configured enabled exercises birth below.
        n['npc']['enabled']=True;c.call('set_npc_templates',templates=[{**n,'id':'template'}]);c.call('spawn_actor',template='template',position=[-6,0.8,4],target='player');c.call('step',ticks=2);s=c.call('inspect_npc_mind')['npcs'];born=next(k for k in s if k not in ['brave','fearful']);assert s[born]['profile']!=s['brave']['profile'];c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        checks.append('Template traits compile per born individual, disabled actor makes no decisions, seeded profile and new identity survive replay')
        c.call('generate',recipe={'width':24,'depth':20,'seed':42,'obstacles':0})
        p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[-8,0.8,6];p.pop('footsteps',None)
        entities=[p]
        for name,z,intelligence in [('low',-2,0),('high',2,1)]:
            n=actor(name,z,True,intelligence);n['character_controller']={};n['transform']['position'][1]=0.81;n['npc'].pop('target');n['npc']['mobility']={'walk':True};n['npc']['local_connections']={'walk':True,'fly':False,'max_distance':8};n['npc']['objectives']=[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[3,0.8,z],'arrival_distance':0.2}}];entities.append(n)
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in entities]);c.call('step',ticks=1);s=c.call('inspect_npc_mind')['npcs'];assert s['low']['planning']['step_limit']==600 and s['high']['planning']['step_limit']==2400
        checks.append('Effective intelligence controls bounded KCC planning effort with visible limits, without changing geometry or movement capacity')
        fixture();c.call('set_debug_view',npc_routes=True,npc_vision=True,npc_hearing=False,noise_events=False);c.call('save_scene',path='examples/npc-mind.scene.json');c.call('load_scene',path='examples/npc-mind.scene.json');assert c.call('inspect')['scene']['version']==SCENE_VERSION
        c.call('step',ticks=30);prefix=c.call('inspect');c.call('capture',path='artifacts/npc-mind-prefix.png',width=1000,height=700);c.call('step',ticks=90);c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched'];recovered=c.call('inspect_replay',path=path,tick=30)['snapshot'];assert recovered['npcs']==prefix['npcs'];c.call('capture_replay',replay_path=path,path='artifacts/npc-mind-prefix-replay.png',tick=30,width=1000,height=700);assert (ROOT/'artifacts/npc-mind-prefix.png').read_bytes()==(ROOT/'artifacts/npc-mind-prefix-replay.png').read_bytes()
        checks.append('MCP-authored brave/fearful demo has deterministic decision diagnostics, prefix state and matching offscreen replay pixels')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/npc-mind-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
