#!/usr/bin/env python3
"""Prepare and open seeded local walking on the real Earth-radius mesh, through MCP.
Performance: source mesh loaded once from buffered Scene; collision/actor/markers
are applied in one cold transaction. No per-frame script, geometry transmission,
new binary, alternate build, socket or change to global gameplay gravity.
"""
import argparse
import copy
import hashlib
import subprocess
import json
from pathlib import Path
from open_world_walk import native_entry
from verify_lab import Client, ROOT

OUT=ROOT/'artifacts/earth-walk'
SCENE='artifacts/earth-walk/walk.scene.json'

def init(c):
    c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'earth-walk','version':'1'}})
    c.request('notifications/initialized',notification=True)

def view():
    return dict(actor='player',yaw_degrees=0,pitch_degrees=0,eye_height=1.65,fov_degrees=60,mode='first_person')

def demo_locomotion():
    # This fixture alone opts into 100x sprint. Engine defaults remain 4/1.5/6.
    return {'walk':{'speed':4,'noise_multiplier':1},
            'sneak':{'speed':1.5,'noise_multiplier':.2},
            'run':{'speed':600,'noise_multiplier':1.8}}

def prepare(binary):
    OUT.mkdir(parents=True,exist_ok=True)
    c=Client(binary,log_path=OUT/'prepare.log')
    try:
        init(c)
        initial=c.call('inspect')
        player=copy.deepcopy(next(e for e in initial['scene']['entities'] if e['id']=='player'))
        post=copy.deepcopy(next(e for e in initial['scene']['entities'] if e.get('body')=='fixed'))
        c.call('set_execution_mode',mode='continuous')
        c.call('load_scene',path='artifacts/earth-horizon/native-eye.scene.json')
        meshes=c.call('inspect_meshes')
        player['transform']={'position':[0,.91,0],'size':[.6,1.8,.6]}
        player['character_controller']={'max_slope_degrees':45,'slide_slope_degrees':50,'offset':.01,'snap_distance':.2}
        player['footsteps']=None
        player['locomotion']=demo_locomotion()
        edits=[]
        for i in meshes['instances']:
            i['color']=[.28,.46,.25]
            edits += [{'op':'mesh','edit':{'kind':'put_instance','instance':i}},
                      {'op':'set_mesh_collision','instance':i['id'],'enabled':True}]
        edits.append({'op':'upsert','entity':player})
        for name,x,z,height,color in [('origin_marker',-3,-8,2,[.9,.75,.15]),('north_marker',0,-30,3,[.2,.45,.9]),('east_marker',20,-15,4,[.8,.25,.2]),('west_marker',-20,-40,4,[.6,.25,.8])]:
            p=copy.deepcopy(post)
            p.update(id=name,name=name,color=color)
            # Reference-sphere y at this local XZ, stable difference of squared radii.
            d2=x*x+z*z
            surface=-d2/(6_371_000+(6_371_000**2-d2)**.5)
            p['transform']={'position':[x,surface+height*.5,z],'size':[.6,height,.6]}
            edits.append({'op':'upsert','entity':p})
        edits.append({'op':'set_camera','camera':{'position':[0,1.66,0],'target':[0,1.66,-100],'fov_degrees':60}})
        c.call('edit_scene',edits=edits)
        c.call('save_scene',path=SCENE)
        c.call('set_player_view',view=view())
        c.call('step',ticks=120)
        start=c.call('inspect_world')['positions']['player']
        c.call('set_player_view_input',input={'x':0,'z':-1})
        c.call('step',ticks=240)
        c.call('set_player_view_input',input={'x':0,'z':0})
        end=c.call('inspect_world')['positions']['player']
        controller=c.call('inspect_character_controllers')
        distance=sum((a-b)**2 for a,b in zip(start,end))**.5
        assert distance>12,{'start':start,'end':end,'distance':distance}
        # inspect_character_controllers schema is retained as evidence; grounding checked below.
        assert controller['controllers']['player']['grounded'],controller
        c.call('capture',path='artifacts/earth-walk/verified.png',width=1280,height=800)
        report={'build_id':initial['build_id'],'scene':SCENE,'start':start,'end':end,'distance_meters':distance,
                'controllers':controller,'local_walk_only':True,'radial_gravity':False,'native_window':False}
        (OUT/'verification.json').write_text(json.dumps(report,indent=2))
        print(json.dumps({'prepared':SCENE,'walk_meters':distance,'grounded':True}),flush=True)
    finally:
        c.close()

def launch(binary):
    OUT.mkdir(parents=True,exist_ok=True)
    scene_hash=hashlib.sha256((ROOT/SCENE).read_bytes()).hexdigest()
    stamp=binary.stat()
    binary_stamp=[stamp.st_ino,stamp.st_size,stamp.st_mtime_ns]
    launch_file=OUT/'launch.json'
    if launch_file.exists():
        previous=json.loads(launch_file.read_text())
        probe=subprocess.run(['ps','-p',str(previous['pid']),'-o','command='],capture_output=True,text=True,check=False)
        if probe.returncode==0 and str(binary) in probe.stdout and '--scene '+SCENE in probe.stdout:
            if previous.get('scene_sha256')!=scene_hash or previous.get('binary_stamp')!=binary_stamp:
                raise RuntimeError('The running demo uses another scene/build; close its window before replacing it')
            print(json.dumps({'pid':previous['pid'],'already_running':True,'scene':SCENE}),flush=True)
            return
    c=Client(binary,window=True,log_path=OUT/'native.log',detached=True,
             startup_args=('--scene',SCENE,'--continuous','--first-person','player'))
    try:
        init(c)
        c.call('set_player_view',view=view())
        c.call('play')
        c.call('set_viewport_performance',settings={'enabled':True,'vsync':True})
        verification=json.loads((OUT/'verification.json').read_text()) if (OUT/'verification.json').exists() else {}
        result={'pid':c.process.pid,'scene':SCENE,'scene_sha256':scene_hash,'binary_stamp':binary_stamp,'seed':verification.get('seed'),'view':c.call('inspect_player_view'),
                'execution':c.call('inspect_execution'),'local_walk_only':True,
                'controls':'Clique+mouse, WASD, Shift corre 100x (600m/s), Tab, Esc solta mouse, P pausa, R reinicia'}
        (OUT/'launch.json').write_text(json.dumps(result,indent=2))
        print(json.dumps({'pid':c.process.pid,'opened':True,'scene':SCENE}),flush=True)
    except BaseException:
        c.close()
        raise
    else:
        # Window outlives MCP stdin EOF and the launching terminal process.
        c.process.stdin.close();c.process.stdout.close();c.log.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--prepare-only',action='store_true')
    p.add_argument('--skip-prepare',action='store_true')
    p.add_argument('--seed',type=int,help='Procedural relief seed; default from examples/earth-procedural.recipe.json')
    p.add_argument('--flat',action='store_true',help='Reproduce the earlier uniform-sphere walking fixture')
    a=p.parse_args()
    if not a.skip_prepare:
        if a.flat:prepare(ROOT/'target/release/ignis')
        else:
            from build_earth_procedural import prepare as prepare_procedural
            prepare_procedural(ROOT/'target/release/ignis',a.seed)
    if not a.prepare_only:launch(native_entry())
