#!/usr/bin/env python3
"""Exercise gradual, individual visual confirmation through real MCP and native rendering."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_targeting import actor


def verify(binary, window):
    (ROOT/'artifacts').mkdir(exist_ok=True); c=Client(binary,window); checks=[]; samples=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-recognition-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':16,'depth':14,'seed':42,'obstacles':0})
        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player')
        player['faction']='intruders';player['transform']['position']=[-2,0.85,-3]
        guard=actor('guard','guards',[0,0.85,3],[0.7,0.2,0.8]);guard['npc'].update(enabled=True,speed=1.5,
            targeting={'kind':'hostile_nearest','switch_margin':0.2},recognition={'acquire_ticks':60,'decay_ticks':120},
            awareness={'mode':'perceptive','vision_range':12,'fov_degrees':120,'initial_facing_degrees':0,'memory_ticks':180,'search_ticks':90,'search_turn_degrees_per_second':90,'hearing':{'range':10}})
        other=actor('other','intruders',[2,0.85,-3],[0.9,0.2,0.1])
        c.call('edit_scene',edits=[*[{'op':'upsert','entity':e} for e in [player,guard,other]],{'op':'set_faction_relations','relations':[{'from':'guards','to':'intruders','attitude':'hostile'}]}])
        c.call('set_debug_view',npc_vision=True,npc_hearing=False,noise_events=False,npc_routes=False);c.call('select_entity',id='guard')
        c.call('step',ticks=59);pending=c.call('inspect');knowledge=c.call('inspect_knowledge')['npcs']['guard'];samples.append(knowledge)
        assert pending['npcs']['guard']['memory'] is None and pending['npcs']['guard']['targeting']['reason']=='awaiting_confirmation'
        assert len(knowledge['tracks'])==2 and all(not t['confirmed'] and t['evidence']==59*120 for t in knowledge['tracks'])
        assert knowledge['tracks']==pending['npcs']['guard']['knowledge']
        c.call('capture',path='artifacts/recognition-pending.png',width=1280,height=900)
        c.call('step',ticks=1);confirmed=c.call('inspect');assert confirmed['npcs']['guard']['memory']['actor_id']=='other'
        assert all(t['confirmed'] and t['progress']==1 for t in confirmed['npcs']['guard']['knowledge'])
        c.call('capture',path='artifacts/recognition-confirmed.png',width=1280,height=900)
        assert (ROOT/'artifacts/recognition-pending.png').read_bytes()!=(ROOT/'artifacts/recognition-confirmed.png').read_bytes()
        c.call('save_replay',path='artifacts/recognition.replay.json');assert c.call('run_replay',path='artifacts/recognition.replay.json')['matched']
        live=c.call('inspect');prefix=c.call('inspect_replay',path='artifacts/recognition.replay.json',tick=59)
        assert prefix['snapshot']['npcs']['guard']['knowledge']==pending['npcs']['guard']['knowledge'] and c.call('inspect')==live
        c.call('capture_replay',path='artifacts/recognition-prefix.png',replay_path='artifacts/recognition.replay.json',tick=59,width=1280,height=900)
        assert (ROOT/'artifacts/recognition-prefix.png').read_bytes()==(ROOT/'artifacts/recognition-pending.png').read_bytes()
        checks.append('Two candidates accumulate independently, exact confirmation boundary, diagnostic color and isolated replay/pixel equality')

        bad=copy.deepcopy(guard);bad['npc']['recognition']['decay_ticks']=0
        c.call('upsert',entity=bad,expected_error=True);after=c.call('inspect');assert after==live, {k:(live.get(k),after.get(k)) for k in live if live.get(k)!=after.get(k)}
        immediate=copy.deepcopy(guard);immediate['npc'].pop('recognition')
        c.call('upsert',entity=immediate);c.call('step',ticks=1);assert c.call('inspect')['npcs']['guard']['observed_this_tick']
        c.call('undo');c.call('step',ticks=1);assert not c.call('inspect')['npcs']['guard']['observed_this_tick']
        c.call('redo');c.call('step',ticks=1);assert c.call('inspect')['npcs']['guard']['observed_this_tick']
        c.call('upsert',entity=guard);c.call('save_scene',path='artifacts/recognition.scene.json');c.call('step',ticks=30)
        c.call('load_scene',path='artifacts/recognition.scene.json');assert c.call('inspect_knowledge')['npcs']['guard']['tracks']==[]
        checks.append('Invalid edit atomicity, legacy immediate reaction, undo/redo and saved configuration with clean runtime evidence')

        c.call('emit_noise',origin={'kind':'entity','id':'player'},radius=10);c.call('step',ticks=1)
        heard=c.call('inspect')['npcs']['guard'];assert heard['memory']['source']=='sound' and heard['memory']['actor_id'] is None
        assert all(not t['confirmed'] and t['evidence']==120 for t in heard['knowledge'])
        omni=copy.deepcopy(guard);omni['npc']['awareness']={'mode':'omniscient'}
        c.call('upsert',entity=omni);c.call('step',ticks=1);assert all(t['confirmed'] for t in c.call('inspect_knowledge')['npcs']['guard']['tracks'])
        c.call('upsert',entity=guard)
        checks.append('Sound investigates without confirming actor; explicit omniscience bypasses visual delay')

        # Clear two-actor demo: player starts visible and stationary. Geometry permits escaping behind a wall.
        guard['npc']['recognition']={'acquire_ticks':120,'decay_ticks':240}
        player['transform']['position']=[0,0.85,-1]
        player['footsteps']={'surface':'floor','stride':0.6,'radius':8,'min_speed':0.1}
        screen={'id':'screen','name':'Occlusion wall','body':'fixed','color':[0.2,0.65,0.45],'transform':{'position':[0,1.5,-3],'size':[8,3,0.4]}}
        c.call('edit_scene',edits=[{'op':'remove','id':'other'},*[{'op':'upsert','entity':e} for e in [guard,player,screen]]])
        c.call('save_scene',path='examples/npc-recognition.scene.json');c.call('step',ticks=60)
        demo_pending=c.call('inspect_knowledge')['npcs']['guard'];assert demo_pending['tracks'][0]['progress']==0.5
        c.call('capture',path='artifacts/recognition-demo-pending.png',width=1280,height=900)
        c.call('step',ticks=60);assert c.call('inspect')['npcs']['guard']['observed_this_tick']
        c.call('capture',path='artifacts/recognition-demo-confirmed.png',width=1280,height=900)
        c.call('save_replay',path='artifacts/recognition-demo.replay.json');assert c.call('run_replay',path='artifacts/recognition-demo.replay.json')['matched']
        c.call('reset');assert c.call('inspect_knowledge')['npcs']['guard']['tracks']==[]
        checks.append('MCP-authored two-actor demo, 2-second visual acquisition, wall, footsteps, reset and replay')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':c.call('inspect')['build_id'],'slow_requests':c.slow_requests}
        (ROOT/'artifacts/recognition-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
