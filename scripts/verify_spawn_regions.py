#!/usr/bin/env python3
"""Bounded regional candidate sampling through the real MCP dispatcher."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-spawn-regions-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/npc-generator-activation.scene.json')
        original=c.call('inspect_generators')['definitions'][0]
        region={'min':[-5,0.8,-1.5],'max':[5,0.8,2.5]}
        marker={'id':'spawn_region_marker','name':'Candidate center region','body':None,'blocks_vision':False,'color':[0.15,0.5,0.6],'transform':{'position':[0,0.025,0.5],'size':[10,0.05,4]}}
        obstacle={'id':'spawn_obstacle','name':'Occupied area','body':'fixed','color':[0.4,0.5,0.5],'transform':{'position':[0,0.2,0.5],'size':[2,0.4,2]}}
        c.call('upsert',entity=marker);c.call('upsert',entity=obstacle)
        g={**original,'points':[],'region':region,'activation':None,'waves':[],'seed':42,'target_alive':3,'interval_ticks':1,'attempts_per_cycle':8,'max_births':12,'retire_dead_after_ticks':0}
        c.call('set_npc_generators',generators=[g])
        def info():return c.call('inspect_generators')
        def state():return info()['state']['generators'][0]
        def receipts():return info()['state']['receipts']
        def refs():
            instances=c.call('inspect_instances')['instances']
            return [next(i['reference'] for i in instances if i['instance_id']==f"{m['instance']['instance']:016x}") for m in state()['members'] if m]
        def kill_all():
            for ref in refs():
                c.call('activate_ability',actor='player',slot=1,target=ref);c.call('step',ticks=1)
        c.call('step',ticks=1)
        assert state()['births']==3
        first=info();prefix_tick=c.call('inspect')['tick']; bounds=first['definitions'][0]['region']
        assert all(r['point'] is None for r in receipts())
        assert all(all(bounds['min'][i]<=r['position'][i]<=bounds['max'][i] for i in range(3)) for r in receipts())
        assert receipts()[0]['position'][1]==bounds['min'][1] and state()['attempts']<=8
        assert c.call('inspect_population')['recorded_inputs']==0
        c.call('capture',path='artifacts/spawn-regions-first.png',width=800,height=600)
        c.call('reset');c.call('step',ticks=1);assert info()==first
        c.call('capture',path='artifacts/spawn-regions-reset.png',width=800,height=600)
        assert (ROOT/'artifacts/spawn-regions-first.png').read_bytes()==(ROOT/'artifacts/spawn-regions-reset.png').read_bytes()
        checks.append('MCP authors an exclusive regional source; three births have bounded center coordinates, repeated seed/state/pixels, no synthetic input log')

        old=refs()
        for ref in old:c.call('despawn_actor',actor=ref)
        c.call('save_replay',path='artifacts/spawn-regions-pending.replay.json')
        assert c.call('run_replay',path='artifacts/spawn-regions-pending.replay.json')['matched']
        c.call('step',ticks=1);assert state()['births']==6 and not set(old)&set(refs())
        c.call('capture',path='artifacts/spawn-regions-live.png',width=800,height=600)
        path='artifacts/spawn-regions.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        assert c.call('inspect_replay',path=path,tick=prefix_tick)['snapshot']['generators']==first
        c.call('capture_replay',replay_path=path,path='artifacts/spawn-regions-prefix.png',tick=prefix_tick,width=800,height=600)
        c.call('capture_replay',replay_path=path,path='artifacts/spawn-regions-replay.png',width=800,height=600)
        for a,b in [('first','prefix'),('live','replay')]:assert (ROOT/f'artifacts/spawn-regions-{a}.png').read_bytes()==(ROOT/f'artifacts/spawn-regions-{b}.png').read_bytes()
        checks.append('External despawn reuses storage with fresh identities and further candidates; pending/final/prefix replay and captures reproduce exact receipts')

        badheight={**g,'region':{'min':[-3,9,0],'max':[3,9,2]}}
        c.call('set_npc_generators',generators=[badheight]);identities=[x['instance_id'] for x in c.call('inspect_instances')['instances']]
        c.call('step',ticks=3)
        assert state()['attempts']==24 and state()['births']==0
        assert all(r['outcome']=='unsupported_position' and r['position'][1]==9 for r in receipts())
        assert [x['instance_id'] for x in c.call('inspect_instances')['instances']]==identities
        tiny={**g,'region':{'min':[-0.02,0.8,0.5],'max':[0.02,0.8,0.5]}}
        c.call('set_npc_generators',generators=[tiny]);c.call('step',ticks=2)
        assert state()['attempts']==16 and state()['births']==0 and all(r['outcome']=='occupied' for r in receipts())
        before=c.call('inspect')
        badregions=[{'min':[0,0,0],'max':[0,0,0]},{'min':[2,0,0],'max':[1,1,1]},{'min':[0,0,0],'max':[10001,1,1]}]
        for r in badregions:c.call('set_npc_generators',generators=[{**g,'region':r}],expected_error=True);assert c.call('inspect')==before
        c.call('set_npc_generators',generators=[{**g,'points':[[0,0.8,0]]}],expected_error=True)
        c.call('set_npc_generators',generators=[{**g,'region':None}],expected_error=True)
        assert c.call('inspect')==before
        checks.append('Unsupported height/occupied region exhaust only bounded attempts and PRNG; invalid bounds/sources are atomic and no identity or birth quota is consumed')

        demo={**original,'points':[],'region':region,'seed':42,'target_alive':3,'interval_ticks':30,'attempts_per_cycle':8,'max_births':6,'retire_dead_after_ticks':120,
              'waves':[{'count':3},{'count':2,'template':'elite','delay_ticks':120},{'count':1,'template':'boss','delay_ticks':120}]}
        c.call('set_npc_generators',generators=[demo])
        authored=info()['definitions'];c.call('set_npc_generators',generators=[]);c.call('undo');assert info()['definitions']==authored and state()['births']==0
        c.call('redo');assert not info()['definitions'];c.call('undo')
        c.call('save_scene',path='examples/npc-spawn-regions.scene.json');c.call('load_scene',path='examples/npc-spawn-regions.scene.json')
        assert info()['definitions']==authored
        c.call('step',ticks=10);assert state()['rng']==42 and state()['births']==0
        c.call('set_input',input={'x':1,'z':0});c.call('step',ticks=50);c.call('set_input',input={'x':0,'z':0});c.call('step',ticks=1)
        assert state()['births']==3
        c.call('capture',path='artifacts/spawn-regions-demo.png',width=800,height=600)
        for wave in range(3):
            for _ in range(20):
                if state()['wave']['phase']=='waiting_for_clear':break
                c.call('step',ticks=30)
            if not (state()['wave']['current']==wave and state()['wave']['phase']=='waiting_for_clear'):
                (ROOT/'artifacts/spawn-regions-failure.json').write_text(json.dumps({'wave_expected':wave,'population':info(),'abilities':c.call('inspect_abilities')},indent=2))
                raise AssertionError({'expected_wave':wave,'state':state(),'counts':info()['counts']})
            kill_all()
            if wave==2:kill_all() # boss requires a second100damage hit
            c.call('step',ticks=1)
        assert state()['births']==6 and state()['wave']['phase']=='completed'
        c.call('save_replay',path='artifacts/spawn-regions-demo.replay.json');assert c.call('run_replay',path='artifacts/spawn-regions-demo.replay.json')['matched']
        checks.append('Undo/reset/load preserve authored region; spatial activation gates seed; MCP-created demo completes three template waves and six regional births with bounded refusals')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']}
        (ROOT/'artifacts/spawn-regions-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
