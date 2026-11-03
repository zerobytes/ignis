#!/usr/bin/env python3
"""Publish and inspect the editable female base through the real native MCP API."""
import argparse
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / 'artifacts/character-base'
sys.path.insert(0, str(ROOT / 'scripts'))
from verify_lab import Client


def qmul(a, b):
    x,y,z,w = a
    X,Y,Z,W = b
    return [w*X+x*W+y*Z-z*Y,w*Y-x*Z+y*W+z*X,w*Z+x*Y-y*X+z*W,w*W-x*X-y*Y-z*Z]


def rotate(q, v):
    return qmul(qmul(q, [*v,0]), [-q[0],-q[1],-q[2],q[3]])[:3]


def posed_clip(asset, rotations, name):
    """Author global-axis deltas in each bone's rest frame, then bake native tracks."""
    rig = asset['rig']
    globals_ = []
    for b in rig['bones']:
        globals_.append(b['rotation'] if b['parent'] is None else qmul(globals_[b['parent']], b['rotation']))
    tracks = []
    for i,b in enumerate(rig['bones']):
        if b['name'] not in rotations:
            continue
        axis, degrees = rotations[b['name']]
        q = globals_[i]
        axis = rotate([-q[0],-q[1],-q[2],q[3]], axis)
        angle = math.radians(degrees)/2
        delta = [*(v*math.sin(angle) for v in axis),math.cos(angle)]
        rotation = qmul(b['rotation'],delta)
        tracks.append({'bone':i,'keys':[{'tick':0,'translation':b['translation'],'rotation':rotation}]})
    return {'id':name,'duration_ticks':120,'tracks':tracks}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    parser.add_argument('--config',type=Path,default=ROOT/'assets/characters/female-base/default.json')
    parser.add_argument('--base-only',action='store_true',help='publish the selected configuration and views without the regression poses/variants')
    args=parser.parse_args()
    OUTPUT.mkdir(parents=True,exist_ok=True)
    client=Client(args.binary,log_path=OUTPUT/'runtime.log')
    call=client.call
    report={'checks':[],'captures':[]}
    default=json.loads(args.config.read_text())
    placement={'prefix':'female','position':[0,.85,0],'rotation':[0,0,0,1],'scale':1}
    def publish(parameters=None):
        configuration=parameters or default
        # This harness owns its generated clips/pose probes. Reset that bake
        # explicitly before the next variant, using the same artist guard as UI.
        editor=call('inspect_character_editor')
        if editor['binding'] is not None and not editor['ready']:
            call('regenerate_character',prefix='female')
        result=call('publish_character',path='artifacts/character-base/female.ignis-character',
                    configuration=configuration,placement=placement)
        (OUTPUT/'publication.json').write_text(json.dumps({'configuration':configuration,'placement':placement,'report':result},indent=2)+'\n')
        return result
    def capture(name,position,target=(0,.85,0),fov=32):
        call('set_camera',camera={'position':position,'target':list(target),'fov_degrees':fov})
        result=call('capture',path=f'artifacts/character-base/{name}.png',width=1200,height=1200)
        report['captures'].append({'name':name,'capture':result})
    try:
        client.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'character-base','version':'1'}})
        client.request('notifications/initialized',notification=True)
        call('generate',recipe={'width':8,'depth':8,'seed':7,'obstacles':0})
        entities=call('inspect')['scene']['entities']
        for e in entities:
            e['visible']=False
        call('edit_scene',edits=[{'op':'upsert','entity':e} for e in entities])
        call('set_light',light={'direction':[.6,-.8,.7],'ambient':.25,'direct_intensity':1,
                              'environment':{'sky':[.5,.52,.56],'horizon':[.26,.28,.31],
                                             'ground':[.10,.09,.08],'axis':[0,1,0],'intensity':.6}})
        report['source']=call('inspect_character_source',path='artifacts/character-base/female.ignis-character')
        report['publication']=publish()
        original=call('inspect_meshes')
        invalid={'parameters':{'adiposity':2}}
        call('publish_character',path='artifacts/character-base/female.ignis-character',configuration=invalid,placement=placement,expected_error=True)
        assert call('inspect_meshes')==original
        report['checks'].append('invalid configuration leaves native document unchanged')
        for name, camera in [('front',[0,.88,-3.7]),('side',[3.7,.88,0]),('back',[0,.88,3.7]),('three-quarter',[-2.4,1.08,-3])]:
            capture(name,camera)
        capture('face',[.22,1.60,-.72],(0,1.56,-.06),34)
        capture('hand',[-.68,1.03,-.65],(-.45,.93,-.23),32)
        capture('feet',[.55,.22,-.65],(0,.09,-.05),38)
        call('save_scene',path='artifacts/character-base/female.scene.json')
        if args.base_only:
            print(json.dumps({'scene':str(OUTPUT/'female.scene.json'),'config':str(args.config),'publication':report['publication']}))
            return
        variants={'slender':{'adiposity':.2,'musculature':.35,'breast_volume':.35},
                  'athletic':{'adiposity':.3,'musculature':.9,'shoulder_width':.2},
                  'fuller':{'adiposity':.8,'musculature':.4,'hip_width':.15,'breast_volume':.7,'buttocks_volume':.2},
                  'tall':{'stature':.8,'upperleg_length':.15},
                  'face-variant':{'nose_width':.35,'jaw_width':-.3,'chin_projection':.2,'mouth_width':.2}}
        for name, values in variants.items():
            config={'parameters':{**default['parameters'],**values}}
            publish(config)
            capture(name,[0,.88,-3.7])
        publish()
        asset=call('inspect_mesh',asset='female_body')['asset']
        poses={
            'arms-up':{'upperarm01_L':([0,0,1],-120),'upperarm01_R':([0,0,1],120)},
            'elbows':{'lowerarm01_L':([1,0,0],90),'lowerarm01_R':([1,0,0],90)},
            'squat':{'upperleg01_L':([1,0,0],65),'upperleg01_R':([1,0,0],65),
                     'lowerleg01_L':([1,0,0],-110),'lowerleg01_R':([1,0,0],-110)},
            'fist':{f'finger{finger}-{phalanx}_L':([1,0,0],65) for finger in range(2,6) for phalanx in range(1,4)},
            'jaw':{'jaw':([1,0,0],-18)},
        }
        instances=call('inspect_meshes')['instances']
        for name, rotations in poses.items():
            clip=posed_clip(asset,rotations,'pose')
            assert len(clip['tracks'])==len(rotations),(name,rotations)
            # All parts share the same bone definition; ocular inserts follow head pose.
            for a in call('inspect_meshes')['assets']:
                call('edit_mesh',edit={'kind':'put_clip','asset':a['id'],'clip':clip})
            for instance in instances:
                call('edit_mesh',edit={'kind':'put_instance','instance':{**instance,'playback':{'clip':'pose','offset_ticks':0,'looping':False}}})
            if name=='fist':
                capture('pose-'+name,[-.68,1.03,-.65],(-.45,.93,-.23),32)
            elif name=='jaw':
                capture('pose-'+name,[.22,1.60,-.72],(0,1.56,-.06),34)
            else:
                capture('pose-'+name,[-2.4,1.08,-3])
        publish()
        # Exercise native animation/replay with real elapsed time, then leave the
        # editable default scene clean. Static bind-pose roundtrips are insufficient.
        probe=posed_clip(asset,poses['elbows'],'probe')
        for track in probe['tracks']:
            bone=asset['rig']['bones'][track['bone']]
            target={**track['keys'][0],'tick':60}
            rest={'tick':0,'translation':bone['translation'],'rotation':bone['rotation']}
            track['keys']=[rest,target,{**rest,'tick':120}]
        for a in call('inspect_meshes')['assets']:
            call('edit_mesh',edit={'kind':'put_clip','asset':a['id'],'clip':probe})
        for instance in instances:
            call('edit_mesh',edit={'kind':'put_instance','instance':{**instance,'playback':{'clip':'probe','offset_ticks':0,'looping':False}}})
        call('set_camera',camera={'position':[-2.4,1.08,-3],'target':[0,.85,0],'fov_degrees':32})
        call('step',ticks=30)
        animation=call('capture',path='artifacts/character-base/animation.png',width=1200,height=1200)
        report['captures'].append({'name':'animation','capture':animation})
        call('save_replay',path='artifacts/character-base/female.replay.json')
        replay=call('run_replay',path='artifacts/character-base/female.replay.json')
        assert replay['matched'] and replay['ticks']==30,replay
        call('capture_replay',replay_path='artifacts/character-base/female.replay.json',path='artifacts/character-base/animation-replay.png',tick=30,width=1200,height=1200)
        assert (OUTPUT/'animation.png').read_bytes()==(OUTPUT/'animation-replay.png').read_bytes()
        report['checks'].append('30 ticks of native weighted animation match replay and PNG exactly')
        report['replay']=replay
        # Loading a portable save creates a new replay baseline at absolute tick30;
        # validate the complete30-tick replay before replacing that baseline.
        call('save_portable_save',path='artifacts/character-base/female.ignis-portable')
        call('load_portable_save',path='artifacts/character-base/female.ignis-portable',mode='recorded')
        call('capture',path='artifacts/character-base/animation-restored.png',width=1200,height=1200)
        assert (OUTPUT/'animation.png').read_bytes()==(OUTPUT/'animation-restored.png').read_bytes()
        report['checks'].append('portable save restores full rig, overflow weights and animated pose with identical PNG')
        publish()
        capture('three-quarter',[-2.4,1.08,-3])
        call('save_scene',path='artifacts/character-base/female.scene.json')
        report['build']=call('inspect')['build_id']
        report['passed']=True
        (OUTPUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'passed':True,'output':str(OUTPUT),'publication':report['publication']}))
    finally:
        client.close()


if __name__=='__main__':
    main()
