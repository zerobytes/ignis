#!/usr/bin/env python3
"""Controlled native before/after views of the cottage structural revision."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path

P=Path(__file__).resolve().parents[1]
R=P/'artifacts/cottage-structure'


def main():
    global R
    ap=argparse.ArgumentParser()
    ap.add_argument('--before',type=Path,default=P/'artifacts/elven-cottage-review/baseline.scene.json')
    ap.add_argument('--after',type=Path,default=P/'artifacts/elven-cottage/examples/elven-cottage.scene.json')
    ap.add_argument('--root',type=Path,default=R)
    ap.add_argument('--baseline-image',type=Path)
    args=ap.parse_args()
    R=args.root.resolve()
    original=args.baseline_image
    if original is None and args.before.resolve()==(P/'artifacts/elven-cottage-review/baseline.scene.json').resolve():
        original=P/'artifacts/elven-cottage-review/baseline.png'
    if original is not None and not original.is_file():ap.error('Baseline image must exist when requested')
    for d in ['examples','artifacts']:(R/d).mkdir(parents=True,exist_ok=True)
    for name,source in [('before',args.before),('after',args.after)]:
        (R/f'examples/{name}.scene.json').write_text(json.dumps(json.loads(source.read_text()),separators=(',',':'))+'\n')
    os.environ['IGNIS_VERIFY_ROOT']=str(R)
    from verify_lab import Client
    c=Client(P/'target/release/ignis');call=c.call
    def capture(name):
        path=f'artifacts/{name}.png';return call('capture',path=path,width=1536,height=1024)
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'cottage-structure-comparison','version':'1'}});c.request('notifications/initialized',notification=True)
        states={};counts={}
        for label in ['before','after']:
            call('load_scene',path=f'examples/{label}.scene.json');states[label]=call('inspect')['scene'];s=states[label]
            counts[label]={'assets':len(s['meshes']['assets']),'instances':len(s['meshes']['instances']),'vertices':sum(len(a['geometry']['positions']) for a in s['meshes']['assets'])}
            if label=='after':
                assert s['camera']==states['before']['camera'] and s['light']==states['before']['light']
            counts[label]['draw_triangles']=capture(label+'-hero')['mesh_triangle_count']
            if label=='before' and original is not None:assert (R/'artifacts/before-hero.png').read_bytes()==original.read_bytes()
            call('set_camera',camera={'position':[8,5,10],'target':[1.4,2.5,2.1],'fov_degrees':30});capture(label+'-entry')
            call('load_scene',path=f'examples/{label}.scene.json')
            edits=[{'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':{}}} for a in s['meshes']['assets']]
            for instance in s['meshes']['instances']:
                i=copy.deepcopy(instance);i['color']=[.65,.65,.65];edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':i}})
            call('edit_scene',edits=edits);capture(label+'-clay')
        call('load_scene',path='examples/after.scene.json');assert call('inspect')['scene']==states['after']
        report={'passed':True,'build_id':call('inspect')['build_id'],'same_camera_and_light':True,'baseline_pixels_exact':original is not None,'baseline_image':str(original) if original else None,'counts':counts,'sources':{label:{'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for label,p in [('before',args.before),('after',args.after)]},'limitations':['No reference camera fitting or geometric ground truth','Changed architecture is a hypothesis; artistic review remains separate']}
        (R/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
    finally:c.close()


if __name__=='__main__':main()
