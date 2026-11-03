#!/usr/bin/env python3
"""Spatial producer permissions through existing triggers and the shared MCP dispatcher."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c=Client(binary,window)
    checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-generator-activation-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/npc-waves.scene.json')
        zone={'id':'encounter_zone','name':'Activation region','body':None,'visible':False,'transform':{'position':[3,1,-4],'size':[2,3,2]},'color':[0.2,0.8,0.4],'trigger':{'enabled':True,'targets':['player']}}
        marker={'id':'encounter_marker','name':'Region floor marker','body':None,'blocks_vision':False,'transform':{'position':[3,0.025,-4],'size':[2,0.05,2]},'color':[0.2,0.8,0.4]}
        c.call('upsert',entity=zone);c.call('upsert',entity=marker)
        original=c.call('inspect_generators')['definitions'][0]
        g=copy.deepcopy(original);g.update(initially_active=True,interval_ticks=4,retire_dead_after_ticks=2,activation={'volume':zone['id'],'mode':'while_occupied'})
        g['waves'][1]['delay_ticks']=40
        c.call('set_npc_generators',generators=[g])
        def info():return c.call('inspect_generators')
        def state():return info()['state']['generators'][0]
        def ctl():return info()['controls'][0]
        def walk(x,ticks=50):
            c.call('set_input',input={'x':x,'z':0});c.call('step',ticks=ticks);c.call('set_input',input={'x':0,'z':0});c.call('step',ticks=1)
        def control(action):c.call('control_npc_generator',generator=g['id'],action=action)
        def refs():
            instances=c.call('inspect_instances')['instances']
            return [next(i['reference'] for i in instances if i['instance_id']==f"{m['instance']['instance']:016x}") for m in state()['members'] if m]
        def kill_all():
            for ref in refs():c.call('activate_ability',actor='player',slot=1,target=ref);c.call('step',ticks=1)
        c.call('step',ticks=10)
        assert not ctl()['production_enabled'] and ctl()['status']=='active' and state()['births']==0
        control('activate');c.call('step',ticks=1);assert state()['births']==0
        walk(1)
        assert ctl()['production_enabled'] and state()['births']==2
        prefix_tick=c.call('inspect')['tick'];prefix=info()
        c.call('capture',path='artifacts/generator-activation-entered.png',width=800,height=600)
        walk(-1);assert not ctl()['condition_open']
        kill_all();c.call('step',ticks=10)
        assert info()['counts'][0]['present']==0 and state()['wave']['current']==0
        assert state()['wave']['phase']=='waiting_for_clear' and state()['births']==2
        walk(1);assert state()['wave']['current']==1 and state()['wave']['phase']=='delay'
        remaining=ctl()['remaining_delay_ticks'];assert remaining>0
        control('pause');c.call('step',ticks=1)
        since=state()['paused_since'];walk(-1);walk(1)
        assert ctl()['condition_open'] and ctl()['status']=='paused'
        assert ctl()['remaining_delay_ticks']==remaining and state()['paused_since']==since
        control('activate');c.call('step',ticks=remaining);assert state()['births']==2
        c.call('step',ticks=1);assert state()['births']==4
        checks.append('While occupied starts via physical entry, freezes outside, keeps corpse cleanup, and combines manual/region pause without double-counting wave delay')

        control('cancel');c.call('step',ticks=1)
        assert ctl()['terminal']=='cancelled' and info()['counts'][0]['alive']==2
        c.call('capture',path='artifacts/generator-activation-live.png',width=800,height=600)
        path='artifacts/generator-activation.replay.json';c.call('save_replay',path=path)
        assert c.call('run_replay',path=path)['matched']
        assert c.call('inspect_replay',path=path,tick=prefix_tick)['snapshot']['generators']==prefix
        c.call('capture_replay',replay_path=path,path='artifacts/generator-activation-prefix.png',tick=prefix_tick,width=800,height=600)
        c.call('capture_replay',replay_path=path,path='artifacts/generator-activation-replay.png',width=800,height=600)
        for a,b in [('entered','prefix'),('live','replay')]:assert (ROOT/f'artifacts/generator-activation-{a}.png').read_bytes()==(ROOT/f'artifacts/generator-activation-{b}.png').read_bytes()
        walk(-1);walk(1);assert state()['births']==4 and ctl()['terminal']=='cancelled'
        checks.append('Manual cancellation survives region reentry; sampled condition, phase, clock and captures match full and prefix replay')

        once={**g,'activation':{'volume':zone['id'],'mode':'once'}}
        c.call('set_npc_generators',generators=[once]);walk(1);walk(-1)
        assert ctl()['condition_open'] and not state()['activation']['occupied']
        kill_all();c.call('step',ticks=45);assert state()['births']==4 and state()['activation']['changes']==1
        # Move the volume vertically while preserving targets, then test initial overlap latency.
        c.call('upsert',entity={**zone,'transform':{'position':[0,8,-4],'size':[2,3,2]}})
        c.call('step',ticks=4);assert not ctl()['condition_open'] and state()['births']==0
        c.call('upsert',entity={**zone,'transform':{'position':[0,1,-4],'size':[2,3,2]}})
        c.call('step',ticks=1);assert not ctl()['condition_open'] and state()['births']==0
        assert c.call('inspect_triggers')['triggers']['volumes'][zone['id']]['occupants']==['player']
        c.call('step',ticks=1);assert ctl()['condition_open'] and state()['births']==2
        checks.append('Once remains open after leaving; XYZ height excludes occupancy and initial overlap is consumed only after the first physics step')

        before=c.call('inspect')
        c.call('remove',id=zone['id'],expected_error=True)
        c.call('set_npc_generators',generators=[{**g,'activation':{'volume':marker['id'],'mode':'once'}}],expected_error=True)
        c.call('set_npc_generators',generators=[{**g,'activation':{'volume':zone['id'],'mode':'unknown'}}],expected_error=True)
        assert c.call('inspect')==before
        c.call('reset');assert not ctl()['condition_open']
        c.call('set_npc_generators',generators=[]);c.call('undo');assert not ctl()['condition_open']
        c.call('redo');assert not info()['definitions'];c.call('undo')
        c.call('upsert',entity=zone)
        demo={**original,'initially_active':True,'activation':{'volume':zone['id'],'mode':'while_occupied'}}
        c.call('set_npc_generators',generators=[demo])
        c.call('save_scene',path='examples/npc-generator-activation.scene.json')
        c.call('load_scene',path='examples/npc-generator-activation.scene.json')
        c.call('step',ticks=10);assert state()['births']==0
        c.call('capture',path='artifacts/generator-activation-demo-outside.png',width=800,height=600)
        walk(1);assert state()['births']==2
        c.call('capture',path='artifacts/generator-activation-demo-inside.png',width=800,height=600)
        result={'passed':True,'checks':checks+['Atomic dependency/schema errors, reset/undo/load rearm conditions; visible marker and playable demo authored through MCP'],'native_window':window,'build_id':c.call('inspect')['build_id']}
        (ROOT/'artifacts/generator-activation-verification.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(result['checks']),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
