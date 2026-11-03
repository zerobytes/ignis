#!/usr/bin/env python3
"""Critical MCP boundary: atomic visual membership and authoring roundtrip."""
import argparse
import json
from verify_lab import Client, ROOT


def main(binary):
    out = ROOT/'artifacts/object-culling'
    out.mkdir(parents=True, exist_ok=True)
    c = Client(binary, log_path=out/'mcp.log')
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                    clientInfo=dict(name='mesh-objects', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe=dict(width=30, depth=30, seed=181, obstacles=0))
        source = c.call('inspect')['scene']
        source['meshes'] = dict(assets=[dict(id='part', geometry=dict(
            positions=[[-1,0,0],[1,0,0],[0,2,0]], triangles=[[0,1,2]]))],
            instances=[dict(id=id, asset='part', position=[0,2,0], rotation=[0,0,0,1],
                            scale=1, color=[.2,.6,.1]) for id in ['stem','crown']])
        (out/'mcp.scene.json').write_text(json.dumps(source))
        c.call('load_scene', path='artifacts/object-culling/mcp.scene.json')
        source = c.call('inspect')['scene']
        c.call('edit_mesh', edit=dict(kind='put_object', object=dict(id='assembly',members=['stem','crown'])))
        inspection = c.call('inspect_mesh_objects', object='assembly')
        assert inspection['objects'] == 1 and inspection['grouped_instances'] == 2
        before = c.call('inspect')
        c.call('edit_mesh', expected_error=True,
               edit=dict(kind='put_object', object=dict(id='invalid',members=['stem'])))
        assert c.call('inspect') == before
        grouped = before['scene']
        assert grouped['meshes']['assets'] == source['meshes']['assets']
        assert grouped['meshes']['instances'] == source['meshes']['instances']
        c.call('undo')
        assert c.call('inspect')['scene'] == source
        c.call('redo')
        assert c.call('inspect')['scene'] == grouped
        c.call('save_scene', path='artifacts/object-culling/mcp.scene.json')
        c.call('load_scene', path='artifacts/object-culling/mcp.scene.json')
        assert c.call('inspect')['scene'] == grouped
        c.call('edit_mesh', edit=dict(kind='remove_object', id='assembly'))
        assert c.call('inspect')['scene'] == source
        report = dict(inspection=inspection, atomic_rejection=True, undo_redo=True,
                      scene_roundtrip=True, remove_preserves_geometry=True,
                      shared_dispatcher=True, ui_human_acceptance=False)
        (out/'mcp.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', default=str(ROOT/'target/release/ignis'))
    main(p.parse_args().binary)
