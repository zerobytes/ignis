"""Bounded topological modeling through the public shared MCP dispatcher."""
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    if window:
        raise ValueError('This regression is headless; native interaction has a separate probe')
    c = Client(binary, False)
    checks = []
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='mesh-modeling', version='1')))
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result']['tools'][0]['inputSchema'])
        assert all(name in schema for name in ['model_editor_mesh', 'model_selection', 'extrude', 'inset', 'bevel', 'extend_collinear'])
        c.call('generate', recipe=dict(width=16, depth=14, obstacles=0, seed=42))
        c.call('create_editor_mesh', primitive='cube', resolution=4, size=2)
        instance = c.call('inspect_editor')['selected_mesh']

        def asset():
            return c.call('inspect_mesh', asset=instance)['asset']

        def ref():
            return c.call('inspect_mesh_topology', asset=instance)['reference']

        def top(a):
            return [i for i, f in enumerate(a['geometry']['triangles']) if all(a['geometry']['positions'][j][1] > .999 for j in f)]

        def request(operation, preview=False, reference=None, **extra):
            return c.call('model_editor_mesh', instance=instance, operation=operation, reference=reference or ref(), preview=preview, **extra)

        original = c.call('inspect')
        faces = top(asset())
        operation = dict(kind='inset', faces=faces, width=.2)
        reference = ref()
        preview = request(operation, preview=True)
        assert c.call('inspect') == original
        result = request(operation)
        assert c.call('inspect')['scene'] == preview['scene']
        assert result['selection']['components'] == dict(kind='faces', indices=result['modeling_report']['selected_faces'])
        assert result['selection']['reference'] != reference
        cap = result['modeling_report']['selected_faces']
        operation = dict(kind='extrude', faces=cap, offset=[0, .3, 0])
        preview = request(operation, preview=True)
        result = request(operation)
        final = c.call('inspect')
        assert final['scene'] == preview['scene']
        assert final['history']['undo'] == original['history']['undo'] + 2
        t = c.call('inspect_mesh_topology', asset=instance)['topology']
        assert t['boundary_edges'] == 0 and t['non_manifold_edges'] == 0
        c.call('undo'); c.call('undo')
        assert c.call('inspect')['scene'] == original['scene']
        c.call('redo'); c.call('redo')
        assert c.call('inspect')['scene'] == final['scene']
        checks.append('Inset followed by region extrusion matches pure preview, keeps closed topology, selects fresh caps and creates one undo per operation')

        frozen = c.call('inspect')
        request(operation, reference=reference, expected_error=True)
        for invalid in [dict(kind='inset', faces=faces, width=0), dict(kind='extrude', faces=[999999], offset=[0,.1,0]), dict(kind='extrude', faces=cap, offset=[0,0,0]), dict(kind='bevel', edges=[[0,0]], width=.1, extend_collinear=True)]:
            request(invalid, expected_error=True)
            assert c.call('inspect') == frozen
        c.call('play')
        frozen = c.call('inspect')
        request(operation, expected_error=True)
        c.call('edit_mesh', edit=dict(kind='model_selection', asset=instance, reference=ref(), operation=operation), expected_error=True)
        assert c.call('inspect') == frozen
        c.call('pause')
        c.call('undo'); c.call('undo')
        checks.append('Invalid regions/widths/edges, stale references and editing during play preserve scene, runtime and history')

        a = asset()
        edge = next(e['vertices'] for e in c.call('inspect_mesh_topology', asset=instance)['topology']['edges'] if all(a['geometry']['positions'][j][0] > .999 and a['geometry']['positions'][j][1] > .999 for j in e['vertices']))
        operation = dict(kind='bevel', edges=[edge], width=.2, extend_collinear=False)
        frozen = c.call('inspect')
        request(operation, expected_error=True)
        assert c.call('inspect') == frozen
        operation['extend_collinear'] = True
        edit = dict(kind='model_selection', asset=instance, reference=ref(), operation=operation)
        preview = c.call('preview_mesh_edit', edit=edit)
        assert c.call('inspect') == frozen
        c.call('edit_scene', edits=[dict(op='mesh', edit=edit), dict(op='mesh', edit=edit)], expected_error=True)
        assert c.call('inspect') == frozen
        result = c.call('edit_mesh', edit=edit)
        assert result['modeling_report'] == preview['modeling_report']
        assert c.call('inspect')['scene']['meshes'] == preview['meshes']
        assert len(result['modeling_report']['effective_edges']) == 4
        assert c.call('inspect_mesh_topology', asset=instance)['topology']['boundary_edges'] == 0
        c.call('undo')
        assert c.call('inspect')['scene'] == frozen['scene']
        request(operation)
        checks.append('Generic EditMesh and native modeling share reports and geometry; partial bevel chains require explicit expansion and all intersections remain closed')

        # A linked procedural source refuses topology changes until explicitly detached.
        definition = dict(id='modeling-limb', profile=dict(id='outline', width=2, height=4), depth=.5, segments=[1,1,1], sampling_tolerance=.0009765625)
        c.call('edit_plant', edit=dict(kind='put', definition=definition, detail_policy='physical', animation_translation_policy='physical'))
        selection = [0]
        edit = dict(kind='model_selection', asset='modeling-limb', reference=c.call('inspect_mesh_topology', asset='modeling-limb')['reference'], operation=dict(kind='extrude', faces=selection, offset=[0,0,.1]))
        frozen = c.call('inspect')
        c.call('preview_mesh_edit', edit=edit, expected_error=True)
        c.call('edit_mesh', edit=edit, expected_error=True)
        assert c.call('inspect') == frozen
        c.call('edit_plant', edit=dict(kind='detach', id='modeling-limb'))
        plant = c.call('inspect_mesh', asset='modeling-limb')['asset']
        p,q,r = [plant['geometry']['positions'][j] for j in plant['geometry']['triangles'][0]]
        u=[q[i]-p[i] for i in range(3)]; v=[r[i]-p[i] for i in range(3)]
        n=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
        length=sum(x*x for x in n)**.5
        edit['operation']['offset']=[x/length*.1 for x in n]
        edit['reference']=c.call('inspect_mesh_topology', asset='modeling-limb')['reference']
        c.call('edit_mesh', edit=edit)
        checks.append('Procedural sources reject topology changes atomically; explicit detach enables the same shared operation')

        c.call('set_camera', camera=dict(position=[6,5,7], target=[2,1.05,0], fov_degrees=42))
        c.call('save_scene', path='examples/mesh-modeling.scene.json')
        authored = c.call('inspect')['scene']
        c.call('capture', path='artifacts/mesh-modeling-start.png', width=960, height=720)
        c.call('step', ticks=3)
        c.call('capture', path='artifacts/mesh-modeling-current.png', width=960, height=720)
        pixels = (ROOT/'artifacts/mesh-modeling-current.png').read_bytes()
        c.call('capture', path='artifacts/mesh-modeling-current.png', width=960, height=720)
        assert pixels == (ROOT/'artifacts/mesh-modeling-current.png').read_bytes()
        c.call('save_replay', path='artifacts/mesh-modeling.replay.json')
        replay = c.call('run_replay', path='artifacts/mesh-modeling.replay.json')
        assert replay['matched'] and replay['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/mesh-modeling.replay.json', path='artifacts/mesh-modeling-replay.png', width=960, height=720)
        assert pixels == (ROOT/'artifacts/mesh-modeling-replay.png').read_bytes()
        c.call('capture_replay', replay_path='artifacts/mesh-modeling.replay.json', path='artifacts/mesh-modeling-prefix.png', tick=0, width=960, height=720)
        assert (ROOT/'artifacts/mesh-modeling-prefix.png').read_bytes() == (ROOT/'artifacts/mesh-modeling-start.png').read_bytes()
        c.call('load_scene', path='examples/mesh-modeling.scene.json')
        assert c.call('inspect')['scene'] == authored
        c.call('reset')
        assert c.call('inspect')['scene'] == authored
        c.call('capture_editor', path='artifacts/mesh-modeling-editor.png', width=1440, height=900)
        checks.append('Authored topology survives save/load/reset/replay, exact live/replay/prefix PNG and offscreen editor capture')
        result = dict(passed=True, build_id=c.call('inspect')['build_id'], checks=checks, native_window=False)
        (ROOT/'artifacts/mesh-modeling-verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({**result, 'checks':len(checks)}))
    finally:
        c.close()


if __name__ == '__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, default=Path(__file__).resolve().parents[1]/'target/release/ignis')
    parser.add_argument('--window', action='store_true')
    args=parser.parse_args()
    verify(args.binary,args.window)
