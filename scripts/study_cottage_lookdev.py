#!/usr/bin/env python3
"""Integrate native cottage composition with materials/probe, preserving geometry."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
P=Path(__file__).resolve().parents[1]


def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,default=P/'artifacts/cottage-lookdev');args=p.parse_args();root=args.root.resolve()
    for name in ['examples','artifacts']:(root/name).mkdir(parents=True,exist_ok=True)
    recipe=json.loads((P/'examples/cottage-lookdev.recipe.json').read_text());source=P/'examples'/recipe['source'];assert hashlib.sha256(source.read_bytes()).hexdigest()==recipe['source_sha256']
    (root/'examples/input.scene.json').write_bytes(source.read_bytes());os.environ['IGNIS_VERIFY_ROOT']=str(root)
    from verify_lab import Client
    from verify_materials import f32
    c=Client(P/'target/release/ignis');checks=[]
    def capture(name,camera=None,width=1536,height=1024):
        if camera is not None:c.call('set_camera',camera=camera)
        path=f'artifacts/{name}.png';meta=c.call('capture',path=path,width=width,height=height);return (root/path).read_bytes(),meta
    def invariant(scene):
        s=copy.deepcopy(scene)
        for k in ['version','camera','light']:s.pop(k,None)
        for a in s['meshes']['assets']:a.pop('material',None)
        for e in s['plants']['entries']:
            e['source'].pop('material',None)
            if e.get('assembly'):
                e['assembly']['context'].pop('material',None)
                for g in e['assembly'].get('context_grafts',[]):g['source'].pop('material',None)
        return s
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='integrated-cottage-study',version='1')));c.request('notifications/initialized',notification=True)
        schema=json.dumps(c.request('tools/list'));assert all(k in schema for k in ['plan_mesh_sweep','environment','secondary_pattern','metallic'])
        c.call('load_scene',path='examples/input.scene.json');original=c.call('inspect')['scene'];cameras=f32(dict(hero=original['camera'],**recipe['cameras']))
        before={}
        for name,cam in cameras.items():before[name]=capture('before-'+name,cam)[0]
        material_recipe=json.loads((P/'examples/cottage-materials.recipe.json').read_text())['materials'];metal=json.loads((P/'examples/cottage-metallic.recipe.json').read_text());material_recipe[metal['asset']]=metal['material']
        assert set(material_recipe)<=set(a['id'] for a in original['meshes']['assets'])
        c.call('set_camera',camera=cameras['hero']);state=c.call('inspect');edits=[dict(op='mesh',edit=dict(kind='set_material',asset=k,material=f32(v))) for k,v in material_recipe.items()]
        preview=c.call('preview_scene_edits',edits=edits)['scene'];assert c.call('inspect')==state
        c.call('edit_scene',edits=edits);assert c.call('inspect')['scene']==preview and invariant(preview)==invariant(original)
        material_pixels={}
        for name,cam in cameras.items():
            material_pixels[name]=capture('materials-'+name,cam)[0];assert material_pixels[name]!=before[name]
        checks.append('Unified MCP schema includes sweep/planner/material layers/metal/environment; same native structural source loads, material preview pure/equal, all geometry/definitions/sources/instances/colors/rig preserved')
        c.call('set_camera',camera=cameras['hero']);state=c.call('inspect');light=f32(dict(original['light'],**recipe['lighting']));edits=[dict(op='set_light',light=light)]
        assert light['direction']==original['light']['direction'] and light.get('shadow')==original['light'].get('shadow')
        assert abs((1-light['ambient'])*light['direct_intensity']-(1-original['light']['ambient']))<1e-6
        preview=c.call('preview_scene_edits',edits=edits)['scene'];assert c.call('inspect')==state
        c.call('edit_scene',edits=edits);after=c.call('inspect')['scene'];assert after==preview and invariant(after)==invariant(original)
        hero,meta=capture('after-hero');stats=meta['adapter']['environment_preparation'];assert stats['probe_bakes']==stats['lut_bakes']==1
        c.call('undo');assert c.call('inspect')['scene']==state['scene'] and capture('undo')[0]==material_pixels['hero']
        c.call('redo');assert c.call('inspect')['scene']==after and capture('redo')[0]==hero
        outputs={}
        for name,cam in cameras.items():
            image,meta=capture('after-'+name,cam);outputs[name]=image;assert image!=material_pixels[name] and capture('repeat-'+name)[0]==image
            assert meta['adapter']['environment_preparation']==stats
        checks.append('Environment preview/history and four repeated fixed views exact; sunlight direction/shadow/effective strength preserved, one probe/LUT bake, no geometry or slot growth')
        no_env=copy.deepcopy(light);no_env.pop('environment');c.call('set_light',light=no_env);assert capture('sun-only',cameras['ornament'])[0]!=outputs['ornament']
        c.call('set_light',light=light);assert capture('probe-restored')[0]==outputs['ornament']
        no_sun=dict(light,direct_intensity=0);c.call('set_light',light=no_sun);assert capture('environment-only')[0]!=outputs['ornament'];c.call('set_light',light=light)
        neutral={a['id']:{} for a in original['meshes']['assets']};c.call('edit_scene',edits=[dict(op='mesh',edit=dict(kind='set_material',asset=k,material=v)) for k,v in neutral.items()]);capture('material-neutral',cameras['hero'])
        c.call('edit_scene',edits=[dict(op='mesh',edit=dict(kind='set_material',asset=k,material=f32(v))) for k,v in material_recipe.items()]+[dict(op='mesh',edit=dict(kind='set_material',asset=a['id'],material=a['material'])) for a in original['meshes']['assets'] if a['id'] not in material_recipe]);assert capture('materials-restored')[0]==hero
        c.call('set_camera',camera=cameras['hero']);c.call('save_scene',path='examples/cottage-lookdev.scene.json');saved=c.call('inspect')['scene'];assert saved==after and saved['version']==SCENE_VERSION
        c.call('step',ticks=60);assert capture('tick60')[0]==hero
        c.call('save_replay',path='artifacts/lookdev.replay.json');assert c.call('run_replay',path='artifacts/lookdev.replay.json')['meshes_matched']
        for name,tick in [('replay',None),('prefix',0)]:
            path=f'artifacts/{name}.png';c.call('capture_replay',replay_path='artifacts/lookdev.replay.json',path=path,tick=tick,width=1536,height=1024);assert (root/path).read_bytes()==hero
        c.call('load_scene',path='examples/cottage-lookdev.scene.json');assert c.call('inspect')['scene']==saved and capture('loaded')[0]==hero
        c.call('reset');assert capture('reset')[0]==hero
        capture('viewport',width=800,height=600);image,meta=capture('viewport-restored');assert image==hero and meta['adapter']['environment_preparation']==stats
        checks.append('Sun/probe/material-neutral ablations, static60ticks, save/load/reset/replay/prefix/viewport exact; all captures retain same native geometry and one-entry cached resources')
        meshes=saved['meshes'];v=sum(len(a['geometry']['positions']) for a in meshes['assets']);t=sum(len(a['geometry']['triangles']) for a in meshes['assets']);assert (len(meshes['assets']),len(meshes['instances']),v,t,meta['mesh_triangle_count'])==tuple(recipe['expected_geometry'])
        report=dict(passed=True,build_id=c.call('inspect')['build_id'],source_sha256=recipe['source_sha256'],recipe_sha256=hashlib.sha256((P/'examples/cottage-lookdev.recipe.json').read_bytes()).hexdigest(),checks=checks,assets=len(meshes['assets']),instances=len(meshes['instances']),vertices=v,triangles=t,draw_triangles=meta['mesh_triangle_count'],preparation=stats,remaining_vertices=32768-v,limitations=[f"Structural hypothesis {recipe['source_revision']}, no claim of complete fidelity",'Distant procedural environment only; no local reflections/AO/HDR','Opaque window panels, simplified tracery/landscape and inferred interior remain','16asset slots full; per-region material/component identity remain gaps'])
        (root/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:c.close()


if __name__=='__main__':main()
