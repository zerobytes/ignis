#!/usr/bin/env python3
"""Exercise native profile/path sweeps through the shared MCP dispatcher."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def fixture():
    return {'profile':[[-.09,-.07],[-.06,-.1],[.06,-.1],[.09,-.07],[.09,.07],[.06,.1],[-.06,.1],[-.09,.07]],
            'curves':[[[-1,0,0],[-1,2,0],[1,2,.5],[1,0,.5]]],
            'spans':24,'profile_x':[0,0,1],
            'sections':[{'at':0,'scale':[1,1]},{'at':.5,'scale':[1.5,.8]},{'at':1,'scale':[1,1]}],
            'capped':True,'centerline_tolerance':.005}


def verify(binary, window):
    c=Client(binary,window);call=c.call;checks=[]
    def asset():return call('inspect_mesh',asset='molding')['asset']
    def capture(name):
        path=f'artifacts/sweep-{name}.png';call('capture',path=path,width=800,height=700)
        return (ROOT/path).read_bytes()
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'sweep-verification','version':'1'}})
        c.request('notifications/initialized',notification=True)
        schema=json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema']);assert 'centerline_tolerance' in schema and 'profile_x' in schema
        call('generate',recipe={'seed':7,'width':20,'depth':20,'obstacles':0})
        sweep=fixture();edit={'kind':'sweep','id':'molding','sweeps':[sweep]};before=call('inspect')
        coarse=copy.deepcopy(sweep);coarse['spans']=1
        planned=call('plan_mesh_sweep',sweeps=[coarse])['plan'];assert call('inspect')==before
        assert planned['pieces'][0]['spans']>1 and planned['pieces'][0]['centerline_deviation_bound']<=coarse['centerline_tolerance'] and not planned['surface_validated']
        coarse['spans']=planned['pieces'][0]['spans'];predicted=call('preview_mesh_edit',edit={**edit,'sweeps':[coarse]})
        assert len(predicted['meshes']['assets'][0]['geometry']['positions'])==planned['vertices'];assert call('inspect')==before
        checks.append('Pure planner chooses bounded centerline tessellation and reports costs without claiming surface admission')
        preview=call('preview_mesh_edit',edit=edit);assert call('inspect')==before
        call('edit_mesh',edit=edit);assert call('inspect')['scene']['meshes']==preview['meshes']
        g=asset()['geometry'];assert len(g['positions'])==200 and len(g['triangles'])==396 and len(g['material_coordinates'])==200
        topology=call('inspect_mesh_topology',asset='molding')['topology'];assert topology['boundary_edges']==topology['non_manifold_edges']==0 and not topology['isolated_vertices']
        assert len(call('inspect_mesh_islands',asset='molding')['topology']['islands'])==1
        checks.append('Native 3D Bezier/profile/sections are discoverable, preview-pure, deterministic and closed manifold')
        variants=[]
        for change in ['reverse','duplicate','outside','parallel','disconnected','station','cusp','coarse','budget']:
            s=copy.deepcopy(sweep)
            if change=='reverse':s['profile'].reverse()
            elif change=='duplicate':s['profile'][1]=s['profile'][0]
            elif change=='outside':s['profile']=[[p[0]+2,p[1]] for p in s['profile']]
            elif change=='parallel':s['profile_x']=[0,1,0]
            elif change=='disconnected':s['curves'].append([[1,1,.5],[1,0,.5],[0,0,0],[0,1,0]])
            elif change=='station':s['sections'][1]['at']=0
            elif change=='cusp':s['curves'][0][1]=s['curves'][0][0]
            elif change=='coarse':s['spans']=1
            elif change=='budget':s['spans']=129
            variants.append({'kind':'sweep','id':'molding','sweeps':[s]})
        variants.append({'kind':'sweep','id':'molding','sweeps':[sweep]*128})
        state=call('inspect')
        for bad in variants:
            call('preview_mesh_edit',edit=bad,expected_error=True);call('edit_mesh',edit=bad,expected_error=True);assert call('inspect')==state
        call('edit_mesh',edit=edit,expected_revision=state['revision']-1,expected_error=True);assert call('inspect')==state
        checks.append('Profile/frame/cusp/continuity/station/tolerance/budget/stale requests reject atomically')
        reference=call('inspect_mesh_topology',asset='molding')['reference']
        call('edit_scene',edits=[{'op':'mesh','edit':edit},{'op':'mesh','edit':{'kind':'subdivide','asset':'molding','reference':reference,'components':None}}],expected_error=True)
        assert call('inspect')==state
        definition={'id':'linked','profile':{'id':'outline','width':1,'height':2},'depth':.4,'segments':[2,4,2]}
        call('edit_plant',edit={'kind':'put','definition':definition,'detail_policy':'physical','animation_translation_policy':'physical'})
        state=call('inspect');call('edit_mesh',edit={**edit,'id':'linked'},expected_error=True);assert call('inspect')==state
        checks.append('Sweep invalidates earlier component references and cannot replace a linked plant')
        # Appending preserves existing indices/channels/appearance; it does not
        # invent influences for new vertices of an already rigged asset.
        source=asset();call('edit_mesh',edit={'kind':'set_material','asset':'molding','material':{'transmission':.2}})
        shifted=copy.deepcopy(sweep)
        shifted['curves']=[[[p[0]+3,p[1],p[2]] for p in curve] for curve in shifted['curves']]
        append={'kind':'sweep','id':'molding','sweeps':[shifted],'append':True}
        call('edit_mesh',edit=append);combined=asset();n=len(source['geometry']['positions']);t=len(source['geometry']['triangles'])
        assert combined['geometry']['positions'][:n]==source['geometry']['positions'] and combined['geometry']['triangles'][:t]==source['geometry']['triangles']
        assert combined['geometry']['material_coordinates'][:n]==source['geometry']['material_coordinates']
        assert abs(combined['material']['transmission']-.2)<1e-6 and len(call('inspect_mesh_islands',asset='molding')['topology']['islands'])==2
        call('undo');call('undo');assert asset()==source
        rig={'bones':[{'name':'root','parent':None,'translation':[0,0,0],'rotation':[0,0,0,1]}],'weights':[{'joints':[0,0,0,0],'weights':[1,0,0,0]} for _ in range(n)]}
        call('edit_mesh',edit={'kind':'set_rig','asset':'molding','rig':rig});state=call('inspect');call('edit_mesh',edit=append,expected_error=True);assert call('inspect')==state;call('undo')
        call('edit_mesh',edit={**append,'id':'missing'},expected_error=True)
        checks.append('Append keeps old geometry/channels/appearance and rejects missing or rigged sources')
        i={'id':'molding','asset':'molding','position':[0,2,0],'rotation':[0,0,0,1],'scale':1,'color':[.8,.66,.4]}
        call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_instance','instance':i}},{'op':'set_camera','camera':{'position':[5,4,7],'target':[0,2.2,0],'fov_degrees':40}}])
        original=asset();neutral=capture('start');assert capture('repeat')==neutral
        altered=copy.deepcopy(edit);altered['sweeps'][0]['sections'][1]['scale']=[2,1.5]
        call('edit_mesh',edit=altered);assert capture('edited')!=neutral;call('undo');assert asset()==original and capture('undo')==neutral
        call('redo');assert asset()!=original;call('undo')
        batch=[{'op':'mesh','edit':{'kind':'remove_instance','id':'molding'}},{'op':'mesh','edit':{'kind':'remove_asset','id':'molding'}}]
        call('edit_scene',edits=batch);assert capture('removed')!=neutral;call('undo');assert capture('restored')==neutral
        checks.append('Changed geometry/removal/undo/redo invalidate render at the same tick without material/rig inheritance')
        call('save_scene',path='examples/native-sweep.scene.json');scene=call('inspect')['scene'];call('step',ticks=30);live=capture('live')
        call('save_replay',path='artifacts/sweep.replay.json');assert call('run_replay',path='artifacts/sweep.replay.json')['meshes_matched']
        for name,t,pixels in [('replay',30,live),('prefix',0,neutral)]:
            path=f'artifacts/sweep-{name}.png';call('capture_replay',replay_path='artifacts/sweep.replay.json',path=path,tick=t,width=800,height=700);assert (ROOT/path).read_bytes()==pixels
        call('load_scene',path='examples/native-sweep.scene.json');assert call('inspect')['scene']==scene and capture('loaded')==neutral
        call('reset');assert capture('reset')==neutral
        call('capture',path='artifacts/sweep-viewport.png',width=640,height=480)
        checks.append('Persistence/reset/replay/prefix and viewport preserve ordinary generated assets and exact equivalent pixels')
        # Scalar callers retain exact geometry; equal arrays select the same samples.
        scalar=call('preview_mesh_edit',edit=edit)['meshes']
        array=copy.deepcopy(edit);array['sweeps'][0]['spans']=[sweep['spans']]
        state=call('inspect')
        assert call('preview_mesh_edit',edit=array)['meshes']==scalar
        assert call('inspect')==state
        checks.append('Scalar sampling and uniform arrays generate exact positions/topology/material channels without changing state')
        chain=copy.deepcopy(sweep)
        chain['curves']=[[[-1,0,0],[-1,1,0],[-1,2,0],[-1,3,0]],
                         [[-1,3,0],[-1,4,0],[0,5,.5],[0,6,.5]]]
        chain['spans']=[1,1]
        plan=call('plan_mesh_sweep',sweeps=[chain])['plan']
        piece=plan['pieces'][0]
        assert piece['requested_spans']==[1,1] and piece['spans'][0]==1 and piece['spans'][1]>1
        assert len(piece['curve_centerline_bounds'])==2 and max(piece['curve_centerline_bounds'])<=chain['centerline_tolerance']
        assert not plan['surface_validated'] and call('inspect')==state
        chain['spans']=piece['spans']
        array_edit=dict(kind='sweep',id='chain',sweeps=[chain])
        predicted=call('preview_mesh_edit',edit=array_edit)
        assert call('inspect')==state
        call('edit_mesh',edit=array_edit)
        assert call('inspect')['scene']['meshes']==predicted['meshes']
        generated=call('inspect_mesh',asset='chain')['asset']['geometry']
        assert len(generated['positions'])==(1+sum(piece['spans']))*len(chain['profile'])
        knot=generated['positions'][len(chain['profile'])]
        assert abs(knot[0]+1.056)<1e-6 and abs(knot[1]-3)<1e-6 and abs(knot[2]+.135)<1e-6
        topo=call('inspect_mesh_topology',asset='chain')['topology']
        assert topo['boundary_edges']==topo['non_manifold_edges']==0
        call('undo');assert call('inspect')['scene']==state['scene']
        call('redo');assert call('inspect_mesh',asset='chain')['asset']['geometry']==generated
        call('save_scene',path='examples/native-sweep-array.scene.json')
        call('step',ticks=10);call('save_replay',path='artifacts/sweep-array.replay.json')
        assert call('run_replay',path='artifacts/sweep-array.replay.json')['meshes_matched']
        call('load_scene',path='examples/native-sweep-array.scene.json')
        assert call('inspect_mesh',asset='chain')['asset']['geometry']==generated
        checks.append('Independent refinement keeps straight1, shares one knot at section0.5, bounded topology/preview/undo/save/load/replay retain generated geometry')
        state=call('inspect')
        for bad in [[],[1],[1,0],[1,129],[1,1,1],[1,1.5],[-1,1],None]:
            invalid=copy.deepcopy(chain);invalid['spans']=bad
            call('plan_mesh_sweep',sweeps=[invalid],expected_error=True)
            call('edit_mesh',edit=dict(kind='sweep',id='chain',sweeps=[invalid]),expected_error=True)
            assert call('inspect')==state
        checks.append('Per-curve shape/length/types/zero/range failures are atomic for planner and publication')
        report={'passed':True,'checks':checks,'build_id':call('inspect')['build_id'],'native_window':window}
        (ROOT/'artifacts/sweep-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'passed':True,'checks':len(checks),'build_id':report['build_id']}))
    finally:c.close()


if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');ap.add_argument('--window',action='store_true');args=ap.parse_args();verify(args.binary.resolve(),args.window)
