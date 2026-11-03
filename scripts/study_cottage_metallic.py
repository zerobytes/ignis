#!/usr/bin/env python3
"""Material-only metal response study on the same native cottage, through MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
P=Path(__file__).resolve().parents[1]


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,default=P/'artifacts/cottage-metallic');args=parser.parse_args();root=args.root.resolve()
    for d in ['examples','artifacts']:(root/d).mkdir(parents=True,exist_ok=True)
    recipe=json.loads((P/'examples/cottage-metallic.recipe.json').read_text());source=P/'examples'/recipe['source'];base_recipe=P/'examples'/recipe['base_material_recipe']
    assert hashlib.sha256(source.read_bytes()).hexdigest()==recipe['source_sha256']
    assert hashlib.sha256(base_recipe.read_bytes()).hexdigest()==recipe['base_recipe_sha256']
    (root/'examples/input.scene.json').write_bytes(source.read_bytes());os.environ['IGNIS_VERIFY_ROOT']=str(root)
    from verify_lab import Client
    from verify_materials import f32
    from png_pixels import rgba8
    c=Client(P/'target/release/ignis');checks=[]
    def capture(name,camera=None):
        if camera is not None:c.call('set_camera',camera=camera)
        p=f'artifacts/{name}.png';meta=c.call('capture',path=p,width=1536,height=1024);return (root/p).read_bytes(),meta
    def set_material(value):c.call('edit_mesh',edit=dict(kind='set_material',asset=recipe['asset'],material=f32(value)))
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='cottage-metal-study',version='1')));c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/input.scene.json')
        c.call('edit_scene',edits=[dict(op='mesh',edit=dict(kind='set_material',asset=k,material=f32(v))) for k,v in json.loads(base_recipe.read_text())['materials'].items()])
        baseline=c.call('inspect')['scene'];base_material=next(a['material'] for a in baseline['meshes']['assets'] if a['id']==recipe['asset'])
        cameras=dict(hero=baseline['camera'],detail=dict(position=[4.9,4.1,8.5],target=[1.3,2.6,1.8],fov_degrees=36),front=dict(position=[0,5,19],target=[0,2.7,.5],fov_degrees=37),ornament=dict(position=[3.35,3.25,5.25],target=[2.8,2.45,2.75],fov_degrees=38))
        before_pixels={}
        for name,camera in cameras.items():
            image,_=capture('before-'+name,camera);before_pixels[name]=image
            if name in recipe['legacy_capture_sha256']:assert hashlib.sha256(image).hexdigest()==recipe['legacy_capture_sha256'][name]
        checks.append('Source/recipe hashes fixed; legacy e0a95a4b90e7fb39 hero/detail/front pixels exact on new build')
        c.call('set_camera',camera=cameras['hero']);before=c.call('inspect');candidate=f32(recipe['material']);preview=c.call('preview_mesh_edit',edit=dict(kind='set_material',asset=recipe['asset'],material=candidate));assert c.call('inspect')==before
        set_material(candidate);after=c.call('inspect')['scene'];assert after['meshes']==preview['meshes']
        expected=copy.deepcopy(before['scene']);next(a for a in expected['meshes']['assets'] if a['id']==recipe['asset'])['material']=candidate;assert after==expected
        c.call('undo');assert c.call('inspect')['scene']==before['scene'] and capture('undo')[0]==before_pixels['hero']
        c.call('redo');assert c.call('inspect')['scene']==after
        outputs={};metrics={}
        for name,camera in cameras.items():
            image,meta=capture('after-'+name,camera);outputs[name]=image;assert image!=before_pixels[name] and capture('repeat-'+name)[0]==image
            w,h,pa=rgba8(before_pixels[name]);_,_,pb=rgba8(image);changed=total=maximum=0
            for pixel in range(w*h):
                d=[abs(pa[pixel*4+k]-pb[pixel*4+k]) for k in range(3)];changed+=any(d);total+=sum(d);maximum=max(maximum,max(d))
            metrics[name]=dict(changed_pixels=changed,mean_absolute_channel_error=total/(w*h*3),maximum_channel_error=maximum)
        checks.append('Only bronze Material changes; geometry/source/instances/colors/plant columns/rig/clips fixed, preview/undo/redo/repeated views exact')
        # Show how absence of environment reflection constrains the metal result.
        ambient=copy.deepcopy(after['light']);ambient['ambient']=1;c.call('set_light',light=ambient);metal_ambient=capture('ambient-metal',cameras['ornament'])[0]
        set_material(base_material);assert capture('ambient-dielectric')[0]==metal_ambient
        c.call('set_light',light=after['light']);set_material(candidate)
        for roughness in [.3,.6,.9]:
            changed=copy.deepcopy(candidate);changed['reflectance']['roughness']=roughness;set_material(changed);assert capture('roughness-'+str(roughness),cameras['ornament'])[0]!=outputs['ornament']
        set_material(candidate);assert capture('restored',cameras['hero'])[0]==outputs['hero']
        c.call('save_scene',path='examples/cottage-metallic.scene.json');saved=c.call('inspect')['scene'];assert saved==after and saved['version']==SCENE_VERSION
        c.call('step',ticks=60);assert capture('tick60')[0]==outputs['hero']
        c.call('save_replay',path='artifacts/metallic.replay.json');assert c.call('run_replay',path='artifacts/metallic.replay.json')['meshes_matched']
        for label,tick in [('replay',None),('prefix',0)]:
            p=f'artifacts/{label}.png';c.call('capture_replay',replay_path='artifacts/metallic.replay.json',path=p,tick=tick,width=1536,height=1024);assert (root/p).read_bytes()==outputs['hero']
        c.call('load_scene',path='examples/cottage-metallic.scene.json');assert c.call('inspect')['scene']==saved
        c.call('reset');assert capture('reset')[0]==outputs['hero']
        c.call('capture',path='artifacts/viewport.png',width=800,height=600);assert capture('viewport-restored')[0]==outputs['hero']
        checks.append('Roughness controls distinct; all-ambient metal/dielectric exact exposes missing IBL; Scene74 load/reset,Replay78/prefix/viewport exact')
        report=dict(passed=True,build_id=c.call('inspect')['build_id'],source_sha256=recipe['source_sha256'],recipe_sha256=hashlib.sha256((P/'examples/cottage-metallic.recipe.json').read_bytes()).hexdigest(),checks=checks,metrics=metrics,assets=len(saved['meshes']['assets']),instances=len(saved['meshes']['instances']),vertices=sum(len(a['geometry']['positions']) for a in saved['meshes']['assets']),triangles=sum(len(a['geometry']['triangles']) for a in saved['meshes']['assets']),draw_triangles=meta['mesh_triangle_count'],limitations=['No environment reflection/IBL: constant ambient remains','UNORM highlight clipping and specular aliasing remain possible','Uniform per-asset metallic/roughness; no oxidation mask','Architecture of original cottage unchanged; composition/sweep handled separately'])
        (root/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:c.close()


if __name__=='__main__':main()
