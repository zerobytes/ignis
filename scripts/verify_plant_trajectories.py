#!/usr/bin/env python3
"""Native trajectory/normal sections: isolated constant-radius cylinder proof."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import argparse
import copy
import json
import math
import struct
from pathlib import Path
from verify_lab import Client, ROOT


def f32(x): return struct.unpack('f',struct.pack('f',x))[0]


def verify(binary,window=False):
    c=Client(binary,window);checks=[]
    call=c.call
    def put(d,operation='edit_plant',**kwargs):
        return call(operation,edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'},**kwargs)
    def asset(id): return call('inspect_mesh',asset=id)['asset']
    def plant(id): return call('inspect_plant',id=id)['plant']
    def mesh(id,kind,**kwargs): return call('edit_mesh',edit={'kind':kind,'asset':id,**kwargs})
    def capture(name):
        path=ROOT/f'artifacts/cylinder-{name}.png'
        call('capture',path=f'artifacts/{path.name}',width=1000,height=1000)
        return path.read_bytes()
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-trajectory-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        schema=json.dumps(c.request('tools/list')['result']);assert 'control_points_xz' in schema and 'surface_tolerance' in schema
        call('generate',recipe={'seed':97,'width':12,'depth':12,'obstacles':0})
        entities=call('inspect')['scene']['entities']
        for entity in entities: entity['visible']=False
        call('edit_scene',edits=[{'op':'upsert','entity':e} for e in entities])
        call('set_light',light={'direction':[-0.6,-0.8,-0.7],'ambient':0.38})
        d=json.loads((ROOT/'examples/native-curved-cylinder.plant.json').read_text());id=d['id']
        before=call('inspect');preview=put(d,'preview_plant');assert call('inspect')==before
        put(d);assert asset(id)==next(a for a in preview['assets'] if a['id']==id)
        mesh(id,'set_shading',shading='smooth')
        instance={'id':'cylinder','asset':id,'position':[0,1.2,0],'rotation':[0,0,0,1],'scale':1,'color':[0.56,0.61,0.68],'playback':None}
        call('edit_mesh',edit={'kind':'put_instance','instance':instance})
        original=asset(id);canonical=plant(id)
        state=call('inspect');sampling=call('inspect_plant_sampling',id=id)
        assert call('inspect')==state and sampling['revision']==state['revision']
        assert sampling['sampling']['geometry']==original['geometry']
        assert len(sampling['sampling']['material'])==len(sampling['sampling']['surface'])==len(original['geometry']['positions'])
        call('inspect_plant_sampling',id='missing',expected_error=True);assert call('inspect')==state
        checks.append('Sampling query exposes the actual native base/domain and per-vertex surface identity without mutating state; missing IDs reject atomically')
        topology=call('inspect_mesh_topology',asset=id)['topology']
        assert topology['boundary_edges']==topology['non_manifold_edges']==0 and not topology['isolated_vertices']
        assert len(original['geometry']['positions'])==3922 and len(original['geometry']['triangles'])==7840
        report=call('inspect_plant',id=id)['report']['trajectory']
        assert report['inverse_derivative_margin']>0 and report['sampled_surface_error']<=f32(d['trajectory']['surface_tolerance'])
        # Independent closed-form S curve, not an author-side mesh generator.
        geometry=original['geometry'];maximum_radius_error=0.;maximum_plane_error=0.;ring_count=0
        height=f32(d['profile']['height']);shift=f32(-0.1);radius=f32(d['profile']['width'])*0.5
        rows={}
        # Recover only the documented shared-grid vertex provenance. Positions
        # and triangles are exclusively native; circle/plane equations below
        # are independent of the engine's frame and inverse implementation.
        keys=[];seen=set();segments=d['segments']
        for axis in range(3):
            u,v=(axis+1)%3,(axis+2)%3
            for side in range(2):
                for j in range(segments[v]+1):
                    for i in range(segments[u]+1):
                        key=[0,0,0];key[axis]=segments[axis] if side else 0;key[u]=i;key[v]=j
                        key=tuple(key)
                        if key not in seen:seen.add(key);keys.append(key)
        assert len(keys)==len(geometry['positions'])
        for key,p in zip(keys,geometry['positions']):
            if key[0] in [0,segments[0]] or key[2] in [0,segments[2]]:
                rows.setdefault(key[1],[]).append(p)
        for row,points in rows.items():
            t=f32(f32(row/d['segments'][1])-0.5)+0.5
            center=[shift*(3*t*t-2*t*t*t),height*(t-0.5),0.]
            derivative=[shift*6*t*(1-t),height,0.];speed=math.sqrt(sum(v*v for v in derivative));tangent=[v/speed for v in derivative]
            for p in points:
                radial=[p[k]-center[k] for k in range(3)]
                maximum_radius_error=max(maximum_radius_error,abs(math.sqrt(sum(v*v for v in radial))-radius))
                maximum_plane_error=max(maximum_plane_error,abs(sum(a*b for a,b in zip(radial,tangent))))
            ring_count+=1
        checked_side_vertices=sum(len(points) for points in rows.values())
        assert ring_count==64 and checked_side_vertices==3584 and all(len(points)==56 for points in rows.values())
        assert maximum_radius_error<3e-8 and maximum_plane_error<3e-8
        checks.append('Native pure preview builds a closed3922v7840t constant circle;64normal cuts match independent radius/plane equations within30nm')

        changed=copy.deepcopy(d)
        changed['trajectory']['control_points_xz']=[[0,0],[0,0],[-0.08,0],[-0.08,0]]
        for _ in range(2):
            put(changed);assert plant(id)['source']==canonical['source'] and plant(id)['reference']==canonical['reference']
            put(d);assert asset(id)==original
        call('undo');assert plant(id)['definition']['trajectory']!=canonical['definition']['trajectory']
        call('redo');assert asset(id)==original
        before=call('inspect')
        for kind in ['thick','coarse','identity','unknown','tolerance','stale']:
            bad=copy.deepcopy(d);options={}
            if kind=='thick':bad['profile']['height']=0.1
            if kind=='coarse':bad['segments']=[2,2,2]
            if kind=='identity':bad['trajectory']['id']='other-axis'
            if kind=='unknown':bad['trajectory']['automatic_weld']=True
            if kind=='tolerance':bad['trajectory']['surface_tolerance']=0
            if kind=='stale':options['expected_revision']=before['revision']-1
            put(bad,expected_error=True,**options);assert call('inspect')==before
        checks.append('Bending cycles preserve canonical source/reference exactly; inverse-risk, coarse surface, identity, unknown fields and stale edits reject atomically')

        proof=copy.deepcopy(d);proof['id']='preservation';proof['segments']=[4,16,4]
        proof['trajectory']['surface_tolerance']=0.002;proof['trajectory']['control_points_xz']=[[0,0]]*4
        put(proof);base=asset('preservation')['geometry']['positions']
        point=next(p for p in base if p[2]>0.019 and abs(p[1])<0.001)
        mesh('preservation','sculpt',stroke={'kind':'grab','centers':[point],'radius':0.003,'strength':0.0007,'direction':[0,0,1]})
        mesh('preservation','set_material_coordinates',coordinates=base)
        tip=[0.004,0.015,0.003];root=[0,-0.12,0]
        bones=[{'name':'root','parent':None,'translation':root,'rotation':[0,0,0,1]},
               {'name':'tip','parent':0,'translation':[tip[k]-root[k] for k in range(3)],'rotation':[0,0,0,1]}]
        weights=[{'joints':[0,1,0,0],'weights':[0.25,0.75,0,0]} for _ in base]
        mesh('preservation','set_rig',rig={'bones':bones,'weights':weights})
        mesh('preservation','put_clip',clip={'id':'bend','duration_ticks':120,'tracks':[{'bone':1,'keys':[
            {'tick':0,'translation':bones[1]['translation'],'rotation':[0,0,0,1]},
            {'tick':60,'translation':bones[1]['translation'],'rotation':[0.1,0,0,math.sqrt(0.99)]},
            {'tick':120,'translation':bones[1]['translation'],'rotation':[0,0,0,1]}]}]})
        decorated=asset('preservation');source=plant('preservation')
        state=call('inspect');sampling=call('inspect_plant_sampling',id='preservation')['sampling']
        assert sampling['geometry']['positions']==base
        assert sampling['geometry']['positions']!=decorated['geometry']['positions']
        assert call('inspect')==state
        curved=copy.deepcopy(proof);curved['trajectory']['control_points_xz']=d['trajectory']['control_points_xz']
        put(curved);result=asset('preservation')
        assert plant('preservation')['source']==source['source'] and plant('preservation')['reference']==source['reference']
        assert result['rig']['weights']==decorated['rig']['weights'] and result['geometry']['material_coordinates']==decorated['geometry']['material_coordinates']
        assert result['rig']['bones'][1]['translation']!=decorated['rig']['bones'][1]['translation']
        assert result['clips'][0]['tracks'][0]['keys'][0]['translation']==result['rig']['bones'][1]['translation']
        assert call('inspect_plant',id='preservation')['report']['max_detail_displacement']>0.0006
        dense=copy.deepcopy(curved);dense['segments']=[6,24,6]
        put(dense);refined=asset('preservation')
        assert len(refined['geometry']['positions'])>len(result['geometry']['positions'])
        assert len(refined['geometry']['material_coordinates'])==len(refined['geometry']['positions'])
        assert plant('preservation')['source']==source['source'] and plant('preservation')['reference']==source['reference']
        assert refined['rig']['bones']==result['rig']['bones']
        assert all(abs(sum(w['weights'])-1)<1e-6 for w in refined['rig']['weights'])
        put(curved);assert asset('preservation')==result
        for _ in range(2):put(proof);assert asset('preservation')==decorated;put(curved);assert asset('preservation')==result
        animated={'id':'animated-proof','asset':'preservation','position':[0.13,1.2,0],'rotation':[0,0,0,1],'scale':1,'color':[0.5,0.6,0.7],
                  'playback':{'clip':'bend','offset_ticks':0,'looping':True}}
        call('edit_mesh',edit={'kind':'put_instance','instance':animated})
        camera={'position':[0.4,1.4,0.65],'target':[0,1.2,0],'fov_degrees':32}
        call('set_camera',camera=camera);call('step',ticks=60)
        assert call('inspect_meshes')['tick']==60
        assert call('inspect_mesh_pose',instance='animated-proof',tick=60)['pose']['positions']!=call('inspect_mesh_pose',instance='animated-proof',tick=0)['pose']['positions']
        live=capture('preservation-animated');assert capture('preservation-repeat')==live
        call('save_replay',path='artifacts/cylinder-preservation.replay.json')
        assert call('run_replay',path='artifacts/cylinder-preservation.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/cylinder-preservation.replay.json',path='artifacts/cylinder-preservation-replay.png',width=1000,height=1000)
        assert (ROOT/'artifacts/cylinder-preservation-replay.png').read_bytes()==live
        checks.append('Separate fixture preserves sculpt/material/weights, maps bind origins and clip translations; explicit tick60 animates and replay pixels match')

        call('edit_mesh',edit={'kind':'remove_instance','id':'animated-proof'})
        call('edit_plant',edit={'kind':'detach','id':'preservation'})
        call('edit_mesh',edit={'kind':'remove_asset','id':'preservation'})
        front={'position':[-0.05,1.2,0.8],'target':[-0.05,1.2,0],'fov_degrees':28}
        oblique={'position':[0.35,1.4,0.65],'target':[-0.05,1.2,0],'fov_degrees':30}
        call('set_camera',camera=front);capture('front')
        mesh(id,'set_shading',shading='flat');capture('front-flat')
        call('undo');assert asset(id)==original
        call('set_camera',camera=oblique);clean=capture('oblique')
        call('save_scene',path='examples/native-curved-cylinder.scene.json')
        call('save_replay',path='artifacts/cylinder.replay.json')
        assert call('run_replay',path='artifacts/cylinder.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/cylinder.replay.json',path='artifacts/cylinder-replay.png',width=1000,height=1000)
        assert (ROOT/'artifacts/cylinder-replay.png').read_bytes()==clean
        before_plant=plant(id)
        call('load_scene',path='examples/native-curved-cylinder.scene.json')
        assert asset(id)==original and plant(id)==before_plant and capture('loaded')==clean
        scene=json.loads((ROOT/'examples/native-curved-cylinder.scene.json').read_text());assert scene['version']==SCENE_VERSION
        scene['version']=63;(ROOT/'examples/cylinder-old-version.scene.json').write_text(json.dumps(scene))
        before=call('inspect');call('load_scene',path='examples/cylinder-old-version.scene.json',expected_error=True);assert call('inspect')==before
        assert json.loads((ROOT/'artifacts/cylinder.replay.json').read_text())['version']==REPLAY_VERSION
        checks.append('Clean neutral cylinder captures expose front/oblique/flat geometry; scene64/replay68, save-load/source and live/replay pixels agree; old format rejects')
        result={'passed':True,'checks':checks,'build_id':call('inspect')['build_id'],'native_window':window,
                'vertices':3922,'triangles':7840,'radius_m':radius,'height_m':height,'endpoint_shift_m':shift,
                'checked_cuts':ring_count,'checked_side_vertices':checked_side_vertices,'max_radius_error_m':maximum_radius_error,'max_normal_plane_error_m':maximum_plane_error,
                'trajectory_report':report,'scope':'Isolated native cubic graph trajectory; no tree/junction work or global Hausdorff guarantee'}
        (ROOT/'artifacts/plant-trajectories-verification.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(checks),'build_id':result['build_id']}))
    finally:c.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');parser.add_argument('--window',action='store_true');args=parser.parse_args()
    verify(args.binary.resolve(),args.window)
