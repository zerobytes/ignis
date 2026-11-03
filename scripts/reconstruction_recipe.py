#!/usr/bin/env python3
"""Publish declarative native-loft recipes. No subject-specific surface formulas.

Geometry comes from public preview_plant/preview_scene_edits. The client
copies indexed vertices through rigid transforms, composes by material and
optionally joins the rims of two native lofts for a hollow shell. It
retains per-part index ranges. This composition is not a native scene source.
"""
import argparse
import copy
import hashlib
import html
import json
import math
import os
from pathlib import Path
import struct
import shutil

PROJECT = Path(__file__).resolve().parents[1]
BINARY = Path('/Volumes/Zero/dev/games/.ignis-build/image-to-3d/release/ignis')


def quat(euler):
    x,y,z = [math.radians(v)/2 for v in euler]
    sx,cx,sy,cy,sz,cz = math.sin(x),math.cos(x),math.sin(y),math.cos(y),math.sin(z),math.cos(z)
    return [sx*cy*cz-cx*sy*sz,cx*sy*cz+sx*cy*sz,cx*cy*sz-sx*sy*cz,cx*cy*cz+sx*sy*sz]


def qmul(a,b):
    x,y,z,w=a; X,Y,Z,W=b
    return [w*X+x*W+y*Z-z*Y,w*Y-x*Z+y*W+z*X,w*Z+x*Y-y*X+z*W,w*W-x*X-y*Y-z*Z]


def rotate(p,q):
    x,y,z=p; a,b,c,d=q
    dot=a*x+b*y+c*z
    return [2*a*dot+(d*d-a*a-b*b-c*c)*x+2*d*(b*z-c*y),
            2*b*dot+(d*d-a*a-b*b-c*c)*y+2*d*(c*x-a*z),
            2*c*dot+(d*d-a*a-b*b-c*c)*z+2*d*(a*y-b*x)]


def compose(a,b):
    p,q,s=a; P,Q,S=b
    r=rotate([v*s for v in P],q)
    return ([p[i]+r[i] for i in range(3)], qmul(q,Q),s*S)


def transform(record):
    return record.get('position',[0,0,0]),quat(record.get('rotation',[0,0,0])),record.get('scale',1)


def f32(v):
    return struct.unpack('f',struct.pack('f',v))[0]


def endpoint_transform(a,b):
    d=[b[i]-a[i] for i in range(3)]; length=math.sqrt(sum(x*x for x in d))
    if length<0.01: raise ValueError('Native segment length must be >=0.01')
    direction=[x/length for x in d]
    # Quaternion rotating local +Y to the endpoint direction.
    q=[direction[2],0,-direction[0],1+direction[1]]
    n=math.sqrt(sum(x*x for x in q))
    q=[1,0,0,0] if n<1e-10 else [x/n for x in q]
    return [(a[i]+b[i])/2 for i in range(3)],q,length


def native_source(dimensions, stations, cross='ellipse', segments=None):
    w,h,d=dimensions
    return {'profile':{'id':'profile','width':w,'height':h},'depth':d,
            'segments':segments or [2,1,2], 'sampling_tolerance':1/1024,
            'sections':{'id':'sections','cross_section':cross,'interpolation':'pchip',
                'stations':[{'id':f's{i}','at':v[0],'width':v[1],'depth':v[2],
                             'offset':v[3:5] if len(v)>3 else [0,0]} for i,v in enumerate(stations)]}}


def expand(document):
    """Generic chain/leaf repetition; all layout and attachment choices are data."""
    sources=copy.deepcopy(document['sources']); parts=copy.deepcopy(document['parts'])
    for chain in document.get('chains',[]):
        stem=chain['radius']; points=chain['points']; name=chain['id']
        for i,(a,b) in enumerate(zip(points,points[1:])):
            p,q,length=endpoint_transform(a,b)
            key=f'{name}-stem-{i}'
            sources[key]=native_source([2*stem,length,2*stem],[[0,1,1],[1,0.7,0.7]],'ellipse',[1,1,1])
            parts.append({'id':key,'source':key,'material':chain['material'],'position':p,'quaternion':q,
                          'frame':chain.get('frame'),'project_to':chain.get('project_to'),'fit_host_band':chain.get('fit_host_band',False),'layout_centerline':chain['points'],'tasks':chain.get('tasks',[])})
        # Leaves are fully explicit transforms; no image-dependent fitting here.
        for i,leaf in enumerate(chain.get('leaves',[])):
            key=f'{name}-leaf-{i}'
            sources[key]=native_source(leaf['dimensions'],[[0,.03,.15],[.35,1,1],[.65,.7,.7],[1,.03,.1]],'ellipse',[1,1,1])
            position=leaf['position']
            if leaf.get('anchor')=='base':
                shift=rotate([0,leaf['dimensions'][1]/2,0],quat(leaf.get('rotation',[0,0,0])))
                position=[position[k]+shift[k] for k in range(3)]
            parts.append({'id':key,'source':key,'material':chain['material'],
                          'position':position,'rotation':leaf.get('rotation',[0,0,0]),
                          'frame':chain.get('frame'),'project_to':chain.get('project_to'),'fit_host_band':chain.get('fit_host_band',False),'layout_centerline':chain['points'],'tasks':chain.get('tasks',[])})
    if len(parts)>1500 or len(sources)>1500: raise ValueError('Recipe part/source budget exceeded')
    return sources,parts


def front_projector(geometry):
    """World +Z ray height, with triangle XY bounds; no camera coordinates."""
    triangles=[]
    for face in geometry['triangles']:
        a,b,c=[geometry['positions'][i] for i in face]
        ax,ay,az=a; bx,by,bz=b; cx,cy,cz=c
        det=(by-cy)*(ax-cx)+(cx-bx)*(ay-cy)
        if abs(det)<1e-12:continue
        triangles.append((min(ax,bx,cx),max(ax,bx,cx),min(ay,by,cy),max(ay,by,cy),a,b,c,det))
    def height(x,y):
        hits=[]
        for xmin,xmax,ymin,ymax,a,b,c,det in triangles:
            if not(xmin-1e-7<=x<=xmax+1e-7 and ymin-1e-7<=y<=ymax+1e-7):continue
            u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/det
            v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/det
            if u>=-1e-7 and v>=-1e-7 and u+v<=1+1e-7:
                hits.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
        return max(hits) if hits else None
    def band(y):
        xs=[]
        for _,_,ymin,ymax,a,b,c,_ in triangles:
            if not ymin-1e-7<=y<=ymax+1e-7:continue
            for u,v in [(a,b),(b,c),(c,a)]:
                if abs(u[1]-y)<1e-7:xs.append(u[0])
                if (u[1]<y<v[1]) or (v[1]<y<u[1]):
                    t=(y-u[1])/(v[1]-u[1]);xs.append(u[0]+t*(v[0]-u[0]))
        return (min(xs),max(xs)) if xs else None
    height.band=band
    return height


def shell(outer,inner,height):
    """Client-side rim joining; loft vertices/triangles originate in native queries."""
    if outer['triangles']!=inner['triangles']: raise ValueError('Shell source topology mismatch')
    n=len(outer['positions'])
    faces=[t for t in outer['triangles'] if not any(all(abs(outer['positions'][i][1]-y)<1e-6 for i in t) for y in [-height/2,height/2])]
    incidences={}
    for t in faces:
        for u,v in zip(t,t[1:]+t[:1]): incidences.setdefault(tuple(sorted([u,v])),[]).append((u,v))
    rims=[uv[0] for uv in incidences.values() if len(uv)==1]
    faces=faces+[[n+i for i in reversed(t)] for t in faces]
    for u,v in rims:faces.extend([[v,u,n+u],[v,n+u,n+v]])
    points=outer['positions']+inner['positions']
    coords=(outer.get('material_coordinates') or outer['positions'])+(inner.get('material_coordinates') or inner['positions'])
    used=sorted({i for t in faces for i in t}); ids={v:i for i,v in enumerate(used)}
    return {'positions':[points[i] for i in used],'material_coordinates':[coords[i] for i in used],
            'triangles':[[ids[i] for i in t] for t in faces]}


def build(document, root, client):
    from reconstruction_structure import inspect_surface
    sources,parts=expand(document)
    frames={}; visiting=set()
    def frame(key):
        if not key: return ([0,0,0],[0,0,0,1],1)
        if key in frames: return frames[key]
        if key in visiting: raise ValueError('Frame cycle')
        visiting.add(key); value=document['frames'][key]
        result=compose(frame(value.get('parent')),transform(value));visiting.remove(key);frames[key]=result
        return result
    limits=client.call('inspect_meshes')['limits']
    (root/'limits.json').write_text(json.dumps(limits,indent=2)+'\n')
    source_meshes={}; cache={}; audits={}
    before=client.call('inspect')
    for name,definition in sources.items():
        d=copy.deepcopy(definition);d['id']='recipe-source'
        hollow=d.pop('hollow',None)
        signature=json.dumps({'definition':d,'hollow':hollow},sort_keys=True)
        if signature not in cache:
            edit={'kind':'put','definition':d,'detail_policy':'physical','animation_translation_policy':'physical'}
            edits=[{'op':'plant','edit':edit}]
            for stroke in definition.get('strokes',[]):
                edits.append({'op':'mesh','edit':{'kind':'sculpt','asset':d['id'],'stroke':stroke}})
            d.pop('strokes',None)
            value=client.call('preview_scene_edits',edits=edits)
            asset=next(a for a in value['scene']['meshes']['assets'] if a['id']==d['id'])
            geometry=asset['geometry']
            if hollow is not None:
                inner=copy.deepcopy(d)
                inner['profile']['width']-=2*hollow
                inner['depth']-=2*hollow
                if min(inner['profile']['width'],inner['depth'])<0.01: raise ValueError('Hollow thickness collapses source')
                v=client.call('preview_plant',edit={'kind':'put','definition':inner,'detail_policy':'physical','animation_translation_policy':'physical'})
                inside=next(a for a in v['assets'] if a['id']==d['id'])['geometry']
                geometry=shell(geometry,inside,d['profile']['height'])
            if sum(len(g['positions']) for g in cache.values())+len(geometry['positions'])>2*limits['total_vertices']:
                raise ValueError('Native source preview cache budget exceeded')
            cache[signature]=geometry
        source_meshes[name]=cache[signature]
        audits[name]=inspect_surface(source_meshes[name])
        if not audits[name]['solid_preflight_passed']: raise ValueError((name,audits[name]))
    assert client.call('inspect')==before, 'Preview mutated scene'
    groups={}; ranges=[]; part_geometry={}; projectors={}; projection_reports=[]; final_audits={}
    for part in parts:
        source=source_meshes[part['source']]
        p,q,s=transform(part)
        if 'quaternion' in part: q=part['quaternion']
        p,q,s=compose(frame(part.get('frame')),(p,q,s))
        if not math.isfinite(s) or s<=0: raise ValueError('Positive finite scale required')
        group=groups.setdefault(part['material'],{'positions':[],'triangles':[],'material_coordinates':[]})
        if sum(len(g['positions']) for g in groups.values())+len(source['positions'])>limits['total_vertices']:
            raise ValueError('Part copies exceed vertex budget before publication')
        offset=len(group['positions']); first_face=len(group['triangles'])
        points=[]
        for point in source['positions']:
            point=rotate([v*s for v in point],q)
            points.append([f32(p[k]+point[k]) for k in range(3)])
        host=part.get('project_to')
        if host:
            if host not in part_geometry:raise ValueError(('Projection host must precede ornament',host))
            if host not in projectors:projectors[host]=front_projector(part_geometry[host])
            height=projectors[host]
            original=copy.deepcopy(points)
            if part.get('fit_host_band'):
                fp,fq,fs=frame(part.get('frame'))
                line=[]
                for v in part.get('layout_centerline',[]):
                    v=rotate([x*fs for x in v],fq);line.append([fp[k]+v[k] for k in range(3)])
                line.sort(key=lambda v:v[1])
                if any(abs(b[1]-a[1])<1e-8 for a,b in zip(line,line[1:])):raise ValueError('Host band layout requires monotone world Y centerline')
                def center_x(y):
                    if not line:return p[0]
                    if y<=line[0][1]:return line[0][0]
                    if y>=line[-1][1]:return line[-1][0]
                    for a,b in zip(line,line[1:]):
                        if a[1]<=y<=b[1]:return a[0]+(b[0]-a[0])*(y-a[1])/(b[1]-a[1])
                spread=max(abs(v[0]-center_x(v[1])) for v in points)
                for v in points:
                    band=height.band(v[1])
                    if band is None:continue
                    mid=(band[0]+band[1])/2;half=(band[1]-band[0])/2
                    v[0]=f32(mid+(v[0]-center_x(v[1]))*min(1,half*.72/max(spread,1e-8)))
            heights=[height(v[0],v[1]) for v in points]
            missing=sum(v is None for v in heights); distances=[]
            if not missing:
                for v,surface in zip(points,heights):
                    v[2]=f32(surface + (v[2]-p[2]) + 0.012)
                    distances.append(abs(v[2]-surface))
            else:
                # Partial vertex projection can stretch a closed decoration into spikes.
                # Keep the whole part unchanged and explicitly report the unresolved anchor.
                points=original
            projection_reports.append({'part':part['id'],'host':host,'unmapped_vertices':missing,
                                       'vertices':len(points),'max_depth_offset':max(distances,default=None)})
        geometry={'positions':points,'triangles':source['triangles']}
        final_audits[part['id']]=inspect_surface(geometry)
        if not final_audits[part['id']]['solid_preflight_passed']:raise ValueError(('Final part structure',part['id'],final_audits[part['id']]))
        part_geometry[part['id']]=geometry
        group['positions'].extend(points)
        group['triangles'].extend([[i+offset for i in t] for t in source['triangles']])
        coordinates=source.get('material_coordinates') or source['positions']
        group['material_coordinates'].extend(coordinates)
        ranges.append({'id':part['id'],'source':part['source'],'material':part['material'],
                       'vertex_range':[offset,len(group['positions'])],'face_range':[first_face,len(group['triangles'])],
                       'bounds':{'min':[min(v[k] for v in points) for k in range(3)],
                                 'max':[max(v[k] for v in points) for k in range(3)]},'tasks':part.get('tasks',[])})
    counts={'assets':len(groups),'instances':len(groups),'vertices':sum(len(v['positions']) for v in groups.values()),
            'triangles':sum(len(v['triangles']) for v in groups.values()),'parts':len(parts),'native_sources':len(sources),'unique_previews':len(cache)}
    (root/'preflight.json').write_text(json.dumps(counts,indent=2)+'\n')
    if counts['assets']>limits['assets'] or counts['instances']>limits['instances'] or counts['vertices']>limits['total_vertices'] or counts['triangles']>limits['total_triangles']:
        raise ValueError(('Global mesh budget',counts,limits))
    edits=[]
    for entity in before['scene']['entities']:
        entity=copy.deepcopy(entity);entity['visible']=False
        edits.append({'op':'upsert','entity':entity})
    for key,g in groups.items():
        if len(g['positions'])>limits['asset_vertices'] or len(g['triangles'])>limits['asset_triangles']:
            raise ValueError(('Asset budget',key,len(g['positions']),len(g['triangles'])))
        material=document['materials'][key]
        edits.append({'op':'mesh','edit':{'kind':'put_asset','asset':{'id':key,'geometry':g,
            'shading':'smooth','material':material.get('surface',{}),'rig':None,'clips':[]}}})
        edits.append({'op':'mesh','edit':{'kind':'put_instance','instance':{'id':key,'asset':key,
            'position':[0,0,0],'rotation':[0,0,0,1],'scale':1,'color':material['color'],'playback':None}}})
    client.call('preview_scene_edits',edits=edits)
    assert client.call('inspect')==before
    client.call('edit_scene',edits=edits,expected_revision=before['revision'])
    (root/'part-ranges.json').write_text(json.dumps(ranges,indent=2)+'\n')
    (root/'projection-report.json').write_text(json.dumps(projection_reports,indent=2)+'\n')
    (root/'final-part-audits.json').write_text(json.dumps(final_audits,indent=2)+'\n')
    (root/'source-audits.json').write_text(json.dumps(audits,indent=2)+'\n')
    client.call('set_light',light=document['light'])
    client.call('set_camera',camera=document['camera'])
    client.call('save_scene',path='model.scene.json')
    size=document['resolution']
    client.call('capture',path='render.png',width=size[0],height=size[1])
    scene=client.call('inspect')['scene']
    omitted=document.get('inspection_omit_materials',[])
    if omitted:
        client.call('edit_scene',edits=[{'op':'mesh','edit':{'kind':'remove_instance','id':key}} for key in omitted])
    for key,camera in document.get('views',{}).items():
        client.call('set_camera',camera=camera)
        client.call('capture',path=f'{key}.png',width=size[0],height=size[1])
    client.call('load_scene',path='model.scene.json')
    assert client.call('inspect')['scene']==scene
    client.call('capture',path='loaded.png',width=size[0],height=size[1])
    assert (root/'render.png').read_bytes()==(root/'loaded.png').read_bytes()
    client.call('set_camera',camera={**document['camera'],'fov_degrees':document['camera']['fov_degrees']+1})
    client.call('undo')
    client.call('capture',path='undo.png',width=size[0],height=size[1])
    assert (root/'render.png').read_bytes()==(root/'undo.png').read_bytes()
    client.call('save_replay',path='static.replay.json')
    replay=client.call('run_replay',path='static.replay.json');assert replay['matched']
    client.call('capture_replay',replay_path='static.replay.json',path='replay.png',width=size[0],height=size[1])
    assert (root/'render.png').read_bytes()==(root/'replay.png').read_bytes()
    comparisons=[]
    regions=document.get('regions',[])
    for start in range(0,max(1,len(regions)),16):
        comparisons.append(client.call('compare_images',reference_path='source.png',candidate_path='render.png',regions=regions[start:start+16]))
    report={'counts':counts,'checks':{'preview_pure':True,'source_surfaces_closed':True,'final_parts_closed':True,'load_exact':True,'undo_pixels_exact':True,'static_replay':True,'replay_pixels_exact':True},
            'build_id':client.call('inspect')['build_id'],'client_surface_operations':['hollow_rim_join','world_Z_host_projection','host_band_layout','rigid_transform','material_grouping'],'comparison':comparisons,
            'hashes':{p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in ['source.png','render.png','model.scene.json']}}
    (root/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
    template=(PROJECT/'scripts/viewers/scene_orbit.html').read_text()
    payload={**scene['meshes'],'instances':[i for i in scene['meshes']['instances'] if i['asset'] not in document.get('inspection_omit_materials',[])],'camera':scene['camera']}
    viewer=template.replace('__SCENE_JSON__',json.dumps(payload,separators=(',',':')).replace('<','\\u003c'))
    viewer=viewer.replace('E02',document['title']).replace('reference.png','source.png').replace('differences.html','index.html')
    viewer=viewer.replace('Fundos e fechamentos do modelo ainda estão incompletos.','Hipóteses de profundidade/costas e fidelidade em avaliação; malha estática.')
    (root/'orbit.html').write_text(viewer)
    index=root/'index.html'
    if not index.exists():
        title=html.escape(document['title'])
        page=f'<!doctype html><html lang="pt-BR"><meta charset="utf-8"><title>{title}</title><style>body{{background:#252b2d;color:#eee;font:16px system-ui;margin:30px}}img{{max-width:48%;height:auto}}a{{color:#bde2df}}</style><h1>{title}</h1><p>Ensaio de reconstrução. Integridade técnica não equivale a fidelidade visual.</p><p><a href="orbit.html">Girar modelo</a> · <a href="verification.json">Verificações</a></p><img src="source.png"><img src="render.png">'
        for name in document.get('views',{}):page+=f'<h2>{html.escape(name)}</h2><img src="{html.escape(name)}.png">'
        index.write_text(page+'</html>')
    print(json.dumps({'counts':counts,'report':str(root/'verification.json')}))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode',choices=['probe','build']);parser.add_argument('--recipe',type=Path)
    parser.add_argument('--root',required=True,type=Path);parser.add_argument('--binary',type=Path,default=BINARY)
    args=parser.parse_args();root=args.root.resolve();(root/'artifacts').mkdir(parents=True,exist_ok=True)
    os.environ['IGNIS_VERIFY_ROOT']=str(root)
    from verify_lab import Client
    client=Client(args.binary)
    try:
        client.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'reconstruction-recipe','version':'1'}})
        client.request('notifications/initialized',notification=True)
        schema=client.request('tools/list')['result']
        (root/'schema.json').write_text(json.dumps(schema,indent=2)+'\n')
        if args.mode=='probe':
            limits=client.call('inspect_meshes')['limits'];(root/'limits.json').write_text(json.dumps(limits,indent=2)+'\n');print(limits)
        else:
            if args.recipe.stat().st_size>8*1024*1024:raise ValueError('Recipe exceeds8MiB')
            document=json.loads(args.recipe.read_text())
            if not (root/'source.png').exists() and document.get('reference_input'):
                shutil.copyfile(PROJECT/document['reference_input'],root/'source.png')
            expected=document.get('reference_sha256')
            if expected and hashlib.sha256((root/'source.png').read_bytes()).hexdigest()!=expected:
                raise ValueError('Reference image hash mismatch')
            build(document,root,client)
    finally:client.close()

if __name__=='__main__':main()
