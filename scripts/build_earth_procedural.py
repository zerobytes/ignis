#!/usr/bin/env python3
"""Seeded Earth-radius relief through the native Rust/MCP planet compiler.
Static performance review: small seeded recipe, native prepared sampling, one
publication per tile, one final physics transaction; no per-frame Python, per-
vertex JSON, new build/device/socket. Saved source and Scene remain independent.
"""
import argparse
import copy
import json
import math
import random
from pathlib import Path
from verify_lab import Client, ROOT

OUT=ROOT/'artifacts/earth-walk'
SCENE='artifacts/earth-walk/walk.scene.json'
RECIPE=ROOT/'examples/earth-procedural.recipe.json'


def init(c):
    c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'earth-procedural','version':'1'}})
    c.request('notifications/initialized',notification=True)


def direction(radius,x,z):
    return [x/radius, math.sqrt(1-(x*x+z*z)/(radius*radius)), z/radius]


def document_for(recipe,seed):
    if type(seed) is not int or not 0<=seed<=0xffffffff:
        raise ValueError('Seed must be a u32 integer')
    rng=random.Random(seed)
    radius=recipe['diameter_meters']/2
    d={'version':1,'space_version':1,'id':f'earth_seed_{seed}','revision':1,'center':copy.deepcopy(recipe['center']),
       'diameter_meters':recipe['diameter_meters'],'surface_gravity_mps2':recipe['gravity_mps2'],
       'base_height_meters':0,'operations':[]}
    for i,layer in enumerate(recipe['noise_layers']):
        d['operations'].append(dict(kind='noise',seed=(seed+0x9e3779b9*(i+1))&0xffffffff,**layer))
    def cap(x,z,width,height):
        width=min(width,math.hypot(x,z)-recipe['spawn_clear_radius_meters'])
        if width<math.radians(.001)*radius:
            raise ValueError('Recipe produced an unresolved cap near the spawn; adjust clear radius')
        d['operations'].append({'kind':'raise','direction':direction(radius,x,z),
                               'angular_radius_degrees':math.degrees(width/radius),'height_meters':height})
    peaks=recipe['peaks']
    for i in range(peaks['count']):
        angle=i*math.tau/peaks['count']+rng.uniform(-.15,.15)
        distance=rng.uniform(*peaks['distance_meters'])
        width=rng.uniform(*peaks['radius_meters'])
        height=rng.uniform(*peaks['height_meters'])
        x,z=math.sin(angle)*distance,-math.cos(angle)*distance
        cap(x,z,width,height)
        # Overlapping editable caps provide shoulders instead of an isolated perfect mound.
        side=angle+rng.choice((-1,1))*math.pi/2
        cap(x+math.sin(side)*width*.5,z-math.cos(side)*width*.5,width*.8,height*.45)
    valleys=recipe['valleys']
    for _ in range(valleys['count']):
        angle=rng.uniform(0,math.tau)
        distance=rng.uniform(*valleys['distance_meters'])
        cap(math.sin(angle)*distance,-math.cos(angle)*distance,rng.uniform(*valleys['radius_meters']),-rng.uniform(*valleys['depth_meters']))
    if len(d['operations'])>32:
        raise ValueError('Recipe exceeds the native planet operation budget')
    return d


def view(yaw=0):
    return dict(actor='player',yaw_degrees=yaw,pitch_degrees=0,eye_height=1.65,fov_degrees=60,mode='first_person')


def prepare(binary,seed=None):
    recipe=json.loads(RECIPE.read_text())
    seed=recipe['seed'] if seed is None else seed
    document=document_for(recipe,seed)
    assert document==document_for(recipe,seed)
    OUT.mkdir(parents=True,exist_ok=True)
    c=Client(binary,log_path=OUT/'prepare.log')
    try:
        init(c)
        initial=c.call('inspect')
        player=copy.deepcopy(next(e for e in initial['scene']['entities'] if e['id']=='player'))
        post=copy.deepcopy(next(e for e in initial['scene']['entities'] if e.get('body')=='fixed'))
        c.call('set_execution_mode',mode='continuous')
        def query(kind,doc=document,**kw):
            return c.call('query_planet',query=dict(kind=kind,document=doc,**kw))
        # Normalize only the vertical datum of the generated recipe. The diameter stays real.
        origin=query('sample',directions=[[0,1,0]])['samples'][0]['height_meters']
        document['base_height_meters']=-origin
        probe=[direction(document['diameter_meters']/2,x,z) for z in range(-4000,4001,1000) for x in range(-4000,4001,1000)]
        first=query('sample',directions=probe)['samples']
        assert query('sample',directions=probe)['samples']==first
        alternate=document_for(recipe,(seed+1)&0xffffffff)
        other=query('sample',doc=alternate,directions=probe)['samples']
        # Compare relative relief, not merely a different baseline/id.
        delta=max(abs((a['height_meters']-first[40]['height_meters'])-(b['height_meters']-other[40]['height_meters'])) for a,b in zip(first,other))
        assert delta>1.,delta
        c.call('save_planet',path='artifacts/earth-walk/earth.ignis-planet',document=document)
        assert c.call('load_planet',path='artifacts/earth-walk/earth.ignis-planet')['document']==document
        (OUT/'source.recipe.json').write_text(json.dumps({'seed':seed,'document':document},indent=2))
        c.call('load_scene',path='artifacts/earth-horizon/native-eye.scene.json')
        tiles=recipe['tiles']
        frame={'kind':'surface','pose':{'direction':[0,1,0],'forward':[0,0,-1],'altitude_meters':0}}
        material=copy.deepcopy(recipe['material'])
        material['pattern']['seed']=seed&0xffff
        reports=[]
        ids=[]
        for y in range(tiles['minimum'][1],tiles['minimum'][1]+tiles['count'][1]):
            for x in range(tiles['minimum'][0],tiles['minimum'][0]+tiles['count'][0]):
                name=f'earth_{x}_{y}'
                ids.append(name)
                result=c.call('publish_planet',publication={'document':document,
                    'area':{'kind':'tile','face':'positive_y','level':tiles['level'],'tile':[x,y],'subdivisions':tiles['subdivisions']},
                    'frame':frame,'asset':name,'instance':name,'color':recipe['color'],'material':material,'collision':False})
                reports.append(result['planet'])
        player['transform']={'position':[0,1.05,0],'size':[.6,1.8,.6]}
        player['character_controller']={'max_slope_degrees':45,'slide_slope_degrees':50,'offset':.01,'snap_distance':.2}
        player['footsteps']=None
        # Preserve the slow verification paths; apply the demo-only sprint after them.
        edits=[{'op':'set_mesh_collision','instance':i,'enabled':True} for i in ids]
        edits.append({'op':'upsert','entity':player})
        markers=[('origin_marker',-3,-8,2,[.9,.75,.15]),('north_marker',0,-30,3,[.2,.45,.9]),('east_marker',20,-15,4,[.8,.25,.2]),('west_marker',-20,-40,4,[.6,.25,.8])]
        marker_samples=query('sample',directions=[direction(document['diameter_meters']/2,x,z) for _,x,z,_,_ in markers])['samples']
        for (name,_,_,height,color),sample in zip(markers,marker_samples):
            p=copy.deepcopy(post)
            local=c.call('query_world_space',query={'kind':'project','point':sample['position'],'origin':reports[0]['global_origin'],'radius_meters':20000,'max_error_meters':.001})['local']
            p.update(id=name,name=name,color=color,transform={'position':[local[0],local[1]+height/2-.03,local[2]],'size':[.6,height,.6]})
            edits.append({'op':'upsert','entity':p})
        edits.append({'op':'set_camera','camera':{'position':[0,1.8,0],'target':[0,1.8,-100],'fov_degrees':60}})
        c.call('edit_scene',edits=edits)
        light=initial['scene']['light']
        light.update(direction=[.7,-.6,-.35],ambient=.3,atmosphere=None,shadow=None)
        c.call('set_light',light=light)
        c.call('save_scene',path=SCENE)
        # Several real KCC paths on the generated surface, returning to authored spawn each time.
        walks=[]
        for yaw in (45,135,225,315):
            c.call('reset')
            c.call('set_player_view',view=view(yaw))
            c.call('step',ticks=120)
            start=c.call('inspect_world')['positions']['player']
            c.call('set_player_view_input',input={'x':0,'z':-1})
            c.call('step',ticks=600)
            c.call('set_player_view_input',input={'x':0,'z':0})
            end=c.call('inspect_world')['positions']['player']
            state=c.call('inspect_character_controllers')['controllers']['player']
            distance=math.dist(start,end)
            assert distance>30 and state['grounded'],{'yaw':yaw,'start':start,'end':end,'state':state}
            walks.append(dict(yaw=yaw,start=start,end=end,distance_meters=distance,grounded=state['grounded']))
        long_walks=[]
        for yaw in (45,315):
            c.call('reset');c.call('set_player_view',view=view(yaw));c.call('step',ticks=120)
            start=c.call('inspect_world')['positions']['player']
            c.call('set_player_view_input',input={'x':0,'z':-1,'mode':'run'})
            c.call('step',ticks=3600)
            c.call('step',ticks=3600)
            c.call('set_player_view_input',input={'x':0,'z':0})
            end=c.call('inspect_world')['positions']['player']
            state=c.call('inspect_character_controllers')['controllers']['player']
            assert math.dist(start,end)>400 and state['grounded'],{'yaw':yaw,'start':start,'end':end,'state':state}
            long_walks.append(dict(yaw=yaw,start=start,end=end,distance_meters=math.dist(start,end),height_change_meters=end[1]-start[1],grounded=state['grounded']))
        c.call('reset');c.call('set_player_view',view=view());c.call('step',ticks=120)
        c.call('capture',path='artifacts/earth-walk/procedural-eye.png',width=1280,height=800)
        c.call('set_player_view',view=None)
        c.call('set_camera',camera={'position':[900,1600,1600],'target':[0,100,-800],'fov_degrees':65})
        c.call('capture',path='artifacts/earth-walk/procedural-aerial.png',width=1280,height=800)
        # Restore the authoring camera in the saved scene: opening starts at the walking spawn.
        c.call('set_camera',camera={'position':[0,1.8,0],'target':[0,1.8,-100],'fov_degrees':60})
        from open_earth_walk import demo_locomotion
        player['locomotion']=demo_locomotion()
        c.call('upsert',entity=player)
        c.call('save_scene',path=SCENE)
        # Direct persisted-geometry seam check, not more JSON through the control loop.
        saved=json.loads((ROOT/SCENE).read_text())
        assets={a['id']:a['geometry'] for a in saved['meshes']['assets'] if a['id'] in ids}
        n=tiles['subdivisions'];w=n+1;checks=0
        for y in range(tiles['minimum'][1],tiles['minimum'][1]+tiles['count'][1]):
            for x in range(tiles['minimum'][0],tiles['minimum'][0]+tiles['count'][0]):
                g=assets[f'earth_{x}_{y}']['positions']
                if f'earth_{x+1}_{y}' in assets:
                    neighbor=assets[f'earth_{x+1}_{y}']['positions']
                    assert all(g[j*w+n]==neighbor[j*w] for j in range(w));checks+=1
                if f'earth_{x}_{y+1}' in assets:
                    neighbor=assets[f'earth_{x}_{y+1}']['positions']
                    assert all(g[n*w+j]==neighbor[j] for j in range(w));checks+=1
        height_range=[min(p[1] for a in assets.values() for p in a['positions']),max(p[1] for a in assets.values() for p in a['positions'])]
        report=dict(seed=seed,build_id=initial['build_id'],scene=SCENE,diameter_meters=document['diameter_meters'],
                    operations=len(document['operations']),vertices=sum(len(a['positions']) for a in assets.values()),
                    triangles=sum(len(a['triangles']) for a in assets.values()),height_range_meters=height_range,
                    same_seed_samples_exact=True,other_seed_relief_delta_meters=delta,shared_edges_exact=checks,
                    source_roundtrip=True,walks=walks,long_walks=long_walks,local_walk_only=True,radial_gravity=False,native_window=False,passed=True)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2))
        print(json.dumps(report),flush=True)
        return report
    finally:
        c.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--seed',type=int)
    a=p.parse_args();prepare(ROOT/'target/release/ignis',a.seed)
