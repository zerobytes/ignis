#!/usr/bin/env python3
"""Compare native relief content with identical lookdev, then inspect the combined house."""
import copy
import argparse
import subprocess
import hashlib
import json
import os
from pathlib import Path
P = Path(__file__).resolve().parents[1]
R = P/'artifacts/cottage-relief-integration'
BEFORE_HERO_SHA256 = '5ac72456cd4ef0b5817743f93da6fcff7aa09a8b818b9dfdec8fb6a83448368d'


def publish_source():
    """Explicit content-authoring step; regenerate bronze via native Sweep only."""
    from demo_elven_cottage import build
    raw_source=subprocess.check_output(['git','show','1054577:examples/elven-cottage.scene.json'],cwd=P)
    assert hashlib.sha256(raw_source).hexdigest()=='a742696faca960b08684a8a269975477c6544568de50b4b38b578536649a57e6'
    original=json.loads(raw_source);cfg=json.loads((P/'examples/elven-cottage.recipe.json').read_text())
    legacy=copy.deepcopy(cfg)
    for key in ['relief_front_offset','relief_bottom','relief_height']:legacy['porch'].pop(key,None)
    raw_before=next(a for a in build(legacy)[0] if a['id']=='bronze')
    raw_after=next(a for a in build(cfg)[0] if a['id']=='bronze')
    assert raw_before['geometry']==raw_after['geometry']
    changed=[i for i,(a,b) in enumerate(zip(raw_before['_sweeps'],raw_after['_sweeps'])) if a!=b]
    assert len(changed)==8 and len(raw_before['_sweeps'])==len(raw_after['_sweeps'])
    for name in ['examples','artifacts']:(R/name).mkdir(parents=True,exist_ok=True)
    (R/'examples/placement-source.scene.json').write_bytes(raw_source)
    os.environ['IGNIS_VERIFY_ROOT']=str(R)
    from verify_lab import Client
    c=Client(P/'target/release/ignis');call=c.call
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='relief-placement-authoring',version='1')))
        c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/placement-source.scene.json')
        source=call('inspect_mesh',asset='bronze')['asset'];before=call('inspect')
        def edits(raw):
            sweeps=copy.deepcopy(raw['_sweeps']);plans=call('plan_mesh_sweep',sweeps=sweeps)['plan']['pieces']
            for sweep,plan in zip(sweeps,plans):sweep['spans']=plan['spans']
            prefix={**source,'geometry':raw['geometry']}
            return [dict(op='mesh',edit=dict(kind='put_asset',asset=prefix)),dict(op='mesh',edit=dict(kind='sweep',id='bronze',sweeps=sweeps,append=True))],plans
        baseline,_=edits(raw_before)
        prediction=call('preview_scene_edits',edits=baseline)['scene']
        assert next(a for a in prediction['meshes']['assets'] if a['id']=='bronze')==source
        revised,plans=edits(raw_after);preview=call('preview_scene_edits',edits=revised)['scene']
        assert call('inspect')==before
        call('edit_scene',edits=revised);assert call('inspect')['scene']==preview
        native=call('inspect_mesh',asset='bronze')['asset']['geometry']
        old=source['geometry'];assert len(native['positions'])==len(old['positions']) and native['triangles']==old['triangles']
        affected=set();cursor=len(raw_before['geometry']['positions'])
        for i,plan in enumerate(plans):
            if i in changed:affected.update(range(cursor,cursor+plan['vertices']))
            cursor+=plan['vertices']
        for i,(a,b) in enumerate(zip(old['positions'],native['positions'])):
            if i in affected:
                assert 3.375<=b[1]<=3.615 and 3.375-.001<=b[2]<=3.375+.025
            else:
                assert a==b and old['material_coordinates'][i]==native['material_coordinates'][i]
        topology=call('inspect_mesh_topology',asset='bronze')['topology']
        assert topology['boundary_edges']==topology['non_manifold_edges']==0
        bounds=[(min(native['positions'][i][k] for i in affected),max(native['positions'][i][k] for i in affected)) for k in range(3)]
        assert abs(bounds[2][0]-3.375)<.001
        call('save_scene',path='examples/placement-evaluated.scene.json')
        serialized=json.loads((R/'examples/placement-evaluated.scene.json').read_text())
        published=copy.deepcopy(original)
        next(a for a in published['meshes']['assets'] if a['id']=='bronze')['geometry']=next(a for a in serialized['meshes']['assets'] if a['id']=='bronze')['geometry']
        encoded=(json.dumps(published,separators=(',',':'))+'\n').encode()
        (R/'examples/placement-published.scene.json').write_bytes(encoded)
        call('load_scene',path='examples/placement-published.scene.json');assert call('inspect_mesh',asset='bronze')['asset']['geometry']==native
        (P/'examples/elven-cottage.scene.json').write_bytes(encoded)
        digest=hashlib.sha256(encoded).hexdigest()
        recipe_path=P/'examples/cottage-lookdev.recipe.json';recipe=json.loads(recipe_path.read_text());recipe['source_sha256']=digest;recipe['source_revision']='v4-relief-front-offset';recipe_path.write_text(json.dumps(recipe,indent=2)+'\n')
        report=dict(passed=True,build_id=call('inspect')['build_id'],before_sha256=hashlib.sha256(raw_source).hexdigest(),after_sha256=digest,
                    changed_sweeps=changed,repositioned_vertices=len(affected),offset=.23,bottom=3.38,height=.22,world_bounds=bounds,signed_surface_gap=bounds[2][0]-3.375,unaffected_vertices_and_channels_exact=True,triangles_exact=True,
                    limitations=['Manual authored offset, no native surface-support/clearance query','Selected vertices fit capital y3.375..3.615, z3.375-1mm..+25mm; near-contact within1mm, no global collision proof'])
        (R/'artifacts/placement-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:c.close()


def main():
    canonical_hash=hashlib.sha256((P/'examples/elven-cottage.scene.json').read_bytes()).hexdigest()
    assert canonical_hash==json.loads((P/'examples/cottage-lookdev.recipe.json').read_text())['source_sha256']
    for name in ['examples','artifacts']:(R/name).mkdir(parents=True,exist_ok=True)
    for filename,source in [('after.scene.json',P/'artifacts/cottage-lookdev/examples/cottage-lookdev.scene.json'),
                            ('integrated.scene.json',P/'artifacts/cottage-normals/examples/cottage-spring.scene.json')]:
        (R/'examples'/filename).write_bytes(source.read_bytes())
    # Independent historical source plus current lookdev make the comparison reproducible without ignored inputs.
    old_source=json.loads(subprocess.check_output(['git','show','fc8fc2f:examples/elven-cottage.scene.json'],cwd=P))
    new_source=json.loads((P/'examples/elven-cottage.scene.json').read_text())
    old_without=copy.deepcopy(old_source);new_without=copy.deepcopy(new_source)
    for scene in [old_without,new_without]:
        next(a for a in scene['meshes']['assets'] if a['id']=='bronze').pop('geometry')
    assert old_without==new_without
    before=json.loads((R/'examples/after.scene.json').read_text())
    next(a for a in before['meshes']['assets'] if a['id']=='bronze')['geometry']=next(a for a in old_source['meshes']['assets'] if a['id']=='bronze')['geometry']
    (R/'examples/before.scene.json').write_text(json.dumps(before,separators=(',',':'))+'\n')
    scenes={name:json.loads((R/'examples'/f'{name}.scene.json').read_text()) for name in ['before','after','integrated']}
    def except_geometry(scene,asset):
        s=copy.deepcopy(scene)
        for a in s['meshes']['assets']:
            if a['id']==asset:a.pop('geometry')
        return s
    assert except_geometry(scenes['before'],'bronze')==except_geometry(scenes['after'],'bronze')
    assert except_geometry(scenes['after'],'carved-stone')==except_geometry(scenes['integrated'],'carved-stone')
    cameras=dict(hero=scenes['before']['camera'],
        finial=dict(position=[7.5,7.1,4],target=[3.72,6.32,-.65],fov_degrees=25),
        chimney=dict(position=[0,7.4,6],target=[-2.7,5.8,.58],fov_degrees=22),
        frieze=dict(position=[-.5,4.3,8],target=[-2.3,3.5,3.2],fov_degrees=24))
    os.environ['IGNIS_VERIFY_ROOT']=str(R)
    from verify_lab import Client
    from verify_materials import f32
    scenes={name:f32(scene) for name,scene in scenes.items()}
    c=Client(P/'target/release/ignis');call=c.call;checks=[];pixels={}
    def capture(name,width=1536,height=1024):
        path='artifacts/'+name+'.png';meta=call('capture',path=path,width=width,height=height)
        return (R/path).read_bytes(),meta
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='relief-integration-study',version='1')))
        c.request('notifications/initialized',notification=True)
        for stage in ['before','after','integrated']:
            call('load_scene',path=f'examples/{stage}.scene.json');pixels[stage]={}
            assert call('inspect')['scene']==scenes[stage]
            for name,camera in cameras.items():
                call('set_camera',camera=camera)
                image,meta=capture(stage+'-'+name);pixels[stage][name]=image
                assert capture(stage+'-'+name+'-repeat')[0]==image
                assert meta['adapter']['environment_preparation']['probe_bakes']==1
                assert meta['adapter']['environment_preparation']['lut_bakes']==1
        assert hashlib.sha256(pixels['before']['hero']).hexdigest()==BEFORE_HERO_SHA256
        (R/'artifacts/baseline-hero.png').write_bytes(pixels['before']['hero'])
        assert all(pixels['before'][name]!=pixels['after'][name] for name in cameras)
        checks.append('Relief-only change proven with all other geometry/appearances/instances/sources/camera/light equal; baseline pixel-exact and four fixed repeated views differ at ornaments')
        assert pixels['after']['hero']!=pixels['integrated']['hero']
        assets=scenes['integrated']['meshes']['assets'];vertices=sum(len(a['geometry']['positions']) for a in assets);triangles=sum(len(a['geometry']['triangles']) for a in assets)
        assert (vertices,triangles,meta['mesh_triangle_count'])==(31948,53436,54656)
        checks.append('Combined scene differs from relief lookdev only in the reauthored carved-stone geometry,16assets/21instances/31948v; same materials and one probe/LUT bake')
        call('set_camera',camera=cameras['hero']);call('save_scene',path='examples/integrated.scene.json')
        hero=capture('integrated-hero')[0];assert hero==pixels['integrated']['hero']
        call('step',ticks=60);assert capture('integrated-tick60')[0]==hero
        call('save_replay',path='artifacts/integrated.replay.json');assert call('run_replay',path='artifacts/integrated.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/integrated.replay.json',path='artifacts/integrated-replay.png',width=1536,height=1024)
        assert (R/'artifacts/integrated-replay.png').read_bytes()==hero
        call('load_scene',path='examples/integrated.scene.json');assert capture('integrated-loaded')[0]==hero
        call('reset');assert capture('integrated-reset')[0]==hero
        capture('integrated-viewport',800,600);assert capture('integrated-viewport-restored')[0]==hero
        checks.append('Integrated authoring save/load/reset/static60/replay and viewport retain exact pixels; canonical source unchanged')
        assert hashlib.sha256((P/'examples/elven-cottage.scene.json').read_bytes()).hexdigest()==canonical_hash
        report=dict(passed=True,checks=checks,build_id=call('inspect')['build_id'],canonical_sha256=canonical_hash,
                    source_sha256={k:hashlib.sha256((R/'examples'/f'{k}.scene.json').read_bytes()).hexdigest() for k in scenes},
                    cameras=cameras,vertices=vertices,triangles=triangles,draw_triangles=meta['mesh_triangle_count'],remaining_vertices=32768-vertices,
                    limitations=['Relief components overlap; no persistent per-piece identity','Centerline bound excludes section/surface error','Pointed arch junctions, glass, stone variety and garden remain simplified'])
        (R/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:c.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--publish-source',action='store_true');args=parser.parse_args()
    if args.publish_source:publish_source()
    else:main()
