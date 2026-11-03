#!/usr/bin/env python3
"""Native chamfered box authoring through MCP, isolated and headless by default."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary,window):
    c=Client(binary,window);call=c.call;checks=[]
    edit={'kind':'chamfer_boxes','id':'blocks','boxes':[{'center':[-1,0,0],'size':[1,1.5,1],'bevel':0},{'center':[1,0,0],'size':[1,1.5,1],'bevel':.12}]}
    def asset():return call('inspect_mesh',asset='blocks')['asset']
    def capture(name):
        path=f'artifacts/chamfer-{name}.png';call('capture',path=path,width=800,height=650);return (ROOT/path).read_bytes()
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'chamfer-verification','version':'1'}});c.request('notifications/initialized',notification=True)
        assert 'chamfer_boxes' in json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        call('generate',recipe={'seed':3,'width':20,'depth':20,'obstacles':0});before=call('inspect')
        preview=call('preview_mesh_edit',edit=edit);assert call('inspect')==before
        call('edit_mesh',edit=edit);assert call('inspect')['scene']['meshes']==preview['meshes']
        g=asset()['geometry'];assert len(g['positions'])==32 and len(g['triangles'])==56 and g['material_coordinates']==g['positions']
        top=call('inspect_mesh_topology',asset='blocks')['topology'];assert top['boundary_edges']==top['non_manifold_edges']==0 and not top['isolated_vertices']
        assert len(call('inspect_mesh_islands',asset='blocks')['topology']['islands'])==2
        checks.append('Schema/preview/commit create analytic zero-bevel and chamfered closed boxes with explicit material coordinates')
        variants=[]
        for key,value in [('size',[0,1,1]),('bevel',-.1),('bevel',.3),('rotation',[0,0,0,0]),('center',[10001,0,0])]:
            bad=copy.deepcopy(edit);bad['boxes'][0][key]=value;variants.append(bad)
        variants.extend([{**edit,'boxes':[]},{**edit,'boxes':edit['boxes'][:1]*513},{**edit,'boxes':[{'center':[9999]*3,'size':[.0001]*3,'bevel':.00001}]}])
        before=call('inspect')
        for bad in variants:
            call('preview_mesh_edit',edit=bad,expected_error=True);call('edit_mesh',edit=bad,expected_error=True);assert call('inspect')==before
        call('edit_mesh',edit=edit,expected_revision=before['revision']-1,expected_error=True);assert call('inspect')==before
        checks.append('Invalid parameters/counts/final f32 precision/stale revisions reject atomically')
        call('edit_mesh',edit={'kind':'set_material','asset':'blocks','material':{'transmission':.2}});source=asset()
        appended={**edit,'boxes':[{'center':[3,0,0],'size':[1,1,1],'bevel':.1}],'append':True}
        before=call('inspect');p=call('preview_mesh_edit',edit=appended);assert call('inspect')==before
        call('edit_mesh',edit=appended);combined=asset();assert call('inspect')['scene']['meshes']==p['meshes']
        for key in ['positions','triangles','material_coordinates']:assert combined['geometry'][key][:len(source['geometry'][key])]==source['geometry'][key]
        assert combined['material']==source['material'] and combined['shading']==source['shading'];call('undo');assert asset()==source
        rig={'bones':[{'name':'root','parent':None,'translation':[0,0,0],'rotation':[0,0,0,1]}],'weights':[{'joints':[0,0,0,0],'weights':[1,0,0,0]} for _ in g['positions']]}
        call('edit_mesh',edit={'kind':'set_rig','asset':'blocks','rig':rig});before=call('inspect');call('edit_mesh',edit=appended,expected_error=True);assert call('inspect')==before;call('undo')
        call('edit_mesh',edit={**appended,'id':'missing'},expected_error=True)
        checks.append('Append preserves source indices/channels/appearance and rejects rigged or missing assets')
        reference=call('inspect_mesh_topology',asset='blocks')['reference'];before=call('inspect')
        call('edit_scene',edits=[{'op':'mesh','edit':edit},{'op':'mesh','edit':{'kind':'subdivide','asset':'blocks','reference':reference,'components':None}}],expected_error=True);assert call('inspect')==before
        definition={'id':'linked','profile':{'id':'outline','width':1,'height':2},'depth':.4,'segments':[2,4,2]}
        call('edit_plant',edit={'kind':'put','definition':definition,'detail_policy':'physical','animation_translation_policy':'physical'});before=call('inspect')
        for mode in [False,True]:call('edit_mesh',edit={**edit,'id':'linked','append':mode},expected_error=True);assert call('inspect')==before
        i={'id':'blocks','asset':'blocks','position':[0,2,0],'rotation':[0,0,0,1],'scale':1,'color':[.72,.66,.51]}
        instances=[{**i,'id':f'blocks-{j}'} for j in range(6)]
        call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_instance','instance':x}} for x in instances]);before=call('inspect')
        huge={**edit,'boxes':[{'center':[0,0,0],'size':[1,1,1],'bevel':.1}]*512}
        call('edit_mesh',edit=huge,expected_error=True);assert call('inspect')==before;call('undo')
        checks.append('Changed asset invalidates batch references, linked plants reject generation, draw budget preflights all instances')
        call('edit_mesh',edit={'kind':'set_material','asset':'blocks','material':{}})
        call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'put_instance','instance':i}},{'op':'set_camera','camera':{'position':[4,4,6],'target':[0,2,0],'fov_degrees':40}}])
        neutral=capture('start');assert capture('repeat')==neutral;baseline=call('inspect')['scene']
        altered=copy.deepcopy(edit);altered['boxes'][1]['bevel']=.23
        call('edit_mesh',edit=altered);assert capture('changed')!=neutral;call('undo');assert call('inspect')['scene']==baseline and capture('undo')==neutral
        call('redo');call('undo');call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'remove_instance','id':'blocks'}},{'op':'mesh','edit':{'kind':'remove_asset','id':'blocks'}}]);assert capture('removed')!=neutral;call('undo');assert capture('restored')==neutral
        checks.append('Same tick edits/remove/undo/redo invalidate geometry and restore exact captures')
        call('save_scene',path='examples/chamfer.scene.json');call('step',ticks=20);assert capture('live')==neutral
        call('save_replay',path='artifacts/chamfer.replay.json');assert call('run_replay',path='artifacts/chamfer.replay.json')['meshes_matched']
        for label,tick in [('replay',20),('prefix',0)]:
            path=f'artifacts/chamfer-{label}.png';call('capture_replay',replay_path='artifacts/chamfer.replay.json',path=path,tick=tick,width=800,height=650);assert (ROOT/path).read_bytes()==neutral
        call('load_scene',path='examples/chamfer.scene.json');assert call('inspect')['scene']==baseline and capture('loaded')==neutral
        call('reset');assert capture('reset')==neutral;call('capture',path='artifacts/chamfer-viewport.png',width=640,height=480)
        checks.append('Persistence/reset/replay/prefix and viewport preserve generated ordinary assets')
        report={'passed':True,'checks':checks,'build_id':call('inspect')['build_id'],'native_window':window};(ROOT/'artifacts/chamfer-verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({**report,'checks':len(checks)}))
    finally:c.close()


if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');ap.add_argument('--window',action='store_true');args=ap.parse_args();verify(args.binary.resolve(),args.window)
