#!/usr/bin/env python3
"""Native material gallery and MCP transaction checks; preserves the user's world."""
import argparse
import copy
import json
import hashlib
import math
import os
from pathlib import Path
import shutil

PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT / 'artifacts/material-transparency'
OUTPUT.mkdir(parents=True, exist_ok=True)
(OUTPUT / 'artifacts').mkdir(exist_ok=True)
(OUTPUT / 'examples').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
from verify_lab import Client


def quad(width, height):
    return {'positions': [[-width/2, -height/2, 0], [width/2, -height/2, 0],
                          [width/2, height/2, 0], [-width/2, height/2, 0]],
            'triangles': [[0,1,2], [0,2,3]]}


def cube(size):
    x,y,z = [v/2 for v in size]
    return {'positions': [[-x,-y,-z],[x,-y,-z],[x,y,-z],[-x,y,-z],[-x,-y,z],[x,-y,z],[x,y,z],[-x,y,z]],
            'triangles': [[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],
                          [3,7,6],[3,6,2],[0,4,7],[0,7,3],[1,2,6],[1,6,5]]}


def verify(binary, baseline=False):
    client = Client(binary, log_path=OUTPUT/'artifacts/runtime.log')
    checks = []
    captures = {}
    def edit(kind, **kw):
        return client.call('edit_mesh', edit={'kind':kind, **kw})
    def asset(name, geometry, material=None):
        edit('put_asset', asset={'id':name,'geometry':geometry,'shading':'flat','material':material or {}})
    def instance(name, source, position, color=(1,1,1), rotation=(0,0,0,1)):
        edit('put_instance', instance={'id':name,'asset':source,'position':position,
                                     'rotation':rotation,'scale':1,'color':color})
    def capture(name, width=1280, height=800):
        metadata = client.call('capture', path=f'artifacts/{name}.png', width=width, height=height)
        captures[name] = {key: metadata[key] for key in ['adapter','width','height','tick'] if key in metadata}
        return (OUTPUT/f'artifacts/{name}.png').read_bytes()
    def set_material(name, material):
        edit('set_material', asset=name, material=material)
    def surface(mode='blend', **kw):
        return {'surface': {'version':1, 'mode':mode, **kw}}
    try:
        client.request('initialize', {'protocolVersion':'2025-11-25','capabilities':{},
                                      'clientInfo':{'name':'material-foundation-gallery','version':'1'}})
        client.request('notifications/initialized', notification=True)
        client.call('generate', recipe={'seed':7,'width':24,'depth':20,'obstacles':0})
        entities = client.call('inspect')['scene']['entities']
        for entity in entities: entity['visible'] = False
        client.call('edit_scene', edits=[{'op':'upsert','entity':e} for e in entities])
        camera = {'position':[0,5.5,22],'target':[0,1.6,-0.7],'fov_degrees':44}
        light = {'direction':[-0.4,-1,-0.25], 'ambient':0.15, 'direct_intensity':0.8,
                 'environment':{'sky':[0.45,0.65,1],'horizon':[0.85,0.89,0.95],
                                'ground':[0.09,0.1,0.13],'axis':[0,1,0],'intensity':0.7}}
        client.call('set_camera', camera=camera)
        client.call('set_light', light=light)
        asset('floor', quad(23,18), {'reflectance':{'roughness':0.8,'f0':0.04}})
        instance('floor','floor',[0,0,-1],[0.19,0.22,0.28],[-math.sqrt(0.5),0,0,math.sqrt(0.5)])
        asset('tile',quad(0.5,0.5))
        # A high-contrast fixed backdrop makes transmitted detail visible.
        for y in range(10):
            for x in range(32):
                gray = 0.12 if (x+y)%2 else 0.78
                instance(f'tile-{x}-{y}','tile',[x*0.5-7.75,y*0.5+0.25,-3.2],[gray]*3)
        asset('object',cube([1.0,1.8,1.0]),{'reflectance':{'roughness':0.3,'f0':0.06}})
        asset('frame-h',cube([2.3,0.07,0.1]),{'reflectance':{'roughness':0.25,'f0':0.04,'metallic':0.85}})
        asset('frame-v',cube([0.07,3.4,0.1]),{'reflectance':{'roughness':0.25,'f0':0.04,'metallic':0.85}})
        names = ['clear','green','blue','translucent','cutout','additive']
        for n,name in enumerate(names):
            x = (n-2.5)*2.6
            asset(name,quad(2.2,3.3))
            instance(f'panel-{name}',name,[x,1.9,0],[1,1,1])
            instance(f'object-{name}','object',[x,1.0,-1.7],[0.85,0.19,0.045])
            for side in [-1,1]:
                instance(f'frame-h-{name}-{side}','frame-h',[x,1.9+side*1.68,0.05],[0.45,0.5,0.6])
                instance(f'frame-v-{name}-{side}','frame-v',[x+side*1.135,1.9,0.05],[0.45,0.5,0.6])
        if baseline:
            capture('legacy-baseline')
            (OUTPUT/'baseline.json').write_text(json.dumps({'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'scene_version':client.call('inspect')['scene']['version']},indent=2))
            print(json.dumps({'baseline':True,'capture':str(OUTPUT/'artifacts/legacy-baseline.png')}))
            return
        neutral = capture('legacy-current')
        reference = OUTPUT/'artifacts/legacy-baseline.png'
        if reference.exists():
            # The native PNG path is deterministic; identical bytes also prove
            # identical decoded RGB without an extra image-library dependency.
            assert reference.read_bytes() == neutral, 'Legacy opaque renderer changed pixels'
            checks.append('Legacy opaque gallery matches pre-change native binary pixel-exactly')
        schema = json.dumps(client.request('tools/list'))
        assert all(v in schema for v in ['attenuation_distance_m','coverage','emissive'])
        glass = {'ior':1.5,'roughness':0.2,'thickness_m':0.006,
                 'attenuation_color':[1,1,1],'attenuation_distance_m':1}
        materials = {
            'clear':surface(glass=glass),
            'green':surface(glass={**glass,'thickness_m':0.02,'attenuation_color':[0.05,0.6,0.18],'attenuation_distance_m':0.04}),
            'blue':surface(glass={**glass,'thickness_m':0.025,'attenuation_color':[0.12,0.35,0.95],'attenuation_distance_m':0.05}),
            'translucent':surface(opacity=0.36),
            'cutout':surface('mask',cutoff=0.5,coverage={'frequency':[3,3,3],'low':0,'high':1,'seed':9,'kind':'cellular'}),
            'additive':surface('additive',opacity=0.3,emissive=[0.08,0.7,1.5],coverage={'frequency':[2,2,2],'low':0,'high':1,'seed':7}),
        }
        before = client.call('inspect')
        preview = client.call('preview_mesh_edit',edit={'kind':'set_material','asset':'clear','material':materials['clear']})
        assert client.call('inspect') == before
        set_material('clear',materials['clear'])
        client.call('undo'); assert capture('undo') == neutral
        client.call('redo')
        for name,value in materials.items(): set_material(name,value)
        camera = {**camera,'position':[0,4.2,14],'target':[0,1.7,-0.7]}
        client.call('set_camera',camera=camera)
        gallery = capture('gallery')
        assert gallery != neutral and capture('repeat') == gallery
        snapshot = client.call('inspect')
        invalid = copy.deepcopy(materials['clear']); invalid['surface']['glass']['ior'] = 0.5
        client.call('edit_mesh',expected_error=True,edit={'kind':'set_material','asset':'clear','material':invalid})
        assert client.call('inspect') == snapshot
        client.call('save_scene',path='examples/material-transparency.scene.json')
        client.call('load_scene',path='examples/material-transparency.scene.json')
        assert capture('reopened') == gallery
        client.call('save_replay',path='artifacts/material.replay.json')
        replay = client.call('run_replay',path='artifacts/material.replay.json')
        assert replay['matched'],replay
        client.call('capture_replay',replay_path='artifacts/material.replay.json',path='artifacts/replay.png',tick=None,width=1280,height=800)
        assert (OUTPUT/'artifacts/replay.png').read_bytes() == gallery
        client.call('save_portable_save',path='artifacts/material.ignis-portable')
        client.call('load_portable_save',path='artifacts/material.ignis-portable',mode='recorded')
        assert capture('portable') == gallery
        checks.append('MCP schema/preview/undo/redo/invalid rollback/save-load/replay/portable save and repeated native captures')
        client.call('set_camera',camera={'position':[8,5,16],'target':[0,1.7,-0.6],'fov_degrees':48})
        capture('oblique')
        client.call('set_camera',camera={**camera,'position':[0,2.2,11],'fov_degrees':58})
        capture('close')
        client.call('set_camera',camera=camera)
        client.call('set_light',light={**light,'shadow':{'center':[0,2,-1],'extent':12,'resolution':1024,'bias':0.0003}})
        capture('shadow')
        client.call('set_light',light={**light,'atmosphere':{'color':[0.5,0.6,0.7],'density':0.018,'sun_color':[1,0.8,0.6],'sun_strength':0.2}})
        capture('atmosphere')
        client.call('set_light',light=light)
        client.call('select_editor_mesh',instance='panel-green')
        client.call('capture_editor',path='artifacts/editor.png',width=1600,height=1800)
        client.call('save_scene',path='examples/material-transparency.scene.json')
        shutil.copyfile(OUTPUT/'examples/material-transparency.scene.json',PROJECT/'examples/material-transparency.scene.json')
        report={'passed':True,'captures':captures,'baseline':json.loads((OUTPUT/'baseline.json').read_text()) if (OUTPUT/'baseline.json').exists() else None,'checks':checks,'replay':replay,'capture':str(OUTPUT/'artifacts/gallery.png')}
        (OUTPUT/'verification.json').write_text(json.dumps(report,indent=2))
        titles={'gallery':'Materiais compartilhados — frente','oblique':'Ângulo oblíquo','close':'Detalhe',
                'shadow':'Sombras opacas e de recorte','atmosphere':'Com atmosfera','editor':'Editor nativo — material selecionado'}
        cards=''.join(f'<figure><img src="artifacts/{name}.png"><figcaption>{title}</figcaption></figure>' for name,title in titles.items())
        (OUTPUT/'review.html').write_text('<!doctype html><html lang="pt-br"><meta charset="utf-8"><title>Materiais — EAF-203</title><style>body{background:#11151b;color:#ddd;font:16px system-ui;margin:32px auto;max-width:1320px}img{width:100%;display:block}figure{margin:28px 0}figcaption{padding:10px;background:#202832}p{line-height:1.6}</style><h1>Materiais e transparência compartilhados</h1><p>Esquerda → direita: vidro claro, vidro verde, vidro azul, translucidez, recorte procedural e aditivo. Imagens do renderer nativo. Vidro fino com Fresnel e absorção; sem refração espacial. A mistura de camadas usa composição ponderada aproximada.</p>'+cards+'</html>')
        print(json.dumps({'passed':True,'checks':checks,'replay_matched':replay['matched'],'capture':report['capture']},indent=2))
    finally:
        client.close()

if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,default=PROJECT/'target/release/ignis')
    parser.add_argument('--baseline',action='store_true')
    args=parser.parse_args()
    verify(args.binary.resolve(),args.baseline)
