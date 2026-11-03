#!/usr/bin/env python3
"""Exercise the native surface/globe camera over MCP, without foreground windows.

The separate flat fixture is a raster coverage oracle, not the demo planet.
No per-frame Python, generated geometry payload or extra app bundle is used.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
from png_pixels import rgba8
from verify_lab import Client, ROOT
from open_earth_walk import init, view

OUT=ROOT/'artifacts/planet-zoom'
SCENE='artifacts/planet-runtime/player.scene.json'
CONFIG='artifacts/planet-runtime/world.json'

def setup(binary, flat=False):
    c=Client(binary,log_path=OUT/('flat.log' if flat else 'runtime.log'))
    init(c)
    c.call('set_execution_mode',mode='continuous')
    c.call('load_scene',path=SCENE)
    config=json.loads((ROOT/CONFIG).read_text())
    if flat:
        config['document'].update(operations=[],base_height_meters=0)
        config.update(material={},color=[0.4,0.6,0.3],compile_distant_meshes=False)
        config['spawn']['altitude_meters']=0.91
        c.call('set_planet_runtime',config=config)
    else:
        c.call('load_planet_runtime',path=CONFIG)
    c.call('step',ticks=120)
    c.call('set_player_view',view=view())
    return c, config

def capture(c,name,width=1280,height=800):
    path=f'artifacts/planet-zoom/{name}.png'
    info=c.call('capture',path=path,width=width,height=height)
    return {'path':path,'sha256':hashlib.sha256((ROOT/path).read_bytes()).hexdigest(),'adapter':info.get('adapter')}

def verify(binary):
    OUT.mkdir(parents=True,exist_ok=True)
    if not (ROOT/SCENE).is_file() or not (ROOT/CONFIG).is_file():
        from open_planet_walk import prepare
        prepare(binary)
    c,config=setup(binary)
    try:
        third=view();third['mode']='third_person';third['third_person']={'distance_meters':4,'shoulder_offset':0.4}
        c.call('set_player_view',view=third)
        inherited=c.call('zoom_planet_view',factor=1)
        assert inherited['settings']['distance_meters']==4
        c.call('set_planet_view',view=None);c.call('set_player_view',view=view())
        baseline=c.call('inspect_planet_runtime')
        scene=c.call('inspect')['scene']
        legacy=capture(c,'legacy-before')
        previous=ROOT/'artifacts/planet-appearance/valley-combined.png'
        baseline_exact=None
        if previous.is_file():
            old=rgba8(previous.read_bytes())[2]
            current=rgba8((ROOT/legacy['path']).read_bytes())[2]
            assert old==current,'pre-orbital opaque baseline changed pixels'
            for name,yaw in [('west',270),('east',90)]:
                v=view();v.update(yaw_degrees=yaw,pitch_degrees=12)
                c.call('set_player_view',view=v)
                candidate=capture(c,f'legacy-{name}')
                expected=ROOT/f'artifacts/planet-appearance/{name}-combined.png'
                assert rgba8(expected.read_bytes())[2]==rgba8((ROOT/candidate['path']).read_bytes())[2],f'pre-orbital {name} baseline changed pixels'
            c.call('set_player_view',view=view())
            baseline_exact=True
        radius=config['document']['diameter_meters']/2
        results=[]
        warm=None
        for name,distance in [('surface',0),('third',4),('near',50),('one-km',1000),('hundred-km',100_000),('globe',radius*3),('distant',radius*30),('astronomical',1e18),('return-surface',0)]:
            state=c.call('set_planet_view',view={'distance_meters':distance,'viewport':[1280,800]})
            assert state['all_six_faces_covered'] and state['conforming_edges']
            assert state['resident_tiles']<=768 and state['vertices']<=2_000_000
            assert c.call('inspect_planet_runtime')==baseline
            image=capture(c,name)
            results.append({'name':name,'view':c.call('inspect_planet_view'),'image':image})
            if name=='globe':
                repeated=capture(c,'globe-repeated')
                assert repeated['sha256']==image['sha256']
                uploaded=image['adapter']['instancing']['geometry_uploads']
                assert repeated['adapter']['instancing']['geometry_uploads']==uploaded
                built=c.call('inspect_planet_view')['geometry_builds']
                c.call('look_planet_view',yaw_delta_degrees=0.1,tilt_delta_degrees=0)
                rotated=capture(c,'globe-look')
                assert c.call('inspect_planet_view')['geometry_builds']==built
                assert rotated['adapter']['instancing']['geometry_uploads']==uploaded
                c.call('frame_planet_view')
                assert c.call('inspect_planet_runtime')==baseline
                warm={'repeat_rgb8_exact':True,'geometry_uploads_delta':0,'small_look_geometry_builds_delta':0,'small_look_geometry_uploads_delta':0}

        assert results[0]['image']['sha256']==results[-1]['image']['sha256'],'surface roundtrip changed pixels'
        before=c.call('inspect_planet_view')
        for op,args in [('zoom_planet_view',{'factor':-1}),('set_planet_view',{'view':{'max_tiles':5}}),('set_planet_viewport',{'width':32,'height':800})]:
            c.call(op,expected_error=True,**args)
            assert c.call('inspect_planet_view')==before
        c.call('set_planet_view',view=None)
        after=capture(c,'legacy-after')
        assert legacy['sha256']==after['sha256'],'legacy None projection changed pixels'
        assert c.call('inspect')['scene']==scene
        assert c.call('inspect_planet_runtime')==baseline
        c.call('set_planet_view',view={'distance_meters':radius*3})
        c.call('set_player_view_input',input={'x':0,'z':-1})
        c.call('step',ticks=120)
        c.call('set_player_view_input',input={'x':0,'z':0})
        walked=c.call('inspect_planet_runtime')
        assert walked['actual_path_meters']>baseline['actual_path_meters']+1
        result={'build_id':c.call('inspect')['build_id'],'source_id':config['document']['id'],'source_sha256':hashlib.sha256(json.dumps(config['document'],sort_keys=True).encode()).hexdigest(),
                'diameter_meters':radius*2,'warm_cache':warm,'poses':results,'legacy_roundtrip_exact':True,'pre_orbit_baseline_exact':baseline_exact,'surface_roundtrip_exact':True,'walk_after_zoom':walked}
    finally:
        c.close()
    c,config=setup(binary,flat=True)
    try:
        oracles=[]
        for factor in [3,30]:
            c.call('set_planet_view',view={'distance_meters':radius*factor,'atmosphere_optical_depth':0,'space_color':[0,0,0]})
            image=capture(c,f'flat-{factor}',640,400)
            v=c.call('inspect_planet_view')
            center=v['planet_center_view_meters']
            assert abs(center[0])<1e-6 and abs(center[1])<1e-6
            distance=-center[2]
            r=200/math.tan(math.radians(30))*radius/math.sqrt(distance*distance-radius*radius)
            w,h,pixels=rgba8((ROOT/image['path']).read_bytes())
            assert (w,h)==(640,400)
            missed=outside=inside=0
            for y in range(400):
                for x in range(640):
                    d=math.hypot(x+.5-320,y+.5-200)
                    index=4*(y*640+x)
                    visible=max(pixels[index:index+3])>3
                    if d<r-2:
                        inside+=1
                        missed+=not visible
                    if d>r+2:
                        outside+=visible
            assert inside>4 and missed==0 and outside==0,(factor,r,inside,missed,outside)
            oracles.append({'distance_radius_factor':factor,'projected_radius_pixels':r,'required_inner_pixels':inside,'missing_inner_pixels':missed,'unexpected_outer_pixels':outside,'image':image})
        result['flat_raster_oracles']=oracles
        # Same flat source, different geometric resolution: atmosphere must not
        # turn coarse triangle sagitta into a visible grid of optical thickness.
        air=[]
        for name,error,tiles in [('air-coarse',64,6),('air-fine',0.5,768)]:
            c.call('set_planet_view',view={'distance_meters':radius*3,'atmosphere_optical_depth':0.3,'max_error_pixels':error,'max_tiles':tiles})
            image=capture(c,name,640,400)
            air.append((image,c.call('inspect_planet_view')))
        _,_,coarse=rgba8((ROOT/air[0][0]['path']).read_bytes())
        _,_,fine=rgba8((ROOT/air[1][0]['path']).read_bytes())
        r=oracles[0]['projected_radius_pixels']
        errors=[abs(coarse[4*(y*640+x)+k]-fine[4*(y*640+x)+k])
                for y in range(400) for x in range(640)
                if math.hypot(x+.5-320,y+.5-200)<r*0.85 for k in range(3)]
        maximum=max(errors)
        assert maximum<=2,('LOD optical grid',maximum)
        result['atmosphere_lod_stability']={'maximum_interior_rgb8_difference':maximum,
            'coarse_tiles':air[0][1]['resident_tiles'],'fine_tiles':air[1][1]['resident_tiles'],
            'images':[a[0] for a in air]}

    finally:
        c.close()
    (OUT/'verification.json').write_text(json.dumps(result,indent=2))
    print(json.dumps({'poses':len(results),'flat_oracles':[{k:v for k,v in o.items() if k!='image'} for o in oracles],'air_lod_max_rgb8':maximum,'legacy_roundtrip_exact':True,'surface_roundtrip_exact':True},indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    verify(p.parse_args().binary)
