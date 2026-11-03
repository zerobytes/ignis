#!/usr/bin/env python3
"""Native offscreen evidence for EAF-165; uses shared MCP commands, no window."""
import argparse
import copy
import json
import math
from pathlib import Path
from png_pixels import rgba8
from verify_lab import Client, ROOT
from verify_materials import f32
from verify_metallic import encoded


def mountains(nx=110, nz=110):
    positions, triangles = [], []
    for row in range(nz + 1):
        z = (row / nz - .5) * 18
        for col in range(nx + 1):
            x = (col / nx - .5) * 70
            ridge = 7 + 3 * math.sin(x * .14) + 1.8 * math.cos(x * .47)
            crest = max(0, 1 - abs(z) / 9)
            y = .2 + ridge * crest + .6 * math.sin(x * 1.9 + z * 1.5) * crest
            positions.append([x, y, z])
    for row in range(nz):
        for col in range(nx):
            a = row * (nx + 1) + col
            b = a + nx + 1
            triangles.extend([[a, b, a + 1], [a + 1, b, b + 1]])
    return f32(dict(id='ridge', geometry=dict(positions=positions, triangles=triangles),
        shading='smooth', material=dict(pattern=dict(kind='ridged', frequency=[2, 2, 2],
        low=[.18, .24, .12], high=[1, .8, .55], bump=.04, warp=.5, octaves=3, seed=17, contrast=1.5))))


def verify(binary):
    out = ROOT / 'artifacts/atmosphere'
    out.mkdir(parents=True, exist_ok=True)
    c = Client(binary)
    checks, samples = [], []

    def capture(name, w=800, h=600):
        c.call('capture', path=f'artifacts/atmosphere/{name}.png', width=w, height=h)
        return (out / f'{name}.png').read_bytes()

    def edit(kind, **kwargs):
        return c.call('edit_mesh', edit=dict(kind=kind, **kwargs))

    def lighting(atmosphere=None, direction=(0, -1, 0), ambient=1):
        light = dict(direction=list(direction), ambient=ambient)
        if atmosphere is not None:
            light['atmosphere'] = f32(atmosphere)
        c.call('set_light', light=light)

    def camera(z):
        c.call('set_camera', camera=dict(position=[0, 5, z], target=[0, 5, 0], fov_degrees=50))

    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
            clientInfo=dict(name='atmosphere-verifier', version='1')))
        c.request('notifications/initialized', notification=True)
        assert 'sun_strength' in json.dumps(c.request('tools/list'))
        c.call('generate', recipe=dict(width=16, depth=14, seed=17, obstacles=0))
        off = capture('disabled')
        if (out / 'legacy.png').exists():
            assert off == (out / 'legacy.png').read_bytes(), 'disabled differs from previous binary'
            checks.append('disabled pixels equal the pre-change native binary')
        base_light = c.call('inspect')['scene']['light']
        air = dict(density=.025, color=[.38, .55, .72], sun_color=[1, .78, .52], sun_strength=0)
        c.call('set_light', light=dict(base_light, atmosphere=dict(air, density=0)))
        assert capture('zero') == off
        checks.append('density zero and absence are byte-exact')
        entities = c.call('inspect')['scene']['entities']
        c.call('edit_scene', edits=[dict(op='upsert', entity=dict(e, visible=False)) for e in entities])
        asset = dict(id='sheet', geometry=dict(positions=[[-3, 2, 0], [3, 2, 0], [3, 8, 0], [-3, 8, 0]],
            triangles=[[0, 1, 2], [0, 2, 3]]))
        edit('put_asset', asset=asset)
        edit('put_instance', instance=dict(id='sheet', asset='sheet', position=[0, 0, 0],
            rotation=[0, 0, 0, 1], scale=1, color=[.2, .4, .05], playback=None))
        for distance in [10, 40, 90]:
            camera(distance)
            lighting()
            reference = rgba8(capture(f'plane-{distance}-clear'))[2]
            lighting(air)
            actual = rgba8(capture(f'plane-{distance}-air'))[2]
            center = (300 * 800 + 400) * 4
            transmission = math.exp(-air['density'] * distance)
            expected = [encoded(b * transmission + a * (1 - transmission))
                        for a, b in zip(air['color'], [.2, .4, .05])]
            observed = list(actual[center:center+3])
            assert max(abs(a - b) for a, b in zip(observed, expected)) <= 2, (observed, expected)
            old_background, new_background = reference[:3], actual[:3]
            mask_a = [reference[i:i+3] != old_background for i in range(0, len(reference), 4)]
            mask_b = [actual[i:i+3] != new_background for i in range(0, len(actual), 4)]
            assert mask_a == mask_b, 'silhouette changed'
            samples.append(dict(distance_m=distance, transmission=transmission, rgb=observed, expected=expected))
        checks.append('10/40/90m analytical linear-light blend within2RGB8; silhouettes identical')
        lighting(dict(air, sun_strength=1), direction=(0, 0, 1))
        toward = capture('sun-toward')
        lighting(dict(air, sun_strength=1), direction=(1, 0, 0))
        away = capture('sun-away')
        assert toward != away
        c.call('undo'); assert capture('sun-undo') == toward
        c.call('redo'); assert capture('sun-redo') == away
        checks.append('sun direction updates surface and background; undo/redo exact')
        edit('remove_instance', id='sheet')
        edit('remove_asset', id='sheet')
        edit('put_asset', asset=mountains())
        for i, (z, scale) in enumerate([(0, 1), (-40, 1.6), (-100, 2.6), (-200, 4), (-370, 6)]):
            edit('put_instance', instance=f32(dict(id=f'ridge-{i}', asset='ridge', position=[i * 3 - 6, 0, z],
                rotation=[0, math.sin(i * .09), 0, math.cos(i * .09)], scale=scale,
                color=[.3 + i*.06, .42 - i*.04, .23 + i*.05], playback=None)))
        # Continuous ground under the height-field patches; no fabricated horizon holes.
        edit('put_asset',asset=dict(id='ground',geometry=dict(
            positions=[[-9000,-.1,-9000],[9000,-.1,-9000],[9000,-.1,9000],[-9000,-.1,9000]],
            triangles=[[0,2,1],[0,3,2]])))
        edit('put_instance',instance=dict(id='ground',asset='ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.17,.22,.12],playback=None))
        views = [(0, 15, 38), (9, 13, 22), (-10, 25, 60)]
        for i, position in enumerate(views):
            c.call('set_camera', camera=dict(position=position, target=[0, 17, -90], fov_degrees=50))
            lighting(None, (.65, -1, -.2), .2)
            capture(f'mountains-{i}-clear', 1280, 800)
            lighting(dict(air, density=.009, sun_strength=.35), (.65, -1, -.2), .2)
            capture(f'mountains-{i}-air', 1280, 800)
        # Save the reference view, with atmosphere, for reproducible inspection.
        c.call('set_camera', camera=dict(position=views[0], target=[0, 17, -90], fov_degrees=50))
        final = capture('mountains-final', 1280, 800)
        c.call('save_scene', path='artifacts/atmosphere/mountains.scene.json')
        c.call('step', ticks=3)
        c.call('save_replay', path='artifacts/atmosphere/mountains.replay.json')
        c.call('capture_replay', path='artifacts/atmosphere/replay.png', replay_path='artifacts/atmosphere/mountains.replay.json', width=1280, height=800)
        assert (out/'replay.png').read_bytes() == final
        c.call('load_scene', path='artifacts/atmosphere/mountains.scene.json')
        assert capture('loaded', 1280, 800) == final
        checks.append('five detailed ridges / three cameras; scene load and replay pixels exact')
        # Explicitly compile, then compare the same geometry/materials + atmosphere
        # against the selected representations. No camera-dependent compilation.
        c.call('edit_scene', edits=[dict(op='compile_distant_meshes')])
        compiled_scene = c.call('inspect')['scene']
        rows = []
        # One repeated dense asset; distances are metres in the native camera.
        for distance in [100, 1000, 9000, 100]:
            c.call('set_camera', camera=dict(position=[0, 20, distance], target=[0, 10, 0], fov_degrees=35))
            light = dict(direction=[.65, -1, -.2], ambient=.2,
                atmosphere=dict(air, density=.009, sun_strength=.35, reduce_details=False,
                    silhouette_error_px=.5, color_error=.002))
            c.call('set_light', light=f32(light))
            reference=capture(f'comparison-{distance}-full',1728,1117)
            light['atmosphere']['reduce_details']=True
            c.call('set_light', light=f32(light))
            candidate=capture(f'comparison-{distance}-selected',1728,1117)
            metadata=c.call('capture',path='artifacts/atmosphere/selection-current.png',width=1728,height=1117)['adapter']['distant_selection']
            r=rgba8(reference)[2]; v=rgba8(candidate)[2]
            differences=[abs(a-b) for i,(a,b) in enumerate(zip(r,v)) if i%4!=3]
            row=dict(distance_m=distance,max_rgb8=max(differences),mae_rgb8=sum(differences)/len(differences),**metadata)
            if distance==100:
                assert reference==candidate, row
            else:
                assert metadata['substituted_instances']>0,row
                assert row['max_rgb8']<=3,row
                assert metadata['selected_vertex_bytes']<metadata['source_triangles']*3*64,row
            if distance==9000:
                assert metadata['geometric_lod_instances']>0,row
            rows.append(row)
        # Exercise the transition band and changing sun on the selected path.
        for i,(distance,density) in enumerate([(650,.009),(800,.009),(1000,.0045)]):
            c.call('set_camera',camera=dict(position=[(-1)**i*12,22,distance],target=[0,10,0],fov_degrees=35))
            light=dict(direction=[(-1)**i*.8,-.6,.3],ambient=.2,
                atmosphere=dict(air,density=density,sun_strength=.8,reduce_details=False,silhouette_error_px=.5,color_error=.002))
            c.call('set_light',light=f32(light)); reference=capture(f'transition-{i}-full',1728,1117)
            light['atmosphere']['reduce_details']=True
            c.call('set_light',light=f32(light)); candidate=capture(f'transition-{i}-selected',1728,1117)
            r=rgba8(reference)[2];v=rgba8(candidate)[2]
            error=max(abs(a-b) for k,(a,b) in enumerate(zip(r,v)) if k%4!=3)
            assert error<=3,(distance,density,error)
            rows.append(dict(distance_m=distance,density=density,max_rgb8=error,transition=True))
        checks.append('compiled native geometry/material substitution:100/1000/9000m + near return, <=3RGB8')
        # Visible distant silhouettes, with an explicit looser interior color budget.
        c.call('set_camera',camera=dict(position=[0,25,9000],target=[0,10,0],fov_degrees=12))
        light=dict(direction=[.6,-1,.2],ambient=.2,atmosphere=dict(air,density=.0004,sun_strength=.5,
            reduce_details=False,silhouette_error_px=.5,color_error=.01))
        c.call('set_light',light=f32(light)); reference=capture('visible-full',1728,1117)
        light['atmosphere']['reduce_details']=True
        c.call('set_light',light=f32(light)); candidate=capture('visible-selected',1728,1117)
        r=rgba8(reference)[2];v=rgba8(candidate)[2]
        visible_error=max(abs(a-b) for k,(a,b) in enumerate(zip(r,v)) if k%4!=3)
        assert visible_error<=5,visible_error
        stats=c.call('capture',path='artifacts/atmosphere/visible-selected.png',width=1728,height=1117)['adapter']['distant_selection']
        assert stats['geometric_lod_instances']>0,stats
        rows.append(dict(distance_m=9000,density=.0004,color_error=.01,max_rgb8=visible_error,visible_silhouettes=True,**stats))
        # Restore reference scene and save an optimized native demo.
        c.call('load_scene',path='artifacts/atmosphere/mountains.scene.json')
        c.call('edit_scene',edits=[dict(op='compile_distant_meshes')])
        light=c.call('inspect')['scene']['light']; light['atmosphere']['reduce_details']=True
        c.call('set_light',light=light)
        c.call('save_scene',path='artifacts/atmosphere/mountains-compiled.scene.json')
        report = dict(passed=True, checks=checks, samples=samples, comparisons=rows, build_id=c.call('inspect')['build_id'],
            geometry_triangles=24200*5+2, additional_uniform_bytes=80, geometry_culling=False, compiled_substitution=True,
            performance_claim='Draw counts and vertex bytes measured; timings reported separately', native_window=False)
        (out/'verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(dict(passed=True, checks=len(checks), native_window=False)))
    finally:
        c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    verify(parser.parse_args().binary.resolve())
