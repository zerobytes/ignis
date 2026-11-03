#!/usr/bin/env python3
"""Build/verify/open the native transported Earth runtime through MCP.
Performance review: Python only authors a small source/config; Rust owns sampling,
residence, collision and frames. No per-frame agent, JSON or window automation.
"""
import argparse
import copy
import hashlib
import json
import math
import subprocess
from verify_lab import Client, ROOT
from open_earth_walk import init, view, demo_locomotion
from open_world_walk import native_entry
from build_earth_procedural import document_for as hills_document, direction
from planet_mountain_recipe import document_for as mountain_document

OUT = ROOT / 'artifacts/planet-runtime'
SCENE = 'artifacts/planet-runtime/player.scene.json'
CONFIG = 'artifacts/planet-runtime/world.json'
LAUNCH = ROOT / 'artifacts/earth-walk/launch.json'


def prepare(binary, seed=731199, recipe_path='examples/earth-mountain-chains.recipe.json'):
    OUT.mkdir(parents=True, exist_ok=True)
    recipe = json.loads((ROOT/recipe_path).read_text())
    document_for = mountain_document if 'chains' in recipe else hills_document
    document = document_for(recipe, seed)
    if document != document_for(recipe, seed):
        raise ValueError('Recipe must be deterministic')
    c = Client(binary, log_path=OUT/'prepare.log')
    try:
        init(c)
        initial = c.call('inspect')
        height = c.call('query_planet', query={'kind':'sample','document':document,'directions':[[0,1,0]]})['samples'][0]['height_meters']
        document['base_height_meters'] = -height
        relief = None
        if 'chains' in recipe:
            xs = [-6000,-4500,-3000,-2000,-1000,0,1000,2000,3000,4500,6000]
            zs = [0,-3000,-7000,-12000,-20000]
            probes = [[x,z] for z in zs for x in xs]
            directions = [direction(document['diameter_meters']/2,x,z) for x,z in probes]
            samples = c.call('query_planet',query={'kind':'sample','document':document,'directions':directions})['samples']
            if c.call('query_planet',query={'kind':'sample','document':document,'directions':directions})['samples'] != samples:
                raise ValueError('Native source sampling must be reproducible')
            profiles = []
            for row,z in enumerate(zs):
                heights=[s['height_meters'] for s in samples[row*len(xs):(row+1)*len(xs)]]
                floor=min(heights[4:7])
                sides=[max(heights[:4])-floor,max(heights[7:])-floor]
                profiles.append({'z_meters':z,'x_meters':xs,'heights_meters':heights,'side_relief_meters':sides})
            if any(min(p['side_relief_meters'])<300 for p in profiles):
                raise ValueError(f'Mountain composition did not preserve the intended valley: {profiles}')
            alternate=document_for(recipe,(seed+1)&0xffffffff)
            other=c.call('query_planet',query={'kind':'sample','document':alternate,'directions':directions})['samples']
            delta=max(abs((a['height_meters']-samples[5]['height_meters'])-(b['height_meters']-other[5]['height_meters'])) for a,b in zip(samples,other))
            if delta<1:
                raise ValueError('Changing the seed must change relative relief')
            relief={'native_operations':len(document['operations']),'profiles':profiles,'other_seed_relief_delta_meters':delta,
                    'sampled_height_range_meters':[min(s['height_meters'] for s in samples),max(s['height_meters'] for s in samples)]}
        scene = copy.deepcopy(initial['scene'])
        player = next(e for e in scene['entities'] if e['id']=='player')
        player.update(transform={'position':[0,.91,0],'size':[.6,1.8,.6]},
                      footsteps=None, avatar=None, abilities=None, vitality=None, trigger=None,
                      locomotion=demo_locomotion(),
                      character_controller={'max_slope_degrees':45,'slide_slope_degrees':50,'offset':.01,'snap_distance':.2,'gravity':9.81})
        scene.update(entities=[player], recipe=None, name='Ignis - Earth procedural planetary runtime')
        scene['light'].update(direction=[.7,-.6,-.35], ambient=.3, shadow=None, environment=None,
                              atmosphere={'density':recipe.get('atmosphere_density',.0008),'reduce_details':True,'silhouette_error_px':.5,'color_error':.002,
                                          'color':[.48,.6,.72],'sun_color':[1,.85,.65],'sun_strength':.15})
        material = copy.deepcopy(recipe['material'])
        material['reflectance']['f0'] = 0
        material['pattern']['seed'] = seed & 0xffff
        config = {'document':document,'actor':'player',
                  'spawn':{'direction':[0,1,0],'forward':[0,0,-1],'altitude_meters':.91},
                  'level':recipe.get('runtime',{}).get('level',13),
                  'subdivisions':recipe.get('runtime',{}).get('subdivisions',64),'physical_radius_meters':256,
                  'color':recipe['color'],'material':material,'compile_distant_meshes':True}
        # Authoring/config boundaries only: never transport individual vertices through Python.
        (ROOT/SCENE).write_text(json.dumps(scene, separators=(',',':')))
        (ROOT/CONFIG).write_text(json.dumps(config, indent=2))
        c.call('set_execution_mode', mode='continuous')
        c.call('load_scene', path=SCENE)
        c.call('load_planet_runtime', path=CONFIG)
        start = c.call('inspect_planet_runtime')
        if relief is not None:
            # Cube-face density varies. Check centers, edges and corners before
            # presenting a longer visibility radius as a safe demo setting.
            counts=[]
            for x in (-1,0,1):
                for y in (-1,0,1):
                    for z in (-1,0,1):
                        length=math.sqrt(x*x+y*y+z*z)
                        if not length:
                            continue
                        unit=[v/length for v in (x,y,z)]
                        forward=[1,0,0] if x==y==0 else [0,0,1]
                        request={'pose':{'direction':unit,'forward':forward,'altitude_meters':.91},
                                 'level':config['level'],'visual_radius_meters':start['visual_radius_meters']+512,
                                 'physical_radius_meters':320}
                        plan=c.call('query_planet',query={'kind':'plan_residence','document':document,'request':request})
                        count=len(plan['visual'])
                        if count*(config['subdivisions']+1)**2>2_000_000:
                            raise ValueError('Demo residence exceeds the source budget at a cube edge/corner')
                        counts.append(count)
            relief['residence_probes']={'count':len(counts),'max_visual_tiles':max(counts),
                                        'max_source_vertices':max(counts)*(config['subdivisions']+1)**2}
        c.call('save_planet', path='artifacts/planet-runtime/earth.ignis-planet', document=document)
        c.call('set_player_view', view=view())
        c.call('step', ticks=120)
        captures = [c.call('capture', path='artifacts/planet-runtime/eye-start.png', width=1280, height=800)]
        for yaw in (-90,90):
            side=view();side['yaw_degrees']=yaw;side['pitch_degrees']=12
            c.call('set_player_view',view=side)
            captures.append(c.call('capture',path=f'artifacts/planet-runtime/ridge-{yaw}.png',width=1280,height=800))
        legs = []
        for yaw in recipe.get('verification_yaws',(90,270)):
            c.call('reset')
            v=view();v['yaw_degrees']=yaw
            c.call('set_player_view', view=v)
            c.call('step', ticks=120)
            c.call('set_player_view_input', input={'x':0,'z':-1,'mode':'run'})
            c.call('step', ticks=1500)
            c.call('set_player_view_input', input={'x':0,'z':0})
            c.call('step', ticks=180)
            result=c.call('inspect_planet_runtime')
            # High-speed crests can launch the real motor. Allow a bounded fall
            # to finish; never teleport/snap to make a traversal assertion pass.
            for _ in range(9):
                if result['controller']['grounded']:
                    break
                c.call('step',ticks=180)
                result=c.call('inspect_planet_runtime')
            assert result['distance_from_spawn_meters']>7000, result
            assert result['controller']['grounded'], result
            assert result['tile_births']>start['tile_births'] and result['tile_retirements']>0, result
            assert result['visual_tiles']<4096 and result['physical_tiles']<result['visual_tiles'], result
            legs.append(result)
            captures.append(c.call('capture', path=f'artifacts/planet-runtime/eye-{yaw}.png', width=1280, height=800))
        c.call('reset');c.call('set_player_view',view=view())
        c.call('step',ticks=120)
        third=view();third['mode']='third_person';third['pitch_degrees']=-20
        c.call('set_player_view',view=third)
        captures.append(c.call('capture',path='artifacts/planet-runtime/third-person.png',width=1280,height=800))
        # Save formats must fail rather than silently omit the new global state.
        c.call('save_portable_save',path='artifacts/planet-runtime/incomplete.ignis-portable',expected_error=True)
        c.call('save_checkpoint',path='artifacts/planet-runtime/incomplete.ignis-save',expected_error=True)
        if c.call('load_planet',path='artifacts/planet-runtime/earth.ignis-planet')['document'] != document:
            raise ValueError('Native source persistence must preserve the document')
        report={'build_id':initial['build_id'],'seed':seed,'recipe':recipe_path,'scene':SCENE,'config':CONFIG,'start':start,'legs':legs,
                'relief':relief,'captures':captures,'diameter_meters':document['diameter_meters'],'passed':True,'window':False}
        (OUT/'verification.json').write_text(json.dumps(report,indent=2))
        print(json.dumps({'verified':True,'seed':seed,'legs_meters':[p['distance_from_spawn_meters'] for p in legs]}),flush=True)
    finally:
        c.close()


def launch(binary):
    scene_hash=hashlib.sha256((ROOT/SCENE).read_bytes()).hexdigest()
    config_hash=hashlib.sha256((ROOT/CONFIG).read_bytes()).hexdigest()
    stamp=binary.stat();binary_stamp=[stamp.st_ino,stamp.st_size,stamp.st_mtime_ns]
    if LAUNCH.exists():
        old=json.loads(LAUNCH.read_text())
        probe=subprocess.run(['ps','-p',str(old['pid']),'-o','command='],capture_output=True,text=True,check=False)
        if probe.returncode==0 and str(binary) in probe.stdout:
            if old.get('scene')==SCENE and old.get('scene_sha256')==scene_hash and old.get('config_sha256')==config_hash and old.get('binary_stamp')==binary_stamp:
                print(json.dumps({'pid':old['pid'],'already_running':True}),flush=True)
                return
            raise RuntimeError('The previous Ignis demo is still open with another scene/build; close its window before replacing it')
    c=Client(binary,window=True,detached=True,log_path=OUT/'native.log',
             startup_args=('--scene',SCENE,'--continuous','--planet-runtime',CONFIG,'--first-person','player'))
    try:
        init(c)
        c.call('set_player_view',view=view());c.call('play')
        c.call('set_viewport_performance',settings={'enabled':True,'vsync':True})
        report={'pid':c.process.pid,'scene':SCENE,'planet_runtime':CONFIG,'scene_sha256':scene_hash,
                'config_sha256':config_hash,'binary_stamp':binary_stamp,'world':c.call('inspect_planet_runtime'),
                'controls':'WASD, Shift 600m/s, click+mouse, Tab first/third, Esc release mouse, P pause, R reset planet spawn'}
        LAUNCH.parent.mkdir(parents=True,exist_ok=True)
        LAUNCH.write_text(json.dumps(report,indent=2))
        print(json.dumps({'opened':True,'pid':c.process.pid,'planet_runtime':True}),flush=True)
    except BaseException:
        c.close();raise
    else:
        c.process.stdin.close();c.process.stdout.close();c.log.close()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--seed',type=int,default=731199)
    p.add_argument('--recipe',default='examples/earth-mountain-chains.recipe.json')
    p.add_argument('--prepare-only',action='store_true')
    p.add_argument('--skip-prepare',action='store_true')
    a=p.parse_args()
    if not a.skip_prepare:prepare(ROOT/'target/release/ignis',a.seed,a.recipe)
    if not a.prepare_only:launch(native_entry())
