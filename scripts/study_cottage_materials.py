#!/usr/bin/env python3
"""Reproduce material-only cottage study through the native MCP dispatcher."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
P=Path(__file__).resolve().parents[1]


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,default=P/'artifacts/cottage-materials');args=parser.parse_args()
    root=args.root.resolve()
    for d in ['examples','artifacts']:(root/d).mkdir(parents=True,exist_ok=True)
    recipe=json.loads((P/'examples/cottage-materials.recipe.json').read_text())
    src=P/'examples'/recipe['source']; (root/'examples/input.scene.json').write_bytes(src.read_bytes())
    os.environ['IGNIS_VERIFY_ROOT']=str(root)
    from verify_lab import Client
    from verify_materials import f32
    from png_pixels import rgba8
    assert hashlib.sha256(src.read_bytes()).hexdigest()==recipe['source_sha256']
    c=Client(P/'target/release/ignis'); checks=[]
    def capture(name,camera):
        c.call('set_camera',camera=camera);p=f'artifacts/{name}.png';r=c.call('capture',path=p,width=1536,height=1024)
        return (root/p).read_bytes(),r
    def edits(materials):return [dict(op='mesh',edit=dict(kind='set_material',asset=k,material=v)) for k,v in materials.items()]
    def invariants(scene):
        s=copy.deepcopy(scene);s.pop('version',None);s.pop('camera',None)
        # Native set_material updates canonical source metadata as well as output.
        for a in s['meshes']['assets']:a.pop('material',None)
        for e in s['plants']['entries']:
            e['source'].pop('material',None)
            if e.get('assembly'):
                e['assembly']['context'].pop('material',None)
                for g in e['assembly'].get('context_grafts',[]):g['source'].pop('material',None)
        return s
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='cottage-material-study',version='1')));c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/input.scene.json');before=c.call('inspect')['scene']; cameras=dict(hero=before['camera'],detail=dict(position=[4.9,4.1,8.5],target=[1.3,2.6,1.8],fov_degrees=36),front=dict(position=[0,5,19],target=[0,2.7,.5],fov_degrees=37))
        baselines={}
        for name,cam in cameras.items():
            image,_=capture('before-'+name,cam);baselines[name]=image
            if name in recipe['legacy_capture_sha256']:
                assert hashlib.sha256(image).hexdigest()==recipe['legacy_capture_sha256'][name]
        checks.append('Scene72 legacy hero/detail exact on new build; same cameras and directional light')
        c.call('set_camera',camera=cameras['hero']); state=c.call('inspect')
        materials=f32(recipe['materials']);batch=edits(materials)
        preview=c.call('preview_scene_edits',edits=batch)['scene'];assert c.call('inspect')==state
        c.call('edit_scene',edits=batch);after=c.call('inspect')['scene'];assert after==preview and invariants(before)==invariants(after)
        changed={a['id'] for a,b in zip(before['meshes']['assets'],after['meshes']['assets']) if a!=b};assert changed==set(materials)
        c.call('undo');assert invariants(c.call('inspect')['scene'])==invariants(before)
        c.call('capture',path='artifacts/undo.png',width=1536,height=1024)
        assert (root/'artifacts/undo.png').read_bytes()==baselines['hero']
        c.call('redo')
        assert c.call('inspect')['scene']==after
        outputs={};metrics={}
        for name,cam in cameras.items():
            image,meta=capture('after-'+name,cam);outputs[name]=image;assert image!=baselines[name]
            _,_,pa=rgba8(baselines[name]);_,_,pb=rgba8(image)
            changed_channels=total_error=maximum_error=0
            for i,(a,b) in enumerate(zip(pa,pb)):
                if i%4==3:continue
                difference=abs(a-b);changed_channels+=difference!=0;total_error+=difference;maximum_error=max(maximum_error,difference)
            metrics[name]=dict(changed_channels=changed_channels,mean_absolute_channel_error=total_error/(len(pa)*3/4),maximum_channel_error=maximum_error)
            assert capture('repeat-'+name,cam)[0]==image
        checks.append('Atomic pure preview/publication, unchanged geometry/instances/rig/clips/columns, repeated views and undo/redo exact')
        # Isolate second-layer color from relief and primary tuning.
        without_second={k:{a:b for a,b in v.items() if a!='secondary_pattern'} for k,v in materials.items()}
        c.call('edit_scene',edits=edits(without_second));ablated=capture('primary-only',cameras['detail'])[0];assert ablated!=outputs['detail']
        c.call('edit_scene',edits=batch);assert capture('restored',cameras['detail'])[0]==outputs['detail']
        # Material-neutral/no-pattern control must be identical across the two versions.
        neutral={a['id']:{} for a in before['meshes']['assets']}
        c.call('edit_scene',edits=edits(neutral));control=capture('neutral-after',cameras['hero'])[0]
        c.call('load_scene',path='examples/input.scene.json');c.call('edit_scene',edits=edits(neutral));assert capture('neutral-before',cameras['hero'])[0]==control
        c.call('load_scene',path='examples/input.scene.json');c.call('edit_scene',edits=batch);c.call('set_camera',camera=cameras['hero'])
        c.call('save_scene',path='examples/cottage-materials.scene.json');saved=c.call('inspect')['scene'];assert invariants(saved)==invariants(before)
        c.call('step',ticks=60);assert capture('tick60',cameras['hero'])[0]==outputs['hero']
        c.call('save_replay',path='artifacts/materials.replay.json');assert c.call('run_replay',path='artifacts/materials.replay.json')['meshes_matched']
        for name,tick in [('replay',None),('prefix',0)]:
            p=f'artifacts/{name}.png';c.call('capture_replay',replay_path='artifacts/materials.replay.json',path=p,tick=tick,width=1536,height=1024);assert (root/p).read_bytes()==outputs['hero']
        c.call('load_scene',path='examples/cottage-materials.scene.json');assert c.call('inspect')['scene']==saved
        c.call('reset');assert capture('reset',cameras['hero'])[0]==outputs['hero']
        c.call('capture',path='artifacts/viewport.png',width=800,height=600);assert capture('viewport-restored',cameras['hero'])[0]==outputs['hero']
        checks.append('Secondary color ablation differs, neutral control exact, scene73 save/load/reset, Replay77/prefix and viewport exact')
        report=dict(passed=True,build_id=c.call('inspect')['build_id'],source_sha256=hashlib.sha256(src.read_bytes()).hexdigest(),recipe_sha256=hashlib.sha256((P/'examples/cottage-materials.recipe.json').read_bytes()).hexdigest(),checks=checks,assets=len(saved['meshes']['assets']),instances=len(saved['meshes']['instances']),vertices=sum(len(a['geometry']['positions']) for a in saved['meshes']['assets']),triangles=sum(len(a['geometry']['triangles']) for a in saved['meshes']['assets']),draw_triangles=meta['mesh_triangle_count'],changed_assets=sorted(changed),metrics=metrics,limitations=['Structural silhouette and entry/bay/gable/columns unchanged','Opaque glass and dielectric bronze unchanged; no PBR metal or transparency','No point light for lantern','Shared rest domain; grouped pieces lack individually oriented grain'])
        (root/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:c.close()


if __name__=='__main__':main()
