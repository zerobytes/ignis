#!/usr/bin/env python3
"""Bounded recovery and physical passage reopening through real MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_targeting import actor

def verify(binary, window):
    c=Client(binary,window);checks=[];samples=[]
    def nav(): return c.call('inspect_navigation')['npcs']['guard']
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-recovery-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('generate',recipe={'width':16,'depth':12,'seed':42,'obstacles':0})
        player=next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player')
        player['transform']['position']=[0,0.85,0];player.pop('footsteps',None)
        guard=actor('guard','guards',[-4,0.85,0],[0.7,0.2,0.8])
        guard['npc'].update(enabled=True,speed=2,stop_distance=0.3,
            navigation_recovery={'retry_ticks':15,'max_retry_ticks':60,'stalled_ticks':30,'min_progress':0.1},
            objectives=[{'priority':1,'when':'always','action':{'kind':'guard','point':[4,0],'arrival_distance':0.3}}])
        walls=[{'id':id,'name':id,'body':'fixed','color':[0.2,0.65,0.45],'transform':{'position':[0,1.5,z],'size':[0.4,3,5.9]}} for id,z in [('north',3.65),('south',-3.65)]]
        c.call('edit_scene',edits=[{'op':'upsert','entity':e} for e in [player,guard,*walls]]+[{'op':'set_camera','camera':{'position':[12,17,17],'target':[0,0,0],'fov_degrees':48}}])
        c.call('set_debug_view',npc_vision=False,npc_hearing=False,noise_events=False,npc_routes=True);c.call('select_entity',id='guard')
        c.call('save_scene',path='examples/npc-recovery.scene.json')
        c.call('step',ticks=100);blocked=nav();samples.append(blocked)
        assert blocked['status']=='retry_wait' and blocked['recovery']['path_attempts']==3 and blocked['recovery']['failures']==3,blocked
        c.call('capture',path='artifacts/recovery-blocked.png',width=1280,height=900)
        c.call('save_replay',path='artifacts/recovery-blocked.replay.json');assert c.call('run_replay',path='artifacts/recovery-blocked.replay.json')['matched']
        c.call('capture_replay',replay_path='artifacts/recovery-blocked.replay.json',path='artifacts/recovery-blocked-replay.png',width=1280,height=900)
        assert (ROOT/'artifacts/recovery-blocked.png').read_bytes()==(ROOT/'artifacts/recovery-blocked-replay.png').read_bytes()
        checks.append('Blocked doorway waits with exact capped retry schedule and matching replay pixels')
        c.call('set_input',input={'x':1,'z':1});c.call('step',ticks=60);c.call('set_input',input={'x':0,'z':0});c.call('step',ticks=360)
        end=c.call('inspect');samples.append(nav());assert end['runtime_positions']['guard'][0]>3.5 and nav()['status']=='arrived',end
        assert nav()['recovery']['consecutive_failures']==0
        c.call('save_replay',path='artifacts/recovery.replay.json');assert c.call('run_replay',path='artifacts/recovery.replay.json')['matched']
        live=c.call('inspect');prefix=c.call('inspect_replay',path='artifacts/recovery.replay.json',tick=100);assert prefix['snapshot']['navigation']['guard']==blocked and c.call('inspect')==live
        c.call('capture',path='artifacts/recovery-arrived.png',width=1280,height=900)
        checks.append('Player physically vacates passage; guard resumes and arrives without scene reset; replay prefix remains isolated')
        before=c.call('inspect');bad=copy.deepcopy(guard);bad['npc']['navigation_recovery']['retry_ticks']=0
        c.call('upsert',entity=bad,expected_error=True);assert c.call('inspect')==before
        c.call('load_scene',path='examples/npc-recovery.scene.json');assert nav()['recovery']['failures']==0
        c.call('undo');assert nav()['recovery']['failures']==0;c.call('redo');assert nav()['recovery']['failures']==0
        c.call('step',ticks=1);before=c.call('inspect');c.call('set_debug_view',npc_routes=False);c.call('capture',path='artifacts/recovery-no-guides.png',width=1280,height=900);after=c.call('inspect');assert before['navigation']==after['navigation'] and before['runtime_positions']==after['runtime_positions']
        c.call('set_debug_view',npc_routes=True);c.call('reset')
        checks.append('MCP authoring, atomic validation, save/load/undo/redo/reset and diagnostic guides preserve simulation')
        result={'passed':True,'checks':checks,'samples':samples,'native_window':window,'build_id':c.call('inspect')['build_id'],'slow_requests':c.slow_requests}
        (ROOT/'artifacts/recovery-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
