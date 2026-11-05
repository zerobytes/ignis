#!/usr/bin/env python3
"""Apply catalog materials, verify offline noclip, and open ONE native inspection window."""
import argparse,copy,hashlib,json,math,subprocess,time
from pathlib import Path
from verify_lab import Client,ROOT
from open_earth_walk import init
from open_world_walk import native_entry
PREFIX='artifacts/volumetric-terrain-inspection';OUT=ROOT/PREFIX
SCENE=f'{PREFIX}/world.scene.json'
VIEW=dict(actor='player',yaw_degrees=25,pitch_degrees=-12,eye_height=1.45,fov_degrees=70,mode='third_person',third_person=dict(distance_meters=8,shoulder_offset=.4,collision_radius=.15))
FLIGHT=dict(speed=100,fast_speed=600,follow_pitch=True,near_clip=.5)

def prepare():
    OUT.mkdir(parents=True,exist_ok=True)
    rock_path=ROOT/'assets/materials/rock/fractured.ignis-material'
    rock=json.loads(rock_path.read_text())['material']
    green=json.loads((ROOT/'assets/materials/terrain/plateau-green.ignis-material').read_text())['material']
    catalog=json.loads((ROOT/'website/dist/catalog/data/catalog.json').read_text())
    entry=next(a for a in catalog['assets'] if a['id']=='ignis.material.rock.fractured')
    sha=hashlib.sha256(rock_path.read_bytes()).hexdigest();assert entry['files'][0]['sha256']==sha
    rock['procedural']['recipe']['tile_size']=[2,2]  # Application scale; catalog source bytes stay unchanged.
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'prepare.log')
    try:
        init(c);c.call('load_scene',path='artifacts/volumetric-terrain-landscape/world.scene.json')
        original=c.call('inspect_meshes');rows=copy.deepcopy(original['instances']);reports=[]
        for row in rows:
            name=row['id']
            style=dict(instance=name,wall_asset=f'{name}_rock',top_asset=f'{name}_green',top_instance=f'{name}_green',min_up=.94,
                chamfer=dict(fraction=.16,variation=.75,seed=2718),
                terrace_lift=dict(spacing=7.6923076923*8,factor=1,phase=0,random_heights=dict(min=3,max=15,seed=2718)),
                wall_rock=dict(amplitude=.65,stratum_height=.9,joint_width=24,seed=2718),
                walls=rock,top=green,wall_color=[1,1,1],top_color=[1,1,1])
            if not reports:
                plain=copy.deepcopy(style);plain['wall_rock']['amplitude']=0
                c.call('style_mesh_by_slope',style=plain)
                baseline=c.call('inspect_mesh',asset=style['wall_asset'])['asset']['geometry']
                c.call('undo')
            result=c.call('style_mesh_by_slope',style=style);reports.append(result['partition'])
            if len(reports)==1:
                sculpted=c.call('inspect_mesh',asset=style['wall_asset'])['asset']['geometry']
                wall_geometry_differs=baseline!=sculpted
                assert wall_geometry_differs,'Rock relief did not change actual wall geometry'

        assert sum(r['input_triangles'] for r in reports)==182714
        assert sum(r['source_triangles'] for r in reports)==sum(r['wall_triangles']+r['top_triangles'] for r in reports)
        assert len(c.call('inspect_meshes')['instances'])==32
        before=c.call('inspect')['revision'];bad=copy.deepcopy(style);bad['min_up']=-1
        c.call('style_mesh_by_slope',style=bad,expected_error=True);assert c.call('inspect')['revision']==before
        # Read-only selection of a sun-facing stone wall near the central region.
        candidate=None
        for row in sorted(rows,key=lambda r:(r['position'][0]+1250)**2+(r['position'][2]+1250)**2)[:4]:
            assert row['rotation']==[0,0,0,1] and row['scale']==1
            g=c.call('inspect_mesh',asset=row['id']+'_rock')['asset']['geometry'];offset=row['position']
            for face in g['triangles']:
                pts=[[g['positions'][v][i]+offset[i] for i in range(3)] for v in face]
                a=[pts[1][i]-pts[0][i] for i in range(3)];b=[pts[2][i]-pts[0][i] for i in range(3)]
                n=[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];length=math.sqrt(sum(v*v for v in n))
                if length<10 or max(p[1] for p in pts)-min(p[1] for p in pts)<.8:continue
                n=[v/length for v in n]
                if n[2]>-.8 or abs(n[1])>.35 or sum(n[i]*[-.5,.78,-.36][i] for i in range(3))<.2:continue
                center=[sum(q[i] for q in pts)/3 for i in range(3)];score=-(max(p[1] for p in pts)-min(p[1] for p in pts))+(center[0]**2+center[2]**2)*1e-12
                if candidate is None or score<candidate[0]:candidate=(score,center,n)
        assert candidate is not None,'No suitable inspection wall in the central blocks'
        _,center,normal=candidate
        horizontal=math.hypot(normal[0],normal[2])
        spawn=[center[0]+normal[0]/horizontal*6,center[1],center[2]+normal[2]/horizontal*6]
        VIEW['yaw_degrees']=math.degrees(math.atan2(-normal[0],normal[2]));VIEW['pitch_degrees']=0
        scene=c.call('inspect')['scene'];player=copy.deepcopy(next(e for e in scene['entities'] if e['id']=='player'))
        player['transform']['position']=spawn;player['footsteps']=None
        c.call('edit_scene',edits=[dict(op='upsert',entity=player)])
        c.call('set_camera',camera=dict(position=[8200,7200,-8900],target=[0,-500,0],fov_degrees=55))
        c.call('save_scene',path=SCENE)
        c.call('capture',path=f'{PREFIX}/landscape.png',width=1600,height=1000,clip=dict(near=20,far=30000,reverse_z=False))
        c.call('load_scene',path=SCENE)
        c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=1000,clip=dict(near=20,far=30000,reverse_z=False))
        assert (OUT/'landscape.png').read_bytes()==(OUT/'reload.png').read_bytes()
        c.call('set_execution_mode',mode='continuous');c.call('set_player_view',view=VIEW);c.call('set_inspection_flight',config=FLIGHT)
        def pos():return c.call('inspect_player_view')['resolved']['pivot']
        authored=c.call('inspect')['scene'];p=pos();c.call('step',ticks=60);assert math.dist(p,pos())<.001
        c.call('set_inspection_flight_vertical',value=1);c.call('step',ticks=60);up=pos();assert abs(up[1]-p[1]-100)<.02
        c.call('set_inspection_flight_vertical',value=-1);c.call('step',ticks=60);assert math.dist(p,pos())<.02
        c.call('set_inspection_flight_vertical',value=0);c.call('set_player_view_input',input=dict(x=0,z=-1,mode='run'))
        c.call('step',ticks=60);moved=pos();assert abs(math.dist(p,moved)-600)<.2
        c.call('set_player_view_input',input=dict(x=0,z=0,mode='walk'));stationary=pos();c.call('step',ticks=60);assert math.dist(stationary,pos())<.001
        precision_start=pos();c.call('set_player_view_input',input=dict(x=1,z=0,mode='sneak'));c.call('step',ticks=60);precision_distance=math.dist(precision_start,pos());assert abs(precision_distance-2)<.02
        c.call('set_player_view_input',input=dict(x=0,z=0,mode='walk'))
        c.call('save_portable_save',path=f'{PREFIX}/blocked.ignis-portable',expected_error=True)
        c.call('set_inspection_flight_vertical',value=2,expected_error=True)
        assert c.call('inspect')['scene']==authored
        c.call('set_inspection_flight',config=None);c.call('load_scene',path=SCENE)
        c.call('set_player_view',view=VIEW);c.call('set_inspection_flight',config=FLIGHT)
        c.call('capture',path=f'{PREFIX}/flight.png',width=1600,height=1000)
        c.call('set_player_view',view={**VIEW,'mode':'first_person'})
        c.call('capture',path=f'{PREFIX}/first-person.png',width=1600,height=1000)
        c.call('set_inspection_flight',config=None);assert not c.call('inspect_inspection_flight')['active']
        report=dict(catalog_id=entry['id'],source_sha256=sha,material_overrides={'recipe.tile_size':[2,2]},partitions=reports,triangles=sum(r['source_triangles'] for r in reports),input_triangles=182714,
            walls=sum(r['wall_triangles'] for r in reports),tops=sum(r['top_triangles'] for r in reports),
            no_overlap=True,wall_geometry_differs_from_zero_relief=wall_geometry_differs,shape_preserved=False,terrace_lift=dict(spacing=7.6923076923*8,factor=1,phase=0,random_heights=dict(min=3,max=15,seed=2718)),chamfer=dict(fraction=.16,variation=.75,seed=2718),scene_reload_pixels_equal=True,hover=True,vertical_movement=True,
            fast_flight_distance=math.dist(p,moved),precision_flight_distance=precision_distance,neutral_input_stops=True,scene_unchanged_by_flight=True,persistence_rejected=True,
            native_window=False,human_acceptance=False,spawn=spawn,inspection_wall=dict(center=center,normal=normal),view=VIEW,flight=FLIGHT)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='partitions'}))
    finally:c.close()

def launch():
    OUT.mkdir(parents=True,exist_ok=True);binary=native_entry();path=OUT/'launch.json';stamp=binary.stat();signature=[stamp.st_ino,stamp.st_size,stamp.st_mtime_ns]
    if path.exists():
        old=json.loads(path.read_text());probe=subprocess.run(['ps','-p',str(old['pid']),'-o','command='],capture_output=True,text=True)
        if probe.returncode==0 and '--scene '+SCENE in probe.stdout:
            if old.get('binary_stamp')!=signature:raise RuntimeError('Previous inspection window uses an older build; preserve it until closed')
            print(json.dumps(dict(already_running=True,pid=old['pid'])));return
    c=Client(binary,window=True,log_path=OUT/'window.log',detached=True,startup_args=('--scene',SCENE,'--continuous','--third-person','player'))
    try:
        init(c);saved=json.loads((OUT/'verification.json').read_text());view=saved['view'];flight=saved['flight'];c.call('set_viewport_performance',settings=dict(enabled=False,vsync=False))
        c.call('set_player_view',view=view);c.call('set_inspection_flight',config=flight);c.call('play')
        c.call('set_viewport_performance',settings=dict(enabled=True,vsync=False))
        performance=None
        for _ in range(8):
            time.sleep(.3);performance=c.call('inspect_viewport_performance')
            if performance['sample']['intervals']>=4 and performance['sample']['seconds']>=.3:break
        assert c.call('inspect_inspection_flight')['active']
        result=dict(pid=c.process.pid,scene=SCENE,binary_stamp=signature,view=view,flight=flight,playing=True,presentation=performance,
            controls='Click+mouse; WASD; Space up; C down; Shift fast; Ctrl slow; Tab first/third person; Esc release')
        path.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
    except BaseException:c.close();raise
    else:c.process.stdin.close();c.process.stdout.close();c.log.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--open',action='store_true');p.add_argument('--prepared',action='store_true');a=p.parse_args()
    if not a.prepared:prepare()
    if a.open:launch()
