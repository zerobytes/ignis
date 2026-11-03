#!/usr/bin/env python3
"""Compare authored stone primitives while proving the rest of the cottage stayed fixed."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path

P=Path(__file__).resolve().parents[1]
R=P/'artifacts/cottage-stone'
VIEWS={'paving':{'position':[3.4,1.8,6.4],'target':[1.45,.30,3.9],'fov_degrees':42},
       'entry-stone':{'position':[4.2,3.3,5.1],'target':[2.75,2.4,2.1],'fov_degrees':30}}


def main():
    ap=argparse.ArgumentParser();ap.add_argument('--before',type=Path,default=R/'examples/before.scene.json');ap.add_argument('--after',type=Path,default=P/'examples/elven-cottage.scene.json');args=ap.parse_args()
    for d in ['examples','artifacts']:(R/d).mkdir(parents=True,exist_ok=True)
    for label,source in [('before',args.before),('after',args.after)]:
        (R/f'examples/{label}.scene.json').write_text(json.dumps(json.loads(source.read_text()),separators=(',',':'))+'\n')
    os.environ['IGNIS_VERIFY_ROOT']=str(R)
    from verify_lab import Client
    c=Client(P/'target/release/ignis');call=c.call
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'cottage-stone-study','version':'1'}});c.request('notifications/initialized',notification=True)
        states={};counts={};build=call('inspect')['build_id']
        for label in ['before','after']:
            call('load_scene',path=f'examples/{label}.scene.json');states[label]=call('inspect')['scene'];s=states[label]
            counts[label]={'vertices':sum(len(a['geometry']['positions']) for a in s['meshes']['assets']),'triangles':sum(len(a['geometry']['triangles']) for a in s['meshes']['assets'])}
            for view,cam in VIEWS.items():
                call('set_camera',camera=cam);call('capture',path=f'artifacts/{label}-{view}.png',width=1000,height=800)
            call('set_camera',camera=s['camera']);assert call('inspect')['scene']==s
        a={v['id']:v for v in states['before']['meshes']['assets']};b={v['id']:v for v in states['after']['meshes']['assets']};assert a.keys()==b.keys()
        changed=sorted(key for key in a if a[key]!=b[key]);assert changed==['paving','stone-blocks'],changed
        for key in changed:
            for field in a[key]:
                if field!='geometry':assert a[key][field]==b[key][field]
            top=call('inspect_mesh_topology',asset=key)['topology'];assert top['boundary_edges']==top['non_manifold_edges']==0 and not top['isolated_vertices']
        before=copy.deepcopy(states['before']);after=copy.deepcopy(states['after']);before['meshes']['assets']=[];after['meshes']['assets']=[];assert before==after
        assert call('inspect')['scene']==states['after']
        for view in VIEWS:
            assert (R/f'artifacts/before-{view}.png').read_bytes()!=(R/f'artifacts/after-{view}.png').read_bytes()
        report={'passed':True,'build_id':build,'changed_assets':changed,'other_assets_scene_materials_preserved':True,'views':VIEWS,'camera_restored':True,'counts':counts,'sources':{label:{'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for label,p in [('before',args.before),('after',args.after)]},'limitations':['Only selected boxes chamfered; no arbitrary-region bevel or physical stone damage','No geometry/collision/interior integration; artistic review remains separate']}
        (R/'artifacts/stone-report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='views'}))
    finally:c.close()


if __name__=='__main__':main()
