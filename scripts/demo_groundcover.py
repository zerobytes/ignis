#!/usr/bin/env python3
"""Compile the editable groundcover library and plant shared native instances.

Own isolated MCP process only. Never modifies an existing human scene/window.
Outputs replace the current gallery/field at stable paths in artifacts/groundcover.
"""
import argparse
import copy
import hashlib
import html
import itertools
import json
import math
import random
import time
from collections import Counter
from pathlib import Path

from groundcover_assets import FAMILIES, atomic_json, source_fingerprint
from botany_collections import collection
from verify_lab import Client, ROOT

OUT=ROOT/'artifacts/groundcover'


def relative(path):
    return str(path.relative_to(ROOT))


def fit(c,lo,hi,path,direction=(.6,.45,1),width=850,height=750):
    import botany_math as b
    target=[(a+d)/2 for a,d in zip(lo,hi)];direction=b.unit(direction)
    right=b.unit(b.cross(b.mul(direction,-1),b.UP));up=b.cross(right,b.mul(direction,-1))
    tan=math.tan(math.radians(36/2))
    distance=max(b.dot(b.sub(p,target),direction)+1.15*max(abs(b.dot(b.sub(p,target),right))/(tan*width/height),abs(b.dot(b.sub(p,target),up))/tan) for p in itertools.product(*zip(lo,hi)))
    c.call('set_camera',camera=dict(position=b.add(target,b.mul(direction,distance)),target=target,fov_degrees=36))
    c.call('capture',path=relative(path),width=width,height=height)


def setup(c):
    c.call('generate',recipe=dict(width=32,depth=32,seed=200,obstacles=0))
    state=c.call('inspect')
    c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in state['scene']['entities']])
    c.call('set_light',light=dict(direction=[.45,-.82,-.35],ambient=.40,direct_intensity=1.1,
        environment=dict(sky=[.55,.65,.78],horizon=[.58,.62,.57],ground=[.13,.16,.075],axis=[0,1,0],intensity=.5),
        shadow=dict(center=[0,0,0],extent=18,resolution=2048,bias=.001)))
    return state['build_id']


def ground(scene):
    # Below the renderer's ±2cm diagnostic-floor grid band; roots remain embedded.
    if not any(a['id']=='gc_ground' for a in scene['meshes']['assets']):
        scene['meshes']['assets'].append(dict(id='gc_ground',geometry=dict(positions=[[-40,-.03,-40],[40,-.03,-40],[-40,-.03,40],[40,-.03,40]],triangles=[[0,2,1],[1,2,3]])))
    scene['meshes']['instances'].append(dict(id='gc_ground',asset='gc_ground',position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.18,.155,.09]))


def base_scene(compiled):
    scene=copy.deepcopy(compiled)
    # Static library copies are mesh instances; authoring curves live separately.
    scene['botany']=[]
    scene['meshes']['instances']=[]
    scene['meshes']['objects']=[]
    ground(scene)
    return scene


def place(scene,templates,entry,id,position,yaw=0,scale=1,tint=1):
    members=[]
    for k,template in enumerate(templates[entry['id']]):
        inst=copy.deepcopy(template);inst.update(id=f'{id}_{k}',position=[position[0],position[1]-.025,position[2]],rotation=[0,math.sin(yaw/2),0,math.cos(yaw/2)],scale=scale)
        # One correlated tint for the entire tuft; preserve relative accent hues.
        inst['color']=[min(1.,v*tint) for v in inst['color']]
        scene['meshes']['instances'].append(inst);members.append(inst['id'])
    scene['meshes']['objects'].append(dict(id=id,members=members))


def noise(seed,family,x,z):
    x/=2.8;z/=2.8;ix,iz=math.floor(x),math.floor(z);u,v=x-ix,z-iz
    u=u*u*(3-2*u);v=v*v*(3-2*v)
    def value(a,b):
        return int.from_bytes(hashlib.sha256(f'{seed}:{family}:{a}:{b}'.encode()).digest()[:4],'little')/2**32
    return (1-v)*((1-u)*value(ix,iz)+u*value(ix+1,iz))+v*((1-u)*value(ix,iz+1)+u*value(ix+1,iz+1))


def field_scene(compiled,templates,manifest,recipe):
    scene=base_scene(compiled);r=random.Random(recipe['seed']);counts=Counter();placements=[]
    families=[f['id'] for f in manifest['families']];by_family={f:[e for e in manifest['entries'] if e['family']==f] for f in families}
    weight=recipe['weights'];count=recipe['count'];size=recipe['size'];cells=math.ceil(math.sqrt(count*1.22))
    candidates=list(itertools.product(range(cells),repeat=2));r.shuffle(candidates)
    for ix,iz in candidates:
        x=((ix+r.random())/cells-.5)*size;z=((iz+r.random())/cells-.5)*size
        if recipe['path'] and abs(x-.65*math.sin(z*.55)) < .40:
            continue
        weights=[weight[f]*(.12+5*noise(recipe['seed'],f,x,z)**3) for f in families]
        family=r.choices(families,weights=weights,k=1)[0];entry=r.choice(by_family[family])
        yaw=r.uniform(0,math.tau);s=r.uniform(.78,1.20);t=r.uniform(.88,1.12);id=f'field_{len(placements):05}'
        place(scene,templates,entry,id,[x,0,z],yaw,s,t);counts[family]+=1
        placements.append(dict(id=id,variant=entry['id'],position=[x,0,z],yaw=yaw,scale=s,tint=t))
        if len(placements)==count:break
    if len(placements)!=count:raise ValueError('Not enough planting cells; reduce count or path width')
    return scene,dict(counts),placements


def scene_stats(scene):
    assets={a['id']:a for a in scene['meshes']['assets']}
    return dict(unique_assets=len(assets),instances=len(scene['meshes']['instances']),tufts=len(scene['meshes'].get('objects',[])),
                unique_vertices=sum(len(a['geometry']['positions']) for a in assets.values()),
                unique_triangles=sum(len(a['geometry']['triangles']) for a in assets.values()),
                instanced_triangles=sum(len(assets[i['asset']]['geometry']['triangles']) for i in scene['meshes']['instances']))


def validate_recipe(recipe,families=FAMILIES):
    families={f['id'] for f in families}
    if set(recipe)!={'version','seed','size','count','path','weights'} or recipe['version']!=1:
        raise ValueError('Unknown field recipe version or fields')
    if type(recipe['count']) is not int or not 1<=recipe['count']<=20000:
        raise ValueError('count requires1..20000')
    if type(recipe['seed']) is not int or not 0<=recipe['seed']<2**32 or type(recipe['path']) is not bool:
        raise ValueError('seed requiresu32 and path requiresboolean')
    if type(recipe['size']) not in (int,float) or not math.isfinite(recipe['size']) or not 2<=recipe['size']<=100:
        raise ValueError('size requires2..100 metres')
    weights=recipe['weights']
    if not isinstance(weights,dict) or set(weights)!=families or any(type(w) not in (float,int) or not math.isfinite(w) or not 0<=w<=1 for w in weights.values()) or sum(weights.values())<=0:
        raise ValueError('weights require each family, finite nonnegative values<=1 and a positive sum')


def verify_closed(assets):
    # Integrity of the actual native output, not a mirror of the generator.
    for asset in assets:
        edges=Counter()
        for tri in asset['geometry']['triangles']:
            assert len(set(tri))==3,asset['id']
            for a,b in zip(tri,tri[1:]+tri[:1]):edges[(a,b)]+=1
        assert all(n==1 and edges[(b,a)]==1 for (a,b),n in edges.items()),asset['id']


def main():
    global OUT
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--collection',choices=['groundcover','tallgrass'],default='groundcover')
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    p.add_argument('--recipe',type=Path)
    p.add_argument('--count',type=int);p.add_argument('--size',type=float)
    p.add_argument('--seed',type=int);p.add_argument('--no-path',action='store_true')
    args=p.parse_args()
    config=collection(args.collection);OUT=config['output'];destination=config['destination'];families=config['families']
    columns=config['columns'];rows=math.ceil(len(families)/columns);spacing_x,spacing_z=config['spacing']
    def gallery_position(row,variant,variants):
        return [(row%columns-(columns-1)/2)*spacing_x+(variant-(variants-1)/2)*config['variant_gap'],0,(row//columns-(rows-1)/2)*spacing_z]
    recipe=json.loads((args.recipe or config['recipe']).read_text())
    for key in ('count','size','seed'):
        if getattr(args,key) is not None:recipe[key]=getattr(args,key)
    if args.no_path:recipe['path']=False
    try:validate_recipe(recipe,families)
    except ValueError as error:p.error(str(error))
    OUT.mkdir(parents=True,exist_ok=True)
    manifest=json.loads((destination/'library.json').read_text())
    expected,files=config['generate'](manifest['seed'],manifest['variants'])
    assert manifest==expected,'Library manifest must match the authored recipe'
    assert all(json.loads((destination/path).read_text())==value for path,value in files.items()),'Generated curves differ; regenerate explicitly'
    atomic_json(OUT/'field.recipe.json',recipe)
    c=Client(args.binary,log_path=OUT/'runtime.log');report={}
    try:
        c.request('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='groundcover-assets',version='1')))
        c.request('notifications/initialized',notification=True)
        (OUT/'command-schema.json').write_text(json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'],indent=2))
        report['build_id']=setup(c);report['components']={};started=time.monotonic()
        for e in manifest['entries']:
            for part in e['components']:
                result=c.call('botany',operation=dict(kind='load',path=relative(destination/part['path']),publish=True))
                report['components'][part['prefix']]=result['compiled']
                # Keep the editable authoring scene useful: all colour layers of
                # each variant share one gallery location, rather than overlap.
                row=next(i for i,f in enumerate(families) if f['id']==e['family'])
                authored=copy.deepcopy(files[part['path']])
                authored['settings']['position']=gallery_position(row,e['variant'],manifest['variants'])
                c.call('botany',operation=dict(kind='publish',recipe=authored))
        report['compile_library_seconds']=time.monotonic()-started
        c.call('set_camera',camera=dict(position=[9,10,14],target=[0,.5,0],fov_degrees=36))
        c.call('save_scene',path=relative(OUT/'authoring.scene.json'))
        compiled=json.loads((OUT/'authoring.scene.json').read_text())
        # Native compiler emits a hidden triangle when a colour category is empty.
        assets=[a for a in compiled['meshes']['assets'] if len(a['geometry']['triangles'])>1]
        ids={a['id'] for a in assets};compiled['meshes']['assets']=assets
        source_instances=[i for i in compiled['meshes']['instances'] if i['asset'] in ids]
        for i in source_instances:i['position']=[0,0,0]
        templates={e['id']:[i for i in source_instances if any(i['asset'] in (p['prefix']+'_wood',p['prefix']+'_leaves') for p in e['components'])] for e in manifest['entries']}
        assert all(templates.values()),'Missing native component templates'
        verify_closed(assets);report['all_native_parts_closed_oriented']=True
        atomic_json(OUT/'library.scene.json',base_scene(compiled))
        atomic_json(OUT/'templates.json',templates)
        atomic_json(OUT/'library-build.json',dict(version=1,source_sha256=source_fingerprint(destination,manifest),
            scene_sha256=hashlib.sha256((OUT/'library.scene.json').read_bytes()).hexdigest(),
            templates_sha256=hashlib.sha256((OUT/'templates.json').read_bytes()).hexdigest()))
        gallery=base_scene(compiled)
        for row,f in enumerate(families):
            for j,e in enumerate(e for e in manifest['entries'] if e['family']==f['id']):
                place(gallery,templates,e,f'gallery_{row}_{j}',gallery_position(row,j,manifest['variants']),j*1.7)
        atomic_json(OUT/'gallery.scene.json',gallery)
        c.call('load_scene',path=relative(OUT/'gallery.scene.json'))
        fit(c,*config['gallery_bounds'],OUT/'gallery.png',direction=config['gallery_direction'],width=1800,height=1250)
        c.call('save_scene',path=relative(OUT/'gallery.scene.json'))
        # Each family's three real 3D variants, same scale within each frame.
        for f in families:
            sample=base_scene(compiled);entries=[e for e in manifest['entries'] if e['family']==f['id']]
            gap=max(.5,f['reach']*2.2)
            for j,e in enumerate(entries):place(sample,templates,e,f'sample_{j}',[(j-(len(entries)-1)/2)*gap,0,0],j*.8)
            atomic_json(OUT/'sample.scene.json',sample);c.call('load_scene',path=relative(OUT/'sample.scene.json'))
            points=[]
            amap={a['id']:a for a in assets}
            from build_representative_forest import world
            for inst in sample['meshes']['instances']:
                if inst['id']=='gc_ground':continue
                points.extend(world(inst,p) for p in amap[inst['asset']]['geometry']['positions'])
            lo=[min(p[k] for p in points) for k in range(3)];hi=[max(p[k] for p in points) for k in range(3)]
            fit(c,lo,hi,OUT/f"{f['id']}.png")
            report.setdefault('families',{})[f['id']]=dict(bounds=[lo,hi],variants=len(entries),triangles_per_variant=[sum(len(amap[i['asset']]['geometry']['triangles']) for i in templates[e['id']]) for e in entries])
        field,counts,placements=field_scene(compiled,templates,manifest,recipe)
        again,_,second=field_scene(compiled,templates,manifest,recipe)
        assert placements==second and field==again
        report['placement_reproducible']=True;report['distribution']=counts;report['field']=scene_stats(field)
        report['source_bytes']=sum((destination/path).stat().st_size for path in files)
        atomic_json(OUT/'placements.json',placements);atomic_json(OUT/'field.scene.json',field)
        c.call('load_scene',path=relative(OUT/'field.scene.json'))
        c.call('set_camera',camera=dict(position=[recipe['size']*.65,recipe['size']*.53,recipe['size']*.77],target=config['field_target'],fov_degrees=49))
        c.call('capture',path=relative(OUT/'field.png'),width=1800,height=1200)
        c.call('set_camera',camera=config['close_camera'])
        c.call('capture',path=relative(OUT/'close.png'),width=1600,height=1000)
        c.call('save_scene',path=relative(OUT/'field.scene.json'))
        c.call('load_scene',path=relative(OUT/'field.scene.json'))
        c.call('capture',path=relative(OUT/'roundtrip.png'),width=1600,height=1000)
        report['scene_roundtrip_pixels_equal']=(OUT/'close.png').read_bytes()==(OUT/'roundtrip.png').read_bytes()
        if not report['scene_roundtrip_pixels_equal']:
            # Keep the discrepancy, rather than silently claim byte equality.
            # A tiny RGB8 rounding delta is allowed only alongside exact scene
            # values and a second, strictly byte-exact native save/load pair.
            comparison=c.call('compare_images',reference_path=relative(OUT/'close.png'),candidate_path=relative(OUT/'roundtrip.png'),difference_path=relative(OUT/'roundtrip-difference.png'))
            report['initial_roundtrip_comparison']=comparison['report']['metrics']
            assert comparison['report']['metrics']['max_channel_error_u8']<=1 and comparison['report']['metrics']['changed_pixels']<=1
            c.call('save_scene',path=relative(OUT/'canonical.scene.json'))
            assert json.loads((OUT/'field.scene.json').read_text())==json.loads((OUT/'canonical.scene.json').read_text())
            report['scene_values_roundtrip_equal']=True
            c.call('load_scene',path=relative(OUT/'canonical.scene.json'))
            c.call('capture',path=relative(OUT/'canonical-roundtrip.png'),width=1600,height=1000)
            assert (OUT/'roundtrip.png').read_bytes()==(OUT/'canonical-roundtrip.png').read_bytes()
            report['canonical_scene_roundtrip_pixels_equal']=True
        grass_recipe=copy.deepcopy(recipe);grass_recipe['path']=False
        grass_recipe['weights']={f['id']:config['foliage_weights'].get(f['id'],0.) for f in families}
        grass,_,_=field_scene(compiled,templates,manifest,grass_recipe)
        atomic_json(OUT/'grass.recipe.json',grass_recipe);atomic_json(OUT/'grass.scene.json',grass)
        report['grass']=scene_stats(grass)
        c.call('load_scene',path=relative(OUT/'grass.scene.json'))
        c.call('set_camera',camera=dict(position=[recipe['size']*.65,recipe['size']*.53,recipe['size']*.77],target=config['field_target'],fov_degrees=49))
        c.call('capture',path=relative(OUT/'grass.png'),width=1800,height=1200)
        c.call('save_scene',path=relative(OUT/'grass.scene.json'))
        atomic_json(OUT/'report.json',report)
        cards=''.join(f'<figure><img src="{f["id"]}.png"><figcaption>{html.escape(f["name"])}</figcaption></figure>' for f in families)
        title=f"{config['title']} · {len(families)} tipos / {len(manifest['entries'])} variações"
        (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Ignis · cobertura vegetal</title><style>body{background:#18201b;color:#e4eadc;font:17px system-ui;max-width:1500px;margin:32px auto;padding:20px}img{width:100%;display:block}figure{margin:0 0 20px}.cards{display:grid;grid-template-columns:repeat(2,1fr);gap:24px}a{color:#b5da8d}</style><h1>'+html.escape(title)+'</h1><p>Geometria procedural nativa; fontes editáveis e instâncias compartilhadas. Arquétipos autorais, sem calibração de espécie. Aceite artístico pendente.</p><h2>Campo misto</h2><img src="field.png"><img src="close.png"><h2>'+html.escape(config['foliage_title'])+'</h2><img src="grass.png"><h2>Biblioteca</h2><img src="gallery.png"><div class="cards">'+cards+'</div><p><a href="field.scene.json">Cena do campo</a> · <a href="grass.scene.json">Cena de folhagens</a> · <a href="authoring.scene.json">Fontes editáveis</a> · <a href="report.json">Contagens e validação</a></p>')
        print(json.dumps({k:v for k,v in report.items() if k!='components'},ensure_ascii=False,indent=2),flush=True)
    finally:c.close()


if __name__=='__main__':main()
