#!/usr/bin/env python3
"""Controller-compatible graph walking, entirely through MCP and offscreen capture."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT
from verify_geometry import box


def npc(id,position,goal,limit=30,color=(0.7,0.2,0.8)):
    e=box(id,position,[0.6,1.6,0.6],body='npc',color=color)
    e['character_controller']={'max_slope_degrees':limit,'slide_slope_degrees':max(50,limit),'snap_distance':0.3,'step':{'max_height':0.3,'min_width':0.4}}
    e['npc']={'surface':'floor','speed':2,'stop_distance':0.2,'enabled':True,'mobility':{'walk':True},
        'objectives':[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':goal,'arrival_distance':0.2}}]}
    return e


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-ground-navigation-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        def fixture(angle=20,limit=30,step=None,ceiling=False):
            c.call('generate',recipe={'width':30,'depth':20,'seed':42,'obstacles':0})
            h=4*math.tan(math.radians(angle)) if step is None else step
            n=npc('guard',[-1.5,0.81,0],[5.5,h+0.8,0],limit)
            p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[-8,0.8,7];p.pop('footsteps',None)
            net={'nodes':[[-1.5,0,0],[5.5,h,0]],'links':[{'from':0,'to':1,'mode':'walk'},{'from':1,'to':0,'mode':'walk'}]}
            terrain=[box('ramp',[2,h/2,0],[4,h,4],geometry={'kind':'wedge','rise':'positive_x'}),box('landing',[7,h/2,0],[6,h,4])] if step is None else [box('step',[4,h/2,0],[8,h,4])]
            if ceiling:terrain.append(box('ceiling',[2,1.95,0],[6,0.4,4]))
            c.call('edit_scene',edits=[{'op':'set_spatial_network','network':net}]+[{'op':'upsert','entity':e} for e in [p,n]+terrain])
            return n,net,h
        n,net,h=fixture()
        before=c.call('inspect')
        q=c.call('query_ground_traversal',entity='guard',start=[-1.5,0.01,0],end=[5.5,h+0.01,0])
        assert q['static_only'] and q['report']['status']=='arrived' and q['report']['ticks']<=600
        assert c.call('inspect')==before
        c.call('query_ground_traversal',entity='guard',start=[0,0,0],end=[50,0,0],expected_error=True)
        bad=copy.deepcopy(n);bad['npc']['mobility']['fly_speed']=0
        c.call('upsert',entity=bad,expected_error=True);assert c.call('inspect')==before
        admitted=c.call('inspect_mobility')['ground_admissions']['guard'];assert all(r['status']=='arrived' for r in admitted)
        c.call('step',ticks=300);assert c.call('inspect_mobility')['states']['guard']['phase']=='arrived'
        checks.append('Read-only explicit feet probe, bounded errors and invalid mobility are atomic; compiled walk links execute uphill and retain legacy controls')
        for angle,limit,step,ceiling,passes in [(40,30,None,False,False),(40,45,None,False,True),(20,30,0.2,False,True),(20,30,0.5,False,False),(20,30,0.2,True,False)]:
            fixture(angle,limit,step,ceiling);admission=c.call('inspect_mobility')['ground_admissions']['guard'][0]
            assert (admission['status']=='arrived')==passes,(angle,limit,step,ceiling,admission)
            c.call('step',ticks=480);assert (c.call('inspect_mobility')['states']['guard']['phase']=='arrived')==passes
        checks.append('Different climb limits choose compatible40deg traversal;20cm step succeeds,50cm step and low ceiling reject without false arrival')

        n,net,h=fixture()
        template=copy.deepcopy(n);template['id']='template';template['npc']['enabled']=False
        c.call('set_npc_templates',templates=[template]);original=c.call('inspect_mobility')['ground_admissions']['guard']
        c.call('spawn_actor',template='template',position=[-6,0.8,-6],target=None)
        c.call('save_replay',path='artifacts/ground-pending.replay.json');assert c.call('run_replay',path='artifacts/ground-pending.replay.json')['matched']
        c.call('step',ticks=1);states=c.call('inspect_character_controllers')['controllers'];born=next(id for id in states if id!='guard');identity=states[born]['instance']
        assert c.call('inspect_mobility')['ground_admissions'][born]==original
        c.call('despawn_actor',actor=born);c.call('step',ticks=1);assert born not in c.call('inspect_mobility')['ground_admissions']
        c.call('spawn_actor',template='template',position=[-6,0.8,-6],target=None);c.call('step',ticks=1)
        states=c.call('inspect_character_controllers')['controllers'];new=next(id for id in states if id!='guard');assert states[new]['instance']!=identity
        checks.append('Templates share compiled geometry admissions; pending births replay, retirement removes reports and reused slots receive a fresh motor identity')

        # Saved comparison: purple takes the gentle detour; blue can use the short steep ramp.
        c.call('generate',recipe={'width':30,'depth':20,'seed':42,'obstacles':0})
        p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[-9,0.8,-7];p.pop('footsteps',None)
        purple=npc('purple',[-4,0.81,0],[4,2.8,0],30)
        blue=npc('blue',[-5,0.81,0],[4,2.8,-1],45,color=(0.15,0.5,0.95))
        gray=npc('gray',[-6,0.81,-5],[-3,1.3,-5],30,color=(0.5,0.5,0.5))
        terrain=[box('steep',[0.2,1,0],[2.4,2,3],geometry={'kind':'wedge','rise':'positive_x'},color=(0.8,0.25,0.25)),
                 box('gentle',[-0.25,1,5],[5.5,2,3],geometry={'kind':'wedge','rise':'positive_x'},color=(0.2,0.7,0.4)),
                 box('red_landing',[3.7,1,0],[4.6,2,3],color=(0.8,0.25,0.25)),
                 box('green_landing',[4.25,1,5],[3.5,2,3],color=(0.2,0.7,0.4)),
                 box('top_bridge',[4.5,1,2.5],[3,2,2],color=(0.25,0.55,0.65)),
                 box('tall_step',[-2,0.25,-5],[4,0.5,3],color=(0.65,0.45,0.2))]
        nodes=[[-4,0,0],[4,2,0],[-4,0,5],[4,2,5],[-5,0,0],[4,2,-1],[-6,0,-5],[-3,0.5,-5]]
        links=[{'from':a,'to':b,'mode':'walk'} for a,b in [(0,1),(0,2),(2,3),(3,1),(4,0),(1,5),(6,7)]]
        c.call('edit_scene',edits=[{'op':'set_spatial_network','network':{'nodes':nodes,'links':links}}]+[{'op':'upsert','entity':e} for e in [p,purple,blue,gray]+terrain]+[{'op':'set_camera','camera':{'position':[17,18,23],'target':[0,1,1],'fov_degrees':48}}])
        c.call('set_debug_view',npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call('save_scene',path='examples/npc-ground-navigation.scene.json');c.call('load_scene',path='examples/npc-ground-navigation.scene.json')
        initial=c.call('inspect');assert initial['scene']['version']==SCENE_VERSION
        admissions=c.call('inspect_mobility')['ground_admissions'];assert admissions['purple'][0]['status']!='arrived' and admissions['blue'][0]['status']=='arrived' and admissions['gray'][6]['status']!='arrived'
        c.call('step',ticks=1);states=c.call('inspect_mobility')['states'];assert states['purple']['route']==[1,2,3] and states['blue']['route']==[4,0,5]
        c.call('step',ticks=89);prefix=c.call('inspect');c.call('capture',path='artifacts/ground-navigation-prefix.png',width=1000,height=700)
        c.call('step',ticks=810);final=c.call('inspect');assert final['mobility']['purple']['phase']=='arrived' and final['mobility']['blue']['phase']=='arrived' and final['mobility']['gray']['completed']==0,final['mobility']
        replay='artifacts/ground-navigation.replay.json';c.call('save_replay',path=replay);assert c.call('run_replay',path=replay)['matched']
        recovered=c.call('inspect_replay',path=replay,tick=90)['snapshot']
        for key in ['runtime_positions','mobility','character_controllers']:assert recovered[key]==prefix[key]
        c.call('capture_replay',replay_path=replay,path='artifacts/ground-navigation-prefix-replay.png',tick=90,width=1000,height=700)
        assert (ROOT/'artifacts/ground-navigation-prefix.png').read_bytes()==(ROOT/'artifacts/ground-navigation-prefix-replay.png').read_bytes()
        c.call('capture',path='artifacts/ground-navigation-final.png',width=1000,height=700)
        c.call('capture_replay',replay_path=replay,path='artifacts/ground-navigation-final-replay.png',width=1000,height=700)
        assert (ROOT/'artifacts/ground-navigation-final.png').read_bytes()==(ROOT/'artifacts/ground-navigation-final-replay.png').read_bytes()
        data=json.loads((ROOT/replay).read_text());data['expected_mobility']['purple']['completed']+=1;(ROOT/'artifacts/ground-navigation-tampered.replay.json').write_text(json.dumps(data));assert not c.call('run_replay',path='artifacts/ground-navigation-tampered.replay.json')['matched']
        assert c.call('inspect')==final
        c.call('reset');assert c.call('inspect')['character_controllers']==initial['character_controllers']
        changed=copy.deepcopy(purple);changed['character_controller']['max_slope_degrees']=45;c.call('upsert',entity=changed);assert c.call('inspect_mobility')['ground_admissions']['purple'][0]['status']=='arrived'
        c.call('undo');assert c.call('inspect_mobility')['ground_admissions']['purple'][0]['status']!='arrived'
        c.call('redo');assert c.call('inspect_mobility')['ground_admissions']['purple'][0]['status']=='arrived'
        c.call('load_scene',path='examples/npc-ground-navigation.scene.json');assert c.call('inspect')['character_controllers']==initial['character_controllers']
        c.call('capture',path='artifacts/ground-navigation-demo.png',width=1000,height=700)
        checks.append('MCP-authored demo chooses gentle detour for30deg and short steep route for45deg; tall-step refusal, cache invalidation, undo/redo/reset/load, isolated replay prefix/tamper and identical GPU pixels')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']}
        (ROOT/'artifacts/ground-navigation-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
