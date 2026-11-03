#!/usr/bin/env python3
"""Weighted pending births through the shared MCP dispatcher with replay/capture evidence."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client,ROOT


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-template-selection-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/npc-spawn-regions.scene.json')
        original=c.call('inspect_generators')['definitions'][0]
        g=copy.deepcopy(original);g.pop('template',None);g.update(templates=[{'template':'guard','weight':3},{'template':'elite','weight':1}],activation=None,waves=[],target_alive=4,interval_ticks=1,max_births=12,retire_dead_after_ticks=0)
        c.call('set_npc_generators',generators=[g])
        def info():return c.call('inspect_generators')
        def state():return info()['state']['generators'][0]
        def ctl():return info()['controls'][0]
        def receipts():return info()['state']['receipts']
        def members():return [m for m in state()['members'] if m]
        def ref(m):
            iid=f"{m['instance']['instance']:016x}"
            return next(i['reference'] for i in c.call('inspect_instances')['instances'] if i['instance_id']==iid)
        def remove_all():
            for m in members():c.call('despawn_actor',actor=ref(m))
            c.call('step',ticks=1)
        def control(action):c.call('control_npc_generator',generator=g['id'],action=action)
        c.call('step',ticks=1)
        assert state()['births']==4 and state()['template_draws']==4 and not state()['pending_template']
        success=[r for r in receipts() if r['outcome']=='spawned'];assert [r['template'] for r in success]==[0,0,0,1]
        assert info()['template_ids']==['guard','elite','boss']
        for m in members():
            iid=f"{m['instance']['instance']:016x}";actor=next(i for i in c.call('inspect_instances')['instances'] if i['instance_id']==iid)
            assert c.call('inspect_vitality')['actors'][actor['authored_id']]['units']==(100 if m['template']==0 else 40)
        first=info();tick=c.call('inspect')['tick'];c.call('capture',path='artifacts/template-selection-first.png',width=800,height=600)
        c.call('reset');c.call('step',ticks=1);assert info()==first
        c.call('capture',path='artifacts/template-selection-reset.png',width=800,height=600)
        assert (ROOT/'artifacts/template-selection-first.png').read_bytes()==(ROOT/'artifacts/template-selection-reset.png').read_bytes()
        remove_all();assert state()['births']==8 and state()['template_draws']==8
        path='artifacts/template-selection.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        assert c.call('inspect_replay',path=path,tick=tick)['snapshot']['generators']==first
        c.call('capture_replay',replay_path=path,path='artifacts/template-selection-prefix.png',tick=tick,width=800,height=600)
        assert (ROOT/'artifacts/template-selection-prefix.png').read_bytes()==(ROOT/'artifacts/template-selection-first.png').read_bytes()
        checks.append('Mixed weighted births have template-specific health, known seed choices, fresh members, reset and full/prefix replay/pixels')

        fixed={**g,'template':'guard','templates':[]};c.call('set_npc_generators',generators=[fixed]);c.call('step',ticks=1)
        position_rng=state()['rng'];positions=[(r['position'],r['point'],r['outcome']) for r in receipts()];assert state()['template_draws']==0
        c.call('set_npc_generators',generators=[{**g,'templates':[{'template':'guard','weight':1}]}]);c.call('step',ticks=1)
        assert state()['rng']==position_rng and [(r['position'],r['point'],r['outcome']) for r in receipts()]==positions
        badheight={**g,'region':{'min':[-3,9,0],'max':[3,9,2]}}
        c.call('set_npc_generators',generators=[badheight]);ids=[i['instance_id'] for i in c.call('inspect_instances')['instances']];c.call('step',ticks=3)
        assert state()['attempts']==24 and state()['births']==0 and state()['template_draws']==1 and ctl()['pending_template']=='guard'
        assert [i['instance_id'] for i in c.call('inspect_instances')['instances']]==ids
        trng=state()['template_rng'];prng=state()['rng'];control('pause');c.call('step',ticks=10)
        assert state()['template_rng']==trng and state()['rng']==prng
        control('activate');c.call('step',ticks=1);assert state()['template_draws']==1 and state()['template_rng']==trng
        c.call('save_replay',path='artifacts/template-selection-blocked.replay.json');assert c.call('run_replay',path='artifacts/template-selection-blocked.replay.json')['matched']
        checks.append('Position stream matches legacy singleton; failed admission keeps one template draw/binding, preserves identity and freezes during manual pause')

        before=c.call('inspect')
        lists=[[{'template':'guard','weight':0}],[{'template':'guard','weight':1000001}],[{'template':'missing','weight':1}],[{'template':'guard','weight':1}]*2,[{'template':'guard','weight':1}]*17]
        for entries in lists:c.call('set_npc_generators',generators=[{**g,'templates':entries}],expected_error=True);assert c.call('inspect')==before
        c.call('set_npc_generators',generators=[{**g,'template':'guard'}],expected_error=True);assert c.call('inspect')==before
        templates=c.call('inspect_templates')['templates'];c.call('set_npc_templates',templates=[templates[0],templates[2]],expected_error=True);assert c.call('inspect')==before
        c.call('reset');assert state()['template_draws']==0 and not state()['pending_template']
        checks.append('Invalid weights/duplicates/source conflicts and removal of any candidate dependency are atomic; reset clears draws and pending reservation')

        demo={**original,'templates':[{'template':'guard','weight':3},{'template':'elite','weight':1}],'target_alive':4,'max_births':8,'retire_dead_after_ticks':120,
              'waves':[{'count':4},{'count':3,'templates':[{'template':'guard','weight':1},{'template':'elite','weight':3}],'delay_ticks':120},{'count':1,'template':'boss','delay_ticks':120}]}
        demo.pop('template',None)
        c.call('set_npc_generators',generators=[demo]);authored=info()['definitions'];c.call('set_npc_generators',generators=[]);c.call('undo');assert info()['definitions']==authored
        c.call('redo');assert not info()['definitions'];c.call('undo')
        c.call('save_scene',path='examples/npc-weighted-population.scene.json');c.call('load_scene',path='examples/npc-weighted-population.scene.json')
        c.call('step',ticks=10);assert state()['template_draws']==0
        c.call('set_input',input={'x':1,'z':0});c.call('step',ticks=50);c.call('set_input',input={'x':0,'z':0});c.call('step',ticks=1)
        assert state()['births']==4 and [m['template'] for m in members()]==[0,0,0,1]
        c.call('capture',path='artifacts/template-selection-demo.png',width=800,height=600)
        for index in range(3):
            for _ in range(20):
                if state()['wave']['phase']=='waiting_for_clear':break
                c.call('step',ticks=30)
            assert state()['wave']['current']==index and state()['wave']['phase']=='waiting_for_clear'
            current=members();assert len(current)==[4,3,1][index]
            if index==1:assert [m['template'] for m in current]==[1,1,0]
            if index==2:assert current[0]['template']==2
            remove_all()
        assert state()['wave']['phase']=='completed' and state()['births']==8 and state()['template_draws']==7
        c.call('save_replay',path='artifacts/template-selection-demo.replay.json');assert c.call('run_replay',path='artifacts/template-selection-demo.replay.json')['matched']
        checks.append('MCP-created demo uses inherited weights, per-wave override and fixed boss; spatial permission gates draws, eight births and seven weighted draws complete with replay')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/template-selection-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
