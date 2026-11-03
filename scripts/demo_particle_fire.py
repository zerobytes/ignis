#!/usr/bin/env python3
"""Build and verify an editable fire gallery with the native particle/material APIs."""
import argparse
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'artifacts/fire'
for folder in (OUT, OUT/'artifacts', OUT/'examples', OUT/'assets/particles'):
    folder.mkdir(parents=True, exist_ok=True)
os.environ['IGNIS_VERIFY_ROOT'] = str(OUT)
from verify_lab import Client

FONT = {
' ':['00000']*7,
'A':['01110','10001','10001','11111','10001','10001','10001'],
'B':['11110','10001','10001','11110','10001','10001','11110'],
'C':['01111','10000','10000','10000','10000','10000','01111'],
'D':['11110','10001','10001','10001','10001','10001','11110'],
'E':['11111','10000','10000','11110','10000','10000','11111'],
'F':['11111','10000','10000','11110','10000','10000','10000'],
'G':['01111','10000','10000','10111','10001','10001','01111'],
'H':['10001','10001','10001','11111','10001','10001','10001'],
'I':['11111','00100','00100','00100','00100','00100','11111'],
'J':['00111','00010','00010','00010','10010','10010','01100'],
'L':['10000','10000','10000','10000','10000','10000','11111'],
'M':['10001','11011','10101','10101','10001','10001','10001'],
'N':['10001','11001','10101','10011','10001','10001','10001'],
'O':['01110','10001','10001','10001','10001','10001','01110'],
'Q':['01110','10001','10001','10001','10101','10010','01101'],
'R':['11110','10001','10001','11110','10100','10010','10001'],
'S':['01111','10000','10000','01110','00001','00001','11110'],
'T':['11111','00100','00100','00100','00100','00100','00100'],
'U':['10001','10001','10001','10001','10001','10001','01110'],
'V':['10001','10001','10001','10001','10001','01010','00100'],
'Z':['11111','00001','00010','00100','01000','10000','11111'],
'0':['01110','10001','10011','10101','11001','10001','01110'],
'1':['00100','01100','00100','00100','00100','00100','01110'],
'2':['01110','10001','00001','00110','01000','10000','11111'],
'3':['11110','00001','00001','01110','00001','00001','11110'],
'4':['00010','00110','01010','10010','11111','00010','00010'],
'5':['11111','10000','10000','11110','00001','00001','11110'],
'6':['01110','10000','10000','11110','10001','10001','01110'],
'7':['11111','00001','00010','00100','01000','01000','01000'],
'8':['01110','10001','10001','01110','10001','10001','01110'],
'9':['01110','10001','10001','01111','00001','00001','01110'],
'.':['00000']*5+['00100','00100'],
}


def cylinder(radius, height, n=32):
    positions = [[radius*math.cos(i*math.tau/n), y, radius*math.sin(i*math.tau/n)]
                 for y in (-height/2, height/2) for i in range(n)]
    positions += [[0, -height/2, 0], [0, height/2, 0]]
    triangles = []
    for i in range(n):
        j = (i+1) % n
        triangles += [[i, i+n, j+n], [i, j+n, j], [2*n, i, j], [2*n+1, j+n, i+n]]
    return dict(positions=positions, triangles=triangles)


def ring(radius, thickness, n=64):
    positions = [[r*math.cos(i*math.tau/n), 0, r*math.sin(i*math.tau/n)]
                 for r in (radius-thickness, radius+thickness) for i in range(n)]
    triangles = []
    for i in range(n):
        j = (i+1) % n
        triangles += [[i, j+n, i+n], [i, j, j+n]]
    return dict(positions=positions, triangles=triangles)


def text_geometry(text):
    p, triangles = [], []
    size = 0.065
    width = len(text)*6*size
    for k, char in enumerate(text):
        for y, row in enumerate(FONT[char]):
            x = 0
            while x < len(row):
                if row[x] != '1':
                    x += 1
                    continue
                first = x
                while x < len(row) and row[x] == '1':
                    x += 1
                left, right = (k*6+first)*size-width/2, (k*6+x)*size-width/2
                bottom = (6-y)*size
                i = len(p)
                p.extend([[left, bottom, 0], [right, bottom, 0], [right, bottom+size, 0], [left, bottom+size, 0]])
                triangles.extend([[i, i+1, i+2], [i, i+2, i+3]])
    return dict(positions=p, triangles=triangles)


def settings(kind, intensity=1, seed=7):
    h, w, turbulence = {'candle': (1, .22, .12), 'campfire': (2, 1.2, .7),
                        'torch': (1.4, .35, .4), 'blue_jet': (2.4, .4, .15),
                        'inferno': (3.5, 2.3, 1.5), 'arcane': (2.2, .85, 1.2)}[kind]
    return dict(kind=kind, intensity=intensity, height=h, width=w, turbulence=turbulence, seed=seed)


def build(binary):
    c = Client(binary, log_path=OUT/'artifacts/verify.log')
    checks, captures = [], {}
    def mesh(kind, **kw): return c.call('edit_mesh', edit={'kind': kind, **kw})
    def asset(name, geometry, material=None):
        mesh('put_asset', asset=dict(id=name, geometry=geometry, shading='flat', material=material or {}))
    def inst(name, source, pos, color=(1, 1, 1), rotation=(0, 0, 0, 1), scale=1):
        mesh('put_instance', instance=dict(id=name, asset=source, position=pos, color=color, rotation=rotation, scale=scale))
    def particle(kind, **kw): return c.call('particles', operation={'kind': kind, **kw})
    def capture(name):
        captures[name] = c.call('capture', path=f'artifacts/{name}.png', width=1600, height=1100)
        return (OUT/f'artifacts/{name}.png').read_bytes()
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='native-fire-gallery', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe=dict(seed=7, width=42, depth=42, obstacles=0))
        entities = c.call('inspect')['scene']['entities']
        for e in entities: e['visible'] = False
        c.call('edit_scene', edits=[dict(op='upsert', entity=e) for e in entities])
        c.call('set_camera', camera=dict(position=[0, 24, 24], target=[0, 1, -2], fov_degrees=36))
        c.call('set_light', light=dict(direction=[-.4, -1, -.25], ambient=.2, direct_intensity=.65))
        asset('floor', dict(positions=[[-23, 0, -23], [23, 0, -23], [23, 0, 23], [-23, 0, 23]], triangles=[[0, 2, 1], [0, 3, 2]]))
        inst('floor', 'floor', [0, -.03, 0], [.025, .035, .055])
        asset('plinth', cylinder(1.65, .26), {'reflectance': {'roughness': .8, 'f0': .04}})
        asset('log', cylinder(.13, 1.65))
        asset('candle', cylinder(.14, .7))
        asset('wick', cylinder(.018, .12))
        asset('stem', cylinder(.07, .85), {'reflectance': {'roughness': .3, 'f0': .08}})
        asset('bowl', cylinder(.27, .22), {'reflectance': {'roughness': .3, 'f0': .08}})
        asset('arcane_ring', ring(1.1, .028), {'surface': {'version': 1, 'mode': 'additive', 'emissive': [.3, .025, .7]}})
        asset('trim', ring(1.57, .018), {'surface': {'version': 1, 'mode': 'additive', 'emissive': [.035, .075, .13]}})
        entries = [('candle', '01 VELA', -5.8, 5.5, 1), ('campfire', '02 FOGUEIRA', 0, 5.5, 1), ('torch', '03 TOCHA', 5.8, 5.5, 1),
                   ('blue_jet', '04 JATO AZUL', -5.8, -2, 1), ('inferno', '05 INCENDIO', 0, -2, 1.8), ('arcane', '06 ARCANO', 5.8, -2, 1),
                   ('campfire', '07 BRANDO 0.3', -5.8, -9.5, .3), ('campfire', '08 MEDIO 1.0', 0, -9.5, 1), ('campfire', '09 FORTE 2.2', 5.8, -9.5, 2.2)]
        for index, (kind, title, x, z, intensity) in enumerate(entries):
            name = f'fire_{index+1:02}_{kind}'
            cfg = settings(kind, intensity=intensity, seed=17+index)
            if kind == 'inferno':
                cfg.update(height=2.6, width=1.7)
            if index >= 6:
                cfg.update(height=1.8, width=.95, turbulence=.8, seed=37)
            particle('fire', id=name, settings=cfg)
            y = .28
            if kind == 'candle':
                inst(name+'_wax', 'candle', [x, .61, z], [.8, .69, .42])
                inst(name+'_wick', 'wick', [x, 1.0, z], [.018, .015, .012])
                y = 1.04
            elif kind == 'torch':
                inst(name+'_stem', 'stem', [x, .72, z], [.16, .11, .055])
                inst(name+'_bowl', 'bowl', [x, 1.25, z], [.16, .12, .07])
                y = 1.35
            elif kind == 'blue_jet':
                inst(name+'_nozzle', 'bowl', [x, .39, z], [.12, .17, .25])
                y = .5
            elif kind in ('campfire', 'inferno'):
                for k, yaw in enumerate((0, math.tau/3, 2*math.tau/3)):
                    # q_yaw * q_z90 maps the cylinder's Y axis into the ground plane.
                    a, b = math.sin(yaw/2), math.cos(yaw/2)
                    inst(name+f'_log{k}', 'log', [x, .42+k*.055, z], [.09, .035, .012], [a/math.sqrt(2), a/math.sqrt(2), b/math.sqrt(2), b/math.sqrt(2)])
                y = .48
            elif kind == 'arcane':
                inst(name+'_circle', 'arcane_ring', [x, .285, z], [0, 0, 0])
            particle('put_instance', instance=dict(id=name, source=name, position={'cell': [str(math.floor(v/1024)) for v in (x, y, z)], 'offset': [v-math.floor(v/1024)*1024 for v in (x, y, z)]}))
            inst(name+'_base', 'plinth', [x, .13, z], [.09, .105, .135])
            inst(name+'_trim', 'trim', [x, .265, z], [0, 0, 0])
            label = name+'_label'
            asset(label, text_geometry(title), {'surface': {'version': 1, 'mode': 'opaque', 'emissive': [.35, .42, .55]}})
            inst(label, label, [x, .28, z+1.9])
            particle('save_source', id=name, path=f'assets/particles/{name}.ignis-vfx')
        checks.append('Nine native recipe-bound effects across six families and three matched intensities')
        compact = sum(p.stat().st_size for p in (OUT/'assets/particles').glob('fire_*.ignis-vfx'))
        assert compact < 20000, compact
        checks.append(f'Compact source recipes: {compact} bytes total; derived maps rebuilt natively')
        for tick in (45, 120, 240):
            particle('preview', tick=tick)
            capture(f'gallery-{tick}')
        a = capture('same-time')
        particle('preview', tick=9000)
        particle('preview', tick=240)
        assert a == capture('seek-return')
        checks.append('Repeated/seek-return native gallery PNG exact')
        c.call('save_scene', path='examples/particle-fire.scene.json')
        particle('preview', tick=None)
        c.call('step', ticks=180)
        a = capture('world')
        c.call('save_portable_save', path='artifacts/fire.ignis-portable')
        c.call('load_portable_save', path='artifacts/fire.ignis-portable')
        assert a == capture('portable-restored')
        checks.append('Portable restore regenerates material maps and native PNG exactly')
        c.call('load_scene', path='examples/particle-fire.scene.json')
        c.call('step', ticks=180)
        assert a == capture('scene-restored')
        checks.append('Scene restore native PNG exact')
        particle('open', open=True)
        c.call('capture_editor', path='artifacts/editor.png', width=1440, height=1000)
        checks.append('Shared native fire controls rendered in the editor')
        example = json.loads((OUT/'examples/particle-fire.scene.json').read_text())
        example['version'] = 118
        (ROOT/'examples/particle-fire.scene.json').write_text(json.dumps(example, indent=2)+'\n')
        (ROOT/'assets/particles/fire').mkdir(parents=True, exist_ok=True)
        for source in (OUT/'assets/particles').glob('fire_*.ignis-vfx'):
            shutil.copyfile(source, ROOT/'assets/particles/fire'/source.name)
        (OUT/'verification.json').write_text(json.dumps(dict(checks=checks, captures=captures), indent=2)+'\n')
        (OUT/'review.html').write_text('<!doctype html><meta charset="utf-8"><title>Ignis fire</title><style>body{background:#111927;color:#eef;font:16px system-ui;margin:32px}img{width:100%;margin:20px 0}</style><h1>Native fire families</h1><p>Candle · campfire · torch · blue jet · inferno · arcane. Back row: matched low / medium / high intensity.</p><img src="artifacts/gallery-240.png"><img src="artifacts/editor.png">')
        print(json.dumps(dict(checks=checks, source_bytes=compact, review=str(OUT/'review.html'))))
    finally:
        c.close()


def open_gallery(binary):
    # Preserve running human scenes; a repeated launch reuses this gallery.
    receipt = OUT/'launch.json'
    if receipt.exists():
        previous = json.loads(receipt.read_text())
        probe = subprocess.run(['ps', '-p', str(previous['pid']), '-o', 'command='], capture_output=True, text=True)
        if probe.returncode == 0 and '--scene examples/particle-fire.scene.json' in probe.stdout:
            print(json.dumps(dict(pid=previous['pid'], already_running=True)))
            return
    c = Client(binary, window=True, detached=True, log_path=OUT/'native.log', startup_args=('--editor', '--scene', 'examples/particle-fire.scene.json', '--continuous', '--root', str(ROOT)))
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='fire-showcase', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('particles', operation=dict(kind='open', open=True))
        c.call('step', ticks=150)
        c.call('play')
        initial = c.call('inspect')['tick']
        deadline = time.monotonic()+20
        while c.call('inspect')['tick'] == initial:
            if time.monotonic() > deadline:
                raise RuntimeError('Native fire gallery clock did not advance')
            time.sleep(.5)
        report = dict(pid=c.process.pid, scene='examples/particle-fire.scene.json', playing=True, controls='Select a fire under Partículas, adjust its controls, then click Aplicar e executar')
        (OUT/'launch.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report))
    except BaseException:
        c.close()
        raise
    else:
        c.process.stdin.close()
        c.process.stdout.close()
        c.log.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    parser.add_argument('--open', action='store_true')
    parser.add_argument('--open-only', action='store_true')
    args = parser.parse_args()
    if not args.open_only:
        build(args.binary.resolve())
    if args.open or args.open_only:
        app = Path.home()/'Applications/Ignis Editor.app/Contents/MacOS/ignis'
        open_gallery(app if app.is_symlink() and app.resolve() == args.binary.resolve() else args.binary.resolve())
