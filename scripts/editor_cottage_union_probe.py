"""Verify disjoint cottage deltas against both committed source fronts."""
from verify_lab import SCENE_VERSION
import copy
import hashlib
import json
import os
import subprocess
from pathlib import Path

P = Path(__file__).resolve().parents[1]
R = P/'artifacts/editor-cottage-union'


def main():
    for name in ['examples', 'artifacts']:
        (R/name).mkdir(parents=True, exist_ok=True)
    refs = {'relief': '460a40761e0e33c314666df0d2c0d11af75e2629',
            'stone': 'f60881843a785d7b041e75f4e93a303f06dafdd2'}
    sources = {name: subprocess.check_output(
        ['git', 'show', ref+':examples/elven-cottage.scene.json'], cwd=P)
        for name, ref in refs.items()}
    sources['preassembly'] = subprocess.check_output(['git','show','5e06a61:examples/elven-cottage.scene.json'],cwd=P)
    sources['joint'] = (P/'examples/elven-cottage.scene.json').read_bytes()
    scenes = {name: json.loads(data) for name, data in sources.items()}
    def without(scene, ids):
        scene = copy.deepcopy(scene)
        for asset in scene['meshes']['assets']:
            if asset['id'] in ids:
                asset.pop('geometry')
        return scene
    assert without(scenes['stone'], ['bronze']) == without(scenes['preassembly'], ['bronze'])
    assert without(scenes['relief'], ['paving', 'stone-blocks']) == without(scenes['preassembly'], ['paving', 'stone-blocks'])
    def asset(stage, id_):
        return next(a for a in scenes[stage]['meshes']['assets'] if a['id'] == id_)
    assert asset('preassembly', 'bronze') == asset('relief', 'bronze')
    for id_ in ['paving', 'stone-blocks']:
        assert asset('preassembly', id_) == asset('stone', id_)
    for name, data in sources.items():
        (R/f'examples/{name}.scene.json').write_bytes(data)
    os.environ['IGNIS_VERIFY_ROOT'] = str(R)
    from verify_lab import Client
    from verify_materials import f32
    c = Client(P/'target/release/ignis')
    call = c.call
    checks = ['Joint source preserves exact bronze from relief front and exact stone/paving from chamfer front; all other scene data/channels invariant']
    views = {
        'frieze': dict(position=[-.5,4.3,8],target=[-2.3,3.5,3.2],fov_degrees=24),
        'paving': dict(position=[3.4,1.8,6.4],target=[1.45,.30,3.9],fov_degrees=42),
        'entry-stone': dict(position=[4.2,3.3,5.1],target=[2.75,2.4,2.1],fov_degrees=30)}
    pixels = {}
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
            clientInfo=dict(name='editor-cottage-union',version='1')))
        c.request('notifications/initialized', notification=True)
        for stage in ['relief', 'stone', 'preassembly', 'joint']:
            call('load_scene', path=f'examples/{stage}.scene.json')
            expected = f32(scenes[stage]); expected['version'] = SCENE_VERSION
            assert call('inspect')['scene'] == expected
            pixels[stage] = {}
            for name, camera in views.items():
                call('set_camera', camera=camera)
                for suffix in ['', '-repeat']:
                    path=f'artifacts/{stage}-{name}{suffix}.png'
                    call('capture',path=path,width=1000,height=800)
                image=(R/f'artifacts/{stage}-{name}.png').read_bytes()
                assert image == (R/f'artifacts/{stage}-{name}-repeat.png').read_bytes()
                pixels[stage][name]=image
        assert pixels['joint']['frieze'] != pixels['stone']['frieze']
        for name in ['paving', 'entry-stone']:
            assert pixels['joint'][name] != pixels['relief'][name]
        checks.append('Fixed native captures repeat exactly; combined frieze differs from old placement, and stone/paving differ from unchamfered front')
        call('load_scene',path='examples/preassembly.scene.json')
        from cottage_composition import adjust_lantern
        adjust_lantern(call, 2.1, [-.27,0,0])
        expected=f32(scenes['joint']);expected['version']=SCENE_VERSION
        assert call('inspect')['scene']==expected
        checks.append('Joint content equals native lantern transform on the exact preassembly relief/chamfer union')
        call('load_scene',path='examples/joint.scene.json')
        state=call('inspect')
        for id_ in ['bronze', 'paving', 'stone-blocks']:
            top=call('inspect_mesh_topology',asset=id_)['topology']
            assert top['boundary_edges']==top['non_manifold_edges']==0
            assert not top['isolated_vertices']
        assert call('inspect')==state
        meshes=state['scene']['meshes']
        vertices=sum(len(a['geometry']['positions']) for a in meshes['assets'])
        triangles=sum(len(a['geometry']['triangles']) for a in meshes['assets'])
        assert (vertices,triangles)==(32596,54728)
        checks.append('Combined bronze/stone/paving remain closed manifold without isolated vertices; 32596vertices/54728triangles and unchanged limits')
        report=dict(passed=True,checks=checks,build_id=state['build_id'],refs=refs,
            source_sha256={k:hashlib.sha256(v).hexdigest() for k,v in sources.items()},
            vertices=vertices,triangles=triangles,remaining_vertices=32768-vertices,
            limitations=['Authored geometry and isolated visual differences only; no artistic acceptance',
                'Source-front studies remain historical and retain their original baseline checks'])
        (R/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2))
    finally:
        c.close()


if __name__ == '__main__':
    main()
