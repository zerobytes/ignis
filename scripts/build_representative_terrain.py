#!/usr/bin/env python3
"""Native regional source and real collision; vegetation is a separate additive stage."""
import argparse,bisect,copy,json,math,struct,subprocess,threading,time
from pathlib import Path
from verify_lab import Client,ROOT
from verify_terrain import position
OUT=ROOT/'artifacts/representative-world'
PREFIX='artifacts/representative-world/'

def source():
    ridges=[]
    for center,angle,width,depth,height,phase,exponent,folds in [((0,-400),0,1800,1000,380,.3,1.15,20),((350,50),-75,1500,800,260,1.7,1.35,16),((-280,-20),62,1450,800,220,2.8,1.45,12)]:
        ridges.append(dict(kind='ridge',center=position(*center),azimuth_degrees=angle,width_meters=width,depth_meters=depth,height_meters=height,phase=phase,crest_wander=.13,slope_exponent=exponent,folds_meters=folds,end_fade=.2))
    river=[(20,80),(30,150),(90,400),(-40,650),(0,900)]
    doc=dict(version=2,space_version=1,id='representative_island',revision=1,datum=position(0,0),base=dict(kind='constant',height_meters=-30),operations=[
        dict(kind='raise',center=position(0,0),radius_meters=900,amount_meters=95),
        dict(kind='noise',seed=183,period_meters=512,octaves=4,persistence=.48,amplitude_meters=18,style='smooth'),
        dict(kind='noise',seed=184,period_meters=64,octaves=3,persistence=.4,amplitude_meters=2,style='smooth'),
        *ridges,
        dict(kind='raise',center=position(-220,350),radius_meters=220,amount_meters=35),
        dict(kind='channel',points=[position(*p) for p in river],width_meters=180,bed_start_meters=-2,bed_end_meters=-3,blend=1),
        dict(kind='level',center=position(145,350),radius_meters=50,height_meters=20,slope=[0,0],blend=1)],water_level_meters=.05)
    return dict(document=doc,region=dict(origin=position(-1024,-1024),cells=[512,512],spacing_meters=4),frame_origin=position(0,0),river_points=[list(p) for p in river],spawn_xz=[145,350],description='Finite native island: real ridges/valley/channel bed; flat visual water (no fluid), no streaming. Landscape179 is the shape/lighting direction, not a byte-identical scene.')

def surface(scene,x,z):
    i=next(i for i in scene['meshes']['instances'] if i['id']=='terrain_surface')
    a=next(a for a in scene['meshes']['assets'] if a['id']==i['asset'])
    g=a['geometry'];cx=cz=512;step=4
    fx=(x-i['position'][0])/step;fz=(z-i['position'][2])/step
    ix=min(cx-1,max(0,math.floor(fx)));iz=min(cz-1,max(0,math.floor(fz)))
    u=fx-ix;v=fz-iz;base=iz*(cx+1)+ix
    aa,b,c,d=[g['positions'][k][1] for k in [base,base+1,base+cx+1,base+cx+2]]
    h=aa+(b-aa)*u+(c-aa)*v if u+v<=1 else d+(c-d)*(1-u)+(b-d)*(1-v)
    return h+i['position'][1]

def build(binary):
    OUT.mkdir(parents=True,exist_ok=True);recipe=source()

    c=Client(binary,log_path=OUT/'terrain-runtime.log')
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='representative-terrain',version='1')));c.request('notifications/initialized',notification=True)
        raw=copy.deepcopy(recipe['document']);raw['operations'].pop()
        height=c.call('query_terrain',query=dict(kind='sample',document=raw,points=[position(*recipe['spawn_xz'])]))['samples'][0]['height_meters']
        recipe['document']['operations'][-1]['height_meters']=height
        (ROOT/'examples/representative-world.recipe.json').write_text(json.dumps(recipe,indent=2)+'\n')
        c.call('generate',recipe=dict(width=30,depth=30,seed=183,obstacles=0))
        scene=c.call('inspect')['scene'];player=copy.deepcopy(next(e for e in scene['entities'] if e['id']=='player'))
        player.pop('footsteps',None);player['character_controller']=dict(max_slope_degrees=40,slide_slope_degrees=45,snap_distance=.2)
        c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in scene['entities']])
        material=dict(pattern=dict(kind='ridged',frequency=[.01,.015,.01],low=[.38,.43,.31],high=[1,.94,.80],bump=.04,warp=.45,octaves=4,seed=47,contrast=1.4))
        result=c.call('publish_terrain_region',publication=dict(document=recipe['document'],region=recipe['region'],frame_origin=recipe['frame_origin'],asset='terrain_surface',instance='terrain_surface',color=[.29,.37,.19],material=material,collision=True))
        assert not result['visual_only'] and result['terrain']['vertices']==263169
        c.call('save_terrain',path='examples/representative-world.ignis-terrain',document=recipe['document'])
        c.call('save_scene',path=PREFIX+'terrain.scene.json')
        scene=json.loads((OUT/'terrain.scene.json').read_text())
        h=surface(scene,*recipe['spawn_xz']);x,z=recipe['spawn_xz']
        player['transform']=dict(position=[x,h+.94,z],size=[.6,1.8,.6]);player['name']='Explorador — personagem controlável'
        water=dict(id='ocean_surface',geometry=dict(positions=[[-9000,0,-9000],[9000,0,-9000],[-9000,0,9000],[9000,0,9000]],triangles=[[0,2,1],[1,2,3]]),shading='smooth',material=dict(reflectance=dict(roughness=.22,f0=.025)))
        wi=dict(id='ocean_surface',asset='ocean_surface',position=[0,.05,0],rotation=[0,0,0,1],scale=3,color=[.035,.19,.25],playback=None)
        c.call('edit_scene',edits=[dict(op='upsert',entity=player),dict(op='mesh',edit=dict(kind='put_asset',asset=water)),dict(op='mesh',edit=dict(kind='put_instance',instance=wi))])
        play=dict(position=[x,h+55,z+100],target=[0,h+70,z-300],fov_degrees=75)
        panorama=dict(position=[930,820,1080],target=[0,160,-50],fov_degrees=55)
        light=dict(direction=[.65,-1,-.25],ambient=.25,direct_intensity=1,atmosphere=dict(density=.00006,color=[.73,.82,.94],sun_color=[.95,.73,.40],sun_strength=.15,layers=dict(sky_color=[.73,.82,.94],mountain_color=[.008,.025,.055],detail_start=500,detail_end=1800,influence=.65),reduce_details=True,silhouette_error_px=.5,color_error=.002))
        c.call('set_camera',camera=play);c.call('set_light',light=light)
        c.call('save_scene',path=PREFIX+'terrain.scene.json')
        placement=dict(bounds_xz=dict(min=[-1024,-1024],max=[1024,1024]),terrain_instances=['terrain_surface'],water_instances=['ocean_surface'],views=dict(play=play,panorama=panorama),spawn=player['transform']['position'],clearings=[dict(center=[x,z],radius_meters=14)],river_channel=dict(points=recipe['river_points'],bed_start_meters=-2,bed_end_meters=-3,radius_meters=180,water_level_meters=.05,simulation=False),source_sha256=result['terrain']['source_sha256'],source_revision=1,spacing_meters=4,cells=[512,512],physical_region_only=True,water_visual_only=True)
        (OUT/'placement.json').write_text(json.dumps(placement,indent=2)+'\n')
        # Freeze base before inspections; camera changes below are session-authoring actions only.
        c.call('capture',path=PREFIX+'terrain-play.png',width=1728,height=1117)
        c.call('set_camera',camera=panorama);c.call('capture',path=PREFIX+'terrain-panorama.png',width=1728,height=1117)
        c.call('load_scene',path=PREFIX+'terrain.scene.json')
        c.call('step',ticks=45)
        before=c.call('inspect')['runtime_positions']['player']
        c.call('set_input',input=dict(x=0,z=-1));c.call('step',ticks=180)
        now=c.call('inspect');after=now['runtime_positions']['player']
        assert before[2]-after[2]>5,(before,after)
        assert now['character_controllers']['player']['grounded'],after
        report=dict(passed=True,build_id=now['build_id'],source=result['terrain'],spawn_before=before,walk_after=after,walk_delta_meters=math.dist(before,after),native_window=False,artistic_acceptance=False,streaming=False,water_simulation=False)
        (OUT/'terrain-verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(dict(passed=True,build_id=report['build_id'],vertices=263169,triangles=524288,walk_delta_meters=report['walk_delta_meters'])))
    finally:c.close()


def expanded_source(extent=20000):
    """One source, conforming quadrants; resolution is static, not camera LOD."""
    if not 4000 <= extent <= 20000 or extent % 8:
        raise ValueError('Extent must be4000..20000m and a multiple of8')
    half=extent//2
    positive=list(range(0,1001,4))+list(range(1032,half,32))+[half]
    negative=[half-v for v in reversed(positive)]
    regions=[]
    for z in [-1,1]:
        for x in [-1,1]:
            axes=dict(x=negative if x<0 else positive,z=negative if z<0 else positive)
            regions.append(dict(id=f'terrain_{x+1}_{z+1}',region=dict(origin=position(-half if x<0 else 0,-half if z<0 else 0),cells=[len(axes['x'])-1,len(axes['z'])-1],spacing_meters=4,axis_offsets_meters=axes),origin_xz=[-half if x<0 else 0,-half if z<0 else 0]))
    ridges=[]
    for center,angle,width,depth,height,phase in [
        ((-1200,-2300),-12,6500,3500,1150,.3),
        ((3800,-3300),-58,7300,3400,1350,1.4),
        ((-5200,-2600),62,7200,4200,1450,2.5),
        ((-5400,3400),-25,7600,4400,1250,3.6),
        ((5200,3500),-65,8000,4200,1300,4.7),
        ((-1000,-7100),14,8200,3300,1650,5.8),
        ((2500,7200),25,8400,3600,1050,6.9),
        ((-7300,7200),-35,5300,3000,1100,8.1),
        ((7400,-7300),40,5200,3100,1400,9.2)]:
        ridges.append(dict(kind='ridge',center=position(*center),azimuth_degrees=angle,width_meters=width,depth_meters=depth,height_meters=height,phase=phase,crest_wander=.16,slope_exponent=1.35,folds_meters=height*.035,end_fade=.22))
    river=[(20,80),(80,800),(-400,2600),(400,6000),(1200,9400)]
    doc=dict(version=2,space_version=1,id='expanded_landscape',revision=1,datum=position(0,0),base=dict(kind='constant',height_meters=-70),operations=[
        dict(kind='raise',center=position(0,0),radius_meters=11000,amount_meters=270),
        dict(kind='noise',seed=185,period_meters=2048,octaves=4,persistence=.48,amplitude_meters=100,style='smooth'),
        dict(kind='noise',seed=186,period_meters=128,octaves=3,persistence=.4,amplitude_meters=4,style='smooth'),
        *ridges,
        dict(kind='channel',points=[position(*p) for p in river],width_meters=280,bed_start_meters=-2,bed_end_meters=-5,blend=1),
        dict(kind='level',center=position(650,400),radius_meters=100,height_meters=0,slope=[0,0],blend=1)],water_level_meters=.05)
    n=sum((r['region']['cells'][0]+1)*(r['region']['cells'][1]+1) for r in regions)
    triangles=sum(2*r['region']['cells'][0]*r['region']['cells'][1] for r in regions)
    return dict(document=doc,regions=regions,extent_meters=extent,frame_origin=position(0,0),spawn_xz=[650,400],river_points=river,
                estimate=dict(vertices=n,triangles=triangles,positions_indices_bytes=n*12+triangles*12,material_coordinates_bytes=n*12),
                resolution=dict(near_bounds_xz=[[-1000,-1000],[1000,1000]],near_meters=4,far_meters=32,static=True),
                streaming=False,water_simulation=False)


def triangle_height(c,recipe,x,z):
    """Same f32 vertices and b-c diagonal as native publication; no giant inspect."""
    for item in recipe['regions']:
        ox,oz=item['origin_xz'];axes=item['region']['axis_offsets_meters']
        u,v=x-ox,z-oz
        if not (0<=u<=axes['x'][-1] and 0<=v<=axes['z'][-1]):continue
        col=min(len(axes['x'])-2,bisect.bisect_right(axes['x'],u)-1)
        row=min(len(axes['z'])-2,bisect.bisect_right(axes['z'],v)-1)
        x0,x1=axes['x'][col:col+2];z0,z1=axes['z'][row:row+2]
        points=[position(ox+a,oz+b) for a,b in [(x0,z0),(x1,z0),(x0,z1),(x1,z1)]]
        samples=c.call('query_terrain',query=dict(kind='sample',document=recipe['document'],points=points))['samples']
        a,b,cc,d=[struct.unpack('f',struct.pack('f',p['height_meters']))[0] for p in samples]
        u=(u-x0)/(x1-x0);v=(v-z0)/(z1-z0)
        return a+(b-a)*u+(cc-a)*v if u+v<=1 else d+(cc-d)*(1-u)+(b-d)*(1-v)
    raise ValueError('Point outside physical region')


def build_expanded(binary,extent=20000,captures=True):
    out=ROOT/'artifacts/expanded-world';out.mkdir(parents=True,exist_ok=True)
    prefix=str(out.relative_to(ROOT))+'/'
    recipe=expanded_source(extent);stages=[];peak=[0];stop=threading.Event()
    c=Client(binary,log_path=out/'terrain-runtime.log')
    def process_rss():
        p=subprocess.run(['ps','-p',str(c.process.pid),'-o','rss='],capture_output=True,text=True)
        return int(p.stdout.strip())*1024 if p.stdout.strip() else 0
    def rss():
        while not stop.is_set():
            peak[0]=max(peak[0],process_rss())
            stop.wait(.25)
    monitor=threading.Thread(target=rss,daemon=True);monitor.start()
    def call(op,**kwargs):
        started=time.monotonic();answer=c.call(op,**kwargs);seconds=time.monotonic()-started
        resident=process_rss();peak[0]=max(peak[0],resident)
        stages.append(dict(op=op,seconds=round(seconds,6),resident_process_rss_bytes=resident,sampled_peak_rss_bytes=peak[0]))
        print(json.dumps(stages[-1]),flush=True);return answer
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='expanded-terrain',version='1')));c.request('notifications/initialized',notification=True)
        call('generate',recipe=dict(width=30,depth=30,seed=185,obstacles=0))
        initial=call('inspect');build_id=initial['build_id'];scene=initial['scene']
        player=copy.deepcopy(next(e for e in scene['entities'] if e['id']=='player'))
        player.pop('footsteps',None);player['character_controller']=dict(max_slope_degrees=40,slide_slope_degrees=45,snap_distance=.2)
        call('set_execution_mode',mode='continuous')
        call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in scene['entities']])
        raw=copy.deepcopy(recipe['document']);raw['operations'].pop()
        h=call('query_terrain',query=dict(kind='sample',document=raw,points=[position(*recipe['spawn_xz'])]))['samples'][0]['height_meters']
        recipe['document']['operations'][-1]['height_meters']=h
        (ROOT/'examples/expanded-world.recipe.json').write_text(json.dumps(recipe,indent=2)+'\n')
        material=dict(pattern=dict(kind='ridged',frequency=[.01,.015,.01],low=[.38,.43,.31],high=[1,.94,.80],bump=.04,warp=.45,octaves=4,seed=47,contrast=1.4))
        publications=[]
        for item in recipe['regions']:
            answer=call('publish_terrain_region',publication=dict(document=recipe['document'],region=item['region'],frame_origin=recipe['frame_origin'],asset=item['id'],instance=item['id'],color=[.29,.37,.19],material=material,collision=True,world_material_coordinates=True))
            assert not answer['visual_only'];publications.append(answer['terrain'])
        assert len({p['source_sha256'] for p in publications})==1
        assert sum(p['vertices'] for p in publications)==recipe['estimate']['vertices']
        assert sum(p['triangles'] for p in publications)==recipe['estimate']['triangles']
        call('save_terrain',path='examples/expanded-world.ignis-terrain',document=recipe['document'])
        x,z=recipe['spawn_xz'];h=triangle_height(c,recipe,x,z)
        player['transform']=dict(position=[x,h+.94,z],size=[.6,1.8,.6]);player['name']='Explorador — terreno20km'
        water=dict(id='ocean_surface',geometry=dict(positions=[[-9000,0,-9000],[9000,0,-9000],[-9000,0,9000],[9000,0,9000]],triangles=[[0,2,1],[1,2,3]]),shading='smooth',material=dict(reflectance=dict(roughness=.22,f0=.025)))
        wi=dict(id='ocean_surface',asset='ocean_surface',position=[0,.05,0],rotation=[0,0,0,1],scale=3,color=[.035,.19,.25],playback=None)
        call('edit_scene',edits=[dict(op='upsert',entity=player),dict(op='mesh',edit=dict(kind='put_asset',asset=water)),dict(op='mesh',edit=dict(kind='put_instance',instance=wi))])
        eye=[x,h+1.69,z];play=dict(position=eye,target=[x,h+1.69,z-10],fov_degrees=75)
        panorama=dict(position=[6500,5200,9000],target=[0,300,-1500],fov_degrees=65)
        light=dict(direction=[.65,-1,-.25],ambient=.25,direct_intensity=1,atmosphere=dict(density=.00006,color=[.73,.82,.94],sun_color=[.95,.73,.40],sun_strength=.15,layers=dict(sky_color=[.73,.82,.94],mountain_color=[.008,.025,.055],detail_start=500,detail_end=1800,influence=.65),reduce_details=True,silhouette_error_px=.5,color_error=.002))
        call('set_camera',camera=play);call('set_light',light=light)
        call('save_scene',path=prefix+'terrain.scene.json')
        half=extent/2
        player_view=dict(actor='player',yaw_degrees=0,pitch_degrees=0,eye_height=1.65,fov_degrees=75)
        placement=dict(bounds_xz=dict(min=[-half,-half],max=[half,half]),terrain_instances=[r['id'] for r in recipe['regions']],water_instances=['ocean_surface'],regions=recipe['regions'],views=dict(play=play,panorama=panorama),player_view=player_view,spawn=player['transform']['position'],clearings=[dict(center=[x,z],radius_meters=14)],river_channel=dict(points=recipe['river_points'],radius_meters=280,water_level_meters=.05,simulation=False),source_sha256=publications[0]['source_sha256'],source_revision=1,spacing_meters=4,resolution=recipe['resolution'],physical_region_only=True,water_visual_only=True,forest_zones=[dict(id='near',bounds_xz=dict(min=[-1000,-1000],max=[1000,1000]),trees=12000),dict(id='outer',bounds_xz=dict(min=[-half,-half],max=[half,half]),exclude_bounds_xz=dict(min=[-1000,-1000],max=[1000,1000]),trees=18000)])
        (out/'placement.json').write_text(json.dumps(placement,indent=2)+'\n')
        if captures:
            call('set_player_view',view=player_view)
            call('capture',path=prefix+'terrain-play.png',width=1728,height=1117)
            call('set_player_view',view=None)
            call('set_camera',camera=panorama);call('capture',path=prefix+'terrain-panorama.png',width=1728,height=1117)
        call('load_scene',path=prefix+'terrain.scene.json')
        walks=[]
        for label,xx,zz,dx,dz in [('spawn',x,z,0,-1),('seam_x',-3,-700,1,0),('seam_z',-800,-3,0,1),('far',-8000,-2000,0,-1)]:
            if abs(xx)>=half-32 or abs(zz)>=half-32:continue
            hh=triangle_height(c,recipe,xx,zz);probe=copy.deepcopy(player);probe['transform']['position']=[xx,hh+.94,zz]
            call('upsert',entity=probe);call('step',ticks=40)
            before=call('inspect_world')['positions']['player']
            call('set_input',input=dict(x=dx,z=dz));started=time.monotonic();call('step',ticks=180);step_seconds=time.monotonic()-started
            after=call('inspect_world')['positions']['player'];controller=call('inspect_character_controllers')['controllers']['player']
            distance=math.dist(before,after);progress=(after[0]-before[0])*dx+(after[2]-before[2])*dz
            assert distance>3 and progress>3 and controller['grounded'],(label,before,after,controller)
            if label=='seam_x':assert before[0]<0<after[0],(label,before,after)
            if label=='seam_z':assert before[2]<0<after[2],(label,before,after)
            walks.append(dict(label=label,before=before,after=after,distance_meters=distance,progress_meters=progress,grounded=controller['grounded'],step180_seconds=step_seconds))
        call('load_scene',path=prefix+'terrain.scene.json')
        report=dict(passed=True,build_id=build_id,estimate=recipe['estimate'],extent_meters=extent,publications=publications,walks=walks,stages=stages,sampled_peak_process_rss_bytes=peak[0],rss_sample_interval_seconds=.25,rss_is_total_unified_memory=False,scene_file_bytes=(out/'terrain.scene.json').stat().st_size,streaming=False,maximum_universal=False,native_window=False,water_simulation=False,artistic_acceptance=False)
        (out/'terrain-verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(dict(passed=True,build_id=build_id,extent_meters=extent,estimate=recipe['estimate'],peak_rss_bytes=peak[0])),flush=True)
    finally:
        stop.set();monitor.join(timeout=2);c.close()


def measure_expanded_residency(binary):
    """Fresh continuous host: no editor undo history and no GPU/capture device."""
    out=ROOT/'artifacts/expanded-world'
    c=Client(binary,log_path=out/'terrain-residency-runtime.log')
    def rss():
        p=subprocess.run(['ps','-p',str(c.process.pid),'-o','rss='],capture_output=True,text=True)
        return int(p.stdout.strip())*1024
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='expanded-residency',version='1')));c.request('notifications/initialized',notification=True)
        build_id=c.call('inspect')['build_id'];c.call('set_execution_mode',mode='continuous')
        started=time.monotonic();c.call('load_scene',path='artifacts/expanded-world/terrain.scene.json');load_seconds=time.monotonic()-started
        after_load=rss();c.call('step',ticks=40)
        before=c.call('inspect_world')['positions']['player'];c.call('set_input',input=dict(x=0,z=-1))
        started=time.monotonic();c.call('step',ticks=600);step_seconds=time.monotonic()-started
        world=c.call('inspect_world');controller=c.call('inspect_character_controllers')['controllers']['player'];execution=c.call('inspect_execution')
        after=world['positions']['player'];assert math.dist(before,after)>20 and controller['grounded']
        result=dict(build_id=build_id,load_seconds=load_seconds,resident_after_load_bytes=after_load,resident_after640ticks_bytes=rss(),step600_seconds=step_seconds,tick=world['tick'],before=before,after=after,distance_meters=math.dist(before,after),grounded=controller['grounded'],execution=execution,graphics_device_created=False,rss_is_total_unified_memory=False,passed=True)
        (out/'terrain-residency-verification.json').write_text(json.dumps(result,indent=2)+'\n')
        print(json.dumps(result),flush=True)
        return result
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--expanded',action='store_true');p.add_argument('--plan-expanded',action='store_true');p.add_argument('--extent-meters',type=int,default=20000);p.add_argument('--no-captures',action='store_true');a=p.parse_args()
    if a.plan_expanded:print(json.dumps(expanded_source(a.extent_meters)['estimate']))
    elif a.expanded:
        build_expanded(a.binary.resolve(),a.extent_meters,not a.no_captures)
        measure_expanded_residency(a.binary.resolve())
    else:build(a.binary.resolve())
