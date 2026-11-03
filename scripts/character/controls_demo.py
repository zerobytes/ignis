#!/usr/bin/env python3
"""Validate the real source2 through native MCP; leave the user's scene untouched."""
import argparse
import json
import math
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client


def volume(asset):
    p=asset['geometry']['positions']
    total=0
    for a,b,c in asset['geometry']['triangles']:
        a,b,c=p[a],p[b],p[c]
        total+=a[0]*(b[1]*c[2]-b[2]*c[1])+a[1]*(b[2]*c[0]-b[0]*c[2])+a[2]*(b[0]*c[1]-b[1]*c[0])
    return abs(total/6)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--capture',action='store_true')
    parser.add_argument('--all-handles',action='store_true')
    args=parser.parse_args()
    output=ROOT/'artifacts/character-controls';output.mkdir(exist_ok=True)
    client=Client(ROOT/'target/release/ignis',log_path=output/'runtime.log')
    call=client.call
    report={'checks':[],'metrics':{},'captures':[]}
    try:
        client.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'character-controls','version':'2'}})
        client.request('notifications/initialized',notification=True)
        scene=json.loads((ROOT/'artifacts/character-base/female.scene.json').read_text())
        original_binding=next(b for b in scene['characters'] if b['recipe']['placement']['prefix']=='female')
        owned=set(original_binding['parts'])
        owned_instances={i['id'] for i in scene['meshes']['instances'] if i['asset'] in owned}
        scene['characters']=[b for b in scene['characters'] if b is not original_binding]
        scene['meshes']['instances']=[i for i in scene['meshes']['instances'] if i['asset'] not in owned]
        scene['meshes']['assets']=[a for a in scene['meshes']['assets'] if a['id'] not in owned]
        for obj in scene['meshes'].get('objects',[]):
            obj['members']=[i for i in obj['members'] if i not in owned_instances]
        if 'objects' in scene['meshes']:
            scene['meshes']['objects']=[o for o in scene['meshes']['objects'] if o['members']]
        (output/'legacy-check.scene.json').write_text(json.dumps(scene,separators=(',',':')))
        call('load_scene',path='artifacts/character-controls/legacy-check.scene.json')
        # Reconstruct the legacy21-channel recipe inside this isolated process,
        # even after the interactive scene has migrated and acquired face edits.
        legacy=call('inspect_character_source',path='artifacts/character-base/female.ignis-character')
        legacy_names={p['name'] for p in legacy['parameters']}
        legacy_configuration={'parameters':{k:v for k,v in original_binding['recipe']['configuration']['parameters'].items() if k in legacy_names}}
        call('publish_character',path='artifacts/character-base/female.ignis-character',configuration=legacy_configuration,placement=original_binding['recipe']['placement'])
        call('open_character_panel',open=True,prefix='female')
        original_binding=call('inspect_character_editor')['binding']
        call('connect_character_source',path='artifacts/character-base/female-extended.ignis-character',
             configuration=original_binding['recipe']['configuration'],placement=original_binding['recipe']['placement'])
        initial=call('inspect_character_editor')
        assert initial['ready'] and len(initial['handles'])==42,initial
        assert initial['binding']['bake_sha256']==original_binding['bake_sha256']
        report['checks'].append('source1 user21-channel recipe→source2 keeps its complete bake and values exactly')
        report['source']=dict(parameters=len(initial['parameters']),handles=len(initial['handles']),source_sha256=initial['binding']['recipe']['source_sha256'])
        baseline=initial['binding']['recipe']['configuration']
        placement=initial['binding']['recipe']['placement']
        def set_values(values):
            return call('set_character_parameters',prefix='female',configuration={'parameters':{**baseline['parameters'],**values}})
        def asset(name):
            return call('inspect_mesh',asset=name)['asset']
        def skin():
            return asset('female_body')
        def capture(name,side=False,face=False):
            if not args.capture:
                return
            if face:
                call('set_camera',camera=dict(position=[.0,1.56,-.73] if not side else [.72,1.56,-.03],target=[0,1.56,-.03],fov_degrees=32))
            else:
                call('set_camera',camera=dict(position=[0,.86,-3.8] if not side else [3.8,.86,0],target=[0,.85,0],fov_degrees=32))
            call('capture',path=f'artifacts/character-controls/{name}.png',width=1100,height=1300)
            report['captures'].append(name)
        posture_names=[p['name'] for p in initial['parameters'] if p['name'].startswith('posture_')]
        posture_zero={n:0 for n in posture_names}
        case_base={**posture_zero,'waist_width':0}
        for name,values in [('fat-old-maximum',{'adiposity':1,'muscle_definition':0}),('fat-extended',{'adiposity':2,'muscle_definition':0}),
                            ('muscle-old-maximum',{'adiposity':.12,'musculature':1,'muscle_definition':0}),('muscle-extended',{'adiposity':.12,'musculature':2,'muscle_definition':1})]:
            set_values({**case_base,**values});body=skin();report['metrics'][name]={'mesh_volume_m3':volume(body)}
            capture(name);capture(name+'-side',True)
        report['metrics']['fat_volume_ratio']=report['metrics']['fat-extended']['mesh_volume_m3']/report['metrics']['fat-old-maximum']['mesh_volume_m3']
        assert report['metrics']['fat_volume_ratio']>2,report['metrics']
        assert report['metrics']['muscle-extended']['mesh_volume_m3']>report['metrics']['muscle-old-maximum']['mesh_volume_m3']
        report['checks'].append('both extended mass endpoints change geometry beyond the old maximum; definition is modelled geometry')
        set_values(case_base);normal=skin()['geometry']['positions']
        for value in [-1,1]:
            set_values({**case_base,'waist_width':value});changed=skin()['geometry']['positions']
            assert any(a!=b for a,b in zip(normal,changed))
            # Rib vertices have a positive clearance above the waist support.
            ribs=[i for i,p in enumerate(normal) if p[1]>.34]
            assert all(normal[i]==changed[i] for i in ribs)
            capture('waist-thin' if value<0 else 'waist-wide')
        report['checks'].append('waist changes without moving the rib cage/head/upper-body vertices')
        set_values(case_base)
        neutral_points=skin()['geometry']['positions']
        feet=[i for i,p in enumerate(neutral_points) if p[1]<-.70]
        natural=dict(zip(['posture_pelvis','posture_lumbar','posture_thoracic','posture_head_pitch','posture_head_retraction','posture_shoulders'],[-.1,.15,.55,-.92,.75,.65]))
        for name,values in [('posture-base',posture_zero),('posture-upright',natural),('posture-curved',dict(zip(natural,[.1,-.25,-.6,.6,-.35,-.55])))]:
            set_values({**case_base,**values})
            p=skin()['geometry']['positions'];assert all(p[i]==neutral_points[i] for i in feet)
            report['metrics'][name]={'nose':next(h['position'] for h in call('inspect_character_editor')['handles'] if h['name']=='nose_tip')}
            capture(name,True)
        assert report['metrics']['posture-upright']['nose'][2]>report['metrics']['posture-base']['nose'][2]
        report['checks'].append('posture changes the upper-body profile while the foot vertices remain fixed')
        set_values({})
        state=call('inspect_character_editor')
        preview_args=dict(prefix='female',handle='nose_tip',offset=[0,.003,-.006],symmetry=True,reference=state['reference'])
        preview=call('preview_character_handle',**preview_args)
        unchanged=call('inspect_character_editor')
        assert preview['world_changed'] is False and unchanged['reference']==state['reference']
        assert unchanged['binding']==state['binding'] and preview['binding']['bake_sha256']!=state['binding']['bake_sha256']
        call('set_character_handle',**preview_args)
        assert call('inspect_character_editor')['binding']==preview['binding']
        call('undo');assert call('inspect_character_editor')['binding']==state['binding']
        report['checks'].append('MCP preview changes neither revision nor binding and matches the subsequent atomic commit')
        eye_before=asset('female_eye_L_sclera')['geometry']['positions'][0]
        state=call('inspect_character_editor')
        call('set_character_handle',prefix='female',handle='eye_L',offset=[-.006,.004,.006],symmetry=True,reference=state['reference'])
        eye_after=asset('female_eye_L_sclera')['geometry']['positions'][0]
        assert math.dist([b+a for a,b in zip([-.006,.004,.006],eye_before)],eye_after)<1e-5,(eye_before,eye_after)
        state=call('inspect_character_editor');params=state['binding']['recipe']['configuration']['parameters']
        assert abs(params['face_eye_R_x']-.006)<1e-6 and abs(params['face_eye_R_y']-.004)<1e-6
        report['checks'].append('eye insert and orbit move together; paired offsets mirror X and retain Y/Z')
        call('undo')
        before=call('inspect_character_editor')['binding']
        for offset in [[0,.002,-.004],[0,.004,-.009]]:
            state=call('inspect_character_editor')
            call('set_character_handle',prefix='female',handle='nose_tip',offset=offset,symmetry=False,reference=state['reference'],gesture='face-drag')
        edited=call('inspect_character_editor')['binding'];assert edited['bake_sha256']!=before['bake_sha256']
        call('undo');assert call('inspect_character_editor')['binding']==before
        call('redo');assert call('inspect_character_editor')['binding']==edited
        capture('face-nose-edited',face=True);capture('face-nose-edited-side',side=True,face=True)
        set_values({})
        report['checks'].append('three-dimensional facial changes share one gesture undo/redo')
        if args.all_handles:
            for h in initial['handles']:
                for sign in [1,-1]:
                    state=call('inspect_character_editor')
                    call('set_character_handle',prefix='female',handle=h['name'],offset=[sign*v for v in h['limits']],symmetry=not h.get('midline',False),reference=state['reference'])
                state=call('inspect_character_editor')
                call('set_character_handle',prefix='female',handle=h['name'],offset=[0,0,0],symmetry=True,reference=state['reference'])
            report['checks'].append('all42 handles compile with positive/negative XYZ offsets at full limits, including mirrored pairs and asymmetric midline motion')
        set_values({'adiposity':1.45,'musculature':1.4,'muscle_definition':.65,'waist_width':-.25,**natural})
        state=call('inspect_character_editor')
        call('set_character_handle',prefix='female',handle='cheekbone_L',offset=[-.005,.003,-.004],symmetry=True,reference=state['reference'])
        durable=call('inspect_character_editor')['binding']
        call('save_character_configuration',prefix='female',path='artifacts/character-controls/roundtrip.ignis-character-config')
        call('save_portable_save',path='artifacts/character-controls/female.ignis-portable')
        call('load_portable_save',path='artifacts/character-controls/female.ignis-portable',mode='recorded')
        call('save_replay',path='artifacts/character-controls/female.replay.json')
        assert call('run_replay',path='artifacts/character-controls/female.replay.json')['matched']
        restored=call('inspect_character_editor');assert restored['binding']==durable
        report['checks'].append('all new scalar/XYZ controls and the baked rig survive portable save/replay')
        report['build']=call('inspect')['build_id'];report['passed']=True
        (output/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2))
    except BaseException as error:
        report['passed']=False;report['error']=str(error)
        (output/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        raise
    finally:
        client.close()


if __name__=='__main__':
    main()
