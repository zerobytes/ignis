"""Native gizmo contract, exercised through the same MCP dispatcher without a window."""
import argparse, copy, json, math
from pathlib import Path
from verify_lab import Client, ROOT

def verify(binary, window):
    if window:
        raise ValueError('Native acceptance is separate; this regression is headless')
    c = Client(binary, False)
    checks = []
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='editor-transform', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe=dict(width=16, depth=14, seed=42, obstacles=0))
        c.call('create_editor_mesh', primitive='sphere', resolution=24, size=2)
        instance = c.call('inspect_editor')['selected_mesh']
        before = c.call('inspect')
        settings = c.call('inspect_editor')['transform']
        settings.update(tool='move', space='local', pivot='custom', custom_pivot=[1, 0, 0], snap=True)
        c.call('set_editor_transform', settings=settings)
        info = c.call('inspect_editor_transform')
        assert info['settings'] == settings and info['selection']['can_rotate']
        bad = copy.deepcopy(settings); bad['move_step'] = 0
        c.call('set_editor_transform', settings=bad, expected_error=True)
        assert c.call('inspect') == before
        assert c.call('inspect_editor_transform')['settings'] == settings
        checks.append('Validated session settings and typed target inspection preserve scene/runtime/history')

        target = info['selection']['target']
        identity = dict(pivot=info['selection']['center'], translation=[0, 0, 0], rotation=[0, 0, 0, 1], scale=1)
        t = copy.deepcopy(identity); t.update(translation=[.3, .1, 0], rotation=[0, math.sin(.2), 0, math.cos(.2)], scale=1.2)
        preview = c.call('preview_editor_transform', target=target, reference=info['selection']['reference'], transform=t)['scene']
        assert c.call('inspect') == before
        c.call('transform_editor', target=target, reference=info['selection']['reference'], transform=t)
        after = c.call('inspect')
        assert after['scene'] == preview
        assert after['history']['undo'] == before['history']['undo'] + 1
        assert after['scene']['meshes']['assets'] == before['scene']['meshes']['assets']
        assert after['scene']['meshes']['instances'] != before['scene']['meshes']['instances']
        c.call('undo'); assert c.call('inspect')['scene'] == before['scene']
        c.call('redo'); assert c.call('inspect')['scene'] == after['scene']
        c.call('transform_editor', target=target, reference=info['selection']['reference'], transform=t, expected_error=True)
        assert c.call('inspect')['scene'] == after['scene']
        checks.append('World-space pivot/rotation/uniform-scale update instance only; undo/redo and stale references are atomic')

        asset = c.call('inspect_mesh', asset=instance)['asset']
        components = dict(kind='vertices', indices=list(range(len(asset['geometry']['positions']))))
        reference = c.call('inspect_mesh_topology', asset=instance)['reference']
        c.call('select_mesh_components', selection=dict(asset=instance, reference=reference, components=components))
        info = c.call('inspect_editor_transform', components=True)
        t = copy.deepcopy(identity); t['translation'] = [.01, .02, -.01]
        c.call('transform_editor', target=info['selection']['target'], reference=info['selection']['reference'], transform=t)
        selected = c.call('inspect_editor_transform', components=True)['selection']
        assert selected['target']['components'] == components
        assert selected['reference'] != reference
        c.call('reset')
        frozen = c.call('inspect')
        c.call('transform_editor', target=selected['target'], reference=selected['reference'], transform=t, expected_error=True)
        assert c.call('inspect') == frozen
        checks.append('Component transforms preserve the component domain with refreshed reference; reset invalidates the old scope')

        c.call('select_entity', id='floor')
        info = c.call('inspect_editor_transform')
        assert info['selection']['target'] == dict(kind='entity', id='floor') and not info['selection']['can_rotate']
        frozen = c.call('inspect'); t = copy.deepcopy(identity); t['rotation'] = [0, math.sin(.2), 0, math.cos(.2)]
        c.call('transform_editor', target=info['selection']['target'], reference=info['selection']['reference'], transform=t, expected_error=True)
        assert c.call('inspect') == frozen
        t = copy.deepcopy(identity); t['translation'] = [0, .01, 0]
        c.call('transform_editor', target=info['selection']['target'], reference=info['selection']['reference'], transform=t)
        c.call('undo'); assert c.call('inspect')['scene'] == frozen['scene']
        c.call('play'); frozen = c.call('inspect')
        info = c.call('inspect_editor_transform')
        c.call('transform_editor', target=info['selection']['target'], reference=info['selection']['reference'], transform=t, expected_error=True)
        assert c.call('inspect') == frozen
        c.call('pause')
        checks.append('Axis-aligned bodies support translation; arbitrary rotation and edits during play are refused without partial mutation')

        c.call('select_editor_mesh', instance=instance)
        settings.update(tool='rotate', pivot='center'); c.call('set_editor_transform', settings=settings)
        c.call('save_scene', path='examples/editor-transform.scene.json')
        c.call('step', ticks=3)
        c.call('save_replay', path='artifacts/editor-transform.replay.json')
        assert c.call('run_replay', path='artifacts/editor-transform.replay.json')['matched']
        before = c.call('inspect')
        path = ROOT/'artifacts/editor-transform-current.png'
        c.call('capture_editor', path='artifacts/editor-transform-current.png', width=1440, height=900)
        pixels = path.read_bytes()
        c.call('capture_editor', path='artifacts/editor-transform-current.png', width=1440, height=900)
        assert pixels == path.read_bytes() and c.call('inspect') == before
        checks.append('Resulting authoring scene replays; repeated offscreen editor capture is pixel-stable and read-only')
        result = dict(passed=True, build_id=before['build_id'], checks=checks, native_window=False)
        (ROOT/'artifacts/editor-transform-verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({**result, 'checks':len(checks)}))
    finally:
        c.close()

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=Path(__file__).resolve().parents[1]/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args(); verify(args.binary, args.window)
