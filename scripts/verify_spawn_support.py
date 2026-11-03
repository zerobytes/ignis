#!/usr/bin/env python3
"""Shared read-only support query and opt-in population placement, headless by default."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def box(id, position, size, color=(0.25, 0.5, 0.5), body='fixed'):
    return {'id':id,'name':id,'body':body,'color':list(color),
            'transform':{'position':position,'size':size}}


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-spawn-support-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/npc-weighted-population.scene.json')
        original=c.call('inspect');templates=copy.deepcopy(original['scene']['npc_templates'])
        templates[1]['name']='Tall NPC';templates[1]['transform']['size']=[0.8,3.2,0.8]
        templates[2]['name']='Small NPC';templates[2]['transform']['size']=[0.5,0.8,0.5]
        g=copy.deepcopy(original['scene']['npc_generators'][0]);g.update(activation=None,waves=[],points=[],region={'min':[-6,8,-1.4],'max':[6,8,1.4]},support_projection={'max_drop':10},target_alive=4,interval_ticks=1,max_births=12,retire_dead_after_ticks=0)
        c.call('edit_scene',edits=[{'op':'set_npc_generators','generators':[g]}, {'op':'set_npc_templates','templates':templates},
            {'op':'remove','id':'spawn_obstacle'}, {'op':'remove','id':'spawn_region_marker'}])
        def query(p, template='guard', drop=10):
            return c.call('query_spawn_support',template=template,position=p,max_drop=drop)
        def info():return c.call('inspect_generators')
        def state():return info()['state']['generators'][0]
        def receipts():return info()['state']['receipts']
        def refs():
            instances=c.call('inspect_instances')['instances']
            return [next(i['reference'] for i in instances if i['instance_id']==f"{m['instance']['instance']:016x}") for m in state()['members'] if m]
        def remove_all():
            for actor in refs():c.call('despawn_actor',actor=actor)
            c.call('step',ticks=1)
        before=c.call('inspect')
        for _ in range(3):
            q=query([2,8,0]);assert q['geometry_only'] and q['surface_id']=='floor' and q['report']['status']=='supported'
            assert q['report']['feet']==[2,0,0] and abs(q['report']['position'][1]-0.8)<0.00001
        assert c.call('inspect')==before
        c.call('query_spawn_support',template='missing',position=[0,8,0],max_drop=10,expected_error=True)
        for drop in [0,-1,20001]:c.call('query_spawn_support',template='guard',position=[0,8,0],max_drop=drop,expected_error=True)
        c.call('query_spawn_support',template='guard',position=[10001,8,0],max_drop=1,expected_error=True)
        assert c.call('inspect')==before
        checks.append('Read-only MCP query resolves feet/body and surface ID, preserves complete snapshot and rejects invalid requests atomically')

        c.call('upsert',entity=box('first_narrow',[0,1,0],[0.2,2,2]))
        q=query([0,8,0]);assert q['surface_id']=='first_narrow' and q['report']['status']=='insufficient_support'
        assert query([0,8,0],drop=5.9)['report']['status']=='no_surface'
        c.call('remove',id='first_narrow')
        c.call('edit_scene',edits=[{'op':'upsert','entity':box('left',[-1,1,0],[2,2,2])}, {'op':'upsert','entity':box('right',[1,1,0],[2,2,2])}])
        assert query([0,8,0])['report']['status']=='supported'
        c.call('upsert',entity=box('right',[1.005,1,0],[2,2,2]))
        assert query([0,8,0])['report']['status']=='insufficient_support'
        c.call('edit_scene',edits=[{'op':'remove','id':'left'},{'op':'remove','id':'right'}, {'op':'upsert','entity':box('ceiling',[0,1.1,0],[4,0.2,4])}])
        assert query([0,0.9,0])['report']['status']=='occupied'
        assert query([0,0.9,0],template='boss')['report']['status']=='supported'
        c.call('remove',id='ceiling')
        checks.append('First surface never falls through to a lower floor; bounded drop, full modular support, thin gaps and template-dependent ceiling clearance are observed through MCP')

        c.call('step',ticks=1);assert state()['births']==4
        for r in receipts():
            if r['outcome']=='spawned':
                assert r['position'][1]==8 and r['support']['status']=='supported'
                assert abs(r['support']['position'][1]-([0.8,1.6,0.4][r['template']]))<0.00001
                assert info()['support_surface_ids'][str(r['support']['surface'])]=='floor'
        first=info();tick=c.call('inspect')['tick'];c.call('capture',path='artifacts/spawn-support-first.png',width=800,height=600)
        c.call('reset');c.call('step',ticks=1);assert info()==first
        c.call('capture',path='artifacts/spawn-support-reset.png',width=800,height=600)
        assert (ROOT/'artifacts/spawn-support-first.png').read_bytes()==(ROOT/'artifacts/spawn-support-reset.png').read_bytes()
        old=refs()
        for actor in old:c.call('despawn_actor',actor=actor)
        c.call('save_replay',path='artifacts/spawn-support-pending.replay.json');assert c.call('run_replay',path='artifacts/spawn-support-pending.replay.json')['matched']
        c.call('step',ticks=1);assert state()['births']==8 and not set(old)&set(refs())
        path='artifacts/spawn-support.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        assert c.call('inspect_replay',path=path,tick=tick)['snapshot']['generators']==first
        c.call('capture_replay',replay_path=path,path='artifacts/spawn-support-prefix.png',tick=tick,width=800,height=600)
        assert (ROOT/'artifacts/spawn-support-first.png').read_bytes()==(ROOT/'artifacts/spawn-support-prefix.png').read_bytes()
        checks.append('Weighted births use their own body heights; candidate/resolved receipts, fresh identities, pending/final/prefix replay and reset/prefix PNGs agree')

        blocked={**g,'region':{'min':[40,8,40],'max':[41,8,41]}}
        c.call('set_npc_generators',generators=[blocked]);ids=c.call('inspect_instances')['instances'];c.call('step',ticks=3)
        assert state()['births']==0 and state()['attempts']==24 and state()['template_draws']==1
        assert c.call('inspect_instances')['instances']==ids and all(r['support']['status']=='no_surface' for r in receipts())
        frozen=state();c.call('control_npc_generator',generator=g['id'],action='pause');c.call('step',ticks=10)
        assert all(state()[k]==frozen[k] for k in ['rng','template_rng','template_draws','attempts','births'])
        before=c.call('inspect');c.call('set_npc_generators',generators=[{**g,'support_projection':{'max_drop':0}}],expected_error=True);assert c.call('inspect')==before
        c.call('set_npc_generators',generators=[g]);defs=info()['definitions'];c.call('set_npc_generators',generators=[]);c.call('undo');assert info()['definitions']==defs;c.call('redo');assert not info()['definitions'];c.call('undo')
        checks.append('No-floor failures consume bounded attempts only, keep weighted pending and identities, pause freezes streams, invalid edits and undo/redo preserve authoring contract')

        # Demo: three separated platforms, a narrow rejected post, no lower floor in the birth region.
        floor=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='floor')
        floor['transform']={'position':[0,-0.25,5.5],'size':[20,0.5,5]}
        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');player['transform']['position']=[0,0.8,6]
        demo={**g,'target_alive':6,'max_births':18,'interval_ticks':60,'retire_dead_after_ticks':120,'region':{'min':[-6.2,8,-1.3],'max':[6.2,8,1.3]},'templates':[{'template':'guard','weight':2},{'template':'elite','weight':1},{'template':'boss','weight':1}]}
        edits=[{'op':'remove','id':'encounter_marker'},{'op':'remove','id':'encounter_zone'},{'op':'upsert','entity':floor},{'op':'upsert','entity':player},
               {'op':'set_npc_generators','generators':[demo]}, {'op':'set_camera','camera':{'position':[14,14,19],'target':[0,1,1],'fov_degrees':48}}]
        for id,p,size,color in [('low_platform',[-5,-0.25,0],[3,0.5,3],[0.2,0.45,0.5]), ('mid_platform',[-1.5,0.75,0],[3,1.5,3],[0.25,0.55,0.45]), ('high_platform',[2,1.5,0],[3,3,3],[0.3,0.45,0.65]), ('narrow_post',[5,1,0],[0.2,2,3],[0.8,0.3,0.2])]:
            edits.append({'op':'upsert','entity':box(id,p,size,color)})
        c.call('edit_scene',edits=edits)
        c.call('save_scene',path='examples/npc-spawn-support.scene.json');c.call('load_scene',path='examples/npc-spawn-support.scene.json')
        assert c.call('inspect')['scene']['version']==SCENE_VERSION
        c.call('step',ticks=241);assert state()['births']==6
        success=[r for r in receipts() if r['outcome']=='spawned'];assert len({r['support']['feet'][1] for r in success})>=2
        assert {r['template'] for r in success}=={0,1,2}
        assert any(r['support']['status']=='no_surface' for r in receipts())
        c.call('capture',path='artifacts/spawn-support-demo.png',width=1000,height=700)
        c.call('save_replay',path='artifacts/spawn-support-demo.replay.json');assert c.call('run_replay',path='artifacts/spawn-support-demo.replay.json')['matched']
        result={'passed':True,'checks':checks+['MCP-authored demo places three body sizes on separated heights, rejects gaps and preserves replay; no locomotion/controller capability implied'],'native_window':window,'build_id':c.call('inspect')['build_id']}
        (ROOT/'artifacts/spawn-support-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(result['checks']),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
