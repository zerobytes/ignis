#!/usr/bin/env python3
"""Native accumulating terraces and irregular natural connections; Python authors data only."""
import argparse,copy,json,math,time
from verify_lab import Client,ROOT
from verify_terrain import position
from open_earth_walk import init
from open_volume_inspection import launch
PREFIX='artifacts/natural-terraces';OUT=ROOT/PREFIX;SCENE=f'{PREFIX}/world.scene.json'

def recipe(seed=2718):
    settings=dict(seed=seed,radius_meters=240,tread_meters=40,minimum_step_meters=3,maximum_step_meters=15,
        aspect=1.1,contour_irregularity=.22,warp_meters=14,warp_period_meters=180,cliff_fraction=.04,top_detail_meters=.08,
        connections=[dict(azimuth_degrees=-100,width_meters=70,meander_radians=.45,meander_scale_meters=80),
                     dict(azimuth_degrees=30,width_meters=48,meander_radians=.7,meander_scale_meters=95)])
    return dict(version=4,space_version=1,id='natural_terraces',revision=1,datum=position(0,0),base=dict(kind='constant',height_meters=1),
        operations=[dict(kind='natural_terraces',center=position(0,0),settings=settings)],water_level_meters=None)

def prepare():
    OUT.mkdir(parents=True,exist_ok=True);doc=recipe();c=Client(ROOT/'target/release/ignis',log_path=OUT/'prepare.log');started=time.monotonic()
    try:
        init(c);route=c.call('query_terrain',query=dict(kind='terrace_connections',document=doc,operation=0,connection=0,from_radius=250,to_radius=0,samples=192))
        assert all(3<=h<=15 for h in route['steps']),route['steps']
        points=route['points'];slopes=[abs(b['height_meters']-a['height_meters'])/math.dist(a['local_xz'],b['local_xz']) for a,b in zip(points,points[1:])]
        assert max(slopes)<.8,max(slopes)
        large=copy.deepcopy(doc);large['operations'][0]['settings']['radius_meters']=10000
        tall=c.call('query_terrain',query=dict(kind='sample',document=large,points=[position(0,0)]))['samples'][0]['height_meters'];assert tall>1000,tall
        c.call('generate',recipe=dict(width=30,depth=30,seed=221,obstacles=0));scene=c.call('inspect')['scene']
        player=copy.deepcopy(next(e for e in scene['entities'] if e['id']=='player'));start=points[0]
        player['transform']=dict(position=[start['local_xz'][0],start['height_meters']+.95,start['local_xz'][1]],size=[.7,1.6,.7])
        player['character_controller']=dict(max_slope_degrees=45,slide_slope_degrees=50,offset=.01,snap_distance=.2);player['footsteps']=None
        c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in scene['entities'] if e['id']!='player']+[dict(op='upsert',entity=player)])
        rock=json.loads((ROOT/'assets/materials/rock/fractured.ignis-material').read_text())['material'];rock['procedural']['recipe']['tile_size']=[4,4]
        green=json.loads((ROOT/'assets/materials/terrain/plateau-green.ignis-material').read_text())['material']
        c.call('save_terrain',path=f'{PREFIX}/source.ignis-terrain',document=doc)
        assert c.call('load_terrain',path=f'{PREFIX}/source.ignis-terrain')['document']==doc
        patches=[];styled=[];plain=[]
        for z in range(3):
            for x in range(3):
                name=f'formation_{x}_{z}';patch=dict(origin=position(-300+x*200,-300+z*200),cells=[100,100],spacing_meters=2)
                c.call('publish_terrain',publication=dict(document=doc,patch=patch,stride=1,asset=name,instance=name,frame_origin=position(0,0),color=[1,1,1],material=green,collision=True))
                style=dict(instance=name,wall_asset=f'{name}_rock',top_asset=f'{name}_grass',top_instance=f'{name}_grass',min_up=.75,
                    chamfer=dict(fraction=.08,variation=.6,seed=2718),walls=rock,top=green,wall_color=[1,1,1],top_color=[1,1,1])
                try:result=c.call('style_mesh_by_slope',style=style)
                except AssertionError as e:
                    if 'requires both top and wall faces' not in str(e):raise
                    plain.append(name)
                else:styled.append(result['partition'])
                patches.append(patch)
            print(f'Published row {z+1}/3',flush=True)
        c.call('set_light',light=dict(direction=[.45,-.82,.32],ambient=.32,direct_intensity=1.15,direct_color=[1,.96,.9]))
        camera=dict(position=[460,340,-460],target=[0,24,0],fov_degrees=52)
        c.call('set_camera',camera=camera);c.call('save_scene',path=SCENE)
        c.call('capture',path=f'{PREFIX}/overview.png',width=1600,height=1000)
        c.call('load_scene',path=SCENE);c.call('capture',path=f'{PREFIX}/reload.png',width=1600,height=1000)
        assert (OUT/'overview.png').read_bytes()==(OUT/'reload.png').read_bytes()
        c.call('set_camera',camera=dict(position=[150,95,-260],target=[0,25,-30],fov_degrees=56));c.call('capture',path=f'{PREFIX}/connections.png',width=1600,height=1000)
        a,b=points[:2];dx=b['local_xz'][0]-a['local_xz'][0];dz=b['local_xz'][1]-a['local_xz'][1]
        view=dict(actor='player',yaw_degrees=math.degrees(math.atan2(dx,-dz)),pitch_degrees=-12,eye_height=1.45,fov_degrees=70,mode='third_person',third_person=dict(distance_meters=10,shoulder_offset=.4,collision_radius=.15))
        c.call('set_execution_mode',mode='continuous');c.call('set_player_view',view=view);c.call('step',ticks=90)
        def pos():return c.call('inspect_player_view')['resolved']['pivot']
        origin=pos();assert c.call('inspect_character_controllers')['controllers']['player']['grounded']
        walked=[];total_ticks=0
        for index,node in enumerate(points[1:]):
            for attempt in range(80):
                p=pos();delta=[node['local_xz'][0]-p[0],node['local_xz'][1]-p[2]];distance=math.hypot(*delta)
                if distance<.55:break
                c.call('set_input',input=dict(x=delta[0]/distance,z=delta[1]/distance,mode='walk'));c.call('step',ticks=4);total_ticks+=4
            else:raise AssertionError(('Walking route blocked',index,p,node))
            if index%16==0:walked.append(dict(index=index,position=pos(),expected_height=node['height_meters']))
        c.call('set_input',input=dict(x=0,z=0));c.call('step',ticks=30);end=pos();grounded=c.call('inspect_character_controllers')['controllers']['player']['grounded']
        assert grounded and end[1]-origin[1]>sum(route['steps'])-.5,(origin,end,grounded,route['steps'])
        c.call('capture',path=f'{PREFIX}/walk-end.png',width=1600,height=1000)
        c.call('load_scene',path=SCENE);c.call('set_execution_mode',mode='continuous');c.call('set_player_view',view=view);c.call('step',ticks=90)
        c.call('capture',path=f'{PREFIX}/start.png',width=1600,height=1000)
        report=dict(view=view,flight=None,steps=route['steps'],total_levels=route['total_levels'],maximum_route_slope=max(slopes),
            thousands_meters_query=tall,walking=dict(start=origin,end=end,rise=end[1]-origin[1],grounded=grounded,ticks=total_ticks,samples=walked,noclip=False),
            scene_reload_pixels_equal=True,styled=styled,all_green_tiles=plain,patches=patches,preparation_seconds=time.monotonic()-started,native_window=False,human_acceptance=False)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        (ROOT/'examples/natural-terraces.recipe.json').write_text(json.dumps(dict(document=doc,patches=patches,view=view,camera=camera),indent=2)+'\n')
        print(json.dumps({k:v for k,v in report.items() if k not in ('styled','patches')}))
    finally:c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--prepared',action='store_true');p.add_argument('--open',action='store_true');a=p.parse_args()
    if not a.prepared:prepare()
    if a.open:launch(PREFIX)
