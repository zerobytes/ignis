#!/usr/bin/env python3
"""Pure bounded volume junctions; exact retained data, native poses and pixels."""
import argparse,copy,json,hashlib
from pathlib import Path
from verify_lab import Client,ROOT

CAMS={'front':{'position':[3.5,3.2,4.7],'target':[.55,2.95,.13],'fov_degrees':30},
      'profile':{'position':[4,3.5,-1.4],'target':[.4,2.95,.12],'fov_degrees':30},
      'back':{'position':[-3.5,3.5,-4.7],'target':[.35,2.95,.13],'fov_degrees':30}}

def verify(binary,window):
    c=Client(binary,window);call=c.call;checks=[]
    def capture(name):
        meta=call('capture',path=f'artifacts/union-{name}.png',width=1000,height=1000)
        data=(ROOT/f'artifacts/union-{name}.png').read_bytes()
        return data,{'sha256':hashlib.sha256(data).hexdigest(),'triangles':meta['mesh_triangle_count']}
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-union-verifier','version':'1'}});c.request('notifications/initialized',notification=True)
        assert 'preview_mesh_junction_union' in json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        fixture=json.loads((ROOT/'examples/mesh-junction-union.input.json').read_text());source=fixture['source'];id=source['id'];n=len(source['geometry']['positions']);t=len(source['geometry']['triangles'])
        call('generate',recipe={'seed':7,'width':20,'depth':20,'obstacles':0})
        # Compare against the engine's canonical f32 serialization, rather than
        # Python's longer decimal spelling of the same input f32 values.
        call('edit_mesh',edit={'kind':'put_asset','asset':source})
        source=call('inspect_mesh',asset=id)['asset'];fixture['source']=source
        call('step',ticks=9);state=call('inspect')
        result=call('preview_mesh_junction_union',**fixture);assert call('inspect')==state
        assert call('preview_mesh_junction_union',**fixture)==result and call('inspect')==state
        out=result['asset'];report=result['report'];assert len(out['geometry']['positions'])==3995 and len(out['geometry']['triangles'])==7986
        assert report['max_bend_degrees']<60 and report['max_field_error']<1e-6
        for channel in ['positions','material_coordinates']:assert out['geometry'][channel][:n]==source['geometry'][channel]
        assert out['geometry']['triangles'][:t]==source['geometry']['triangles']
        assert out['rig']['weights'][:n]==source['rig']['weights'] and out['rig']['bones']==source['rig']['bones']
        assert out['clips']==source['clips'] and out['material']==source['material'] and out['shading']==source['shading']
        failures=[]
        for case in range(8):
            bad=copy.deepcopy(fixture)
            if case==0:bad['ports'][0].reverse()
            elif case==1:bad['ports'][1]=bad['ports'][0]
            elif case==2:bad['options']['hub']=[100,100,100]
            elif case==3:bad['options']['cell_size']=.001
            elif case==4:bad['options']['collar_length']=0.
            elif case==5:bad['root']=None
            elif case==6:bad['source']['rig']['weights'][bad['ports'][1][0]]={'joints':[0,1,0,0],'weights':[.5,.5,0,0]}
            else:bad['options']['relaxation_steps']=9
            failures.append(call('preview_mesh_junction_union',expected_error=True,**bad));assert call('inspect')==state
        checks.append('Pure deterministic dispatcher/schema query, exact retained points/triangles/domain/weights/bones/clips/material and atomic winding/root/options/grid/collar refusals')
        baseline=result
        variants={}
        for steps in [1,2,4]:
            candidate=copy.deepcopy(fixture)
            candidate['options'].update(seam_relaxation_steps=steps,seam_max_displacement=.025,material_relaxation_steps=64)
            result=call('preview_mesh_junction_union',**candidate);assert call('inspect')==state
            assert call('preview_mesh_junction_union',**candidate)==result
            asset=result['asset'];rep=result['report'];iso=n+rep['extracted_vertices']
            assert asset['geometry']['positions'][:n]==source['geometry']['positions']
            assert asset['geometry']['positions'][iso:]==baseline['asset']['geometry']['positions'][iso:]
            assert asset['geometry']['triangles']==baseline['asset']['geometry']['triangles']
            assert asset['rig']==baseline['asset']['rig'] and asset['clips']==source['clips']
            coords=asset['geometry']['material_coordinates'];assert coords[:n]==source['geometry']['material_coordinates']
            guard=iso
            for lp in fixture['ports']:
                for src in reversed(lp):assert coords[guard]==coords[src];guard+=1
            assert rep['max_bend_degrees']<baseline['report']['max_bend_degrees'] and rep['mean_bend_degrees']<baseline['report']['mean_bend_degrees']
            assert rep['source_seam_max_degrees']==baseline['report']['source_seam_max_degrees']
            assert rep['refinement']['max_seam_displacement']<=.025001
            assert rep['refinement']['material_guard_jump_after']==0 and rep['refinement']['material_residual_max']<1e-4
            assert rep['max_field_error_after_seam']>rep['max_field_error']
            variants[str(steps)]=rep
        for case in range(7):
            bad=copy.deepcopy(fixture)
            if case==0:bad['options']['seam_relaxation_steps']=9
            elif case==1:bad['options']['seam_relaxation_steps']=1
            elif case==2:bad['options']['seam_max_displacement']=.04
            elif case==3:bad['options']['material_relaxation_steps']=65
            elif case==4:
                bad['options']['material_relaxation_steps']=1;bad['source']['geometry']['material_coordinates']=None
            elif case==5:bad['options'].update(seam_relaxation_steps=8,seam_max_displacement=.025)
            else:bad['options']['seam_max_displacement']=-.01
            failures.append(call('preview_mesh_junction_union',expected_error=True,**bad));assert call('inspect')==state
        out=result['asset'];report=result['report']
        assert report['max_bend_degrees']<44 and report['mean_bend_degrees']<5
        checks.append('Finite1/2/4 seam refinement improves maximum/mean with exact source/first guard, bounded displacement and unchanged geometry count; material guard continuity, quantized residual and seven atomic refusals including worsening8')
        call('edit_mesh',edit={'kind':'put_asset','asset':source})
        instance={'id':'junction-proof','asset':id,'position':fixture['options']['material_offset'],'rotation':[0,0,0,1],'scale':1,'color':[.45,.4,.32],
                  'playback':{'clip':'secondary-breeze','offset_ticks':0,'looping':True}}
        call('edit_mesh',edit={'kind':'put_instance','instance':instance})
        marker={'id':'socket-marker','geometry':{'positions':[[0,0,0],[.6,0,0],[0,.6,.15]],'triangles':[[0,1,2]]}}
        child={'id':'socket-marker','asset':'socket-marker','position':[0,0,.3],'rotation':[0,0,0,1],'scale':1,'color':[.15,.8,.65],'attachment':{'parent':instance['id'],'bone':'secondary-06'}}
        call('edit_mesh',edit={'kind':'put_asset','asset':marker});call('edit_mesh',edit={'kind':'put_instance','instance':child})
        poses={tick:call('inspect_mesh_pose',instance=instance['id'],tick=tick)['pose'] for tick in [0,60]}
        child_poses={tick:call('inspect_mesh_pose',instance=child['id'],tick=tick)['pose'] for tick in [0,60]}
        assert child_poses[0]['positions']!=child_poses[60]['positions']
        call('edit_mesh',edit={'kind':'put_asset','asset':out})
        for tick,pose in poses.items():
            actual=call('inspect_mesh_pose',instance=instance['id'],tick=tick)['pose'];assert actual['positions'][:n]==pose['positions'] and actual['joints']==pose['joints'] and actual['world_transform']==pose['world_transform']
            assert call('inspect_mesh_pose',instance=child['id'],tick=tick)['pose']==child_poses[tick]
        topology=call('inspect_mesh_topology',asset=id)['topology'];assert topology['boundary_edges']==topology['non_manifold_edges']==0 and not topology['isolated_vertices']
        assert call('inspect_mesh_boundaries',asset=id)['boundaries']==[]
        islands=call('inspect_mesh_islands',asset=id)['topology']['islands'];assert len(islands)==1
        call('undo');assert call('inspect_mesh',asset=id)['asset']==source
        call('redo');assert call('inspect_mesh',asset=id)['asset']==out
        checks.append('Closed oriented single component, rigid added interior and exact retained world poses/joints at0/60, complete source rig/clip and exact animated socket-child0/60 preserved, one native undo/redo publication')
        call('edit_mesh',edit={'kind':'remove_instance','id':child['id']})
        call('set_light',light={'direction':[-.8,-1,-.4],'ambient':.5})
        views={}
        for kind in ['authored','neutral']:
            if kind=='neutral':call('edit_mesh',edit={'kind':'set_material','asset':id,'material':{}})
            for name,camera in CAMS.items():
                call('set_camera',camera=camera);call('step',ticks=60);assert call('inspect')['tick']==60
                _,views[kind+'-'+name]=capture(kind+'-'+name)
        # Same cameras/tick/light for the unrefined control, without the marker.
        call('edit_mesh',edit={'kind':'put_asset','asset':baseline['asset']})
        for kind in ['authored','neutral']:
            if kind=='neutral':call('edit_mesh',edit={'kind':'set_material','asset':id,'material':{}})
            for name,camera in CAMS.items():
                call('set_camera',camera=camera);call('step',ticks=60)
                _,views['baseline-'+kind+'-'+name]=capture('baseline-'+kind+'-'+name)
        call('edit_mesh',edit={'kind':'put_asset','asset':out})
        call('edit_mesh',edit={'kind':'set_material','asset':id,'material':out['material']})
        call('edit_mesh',edit={'kind':'put_instance','instance':child})
        call('set_camera',camera=CAMS['front']);call('save_scene',path='examples/mesh-junction-union.scene.json');authored=call('inspect')['scene'];rest,_=capture('rest')
        call('step',ticks=60);live,_=capture('live');assert live!=rest
        assert live==capture('repeat')[0]
        call('save_replay',path='artifacts/mesh-junction-union.replay.json');assert call('run_replay',path='artifacts/mesh-junction-union.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/mesh-junction-union.replay.json',path='artifacts/union-replay.png',width=1000,height=1000)
        assert (ROOT/'artifacts/union-replay.png').read_bytes()==live
        call('capture_replay',replay_path='artifacts/mesh-junction-union.replay.json',path='artifacts/union-prefix.png',tick=0,width=1000,height=1000)
        assert (ROOT/'artifacts/union-prefix.png').read_bytes()==rest
        call('load_scene',path='examples/mesh-junction-union.scene.json');assert call('inspect')['scene']==authored
        call('reset');assert capture('reset')[0]==rest
        call('capture',path='artifacts/union-viewport.png',width=640,height=480)
        checks.append('Native authored/neutral unshadowed front/profile/back at tick60; repeated cache, save/load/reset/live/replay/prefix pixels exact and viewport')
        call('step',ticks=11);before=call('inspect');bad=copy.deepcopy(fixture);bad['options']['guard_length']=.5
        call('preview_mesh_junction_union',expected_error=True,**bad);assert call('inspect')==before
        checks.append('Post-publication failed query preserves scene, running state and history; no Scene/replay/layout/budget change')
        verification={'passed':True,'checks':checks,'build_id':call('inspect')['build_id'],'native_window':window,'kernel':report,'views':views,'failures':failures,'vertices':len(out['geometry']['positions']),'triangles':len(out['geometry']['triangles']),
                      'retained_world_poses_exact':True,'standalone_proof':True,'whole_tree_integration':False,
                      'baseline':baseline['report'],'refinement_variants':variants}
        (ROOT/'artifacts/mesh-junction-union-verification.json').write_text(json.dumps(verification,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(checks),'build_id':verification['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
