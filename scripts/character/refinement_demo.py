#!/usr/bin/env python3
"""Native MCP regression for lips, adult male anatomy, rigid pivots and Source3 relief."""
import argparse
import json
import math
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--capture',action='store_true');args=parser.parse_args()
    out=ROOT/'artifacts/character-refinement';out.mkdir(exist_ok=True)
    saved=json.loads((ROOT/'artifacts/character-base/female.scene.json').read_text())
    user=saved['characters'][0]['recipe'];scene=json.loads(json.dumps(saved));scene['characters']=[];scene['meshes']={'assets':[],'instances':[]}
    (out/'probe.scene.json').write_text(json.dumps(scene,separators=(',',':')))
    source=json.loads((ROOT/'artifacts/character-base/human.ignis-character').read_text())
    skin=next(p for p in source['parts'] if p['name']=='body');source_ids={v:i for i,v in enumerate(skin['vertices'])}
    protected=json.loads((out/'source.json').read_text())['protected_head_vertices'];head=[source_ids[v] for v in protected]
    client=Client(ROOT/'target/release/ignis',log_path=out/'runtime.log');call=client.call
    report={'checks':[],'captures':[],'metrics':{}}
    try:
        client.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'character-refinement','version':'3'}})
        client.request('notifications/initialized',notification=True)
        call('load_scene',path='artifacts/character-refinement/probe.scene.json')
        placement={**user['placement'],'prefix':'proof'}
        legacy={**user['configuration']['parameters'],'muscle_definition':0}
        # This test stays reproducible after the interactive scene migrates.
        original_source=json.loads((ROOT/'artifacts/character-base/female-extended.ignis-character').read_text())
        legacy_names={p['name'] for p in original_source['parameters']};legacy={k:v for k,v in legacy.items() if k in legacy_names}
        call('publish_character',path='artifacts/character-base/female-extended.ignis-character',configuration={'parameters':legacy},placement=placement)
        call('open_character_panel',open=True,prefix='proof');old=call('inspect_character_editor')['binding']
        call('connect_character_source',path='artifacts/character-base/human.ignis-character',configuration={'parameters':legacy},placement=placement)
        initial=call('inspect_character_editor');assert initial['ready'] and len(initial['handles'])==58
        assert initial['binding']['bake_sha256']==old['bake_sha256']
        baseline=initial['binding']['recipe']['configuration']['parameters']
        report['checks'].append('source2→3 keeps existing spine, shoulder and upper-head shape fields and their bake exactly')
        def set_values(values):return call('set_character_parameters',prefix='proof',configuration={'parameters':{**baseline,**values}})
        def body():return call('inspect_mesh',asset='proof_body')['asset']
        def capture(name,side=False,face=False,lips=False):
            if not args.capture:return
            if face or lips:
                center=[0,1.45 if lips else 1.49,-.10 if lips else 0]
                camera=dict(position=[1.5,1.52,0] if side else [0,center[1],-.36 if lips else -.78],target=center,fov_degrees=32)
            else:camera=dict(position=[3.8,.9,0] if side else [0,.9,-3.8],target=[0,.88,0],fov_degrees=32)
            call('set_camera',camera=camera);call('capture',path=f'artifacts/character-refinement/{name}.png',width=1000,height=1200);report['captures'].append(name)
        neutral={k:0 for k in baseline if k.startswith('posture_') or k in ['neck_pitch','head_pitch','head_lower_pitch','head_height']}
        set_values(neutral);reference=body()['geometry']['positions'];reference_handles=call('inspect_character_editor')['handles']
        eye_reference={side:call('inspect_mesh',asset=f'proof_eye_{side}_sclera')['asset']['geometry']['positions'] for side in ['L','R']}
        anchors=[head[0],head[len(head)//2],head[-1]];max_error=0
        for name,values in [('neck-forward',{'neck_pitch':-1}),('neck-back',{'neck_pitch':1}),('head-down',{'head_pitch':-1}),('head-up',{'head_pitch':1}),('neck-head-combined',{'neck_pitch':.6,'head_pitch':-.4})]:
            set_values({**neutral,**values});changed=body()['geometry']['positions']
            error=max(abs(math.dist(reference[i],reference[a])-math.dist(changed[i],changed[a])) for i in head for a in anchors)
            max_error=max(max_error,error);assert error<3e-6,(name,error)
            for side,old_eye in eye_reference.items():
                eye=call('inspect_mesh',asset=f'proof_eye_{side}_sclera')['asset']['geometry']['positions']
                eye_error=max(abs(math.dist(p,reference[a])-math.dist(q,changed[a])) for p,q in zip(old_eye,eye) for a in anchors)
                assert eye_error<3e-6,(name,side,eye_error)
            assert max(math.dist(reference[i],changed[i]) for i in head)>.02
            capture(name,side=True)
        report['metrics']['new_pivots_max_head_distance_error_m']=max_error
        stylized={**neutral,'posture_head_pitch':.6,'posture_head_retraction':-.4,'posture_pelvis':-.53,'posture_lumbar':.69,'posture_thoracic':-.15,'posture_shoulders':.44}
        set_values(stylized);formed=body()['geometry']['positions']
        set_values({**stylized,'neck_pitch':.5,'head_pitch':-.3});rotated=body()['geometry']['positions']
        error=max(abs(math.dist(formed[i],formed[a])-math.dist(rotated[i],rotated[a])) for i in head for a in anchors)
        assert error<3e-6,error
        report['metrics']['formed_head_rigid_error_m']=error
        report['checks'].append('new neck/head pivots move the entire craniofacial region rigidly at both limits and combined')
        for name in ['posture_head_pitch','posture_head_retraction','head_lower_pitch','head_height']:
            for value in [-1,1]:
                set_values({**neutral,name:value});changed=body()['geometry']['positions']
                assert any(reference[i]!=changed[i] for i in head)
                capture(name+('-min' if value<0 else '-max'),side=True,face=True)
        report['checks'].append('old upper-head forms remain active; new lower-head inclination and wide height range compile at both limits')
        # Reproduce the user's successful posture without introducing cranial morphs.
        set_values({**neutral,'posture_lumbar':.69,'posture_pelvis':-.53,'posture_shoulders':.44,'posture_thoracic':-.15})
        capture('heroic-user-profile',side=True);capture('heroic-user')
        report['checks'].append('heroic preset uses the human-selected spine/shoulder values; upper-head shapes stay zero')
        lips=[p for p in source['parameters'] if p['name'].startswith('lip_')]
        assert len(lips)==14
        for p in lips:
            for value in [-1,1]:
                set_values({**neutral,p['name']:value})
                assert body()['geometry']['positions']!=reference
        for name,values in [('lips-thin',{'lip_upper_height':-.8,'lip_lower_height':-.8,'lip_upper_volume':-.6,'lip_lower_volume':-.6}),('lips-full',{'lip_upper_height':.65,'lip_lower_height':.65,'lip_upper_volume':.8,'lip_lower_volume':.8}),('lips-cupid',{'lip_cupid_shape':.8,'lip_cupid_width':.6,'lip_upper_middle':.25})]:
            set_values({**neutral,**values});capture(name,lips=True)
        set_values(neutral)
        state=call('inspect_character_editor')
        call('set_character_handle_view',view={'enabled':True,'symmetry':True,'selected':'cupid_peak_L','group':'Lábios'})
        assert call('inspect_character_editor')['handle_view']['group']=='Lábios'
        call('set_character_handle_view',view={'enabled':True,'symmetry':True,'selected':None,'group':'missing'},expected_error=True)
        for h in initial['handles'][42:]:
            for sign in [-1,1]:
                state=call('inspect_character_editor')
                call('set_character_handle',prefix='proof',handle=h['name'],offset=[sign*v for v in h['limits']],symmetry=not h['midline'],reference=state['reference'])
            state=call('inspect_character_editor');call('set_character_handle',prefix='proof',handle=h['name'],offset=[0,0,0],symmetry=True,reference=state['reference'])
        report['checks'].append('14 lip shape controls/28 endpoints and16 new XYZ handles/32 bounded endpoints, symmetry and lip-only group are usable')
        defaults={p['name']:p['default'] for p in source['parameters']}
        for sex,text in [(0,'female'),(1,'male')]:
            for value in [0,1]:
                values={**defaults,**neutral,'body_masculinity':sex,'adiposity':.12,'musculature':1.4,'muscle_definition':value,'muscle_striation':1}
                set_values(values);mesh=body()
                assert (mesh['material'].get('detail') is not None)==bool(value)
                assert (mesh['geometry'].get('material_coordinates') is not None)==bool(value)
                capture(text+('-defined' if value else '-smooth'));capture(text+('-defined-side' if value else '-smooth-side'),side=True)
                if value:call('save_character_configuration',prefix='proof',path=f'artifacts/character-refinement/{text}.ignis-character-config')
        report['checks'].append('male/female anatomy and mass/definition compile; fine relief switches existing native material and rest coordinates together')
        set_values({**defaults,**neutral,'body_masculinity':1,'adiposity':.12,'musculature':2,'muscle_definition':1,'muscle_striation':1})
        capture('male-extreme')
        if args.capture:
            set_values({**defaults,**neutral,'body_masculinity':1,'adiposity':.12,'musculature':1.4,'muscle_definition':1,'muscle_striation':1})
            call('set_camera',camera=dict(position=[0,1.17,-.95],target=[0,1.17,0],fov_degrees=32))
            call('capture',path='artifacts/character-refinement/male-muscle-detail.png',width=1200,height=1200)
            report['captures'].append('male-muscle-detail')
        set_values({**neutral,'body_masculinity':1,'muscle_definition':1,'muscle_striation':.9,'neck_pitch':.2,'head_pitch':-.2,'lip_upper_volume':.4})
        durable=call('inspect_character_editor')['binding']
        set_values(neutral);call('undo');assert call('inspect_character_editor')['binding']==durable
        call('save_portable_save',path='artifacts/character-refinement/body.ignis-portable')
        call('load_portable_save',path='artifacts/character-refinement/body.ignis-portable',mode='recorded')
        call('save_replay',path='artifacts/character-refinement/body.replay.json')
        assert call('run_replay',path='artifacts/character-refinement/body.replay.json')['matched']
        assert call('inspect_character_editor')['binding']==durable
        report['checks'].append('source3 body/forms/relief/coordinates and recipe remain atomic through undo, portable save and replay')
        report.update(passed=True,build=call('inspect')['build_id'],source=source['id'],parameters=len(source['parameters']),handles=len(source['handles']))
    except BaseException as e:report.update(passed=False,error=str(e));raise
    finally:
        (out/'verification.json').write_text(json.dumps(report,indent=2)+'\n');client.close()
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
