#!/usr/bin/env python3
"""Terrain authoring, native publication and conservative coverage through real MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def position(x, z):
    cx, cz = int(x // 1024), int(z // 1024)
    return {'cell': [str(cx), '0', str(cz)], 'offset': [x-cx*1024, 0, z-cz*1024]}


def verify(binary, window=False):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'terrain-tools', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result'])
        assert all(name in schema for name in ['query_terrain', 'plan_terrain_coverage', 'publish_terrain', 'save_terrain', 'load_terrain'])
        recipe = json.loads((ROOT / 'examples/terrain-tools.recipe.json').read_text())
        doc = recipe['document']
        before = c.call('inspect')
        inspected = c.call('query_terrain', query={'kind': 'inspect', 'document': doc})
        assert inspected['document'] == doc and not inspected['collision']
        seam = [position(0, z) for z in range(-32, 33)]
        samples = c.call('query_terrain', query={'kind': 'sample', 'document': doc, 'points': seam})
        full = [c.call('query_terrain', query={'kind': 'bake', 'document': doc, 'patch': patch})['document'] for patch in recipe['patches']]
        for z in range(65):
            assert full[0]['base']['heights'][z*65+64] == full[1]['base']['heights'][z*65]
        # A fixed map is ordinary editable data, and never invents terrain outside it.
        fixed = full[0]
        fixed_before = copy.deepcopy(fixed)
        changed = c.call('query_terrain', query={'kind': 'edit', 'document': fixed, 'expected_revision': fixed['revision'], 'edits': [{'kind': 'patch_samples', 'start': [0, 0], 'size': [1, 1], 'heights': [12]}]})['document']
        assert changed['revision'] == fixed['revision']+1 and changed['base']['heights'][0] == 12
        missing = c.call('query_terrain', query={'kind': 'sample', 'document': fixed, 'points': [position(-65, 0)]})
        assert missing['samples'][0]['height_meters'] is None
        c.call('query_terrain', query={'kind': 'edit', 'document': fixed, 'expected_revision': fixed['revision'], 'edits': [{'kind': 'set_water', 'height_meters': 2}, {'kind': 'remove', 'index': 63}]}, expected_error=True)
        assert fixed == fixed_before
        c.call('save_terrain', path='examples/terrain-tools.ignis-terrain', document=doc)
        assert c.call('load_terrain', path='examples/terrain-tools.ignis-terrain')['document'] == doc
        saved = (ROOT / 'examples/terrain-tools.ignis-terrain').read_bytes()
        broken = bytearray(saved)
        broken[-1] ^= 1
        corrupt = ROOT / 'artifacts/terrain-corrupt.ignis-terrain'
        corrupt.write_bytes(broken)
        c.call('load_terrain', path='artifacts/terrain-corrupt.ignis-terrain', expected_error=True)
        corrupt.unlink()
        assert c.call('inspect') == before
        checks.append('Pure shared kernel, exact neighboring sampled borders, fixed-map edits/domain, late failure, binary source roundtrip and checksum rejection preserve live state')

        nodes = [{'id': 'parent', 'region': {'origin': position(-16, -16), 'size_meters': 32}, 'center': position(0, 0), 'radius_meters': 100, 'children': ['a', 'b', 'c', 'd'], 'ready': True, 'children_seams_ready': True, 'approximation_error_meters': 10}]
        nodes += [{'id': name, 'region': {'origin': position(-16+16*(i%2), -16+16*(i//2)), 'size_meters': 16}, 'center': position(-8+16*(i%2), -8+16*(i//2)), 'radius_meters': 12, 'children': None, 'ready': name != 'd', 'children_seams_ready': False, 'approximation_error_meters': 0} for i, name in enumerate(['a', 'b', 'c', 'd'])]
        query = {'viewer': position(0, 0), 'atmosphere': {'kind': 'disabled'}, 'maximum_error_meters': 1, 'representation_budget': 4, 'nodes': nodes}
        plan = c.call('plan_terrain_coverage', query=query)
        assert plan['selected'] == ['parent'] and plan['requests'] == ['d'] and plan['proposals_only'] and plan['renderer_binding_required']
        nodes[4]['ready'] = True
        assert c.call('plan_terrain_coverage', query=query)['selected'] == ['a', 'b', 'c', 'd']
        query['representation_budget'] = 1
        assert c.call('plan_terrain_coverage', query=query)['selected'] == ['parent']
        query['viewer'] = position(1000, 0)
        query['atmosphere'] = {'kind': 'uniform', 'extinction_per_meter': .01, 'maximum_contrast': 1, 'negligible_contrast': .01}
        assert c.call('plan_terrain_coverage', query=query)['omitted_by_atmosphere_model'] == ['parent']
        query['atmosphere']['extinction_per_meter'] = 0
        assert c.call('plan_terrain_coverage', query=query)['selected'] == ['parent']
        assert c.call('inspect') == before
        checks.append('Declared-coverage planner retains parent through pending child/budget, requires seam readiness, and clear air has no finite horizon; no renderer atmosphere or streaming claimed')

        c.call('generate', recipe={'width': 12, 'depth': 12, 'seed': 731, 'obstacles': 0})
        entities = c.call('inspect')['scene']['entities']
        c.call('edit_scene', edits=[{'op': 'remove', 'id': e['id']} for e in entities])
        c.call('set_camera', camera=recipe['camera'])
        native_before = c.call('inspect')['scene']
        for i, patch in enumerate(recipe['patches']):
            publication = {'document': doc, 'patch': patch, 'stride': 1, 'asset': f'terrain_{i}', 'instance': f'terrain_{i}', 'frame_origin': recipe['frame_origin'], 'color': [.36, .53, .29]}
            if i == 0:
                bad = copy.deepcopy(publication)
                bad['instance'] = ''  # late failure after a valid candidate asset
                c.call('publish_terrain', publication=bad, expected_error=True)
                assert c.call('inspect')['scene'] == native_before
            result = c.call('publish_terrain', publication=publication)
            assert result['visual_only'] and not result['source_embedded_in_scene'] and result['sampled_error_meters'] == 0
            asset = c.call('inspect_mesh', asset=f'terrain_{i}')['asset']
            assert len(asset['geometry']['positions']) == 4225 and len(asset['geometry']['triangles']) == 8192
            if i == 0:
                after = c.call('inspect')['scene']
                c.call('undo')
                assert c.call('inspect')['scene'] == native_before
                c.call('redo')
                assert c.call('inspect')['scene'] == after
        # Static geographic water level for visualization only, not a water system.
        water = {'id': 'water_level', 'geometry': {'positions': [[-64, 0, -32], [64, 0, -32], [-64, 0, 32], [64, 0, 32]], 'triangles': [[0, 2, 1], [1, 2, 3]]}, 'shading': 'flat'}
        instance = {'id': 'water_level', 'asset': 'water_level', 'position': [0, 0, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [.12, .35, .51]}
        c.call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'put_asset', 'asset': water}}, {'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': instance}}])
        c.call('save_scene', path='artifacts/terrain-tools.scene.json')
        scene = c.call('inspect')['scene']
        def capture(suffix):
            path = f'artifacts/terrain-{suffix}.png'
            c.call('capture', path=path, width=960, height=640)
            return (ROOT / path).read_bytes()
        pixels = capture('tools')
        c.call('step', ticks=2)
        c.call('save_replay', path='artifacts/terrain.replay.json')
        assert c.call('run_replay', path='artifacts/terrain.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/terrain.replay.json', path='artifacts/terrain-replay.png', tick=2, width=960, height=640)
        assert (ROOT / 'artifacts/terrain-replay.png').read_bytes() == pixels
        c.call('load_scene', path='artifacts/terrain-tools.scene.json')
        assert c.call('inspect')['scene'] == scene and capture('loaded') == pixels
        checks.append('Native publication rejects late error atomically; 2 neighboring meshes, one undo per publication, scene source retained separately, save/load/replay and offscreen pixels exact')
        report = {'passed': True, 'build_id': c.call('inspect')['build_id'], 'native_window': window, 'checks': checks, 'scope': {'terrain_physics': False, 'streaming': False, 'nanites_integrated': False, 'atmospheric_renderer': False, 'water_fluid': False}, 'seam_samples': len(samples['samples'])}
        (ROOT / 'artifacts/terrain-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    p.add_argument('--window', action='store_true')
    a = p.parse_args()
    verify(a.binary.resolve(), a.window)
