#!/usr/bin/env python3
"""Native three-view structural gate before the colour-gallery review."""
import argparse
import copy
import json

from colored_shrub_assets import FAMILIES,library
from demo_groundcover import fit,relative,setup,verify_closed
from groundcover_assets import ROOT,atomic_json
from shrub_assets import unit
from verify_lab import Client

OUT=ROOT/'artifacts/colored-shrubs'


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',default=ROOT/'target/release/ignis')
    p.add_argument('--leaf-details',action='store_true');args=p.parse_args()
    OUT.mkdir(parents=True,exist_ok=True);m,files=library();c=Client(args.binary,log_path=OUT/'structure-runtime.log');report={}
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='colored-shrub-structure',version='1')));c.request('notifications/initialized',notification=True)
        schema=c.request('tools/list')['result']['tools'][0]['inputSchema']
        assert all(key in json.dumps(schema) for key in ['blade_samples','bark_color','leaf_color','compare_images'])
        atomic_json(OUT/'structure-native-schema.json',schema)
        report['build_id']=setup(c);report['components']={}
        for j,f in enumerate(FAMILIES):
            e=next(e for e in m['entries'] if e['family']==f['id'])
            for part in e['components']:
                r=copy.deepcopy(files[part['path']]);r['settings'].update(bark_color=[.32,.32,.32],leaf_color=[.32,.32,.32],position=[(j-1.5)*2.6,-.025,0])
                report['components'][part['prefix']]=c.call('botany',operation=dict(kind='publish',recipe=r))['compiled']
        c.call('save_scene',path=relative(OUT/'structure.scene.json'))
        scene=json.loads((OUT/'structure.scene.json').read_text())
        assets=[a for a in scene['meshes']['assets'] if len(a['geometry']['triangles'])>1]
        verify_closed(assets);report['closed_oriented_parts']=True
        from demo_groundcover import ground
        ground(scene);atomic_json(OUT/'structure.scene.json',scene);c.call('load_scene',path=relative(OUT/'structure.scene.json'))
        for name,d in [('front',(.15,.38,1)),('back',(-.30,.42,-1)),('top',(.02,1,.03))]:
            fit(c,[-5.0,-.03,-1.25],[5.0,2.15,1.25],OUT/f'structure-{name}.png',direction=d,width=1600,height=900)
        atomic_json(OUT/'structure-report.json',report)
        if args.leaf_details:
            details={}
            for f in FAMILIES:
                setup(c);entry=next(e for e in m['entries'] if e['family']==f['id']);extent=f['length']*1.35
                for index,layer in enumerate(['green','flush']):
                    part=next(p for p in entry['components'] if p['layer']==layer)
                    recipe=copy.deepcopy(files[part['path']]);organ=copy.deepcopy(next(o for o in recipe['source']['organs'] if o['shape']['kind']=='blade'))
                    origin=organ['path'][0];t=unit([b-a for a,b in zip(organ['path'][0],organ['path'][1])]);n=unit(organ['normal'])
                    projection=sum(a*b for a,b in zip(t,n));n=unit([a-projection*b for a,b in zip(n,t)])
                    b=[t[1]*n[2]-t[2]*n[1],t[2]*n[0]-t[0]*n[2],t[0]*n[1]-t[1]*n[0]]
                    organ['path']=[[sum((x-y)*d for x,y,d in zip(point,origin,axis)) for axis in (n,b,t)] for point in organ['path']]
                    organ['normal']=[1,0,0];recipe['prefix']=f'macro_{layer}';recipe['source']['organs']=[organ]
                    recipe['settings']['position']=[(index-.5)*extent,.08,0]
                    c.call('botany',operation=dict(kind='publish',recipe=recipe))
                fit(c,[-extent,.07,0],[extent,.12,extent],OUT/f"{f['id']}-leaf.png",direction=(.05,1,.12),width=1100,height=750)
                details[f['id']]=dict(transform='rigid canonical leaf frame; no scaling',left='mature',right='flush',bounds=[[-extent,.07,0],[extent,.12,extent]])
            atomic_json(OUT/'leaf-review.json',details)
        print(json.dumps(dict(build=report['build_id'],parts=len(assets),closed=True),indent=2))
    finally:c.close()


if __name__=='__main__':main()
