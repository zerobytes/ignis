#!/usr/bin/env python3
"""Author a reusable native sequoia grove; optional interactive editor launch."""
import argparse
import copy
import json
import math
import random
from pathlib import Path
from verify_lab import Client, ROOT

OUT = ROOT / 'artifacts/sequoia-forest'
REL = 'artifacts/sequoia-forest'

def position(x, z):
    cx, cz = math.floor(x / 1024), math.floor(z / 1024)
    return dict(cell=[str(cx), '0', str(cz)], offset=[x-cx*1024, 0, z-cz*1024])

def initialize(c):
    c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='sequoia-forest-author', version='1')))
    c.request('notifications/initialized', notification=True)

def author():
    OUT.mkdir(parents=True, exist_ok=True)
    config = json.loads((ROOT/'examples/sequoia-forest.recipe.json').read_text())
    c = Client(ROOT/'target/release/ignis', log_path=OUT/'author.log')
    try:
        initialize(c)
        c.call('generate', recipe=dict(width=32, depth=32, seed=config['seed'], obstacles=0))
        initial = c.call('inspect')['scene']
        player = copy.deepcopy(next(e for e in initial['entities'] if e['id']=='player'))
        c.call('edit_scene', edits=[dict(op='remove', id=e['id']) for e in initial['entities']])
        c.call('save_scene', path=f'{REL}/base.scene.json')
        base = json.loads((OUT/'base.scene.json').read_text())
        base['meshes'] = dict(assets=[], instances=[], objects=[])
        base['entities'] = []
        base['botany'] = []
        (OUT/'base.scene.json').write_text(json.dumps(base))
        c.call('load_scene', path=f'{REL}/base.scene.json')
        terrain = config['terrain']
        c.call('save_terrain', path=f'{REL}/ground.ignis-terrain', document=terrain)
        ground = dict(seed=9281, unique_instances=False, plane='xz', tile_size=[6,6], height_m=.025, relief='bump', view_steps=8, shadow_steps=0,
            base=dict(color_roughness=[.055,.037,.019,.98], response=[0,.4,1,1], emission=[0,0,0,0]),
            layers=[dict(field='noise',frequency=[5,5],seed=13,octaves=3,warp=.4,threshold=[.18,.82],strength=1,invert=False,
                target=dict(color_roughness=[.16,.115,.055,.96], response=[0,.6,1,1], emission=[0,0,0,0])),
                dict(field='noise',frequency=[2,2],seed=93,octaves=2,warp=.25,threshold=[.4,.72],strength=.65,invert=False,
                target=dict(color_roughness=[.075,.12,.035,.94], response=[0,.6,1,1], emission=[0,0,0,0]))])
        c.call('publish_terrain', publication=dict(document=terrain,patch=dict(origin=position(-256,-256),cells=[64,64],spacing_meters=8),stride=1,asset='grove_ground',instance='grove_ground',frame_origin=position(0,0),color=[1,1,1],collision=True))
        c.call('material',operation=dict(kind='generate',asset='grove_ground',recipe=ground,execution='runtime'))
        for v in config['variants']:
            d=json.loads((ROOT/'examples/sequoia-volume.recipe.json').read_text())
            d.update(prefix=v['id'],seed=v['seed'],maximum_nodes=v['nodes'])
            d['foliage'].update(length_m=v['spray_length'],width_m=v['spray_width'])
            c.call('botany',operation=dict(kind='create_living',definition=d,born=0))
            r=next(r for r in c.call('botany',operation=dict(kind='inspect'))['recipes'] if r['prefix']==v['id'])
            r['settings'].update(leaf_density=v['density'],axis_samples=6,sides=6,blade_samples=4)
            candidate=OUT/'candidate.ignis-botany';candidate.write_text(json.dumps(r,separators=(',',':')))
            c.call('botany',operation=dict(kind='load',path=f'{REL}/candidate.ignis-botany',publish=True))
            c.call('botany',operation=dict(kind='living',prefix=v['id'],action=dict(kind='advance',time=v['years']*365)))
            for part,name in [('wood','bark'),('leaves','foliage')]:
                c.call('material',operation=dict(kind='load',asset=v['id']+'_'+part,path=f'assets/materials/sequoia/{name}.ignis-material'))
            c.call('botany',operation=dict(kind='save',prefix=v['id'],path=f'{REL}/{v["id"]}.ignis-botany'))
            print(f'Authored {v["id"]}: {v["years"]} years',flush=True)
        candidate.unlink(missing_ok=True)
        c.call('botany',operation=dict(kind='load',path='assets/botany/groundcover/capim_arqueado_0-green.ignis-botany',publish=True))
        c.call('save_scene',path=f'{REL}/base.scene.json')
        scene=json.loads((OUT/'base.scene.json').read_text())
        assets={a['id']:a for a in scene['meshes']['assets']}
        originals={v['id']:[i for i in scene['meshes']['instances'] if i['asset'].startswith(v['id']+'_')] for v in config['variants']}
        scene['meshes']['instances']=[i for i in scene['meshes']['instances'] if not any(i['asset'].startswith(v['id']+'_') for v in config['variants'])]
        grass=[i for i in scene['meshes']['instances'] if i['asset'].startswith('gc_capim_arqueado_0_green_')]
        scene['meshes']['instances']=[i for i in scene['meshes']['instances'] if i not in grass]
        rng=random.Random(config['seed']); sites=[(-17,26,2),(18,5,1),(-27,-27,2),(32,-45,1),(-56,4,0)]
        tries=0
        while len(sites)<config['trees'] and tries<20000:
            tries+=1;x=rng.uniform(-155,155);z=rng.uniform(-210,110)
            path_x=12*math.sin(z/47)
            if abs(x-path_x)<9 or (x*x+(z-55)**2<25**2):continue
            if any((x-a)**2+(z-b)**2<17**2 for a,b,_ in sites):continue
            sites.append((x,z,rng.choices([0,1,2],[.3,.45,.25])[0]))
        heights=c.call('query_terrain',query=dict(kind='sample',document=terrain,points=[position(x,z) for x,z,_ in sites]+[position(0,78)]))['samples']
        records=[]
        for n,((x,z,k),sample) in enumerate(zip(sites,heights)):
            v=config['variants'][k];yaw=rng.uniform(-math.pi,math.pi);scale=rng.uniform(.86,1.14);y=sample['height_meters']-.15
            members=[]
            for original in originals[v['id']]:
                if not assets[original['asset']]['geometry']['triangles']:continue
                i=copy.deepcopy(original);i.update(id=f'grove_{n}_{original["asset"].removeprefix(v["id"]+"_")}',position=[x,y,z],rotation=[0,math.sin(yaw/2),0,math.cos(yaw/2)],scale=scale)
                if i['asset'].endswith('_leaves'):i['color']=[rng.uniform(.82,1.0),rng.uniform(.9,1.0),rng.uniform(.78,1.0)]
                scene['meshes']['instances'].append(i);members.append(i['id'])
            scene['meshes'].setdefault('objects',[]).append(dict(id=f'grove_{n}',members=members))
            records.append(dict(variant=v['id'],years=v['years'],position=[x,y,z],scale=scale))
        grass_sites=[]
        while len(grass_sites)<600:
            x=rng.uniform(-100,100);z=rng.uniform(-100,100)
            if abs(x-12*math.sin(z/47))>8:grass_sites.append((x,z))
        ground_samples=c.call('query_terrain',query=dict(kind='sample',document=terrain,points=[position(x,z) for x,z in grass_sites]))['samples']
        for n,((x,z),h) in enumerate(zip(grass_sites,ground_samples)):
            yaw=rng.uniform(-math.pi,math.pi);scale=rng.uniform(.7,1.7)
            for original in grass:
                if len(assets[original['asset']]['geometry']['triangles'])<2:continue
                i=copy.deepcopy(original);i.update(id=f'ground_tuft_{n}_{original["asset"]}',position=[x,h['height_meters']-.06,z],rotation=[0,math.sin(yaw/2),0,math.cos(yaw/2)],scale=scale)
                scene['meshes']['instances'].append(i)
        # The forest shares rendered snapshots; editable living sources are saved beside it.
        scene['botany']=[]
        py=heights[-1]['height_meters']
        player['transform']=dict(position=[0,py+.95,78],size=[.7,1.6,.7])
        player['character_controller']=dict(max_slope_degrees=45,slide_slope_degrees=50,offset=.01,snap_distance=.2)
        scene['entities']=[player];scene['name']='Bosque de sequoias · luz da manhã'
        scene['camera']=dict(position=[0,py+3,78],target=[-6,py+25,-48],fov_degrees=62)
        scene['light']=config['light']
        path=OUT/'forest.scene.json';path.write_text(json.dumps(scene,separators=(',',':')))
        c.call('load_scene',path=f'{REL}/forest.scene.json')
        c.call('save_scene',path=f'{REL}/forest.scene.json')
        c.call('capture',path=f'{REL}/forest.png',width=1600,height=1100)
        c.call('set_camera',camera=dict(position=[180,110,215],target=[0,34,-40],fov_degrees=54))
        c.call('capture',path=f'{REL}/overview.png',width=1600,height=1100)
        counts={a['id']:len(a['geometry']['triangles']) for a in scene['meshes']['assets']}
        info=dict(trees=len(records),area_m=[512,512],variants=config['variants'],shared_assets=len(assets),source_triangles=sum(counts.values()),instanced_triangles=sum(counts[i['asset']] for i in scene['meshes']['instances']),population=records,build=c.call('inspect')['build_id'],scene=f'{REL}/forest.scene.json',notes=['Shared static forest instances; three editable living source files saved beside the scene.','No per-tree collision or simultaneous growth/wind simulation in this scene.','No FPS claim; native captures inspected separately.'])
        (OUT/'population.json').write_text(json.dumps(info,indent=2)+'\n')
        (OUT/'base.scene.json').unlink(missing_ok=True)
        print(json.dumps({k:v for k,v in info.items() if k!='population'}),flush=True)
    finally:c.close()

def launch():
    c=Client(ROOT/'target/release/ignis',window=True,detached=True,log_path=OUT/'window.log',startup_args=('--editor','--scene',f'{REL}/forest.scene.json','--continuous'))
    try:
        initialize(c);info=dict(pid=c.process.pid,scene=f'{REL}/forest.scene.json',build=c.call('inspect')['build_id'])
        (OUT/'window.json').write_text(json.dumps(info,indent=2)+'\n');print(json.dumps(info))
    except BaseException:c.close();raise
    else:c.process.stdin.close();c.process.stdout.close();c.log.close()

if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--open',action='store_true');ap.add_argument('--prepared',action='store_true');a=ap.parse_args()
    if not a.prepared:author()
    if a.open:launch()
