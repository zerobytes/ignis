#!/usr/bin/env python3
"""Relations, perception-gated selection and persistence through the real MCP dispatcher."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def actor(id, faction, position, color):
    return {'id':id,'name':id,'body':'npc','faction':faction,'blocks_vision':False,'color':color,
            'transform':{'position':position,'size':[0.7,1.6,0.7]},
            'npc':{'surface':'floor','speed':1.2,'stop_distance':1.2,'enabled':False}}


def verify(binary, window):
    (ROOT/'artifacts').mkdir(exist_ok=True)
    c=Client(binary,window); checks=[]; samples=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-targeting-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':16,'depth':14,'seed':42,'obstacles':0})
        scene=c.call('inspect')['scene']; assert scene['version']==SCENE_VERSION
        player=copy.deepcopy(next(e for e in scene['entities'] if e['id']=='player'))
        player['faction']='players'; player['transform']['position']=[3,0.85,-3]
        guard=actor('guard','guards',[0,0.85,3],[0.7,0.2,0.8])
        guard['npc'].update(enabled=True,speed=0.1,targeting={'kind':'hostile_nearest','switch_margin':0.2},awareness={
            'mode':'perceptive','vision_range':12,'fov_degrees':360,'initial_facing_degrees':0,
            'memory_ticks':180,'search_ticks':90,'search_turn_degrees_per_second':90,'hearing':{'range':10}})
        raider=actor('raider','raiders',[-3,0.85,-3],[0.9,0.2,0.15])
        friend=actor('friend','guards',[0,0.85,1],[0.15,0.75,0.9])
        rules=[{'from':'guards','to':'raiders','attitude':'hostile'},{'from':'guards','to':'players','attitude':'hostile'}]
        c.call('edit_scene',edits=[*[{'op':'upsert','entity':e} for e in [player,guard,raider,friend]],{'op':'set_faction_relations','relations':rules}])
        c.call('set_debug_view',npc_vision=True,npc_hearing=True,noise_events=True,npc_routes=True)
        c.call('select_entity',id='guard')
        relations=c.call('inspect_relations')
        attitudes={(r['from'],r['to']):r['attitude'] for r in relations['resolved']}
        assert attitudes['guard','friend']=='allied' and attitudes['guard','raider']=='hostile' and attitudes['raider','guard']=='neutral'
        prior=c.call('inspect')['runtime_positions'];c.call('step',ticks=1)
        first=c.call('inspect_targeting')['npcs']['guard'];samples.append(first)
        assert first['selection']['observed_actor']=='player' and first['selection']['observed_candidates']==2
        assert first['memory']['actor_id']=='player' and first['memory']['position']==prior['player']
        c.call('capture',path='artifacts/targeting-acquired.png',width=1280,height=900)
        checks.append('Directional rules/default alliance; nearer ally excluded; visible-hostile ID tie and pre-physics evidence')

        c.call('step',ticks=60);state=c.call('inspect');assert state['npcs']['guard']['targeting']['reason']=='retained'
        c.call('set_input',input={'x':1,'z':0})
        switched=False
        for _ in range(180):
            c.call('step',ticks=1)
            selection=c.call('inspect_targeting')['npcs']['guard']['selection']
            if selection['reason']=='switched_closer':
                assert selection['observed_actor']=='raider'
                assert selection['nearest_distance_squared'] < selection['incumbent_distance_squared'] * 0.8 * 0.8
                samples.append(selection);switched=True;break
        c.call('set_input',input={'x':0,'z':0});assert switched
        c.call('save_replay',path='artifacts/targeting.replay.json');assert c.call('run_replay',path='artifacts/targeting.replay.json')['matched']
        live=c.call('inspect');prefix=c.call('inspect_replay',path='artifacts/targeting.replay.json',tick=1)
        assert prefix['snapshot']['npcs']['guard']['memory']['actor_id']=='player' and c.call('inspect')==live
        c.call('capture_replay',path='artifacts/targeting-prefix.png',replay_path='artifacts/targeting.replay.json',tick=1,width=1280,height=900)
        assert c.call('inspect')==live
        checks.append('Retained then switched with live movement and margin, replay equality, isolated prefix identity and capture without changing the live session')

        c.call('set_faction_relations',relations=rules+rules,expected_error=True);assert c.call('inspect')==live
        bad=copy.deepcopy(guard);bad['npc']['target']='player'
        c.call('upsert',entity=bad,expected_error=True);assert c.call('inspect')==live
        c.call('set_faction_relations',relations=[]);c.call('step',ticks=1)
        assert c.call('inspect_targeting')['npcs']['guard']['selection']['reason']=='no_observed_hostile'
        c.call('undo');c.call('step',ticks=1);assert c.call('inspect_targeting')['npcs']['guard']['memory']['actor_id']=='player'
        c.call('redo');c.call('step',ticks=1);assert c.call('inspect_targeting')['npcs']['guard']['memory'] is None
        c.call('set_faction_relations',relations=rules)
        checks.append('Invalid mutation atomicity, exclusive policies, compiled relation invalidation and undo/redo')

        guard['npc']['awareness']['fov_degrees']=120
        c.call('upsert',entity=guard)
        player['transform']['position']=[-3,0.85,5];raider['transform']['position']=[3,0.85,5]
        c.call('edit_scene',edits=[{'op':'upsert','entity':player},{'op':'upsert','entity':raider}]);c.call('step',ticks=1)
        assert c.call('inspect_targeting')['npcs']['guard']['memory'] is None
        c.call('emit_noise',origin={'kind':'entity','id':'player'},radius=10);c.call('step',ticks=1)
        heard=c.call('inspect_targeting')['npcs']['guard'];samples.append(heard)
        assert heard['memory']['source']=='sound' and heard['memory']['actor_id'] is None and heard['selection']['observed_actor'] is None
        c.call('save_replay',path='artifacts/targeting-sound.replay.json');assert c.call('run_replay',path='artifacts/targeting-sound.replay.json')['matched']
        checks.append('Hidden hostiles do not acquire; audible cue remains anonymous and replays deterministically')

        # The demo is authored and persisted through MCP, like every test mutation.
        guard['npc']['awareness']['fov_degrees']=120
        guard['npc']['speed']=1.5
        player['footsteps']={'surface':'floor','stride':0.6,'radius':8,'min_speed':0.1}
        player['transform']['position']=[3,0.85,-3];raider['transform']['position']=[-3,0.85,-3]
        raider['npc'].update(enabled=True,patrol={'route':{'kind':'points','points':[[-3,-3],[-5,-3],[-5,1],[-3,1]]},'traversal':'loop','travel_ticks':1200,'arrival_distance':0.4,'wait_ticks':60,'retry_ticks':120})
        friend['transform']['position']=[4,0.85,2]
        c.call('edit_scene',edits=[{'op':'upsert','entity':player},{'op':'upsert','entity':raider},{'op':'upsert','entity':friend},{'op':'upsert','entity':guard}])
        c.call('save_scene',path='examples/npc-relations.scene.json');c.call('load_scene',path='examples/npc-relations.scene.json')
        c.call('select_entity',id='guard');c.call('step',ticks=180)
        demo=c.call('inspect');assert demo['runtime_positions']['raider']!=raider['transform']['position']
        assert demo['npcs']['friend']['memory'] is None
        c.call('capture',path='artifacts/targeting-demo.png',width=1280,height=900)
        c.call('save_replay',path='artifacts/targeting-demo.replay.json');assert c.call('run_replay',path='artifacts/targeting-demo.replay.json')['matched']
        checks.append('MCP-authored playable demo, independent hostile patrol, ally exclusion, scene persistence and replay')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':demo['build_id'],'slow_requests':c.slow_requests}
        (ROOT/'artifacts/targeting-verification.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(checks),'build_id':demo['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');args=p.parse_args();verify(args.binary.resolve(),args.window)
