#!/usr/bin/env python3
"""NPC routines without a player and explicit NPC-to-NPC perception via real MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary,window):
    (ROOT/'artifacts').mkdir(exist_ok=True);c=Client(binary,window);checks=[];samples=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-npc-autonomy-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/patrol.scene.json')
        state=c.call('inspect');assert state['scene']['version']==SCENE_VERSION
        guard=copy.deepcopy(next(e for e in state['scene']['entities'] if e['id']=='hunter'))
        guard['npc']['target']=None;guard['npc']['awareness']['memory_ticks']=60;guard['npc']['awareness']['search_ticks']=30
        c.call('edit_scene',edits=[{'op':'remove','id':'player'},{'op':'upsert','entity':guard}])
        c.call('set_debug_view',npc_vision=True,npc_hearing=True,noise_events=True,npc_routes=True)
        c.call('step',ticks=30);before=c.call('inspect');index=before['patrols']['hunter']['point_index']
        assert all(e['body']!='player' for e in before['scene']['entities']) and before['npcs']['hunter']['memory'] is None
        assert before['patrols']['hunter']['phase']=='moving'
        c.call('emit_noise',origin={'kind':'position','position':[0,0.8,4]},radius=12);c.call('step',ticks=1)
        heard=c.call('inspect');assert heard['npcs']['hunter']['memory']['source']=='sound' and heard['patrols']['hunter']['phase']=='suspended'
        c.call('capture',path='artifacts/npc-autonomy-sound.png',width=1280,height=900)
        c.call('step',ticks=65);resumed=c.call('inspect');assert resumed['npcs']['hunter']['memory'] is None
        assert resumed['patrols']['hunter']['phase']=='moving' and resumed['patrols']['hunter']['point_index']==index
        c.call('save_scene',path='artifacts/npc-targetless.scene.json');c.call('save_replay',path='artifacts/npc-targetless.replay.json')
        assert c.call('run_replay',path='artifacts/npc-targetless.replay.json')['matched']
        c.call('load_scene',path='artifacts/npc-targetless.scene.json');c.call('step',ticks=1);assert c.call('inspect')['npcs']['hunter']['memory'] is None
        checks.append('No player/target: autonomous patrol, anonymous sound interruption, same-point resume and persistence/replay')

        c.call('load_scene',path='examples/patrol.scene.json');state=c.call('inspect')
        hunter=copy.deepcopy(next(e for e in state['scene']['entities'] if e['id']=='hunter'))
        civilian=copy.deepcopy(guard);civilian['id']='civilian';civilian['name']='Civilian';civilian['color']=[0.15,0.7,0.9];civilian['transform']['position']=[-4,0.85,2]
        civilian['npc']['speed']=1.2;civilian['npc']['patrol']['route']={'kind':'points','points':[[-4,2],[-4,-2],[4,-2],[4,2]]}
        hunter['npc']['target']='civilian';hunter['transform']['position']=[1,0.85,2];hunter['npc']['speed']=1.5
        hunter['npc']['awareness']['vision_range']=12;hunter['npc']['awareness']['fov_degrees']=120;hunter['npc']['awareness']['initial_facing_degrees']=-90
        c.call('edit_scene',edits=[{'op':'upsert','entity':hunter},{'op':'upsert','entity':civilian}])
        c.call('save_scene',path='examples/npc-autonomy.scene.json')
        c.call('select_entity',id='hunter');c.call('capture',path='artifacts/npc-autonomy-before.png',width=1280,height=900)
        prior=c.call('inspect')['runtime_positions'];c.call('step',ticks=1);seen=c.call('inspect')
        assert seen['npcs']['hunter']['observed_this_tick'] and seen['npcs']['hunter']['memory']['position']==prior['civilian']
        assert seen['npcs']['civilian']['memory'] is None
        c.call('step',ticks=240);moving=c.call('inspect');assert moving['runtime_positions']['civilian']!=prior['civilian']
        assert moving['runtime_positions']['hunter']!=prior['hunter']
        c.call('capture',path='artifacts/npc-autonomy-moving.png',width=1280,height=900)
        samples.append({'tick':moving['tick'],'hunter':moving['npcs']['hunter'],'civilian_patrol':moving['patrols']['civilian']})
        c.call('save_replay',path='artifacts/npc-autonomy.replay.json');assert c.call('run_replay',path='artifacts/npc-autonomy.replay.json')['matched']
        live=c.call('inspect');prefix=c.call('inspect_replay',path='artifacts/npc-autonomy.replay.json',tick=1)
        assert prefix['snapshot']['npcs']['hunter']['memory']['position']==prior['civilian'] and c.call('inspect')==live
        checks.append('MCP-authored NPC target, pre-step observation, autonomous civilian movement, pursuit and isolated replay prefix')

        for target in ['hunter','floor','missing']:
            bad=copy.deepcopy(hunter);bad['npc']['target']=target;c.call('upsert',entity=bad,expected_error=True);assert c.call('inspect')==live
        c.call('remove',id='civilian',expected_error=True);assert c.call('inspect')==live
        no_target=copy.deepcopy(hunter);no_target['npc']['target']=None
        c.call('edit_scene',edits=[{'op':'remove','id':'civilian'},{'op':'upsert','entity':no_target}]);c.call('step',ticks=1)
        assert c.call('inspect')['npcs']['hunter']['memory'] is None
        c.call('undo');c.call('step',ticks=1);assert c.call('inspect')['npcs']['hunter']['observed_this_tick']
        c.call('redo');c.call('step',ticks=1);assert c.call('inspect')['npcs']['hunter']['memory'] is None
        c.call('load_scene',path='examples/npc-autonomy.scene.json');assert c.call('inspect')['scene']['version']==SCENE_VERSION
        checks.append('Invalid/self/non-agent/removal references atomic; detach/remove batch, undo/redo and load restore contracts')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'slow_requests':c.slow_requests,'build_id':c.call('inspect')['build_id']}
        (ROOT/'artifacts/npc-autonomy-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'samples':samples}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');args=p.parse_args();verify(args.binary.resolve(),args.window)
