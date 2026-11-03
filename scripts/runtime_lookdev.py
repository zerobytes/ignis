#!/usr/bin/env python3
"""Reproducible native lighting ablations, without opening the user's window.

Reads existing assets, never regenerates their canonical sources. All edits use
the same MCP commands as the editor. Outputs replace the current laboratory.
"""
import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import time

PROJECT = Path(__file__).resolve().parents[1]
OUT = PROJECT / 'artifacts/runtime-lookdev'
OUT.mkdir(parents=True, exist_ok=True)
(OUT / 'artifacts').mkdir(exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUT)
from verify_lab import Client


def write(path, value):
    path.write_text(json.dumps(value, separators=(',', ':')))


def link_input(source, name):
    path = OUT / name
    path.unlink(missing_ok=True)
    os.link(source, path)
    return name


def review_html():
    panels = ['<h2>Shared-geometry garden</h2><div class="grid">'+''.join(f'<figure><a href="artifacts/garden-{n}.png"><img src="artifacts/garden-{n}.png"></a><figcaption>{n} library specimens; identical source geometry</figcaption></figure>' for n in [4,16,64])+'</div>']
    for case in ['hero', 'library']:
        panels.append(f'<h2>{case}</h2><div class="pair"><figure><img src="artifacts/{case}-before.png"><figcaption>Original light, matched camera</figcaption></figure><figure><img src="artifacts/{case}-after.png"><figcaption>Calibrated native lighting + receiver-plane PCF</figcaption></figure></div>')
        panels.append('<div class="grid">'+''.join(f'<figure><a href="artifacts/{case}-{name}.png"><img loading="lazy" src="artifacts/{case}-{name}.png"></a><figcaption>{name}</figcaption></figure>' for name in ['shadow-only','lighting','pcf-legacy','close','side','back','no-shadow','no-ibl'])+'</div>')
        panels.append(f'<figure><img id="{case}-motion" src="artifacts/{case}-motion-00.png"><figcaption>Eight discrete camera samples. <input type="range" min="0" max="7" value="0" oninput="document.getElementById(\'{case}-motion\').src=\'artifacts/{case}-motion-\'+this.value.padStart(2,\'0\')+\'.png\'"></figcaption></figure>')
    (OUT/'review.html').write_text('''<!doctype html><meta charset="utf-8"><title>Ignis native look development</title><style>body{background:#15191e;color:#e8edf1;font:16px system-ui;max-width:1500px;margin:40px auto;padding:20px}img{width:100%}figure{margin:0}figcaption{padding:10px}.pair{display:grid;grid-template-columns:1fr 1fr;gap:12px}.grid{display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin:24px 0}figure:has(input){max-width:720px}a{color:#98ddbf}</style><h1>Native runtime look development</h1><p>Same geometry and pinhole camera per paired case. Native captures, no image retouching. Lighting/material calibration and PCF correction are distinct ablations. These are not Cycles-equivalent lighting or gameplay FPS.</p><p><a href="verification.json">Provenance and checks</a> · <a href="hero-cost.json">Hero cost</a> · <a href="library-cost.json">Library scaling cost</a></p>'''+''.join(panels))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=PROJECT/'target/release/ignis')
    args = parser.parse_args()
    report = {'binary_sha256': hashlib.sha256(args.binary.read_bytes()).hexdigest(),
              'captures': {}, 'sources': {}, 'cases': [],
              'timing_scope': 'MCP capture wall time includes frame preparation, GPU, readback, PNG and IPC; not FPS'}
    manifest = json.loads((PROJECT/'assets/botany/reference-shrubs/library.json').read_text())
    specimen = next(e for e in manifest['entries'] if e['id'] == 'esfera_miuda_0')
    sources = [PROJECT/'assets/botany/reference-shrubs'/p['path'] for p in specimen['components']]
    for source in sources:
        report['sources'][str(source.relative_to(PROJECT))] = hashlib.sha256(source.read_bytes()).hexdigest()
    client = Client(args.binary, log_path=OUT/'artifacts/runtime.log')
    def capture(name):
        started = time.monotonic()
        result = client.call('capture', path=f'artifacts/{name}.png', width=1200, height=1200)
        report['captures'][name] = {'wall_seconds': time.monotonic()-started,
                                    'camera': result['camera'], 'adapter': result['adapter']}
        write(OUT/'verification.json', report)
        print(name, round(report['captures'][name]['wall_seconds'], 3), flush=True)
    def camera(angle=0, close=False):
        # Exactly the Cycles pinhole pose and 85mm/36mm gate at angle zero.
        x,z = 1.35,3.1
        if close: x,z = .56,1.29
        return {'position':[x*math.cos(angle)+z*math.sin(angle), 1.02 if close else 1.55,
                            z*math.cos(angle)-x*math.sin(angle)],
                'target':[0,.74 if close else .52,0],
                'fov_degrees':math.degrees(2*math.atan(36/170))}
    try:
        client.request('initialize', {'protocolVersion':'2025-11-25','capabilities':{},
                                     'clientInfo':{'name':'runtime-lookdev','version':'1'}})
        client.request('notifications/initialized', notification=True)
        for case in ['hero', 'library']:
            started = time.monotonic()
            if case == 'hero':
                client.call('load_scene', path=link_input(PROJECT/'artifacts/shrub-hero/native.scene.json', 'input.scene.json'))
                # Read only the small light recipe, avoiding a Python copy of the hero mesh.
                original = dict(direction=[.45,-.82,-.35], ambient=.4, direct_intensity=1.1,
                    environment=dict(sky=[.55,.65,.78],horizon=[.58,.62,.57],ground=[.13,.16,.075],axis=[0,1,0],intensity=.5),
                    shadow=dict(center=[0,0,0],extent=18,resolution=2048,bias=.001))
            else:
                client.call('generate', recipe=dict(width=8,depth=8,seed=208,obstacles=0))
                state = client.call('inspect')
                report['build_id'] = state['build_id']
                entities = state['scene']['entities']
                client.call('edit_scene', edits=[dict(op='remove',id=e['id']) for e in entities])
                for index,source in enumerate(sources):
                    client.call('botany', operation=dict(kind='load', path=link_input(source, f'input-{index}.ignis-botany'),publish=True))
            report['cases'].append({'case':case,'load_or_compile_seconds':time.monotonic()-started})
            client.call('set_camera', camera=camera())
            client.call('set_light', light=original)
            # Shared ground and camera are identical in every paired ablation.
            client.call('edit_mesh', edit=dict(kind='put_asset',asset=dict(id='lookdev_ground',
                geometry=dict(positions=[[-4,-.03,-4],[4,-.03,-4],[-4,-.03,4],[4,-.03,4]],triangles=[[0,2,1],[1,2,3]]),
                material=dict(reflectance=dict(roughness=.92,f0=.025)))))
            client.call('edit_mesh', edit=dict(kind='put_instance',instance=dict(id='lookdev_ground',asset='lookdev_ground',
                position=[0,0,0],rotation=[0,0,0,1],scale=1,color=[.018,.012,.006])))
            capture(f'{case}-before')
            baseline = OUT/'legacy-baseline.json'
            if baseline.exists():
                assert hashlib.sha256((OUT/f'artifacts/{case}-before.png').read_bytes()).hexdigest() == json.loads(baseline.read_text())[case], 'Legacy renderer changed'
            fitted = copy.deepcopy(original)
            fitted['shadow'] = dict(center=[0,.52,0],extent=1.15,resolution=2048,bias=.00006)
            client.call('set_light', light=fitted)
            capture(f'{case}-shadow-only')
            final = copy.deepcopy(fitted)
            final.update(direction=[.6,-1,-.8],ambient=.015,direct_intensity=1.8)
            final['environment']['intensity'] = .12
            client.call('set_light', light=final)
            capture(f'{case}-lighting')
            # Explicit asset IDs are authoring assignments, not shader species rules.
            leaf_assets = [f"{p['prefix']}_leaves" for p in specimen['components']]
            for asset in leaf_assets:
                client.call('edit_mesh', edit=dict(kind='set_material',asset=asset,
                    material=dict(transmission=.12,reflectance=dict(roughness=.48,f0=.035))))
            capture(f'{case}-pcf-legacy')
            final['shadow']['receiver_plane_bias'] = True
            client.call('set_light', light=final)
            capture(f'{case}-after')
            for name,angle,close in [('side',1.3,False),('back',math.pi,False),('close',0,True)]:
                client.call('set_camera', camera=camera(angle,close))
                capture(f'{case}-{name}')
            client.call('set_camera', camera=camera())
            client.call('save_scene', path=f'{case}.scene.json')
            client.call('load_scene', path=f'{case}.scene.json')
            capture(f'{case}-reopened')
            assert (OUT/f'artifacts/{case}-reopened.png').read_bytes() == (OUT/f'artifacts/{case}-after.png').read_bytes()
            # Paired controls isolate the remaining unoccluded environment fill.
            no_shadow = copy.deepcopy(final); no_shadow.pop('shadow')
            client.call('set_light', light=no_shadow); capture(f'{case}-no-shadow')
            no_probe = copy.deepcopy(final); no_probe.pop('environment')
            client.call('set_light', light=no_probe); capture(f'{case}-no-ibl')
            client.call('set_light', light=final)
            for index in range(8):
                client.call('set_camera', camera=camera((index-3.5)*.025))
                capture(f'{case}-motion-{index:02}')
            client.call('set_camera', camera=camera())
            capture(f'{case}-repeat')
            assert (OUT/f'artifacts/{case}-repeat.png').read_bytes() == (OUT/f'artifacts/{case}-after.png').read_bytes()
        report['repeated_pose_png_exact'] = True
        report['save_load_png_exact'] = True
        # Shared geometry, progressive garden density. Replace one current scene;
        # image names identify different workloads, never historical builds.
        garden = json.loads((OUT/'library.scene.json').read_text())
        garden['botany'] = []
        templates = [i for i in garden['meshes']['instances'] if i['id'] != 'lookdev_ground']
        ground = next(i for i in garden['meshes']['instances'] if i['id'] == 'lookdev_ground')
        for count in [4,16,64]:
            side = math.ceil(math.sqrt(count))
            ground['scale'] = max(1, side/2)
            garden['meshes']['instances'] = [ground]
            garden['meshes']['objects'] = []
            for n in range(count):
                for original in templates:
                    item = copy.deepcopy(original)
                    item.update(id=f'copy_{n}_{original["id"]}', position=[(n%side-(side-1)/2)*1.45,0,(n//side-(side-1)/2)*1.45])
                    garden['meshes']['instances'].append(item)
            garden['camera'].update(position=[side*2.2,side*2.6,side*4],target=[0,.5,0])
            garden['light']['shadow'].update(extent=side*1.5,center=[0,.5,0])
            write(OUT/'garden.scene.json', garden)
            client.call('load_scene', path='garden.scene.json')
            capture(f'garden-{count}')
        assert all(hashlib.sha256((PROJECT/p).read_bytes()).hexdigest()==h for p,h in report['sources'].items())
        report['canonical_sources_unchanged'] = True
        write(OUT/'verification.json', report)
        review_html()
    finally:
        client.close()
        for name in ['input.scene.json','input-0.ignis-botany','input-1.ignis-botany']:
            (OUT/name).unlink(missing_ok=True)


if __name__ == '__main__':
    main()
