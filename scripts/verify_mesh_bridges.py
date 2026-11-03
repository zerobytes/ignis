#!/usr/bin/env python3
"""Real boundary cuts, constrained lofts and preserved channels through MCP."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def fixture():
    points, triangles = [], []
    for part, ys in enumerate([[0, 1], [2, 3]]):
        base = part*16
        for y in ys:
            for j in range(8):
                a = j*math.tau/8
                points.append([math.sin(a)*.6, y, math.cos(a)*.6])
        for j in range(8):
            a, b, c, d = base+j, base+(j+1)%8, base+8+j, base+8+(j+1)%8
            triangles.extend([[a,b,c], [b,d,c]])
        for j in range(1,7):
            triangles.extend([[base,base+j+1,base+j], [base+8,base+8+j,base+9+j]])
    rig = {'bones':[{'name':'root','parent':None,'translation':[0,0,0],'rotation':[0,0,0,1]}],
           'weights':[{'joints':[0,0,0,0],'weights':[1,0,0,0]} for _ in points]}
    clip = {'id':'move','duration_ticks':120,'tracks':[{'bone':0,'keys':[
        {'tick':0,'translation':[0,0,0],'rotation':[0,0,0,1]},
        {'tick':60,'translation':[.4,0,0],'rotation':[0,0,0,1]},
        {'tick':120,'translation':[0,0,0],'rotation':[0,0,0,1]}]}]}
    return {'id':'tubes','geometry':{'positions':points,'triangles':triangles,'material_coordinates':points},
            'shading':'smooth','rig':rig,'clips':[clip]}


def verify(binary, window):
    c = Client(binary, window); call = c.call; checks = []
    def asset(): return call('inspect_mesh',asset='tubes')['asset']
    def reference(): return call('inspect_mesh_boundaries',asset='tubes')['reference']
    def capture(name, **kw):
        call('capture',path=f'artifacts/bridge-{name}.png',width=800,height=700,**kw)
        return (ROOT/f'artifacts/bridge-{name}.png').read_bytes()
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-bridge-verifier','version':'1'}})
        c.request('notifications/initialized',notification=True)
        schema=json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        assert all(s in schema for s in ['inspect_mesh_boundaries','remove_faces','bridge_boundary_loops'])
        call('generate',recipe={'seed':7,'width':20,'depth':20,'obstacles':0})
        call('edit_mesh',edit={'kind':'put_asset','asset':fixture()})
        call('edit_mesh',edit={'kind':'put_instance','instance':{'id':'tubes','asset':'tubes','position':[0,1,0],
            'rotation':[0,0,0,1],'scale':1,'color':[.3,.6,.2],'playback':{'clip':'move','offset_ticks':0,'looping':True}}})
        call('set_camera',camera={'position':[4,4,8],'target':[0,2.5,0],'fov_degrees':40})
        source=asset(); before=call('inspect'); assert call('inspect_mesh_boundaries',asset='tubes')['boundaries']==[]
        cut={'kind':'remove_faces','asset':'tubes','reference':reference(),'faces':[v for j in range(6) for v in [17+j*2,44+j*2]]}
        preview=call('preview_mesh_edit',edit=cut); assert call('inspect')==before
        call('edit_mesh',edit=cut); assert call('inspect')['scene']['meshes']==preview['meshes']
        boundaries=call('inspect_mesh_boundaries',asset='tubes'); assert len(boundaries['boundaries'])==2
        assert boundaries==call('inspect_mesh_boundaries',asset='tubes')
        cut_asset=asset(); assert cut_asset['geometry']['positions']==source['geometry']['positions']
        gap=capture('gap'); state=call('inspect')
        bridge={'kind':'bridge_boundary_loops','asset':'tubes','reference':reference(),'loops':[list(range(8,16)),list(range(16,24))],'spans':4,'handles':[.3,.3]}
        invalid=[dict(bridge,loops=[list(reversed(bridge['loops'][0])),bridge['loops'][1]]),
            dict(bridge,loops=[bridge['loops'][0],bridge['loops'][0]]),dict(bridge,handles=[0,.3]),dict(bridge,spans=33),dict(bridge,spans=8,handles=[2,2]),
            dict(cut,reference=reference(),faces=[999]),dict(cut,reference=reference(),faces=[0,0]),cut]
        for edit in invalid:
            call('preview_mesh_edit',edit=edit,expected_error=True); call('edit_mesh',edit=edit,expected_error=True)
            assert call('inspect')==state
        # A reference may not silently rebind after earlier topology edits in the same batch.
        call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_asset','asset':cut_asset}}, {'op':'mesh','edit':bridge}],expected_error=True)
        assert call('inspect')==state
        predicted=call('preview_mesh_edit',edit=bridge); assert call('inspect')==state
        call('edit_mesh',edit=bridge); assert call('inspect')['scene']['meshes']==predicted['meshes']
        final=asset(); assert len(final['geometry']['positions'])==56 and len(final['geometry']['triangles'])==108
        for channel in ['positions','material_coordinates']:
            assert final['geometry'][channel][:32]==source['geometry'][channel]
        assert final['rig']['weights'][:32]==source['rig']['weights']
        assert final['rig']['bones']==source['rig']['bones'] and final['clips']==source['clips'] and final['material']==source['material']
        topology=call('inspect_mesh_topology',asset='tubes')['topology']
        assert topology['boundary_edges']==topology['non_manifold_edges']==0 and not topology['isolated_vertices']
        assert call('inspect_mesh_boundaries',asset='tubes')['boundaries']==[]
        joined=capture('joined'); assert gap!=joined and joined==capture('repeat')
        call('undo'); assert asset()==cut_asset and capture('undo')==gap
        call('redo'); assert asset()==final and capture('redo')==joined
        checks.append('Native schema, deterministic real cycles, pure previews, winding/IDs/handles/stale/batch errors atomic; exact retained geometry/domain/rig/clips/material and closed joined topology')
        rest=call('inspect_mesh_pose',instance='tubes',tick=0)['pose']['positions']
        moved=call('inspect_mesh_pose',instance='tubes',tick=60)['pose']['positions']
        for a,b in zip(rest,moved): assert math.dist([a[0]+.4,a[1],a[2]],b)<1e-6
        call('save_scene',path='examples/mesh-bridges.scene.json'); authored=call('inspect')['scene']
        call('step',ticks=60); motion=capture('motion'); assert motion!=joined
        call('save_replay',path='artifacts/mesh-bridges.replay.json')
        assert call('run_replay',path='artifacts/mesh-bridges.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/mesh-bridges.replay.json',path='artifacts/bridge-replay.png',width=800,height=700)
        assert (ROOT/'artifacts/bridge-replay.png').read_bytes()==motion
        call('capture_replay',replay_path='artifacts/mesh-bridges.replay.json',path='artifacts/bridge-prefix.png',tick=0,width=800,height=700)
        assert (ROOT/'artifacts/bridge-prefix.png').read_bytes()==joined
        call('load_scene',path='examples/mesh-bridges.scene.json'); assert call('inspect')['scene']==authored
        call('reset'); assert capture('reset')==joined
        call('capture',path='artifacts/bridge-viewport.png',width=640,height=480)
        checks.append('New vertices follow preserved clip; rest/motion captures, cache repeat/undo/redo, save/load/reset and live/replay/prefix pixels exact, viewport')
        # The tree is a protected parametric-source fixture, not a special bridge path.
        call('load_scene',path='examples/native-tree-study.scene.json'); before=call('inspect')
        linked=call('inspect_mesh_boundaries',asset='branch-editable-1')['reference']
        for edit in [{'kind':'remove_faces','asset':'branch-editable-1','reference':linked,'faces':[0]},
            dict(bridge,asset='branch-editable-1',reference=linked)]:
            error=call('edit_mesh',edit=edit,expected_error=True); assert 'Detach plant explicitly' in str(error)
            assert call('inspect')==before
        checks.append('Linked plant rejects local cuts and bridges without losing canonical source, rig, execution or history')
        report={'passed':True,'checks':checks,'build_id':call('inspect')['build_id'],'native_window':window}
        (ROOT/'artifacts/mesh-bridges-verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'passed':True,'checks':len(checks),'build_id':report['build_id']}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
