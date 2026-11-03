#!/usr/bin/env python3
"""Critical large-document MCP admission/save/load and native capture boundary."""
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary):
    out = ROOT / 'artifacts/mesh-resources'
    out.mkdir(parents=True, exist_ok=True)
    client = Client(binary)
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                      'clientInfo': {'name': 'mesh-resources', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        client.call('generate', recipe={'seed': 99, 'width': 20, 'depth': 20, 'obstacles': 0})
        client.call('edit_mesh', edit={'kind': 'grid', 'id': 'tile', 'segments': 8, 'size': 1.5})
        scene = client.call('inspect')['scene']
        for entity in scene['entities']:
            entity['visible'] = False
        scene['camera'] = dict(position=[100, 180, 210], target=[100, 0, 100], fov_degrees=65)
        scene['meshes']['instances'] = [dict(id=f'tile-{n}', asset='tile',
            position=[n % 100 * 2, n % 5 * 0.25, n // 100 * 2], rotation=[0, 0, 0, 1],
            scale=1, color=[0.25, 0.6, 0.15], playback=None) for n in range(10000)]
        path = 'artifacts/mesh-resources/population.scene.json'
        (ROOT / path).write_text(json.dumps(scene))
        client.call('load_scene', path=path)
        state = client.call('inspect')['scene']
        limits = client.call('inspect_meshes')['limits']
        assert limits['instances'] is None and limits['assets'] is None
        assert len(state['meshes']['instances']) == 10000
        first = client.call('capture', path='artifacts/mesh-resources/population.png', width=1728, height=1117)
        assert first['mesh_triangle_count'] == 1280000
        assert first['mesh_resident_vertex_count'] == 384
        assert first['adapter']['instancing']['instances'] == 10000
        pixels = (out / 'population.png').read_bytes()
        client.call('save_scene', path=path)
        client.call('load_scene', path=path)
        assert client.call('inspect')['scene'] == state
        client.call('capture', path='artifacts/mesh-resources/population.png', width=1728, height=1117)
        assert pixels == (out / 'population.png').read_bytes()
        result = dict(passed=True, build_id=client.call('inspect')['build_id'], instances=10000,
                      triangles=1280000, resident_vertices=384, capture=first,
                      checks=['10000 instances admitted by MCP with no quantity ceiling',
                              'save/load preserves all authored identities and native pixels'])
        (out / 'mcp-verification.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps({k: result[k] for k in ['passed', 'build_id', 'instances', 'triangles']}))
    finally:
        client.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    verify(parser.parse_args().binary)
