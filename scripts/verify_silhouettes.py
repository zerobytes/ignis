#!/usr/bin/env python3
"""Native integration evidence for exact planar compilation and editor coverage."""
import json
from verify_lab import Client, ROOT
from png_pixels import rgba8


def verify():
    out = ROOT / 'artifacts/selection-outline'
    out.mkdir(parents=True, exist_ok=True)
    c = Client(ROOT / 'target/release/ignis')
    checks = []
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                  clientInfo=dict(name='silhouette-integration', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('load_scene', path='artifacts/selection-outline/ring.scene.json')
        camera = c.call('inspect')['scene']['camera']
        c.call('edit_editor', action=dict(action='camera', camera=camera))
        c.call('select_editor_mesh', instance='ring')
        c.call('capture_editor', path='artifacts/selection-outline/editor.png', width=1440, height=900)
        checks.append('select_editor_mesh and capture_editor use the native coverage path')
        c.call('generate', recipe=dict(width=16, depth=14, seed=17, obstacles=0))
        entities = c.call('inspect')['scene']['entities']
        c.call('edit_scene', edits=[dict(op='upsert', entity=dict(e, visible=False)) for e in entities])
        c.call('edit_mesh', edit=dict(kind='grid', id='flat', segments=32, size=16))
        c.call('edit_mesh', edit=dict(kind='put_instance', instance=dict(id='flat', asset='flat',
               position=[0,0,0], rotation=[0,0,0,1], scale=1, color=[.1,.4,.2], playback=None)))
        c.call('edit_scene', edits=[dict(op='compile_distant_meshes')])
        light=dict(direction=[.65,-1,-.2],ambient=.2,atmosphere=dict(density=.001,
            color=[.44,.59,.75],sun_color=[.95,.73,.4],sun_strength=.15,reduce_details=False,
            silhouette_error_px=0,color_error=.002,layers=dict(sky_color=[.73,.82,.94],
            mountain_color=[.008,.025,.055],detail_start=0,detail_end=1,influence=1)))
        rows=[]
        for i, eye in enumerate(([0,8,12],[5,3,10],[-7,10,10])):
            c.call('set_camera',camera=dict(position=eye,target=[0,0,0],fov_degrees=55))
            results=[]
            for enabled in [False,True]:
                light['atmosphere']['reduce_details']=enabled
                c.call('set_light',light=light)
                suffix='selected' if enabled else 'full'
                results.append(c.call('capture',path=f'artifacts/selection-outline/flat-{i}-{suffix}.png',width=800,height=600)['adapter']['distant_selection'])
            a=rgba8((out/f'flat-{i}-full.png').read_bytes())[2]
            b=rgba8((out/f'flat-{i}-selected.png').read_bytes())[2]
            maximum=max(abs(x-y) for x,y in zip(a,b))
            assert maximum<=1,(i,maximum)
            assert results[1]['geometric_lod_instances']==1,results
            assert results[1]['cheap_triangles']<results[0]['source_triangles'],results
            rows.append(dict(view=i,max_rgb8=maximum,full=results[0],selected=results[1]))
            if i==0:
                light['atmosphere']['edge_softness']=dict(strength=1,radius_px=6,start=0,end=1)
                light['atmosphere']['reduce_details']=False;c.call('set_light',light=light)
                c.call('capture',path='artifacts/selection-outline/flat-soft-full.png',width=800,height=600)
                light['atmosphere']['reduce_details']=True;c.call('set_light',light=light)
                meta=c.call('capture',path='artifacts/selection-outline/flat-soft-selected.png',width=800,height=600)['adapter']['distant_selection']
                assert meta['substituted_instances']==1 and meta['geometric_lod_instances']==0
                a=rgba8((out/'flat-soft-full.png').read_bytes())[2];b=rgba8((out/'flat-soft-selected.png').read_bytes())[2]
                assert max(abs(x-y) for x,y in zip(a,b))<=1
                light['atmosphere'].pop('edge_softness')
                checks.append('edge finish keeps original depth geometry while retaining material substitution')

        checks.append('three views: zero geometric error permits exact planar reduction with unchanged boundary')
        (out/'integration.json').write_text(json.dumps(dict(build_id=c.call('inspect')['build_id'],passed=True,checks=checks,planar=rows),indent=2)+'\n')
        print(json.dumps(dict(passed=True,checks=checks)))
    finally:
        c.close()


if __name__ == '__main__':
    verify()
