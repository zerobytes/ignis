#!/usr/bin/env python3
"""Compare boundary continuity on the existing saved tree, without reauthoring its canopy."""
import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import random
import shutil
import struct

PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT/'artifacts/tree-continuity'
(OUTPUT/'artifacts').mkdir(parents=True, exist_ok=True)
(OUTPUT/'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client
from tree_study_geometry import add, sub, mul
from tree_junction_geometry import connected_first_fork
from demo_tree_authoring import CAMERA, LIGHT, SEED, trunk, trunk_radius


def f32(x):
    return struct.unpack('f', struct.pack('f', x))[0]


def edge_angles(g, loop):
    faces = {}
    for tri in g['triangles']:
        a,b,c = [g['positions'][i] for i in tri]
        u,v = sub(b,a),sub(c,a)
        n = [u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        length = math.sqrt(sum(x*x for x in n))
        assert length > 1e-12
        n = [x/length for x in n]
        for i in range(3):
            faces.setdefault(tuple(sorted([tri[i],tri[(i+1)%3]])),[]).append(n)
    angles=[]
    for i,a in enumerate(loop):
        ns = faces[tuple(sorted([a,loop[(i+1)%len(loop)]]))]
        assert len(ns)==2
        angles.append(math.degrees(math.acos(max(-1,min(1,sum(x*y for x,y in zip(*ns)))))))
    return {'max_degrees': max(angles), 'mean_degrees':sum(angles)/len(angles)}


def run(input_path):
    snapshot = OUTPUT/'examples/input.scene.json'
    if input_path.resolve()!=snapshot.resolve():shutil.copyfile(input_path,snapshot)
    c = Client(PROJECT/'target/release/ignis',False); call=c.call
    def asset(): return call('inspect_mesh',asset='wood')['asset']
    def capture(name,camera,tick=60):
        call('set_camera',camera=camera)
        # Camera edits reset execution, even when reapplying the same value.
        # Step AFTER the edit so comparisons/replay really show the stated pose.
        if tick:call('step',ticks=tick)
        path=f'artifacts/{name}.png'
        call('capture',path=path,width=1000,height=900)
        return hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest()
    cameras={'whole':CAMERA,'bark':{'position':[1.3,2.3,2],'target':[0.10,2,0.1],'fov_degrees':32},
             'fork':{'position':[1.4,2.2,3.5],'target':[-0.05,1.7,0.1],'fov_degrees':40}}
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-continuity-study','version':'1'}})
        c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json')
        call('set_light',light=LIGHT)
        initial=call('inspect')['scene']; original=asset(); old=original['geometry']
        assert 'rig' not in original and 'clips' not in original
        # Exact deterministic recipe used only to identify the saved first
        # component. All remaining geometry and authored assets come from input.
        rng=random.Random(SEED)
        for _ in range(8): rng.uniform(-.12,.12);rng.uniform(.8,1.1)
        start=trunk(.34);end=[-1.55,5.25,.70]
        controls=[start,add(start,mul(sub(end,start),.28)),add(end,[rng.uniform(-.45,.45),-1.15,rng.uniform(-.4,.4)]),end]
        controls[1][1]+=.45
        previous,old_report=connected_first_fork(controls,trunk,trunk_radius,call)
        nv,nt=old_report['assembly_vertices'],old_report['assembly_triangles']
        assert [[f32(x) for x in p] for p in previous['positions']]==old['positions'][:nv]
        assert previous['triangles']==old['triangles'][:nt]
        assert previous['material_coordinates']==previous['positions']
        assert old['material_coordinates'][:nv]==old['positions'][:nv]
        rest={tick:{i['id']:call('inspect_mesh_pose',instance=i['id'],tick=tick)['pose'] for i in initial['meshes']['instances'] if i['asset']!='wood'} for tick in [0,60]}
        views={name:{'before':capture('before-'+name,camera)} for name,camera in cameras.items()}
        before_queries=call('inspect')
        changed,report=connected_first_fork(controls,trunk,trunk_radius,call,continuity=True)
        assert call('inspect')==before_queries
        delta=len(changed['positions'])-nv
        assert delta==128
        g={'positions':changed['positions']+old['positions'][nv:],
           'triangles':changed['triangles']+[[i+delta for i in t] for t in old['triangles'][nt:]],
           'material_coordinates':changed['material_coordinates']+old['material_coordinates'][nv:]}
        assert all(i>=nv for t in old['triangles'][nt:] for i in t)
        edited=dict(original,geometry=g)
        pre=call('inspect');predicted=call('preview_mesh_edit',edit={'kind':'put_asset','asset':edited});assert call('inspect')==pre
        call('edit_mesh',edit={'kind':'put_asset','asset':edited})
        assert call('inspect')['scene']['meshes']['assets']==predicted['meshes']['assets']
        actual=asset()
        # Existing material domain is held on the native patch and all existing
        # root/trunk vertices. New neck sampling gets its explicit rest domain.
        common=report['neck_vertex_start']
        coords=copy.deepcopy(actual['geometry']['material_coordinates'])
        coords[:common]=old['material_coordinates'][:common]
        call('edit_mesh',edit={'kind':'set_material_coordinates','asset':'wood','coordinates':coords})
        actual=asset(); final=call('inspect')['scene']
        assert actual['material']==original['material']
        assert final['plants']==initial['plants']
        assert final['meshes']['instances']==initial['meshes']['instances']
        assert [a for a in final['meshes']['assets'] if a['id']!='wood']==[a for a in initial['meshes']['assets'] if a['id']!='wood']
        assert actual['geometry']['positions'][len(changed['positions']):]==old['positions'][nv:]
        assert actual['geometry']['material_coordinates'][len(changed['positions']):]==old['material_coordinates'][nv:]
        for tick,poses in rest.items():
            for id,expected in poses.items(): assert call('inspect_mesh_pose',instance=id,tick=tick)['pose']==expected
        call('undo');call('undo');assert asset()==original
        call('redo');call('redo');assert asset()==actual
        for name,camera in cameras.items():views[name]['after']=capture('after-'+name,camera)
        rings={}
        for i,id in enumerate(['trunk_below','limb_zero','trunk_above']):
            rings[id]={}
            for suffix,loop in [('outer',list(range(i*32,(i+1)*32))),('inner',list(range(96+i*32,96+(i+1)*32)))]:
                rings[id][suffix]={'before':edge_angles(old,loop),'after':edge_angles(actual['geometry'],loop)}
        outer=rings['limb_zero']['outer'];inner=rings['limb_zero']['inner']
        assert outer['after']['max_degrees']<outer['before']['max_degrees']*0.3
        assert inner['after']['max_degrees']<inner['before']['max_degrees']
        # Every side face of the revised neck must face away from its local
        # centreline; this rejected an earlier, mathematically smooth but folded
        # offset with radius*curvature >1. Endpoint tangency alone was insufficient.
        positions=actual['geometry']['positions']
        loops=[list(range(32,64))]+[list(range(common+32*i,common+32*(i+1))) for i in range(15)]
        centres=[[sum(positions[v][k] for v in loop)/32 for k in range(3)] for loop in loops]
        for row in range(15):
            for j in range(32):
                a,b,c0=loops[row][j],loops[row][(j+1)%32],loops[row+1][j]
                b1,c1,d=loops[row][(j+1)%32],loops[row+1][j],loops[row+1][(j+1)%32]
                for tri,blend in [([a,b,c0],1/3),([b1,d,c1],2/3)]:
                    pa,pb,pc=[positions[v] for v in tri];u,v=sub(pb,pa),sub(pc,pa)
                    normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
                    centre=[centres[row][k]*(1-blend)+centres[row+1][k]*blend for k in range(3)]
                    radial=[(pa[k]+pb[k]+pc[k])/3-centre[k] for k in range(3)]
                    assert sum(x*y for x,y in zip(normal,radial))>0
        live=capture('animated-bark',cameras['bark'])
        assert call('inspect')['tick']==60
        call('save_scene',path='examples/candidate.scene.json')
        call('save_replay',path='artifacts/tree.replay.json')
        assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/tree.replay.json',path='artifacts/replay.png',width=1000,height=900)
        assert hashlib.sha256((OUTPUT/'artifacts/replay.png').read_bytes()).hexdigest()==live
        call('capture_replay',replay_path='artifacts/tree.replay.json',path='artifacts/prefix.png',tick=0,width=1000,height=900)
        meshes=call('inspect_meshes');plants=call('inspect')['scene']['plants']
        call('load_scene',path='examples/candidate.scene.json')
        assert call('inspect_meshes')['assets']==meshes['assets'] and call('inspect')['scene']['plants']==plants
        assert capture('loaded-prefix',cameras['bark'],tick=0)==hashlib.sha256((OUTPUT/'artifacts/prefix.png').read_bytes()).hexdigest()
        result={'passed':True,'build_id':call('inspect')['build_id'],'input_sha256':hashlib.sha256(snapshot.read_bytes()).hexdigest(),
                'views':views,'junction':report,'guard_dihedrals':rings,'geometry_vertices_added':delta,
                'other_assets_instances_plants_exact':True,'other_poses_ticks_0_60_exact':True,
                'unchanged_wood_tail_exact':True,'materials_exact':True,'native_window':False,
                'comparison_tick':60,'live_and_prefix_replay_pixels_exact':True,
                'notes':['Local first fork revision; other overlapped insertions remain open','Implicit tangent condition differs from finite triangle approximation','New neck is explicit reauthoring without inherited sculpt identity']}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps({'passed':True,'build_id':result['build_id'],'added_vertices':delta,'guard_dihedrals':rings}))
    finally:c.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--input',type=Path,default=PROJECT/'examples/native-tree-study.scene.json');args=parser.parse_args()
    run(args.input.resolve())
