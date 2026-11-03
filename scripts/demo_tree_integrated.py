#!/usr/bin/env python3
"""Integrate the local fork correction into the saved tree, retaining all materials."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

PROJECT=Path(__file__).resolve().parents[1]
OUTPUT=PROJECT/'artifacts/tree-integrated'
(OUTPUT/'artifacts').mkdir(parents=True,exist_ok=True)
(OUTPUT/'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT']=str(OUTPUT)
from verify_lab import Client


def run(input_path,frames):
    snapshot=OUTPUT/'examples/input.scene.json'
    if input_path.resolve()!=snapshot.resolve():shutil.copyfile(input_path,snapshot)
    # Separate processes keep each existing MCP demo's workspace unambiguous.
    subprocess.run([sys.executable,str(PROJECT/'scripts/demo_tree_continuity.py'),'--input',str(snapshot)],cwd=PROJECT,check=True)
    candidate=PROJECT/'artifacts/tree-continuity/examples/candidate.scene.json'
    subprocess.run([sys.executable,str(PROJECT/'scripts/demo_tree_smoothing.py'),'--input',str(candidate),'--frames',str(frames),'--segments-y','4'],cwd=PROJECT,check=True)
    source=PROJECT/'examples/native-tree-study.scene.json'
    shutil.copyfile(source,OUTPUT/'examples/integrated.scene.json')
    c=Client(PROJECT/'target/release/ignis',False);call=c.call
    def scene():return call('inspect')['scene']
    def poses(instances):return {tick:{i['id']:call('inspect_mesh_pose',instance=i['id'],tick=tick)['pose'] for i in instances if i['asset']!='wood'} for tick in [0,60]}
    def capture(name,width=1000,height=900):
        path=f'artifacts/{name}.png';meta=call('capture',path=path,width=width,height=height)
        return hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest(),meta
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'ignis-tree-integration','version':'1'}})
        c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json');before=scene(); old_assets={a['id']:a for a in before['meshes']['assets']}
        original_poses=poses(before['meshes']['instances'])
        call('load_scene',path='examples/integrated.scene.json');after=scene();new_assets={a['id']:a for a in after['meshes']['assets']}
        assert before['version']==after['version']>=60
        assert before['plants']==after['plants']
        assert {k:v for k,v in before.items() if k!='meshes'}=={k:v for k,v in after.items() if k!='meshes'}
        assert before['meshes']['instances']==after['meshes']['instances']
        assert poses(after['meshes']['instances'])==original_poses
        assert old_assets.keys()==new_assets.keys()
        for id,a in old_assets.items():
            b=new_assets[id]
            assert a['material']==b['material'],id
            if id!='wood':assert a==b,id
            else:
                assert {k:v for k,v in a.items() if k!='geometry'}=={k:v for k,v in b.items() if k!='geometry'}
                assert len(b['geometry']['positions'])-len(a['geometry']['positions'])==128
                assert len(b['geometry']['triangles'])-len(a['geometry']['triangles'])==256
        # Same read channel verifies source and evaluated material metadata,
        # avoiding inspect-f64 versus file-f32 JSON formatting differences.
        materials={id:a['material'] for id,a in new_assets.items()}
        assert sum('reflectance' in m for m in materials.values())==9
        start,_=capture('rest',1200,1100)
        call('step',ticks=60);live,meta=capture('after',1200,1100)
        assert call('inspect')['tick']==60 and meta['mesh_triangle_count']==65050
        call('save_replay',path='artifacts/integrated.replay.json')
        assert call('run_replay',path='artifacts/integrated.replay.json')['meshes_matched']
        for name,tick,expected in [('replay',None,live),('prefix',0,start)]:
            path=f'artifacts/{name}.png'
            call('capture_replay',replay_path='artifacts/integrated.replay.json',path=path,tick=tick,width=1200,height=1100)
            assert hashlib.sha256((OUTPUT/path).read_bytes()).hexdigest()==expected
        call('reset');assert capture('reset',1200,1100)[0]==start
        capture('viewport',640,480);assert capture('viewport-restored',1200,1100)[0]==start
        report={'passed':True,'build_id':call('inspect')['build_id'],'native_window':False,
            'input_sha256':hashlib.sha256(snapshot.read_bytes()).hexdigest(),'output_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
            'materials_and_reflectance_exact':True,'other_assets_instances_plants_poses_0_60_exact':True,
            'live_prefix_reset_viewport_pixels_exact':True,'geometry_added_vertices':128,'geometry_added_triangles':256,
            'draw_triangles':meta['mesh_triangle_count'],'draw_headroom':call('inspect_meshes')['limits']['draw_triangles']-meta['mesh_triangle_count'],
            'assets':len(new_assets),'instances':len(after['meshes']['instances']),'source':str(source),'source_bytes':source.stat().st_size,
            'frames':frames,'continuity_report':str(PROJECT/'artifacts/tree-continuity/artifacts/report.json'),
            'finish_report':str(PROJECT/'artifacts/tree-study/artifacts/report.json'),
            'notes':['Only first fork corrected; other overlapping insertions remain open','Artistic realism and UI remain separate acceptance gates']}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2))
    finally:c.close()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',type=Path,default=PROJECT/'examples/native-tree-study.scene.json');p.add_argument('--frames',type=int,default=48);a=p.parse_args()
    if a.frames<0 or a.frames>120 or (a.frames and 240%a.frames):p.error('frames must be0 or a positive divisor of240 up to120')
    run(a.input.resolve(),a.frames)
