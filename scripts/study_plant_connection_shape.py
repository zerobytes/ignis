#!/usr/bin/env python3
"""Bounded native recipe comparisons; no replacement geometry or global smoothing."""
import argparse
import copy
import json
import math
import os
from pathlib import Path
from tree_study_geometry import cross, dot, sub
P=Path(__file__).resolve().parents[1]
R=P/'artifacts/plant-connection-shape'
ID='branch-editable-1'

def geometry_metrics(g,added):
    p,t=g['positions'],g['triangles'];start=len(t)-added
    normals=[];areas=[];edges={};volume=0.
    for i,tri in enumerate(t):
        a,b,c=[p[j] for j in tri];n=cross(sub(b,a),sub(c,a));length=math.sqrt(dot(n,n));assert length>0
        normals.append([x/length for x in n]);areas.append(length/2);volume+=dot(a,cross(b,c))/6
        for u,v in zip(tri,tri[1:]+tri[:1]):edges.setdefault(tuple(sorted((u,v))),[]).append(i)
    assert all(len(f)==2 for f in edges.values())
    bends=[]
    for faces in edges.values():
        a,b=faces
        if max(a,b)<start:continue
        angle=math.degrees(math.acos(max(-1,min(1,dot(normals[a],normals[b])))))
        weight=sum(areas[f] for f in faces if f>=start)/2
        bends.append((angle,weight))
    bends.sort();total=sum(w for _,w in bends)
    def percentile(q):
        at=0
        for value,w in bends:
            at+=w
            if at>=q*total:return value
        return bends[-1][0]
    return {'signed_volume_m3':volume,'new_surface_area_m2':sum(areas[start:]),'area_weighted_bend_p50':percentile(.5),'area_weighted_bend_p90':percentile(.9),'area_weighted_bend_p99':percentile(.99),'max_bend_degrees':bends[-1][0]}

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--phase',choices=['initial','refined'],default='initial');ap.add_argument('--input',type=Path,default=P/'artifacts/plant-connections/examples/input.scene.json');a=ap.parse_args()
    for d in ['examples','artifacts']:(R/d).mkdir(parents=True,exist_ok=True)
    (R/'examples/input.scene.json').write_bytes(a.input.read_bytes())
    spec=json.loads((P/'examples/native-tree-connection-shape.input.json').read_text())
    baseline=spec['baseline']
    (R/'examples/baseline.recipe.json').write_text(json.dumps(baseline,indent=2)+'\n')
    variants=spec['phases'][a.phase]
    assert len(variants)<=12
    os.environ['IGNIS_VERIFY_ROOT']=str(R)
    from verify_lab import Client
    client=Client(P/'target/release/ignis');call=client.call;results=[]
    try:
        client.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'plant-connection-shape','version':'1'}});client.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json')
        call('set_camera',camera={'position':[2.1,3.4,3.5],'target':[.75,3.25,.36],'fov_degrees':19})
        call('set_light',light={'direction':[-.7,-.3,-1],'ambient':.18})
        edits=[{'op':'mesh','edit':{'kind':'set_material','asset':id,'material':{}}} for id in ['wood',ID,'secondary-06']]
        for i in call('inspect')['scene']['meshes']['instances']:
            if i['asset'] in ['wood',ID,'secondary-06']:
                i=copy.deepcopy(i);i['color']=[.65,.65,.65];edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':i}})
        call('edit_scene',edits=edits);sources=call('inspect')['scene']['plants']['entries']
        for name,options in variants:
            recipe=copy.deepcopy(baseline);recipe['shape'].update(options);edit={'kind':'put_connection','connection':recipe};state=call('inspect')
            response=client.request('tools/call',{'name':'engine_command','arguments':{'command':{'op':'preview_plant','edit':edit}}})['result']
            assert call('inspect')==state
            if response['isError']:
                results.append({'name':name,'accepted':False,'error':response.get('structuredContent',response['content']),'recipe':recipe});print(name,'rejected',flush=True);continue
            predicted=response['structuredContent'];call('edit_plant',edit=edit)
            scene=call('inspect')['scene'];assert scene['plants']['entries']==sources and scene['meshes']['assets']==predicted['assets']
            asset=call('inspect_mesh',asset=ID)['asset'];report=call('inspect_plant_connections')['reports'][0]
            metrics=geometry_metrics(asset['geometry'],report['junction']['added_triangles'])
            path=f'artifacts/{name}.png';call('capture',path=path,width=1100,height=1000)
            result={'name':name,'accepted':True,'recipe':recipe,'report':report,'metrics':metrics,'vertices':len(asset['geometry']['positions']),'triangles':len(asset['geometry']['triangles']),'image':str(R/path)};results.append(result)
            print(name,json.dumps(metrics),flush=True)
        output={'build_id':call('inspect')['build_id'],'results':results,'bounded_candidates':len(variants),'phase':a.phase,'native_window':False}
        (R/f'artifacts/{a.phase}-report.json').write_text(json.dumps(output,indent=2)+'\n')
    finally:client.close()
if __name__=='__main__':main()
