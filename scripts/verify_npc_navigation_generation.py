#!/usr/bin/env python3
"""Critical derivation contract: preview/edits/layout knowledge remain atomic and replayable."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client,ROOT
from verify_geometry import box


def actor():
    e=box('navigator',[-4,0.81,-4],[0.6,1.6,0.6],body='npc',color=(0.7,0.2,0.8));e['character_controller']={'snap_distance':0.3}
    e['npc']={'surface':'floor','speed':2,'stop_distance':0.2,'enabled':True,'mobility':{'walk':True},'local_connections':{'walk':True,'fly':False,'max_distance':2.5},'mind':{'intelligence':1},'objectives':[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[4,0.8,4],'arrival_distance':0.2}}]};return e


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-navigation-generation','version':'1'}});c.request('notifications/initialized',notification=True)
        def base():
            c.call('generate',recipe={'width':24,'depth':20,'seed':42,'obstacles':0});p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p['transform']['position']=[-8,0.8,7];p.pop('footsteps',None);n=actor();wall=box('wall',[0,1.5,0],[1,3,6]);c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [p,n,wall]]);return n,wall
        n,wall=base();recipe={'entity':'navigator','min':[-4,0,-4],'max':[4,0,4],'cell_size':2,'connection_radius':2.1}
        before=c.call('inspect');preview=c.call('query_navigation_generation',recipe=recipe);assert c.call('inspect')==before and preview['report']['nodes']>0
        c.call('set_navigation_generation',recipe=recipe);generated=c.call('inspect_mobility');assert generated['network']==preview['network'] and generated['generation']['report']==preview['report'];original=generated['network'];assert not generated['generation']['report']['limited']
        before=c.call('inspect');bad={**recipe,'cell_size':0};c.call('set_navigation_generation',recipe=bad,expected_error=True);assert c.call('inspect')==before
        c.call('set_debug_view',npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False);c.call('save_scene',path='examples/npc-generated-navigation.scene.json');c.call('step',ticks=120);prefix=c.call('inspect');c.call('capture',path='artifacts/npc-generated-navigation-prefix.png',width=1000,height=700);c.call('step',ticks=580);assert c.call('inspect_mobility')['states']['navigator']['phase']=='arrived'
        path='artifacts/npc-generated-navigation.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched'];recovered=c.call('inspect_replay',path=path,tick=120)['snapshot'];assert recovered['runtime_positions']==prefix['runtime_positions'];assert recovered['mobility']==prefix['mobility'];c.call('capture_replay',replay_path=path,path='artifacts/npc-generated-navigation-prefix-replay.png',tick=120,width=1000,height=700);assert (ROOT/'artifacts/npc-generated-navigation-prefix.png').read_bytes()==(ROOT/'artifacts/npc-generated-navigation-prefix-replay.png').read_bytes()
        moved=copy.deepcopy(wall);moved['transform']['position'][2]=8;c.call('upsert',entity=moved);assert c.call('inspect_mobility')['network']!=original;c.call('undo');assert c.call('inspect_mobility')['network']==original
        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');player['transform']['position']=[8,0.8,7];c.call('upsert',entity=player);assert c.call('inspect_mobility')['network']==original
        c.call('load_scene',path='examples/npc-generated-navigation.scene.json');assert c.call('inspect_mobility')['network']==original
        limited={**recipe,'max_probes':64};c.call('set_navigation_generation',recipe=limited);assert c.call('inspect_mobility')['generation']['report']['limited'];c.call('set_spatial_network',network=original);assert c.call('inspect_mobility')['generation'] is None
        checks.append('Geometry-derived wall detour, read-only preview, atomic errors, source invalidation versus actor movement, recipe save/load/undo, bounded truncation/manual override and exact replay/offscreen pixels')
        n,wall=base();n['transform']['position']=[-2,0.81,0];h=4*math.tan(math.radians(20));n['npc']['objectives'][0]['action']['point']=[6,h+0.8,0]
        c.call('edit_scene',edits=[{'op':'remove','id':'wall'},{'op':'upsert','entity':n},{'op':'upsert','entity':box('ramp',[2,h/2,0],[4,h,4],geometry={'kind':'wedge','rise':'positive_x'})},{'op':'upsert','entity':box('landing',[7,h/2,0],[6,h,4])}]);ramp_recipe={'entity':'navigator','min':[-2,0,-2],'max':[6,2,2],'cell_size':2,'connection_radius':2.5};c.call('set_navigation_generation',recipe=ramp_recipe);c.call('step',ticks=600);assert c.call('inspect_mobility')['states']['navigator']['phase']=='arrived',c.call('inspect_mobility')
        checks.append('Ramp sample heights accommodate actor footprint and KCC-generated links execute a real climb without authored waypoints')
        n,_=base();n['transform']['position']=[-2,0.81,-2];n['npc']['mobility']['fly_speed']=3;n['npc']['local_connections']['fly']=True;n['npc']['objectives'][0]['action']['point']=[2,4.8,2];c.call('edit_scene',edits=[{'op':'remove','id':'wall'},{'op':'upsert','entity':n}]);air={'entity':'navigator','min':[-2,0,-2],'max':[2,4,2],'cell_size':2,'connection_radius':2.1,'include_air':True};c.call('set_navigation_generation',recipe=air);network=c.call('inspect_mobility')['network'];assert any(l['mode']=='fly' for l in network['links']);c.call('step',ticks=600);assert c.call('inspect_mobility')['states']['navigator']['phase']=='arrived'
        checks.append('Free-space XYZ nodes require fly capability; generated aerial links use the same hybrid physics and known-goal planner')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/npc-navigation-generation-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
