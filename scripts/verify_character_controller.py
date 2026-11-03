#!/usr/bin/env python3
"""Terrestrial controller through the actual MCP executable, without a native window."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT
from verify_geometry import box, npc


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-controller-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        def base():
            c.call('generate', recipe={'width':30,'depth':20,'seed':42,'obstacles':0})
            p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player')
            p['transform']={'position':[-1.5,0.81,0],'size':[0.6,1.6,0.6]}
            p.pop('footsteps',None)
            p['character_controller']={'step':{'max_height':0.3,'min_width':0.4}}
            c.call('upsert',entity=p)
            return p
        def move(ticks,x=1):
            c.call('set_input',input={'x':x,'z':0})
            c.call('step',ticks=ticks)
            return c.call('inspect')
        def ramps(angle):
            h=4*math.tan(math.radians(angle))
            c.call('edit_scene',edits=[{'op':'upsert','entity':box('ramp',[2,h/2,0],[4,h,4],geometry={'kind':'wedge','rise':'positive_x'})},
                {'op':'upsert','entity':box('landing',[7,h/2,0],[6,h,4])}])
        p=base()
        before=c.call('inspect')
        assert c.call('inspect_character_controllers')['controllers']==before['character_controllers']
        assert before==c.call('inspect')
        for key,val in [('offset',0),('gravity',-1),('max_slope_degrees',90),('snap_distance',2),('step',{'max_height':2,'min_width':0.4})]:
            invalid=copy.deepcopy(p);invalid['character_controller'][key]=val
            c.call('upsert',entity=invalid,expected_error=True)
            assert before==c.call('inspect')
        legacy=copy.deepcopy(p);legacy.pop('character_controller')
        c.call('capture',path='artifacts/controller-authored.png',width=800,height=600)
        c.call('upsert',entity=legacy);assert not c.call('inspect_character_controllers')['controllers']
        c.call('capture',path='artifacts/controller-legacy.png',width=800,height=600)
        assert (ROOT/'artifacts/controller-authored.png').read_bytes()==(ROOT/'artifacts/controller-legacy.png').read_bytes()
        c.call('undo');assert c.call('inspect_character_controllers')['controllers']
        c.call('redo');assert not c.call('inspect_character_controllers')['controllers']
        c.call('undo');c.call('step',ticks=1);assert c.call('inspect_character_controllers')['controllers']['player']['grounded']
        checks.append('Read-only inspection, bounded validation without mutation, default legacy, undo/redo and identical authored pixels; initial floor published before first movement')

        for angle,passes in [(20,True),(40,False)]:
            base();ramps(angle);st=move(140)
            x,y,_=st['runtime_positions']['player']
            assert (x>4)==passes,(angle,x,y)
            if passes: assert y>2
            c.call('save_replay',path='artifacts/controller-ramp.replay.json')
            result=c.call('run_replay',path='artifacts/controller-ramp.replay.json')
            assert result['matched'] and result['character_controllers_matched']
        for h,ceiling,passes in [(0.2,False,True),(0.5,False,False),(0.2,True,False)]:
            base();c.call('upsert',entity=box('step',[4,h/2,0],[8,h,4]))
            if ceiling:c.call('upsert',entity=box('ceiling',[2,1.95,0],[6,0.4,4]))
            st=move(140);assert (st['runtime_positions']['player'][0]>2)==passes
        base();c.call('upsert',entity=box('ledge',[-2,0.5,0],[6,1,4]))
        p['transform']['position']=[-1.5,1.81,0];c.call('upsert',entity=p)
        st=move(130);assert st['runtime_positions']['player'][1]<0.9
        checks.append('Real runtime climbs20deg, refuses40deg, steps20cm, refuses50cm and low ceiling, falls from1m without long snap; replay compares motor state')

        base();n=npc('guard',0,3)
        n['npc'].pop('objectives');n['character_controller']={}
        template=copy.deepcopy(n);template['id']='guard_template';template['npc']['enabled']=False
        c.call('edit_scene',edits=[{'op':'upsert','entity':n},{'op':'set_npc_templates','templates':[template]}])
        c.call('step',ticks=90);assert c.call('inspect')['runtime_positions']['guard'][2]<1
        c.call('spawn_actor',template='guard_template',position=[3,0.8,3],target=None);c.call('step',ticks=2)
        controllers=c.call('inspect_character_controllers')['controllers']
        born=next(id for id in controllers if id not in ('player','guard'));old=controllers[born]['instance']
        c.call('despawn_actor',actor=born);c.call('step',ticks=1)
        assert born not in c.call('inspect_character_controllers')['controllers']
        c.call('spawn_actor',template='guard_template',position=[4,0.8,3],target=None);c.call('step',ticks=1)
        new=next(s for id,s in c.call('inspect_character_controllers')['controllers'].items() if id not in ('player','guard'))
        assert new['instance']!=old and new['vertical_velocity']==0
        checks.append('NPC planar pursuit uses same motor, template births get independent state, despawn/reuse clears old controller and identity')

        # Saved manual laboratory, authored exclusively through MCP.
        p=base();p['transform']['position']=[-5,0.81,4]
        guard=npc('guard',-6,7);guard['npc'].pop('objectives');guard['character_controller']={}
        edits=[{'op':'upsert','entity':p},{'op':'upsert','entity':guard},
            {'op':'set_camera','camera':{'position':[16,17,21],'target':[0,1,0],'fov_degrees':48}}]
        for z,angle,id,color in [(4,20,'ramp_allowed',[0.2,0.75,0.4]),(-4,40,'ramp_steep',[0.85,0.25,0.25])]:
            h=4*math.tan(math.radians(angle))
            edits.append({'op':'upsert','entity':box(id,[-1,h/2,z],[4,h,2.5],geometry={'kind':'wedge','rise':'positive_x'},color=color)})
            edits.append({'op':'upsert','entity':box(id+'_landing',[3,h/2,z],[4,h,2.5],color=color)})
        for i in range(4):
            h=(i+1)*0.2;edits.append({'op':'upsert','entity':box('stair_'+str(i),[-2+i, h/2,0],[1,h,2],color=(0.2,0.5,0.8))})
        c.call('edit_scene',edits=edits);c.call('save_scene',path='examples/npc-character-controller.scene.json')
        c.call('load_scene',path='examples/npc-character-controller.scene.json')
        initial=c.call('inspect');assert initial['scene']['version']==SCENE_VERSION
        c.call('capture',path='artifacts/controller-demo.png',width=1000,height=700)
        move(40);prefix=c.call('inspect');move(60);final=c.call('inspect')
        assert final['runtime_positions']['player'][0]>0 and final['runtime_positions']['player'][1]>1.5
        path='artifacts/controller-demo.replay.json';c.call('save_replay',path=path)
        assert c.call('run_replay',path=path)['matched']
        recovered=c.call('inspect_replay',path=path,tick=40)['snapshot']
        assert recovered['runtime_positions']==prefix['runtime_positions'] and recovered['character_controllers']==prefix['character_controllers']
        c.call('capture',path='artifacts/controller-final.png',width=800,height=600)
        c.call('capture_replay',replay_path=path,path='artifacts/controller-replay.png',width=800,height=600)
        assert (ROOT/'artifacts/controller-final.png').read_bytes()==(ROOT/'artifacts/controller-replay.png').read_bytes()
        data=json.loads((ROOT/path).read_text());data['expected_character_controllers']['player']['grounded']^=True
        (ROOT/'artifacts/controller-tampered.replay.json').write_text(json.dumps(data))
        assert not c.call('run_replay',path='artifacts/controller-tampered.replay.json')['matched']
        assert c.call('inspect')==final
        c.call('reset');assert c.call('inspect')['character_controllers']==initial['character_controllers']
        c.call('load_scene',path='examples/npc-character-controller.scene.json');assert c.call('inspect')['character_controllers']==initial['character_controllers']
        checks.append('MCP-authored demo with green20deg/red40deg/blue20cm stairs and planar guard; save/load/reset, isolated prefix, tamper detection and live/replay PNG equality')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']}
        (ROOT/'artifacts/character-controller-verification.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
