#!/usr/bin/env python3
"""Critical MCP proof: routine resources, dialogue revision and real mutable colliders."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client,ROOT
from verify_geometry import box

def npc(name,p):
    e=box(name,p,[0.6,1.6,0.6],body='npc',color=(0.6,0.2,0.8));e['blocks_vision']=False;e['vitality']={'initial':100,'maximum':100,'scale':1};e['character_controller']={'snap_distance':0.3}
    e['npc']={'surface':'floor','speed':2,'stop_distance':0.2,'enabled':True,'mind':{'intelligence':1},'circulation':{},'mobility':{'walk':True},'local_connections':{'walk':True,'fly':False,'max_distance':8},'objectives':[{'priority':0,'when':'always','action':{'kind':'wait'}}]};return e

def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-routines-interactions','version':'1'}});c.request('notifications/initialized',notification=True)
        def base():
            c.call('generate',recipe={'width':30,'depth':24,'seed':42,'obstacles':0});p=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player');p.pop('footsteps',None);return p
        def level(name,key):
            return next(r['units'] for r in c.call('inspect_abilities')['actors'][name]['resources'] if r['id']==key)
        player=base();player['transform']['position']=[-6.5,0.8,0];player['abilities']={'resources':[{'resource':'energy','initial':10,'capacity':10}],'grants':[]}
        catalog={'resources':[{'id':r,'scale':1} for r in ['charge','bold','safe','energy']],'abilities':[]}
        guide=npc('guide',[-5,0.81,0]);guide['npc']['dialogue']='guide';members=[]
        for name,z,courage,fear,color in [('brave',-2,1,0,[0.2,0.5,0.9]),('careful',2,0,1,[0.9,0.4,0.2])]:
            n=npc(name,[-3,0.81,z]);n['color']=color;n['abilities']={'resources':[{'resource':r,'initial':0,'capacity':100} for r in ['charge','bold','safe']],'grants':[]};n['npc']['mind']['personality']={'courage':courage,'fear':fear}
            n['npc']['routine']={'auto_dialogue':True,'needs':[{'resource':'charge','tag':'recharge','action':'charge','delta':0}],'schedule':[{'start':0,'end':800,'point':'shrine','action':'speak'}]};n['npc']['objectives']=[{'priority':0,'when':'always','action':{'kind':'routine'}}];members.append(n)
        well=box('well',[4,0.8,0],[0.2,0.2,0.2],body=None);shrine=box('shrine',[5,0.8,0],[0.2,0.2,0.2],body=None)
        document={'values':[{'id':'visits','initial':0,'minimum':0,'maximum':10}],'points':[{'id':'well','entity':'well','tags':['recharge'],'actions':['charge','fail']},{'id':'shrine','entity':'shrine','actions':['speak']}],
            'actions':[{'id':'charge','range':2,'cooldown_ticks':10,'effects':[{'kind':'resource','resource':'charge','delta':100}]},{'id':'speak','range':3,'dialogue':'oracle'},{'id':'fail','range':8,'effects':[{'kind':'resource','resource':'energy','delta':-5},{'kind':'add_value','key':'visits','delta':1000}]}],
            'dialogues':[{'id':'oracle','entry':'offer','nodes':[{'id':'offer','text':'Escolha um caminho.','choices':[{'id':'bold','text':'Aceitar desafio','personality_weights':[0,0,0,-1000,1000,0,0,0,0],'effects':[{'kind':'resource','resource':'bold','delta':1}]},{'id':'safe','text':'Escolher seguranca','personality_weights':[0,0,0,1000,-1000,0,0,0,0],'effects':[{'kind':'resource','resource':'safe','delta':1}]}]}]},
                {'id':'guide','entry':'loop','lock_locomotion':True,'nodes':[{'id':'loop','text':'Visitar?','choices':[{'id':'repeat','text':'Uma visita','next':'loop','effects':[{'kind':'resource','resource':'energy','delta':-1},{'kind':'add_value','key':'visits','delta':1}]},{'id':'finish','text':'Terminar','next':'thanks'}]},{'id':'thanks','text':'Fechado.','choices':[]}]}]}
        c.call('edit_scene',edits=[{'op':'set_ability_catalog','catalog':catalog},{'op':'set_interactions','document':document}]+[{'op':'upsert','entity':e} for e in [player,guide,well,shrine,*members]])
        c.call('set_debug_view',npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False);c.call('save_scene',path='examples/npc-routines.scene.json');c.call('step',ticks=300)
        prefix=c.call('inspect');c.call('capture',path='artifacts/npc-routines-prefix.png',width=1000,height=700);c.call('step',ticks=300)
        assert level('brave','charge')==100 and level('careful','charge')==100
        assert level('brave','bold')==1 and level('brave','safe')==0 and level('careful','safe')==1 and level('careful','bold')==0
        path='artifacts/npc-routines.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched'];recovered=c.call('inspect_replay',path=path,tick=300)['snapshot'];assert recovered['npcs']==prefix['npcs'] and recovered['interactions']==prefix['interactions']
        c.call('capture_replay',replay_path=path,path='artifacts/npc-routines-prefix-replay.png',tick=300,width=1000,height=700);assert (ROOT/'artifacts/npc-routines-prefix.png').read_bytes()==(ROOT/'artifacts/npc-routines-prefix-replay.png').read_bytes()
        checks.append('Both NPCs physically replenish their own registered need, then choose opposite dialogue branches by personality exactly once per schedule cycle; prefix/state/pixels replay exactly')
        c.call('talk_npc',actor='player',npc='guide');c.call('step',ticks=1);handle=c.call('inspect_dialogue',actor='player')['handle']
        for _ in range(2):c.call('choose_dialogue',actor='player',session=handle,choice='repeat')
        c.call('step',ticks=1);assert level('player','energy')==9 and c.call('inspect_interactions')['state']['values'][0]==1
        c.call('choose_dialogue',actor='player',session=handle,choice='finish');c.call('step',ticks=1);assert c.call('inspect_dialogue',actor='player')['session'] is None
        before=c.call('inspect');c.call('choose_dialogue',actor='player',session=handle,choice='repeat',expected_error=True);assert c.call('inspect')==before
        c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        checks.append('Duplicate queued choice cannot reuse a node revision to double-debit or double-advance; ended handle rejects atomically, session progress survives ring-independent replay')
        player=base();player['transform']['position']=[-2,0.8,4];player['abilities']={'resources':[{'resource':'energy','initial':10,'capacity':10}],'grants':[]}
        door=box('door',[0,1.5,4],[0.6,3,1]);traveller=npc('traveller',[-4,0.81,0]);traveller['npc']['mobility']['fly_speed']=3;traveller['npc']['objectives']=[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[4,0.8,0],'arrival_distance':0.2}}]
        template=copy.deepcopy(traveller);template['id']='template';flyer=npc('flyer',[-4,4.81,0]);flyer['npc']['mobility']['fly_speed']=3;flyer['npc'].pop('local_connections');flyer['npc']['flight']={'mode':'curved','acceleration':8,'deceleration':8,'turn_degrees_per_second':180,'smoothing_seconds':0.1,'minimum_speed':0,'can_hover':True,'initial_direction':[1,0,0]};flyer['npc']['objectives']=[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[4,4.8,0],'arrival_distance':0.2}}]
        objects={'values':[{'id':'closes','initial':0,'minimum':0,'maximum':10}],'points':[{'id':'door','entity':'door','mutable':True,'actions':['ghost','close']}],'actions':[{'id':'ghost','range':3,'effects':[{'kind':'object','point':'door','state':{'solid':False,'visible':True,'blocks_vision':False,'sound_transmission':1}}]},{'id':'close','range':3,'effects':[{'kind':'resource','resource':'energy','delta':-5},{'kind':'add_value','key':'closes','delta':1},{'kind':'object','point':'door','state':{'solid':True,'visible':True,'blocks_vision':True,'sound_transmission':0}}]}]}
        recipe={'entity':'traveller','min':[-4,0,0],'max':[4,4,4],'cell_size':4,'connection_radius':4.1,'include_air':True}
        c.call('edit_scene',edits=[{'op':'set_ability_catalog','catalog':{'resources':[{'id':'energy','scale':1}],'abilities':[]}},{'op':'set_interactions','document':objects},{'op':'set_npc_templates','templates':[template]},{'op':'set_navigation_generation','recipe':recipe}]+[{'op':'upsert','entity':e} for e in [player,door,traveller,flyer]])
        c.call('step',ticks=120);assert c.call('inspect')['character_controllers']['flyer']['airborne'];old=c.call('inspect_mobility')['network']
        c.call('despawn_actor',actor='traveller');c.call('spawn_actor',template='template',position=[-6,0.81,6]);c.call('interact',actor='player',point='door',action='ghost');c.call('step',ticks=1);assert c.call('inspect_mobility')['network']!=old
        c.call('set_input',input={'x':1,'z':0})
        for _ in range(120):
            if c.call('inspect')['runtime_positions']['player'][0]>=-0.05:break
            c.call('step',ticks=1)
        c.call('set_input',input={'x':0,'z':0});c.call('interact',actor='player',point='door',action='close');c.call('step',ticks=1)
        state=c.call('inspect_interactions')['state'];assert not state['objects'][0]['solid'] and state['values'][0]==0 and level('player','energy')==10
        c.call('set_input',input={'x':-1,'z':0})
        for _ in range(120):
            if c.call('inspect')['runtime_positions']['player'][0]<=-2:break
            c.call('step',ticks=1)
        c.call('set_input',input={'x':0,'z':0});c.call('interact',actor='player',point='door',action='close');c.call('step',ticks=1);assert level('player','energy')==5
        c.call('set_input',input={'x':1,'z':0});c.call('step',ticks=90);c.call('set_input',input={'x':0,'z':0});assert c.call('inspect')['runtime_positions']['player'][0]<-0.5
        c.call('step',ticks=300);assert c.call('inspect_mobility')['states']['flyer']['phase']=='arrived';path='artifacts/npc-runtime-objects.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched']
        checks.append('Open collider really permits passage; occupied close rolls back resource/value/object writes, empty close restores collision; retired bake source and changed graph preserve curved flight and replay')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/npc-routines-interactions-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
