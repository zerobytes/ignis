#!/usr/bin/env python3
"""Native character authoring check for an independently editable Cupid valley."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from verify_lab import Client

def main():
    out=ROOT/'artifacts/character-cupid';out.mkdir(exist_ok=True)
    c=Client(ROOT/'target/release/ignis',log_path=out/'runtime.log');call=c.call
    report={'checks':[],'captures':[],'metrics':{}}
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'character-cupid','version':'5'}});c.request('notifications/initialized',notification=True)
        call('load_scene',path='artifacts/character-cupid/user-before-cupid.scene.json');call('open_character_panel',open=True,prefix='female');old=call('inspect_character_editor');assert old['ready']
        result=call('upgrade_character_source',prefix='female',path='artifacts/character-base/human-cupid.ignis-character',reference=old['reference']);state=call('inspect_character_editor');binding=state['binding'];assert binding['bake_sha256']==old['binding']['bake_sha256']
        values=binding['recipe']['configuration']['parameters'];assert len(values)==256 and len(state['handles'])==65
        assert all(values[k]==v for k,v in old['binding']['recipe']['configuration']['parameters'].items())
        assert sum(h['group']=='Lábios' for h in state['handles'])==21
        report['checks'].append('source4→5 at zero preserves the entire current user bake bit-exactly and all253 values; one new XYZ landmark/65points/256channels')
        call('undo');assert call('inspect_character_editor')['binding']==old['binding'];call('redo');assert call('inspect_character_editor')['binding']==binding
        base_handles={h['name']:h for h in call('inspect_character_editor')['handles']}
        def set_handle(offset,preview=False,symmetry=True):
            ref=call('inspect_character_editor')['reference'];return call('preview_character_handle' if preview else 'set_character_handle',prefix='female',handle='cupid_valley',offset=offset,symmetry=symmetry,reference=ref)
        def capture(name,side=False):
            center=base_handles['cupid_valley']['position'];camera=dict(position=[center[0]+.26,center[1]+.005,center[2]-.035] if side else [center[0],center[1],center[2]-.21],target=[center[0],center[1]-.003,center[2]],fov_degrees=32)
            call('set_camera',camera=camera);call('capture',path=f'artifacts/character-cupid/{name}.png',width=1100,height=900);report['captures'].append(name)
        for side in [False,True]:capture('current'+('-profile' if side else '-front'),side)
        for offset in [[0,-.0012,.0018],[0,0,-.004]]:
            set_handle(offset);state=call('inspect_character_editor');h=next(h for h in state['handles'] if h['name']=='cupid_valley');base=base_handles['cupid_valley'];delta=[h['position'][a]-base['position'][a] for a in range(3)]
            # Source offsets are metres before the retained anatomical warps.
            # The rig-derived drawing frame also contains upper-cranium shape:
            # it is not an exact derivative of this local lip field.
            projected=[sum(delta[k]*base['basis'][a][k] for k in range(3)) for a in range(3)]
            length=math.sqrt(sum(v*v for v in offset));actual=math.sqrt(sum(v*v for v in delta));error=abs(actual-length);assert error<3e-7,(error,projected,offset)
            cosine=sum(projected[a]*offset[a] for a in range(3))/(length*actual)
            angle=math.degrees(math.acos(max(-1,min(1,cosine))));assert angle<1,angle
            report['metrics']['max_rig_frame_axis_angle_degrees']=max(angle,report['metrics'].get('max_rig_frame_axis_angle_degrees',0))
            for name in ['lower_lip','lower_lip_lobe_L','lower_lip_lobe_R','mouth_corner_L','mouth_corner_R','nose_tip','chin','ear_lobe_R']:
                now=next(x for x in state['handles'] if x['name']==name);assert math.dist(now['position'],base_handles[name]['position'])<3e-7,name
            for name in ['cupid_peak_L','cupid_peak_R','upper_lip']:
                now=next(x for x in state['handles'] if x['name']==name);movement=math.dist(now['position'],base_handles[name]['position']);assert movement<.001,(name,movement);report['metrics'][name+'_max_movement_m']=max(movement,report['metrics'].get(name+'_max_movement_m',0))
            report['metrics']['handle_max_distance_error_m']=max(error,report['metrics'].get('handle_max_distance_error_m',0))
            text='inset-defined' if offset[2]>0 else 'projected-soft'
            for side in [False,True]:capture(text+('-profile' if side else '-front'),side)
            set_handle([0,0,0])
        report['checks'].append('central valley moves independently in depth/height; lower lip, corners/nose/chin/ears unchanged; both peaks and upper vermilion center protected; front/profile images')
        limits=base_handles['cupid_valley']['limits']
        for axis in range(3):
            for sign in [-1,1]:
                offset=[0,0,0];offset[axis]=sign*limits[axis];set_handle(offset)
                got=next(h for h in call('inspect_character_editor')['handles'] if h['name']=='cupid_valley')['offset'];assert (axis==0 and got[0]==0) or abs(got[axis]-offset[axis])<1e-8
        set_handle([.002,0,0],symmetry=False);assert abs(next(h for h in call('inspect_character_editor')['handles'] if h['name']=='cupid_valley')['offset'][0]-.002)<1e-8
        set_handle([0,0,0]);before=call('inspect_character_editor')['binding'];set_handle([0,-.001,.004],preview=True);assert call('inspect_character_editor')['binding']==before
        state=call('inspect_character_editor');call('set_character_handle',prefix='female',handle='cupid_valley',offset=[0,0,.011],symmetry=True,reference=state['reference'],expected_error=True);assert call('inspect_character_editor')['binding']==before
        call('set_character_handle',prefix='female',handle='cupid_valley',offset=[0,0,0],symmetry=True,reference=old['reference'],expected_error=True)
        call('set_character_handle_view',view={'enabled':True,'symmetry':True,'selected':'cupid_valley','group':'Lábios'});assert call('inspect_character_editor')['handle_view']['selected']=='cupid_valley'
        report['checks'].append('six individual limits, midline symmetry, disposable preview, stale/out-of-range rejection and native selector share the existing guarded command path')
        set_handle([0,-.0012,.0018]);durable=call('inspect_character_editor')['binding'];call('save_scene',path='artifacts/character-cupid/inset.scene.json');call('save_character_configuration',prefix='female',path='artifacts/character-cupid/inset.ignis-character-config');call('save_portable_save',path='artifacts/character-cupid/body.ignis-portable');call('load_portable_save',path='artifacts/character-cupid/body.ignis-portable',mode='recorded');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==durable
        call('load_scene',path='artifacts/character-cupid/inset.scene.json');call('open_character_panel',open=True,prefix='female');assert call('inspect_character_editor')['binding']==durable
        report['checks'].append('new local channels and exact source identity survive configuration/scene/portable save and reopen; legacy sources remain intact')
        report.update(passed=True,build=call('inspect')['build_id'],source_sha256=binding['recipe']['source_sha256'])
    except BaseException as e:report.update(passed=False,error=str(e));raise
    finally:(out/'verification.json').write_text(json.dumps(report,indent=2)+'\n');c.close()
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
