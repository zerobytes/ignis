#!/usr/bin/env python3
"""In-place island composition through the shared MCP dispatcher."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c=Client(binary,window);call=c.call;checks=[]
    def inspect():return call('inspect')
    def asset():return call('inspect_mesh',asset='parts')['asset']
    def capture(name):
        path=f'artifacts/island-transform-{name}.png';call('capture',path=path,width=800,height=650);return (ROOT/path).read_bytes()
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'island-transform-verifier','version':'1'}});c.request('notifications/initialized',notification=True)
        assert 'transform_islands' in json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        call('generate',recipe={'seed':7,'width':20,'depth':20,'obstacles':0})
        call('edit_mesh',edit={'kind':'chamfer_boxes','id':'parts','boxes':[{'center':[-1,0,0],'size':[1,1,1],'bevel':.1},{'center':[1,0,0],'size':[1,1,1],'bevel':.1}]})
        # Implicit material domain and animation exercise channel preservation.
        call('edit_mesh',edit={'kind':'set_material_coordinates','asset':'parts','coordinates':None})
        rig={'bones':[{'name':'root','parent':None,'translation':[0,0,0],'rotation':[0,0,0,1]}],'weights':[{'joints':[0,0,0,0],'weights':[1,0,0,0]} for _ in asset()['geometry']['positions']]}
        clip={'id':'move','duration_ticks':120,'tracks':[{'bone':0,'keys':[{'tick':0,'translation':[0,0,0],'rotation':[0,0,0,1]},{'tick':60,'translation':[0,.4,0],'rotation':[0,0,0,1]},{'tick':120,'translation':[0,0,0],'rotation':[0,0,0,1]}]}]}
        instance={'id':'one','asset':'parts','position':[0,2,0],'rotation':[0,0,0,1],'scale':1,'color':[.7,.6,.3],'playback':{'clip':'move','offset_ticks':0,'looping':True}}
        call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'set_rig','asset':'parts','rig':rig}},{'op':'mesh','edit':{'kind':'put_clip','asset':'parts','clip':clip}},{'op':'mesh','edit':{'kind':'put_instance','instance':instance}},{'op':'set_camera','camera':{'position':[4,5,7],'target':[0,2,0],'fov_degrees':42}}])
        original=asset();base_scene=inspect()['scene'];base=capture('before');assert capture('repeat')==base
        island_info=call('inspect_mesh_islands',asset='parts');reference=island_info['reference'];selected=island_info['topology']['islands'][1]['id']
        edit={'kind':'transform_islands','asset':'parts','reference':reference,'islands':[selected],'transform':{'pivot':[1,0,0],'translation':[-.5,.7,0],'rotation':[0,0,0,1],'scale':.8}}
        before=inspect();p=call('preview_mesh_edit',edit=edit);assert inspect()==before
        call('edit_mesh',edit=edit);assert inspect()['scene']['meshes']==p['meshes'];changed=asset();after=capture('after');assert after!=base
        assert changed['geometry']['positions'][:24]==original['geometry']['positions'][:24]
        assert changed['geometry']['triangles']==original['geometry']['triangles']
        assert changed['geometry']['material_coordinates']==original['geometry']['positions']
        for k in ['rig','clips','material','shading']:assert changed[k]==original[k]
        assert inspect()['scene']['meshes']['instances']==base_scene['meshes']['instances']
        checks.append('Schema/preview/commit transform selected complete island without extra slots or indices and freeze implicit material domain')
        before=inspect();call('edit_mesh',edit=edit,expected_error=True);assert inspect()==before
        edit['reference']=call('inspect_mesh_islands',asset='parts')['reference']
        for patch in [{'scale':0},{'scale':101},{'rotation':[0,0,0,0]},{'translation':[10000]*3},{'pivot':[10001,0,0]}]:
            bad=copy.deepcopy(edit);bad['transform'].update(patch)
            for op in ['preview_mesh_edit','edit_mesh']:call(op,edit=bad,expected_error=True);assert inspect()==before
        for ids in [[],[selected,selected],[99999]]:
            call('edit_mesh',edit={**edit,'islands':ids},expected_error=True);assert inspect()==before
        call('edit_scene',edits=[{'op':'mesh','edit':edit},{'op':'mesh','edit':edit}],expected_error=True);assert inspect()==before
        checks.append('Stale revisions/invalid transforms/selections and earlier same-asset batch changes reject with full rollback')
        poses={t:call('inspect_mesh_pose',instance='one',tick=t)['pose'] for t in [0,60]}
        call('undo');assert inspect()['scene']==base_scene and capture('undo')==base
        call('redo');assert asset()==changed and capture('redo')==after
        for t in [0,60]:assert call('inspect_mesh_pose',instance='one',tick=t)['pose']==poses[t]
        checks.append('Rig/weights/clips survive rest editing; undo/redo restore exact geometry, posed queries and GPU captures')
        definition={'id':'linked','profile':{'id':'outline','width':1,'height':2},'depth':.4,'segments':[2,4,2]}
        call('edit_plant',edit={'kind':'put','definition':definition,'detail_policy':'physical','animation_translation_policy':'physical'})
        info=call('inspect_mesh_islands',asset='linked');before=inspect()
        linked={**edit,'asset':'linked','reference':info['reference'],'islands':[info['topology']['islands'][0]['id']]}
        for op in ['preview_mesh_edit','edit_mesh']:call(op,edit=linked,expected_error=True);assert inspect()==before
        checks.append('Linked parametric sources require explicit detach before island transformation')
        scene=inspect()['scene'];call('save_scene',path='examples/island-transform.scene.json');call('step',ticks=60);animated=capture('animated');assert animated!=after
        call('save_replay',path='artifacts/island-transform.replay.json');assert call('run_replay',path='artifacts/island-transform.replay.json')['meshes_matched']
        for tick,pixels in [(0,after),(60,animated)]:
            path=f'artifacts/island-transform-replay-{tick}.png';call('capture_replay',replay_path='artifacts/island-transform.replay.json',path=path,tick=tick,width=800,height=650);assert (ROOT/path).read_bytes()==pixels
        call('load_scene',path='examples/island-transform.scene.json');assert inspect()['scene']==scene and capture('load')==after
        call('reset');assert capture('reset')==after
        call('capture',path='artifacts/island-transform-viewport.png',width=640,height=480)
        checks.append('Save/load/reset/replay/prefix/viewport preserve edited rest geometry and animation')
        result={'passed':True,'checks':checks,'build_id':inspect()['build_id'],'native_window':window};(ROOT/'artifacts/island-transform-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({**result,'checks':len(checks)}))
    finally:c.close()


if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');ap.add_argument('--window',action='store_true');a=ap.parse_args();verify(a.binary.resolve(),a.window)
