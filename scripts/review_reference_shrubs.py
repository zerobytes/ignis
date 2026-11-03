#!/usr/bin/env python3
"""Neutral three-view structural gate for the eight reference shrub profiles."""
import argparse
import copy
import json

from demo_groundcover import fit,ground,publish_recipe,relative,setup,verify_closed
from groundcover_assets import ROOT,atomic_json
from reference_shrub_assets import FAMILIES,library
from verify_lab import Client

OUT=ROOT/'artifacts/reference-shrubs'


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',default=ROOT/'target/release/ignis');args=p.parse_args()
    OUT.mkdir(parents=True,exist_ok=True);manifest,files=library();c=Client(args.binary,log_path=OUT/'structure-runtime.log')
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='reference-shrub-structure',version='1')));c.request('notifications/initialized',notification=True)
        schema=c.request('tools/list')['result']['tools'][0]['inputSchema']
        assert all(key in json.dumps(schema) for key in ['blade_samples','bark_color','leaf_color','compare_images'])
        atomic_json(OUT/'structure-native-schema.json',schema);report=dict(build_id=setup(c),components={})
        for j,f in enumerate(FAMILIES):
            entry=next(e for e in manifest['entries'] if e['family']==f['id'])
            for part in entry['components']:
                recipe=copy.deepcopy(files[part['path']]);recipe['settings'].update(bark_color=[.32,.32,.32],leaf_color=[.32,.32,.32],position=[(j%4-1.5)*2.5,-.025,(j//4-.5)*3])
                report['components'][part['prefix']]=publish_recipe(c,recipe,OUT/'structure-publication.ignis-botany')['compiled']
        c.call('save_scene',path=relative(OUT/'structure.scene.json'));scene=json.loads((OUT/'structure.scene.json').read_text())
        assets=[a for a in scene['meshes']['assets'] if len(a['geometry']['triangles'])>1];verify_closed(assets)
        report['closed_oriented_parts']=True;report['parts']=len(assets)
        ground(scene);atomic_json(OUT/'structure.scene.json',scene);c.call('load_scene',path=relative(OUT/'structure.scene.json'))
        for name,d in [('front',(.15,.60,1)),('back',(-.25,.65,-1)),('top',(.02,1,.03))]:
            fit(c,[-4.9,-.03,-2.6],[4.9,1.4,2.6],OUT/f'structure-{name}.png',direction=d,width=1800,height=1100)
        atomic_json(OUT/'structure-report.json',report)
        print(json.dumps(dict(build=report['build_id'],parts=len(assets),closed=True)))
    finally:c.close()


if __name__=='__main__':main()
