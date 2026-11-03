#!/usr/bin/env python3
"""Wedges through the shared dispatcher: physical/sensory solid, authoring and offscreen parity."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def box(id,p,size,body='fixed',geometry=None,color=(0.2,0.6,0.5)):
    e={'id':id,'name':id,'body':body,'color':list(color),'transform':{'position':p,'size':size}}
    if geometry:e['geometry']=geometry
    return e


def npc(id,x,z):
    e=box(id,[x,0.8,z],[0.6,1.6,0.6],body='npc',color=(0.7,0.2,0.8))
    e['npc']={'surface':'floor','target':'player','speed':2,'stop_distance':0.3,'enabled':True,
              'awareness':{'mode':'perceptive','vision_range':20,'fov_degrees':360,'initial_facing_degrees':0,'memory_ticks':120,'search_ticks':60,'search_turn_degrees_per_second':0},
              'objectives':[{'priority':0,'when':'always','action':{'kind':'wait'}}]}
    e['vitality']={'initial':100,'maximum':100,'scale':1}
    return e


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-geometry-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':24,'depth':20,'seed':42,'obstacles':0})
        ramp=box('ramp',[0,1,0],[6,2,4],geometry={'kind':'wedge','rise':'positive_x'});ramp['sound_transmission']=0
        c.call('upsert',entity=ramp)
        def query(start,end,half=(0,0,0),interior=False):return c.call('query_geometry',entity='ramp',start=start,end=end,half_extent=list(half),interior=interior)
        before=c.call('inspect')
        assert query([-2,1.5,-3],[-2,1.5,3])['fraction'] is None
        assert query([2,1,-3],[2,1,3])['fraction'] is not None
        assert query([-2,0.6,-3],[-2,0.6,3],half=(0.4,0.4,0.4))['fraction'] is not None
        assert query([0,1,0],[0,1,0])['fraction']==0
        assert query([0,1,0],[0,1,0],interior=True)['fraction'] is None
        assert c.call('inspect')==before
        for bad in [dict(ramp,body='dynamic'),dict(ramp,body='player'),dict(ramp,geometry={'kind':'wedge','rise':'bad'}),dict(ramp,trigger={'enabled':True,'targets':['player']})]:
            c.call('upsert',entity=bad,expected_error=True);assert c.call('inspect')==before
        c.call('query_geometry',entity='ramp',start=[0,0,0],end=[1,1,1],half_extent=[-1,0,0],expected_error=True)
        checks.append('Shared query distinguishes solid/void, expanded volume and closed/open boundary; reads preserve snapshot and invalid combinations are atomic')

        c.call('capture',path='artifacts/geometry-wedge.png',width=800,height=600)
        c.call('capture',path='artifacts/geometry-wedge-repeat.png',width=800,height=600)
        assert (ROOT/'artifacts/geometry-wedge.png').read_bytes()==(ROOT/'artifacts/geometry-wedge-repeat.png').read_bytes()
        c.call('upsert',entity={**ramp,'geometry':{'kind':'box'}})
        c.call('capture',path='artifacts/geometry-box.png',width=800,height=600)
        assert (ROOT/'artifacts/geometry-box.png').read_bytes()!=(ROOT/'artifacts/geometry-wedge.png').read_bytes()
        c.call('upsert',entity={k:v for k,v in ramp.items() if k!='geometry'})
        c.call('capture',path='artifacts/geometry-box-default.png',width=800,height=600)
        assert (ROOT/'artifacts/geometry-box.png').read_bytes()==(ROOT/'artifacts/geometry-box-default.png').read_bytes()
        c.call('undo');c.call('undo');c.call('capture',path='artifacts/geometry-wedge-undo.png',width=800,height=600)
        assert (ROOT/'artifacts/geometry-wedge.png').read_bytes()==(ROOT/'artifacts/geometry-wedge-undo.png').read_bytes()
        # Looking down through a point in the void must hit the floor at Y0, not bounding-box top Y2.
        camera=c.call('inspect')['scene']['camera']
        c.call('set_camera',camera={'position':[-2,1.5,-8],'target':[-2,1.5,0],'fov_degrees':48})
        assert c.call('select_at',x=400,y=300,width=800,height=600)['selection'] is None
        c.call('set_camera',camera={'position':[2,1,-8],'target':[2,1,0],'fov_degrees':48})
        hit=c.call('select_at',x=400,y=300,width=800,height=600)
        assert hit['selection']=='ramp'
        c.call('capture',path='artifacts/geometry-selected.png',width=800,height=600)
        c.call('capture',path='artifacts/geometry-selected-small.png',width=400,height=300)
        c.call('set_camera',camera=camera);c.call('select_entity',id=None)
        checks.append('Shape changes invalidate render cache; box/default pixels agree, wedge/undo pixels agree, picking respects void/solid, outline and viewport captures succeed')

        base=c.call('inspect')['scene'];player=next(e for e in base['entities'] if e['id']=='player')
        abilities=[{'id':'direct','selection':'actor','range':20,'cast_ticks':0,'cooldown_ticks':0,'costs':[],'effects':[{'kind':'damage','recipient':'target','amount':20}]},
                   {'id':'shot','selection':'spatial','range':0,'cast_ticks':0,'cooldown_ticks':0,'costs':[],'delivery':{'kind':'projectile','speed':30,'half_extent':0.05,'max_distance':20},'effects':[{'kind':'damage','recipient':'target','amount':20}]},
                   {'id':'area','selection':'spatial','range':20,'cast_ticks':0,'cooldown_ticks':0,'costs':[],'area':{'radius':9,'max_targets':9,'include_source':False,'occlusion':True},'effects':[{'kind':'damage','recipient':'target','amount':20}]}]
        player['abilities']={'resources':[],'grants':['direct','shot','area']}
        for x,clear in [(-2,True),(2,False)]:
            observer=npc('observer',x,4);p=copy.deepcopy(player);p['transform']['position']=[x,0.8,-4]
            c.call('edit_scene',edits=[{'op':'upsert','entity':p},{'op':'upsert','entity':observer},{'op':'set_ability_catalog','catalog':{'resources':[],'abilities':abilities}}])
            c.call('step',ticks=1);assert c.call('inspect')['npcs']['observer']['observed_this_tick']==clear
            for slot in [1,2,3]:
                c.call('reset')
                if slot==1:c.call('activate_ability',actor='player',slot=slot,target='observer')
                else:c.call('aim_ability',actor='player',slot=slot,aim={'kind':'point','position':[x,0.8,4 if slot==2 else -4]})
                c.call('step',ticks=40);assert c.call('inspect_vitality')['actors']['observer']['units']==(80 if clear else 100)
            observer['npc']['awareness']['vision_range']=0.1;observer['npc']['awareness']['hearing']={'range':20}
            c.call('upsert',entity=observer);c.call('emit_noise',origin={'kind':'position','position':[x,0.8,-4]},radius=20);c.call('step',ticks=1)
            assert c.call('inspect')['npcs']['observer']['heard_this_tick']==clear
        checks.append('Vision, anonymous sound attenuation, direct ability LOS, free projectile sweep and area occlusion agree on void versus solid in live MCP runtime')

        c.call('remove',id='observer');flyer=npc('flyer',-2,-3);flyer['transform']={'position':[-2,1.2,-3],'size':[0.2,0.2,0.2]}
        flyer['npc'].update(target=None,mobility={'walk':False,'fly_speed':2},local_connections={'walk':False,'fly':True,'max_distance':8},objectives=[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[-2,1.2,3],'arrival_distance':0.2}}])
        template=copy.deepcopy(flyer);template['id']='hover_template';template['npc']['enabled']=False
        c.call('edit_scene',edits=[{'op':'upsert','entity':flyer},{'op':'set_npc_templates','templates':[template]}])
        q=c.call('query_spawn_support',template='hover_template',position=[-2,3,0],max_drop=5)
        assert q['report']['status']=='sloped_surface' and abs(q['report']['feet'][1]-1/3)<0.00001
        c.call('spawn_actor',template='hover_template',position=[-2,1.5,0]);c.call('step',ticks=1)
        assert c.call('inspect_population')['receipts'][-1]['outcome']=='spawned'
        c.call('reset');c.call('step',ticks=240);assert c.call('inspect')['runtime_positions']['flyer'][2]>2.7
        final=c.call('inspect');path='artifacts/geometry.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        prefix=c.call('inspect_replay',path=path,tick=40)['snapshot'];assert prefix['scene']['entities']==final['scene']['entities']
        c.call('capture',path='artifacts/geometry-final.png',width=800,height=600)
        c.call('capture_replay',replay_path=path,path='artifacts/geometry-replay.png',width=800,height=600)
        assert (ROOT/'artifacts/geometry-final.png').read_bytes()==(ROOT/'artifacts/geometry-replay.png').read_bytes()
        checks.append('Flight uses empty-volume clearance, hover spawn uses exact overlap, slope support rejects explicitly; replay and final capture reproduce geometry')

        # Saved demonstration: four static wedge orientations, one flying NPC traverses low-side void.
        c.call('set_npc_templates',templates=[])
        p=copy.deepcopy(player);p['transform']['position']=[0,0.8,7]
        demo=box('ramp',[-4,1,0],[6,2,4],geometry={'kind':'wedge','rise':'positive_x'})
        flyer['transform']={'position':[-6,1.3,-3],'size':[0.5,1,0.5]};flyer['npc']['objectives'][0]['action']['point']=[-6,1.3,3]
        edits=[{'op':'upsert','entity':demo},{'op':'upsert','entity':p},{'op':'upsert','entity':flyer},
               {'op':'set_camera','camera':{'position':[16,15,20],'target':[0,1,0],'fov_degrees':48}}]
        for i,(rise,pos) in enumerate([('negative_x',[4,1,0]),('positive_z',[-4,1,-6]),('negative_z',[4,1,-6])]):
            edits.append({'op':'upsert','entity':box('ramp_'+rise,pos,[6,2,4],geometry={'kind':'wedge','rise':rise},color=(0.3+0.15*i,0.5,0.7-0.1*i))})
        c.call('edit_scene',edits=edits);c.call('save_scene',path='examples/npc-geometry.scene.json');c.call('load_scene',path='examples/npc-geometry.scene.json')
        assert c.call('inspect')['scene']['version']==SCENE_VERSION
        c.call('capture',path='artifacts/geometry-demo.png',width=1000,height=700)
        c.call('step',ticks=240);assert c.call('inspect')['runtime_positions']['flyer'][2]>2.7
        c.call('save_replay',path='artifacts/geometry-demo.replay.json');assert c.call('run_replay',path='artifacts/geometry-demo.replay.json')['matched']
        result={'passed':True,'checks':checks+['MCP-created saved demo has all four orientations and unchanged legacy player motor; scripted flyer crosses a real wedge void'],'native_window':window,'build_id':c.call('inspect')['build_id']}
        (ROOT/'artifacts/geometry-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(result['checks']),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
