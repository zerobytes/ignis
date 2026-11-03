#!/usr/bin/env python3
"""Retessellate the existing Ignis shrub for an offline hero render.

Canonical curves and library assets are never rewritten. All temporary sources
and the current native bake live in artifacts/shrub-hero.
"""
import copy
import hashlib
import json

from demo_groundcover import fit,publish_recipe,relative,setup
from groundcover_assets import ROOT,atomic_json
from reference_shrub_assets import DEST
from verify_lab import Client

OUT=ROOT/'artifacts/shrub-hero'


def main():
    OUT.mkdir(parents=True,exist_ok=True)
    manifest=json.loads((DEST/'library.json').read_text());entry=next(e for e in manifest['entries'] if e['id']=='esfera_miuda_0')
    recipes=[json.loads((DEST/p['path']).read_text()) for p in entry['components']]
    hashes={p['path']:hashlib.sha256((DEST/p['path']).read_bytes()).hexdigest() for p in entry['components']}
    # Use the largest common leaf tessellation that fits the existing native
    #3M-vertex/6M-triangle budget per source. Counts use the strip/axis compiler.
    counts=[dict(blades=sum(o['shape']['kind']=='blade' for o in r['source']['organs']),axes=sum(o['shape']['kind']=='axis' for o in r['source']['organs'])) for r in recipes]
    samples=next(n for n in range(32,3,-1) if all(c['blades']*(10*n-6)+c['axes']*110<2_950_000 and c['blades']*(20*n-16)+c['axes']*216<5_950_000 for c in counts))
    report=dict(asset=entry['id'],source_hashes=hashes,blade_samples=samples,axis_samples=8,sides=12,components={})
    c=Client(ROOT/'target/release/ignis',log_path=OUT/'native-runtime.log')
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='shrub-hero',version='1')));c.request('notifications/initialized',notification=True)
        schema=c.request('tools/list')['result']['tools'][0]['inputSchema'];assert all(k in json.dumps(schema) for k in ['blade_samples','axis_samples','sides'])
        report['build_id']=setup(c)
        for original,part,count in zip(recipes,entry['components'],counts):
            recipe=copy.deepcopy(original);recipe['settings'].update(blade_samples=samples,axis_samples=8,sides=12)
            assert recipe['source']==original['source']
            result=publish_recipe(c,recipe,OUT/'publication.ignis-botany')['compiled']
            report['components'][part['prefix']]=dict(**result,layer=part['layer'],path=part['path'],**count)
        c.call('save_scene',path=relative(OUT/'native.scene.json'))
        fit(c,[-.63,-.03,-.63],[.63,1.14,.63],OUT/'native.png',direction=(.42,.25,1),width=1200,height=1200)
    finally:c.close()
    assert all(hashlib.sha256((DEST/p).read_bytes()).hexdigest()==h for p,h in hashes.items())
    report['canonical_sources_unchanged']=True;atomic_json(OUT/'native-report.json',report)
    print(json.dumps({k:v for k,v in report.items() if k!='source_hashes'},indent=2))


if __name__=='__main__':main()
