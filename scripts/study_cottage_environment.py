#!/usr/bin/env python3
"""Reproduce native distant environment on the maintained cottage, through MCP."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
P=Path(__file__).resolve().parents[1]


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--root',type=Path,default=P/'artifacts/cottage-environment');args=parser.parse_args();root=args.root.resolve()
    for d in ['examples','artifacts']:(root/d).mkdir(parents=True,exist_ok=True)
    recipe=json.loads((P/'examples/cottage-environment.recipe.json').read_text());source=P/'examples'/recipe['source'];materials_path=P/'examples/cottage-materials.recipe.json';metal_path=P/'examples/cottage-metallic.recipe.json'
    assert hashlib.sha256(source.read_bytes()).hexdigest()==recipe['source_sha256']
    assert hashlib.sha256(metal_path.read_bytes()).hexdigest()==recipe['metal_recipe_sha256']
    (root/'examples/input.scene.json').write_bytes(source.read_bytes());os.environ['IGNIS_VERIFY_ROOT']=str(root)
    from verify_lab import Client
    from verify_materials import f32
    from png_pixels import rgba8
    c=Client(P/'target/release/ignis');checks=[]
    def capture(name,camera=None):
        if camera is not None:c.call('set_camera',camera=camera)
        p=f'artifacts/{name}.png';meta=c.call('capture',path=p,width=1536,height=1024);return (root/p).read_bytes(),meta
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='cottage-environment-study',version='1')));c.request('notifications/initialized',notification=True)
        c.call('load_scene',path='examples/input.scene.json');materials=json.loads(materials_path.read_text())['materials'];metal=json.loads(metal_path.read_text());materials[metal['asset']]=metal['material']
        c.call('edit_scene',edits=[dict(op='mesh',edit=dict(kind='set_material',asset=k,material=f32(v))) for k,v in materials.items()]);before=c.call('inspect')['scene']
        cameras=dict(hero=before['camera'],detail=dict(position=[4.9,4.1,8.5],target=[1.3,2.6,1.8],fov_degrees=36),front=dict(position=[0,5,19],target=[0,2.7,.5],fov_degrees=37),ornament=dict(position=[3.35,3.25,5.25],target=[2.8,2.45,2.75],fov_degrees=38))
        baselines={}
        for name,camera in cameras.items():
            image,meta=capture('before-'+name,camera);baselines[name]=image;assert hashlib.sha256(image).hexdigest()==recipe['legacy_capture_sha256'][name]
            assert meta['adapter']['environment_preparation']['probe_bakes']==0
        checks.append('Legacy993076c35e8162d4 hero/detail/front/ornament byte-exact and no preprocessing with environment absent')
        c.call('set_camera',camera=cameras['hero']);state=c.call('inspect');light=f32(dict(before['light'],**recipe['lighting']))
        # Preserve the exact authored direction, shadow and effective direct factor.
        assert light['direction']==before['light']['direction'] and light.get('shadow')==before['light'].get('shadow')
        assert abs((1-light['ambient'])*light['direct_intensity']-(1-before['light']['ambient']))<1e-6
        edits=[dict(op='set_light',light=light)];predicted=c.call('preview_scene_edits',edits=edits)['scene'];assert c.call('inspect')==state
        c.call('edit_scene',edits=edits);after=c.call('inspect')['scene'];expected=copy.deepcopy(state['scene']);expected['light']=light;assert after==predicted==expected
        hero,meta=capture('after-hero');stats=meta['adapter']['environment_preparation'];assert stats['probe_bakes']==stats['lut_bakes']==1
        c.call('undo');assert c.call('inspect')['scene']==state['scene'] and capture('undo')[0]==baselines['hero']
        c.call('redo');assert c.call('inspect')['scene']==after and capture('redo')[0]==hero
        outputs={};metrics={}
        for name,camera in cameras.items():
            image,meta=capture('after-'+name,camera);outputs[name]=image;assert image!=baselines[name] and capture('repeat-'+name)[0]==image
            assert meta['adapter']['environment_preparation']==stats
            w,h,pa=rgba8(baselines[name]);_,_,pb=rgba8(image);changed=total=maximum=0
            for pixel in range(w*h):
                d=[abs(pa[pixel*4+k]-pb[pixel*4+k]) for k in range(3)];changed+=any(d);total+=sum(d);maximum=max(maximum,max(d))
            metrics[name]=dict(changed_pixels=changed,mean_absolute_channel_error=total/(w*h*3),maximum_channel_error=maximum)
        checks.append('Only Light changes, direction/shadow/effective sunlight fixed; all geometry/sources/columns/instances/colors/rig/clips intact; preview/undo/redo/repeats exact and one probe/LUT bake')
        without_env=copy.deepcopy(light);without_env.pop('environment');c.call('set_light',light=without_env);sun_only=capture('sun-only',cameras['ornament'])[0];assert sun_only!=outputs['ornament']
        c.call('set_light',light=light);assert capture('environment-restored')[0]==outputs['ornament']
        no_sun=dict(light,direct_intensity=0);c.call('set_light',light=no_sun);assert capture('environment-only')[0]!=outputs['ornament']
        c.call('set_light',light=light)
        nonmetal=copy.deepcopy(metal['material']);nonmetal['reflectance'].pop('metallic',None);c.call('edit_mesh',edit=dict(kind='set_material',asset=metal['asset'],material=nonmetal));assert capture('nonmetal-control')[0]!=outputs['ornament']
        c.call('edit_mesh',edit=dict(kind='set_material',asset=metal['asset'],material=metal['material']));assert capture('metal-restored')[0]==outputs['ornament']
        dim=copy.deepcopy(light);dim['environment']['intensity']*=.5;c.call('set_light',light=dim);dim_image,meta=capture('half-intensity');assert dim_image!=outputs['ornament'] and meta['adapter']['environment_preparation']==stats
        rotated=copy.deepcopy(light);rotated['environment']['axis']=[1,0,0];c.call('set_light',light=rotated);rotated_image,meta=capture('axis-rotated');assert rotated_image!=outputs['ornament'] and meta['adapter']['environment_preparation']['probe_bakes']==2
        c.call('set_light',light=light);assert capture('axis-restored')[0]==outputs['ornament']
        c.call('set_camera',camera=cameras['hero']);c.call('save_scene',path='examples/cottage-environment.scene.json');saved=c.call('inspect')['scene'];assert saved==after
        c.call('step',ticks=60);static,meta=capture('tick60');assert static==hero
        c.call('save_replay',path='artifacts/environment.replay.json');assert c.call('run_replay',path='artifacts/environment.replay.json')['meshes_matched']
        for name,tick in [('replay',None),('prefix',0)]:
            p=f'artifacts/{name}.png';c.call('capture_replay',replay_path='artifacts/environment.replay.json',path=p,tick=tick,width=1536,height=1024);assert (root/p).read_bytes()==hero
        c.call('load_scene',path='examples/cottage-environment.scene.json');assert c.call('inspect')['scene']==saved and capture('loaded')[0]==hero
        c.call('reset');assert capture('reset')[0]==hero
        c.call('capture',path='artifacts/viewport.png',width=800,height=600);_,meta=capture('viewport-restored');assert (root/'artifacts/viewport-restored.png').read_bytes()==hero
        final_stats=meta['adapter']['environment_preparation'];assert final_stats['probe_bakes']==3 and final_stats['lut_bakes']==1
        checks.append('Sun/environment and metal ablations distinct, intensity reuses cache, axis rebakes once; scene76/load/reset/tick60/Replay80/prefix/viewport exact, final3probe/1LUT bakes')
        report=dict(passed=True,build_id=c.call('inspect')['build_id'],source_sha256=recipe['source_sha256'],recipe_sha256=hashlib.sha256((P/'examples/cottage-environment.recipe.json').read_bytes()).hexdigest(),checks=checks,lighting=light,preparation=final_stats,metrics=metrics,assets=len(saved['meshes']['assets']),instances=len(saved['meshes']['instances']),vertices=sum(len(a['geometry']['positions']) for a in saved['meshes']['assets']),triangles=sum(len(a['geometry']['triangles']) for a in saved['meshes']['assets']),draw_triangles=meta['mesh_triangle_count'],limitations=['Distant procedural3color environment, no reflected scene objects or local occlusion','Finite GGX/LUT8bit cube quantization and split-sum v=n approximation','No HDR output/tonemapping/specular antialiasing','Original cottage geometry unchanged; architecture/sweep is separate'])
        (root/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:c.close()


if __name__=='__main__':main()
