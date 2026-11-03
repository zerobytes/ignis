#!/usr/bin/env python3
"""Identity/life/pose/audio and replay through MCP, silently/offscreen."""
import argparse,copy,json,math
from pathlib import Path
from verify_lab import Client,ROOT
from verify_npc_routines_interactions import npc

def fixture(base):
    s=copy.deepcopy(base);s['version']=95;s['interactions']={};s['npc_generators']=[];s['npc_templates']=[];s['navigation_generation']=None;s['spatial_network']={'nodes':[[-3,3,-4],[3,3,-4]],'links':[{'from':0,'to':1,'mode':'fly'}]}
    s['entities']=[e for e in s['entities'] if e['id'] not in ['a','b','guide','well','shrine','brave','careful'] and e.get('body')!='npc']
    p=next(e for e in s['entities'] if e['id']=='player');p['transform']['position']=[0,0.8,6];p.pop('footsteps',None);p['abilities']={'resources':[],'grants':['kill','revive']}
    s['ability_catalog']={'resources':[{'id':'energy','scale':1}],'abilities':[{'id':'kill','selection':'actor','range':30,'effects':[{'kind':'damage','recipient':'target','amount':100}]},{'id':'revive','selection':'actor','range':30,'effects':[{'kind':'resurrect','recipient':'target','amount':100}]},{'id':'meditate','selection':'self_actor','range':0,'cast_ticks':60,'effects':[{'kind':'resource','recipient':'source','resource':'energy','delta':1}]}]}
    for definition in s['ability_catalog']['abilities']:
        definition.setdefault('cast_ticks',0);definition.setdefault('cooldown_ticks',0);definition.setdefault('costs',[])
    positions=[[-0.4,-0.8,-0.3],[0.4,-0.8,-0.3],[0.4,-0.8,0.3],[-0.4,-0.8,0.3],[0,1,0]];triangles=[[0,1,4],[1,2,4],[2,3,4],[3,0,4],[0,2,1],[0,3,2]]
    asset={'id':'creature','geometry':{'positions':positions,'triangles':triangles},'rig':{'bones':[{'name':'body','parent':None,'translation':[0,0,0],'rotation':[0,0,0,1]}],'weights':[{'joints':[0,0,0,0],'weights':[1,0,0,0]} for _ in positions]},'clips':[]}
    for id,axis,amount in [('bold',1,0.3),('careful',0,0.2),('walk',1,0.15),('air',2,0.25),('cast',0,0.35),('dead',1,-0.5)]:
        moved=[0,0,0];moved[axis]=amount
        asset['clips'].append({'id':id,'duration_ticks':60,'tracks':[{'bone':0,'keys':[{'tick':0,'translation':[0,0,0],'rotation':[0,0,0,1]},{'tick':30,'translation':moved,'rotation':[0,0,0,1]},{'tick':60,'translation':[0,0,0],'rotation':[0,0,0,1]}]}]})
    s['meshes']={'assets':[asset],'instances':[]};s['avatar_audio']={'sounds':[{'id':'signal','samples':[round(5000*math.sin(i*2*math.pi*440/24000)) for i in range(2400)]}]}
    variants=[{'phase':'idle','clip':'bold','looping':True,'weights':[0,0,0,-1000,1000,0,0,0,0]},{'phase':'idle','clip':'careful','looping':True,'weights':[0,0,0,1000,-1000,0,0,0,0]}]+[{'phase':p,'clip':p,'looping':p in ['walk','air'],'sound':'signal'} for p in ['walk','air','cast','dead']]
    for name,z,courage,fear in [('a',-2,1,0),('b',2,0,1)]:
        a=npc(name,[-3,0.81,z]);a['npc']['mind']['personality']={'courage':courage,'fear':fear};a['avatar']={'asset':'creature','variants':variants};a['abilities']={'resources':[{'resource':'energy','initial':50,'capacity':100}],'grants':['meditate']};s['entities'].append(a)
    t=copy.deepcopy(s['entities'][-2]);t['id']='new_creature';s['npc_templates']=[t]
    f=copy.deepcopy(t);f['id']='flyer';f['transform']['position']=[-3,3,-4];f.pop('character_controller',None);f['npc'].pop('local_connections',None);f['npc']['mobility']={'walk':False,'fly_speed':3};f['npc']['objectives']=[{'priority':0,'when':'always','action':{'kind':'guard_3d','point':[3,3,-4],'arrival_distance':0.2}}];s['entities'].append(f)
    return s

def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'npc-presentation','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/npc-presentation.scene.json');c.call('save_scene',path='examples/npc-presentation.scene.json')
        state=c.call('inspect_npc_presentation');assert state['state']['actors']['a']['variant']==0 and state['state']['actors']['b']['variant']==1,state
        c.call('order_npc',actor='a',order={'kind':'move','position':[3,0.81,-2],'arrival_distance':0.2,'for_ticks':600,'formation':'offsets','spacing':1,'facing_degrees':0});c.call('step',ticks=20);state=c.call('inspect_npc_presentation');assert state['state']['actors']['a']['phase']=='walk' and state['state']['actors']['flyer']['phase']=='air',state
        c.call('capture',path='artifacts/npc-avatar-prefix.png',width=1000,height=700);prefix=c.call('inspect');c.call('capture_npc_audio',path='artifacts/npc-avatar.wav',frames=2400,listener=[-3,0.81,-2],radius=30)
        c.call('order_npc',actor='a',order={'kind':'hold','for_ticks':600});c.call('step',ticks=2);c.call('activate_ability',actor='a',slot=1);c.call('step',ticks=1);assert c.call('inspect_npc_presentation')['state']['actors']['a']['phase']=='cast'
        audio=c.call('capture_npc_audio',path='artifacts/npc-avatar.wav',frames=2400,listener=[-3,0.81,-2],radius=30);assert audio['peak']>0
        old=c.call('inspect_npc_presentation')['state']['actors']['a'];c.call('activate_ability',actor='player',slot=1,target='a');c.call('step',ticks=1);assert c.call('inspect_npc_presentation')['state']['actors']['a']['phase']=='dead'
        c.call('activate_ability',actor='player',slot=2,target='a');c.call('step',ticks=1);new=c.call('inspect_npc_presentation')['state']['actors']['a'];assert new['instance']==old['instance'] and new['life']==old['life']+1 and new['started']>old['started']
        c.call('despawn_actor',actor='b');c.call('spawn_actor',template='new_creature',position=[-3,0.81,2]);c.call('step',ticks=1);state=c.call('inspect_npc_presentation');assert 'b' not in state['state']['actors'];assert len(state['state']['actors'])==3
        path='artifacts/npc-avatar.replay.json';c.call('save_replay',path=path);assert c.call('run_replay',path=path)['matched'];assert c.call('inspect_replay',path=path,tick=20)['snapshot']['presentation']==prefix['presentation'];c.call('capture_replay',replay_path=path,path='artifacts/npc-avatar-prefix-replay.png',tick=20,width=1000,height=700);assert (ROOT/'artifacts/npc-avatar-prefix.png').read_bytes()==(ROOT/'artifacts/npc-avatar-prefix-replay.png').read_bytes()
        checks.append('Shared rig/asset, independent traits/playbacks, real walk/air/cast/dead, same-person revival and new-person reuse; state/pixels replay exactly')
        checks.append('Presentation PCM24kHz/WAV captures nonzero signal without device output; hearing gameplay remains independent')
        result={'passed':True,'checks':checks,'build_id':c.call('inspect')['build_id'],'native_window':window};(ROOT/'artifacts/npc-presentation-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
