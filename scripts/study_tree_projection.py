#!/usr/bin/env python3
"""Native persistent opt-in projection on the same material-finished tree."""
import argparse
import copy
import json
import os
from pathlib import Path
from demo_tree_material_frames import CAMS, ID
PROJECT = Path(__file__).resolve().parents[1]
WOOD = {'wood', ID, 'secondary-06', 'secondary-twigs'}

def main():
    p=argparse.ArgumentParser()
    p.add_argument('--input',type=Path,default=PROJECT/'artifacts/tree-material-recipe/examples/combined.scene.json')
    args=p.parse_args()
    root=PROJECT/'artifacts/tree-projection'
    (root/'examples').mkdir(parents=True,exist_ok=True);(root/'artifacts').mkdir(exist_ok=True)
    (root/'examples/input.scene.json').write_bytes(args.input.read_bytes())
    os.environ['IGNIS_VERIFY_ROOT']=str(root)
    from verify_lab import Client
    c=Client(PROJECT/'target/release/ignis',False);call=c.call
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-projection','version':'1'}})
        c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json');before=call('inspect')['scene']
        call('save_scene',path='examples/planar.scene.json')
        edits=[]
        for asset in before['meshes']['assets']:
            if asset['id'] in WOOD and asset.get('material',{}).get('detail'):
                m=copy.deepcopy(asset['material']);m['detail']['triplanar']={'sharpness':2}
                edits.append({'op':'mesh','edit':{'kind':'set_material','asset':asset['id'],'material':m}})
        assert edits
        state=call('inspect');pred=call('preview_scene_edits',edits=edits)['scene'];assert call('inspect')==state
        call('edit_scene',edits=edits);after=call('inspect')['scene'];assert after==pred
        for a,b in zip(before['meshes']['assets'],after['meshes']['assets']):
            for k in ['id','geometry','rig','clips','shading']:assert a.get(k)==b.get(k)
        assert before['meshes']['instances']==after['meshes']['instances']
        call('save_scene',path='examples/triplanar.scene.json')
        call('undo');assert call('inspect')['scene']==before
        call('redo');assert call('inspect')['scene']==after
        images={}
        for mode in ['planar','triplanar']:
            for name,cam in dict(CAMS,whole=before['camera']).items():
                for neutral in [False,True]:
                    call('load_scene',path=f'examples/{mode}.scene.json')
                    if neutral:
                        call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':{}}} for a in after['meshes']['assets'] if a['id'] in WOOD])
                    call('set_camera',camera=cam);call('step',ticks=60)
                    path=f'artifacts/{mode}-'+('neutral-' if neutral else '')+name+'.png'
                    call('capture',path=path,width=1100,height=1100);images[(mode,name,neutral)]=(root/path).read_bytes()
        for name in dict(CAMS,whole=None):assert images['planar',name,True]==images['triplanar',name,True]
        assert any(images['planar',name,False]!=images['triplanar',name,False] for name in CAMS)
        call('load_scene',path='examples/triplanar.scene.json');assert call('inspect')['scene']==after
        call('step',ticks=60);call('capture',path='artifacts/live.png',width=800,height=800)
        call('save_replay',path='artifacts/tree.replay.json');assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/tree.replay.json',path='artifacts/replay.png',tick=60,width=800,height=800)
        assert (root/'artifacts/live.png').read_bytes()==(root/'artifacts/replay.png').read_bytes()
        report={'passed':True,'build_id':call('inspect')['build_id'],'geometry_rig_clips_instances_exact':True,'four_neutral_views_exact':True,'canonical_tree_unchanged':True,'visual_realism_accepted':False}
        (root/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
    finally:c.close()
if __name__=='__main__':main()
