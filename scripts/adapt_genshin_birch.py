#!/usr/bin/env python3
"""Author native living versions of the three existing Birch sources and open them."""
import argparse
import json
from verify_lab import Client, ROOT
OUT=ROOT/'artifacts/genshin-birch-living'
REL='artifacts/genshin-birch-living'

def init(c):
    c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='birch-living-author',version='1')))
    c.request('notifications/initialized',notification=True)

def author():
    OUT.mkdir(parents=True,exist_ok=True)
    cfg=json.loads((ROOT/'examples/genshin-birch-living.recipe.json').read_text())
    library_path=ROOT/cfg['source_library'];library=json.loads(library_path.read_text())
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'author.log')
    try:
        init(c);records=[]
        c.call('generate',recipe=dict(width=32,depth=32,seed=220041,obstacles=0))
        scene=c.call('inspect')['scene'];scene['entities']=[];scene['botany']=[]
        scene['meshes']=dict(assets=[dict(id='birch_ground',geometry=dict(positions=[[-1000,-.025,-1000],[1000,-.025,-1000],[-1000,-.025,1000],[1000,-.025,1000]],triangles=[[0,2,1],[1,2,3]]),shading='smooth')],instances=[dict(id='birch_ground',asset='birch_ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.16,.23,.075],playback=None)])
        (OUT/'base.scene.json').write_text(json.dumps(scene))
        c.call('load_scene',path=f'{REL}/base.scene.json')
        for n,entry in enumerate(library['entries']):
            prefix=entry['id'];path=library_path.parent/entry['path']
            r=json.loads(path.read_text());r['settings']['position']=[(n-1)*6.,0,0]
            candidate=OUT/'candidate.ignis-botany';candidate.write_text(json.dumps(r,separators=(',',':')))
            c.call('botany',operation=dict(kind='load',path=f'{REL}/candidate.ignis-botany',publish=True))
            for part in ['wood','leaves']:
                c.call('material',operation=dict(kind='load',asset=prefix+'_'+part,path=str((library_path.parent/entry['materials'][part]).relative_to(ROOT))))
            c.call('botany',operation=dict(kind='adopt_living',prefix=prefix,timing=cfg['timing'],born=0))
            c.call('botany',operation=dict(kind='set_wood_details',prefix=prefix,details=dict(seed=entry['seed'],age_years=0,healing_years=20,scars=[])))
            c.call('botany',operation=dict(kind='save',prefix=prefix,path=f'{REL}/{prefix}.ignis-botany'))
            c.call('botany',operation=dict(kind='living',prefix=prefix,action=dict(kind='advance',time=20*365)))
            records.append(dict(id=prefix,source=str(path.relative_to(ROOT)),living_source=f'{REL}/{prefix}.ignis-botany',reference_years=20))
            print(f'Adapted {prefix}',flush=True)
        candidate.unlink(missing_ok=True)
        light=dict(direction=[.45,-.78,-.5],ambient=.22,direct_intensity=1.3,direct_color=[1,.94,.83],environment=dict(sky=[.42,.58,.76],horizon=[.56,.62,.61],ground=[.18,.21,.10],axis=[0,1,0],intensity=.42),atmosphere=dict(density=0,reduce_details=False,color=[.45,.6,.7],sun_color=[1,.95,.8],sun_strength=0,layers=dict(sky_color=[.32,.54,.78],mountain_color=[.32,.45,.56],detail_start=10000,detail_end=20000,influence=0)),shadow=dict(center=[0,4,0],extent=18,resolution=2048,bias=.0003,receiver_plane_bias=True))
        c.call('set_light',light=light)
        camera=dict(position=[8,6,21],target=[0,4,0],fov_degrees=43)
        c.call('set_camera',camera=camera)
        c.call('capture',path=f'{REL}/adult-gallery.png',width=1600,height=1100)
        c.call('botany',operation=dict(kind='open',open=True,prefix='birch_0'))
        c.call('save_scene',path=f'{REL}/gallery.scene.json')
        # Direct authoring previews; the saved individual remains at its adult age.
        c.call('set_camera',camera=dict(position=[-1,5,11],target=[-6,3.8,0],fov_degrees=46))
        for year in cfg['stages_years']:
            frame={0:.75,2:2.6,8:6.2,20:9.1,80:18.8}[year]
            c.call('set_camera',camera=dict(position=[-6,frame*.6,frame*1.65],target=[-6,frame*.46,0],fov_degrees=44))
            c.call('botany',operation=dict(kind='preview_living_age',prefix='birch_0',time=round(year*365)))
            c.call('capture',path=f'{REL}/age-{year}.png',width=1200,height=1100)
            print(f'Captured Birch at {year} years',flush=True)
        c.call('botany',operation=dict(kind='cancel_preview'))
        # Present all three at distinct ages in a second authored scene.
        for prefix,year in [('birch_0',2),('birch_1',8),('birch_2',20)]:
            c.call('botany',operation=dict(kind='load',path=f'{REL}/{prefix}.ignis-botany',publish=True))
            c.call('botany',operation=dict(kind='living',prefix=prefix,action=dict(kind='advance',time=year*365)))
        c.call('set_camera',camera=camera)
        c.call('capture',path=f'{REL}/ages.png',width=1600,height=1100)
        c.call('save_scene',path=f'{REL}/ages.scene.json')
        c.call('load_scene',path=f'{REL}/gallery.scene.json')
        info=dict(entries=records,build=c.call('inspect')['build_id'],scene=f'{REL}/gallery.scene.json',notes=['Native authored grammar9; original adult sources retained unchanged.','Authorial development profile; no invented apple-like Birch fruit.','Explicit age commands, not automatic world-clock ticking.'])
        (OUT/'base.scene.json').unlink(missing_ok=True)
        (OUT/'library.json').write_text(json.dumps(info,indent=2)+'\n')
        print(json.dumps(info),flush=True)
    finally:c.close()

def launch():
    c=Client(ROOT/'target/release/ignis',window=True,detached=True,log_path=OUT/'window.log',startup_args=('--editor','--scene',f'{REL}/ages.scene.json'))
    try:
        init(c);c.call('botany',operation=dict(kind='open',open=True,prefix='birch_0'))
        info=dict(pid=c.process.pid,build=c.call('inspect')['build_id'],scene=f'{REL}/ages.scene.json')
        (OUT/'window.json').write_text(json.dumps(info,indent=2)+'\n');print(json.dumps(info))
    except BaseException:c.close();raise
    else:c.process.stdin.close();c.process.stdout.close();c.log.close()

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--prepared',action='store_true');p.add_argument('--open',action='store_true');a=p.parse_args()
    if not a.prepared:author()
    if a.open:launch()
