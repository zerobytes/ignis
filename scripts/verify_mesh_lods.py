#!/usr/bin/env python3
"""IGL1 shared MCP boundary + visual optional path; no FPS claim from this probe."""
import argparse
import json
from verify_lab import Client, ROOT
from build_landscape_forest import tree_assets


def main(binary):
    out = ROOT / 'artifacts/automatic-lod'
    out.mkdir(parents=True, exist_ok=True)
    c = Client(binary, log_path=out / 'mcp.log')
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='mesh-lods', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe=dict(width=30, depth=30, seed=181, obstacles=0))
        assets, _ = tree_assets()
        meshes = dict(assets=assets, instances=[])
        for i in range(32):
            for j, asset in enumerate(assets):
                meshes['instances'].append(dict(id=f'object_{i}_{j}', asset=asset['id'], position=[(i % 8 - 3.5)*5, 0, -(i//8)*10], rotation=[0, 0, 0, 1], scale=1, color=[.26,.13,.05] if j == 0 else [.15,.32,.08]))
        source = c.call('inspect')['scene']
        source['meshes'] = meshes
        (out/'trees.scene.json').write_text(json.dumps(source))
        c.call('load_scene', path='artifacts/automatic-lod/trees.scene.json')
        source = c.call('inspect')['scene']
        compiled = c.call('compile_mesh_lods', path='artifacts/automatic-lod/trees.ignis-lod')
        before = c.call('inspect_mesh_lods')
        (out / 'corrupt.ignis-lod').write_bytes(b'invalid')
        rejected = False
        try:
            c.call('load_mesh_lods', path='artifacts/automatic-lod/corrupt.ignis-lod')
        except (AssertionError, RuntimeError):
            rejected = True
        assert rejected and c.call('inspect_mesh_lods') == before
        if compiled['output_written']:
            c.call('load_mesh_lods', path='artifacts/automatic-lod/trees.ignis-lod')
        c.call('set_mesh_lod_settings', settings=dict(enabled=True, error_px=.25, color_error=.002))
        assert c.call('inspect')['scene'] == source
        c.call('save_scene', path='artifacts/automatic-lod/trees.scene.json')
        (out/'mcp.json').write_text(json.dumps(dict(compiled=compiled, invalid_load_atomic=True, source_unchanged=True, measured_fps=False), indent=2))
        # A coplanar subdivided surface exercises an ACTUALLY selected LOD and
        # direct u16 upload without changing coverage, normals or any pixel.
        c.call('generate', recipe=dict(width=30, depth=30, seed=181, obstacles=0))
        plane = c.call('inspect')['scene']
        for entity in plane['entities']:
            entity['visible'] = False
        n = 32
        positions = [[x/n*10-5, 0, z/n*10-5] for z in range(n+1) for x in range(n+1)]
        triangles = []
        for z in range(n):
            for x in range(n):
                a = z*(n+1)+x
                triangles += [[a,a+n+1,a+1],[a+1,a+n+1,a+n+2]]
        plane['meshes'] = dict(assets=[dict(id='surface', shading='smooth', geometry=dict(positions=positions, triangles=triangles))], instances=[dict(id='surface',asset='surface',position=[0,5,0],rotation=[0,0,0,1],scale=1,color=[.4,.3,.2])])
        plane['meshes']['instances'].append(dict(plane['meshes']['instances'][0],id='surface_offscreen',position=[1000,5,0]))
        plane['camera'] = dict(position=[0,120,160],target=[0,5,0],fov_degrees=65)
        plane['light'].pop('shadow', None)
        plane['light'].pop('environment', None)
        plane['light'].pop('atmosphere', None)
        (out/'plane.scene.json').write_text(json.dumps(plane))
        c.call('load_scene',path='artifacts/automatic-lod/plane.scene.json')
        c.call('compile_mesh_lods',path='artifacts/automatic-lod/plane.ignis-lod')
        c.call('set_mesh_lod_settings',settings=dict(enabled=False,error_px=.25,color_error=.002))
        c.call('capture',path='artifacts/automatic-lod/plane-previous.png',width=1280,height=800)
        c.call('set_mesh_lod_settings',settings=dict(enabled=True,error_px=.25,color_error=.002))
        chosen=c.call('capture',path='artifacts/automatic-lod/plane-lod.png',width=1280,height=800)
        assert chosen['adapter']['mesh_lods']['selected_instances'] == 1, chosen['adapter']['mesh_lods']
        assert chosen['adapter']['mesh_lods']['selected_triangles'] < chosen['adapter']['mesh_lods']['source_triangles']
        assert (out/'plane-previous.png').read_bytes()==(out/'plane-lod.png').read_bytes()
        (out/'plane-render.json').write_text(json.dumps(chosen,indent=2))
        print('Mesh LOD MCP: atomic invalid-load, source preserved, optional settings exposed; planar image exact')
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', default=str(ROOT/'target/release/ignis'))
    main(p.parse_args().binary)
