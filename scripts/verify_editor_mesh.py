#!/usr/bin/env python3
"""Real MCP mesh creation, surface queries, sculpting and editor persistence."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    if window:
        raise ValueError('Native brush input has a separate gate')
    c = Client(binary, False)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'editor-mesh-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list'))
        assert all(op in schema for op in ['create_editor_mesh', 'select_editor_mesh', 'pick_editor_mesh'])
        c.call('generate', recipe={'width': 16, 'depth': 14, 'seed': 42, 'obstacles': 0})
        for primitive in ['sphere', 'cube', 'cylinder', 'plane']:
            result = c.call('create_editor_mesh', primitive=primitive, resolution=16, size=2)
            assert result['selected_mesh']
        state = c.call('inspect')
        assert len(state['scene']['meshes']['assets']) == 4
        original = copy.deepcopy(state['scene'])
        for op, args in [('create_editor_mesh', {'primitive': 'sphere', 'resolution': 64, 'size': 2}), ('select_editor_mesh', {'instance': 'missing'}), ('pick_editor_mesh', {'x': -1, 'y': 0, 'width': 800, 'height': 600})]:
            c.call(op, expected_error=True, **args)
            assert c.call('inspect') == state
        # Move other primitives away so the surface query has one visible candidate.
        edits = []
        for i, instance in enumerate(original['meshes']['instances']):
            if i:
                instance['position'][0] += i * 4
                edits.append({'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': instance}})
        c.call('edit_scene', edits=edits)
        c.call('select_editor_mesh', instance='mesh_0')
        camera = copy.deepcopy(c.call('inspect')['scene']['camera'])
        camera.update(position=[2, 1.05, 6], target=[2, 1.05, 0])
        c.call('edit_editor', action={'action': 'camera', 'camera': camera})
        before = c.call('inspect')
        hit = c.call('pick_editor_mesh', x=400, y=300, width=800, height=600)['hit']
        assert hit['instance'] == 'mesh_0' and abs(hit['position'][2]-1) < 0.001
        assert c.call('inspect') == before
        checks.append('MCP primitive creation, real nearest triangle surface query, session selection and atomic errors')
        for brush in ['draw', 'grab', 'smooth']:
            before = c.call('inspect')
            stroke = {'kind': brush, 'centers': [hit['local_position']], 'radius': 0.7, 'strength': 0.08, 'direction': [0, 1, 0]}
            edit = {'kind': 'sculpt', 'asset': 'mesh_0', 'stroke': stroke}
            preview = c.call('preview_mesh_edit', edit=edit)['meshes']
            assert c.call('inspect') == before
            c.call('edit_mesh', edit=edit)
            assert c.call('inspect')['scene']['meshes'] == preview
            c.call('undo')
            assert c.call('inspect')['scene'] == before['scene']
            c.call('redo')
            assert c.call('inspect')['scene']['meshes'] == preview
            c.call('undo')
        checks.append('Three sculpt kernels match preview/commit with one undo/redo per stroke')
        # Same scoped detailed operator used by the native Volume brush. Exercise
        # real transport, topology/material persistence and one undo transaction.
        before = c.call('inspect')
        ref = c.call('inspect_mesh_topology', asset='mesh_0')['reference']
        detailed = {'kind': 'sculpt_detailed', 'asset': 'mesh_0', 'reference': ref,
            'max_edge': 0.7 / 8, 'stroke': {'kind': 'draw', 'centers': [hit['local_position']],
                'radius': 0.7, 'strength': 0.08, 'direction': [0, 0, 1]}}
        preview = c.call('preview_mesh_edit', edit=detailed)['meshes']
        assert c.call('inspect') == before
        assert len(preview['assets'][0]['geometry']['positions']) > len(before['scene']['meshes']['assets'][0]['geometry']['positions'])
        c.call('edit_mesh', edit=detailed)
        refined = c.call('inspect')
        assert refined['scene']['meshes'] == preview
        assert refined['history']['undo'] == before['history']['undo'] + 1
        c.call('save_scene', path='artifacts/editor-detail.scene.json')
        c.call('undo')
        assert c.call('inspect')['scene'] == before['scene']
        c.call('redo')
        assert c.call('inspect')['scene']['meshes'] == preview
        c.call('load_scene', path='artifacts/editor-detail.scene.json')
        assert c.call('inspect')['scene']['meshes'] == preview
        c.call('step', ticks=2)
        c.call('save_replay', path='artifacts/editor-detail.replay.json')
        assert c.call('run_replay', path='artifacts/editor-detail.replay.json')['matched']
        stale = c.call('inspect')
        c.call('edit_mesh', edit=detailed, expected_error=True)
        assert c.call('inspect') == stale
        # Restore the source through the same authored document, without retaining
        # detailed state in the independent lighting/settings checks below.
        c.call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'put_asset', 'asset': asset}}
            for asset in before['scene']['meshes']['assets']])
        c.call('select_editor_mesh', instance='mesh_0')
        checks.append('Detailed sculpt transport: pure preview, one undo/redo, stale refusal, save/load and replay')
        for kind in ['draw', 'grab']:
            before = c.call('inspect')
            ref = c.call('inspect_mesh_topology', asset='mesh_0')['reference']
            interactive = {'kind': 'sculpt_stroke', 'asset': 'mesh_0', 'reference': ref,
                'adaptive_detail': kind == 'draw', 'stroke': {'kind': kind,
                    'centers': [hit['local_position']], 'radius': 0.7, 'strength': 0.6,
                    'direction': [1, 0, 0]}}
            preview = c.call('preview_mesh_edit', edit=interactive)['meshes']
            assert c.call('inspect') == before
            c.call('edit_mesh', edit=interactive)
            assert c.call('inspect')['scene']['meshes'] == preview
            assert c.call('inspect')['history']['undo'] == before['history']['undo'] + 1
            c.call('save_scene', path='artifacts/editor-interactive.scene.json')
            c.call('undo')
            assert c.call('inspect')['scene'] == before['scene']
            c.call('redo')
            assert c.call('inspect')['scene']['meshes'] == preview
            c.call('load_scene', path='artifacts/editor-interactive.scene.json')
            assert c.call('inspect')['scene']['meshes'] == preview
            c.call('step', ticks=2)
            c.call('save_replay', path='artifacts/editor-interactive.replay.json')
            assert c.call('run_replay', path='artifacts/editor-interactive.replay.json')['matched']
            c.call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'put_asset', 'asset': asset}}
                for asset in before['scene']['meshes']['assets']])
            c.call('select_editor_mesh', instance='mesh_0')
        checks.append('Interactive Volume/Grab: shared pure preview, one undo, scene roundtrip and replay')
        # Brush configuration and lighting are editor state; the exact same
        # typed commands are used by native controls and MCP.
        c.call('save_scene', path='artifacts/editor-mesh.scene.json')
        world = c.call('inspect')
        editor = c.call('inspect_editor')['editor']
        light = copy.deepcopy(editor['work_light'])
        settings = copy.deepcopy(editor['sculpt'])
        settings.update(radius=0.65, draw_strength=0.6, draw_strength_is_relative=True, grab_strength=0.4, smooth_strength=0.3)
        c.call('edit_editor', action={'action': 'sculpt', 'settings': settings})
        settings = c.call('inspect_editor')['editor']['sculpt']
        c.call('save_editor', path='artifacts/editor-brush.layout.json')
        c.call('edit_editor', action={'action': 'lighting', 'light': None})
        c.call('load_editor', path='artifacts/editor-brush.layout.json')
        loaded = c.call('inspect_editor')['editor']
        assert loaded['sculpt'] == settings and loaded['work_light'] == light
        for action in [
            {'action': 'sculpt', 'settings': dict(settings, radius=0)},
            {'action': 'sculpt', 'settings': dict(settings, draw_strength=1.1)},
            {'action': 'lighting', 'light': dict(light, ambient=-0.1)},
            {'action': 'lighting', 'light': dict(light, direction=[0, 0, 0])},
            {'action': 'lighting', 'light': dict(light, shadow=dict(light['shadow'], resolution=4096))},
        ]:
            before_editor = c.call('inspect_editor')
            c.call('edit_editor', action=action, expected_error=True)
            assert c.call('inspect_editor') == before_editor
        assert c.call('inspect') == world
        checks.append('Shared per-brush settings and work light persist, reject invalid edits atomically and preserve the world')

        # Compare the viewport itself, excluding status/revision text. Confirm
        # normals, directional light and actual shadow toggles affect pixels.
        def capture(name):
            path = 'artifacts/editor-light-' + name + '.png'
            c.call('capture_editor', path=path, width=1440, height=900)
            return path
        def difference(a, b):
            return c.call('compare_images', reference_path=a, candidate_path=b,
                regions=[{'id': 'surface', 'x': 50, 'y': 250, 'width': 900, 'height': 500}])['report']['regions'][0]['metrics']
        plain = capture('base')
        c.call('edit_mesh', edit={'kind': 'sculpt', 'asset': 'mesh_0', 'stroke': {
            'kind': 'draw', 'centers': [hit['local_position']], 'radius': 0.65,
            'strength': 0.2, 'direction': [0, 1, 0]}})
        sculpted = capture('sculpted')
        assert difference(plain, sculpted)['changed_pixels'] > 100
        c.call('edit_editor', action={'action': 'lighting', 'light': dict(light, direction=[-0.7, -0.3, -1])})
        turned = capture('turned')
        assert difference(sculpted, turned)['changed_pixels'] > 1000
        c.call('edit_editor', action={'action': 'lighting', 'light': dict(light, shadow=None)})
        no_shadow = capture('no-shadow')
        assert difference(sculpted, no_shadow)['changed_pixels'] > 100
        c.call('edit_editor', action={'action': 'lighting', 'light': dict(light, ambient=0.7)})
        ambient = capture('ambient')
        assert difference(sculpted, ambient)['changed_pixels'] > 1000
        c.call('edit_editor', action={'action': 'lighting', 'light': light})
        restored = capture('restored')
        assert difference(sculpted, restored)['exact_rgb_match']
        c.call('undo')
        assert c.call('inspect')['scene'] == world['scene']
        c.call('edit_editor', action={'action': 'lighting', 'light': None})
        capture('scene')
        c.call('edit_editor', action={'action': 'lighting', 'light': light})
        checks.append('GPU viewport deformation, direction, ambient and shadows are visible and repeat exactly after restore')
        c.call('save_scene', path='artifacts/editor-mesh.scene.json')
        c.call('capture_editor', path='artifacts/editor-mesh.png', width=1440, height=900)
        pixels = (ROOT/'artifacts/editor-mesh.png').read_bytes()
        c.call('capture_editor', path='artifacts/editor-mesh-repeat.png', width=1440, height=900)
        assert (ROOT/'artifacts/editor-mesh-repeat.png').read_bytes() == pixels
        c.call('save_scene', path='artifacts/editor-mesh.scene.json')
        saved = c.call('inspect')['scene']
        c.call('load_scene', path='artifacts/editor-mesh.scene.json')
        assert c.call('inspect')['scene'] == saved
        assert c.call('inspect_editor')['selected_mesh'] is None
        c.call('step', ticks=10)
        c.call('save_replay', path='artifacts/editor-mesh.replay.json')
        assert c.call('run_replay', path='artifacts/editor-mesh.replay.json')['matched']
        checks.append('Real editor GPU composition, repeated pixels, save/load selection isolation and gameplay replay')
        report = {'passed': True, 'checks': checks, 'native_window': False, 'build_id': c.call('inspect')['build_id']}
        (ROOT/'artifacts/editor-mesh-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    a = p.parse_args()
    verify(a.binary.resolve(), a.window)
