#!/usr/bin/env python3
"""Headless/offscreen full presentation integration. Never enables device output."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_network import wait
from npc_presentation_demo import scene as demo_scene


def verify(binary, window=False):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'presentation-complete','version':'1'}})
        c.request('notifications/initialized', notification=True)
        fixture=ROOT/'artifacts/npc-presentation-input.scene.json'
        fixture.write_text(json.dumps(demo_scene(ROOT/'examples/npc-presentation.scene.json'))+'\n')
        c.call('load_scene', path='artifacts/npc-presentation-input.scene.json')
        assert not c.call('inspect_npc_audio_output')['enabled']
        c.call('set_npc_audio_output', enabled=False, listener=[0,1,0], radius=30)
        # Retarget live through the common authoring transaction, then use baked clip.
        source = c.call('inspect_mesh', asset='canid')['asset']
        target = c.call('inspect_mesh', asset='grazer')['asset']
        mappings = [dict(source=a['name'], target=b['name'], basis=[0,0,0,1], translation_scale=1.2 if i==0 else 0) for i,(a,b) in enumerate(zip(source['rig']['bones'],target['rig']['bones']))]
        c.call('edit_mesh', edit=dict(kind='retarget_clip', asset='grazer', source='canid', clip='walk', output='walk', mappings=mappings))
        baked=c.call('inspect_mesh',asset='grazer')['asset']
        assert next(x for x in baked['clips'] if x['id']=='walk')!=next(x for x in target['clips'] if x['id']=='walk')
        c.call('save_scene', path='examples/npc-presentation-complete.scene.json')
        c.call('capture', path='artifacts/npc-presentation-morphologies.png', width=1200, height=850)
        c.call('order_npc', actor='a', order=dict(kind='move', position=[3,.81,-2], arrival_distance=.2, for_ticks=600, formation='offsets', spacing=1, facing_degrees=0))
        flyer_start=c.call('inspect')['runtime_positions']['flyer']
        c.call('step', ticks=61)
        flying=c.call('inspect')
        assert flying['runtime_positions']['flyer'][0]-flyer_start[0]>2, flying['mobility']['flyer']
        assert flying['mobility']['flyer']['phase']=='traversing', flying['mobility']['flyer']
        state=c.call('inspect_npc_presentation')
        next_step=state['state']['actors']['a']['next_step']
        if next_step>state['tick']:
            c.call('step',ticks=next_step-state['tick'])
        snapshot = c.call('inspect_npc_presentation')
        assert snapshot['state']['actors']['a']['phase']=='walk', snapshot
        assert 'a' in snapshot['state']['motion'], snapshot
        assert any(p['joint_count']>0 for p in snapshot['poses']), snapshot
        assert len({p['asset'] for p in snapshot['poses']})==3
        c.call('capture', path='artifacts/npc-presentation-contact.png', width=1200, height=850)
        audio=c.call('capture_npc_audio', path='artifacts/npc-presentation-complete.wav', frames=2400, listener=[-2,.81,-2], radius=30)
        assert audio['peak']>0, audio
        c.call('save_checkpoint', path='artifacts/npc-presentation-complete.ignis-save')
        c.call('step', ticks=20)
        continued = c.call('inspect_npc_presentation')
        c.call('load_checkpoint', path='artifacts/npc-presentation-complete.ignis-save')
        assert c.call('inspect_npc_presentation')['poses']==snapshot['poses']
        c.call('step', ticks=20)
        assert c.call('inspect_npc_presentation')['state']==continued['state']
        assert c.call('inspect_npc_presentation')['poses']==continued['poses']
        path='artifacts/npc-presentation-complete.replay.json'
        c.call('save_replay', path=path)
        assert c.call('run_replay', path=path)['matched']
        c.call('capture_replay', replay_path=path, path='artifacts/npc-presentation-contact-replay.png', tick=0, width=1200, height=850)
        assert (ROOT/'artifacts/npc-presentation-contact.png').read_bytes()==(ROOT/'artifacts/npc-presentation-contact-replay.png').read_bytes()
        checks.append('Mapped rig bake, articulated quadrupeds/six limbs, physical contacts and root clock through native/MCP')
        checks.append('Checkpoint with active root motion resumes exact state/poses; baseline replay pixels match')
        checks.append('Audio commands available; PCM capture works with device output disabled')
        old=c.call('inspect_npc_presentation')['state']['actors']['a']
        c.call('activate_ability',actor='a',slot=1)
        c.call('step',ticks=30)
        assert c.call('inspect_npc_presentation')['state']['actors']['a']['phase']=='cast'
        c.call('capture',path='artifacts/npc-presentation-cast.png',width=1200,height=850)
        c.call('activate_ability',actor='player',slot=1,target='a')
        c.call('step',ticks=31)
        assert c.call('inspect_npc_presentation')['state']['actors']['a']['phase']=='dead'
        c.call('capture',path='artifacts/npc-presentation-dead.png',width=1200,height=850)
        c.call('activate_ability',actor='player',slot=2,target='a')
        c.call('step',ticks=1)
        new=c.call('inspect_npc_presentation')['state']['actors']['a']
        assert new['instance']==old['instance'] and new['life']==old['life']+1
        checks.append('Articulated cast/death/revival captured offscreen with the same life and movement contracts')
        # Every actual change is transactional; invalid root configuration preserves scene.
        before=c.call('inspect')['scene']
        actor=copy.deepcopy(next(e for e in before['entities'] if e['id']=='a'))
        actor['avatar']['root_motion']['bone']=31
        c.call('edit_scene', edits=[dict(op='upsert',entity=actor)], expected_error=True)
        assert c.call('inspect')['scene']==before
        checks.append('Invalid morphology/root dependency rejected atomically')
        peer=Client(binary,False)
        try:
            peer.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'public-avatar','version':'1'}})
            peer.request('notifications/initialized',notification=True)
            c.call('reset')
            c.call('start_multiplayer_host',address='127.0.0.1:0',capacity=1)
            server=wait(c,'inspect_multiplayer_host',lambda s:s['transport']['state']=='listening')['transport']
            peer.call('connect_multiplayer',address=server['address'],certificate_der=server['certificate_der'])
            wait(peer,'inspect_multiplayer_client',lambda s:s.get('content') and s.get('replica_tick') is not None)
            c.call('pause')
            tick=c.call('inspect')['tick']
            align=(1-tick)%6
            if align:c.call('step',ticks=align)
            tick=c.call('inspect')['tick']
            wait(peer,'inspect_multiplayer_client',lambda s:s.get('replica_tick')==tick)
            public=peer.call('inspect_npc_presentation')
            authoritative=c.call('inspect_npc_presentation')
            assert public['public_only'] and public['poses']==authoritative['poses'], (public,authoritative)
            assert not public['audio']['bank_available'] and 'state' not in public
            remote=peer.call('capture_multiplayer',path='artifacts/npc-presentation-remote.png',width=1200,height=850)
            local=c.call('capture',path='artifacts/npc-presentation-host.png',width=1200,height=850)
            assert remote['mesh_vertex_count']==local['mesh_vertex_count']>10000
            metrics=c.call('compare_images',reference_path='artifacts/npc-presentation-host.png',candidate_path='artifacts/npc-presentation-remote.png',difference_path='artifacts/npc-presentation-network-difference.png')['report']['metrics']
            assert metrics['ssim_luma']>.995,metrics
            peer.call('capture_npc_audio',path='artifacts/npc-remote-forbidden.wav',frames=400,listener=[0,1,0],radius=30,expected_error=True)
            peer.call('disconnect_multiplayer')
            c.call('stop_multiplayer_host')
            checks.append('Real QUIC/public content preserves IK/root/cast pose fields and render; no private state or PCM bank on client')
        finally:
            peer.close()
        report=dict(passed=True, checks=checks, build_id=c.call('inspect')['build_id'], native_window=window, device_output=False)
        (ROOT/'artifacts/npc-presentation-complete-verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(dict(passed=True, checks=len(checks), build_id=report['build_id'])))
    finally:
        c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
