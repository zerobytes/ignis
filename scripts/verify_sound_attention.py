#!/usr/bin/env python3
"""Verify acoustic interruption rules through the real MCP dispatcher and window."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_targeting import actor


def verify(binary,window):
    (ROOT/'artifacts').mkdir(exist_ok=True);c=Client(binary,window);checks=[];samples=[]
    def emit(x):return c.call('emit_noise',origin={'kind':'position','position':[x,0.85,0]},radius=10)['queued']
    def attention():return c.call('inspect_hearing')['npcs']['guard']
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-sound-attention-verifier','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':20,'depth':20,'seed':42,'obstacles':0})
        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');player['transform']['position']=[0,0.85,5]
        guard=actor('guard','guards',[0,0.85,0],[0.7,0.2,0.8]);guard['npc'].update(enabled=True,speed=0.1,
            awareness={'mode':'perceptive','vision_range':12,'fov_degrees':90,'initial_facing_degrees':0,'memory_ticks':300,'search_ticks':90,'search_turn_degrees_per_second':90,'hearing':{'range':10}},
            sound_attention={'commitment_ticks':60,'switch_margin':0.15,'repeat_radius':1,'visual_priority_ticks':60})
        c.call('edit_scene',edits=[{'op':'upsert','entity':player},{'op':'upsert','entity':guard}]);c.call('set_debug_view',npc_vision=False,npc_hearing=True,noise_events=True,npc_routes=True);c.call('select_entity',id='guard')
        first=emit(6);c.call('step',ticks=1);initial=attention();assert initial['attention']['reason']=='acquired'
        emit(-1);c.call('step',ticks=1);held=attention();samples.append(held);assert held['attention']['commitment_rejections']==1 and held['memory']==initial['memory']
        c.call('capture',path='artifacts/sound-attention-held.png',width=1280,height=900)
        emit(6.2);c.call('step',ticks=1);assert attention()['attention']['repeated_rejections']==1
        c.call('step',ticks=58);emit(-6);c.call('step',ticks=1);weak=attention();assert weak['attention']['margin_rejections']==1 and weak['memory']==initial['memory']
        stronger=emit(-1);c.call('step',ticks=1);switched=attention();samples.append(switched);assert switched['attention']['reason']=='replaced_sound' and switched['memory']['noise_id']==stronger['id'] and switched['memory']['actor_id'] is None
        c.call('capture',path='artifacts/sound-attention-switched.png',width=1280,height=900)
        c.call('save_replay',path='artifacts/sound-attention.replay.json');assert c.call('run_replay',path='artifacts/sound-attention.replay.json')['matched']
        live=c.call('inspect');prefix=c.call('inspect_replay',path='artifacts/sound-attention.replay.json',tick=2);assert prefix['snapshot']['npcs']['guard']['sound_attention']['commitment_rejections']==1 and c.call('inspect')==live
        checks.append('Commitment, spatial suppression, weak rejection, strong interruption, anonymous memory and exact replay/prefix')

        bad=copy.deepcopy(guard);bad['npc']['sound_attention']['switch_margin']=1.1;c.call('upsert',entity=bad,expected_error=True);after=c.call('inspect');assert after==live,{k:(live.get(k),after.get(k)) for k in live if live.get(k)!=after.get(k)}
        legacy=copy.deepcopy(guard);legacy['npc'].pop('sound_attention');c.call('upsert',entity=legacy);emit(1);c.call('step',ticks=1);emit(8);c.call('step',ticks=1);assert attention()['attention']['reason']=='legacy_strongest'
        c.call('undo');assert attention()['attention']['active_noise_id'] is None;c.call('redo');assert attention()['policy'] is None
        expiring=copy.deepcopy(guard);expiring['npc']['awareness']['memory_ticks']=2;expiring['npc']['sound_attention']['commitment_ticks']=300
        c.call('upsert',entity=expiring);emit(1);c.call('step',ticks=2);emit(8);c.call('step',ticks=1);assert attention()['attention']['reason']=='acquired'
        c.call('save_scene',path='artifacts/sound-attention.scene.json');c.call('load_scene',path='artifacts/sound-attention.scene.json');assert attention()['attention']['active_noise_id'] is None
        checks.append('Atomic invalid policy, legacy immediate replacement, undo/redo, finite memory overriding commitment and persistence')

        visual=copy.deepcopy(guard);visual['npc']['target']='player';visual['npc']['awareness']['vision_range']=3
        player['transform']['position']=[0,0.85,-2];c.call('edit_scene',edits=[{'op':'upsert','entity':player},{'op':'upsert','entity':visual}])
        emit(1);c.call('step',ticks=1);assert attention()['attention']['reason']=='visual_priority'
        c.call('set_input',input={'x':0,'z':-1});c.call('step',ticks=30);c.call('set_input',input={'x':0,'z':0})
        emit(1);c.call('step',ticks=1);protected=attention();assert protected['attention']['visual_memory_rejections']==1 and protected['memory']['source']=='vision'
        c.call('step',ticks=60);emit(1);c.call('step',ticks=1);assert attention()['attention']['reason']=='replaced_visual_memory'
        c.call('save_replay',path='artifacts/sound-attention-visual.replay.json');assert c.call('run_replay',path='artifacts/sound-attention-visual.replay.json')['matched']
        checks.append('Confirmed vision priority, live player leaving range, protected last-seen memory and later acoustic interruption')

        # Playable sound-only listener: no visual target and no footstep emitter, so N isolates each cue.
        demo=copy.deepcopy(guard);demo['npc']['speed']=1.5;demo['npc']['sound_attention']['commitment_ticks']=90;demo['npc']['sound_attention']['repeat_radius']=1.25;demo['npc']['awareness']['memory_ticks']=600;demo['npc']['awareness']['search_ticks']=180
        demo['transform']['position']=[0,0.85,2];player['transform']['position']=[0,0.85,-4];player.pop('footsteps',None)
        wall={'id':'screen','name':'Wall','body':'fixed','color':[0.2,0.65,0.45],'transform':{'position':[-2,1.5,0],'size':[6,3,0.4]}}
        c.call('edit_scene',edits=[{'op':'upsert','entity':player},{'op':'upsert','entity':demo},{'op':'upsert','entity':wall}]);c.call('save_scene',path='examples/npc-sound-attention.scene.json')
        c.call('emit_noise',origin={'kind':'entity','id':'player'},radius=12);c.call('step',ticks=30);assert attention()['memory']['source']=='sound' and attention()['memory']['actor_id'] is None
        c.call('capture',path='artifacts/sound-attention-demo.png',width=1280,height=900);c.call('save_replay',path='artifacts/sound-attention-demo.replay.json');assert c.call('run_replay',path='artifacts/sound-attention-demo.replay.json')['matched']
        c.call('reset');assert attention()['attention']['active_noise_id'] is None
        checks.append('MCP-authored sound-only listener demo with wall, isolated N cues, route, replay and clean reset')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':c.call('inspect')['build_id'],'slow_requests':c.slow_requests};(ROOT/'artifacts/sound-attention-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
