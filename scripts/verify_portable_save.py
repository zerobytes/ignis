#!/usr/bin/env python3
"""Portable saves across live gameplay, service atomicity and local replay. No window/audio."""
import argparse,json,hashlib
from pathlib import Path
from verify_lab import Client,ROOT

def verify(binary,window=False):
    c=Client(binary,False);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'portable-save','version':'1'}})
        c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/npc-v1.scene.json')
        c.call('save_scene',path='examples/npc-v1.scene.json')
        control=c.call('bind_control',actor='player')['control']
        c.call('talk_npc',actor='player',npc='ally')
        c.call('interact',actor='player',point='door',action='open')
        path='artifacts/current.ignis-portable'
        c.call('save_portable_save',path=path,expected_error=True)
        c.call('step',ticks=60)
        session=c.call('inspect_dialogue',actor='player')['handle']
        before=c.call('inspect');c.call('save_portable_save',path=path)
        saved=(ROOT/path).read_bytes();assert saved[:8]==b'IGNISPSV'
        assert c.call('inspect_portable_save',path=path)['validated']
        c.call('capture',path='artifacts/portable-before.png',width=600,height=420)
        c.call('step',ticks=10);c.call('load_portable_save',path=path,mode='recorded')
        after=c.call('inspect')
        for key in ['runtime_positions','npcs','interactions','presentation','character_controllers']:
            assert after[key]==before[key],key
        c.call('set_control_input',control=control,input={'x':1,'z':0},expected_error=True)
        c.call('choose_dialogue',actor='player',session=session,choice='yes',expected_error=True)
        new=c.call('inspect_dialogue',actor='player')['handle'];assert new!=session
        c.call('choose_dialogue',actor='player',session=new,choice='yes')
        c.call('step',ticks=180)
        replay='artifacts/portable.replay.json';c.call('save_replay',path=replay)
        assert c.call('run_replay',path=replay)['matched']
        prefix=c.call('inspect_replay',path=replay,tick=0)
        assert prefix['snapshot']['tick']==60 and prefix['snapshot']['runtime_positions']==before['runtime_positions']
        c.call('capture_replay',replay_path=replay,path='artifacts/portable-prefix.png',tick=0,width=600,height=420)
        assert (ROOT/'artifacts/portable-before.png').read_bytes()==(ROOT/'artifacts/portable-prefix.png').read_bytes()
        checks.append('Portable live gameplay restore preserves exact initial state/pose, expires external tokens and continues local replay/prefix pixels without old history')
        native='artifacts/current-native.ignis-save';c.call('save_checkpoint',path=native)
        c.call('migrate_portable_save',source_path=native,path=path)
        c.call('inspect_portable_save',path=path)
        stable=(ROOT/path).read_bytes();before=c.call('inspect')
        broken=bytearray(stable);broken[-1]^=1
        bad='artifacts/corrupt.ignis-portable';(ROOT/bad).write_bytes(broken)
        c.call('load_portable_save',path=bad,expected_error=True)
        c.call('migrate_portable_save',source_path=bad,path=path,expected_error=True)
        assert c.call('inspect')==before and (ROOT/path).read_bytes()==stable
        c.call('load_portable_save',path=path,mode='continuous');c.call('step',ticks=10)
        c.call('save_portable_save',path=path);c.call('load_portable_save',path=path)
        assert c.call('inspect_execution')['retained']['ticks']==0
        c.call('start_multiplayer_host',address='127.0.0.1:0',capacity=1);c.call('pause')
        before=c.call('inspect');c.call('load_portable_save',path=path,expected_error=True)
        assert c.call('inspect')==before;c.call('stop_multiplayer_host')
        checks.append('Native migration, corrupt-load/migration rollback preserving destination, continuous no-log restore and active-host refusal')
        report={'passed':True,'checks':checks,'build_id':c.call('inspect')['build_id'],'format_sha256':hashlib.sha256(saved).hexdigest(),'platform_validation':'macOS only; no Windows/Linux physical acceptance','native_window':False}
        (ROOT/'artifacts/portable-save-verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(checks),'build_id':report['build_id']}))
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),False)
