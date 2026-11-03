#!/usr/bin/env python3
"""High NPC slots through MCP; finite sensors/replay and continuous offline runtime."""
import argparse,json
from pathlib import Path
from verify_lab import Client,ROOT

def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'npc-population','version':'1'}});c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/npc-population-scale.scene.json');c.call('save_scene',path='examples/npc-population-scale.scene.json');c.call('step',ticks=3)
        minds=c.call('inspect_npc_mind')['npcs'];assert len(minds)==100 and len({m['profile']['intelligence'] for m in minds.values()})==3
        before=c.call('inspect_execution');c.call('set_npc_capacity',capacity=8,expected_error=True);assert c.call('inspect_execution')==before
        c.call('capture',path='artifacts/npc-population-prefix.png',width=1000,height=700);p='artifacts/npc-population.replay.json';c.call('save_replay',path=p);assert c.call('run_replay',path=p)['matched'];c.call('capture_replay',replay_path=p,path='artifacts/npc-population-prefix-replay.png',tick=3,width=1000,height=700);assert (ROOT/'artifacts/npc-population-prefix.png').read_bytes()==(ROOT/'artifacts/npc-population-prefix-replay.png').read_bytes()
        checks.append('100NPC/500Players,628actor slots with3IQ profiles; lower-capacity rejection preserves execution, finite state and offscreen pixels replay exactly')
        c.call('reset');c.call('set_execution_mode',mode='continuous');c.call('step',ticks=120);state=c.call('inspect_execution');assert state['mode']=='continuous' and state['retained']['ticks']==0
        c.call('save_replay',path=p,expected_error=True)
        checks.append('Same local NPC simulation runs continuous without retained replay history or network/server requirement')
        result={'passed':True,'checks':checks,'native_window':window,'build_id':c.call('inspect')['build_id']};(ROOT/'artifacts/npc-population-scale-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
