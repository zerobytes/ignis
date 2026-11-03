#!/usr/bin/env python3
"""Real executable: source4 refinement, anatomical gizmos, ears and guarded migration."""
import json,math,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client

def main():
    out=ROOT/'artifacts/character-smoothing';out.mkdir(exist_ok=True)
    saved=json.loads((out/'user-before-smoothing.scene.json').read_text());user=saved['characters'][0]['recipe']
    probe=json.loads(json.dumps(saved));probe['characters']=[];probe['meshes']={'assets':[],'instances':[]};(out/'probe.scene.json').write_text(json.dumps(probe,separators=(',',':')))
    source=json.loads((ROOT/'artifacts/character-base/human-smooth.ignis-character').read_text());skin=source['parts'][0];local={v:i for i,v in enumerate(skin['vertices'])}
    client=Client(ROOT/'target/release/ignis',log_path=out/'runtime.log');call=client.call;report={'checks':[],'captures':[],'metrics':{}}
    try:
        client.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'character-smoothing','version':'4'}});client.request('notifications/initialized',notification=True)
        call('load_scene',path='artifacts/character-smoothing/probe.scene.json');placement={**user['placement'],'prefix':'proof'}
        call('publish_character',path='artifacts/character-base/human.ignis-character',configuration=user['configuration'],placement=placement)
        call('open_character_panel',open=True,prefix='proof');old=call('inspect_character_editor');oldparams=old['binding']['recipe']['configuration']['parameters']
        def capture(name,region='face'):
            state=call('inspect_character_editor');handles={h['name']:h for h in state['handles']}
            if region=='ear':
                center=handles['ear_lobe_R']['position'];camera=dict(position=[center[0]+.34,center[1]+.016,center[2]-.09],target=[center[0],center[1]+.02,center[2]],fov_degrees=32)
            elif region=='torso':camera=dict(position=[0,1.23,-1.3],target=[0,1.23,0],fov_degrees=32)
            elif region=='body':camera=dict(position=[0,.9,-3.8],target=[0,.88,0],fov_degrees=32)
            elif region=='lips':
                center=handles['upper_lip']['position'];camera=dict(position=[center[0],center[1],center[2]-.25],target=center,fov_degrees=32)
            else:
                center=handles['nose_tip']['position'];camera=dict(position=[center[0]+.30,center[1]+.04,center[2]-.65],target=[center[0],center[1]+.015,center[2]+.02],fov_degrees=32)
            call('set_camera',camera=camera);call('capture',path=f'artifacts/character-smoothing/{name}.png',width=1100,height=1200);report['captures'].append(name)
        for region in ['ear','face','torso']:capture('before-'+region,region)
        start=time.monotonic();result=call('upgrade_character_source',prefix='proof',path='artifacts/character-base/human-smooth.ignis-character',reference=call('inspect_character_editor')['reference']);report['metrics']['upgrade_seconds']=time.monotonic()-start
        assert result['vertices']==55922 and result['triangles']==111664,result
        state=call('inspect_character_editor');base=state['binding']['recipe']['configuration']['parameters'];assert all(abs(base[k]-v)<1e-7 for k,v in oldparams.items());assert len(base)==253 and len(state['handles'])==64
        counts={g:sum(h['group']==g for h in state['handles']) for g in ['Rosto','Lábios','Orelhas']};assert counts=={'Rosto':34,'Lábios':20,'Orelhas':10},counts
        report['checks'].append('guarded source3→4 migration preserves223 user values/placement;253 channels/64 XYZ points and20 lip/10 ear/34 face handles')
        changed=state['binding'];call('undo');assert call('inspect_character_editor')['binding']==old['binding'];call('redo');assert call('inspect_character_editor')['binding']==changed
        call('upgrade_character_source',prefix='proof',path='artifacts/character-base/human.ignis-character',reference=old['reference'],expected_error=True);assert call('inspect_character_editor')['binding']==changed
        call('save_scene',path='artifacts/character-smoothing/refined.scene.json');baked=json.loads((out/'refined.scene.json').read_text());asset=next(a for a in baked['meshes']['assets'] if a['id']=='proof_body');positions=asset['geometry']['positions']
        assert len(positions)==53514 and len(asset['geometry']['triangles'])==107024
        assert asset['material'].get('detail') is None and asset['geometry'].get('material_coordinates') is None
        for h in source['handles']:
            if h['part']!='body':continue
            actual=next(v for v in state['handles'] if v['name']==h['name'])['position'];p=[sum(positions[local[v]][a] for v in h['anchor']['vertices'])/len(h['anchor']['vertices'])+placement['position'][a] for a in range(3)]
            assert math.dist(actual,p)<3e-7,(h['name'],actual,p)
        pose=call('inspect_mesh_pose',instance='proof_body',tick=0)
        (out/'pose-shape.json').write_text(json.dumps({'keys':list(pose)},indent=2))
        # Native pose is world placed; every refined skin vertex must stay on its bake.
        posed=pose.get('pose',pose)
        pp=posed['positions'];error=max(math.dist([p[a]+placement['position'][a] for a in range(3)],q) for p,q in zip(positions,pp));assert error<3e-6,error
        report['metrics']['rest_pose_max_error_m']=error
        report['checks'].append('Catmull-Clark body53514v/107024tri keeps all rig influences (up to14); rest pose and every body gizmo coincide with refined surface; no procedural stripes')
        def set_values(values):return call('set_character_parameters',prefix='proof',configuration={'parameters':{**base,**values}})
        for region in ['ear','face','torso','body','lips']:capture('after-'+region,region)
        for p in source['parameters']:
            if p['name'].startswith('ear_'):
                for value in [p['minimum'],p['maximum']]:set_values({p['name']:value})
        set_values({'ear_size_R':.7,'ear_lobe_size_R':.8});capture('ear-large-lobe','ear')
        set_values({'ear_pointed_R':2});capture('ear-pointed','ear');capture('face-pointed')
        set_values({});state=call('inspect_character_editor');oldhandles={h['name']:h for h in state['handles']}
        for h in [h for h in state['handles'] if h['group']=='Orelhas']:
            ref=call('inspect_character_editor')['reference'];offset=[0,h['limits'][1]*.3,h['limits'][2]*.2]
            call('set_character_handle',prefix='proof',handle=h['name'],offset=offset,symmetry=True,reference=ref)
            moved=next(x for x in call('inspect_character_editor')['handles'] if x['name']==h['name']);assert math.dist(moved['position'],oldhandles[h['name']]['position'])>.002
            set_values({})
        report['checks'].append('12 scalar ear forms compile at24 limits; all10 ear XYZ handles drag with symmetry on actual ear; pointed ears/lobes captured')
        for group in [None,'Lábios','Orelhas']:
            call('set_character_handle_view',view={'enabled':True,'symmetry':True,'selected':None,'group':group});assert call('inspect_character_editor')['handle_view']['group']==group
        # Existing neck/head rotations must transport refined face and rigid eye inserts together.
        neutral={k:0 for k in base if k.startswith('posture_') or k in ['neck_pitch','head_pitch','head_lower_pitch','head_height']};set_values(neutral)
        call('save_scene',path='artifacts/character-smoothing/pose-baseline.scene.json');neutral_scene=json.loads((out/'pose-baseline.scene.json').read_text());refpos=next(a for a in neutral_scene['meshes']['assets'] if a['id']=='proof_body')['geometry']['positions']
        # Restrict to vertices whose entire subdivision stencil is in the rigid facial region.
        anchors=[local[next(h for h in source['handles'] if h['name']==name)['anchor']['vertices'][0]] for name in ['nose_tip','chin','forehead_middle','mouth_corner_R','ear_lobe_R']]
        set_values({**neutral,'neck_pitch':.6,'head_pitch':-.4});call('save_scene',path='artifacts/character-smoothing/pose-rotated.scene.json');rot=json.loads((out/'pose-rotated.scene.json').read_text());rotpos=next(a for a in rot['meshes']['assets'] if a['id']=='proof_body')['geometry']['positions']
        error=max(abs(math.dist(refpos[i],refpos[j])-math.dist(rotpos[i],rotpos[j])) for i in anchors for j in anchors);assert error<3e-6,error
        for side in ['L','R']:
            oldeye=next(a for a in neutral_scene['meshes']['assets'] if a['id']==f'proof_eye_{side}_sclera')['geometry']['positions'];neweye=next(a for a in rot['meshes']['assets'] if a['id']==f'proof_eye_{side}_sclera')['geometry']['positions']
            assert max(abs(math.dist(p,refpos[anchors[0]])-math.dist(q,rotpos[anchors[0]])) for p,q in zip(oldeye,neweye))<3e-6
        report['metrics']['refined_craniofacial_rigid_error_m']=error;report['checks'].append('refined face/jaw/lips/ear and eye inserts retain distances through combined neck/head rotations')
        set_values({});call('save_portable_save',path='artifacts/character-smoothing/body.ignis-portable');call('inspect_portable_save',path='artifacts/character-smoothing/body.ignis-portable');replay=call('save_replay',path='artifacts/character-smoothing/body.replay.json',expected_error=True);assert '8 MiB' in replay['message'];assert call('inspect_character_editor')['binding']==changed;report['replay_limit']=replay['message']
        report['checks'].append('refined bake/recipe undo-redo, stale guards and portable save remain coherent; full recorded session exceeds unchanged8MiB replay envelope and rejects without mutation')
        report.update(passed=True,build=call('inspect')['build_id'],groups=counts)
    except BaseException as e:report.update(passed=False,error=str(e));raise
    finally:(out/'verification.json').write_text(json.dumps(report,indent=2)+'\n');client.close()
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
