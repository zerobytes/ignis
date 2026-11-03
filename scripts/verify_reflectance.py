#!/usr/bin/env python3
"""Optional direct dielectric response through actual native GPU/MCP paths."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_materials import f32


def verify(binary, window):
    c = Client(binary, window); checks = []
    def edit(kind, expected_error=False, **kw):
        return c.call('edit_mesh', expected_error=expected_error, edit={'kind': kind, **kw})
    def capture(name, width=900, height=750):
        c.call('capture', path=f'artifacts/reflectance-{name}.png', width=width, height=height)
        return (ROOT/f'artifacts/reflectance-{name}.png').read_bytes()
    def response(r=0.45, f0=0.04): return f32({'reflectance': {'roughness': r, 'f0': f0}})
    def set_material(m): edit('set_material', asset='surface', material=m)
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'ignis-reflectance-verifier', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        assert 'reflectance' in json.dumps(c.request('tools/list'))
        c.call('generate', recipe={'seed': 7, 'width': 16, 'depth': 14, 'obstacles': 0})
        scene = c.call('inspect')['scene']
        entities = scene['entities']
        for e in entities: e['visible'] = False
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': e} for e in entities])
        edit('grid', id='unused', segments=1, size=1)
        edit('grid', id='surface', segments=4, size=3)
        edit('set_shading', asset='surface', shading='smooth')
        instance = {'id': 'sheet', 'asset': 'surface', 'position': [0, 1.5, 0], 'rotation': [0,0,0,1], 'scale': 1, 'color': [0.12, 0.3, 0.07], 'playback': None}
        edit('put_instance', instance=instance)
        camera = {'position': [-2.5, 6.5, -1.25], 'target': [0, 1.5, 0], 'fov_degrees': 36}
        light = {'direction': [-0.5, -1, -0.25], 'ambient': 0.2}
        c.call('set_camera', camera=camera); c.call('set_light', light=light)
        neutral = capture('neutral')
        for r in [0.2, 1.0]:
            set_material(response(r, 0)); assert capture(f'zero-{r}') == neutral
        set_material({})
        before = c.call('inspect'); asset = c.call('inspect_mesh', asset='surface')['asset']; pose = c.call('inspect_mesh_pose', instance='sheet')['pose']
        material = response()
        preview = c.call('preview_mesh_edit', edit={'kind': 'set_material', 'asset': 'surface', 'material': material})
        assert c.call('inspect') == before
        set_material(material)
        assert c.call('inspect_mesh', asset='surface')['asset'] == preview['meshes']['assets'][1]
        assert c.call('inspect_mesh_pose', instance='sheet')['pose'] == pose
        assert c.call('inspect_mesh', asset='surface')['asset']['geometry'] == asset['geometry']
        glossy = capture('glossy'); assert glossy != neutral and capture('repeat') == glossy
        c.call('undo'); assert capture('undo') == neutral
        c.call('redo'); assert capture('redo') == glossy
        set_material(response(0.9)); matte = capture('matte'); assert matte != glossy and matte != neutral
        set_material(response(0.45, 0.1)); assert capture('f0') != glossy
        set_material(material); edit('remove_asset', id='unused'); assert capture('slot-reindexed') == glossy
        checks.append('Schema/preview and preserved geometry/pose; None/F0zero neutral, roughness/F0 distinct, deterministic repeat/undo/redo and slot reindex')
        before = c.call('inspect')
        for r, f0 in [(0.19, 0.04), (1.1, 0.04), (0.5, -0.01), (0.5, 0.17)]:
            edit('set_material', material=response(r, f0), asset='surface', expected_error=True); assert c.call('inspect') == before
        c.call('save_scene', path='examples/reflectance.scene.json')
        saved = c.call('inspect')['scene']; assert saved['version'] == SCENE_VERSION
        forged = copy.deepcopy(saved); forged['version'] = 59
        (ROOT/'artifacts/reflectance-old.scene.json').write_text(json.dumps(forged))
        c.call('load_scene', path='artifacts/reflectance-old.scene.json', expected_error=True); assert c.call('inspect') == before
        checks.append('Bounds and old-format capability smuggling reject before runtime/history mutation')
        for name, camera_value in [('off-axis', {**camera, 'position': [3, 6, 4]}), ('grazing', {**camera, 'position': [4, 1.501, 4]}), ('coincident-plane', {'position': [0, 1.5, 0], 'target': [0, 1.5, 1], 'fov_degrees': 36})]:
            c.call('set_camera', camera=camera_value); assert capture(name) != glossy
        c.call('set_camera', camera=camera); assert capture('camera-restored') == glossy
        c.call('set_light', light={**light, 'direction': [0.5, -1, 0.25]}); assert capture('light-moved') != glossy
        c.call('set_light', light={**light, 'direction': [0, 1, 0]})
        back = capture('back'); set_material({}); assert capture('back-neutral') == back
        set_material(material)
        c.call('set_light', light={**light, 'ambient': 1}); ambient = capture('ambient')
        set_material({}); assert capture('ambient-neutral') == ambient
        set_material(material); c.call('set_light', light=light)
        patterned = {**material, 'pattern': {'frequency': [3,1,3], 'low': [0.8]*3, 'high': [1.2]*3, 'bump': 0.004, 'warp': 0.2, 'octaves': 2, 'seed': 11, 'contrast': 1}}
        set_material(patterned); bumped = capture('bump'); assert bumped != glossy
        set_material({**patterned, 'reflectance': {'roughness': 0.9, 'f0': 0.04}}); assert capture('bump-matte') != bumped
        set_material(material)
        checks.append('Camera/light angle and bump affect highlight, grazing/coincident-plane render safely; no direct lobe in backlight or all-ambient')
        # A caster covers the entire sheet along the directional light. Ambient
        # survives but direct diffuse/specular must both vanish in shadow.
        box = copy.deepcopy(next(e for e in entities if e['id']=='wall_north'))
        box.update(id='cover', visible=True, color=[0.3]*3)
        box['transform'] = {'position': [3, 7.5, 1.5], 'size': [4.5, 0.3, 4.5]}
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': box}])
        c.call('set_light', light={**light, 'shadow': {'center': [1,4,0.5], 'extent': 8, 'resolution': 2048, 'bias': 0.0001}})
        shadow = capture('shadow'); assert shadow != glossy
        set_material({}); assert capture('shadow-neutral') == shadow
        set_material(material); c.call('remove', id='cover'); c.call('set_light', light=light)
        assert capture('uncovered') == glossy
        count = len(asset['geometry']['positions'])
        edit('set_rig', asset='surface', rig={'bones': [{'name': 'root', 'parent': None, 'translation': [0,0,0], 'rotation': [0,0,0,1]}], 'weights': [{'joints': [0]*4, 'weights': [1,0,0,0]} for _ in range(count)]})
        edit('put_clip', asset='surface', clip={'id': 'move', 'duration_ticks': 120, 'tracks': [{'bone': 0, 'keys': [{'tick': t, 'translation': [0,y,0], 'rotation': [0,0,0,1]} for t,y in [(0,0),(60,0.5),(120,0)]]}]})
        edit('put_instance', instance=dict(instance, position=[0,1,0], playback={'clip': 'move', 'offset_ticks': 60, 'looping': True}))
        assert capture('rig-compensated') == glossy
        edit('put_instance', instance=dict(instance, playback={'clip': 'move', 'offset_ticks': 0, 'looping': True}))
        assert capture('rig-restored') == glossy
        checks.append('Full directional shadow disables front specular and diffuse together; compensated rig retains exact world-space highlight')
        c.call('save_scene', path='examples/reflectance.scene.json'); scene = c.call('inspect')['scene']
        start = capture('start'); c.call('step', ticks=60); live = capture('live'); assert live != start
        c.call('save_replay', path='artifacts/reflectance.replay.json')
        assert c.call('run_replay', path='artifacts/reflectance.replay.json')['meshes_matched']
        for name, tick, expected in [('replay', None, live), ('prefix', 0, start)]:
            c.call('capture_replay', replay_path='artifacts/reflectance.replay.json', path=f'artifacts/reflectance-{name}.png', tick=tick, width=900, height=750)
            assert (ROOT/f'artifacts/reflectance-{name}.png').read_bytes() == expected
        c.call('load_scene', path='examples/reflectance.scene.json'); assert c.call('inspect')['scene'] == scene
        c.call('reset'); assert capture('reset') == start
        capture('viewport', 640, 480); assert capture('viewport-restored') == start
        checks.append('Scene67 load/reset and Replay71 live/prefix pixels exact, animated highlight and viewport/cache restoration')
        report = {'passed': True, 'checks': checks, 'build_id': c.call('inspect')['build_id'], 'native_window': window}
        (ROOT/'artifacts/reflectance-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'checks': len(checks), 'build_id': report['build_id']}))
    finally: c.close()


if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,default=ROOT/'target/release/ignis');p.add_argument('--window',action='store_true');a=p.parse_args();verify(a.binary.resolve(),a.window)
