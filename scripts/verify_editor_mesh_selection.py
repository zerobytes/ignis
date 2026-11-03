"""Visible-surface/X-ray rectangle selection through the public session commands."""
import argparse, copy, json
from pathlib import Path
from verify_lab import Client, ROOT

def verify(binary, window):
    if window:
        raise ValueError('Native window acceptance is a separate gate')
    c = Client(binary, False)
    checks = []
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='editor-mesh-selection', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe=dict(width=16, depth=14, seed=42, obstacles=0))
        c.call('create_editor_mesh', primitive='sphere', resolution=24, size=2)
        instance = c.call('inspect_editor')['selected_mesh']
        source = c.call('inspect')
        camera = copy.deepcopy(source['scene']['camera'])
        camera.update(position=[2, 1.05, 6], target=[2, 1.05, 0])
        c.call('edit_editor', action=dict(action='camera', camera=camera))
        asset = c.call('inspect_mesh', asset=instance)['asset']
        def select(component='vertices', minimum=(0, 0), maximum=(800, 600), additive=False):
            return c.call('select_editor_mesh_region', instance=instance,
                          reference=c.call('inspect_mesh_topology', asset=instance)['reference'],
                          component=component, min=list(minimum), max=list(maximum), width=800, height=600, additive=additive)['selection']
        front = select()
        front_indices = front['components']['indices']
        assert 100 < len(front_indices) < len(asset['geometry']['positions'])
        assert all(asset['geometry']['positions'][i][2] > 0 for i in front_indices)
        assert c.call('inspect')['scene'] == source['scene']
        assert c.call('inspect')['history'] == source['history']
        checks.append('Default surface selection excludes back vertices without changing authoring/history')

        assert select(minimum=(0, 0), maximum=(10, 10), additive=True) == front
        assert select(minimum=(0, 0), maximum=(10, 10)) is None
        c.call('set_editor_mesh_view', view=dict(edges=True, vertices=True, xray=True))
        select(maximum=(400, 600))
        all_vertices = select(minimum=(400, 0), additive=True)
        assert len(all_vertices['components']['indices']) == len(asset['geometry']['positions'])
        assert select(additive=True) == all_vertices
        before = c.call('inspect')
        c.call('set_editor_mesh_view', view=dict(xray='invalid'), expected_error=True)
        assert c.call('inspect') == before
        assert c.call('inspect_editor')['mesh_view']['xray']
        checks.append('X-ray, additive union, duplicate elimination and empty replacement share one validated session contract')

        reference = c.call('inspect_mesh_topology', asset=instance)['reference']
        obstacle = copy.deepcopy(next(e for e in source['scene']['entities'] if e['id'] == 'floor'))
        obstacle.update(id='occluder', name='occluder')
        obstacle['transform'].update(position=[2, 1.05, 3], size=[4, 4, .1])
        c.call('upsert', entity=obstacle)
        before = c.call('inspect')
        c.call('select_editor_mesh_region', instance=instance, reference=reference, component='vertices',
               min=[0, 0], max=[800, 600], width=800, height=600, expected_error=True)
        assert c.call('inspect') == before
        c.call('set_editor_mesh_view', view=dict(edges=True, vertices=True, xray=False))
        assert select() is None
        before = c.call('inspect')
        c.call('select_editor_mesh_region', instance=instance,
               reference=c.call('inspect_mesh_topology', asset=instance)['reference'], component='vertices',
               min=[-1, 0], max=[800, 600], width=800, height=600, expected_error=True)
        assert c.call('inspect') == before
        obstacle['geometry'] = dict(kind='wedge', rise='positive_x')
        c.call('upsert', entity=obstacle)
        assert select() is not None
        c.call('remove', id='occluder')
        checks.append('Foreign boxes and true wedge volume occlude selection; stale references and invalid rectangles are atomic')

        for domain, field in [('edges', 'vertices'), ('faces', 'indices')]:
            c.call('set_editor_mesh_view', view=dict(edges=True, vertices=True, xray=False))
            visible = select(component=domain)
            c.call('set_editor_mesh_view', view=dict(edges=True, vertices=True, xray=True))
            complete = select(component=domain)
            assert visible['components']['kind'] == complete['components']['kind'] == domain
            assert 0 < len(visible['components'][field]) < len(complete['components'][field])
        before = c.call('inspect')
        path = ROOT/'artifacts/editor-selection-current.png'
        c.call('capture_editor', path='artifacts/editor-selection-current.png', width=1440, height=900)
        pixels = path.read_bytes()
        c.call('capture_editor', path='artifacts/editor-selection-current.png', width=1440, height=900)
        assert path.read_bytes() == pixels and c.call('inspect') == before
        c.call('save_scene', path='examples/editor-selection.scene.json')
        c.call('reset')
        c.call('step', ticks=3)
        c.call('save_replay', path='artifacts/editor-selection.replay.json')
        assert c.call('run_replay', path='artifacts/editor-selection.replay.json')['matched']
        c.call('load_scene', path='examples/editor-selection.scene.json')
        assert c.call('inspect')['scene'] == before['scene']
        checks.append('Edge/face domains, repeated editor pixels, pure captures, save/load and replay remain compatible')
        result = dict(passed=True, checks=checks, build_id=c.call('inspect')['build_id'], native_window=False)
        (ROOT/'artifacts/editor-mesh-selection-verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({**result, 'checks':len(checks)}))
    finally:
        c.close()

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=Path(__file__).resolve().parents[1]/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args()
    verify(args.binary, args.window)
