#!/usr/bin/env python3
"""Native in-place part editing on the existing cottage, with exact preservation proof."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
from cottage_composition import lantern_edits
P=Path(__file__).resolve().parents[1];R=P/'artifacts/cottage-composition'
VIEWS={'hero':None,'lantern':{'position':[4.9,3.7,7.1],'target':[2.35,2.65,2.55],'fov_degrees':23}}


def main():
    for d in ['examples','artifacts']:(R/d).mkdir(parents=True,exist_ok=True)
    ap=argparse.ArgumentParser();ap.add_argument('--before',type=Path,default=R/'examples/before.scene.json' if (R/'examples/before.scene.json').exists() else P/'examples/elven-cottage.scene.json');ap.add_argument('--after',type=Path);args=ap.parse_args()
    source=args.before;raw=source.read_bytes()
    (R/'examples/before.scene.json').write_bytes(raw)
    os.environ['IGNIS_VERIFY_ROOT']=str(R)
    from verify_lab import Client
    c=Client(P/'target/release/ignis');call=c.call
    def capture(name,cam=None):
        if cam:call('set_camera',camera=cam)
        path=f'artifacts/{name}.png';meta=call('capture',path=path,width=1200,height=900);return (R/path).read_bytes(),meta
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'cottage-composition','version':'1'}});c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/before.scene.json');before=call('inspect')['scene'];camera=before['camera'];wall_z=2.1;offset=[-.27,0,0]
        for key,cam in VIEWS.items():capture('before-'+key,cam or camera)
        call('set_camera',camera=camera)
        edits,selected=lantern_edits(call,wall_z,offset);original=call('inspect');predicted=call('preview_scene_edits',edits=edits)['scene'];assert call('inspect')==original
        call('edit_scene',edits=edits);after=call('inspect')['scene'];assert after==predicted
        aa={a['id']:a for a in before['meshes']['assets']};bb={a['id']:a for a in after['meshes']['assets']};changed=sorted(k for k in aa if aa[k]!=bb[k]);assert changed==['bronze','flowers'],changed
        details={}
        for key in changed:
            a=aa[key];b=bb[key];ga=a['geometry'];gb=b['geometry'];p=copy.deepcopy(a);q=copy.deepcopy(b);p.pop('geometry');q.pop('geometry');assert p==q
            assert ga['triangles']==gb['triangles']
            assert gb['material_coordinates']==ga.get('material_coordinates',ga['positions'])
            info=call('inspect_mesh_islands',asset=key);old_ids={s['id'] for s in selected[key]};vertices=set()
            # Connectivity comes from Ignis; face traversal here only verifies the submitted result.
            parent=list(range(len(ga['positions'])))
            def root(i):
                while parent[i]!=i:i=parent[i]
                return i
            for t in ga['triangles']:
                for v in t[1:]:parent[root(v)]=root(t[0])
            roots={root(ga['triangles'][i][0]) for i in old_ids}
            vertices={i for i in range(len(parent)) if root(i) in roots}
            actually={i for i,(x,y) in enumerate(zip(ga['positions'],gb['positions'])) if x!=y};assert actually==vertices
            for i in vertices:
                assert all(abs(gb['positions'][i][axis]-(ga['positions'][i][axis]+offset[axis]))<3e-7 for axis in range(3))
            assert old_ids.issubset({s['id'] for s in info['topology']['islands']})
            details[key]={'selected_islands':len(old_ids),'moved_vertices':len(vertices),'total_vertices':len(ga['positions'])}
        rest_a=copy.deepcopy(before);rest_b=copy.deepcopy(after);rest_a['meshes']['assets']=[];rest_b['meshes']['assets']=[];assert rest_a==rest_b
        heroes={}
        for key,cam in VIEWS.items():heroes[key]=capture('after-'+key,cam or camera)[0];assert heroes[key]!=(R/f'artifacts/before-{key}.png').read_bytes()
        call('set_camera',camera=camera);assert call('inspect')['scene']==after
        hero=capture('after-repeat')[0];assert hero==heroes['hero']
        # Reload baseline and rerun one batch to isolate undo from capture cameras.
        call('load_scene',path='examples/before.scene.json');edits,_=lantern_edits(call,wall_z,offset);call('edit_scene',edits=edits);assert call('inspect')['scene']==after
        call('undo');assert call('inspect')['scene']==before;call('redo');assert call('inspect')['scene']==after and capture('after-redo')[0]==hero
        call('save_scene',path='examples/after.scene.json');call('step',ticks=60);assert capture('after-tick60')[0]==hero
        call('save_replay',path='artifacts/composition.replay.json');assert call('run_replay',path='artifacts/composition.replay.json')['meshes_matched']
        for tick in [0,60]:
            path=f'artifacts/replay-{tick}.png';call('capture_replay',replay_path='artifacts/composition.replay.json',path=path,tick=tick,width=1200,height=900);assert (R/path).read_bytes()==hero
        call('load_scene',path='examples/after.scene.json');assert call('inspect')['scene']==after and capture('after-load')[0]==hero
        call('reset');assert capture('after-reset')[0]==hero
        if args.after:
            (R/'examples/published.scene.json').write_bytes(args.after.read_bytes());call('load_scene',path='examples/published.scene.json')
            assert call('inspect')['scene']==after and capture('published')[0]==hero
        assert source.read_bytes()==raw
        report={'passed':True,'build_id':call('inspect')['build_id'],'before_sha256':hashlib.sha256(raw).hexdigest(),'offset':offset,'selection':selected,'details':details,'changed_assets':changed,'counts':{'assets':len(bb),'instances':len(after['meshes']['instances']),'vertices':sum(len(a['geometry']['positions']) for a in bb.values()),'triangles':sum(len(a['geometry']['triangles']) for a in bb.values())},'checks':['preview_pure','exact_selected_vertices','channels_rest_scene_preserved','unchanged_counts','visual_change_repeat','one_batch_undo_redo','save_load_reset','static_tick_replay_prefix']+(['published_scene_matches'] if args.after else []),'limitations':['Asset-local rest edit affects every instance; bones are not transformed','Selection recipe is cottage-specific; native transform is generic','No persistent semantic parts, budget reduction or artistic acceptance']}
        (R/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='selection'}))
    finally:c.close()


if __name__=='__main__':main()
