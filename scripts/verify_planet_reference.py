#!/usr/bin/env python3
"""Earth-scale reference camera, large global translation, GPU/CPU silhouette and native terrain.
No GUI, external Python packages, atmosphere/refraction or gameplay acceptance.
"""
import argparse
import base64
import copy
import html
import json
import math
from pathlib import Path
from png_pixels import rgba8
from verify_lab import Client, ROOT

OUT = ROOT / 'artifacts/earth-horizon'
RADIUS = 6_371_000.

def camera(h, pitch=0, mask=False):
    return {'pose': {'direction': [0, 1, 0], 'forward': [0, 0, -1], 'altitude_meters': h},
            'pitch_degrees': pitch, 'fov_degrees': 60, 'diagnostic_mask': mask}

def oracle(view, width, height):
    """Independent f64 physical-meter quadratic, evaluated at pixel centers."""
    h = view['pose']['altitude_meters']
    pitch = math.radians(view['pitch_degrees'])
    s, c = math.sin(pitch), math.cos(pitch)
    ty = math.tan(math.radians(view['fov_degrees']) / 2)
    tx = ty * width / height
    rows = bytearray(width * height)
    for y in range(height):
        yy = (1 - 2 * (y + .5) / height) * ty
        for x in range(width):
            xx = (2 * (x + .5) / width - 1) * tx
            dot = (s + yy * c) / math.sqrt(1 + xx * xx + yy * yy)
            b = (RADIUS + h) * dot
            rows[y * width + x] = b < 0 and b * b >= h * (2 * RADIUS + h)
    return rows

def compare_mask(png, view, native=False):
    w, h, pixels = rgba8(png)
    expected = oracle(view, w, h)
    mismatch = []
    off_boundary = []
    for i, truth in enumerate(expected):
        if native:
            hit = pixels[4*i + 1] > pixels[4*i] + 50
        else:
            assert pixels[4*i] in (0, 255), pixels[4*i:4*i+4]
            hit = pixels[4*i] == 255
        if hit != bool(truth):
            mismatch.append(i)
            x, y = i % w, i // w
            # Only raster/float uncertainty immediately at the silhouette is tolerated.
            neighbor = any(expected[yy*w+xx] != truth
                           for yy in range(max(0,y-1),min(h,y+2))
                           for xx in range(max(0,x-1),min(w,x+2)))
            if not neighbor: off_boundary.append(i)
    assert not off_boundary, {'mismatch': len(mismatch), 'off_boundary': len(off_boundary), 'first': off_boundary[:8]}
    assert 0 < sum(expected) < len(expected), 'Fixture must include visible sphere and sky'
    return {'width': w, 'height': h, 'mismatching_pixels': len(mismatch),
            'off_boundary_pixels': len(off_boundary), 'sphere_pixels_cpu': sum(expected)}

def verify(binary, window=False):
    OUT.mkdir(parents=True, exist_ok=True)
    source = json.loads((ROOT / 'examples/planet.recipe.json').read_text())
    source.update(id='earth_reference', diameter_meters=RADIUS*2, base_height_meters=0, operations=[])
    remote = copy.deepcopy(source)
    remote['center'] = {'cell': ['9007199254740993', '-9007199254740993', '9007199254740993'], 'offset': [.125, .5, .25]}
    (OUT/'source.recipe.json').write_text(json.dumps(source, indent=2))
    c = Client(binary, log_path=OUT/'runtime.log')
    report = {'native_window': False, 'passed': False, 'reference_only': True, 'translated_center': remote['center'], 'images': [], 'gpu_oracle': []}
    def capture(name, document, view, width=1280, height=800):
        path = f'artifacts/earth-horizon/{name}.png'
        response = c.request('tools/call', {'name': 'engine_command', 'arguments': {'command': {
            'op': 'capture_planet_reference', 'document': document, 'view': view,
            'path': path, 'width': width, 'height': height}}})
        assert 'error' not in response, response
        result = response['result']
        assert not result['isError'], result
        image = next(x for x in result['content'] if x['type'] == 'image')
        png = base64.b64decode(image['data'])
        assert png == (ROOT/path).read_bytes()
        return result['structuredContent'], png
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                  'clientInfo': {'name': 'earth-horizon', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        before = c.call('inspect')
        report['build_id'] = before['build_id']
        # Bad input must preserve both live state and an existing output file.
        protected = OUT/'protected.png'
        protected.write_bytes(b'preserved-output')
        bad = copy.deepcopy(source)
        bad['base_height_meters'] = 100
        c.call('capture_planet_reference', document=bad, view=camera(2), path='artifacts/earth-horizon/protected.png', width=1280, height=800, expected_error=True)
        c.call('capture_planet_reference', document=source, view=camera(0), path='artifacts/earth-horizon/protected.png', width=1280, height=800, expected_error=True)
        assert protected.read_bytes() == b'preserved-output'
        protected.unlink()
        for name, h, pitch in [('eye-2m',2,0),('tower-100m',100,0),('flight-10km',10000,0),('orbit-400km',400000,-25),('full-earth',10000000,-90)]:
            v = camera(h, pitch)
            metadata, png = capture(name, source, v)
            assert metadata['diameter_meters'] == RADIUS*2 and metadata['whole_reference_sphere']
            shifted, remote_png = capture('translated-check', remote, v)
            assert png == remote_png, name
            assert shifted['camera_position']['cell'][0] == '9007199254740993'
            assert shifted['camera_position']['offset'][0] == .125
            metadata.update(name=name, file=f'{name}.png')
            report['images'].append(metadata)
            _, mask = capture(f'mask-{name}', source, camera(h,pitch,True), width=400, height=256)
            report['gpu_oracle'].append({'name':name, **compare_mask(mask,camera(h,pitch,True))})
        (OUT/'translated-check.png').unlink()
        assert c.call('inspect') == before
        report['global_translation_pixel_exact'] = True
        # A complete ~12.4km square of Earth geometry covers the entire 5.048km eye-level
        # horizon disk. All outer edges are beyond the horizon, never used as a fake horizon.
        c.call('generate', recipe={'width':12,'depth':12,'seed':1,'obstacles':0})
        empty = c.call('inspect')['scene']
        c.call('edit_scene', edits=[{'op':'remove','id':e['id']} for e in empty['entities']])
        frame = {'kind':'surface','pose':camera(2)['pose']}
        frame['pose']['altitude_meters'] = 0
        native = {'vertices':0,'triangles':0,'maximum_projection_error_meters':0}
        for y in range(2046,2050):
            for x in range(2046,2050):
                name=f'earth_{x}_{y}'
                result=c.call('publish_planet', publication={'document':remote,
                    'area':{'kind':'tile','face':'positive_y','level':12,'tile':[x,y],'subdivisions':128},
                    'frame':frame,'asset':name,'instance':name,'color':[.05,.8,.05],'collision':False})['planet']
                native['vertices']+=result['vertices'];native['triangles']+=result['triangles']
                native['global_origin']=result['global_origin']
                native['maximum_projection_error_meters']=max(native['maximum_projection_error_meters'],result['maximum_projection_error_meters'])
        light = empty['light']
        light.update(direction=[.3,-1,.2],ambient=.8,atmosphere=None,shadow=None)
        c.call('set_light',light=light)
        c.call('set_camera',camera={'position':[0,2,0],'target':[0,2,-100],'fov_degrees':60})
        c.call('capture',path='artifacts/earth-horizon/native-eye-2m.png',width=1280,height=800)
        native['oracle']=compare_mask((OUT/'native-eye-2m.png').read_bytes(),camera(2),native=True)
        native['minimum_edge_surface_distance_meters']=RADIUS*math.atan(1/1024)
        native['horizon_surface_distance_meters']=RADIUS*math.atan2(math.sqrt(2*(2*RADIUS+2)),RADIUS)
        assert native['minimum_edge_surface_distance_meters'] > native['horizon_surface_distance_meters']
        c.call('save_scene',path='artifacts/earth-horizon/native-eye.scene.json')
        report['native_mesh']=native
        report.update(passed=True, checks=4, physics_origin_shift=False, legacy_scene_guards_removed=False)
        (OUT/'verification.json').write_text(json.dumps(report,indent=2))
        sections=[]
        for m in report['images']:
            text=(f"h={m['altitude_meters']:,.2f} m · horizonte {m['horizon_line_of_sight_meters']/1000:,.3f} km · "
                  f"depressão {m['horizon_dip_degrees']:.6f}° · FOV vertical60° · pitch{m['view']['pitch_degrees']}°")
            sections.append(f"<section><h2>{html.escape(m['name'])}</h2><p>{html.escape(text)}</p><img src='{m['file']}'></section>")
        (OUT/'review.html').write_text("""<!doctype html><meta charset='utf-8'><title>Ignis — horizonte terrestre</title>
<style>body{background:#151e2b;color:#eef2f7;font:16px system-ui;max-width:1400px;margin:30px auto;padding:20px}img{width:100%;height:auto}section{margin:36px 0}p{line-height:1.5}</style>
<h1>Planeta de12.742km — câmera nativa de referência</h1><p>Esfera ideal. Sem relevo, refração ou atmosfera. Cada imagem renderizada pelo wgpu da Ignis, sem imagem externa ou deformação de tela. Mesmo diâmetro em todas as vistas.</p>"""+''.join(sections)+"<h2>Malha nativa, câmera2m</h2><p>16tiles da mesma esfera, cobrindo todo o disco do horizonte. Centro em coordenadas globais acima de2^53 células.</p><img src='native-eye-2m.png'>")
        print(json.dumps({'passed':True,'checks':4,'build_id':report['build_id'],'native_window':False,
                          'native_mesh':native,'gpu_oracle':report['gpu_oracle']}))
    finally:
        c.close()
    return report

if __name__=='__main__':
    p=argparse.ArgumentParser()
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis')
    p.add_argument('--window',action='store_true',help='Runner compatibility; captures always remain offscreen')
    a=p.parse_args();verify(a.binary,a.window)
