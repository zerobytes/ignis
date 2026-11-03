#!/usr/bin/env python3
"""Add a flat groundcover patch to a COPY of an authored native scene.

Run demo_groundcover.py once to compile library.scene.json and templates.json.
This command never overwrites its input and validates the result in native Ignis
before publishing the destination. Existing camera, terrain and physics persist.
"""
import argparse
import copy
import hashlib
import json
import math
import re
from pathlib import Path

from demo_groundcover import field_scene, relative, scene_stats, validate_recipe
from groundcover_assets import atomic_json, source_fingerprint
from botany_collections import COLLECTIONS, collection
from verify_lab import Client, ROOT


def compose(original,patch,prefix,x,z,ground_y):
    scene=copy.deepcopy(original)
    meshes=scene.setdefault('meshes',dict(assets=[],instances=[]))
    assets={a['id']:a for a in meshes['assets']}
    instance_ids={i['id'] for i in meshes['instances']}
    object_ids={o['id'] for o in meshes.get('objects',[])}
    instances=[copy.deepcopy(i) for i in patch['meshes']['instances'] if i['id']!='gc_ground']
    needed={i['asset'] for i in instances}
    for asset in patch['meshes']['assets']:
        if asset['id'] not in needed:continue
        if asset['id'] in assets and asset!=assets[asset['id']]:
            raise ValueError(f"Asset conflict: {asset['id']}; do not overwrite existing geometry")
        if asset['id'] not in assets:meshes['assets'].append(asset)
    for inst in instances:
        inst['id']=prefix+'_'+inst['id']
        if inst['id'] in instance_ids:raise ValueError(f"Instance conflict: {inst['id']}; choose another prefix")
        inst['position']=[inst['position'][0]+x,inst['position'][1]+ground_y+.03,inst['position'][2]+z]
    meshes['instances'].extend(instances)
    objects=meshes.setdefault('objects',[])
    for obj in patch['meshes']['objects']:
        name=prefix+'_'+obj['id']
        if name in object_ids:raise ValueError(f'Object conflict: {name}')
        objects.append(dict(id=name,members=[prefix+'_'+m for m in obj['members']]))
    return scene


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--collection',choices=COLLECTIONS,default='groundcover')
    p.add_argument('--scene',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--recipe',type=Path)
    p.add_argument('--prefix',default='meadow');p.add_argument('--x',type=float,default=0.)
    p.add_argument('--z',type=float,default=0.);p.add_argument('--ground-y',type=float,default=0.)
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    args=p.parse_args();source=args.scene.resolve();destination=args.output.resolve()
    if source==destination:p.error('output must differ from the input scene')
    if not destination.is_relative_to(ROOT):p.error('output must be inside the Ignis project')
    if not re.fullmatch(r'[a-zA-Z0-9_-]{1,16}',args.prefix):p.error('prefix requires1..16 ASCII letters/digits/_/-')
    if not all(math.isfinite(v) for v in (args.x,args.z,args.ground_y)):p.error('coordinates must be finite')
    config=collection(args.collection);out=config['output'];destination_library=config['destination']
    recipe=json.loads((args.recipe or config['recipe']).read_text());validate_recipe(recipe,config['families'])
    manifest=json.loads((destination_library/'library.json').read_text())
    built=json.loads((out/'library-build.json').read_text())
    if built.get('version')!=1 or source_fingerprint(destination_library,manifest)!=built['source_sha256'] or any(hashlib.sha256((out/file).read_bytes()).hexdigest()!=built[key] for file,key in [('library.scene.json','scene_sha256'),('templates.json','templates_sha256')]):
        p.error(f'Compiled library is stale or changed; run demo_groundcover.py --collection {args.collection} again')
    compiled=json.loads((out/'library.scene.json').read_text());templates=json.loads((out/'templates.json').read_text())
    patch,counts,_=field_scene(compiled,templates,manifest,recipe)
    original=json.loads(source.read_text());result=compose(original,patch,args.prefix,args.x,args.z,args.ground_y)
    destination.parent.mkdir(parents=True,exist_ok=True)
    candidate=destination.with_suffix('.candidate.scene.json')
    if candidate.resolve()==source:p.error('input is reserved as the validation candidate; choose another output')
    atomic_json(candidate,result)
    c=None
    try:
        c=Client(args.binary,log_path=out/'plant-runtime.log')
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='plant-groundcover',version='1')))
        c.request('notifications/initialized',notification=True)
        c.call('load_scene',path=relative(candidate))
        c.call('save_scene',path=relative(destination))
    finally:
        try:
            if c is not None:c.close()
        finally:candidate.unlink(missing_ok=True)
    print(json.dumps(dict(output=str(destination),counts=counts,stats=scene_stats(result)),ensure_ascii=False))


if __name__=='__main__':main()
