"""Six shaping kernels, persistent preferences and deterministic public sculpt commands."""
import argparse, copy, json, math
from pathlib import Path
from verify_lab import Client, ROOT
SHAPES = ['flatten', 'pinch', 'crease', 'scrape', 'fill', 'relax']

def verify(binary, window):
    if window:
        raise ValueError('This verification never opens a window')
    c = Client(binary, False); checks = []
    try:
        c.request('initialize', dict(protocolVersion='2025-11-25', capabilities={}, clientInfo=dict(name='sculpt-brushes', version='1')))
        c.request('notifications/initialized', notification=True)
        c.call('generate', recipe=dict(width=16, depth=14, seed=42, obstacles=0))
        c.call('create_editor_mesh', primitive='plane', resolution=24, size=2)
        asset = c.call('inspect_mesh', asset='mesh_0')['asset']
        for p in asset['geometry']['positions']:
            p[1] = .12*math.sin(p[0]*7)*math.cos(p[2]*4)
            p[0] += .012*math.sin(p[2]*19+p[0]*11)
        c.call('edit_mesh', edit=dict(kind='put_asset', asset=asset))
        base = c.call('inspect'); outputs = []
        centers = [[-.15, -.1, 0], [0, 0, 0], [.15, .1, 0]]
        for brush in SHAPES:
            stroke = dict(kind=brush, centers=centers, radius=.65, strength=.25, direction=[0, 1, 0])
            edit = dict(kind='sculpt', asset='mesh_0', stroke=stroke)
            before = c.call('inspect')
            preview = c.call('preview_mesh_edit', edit=edit)['meshes']
            assert c.call('inspect') == before
            assert preview != before['scene']['meshes']
            c.call('edit_mesh', edit=edit)
            after = c.call('inspect')
            assert after['scene']['meshes'] == preview and after['history']['undo'] == before['history']['undo'] + 1
            c.call('undo'); assert c.call('inspect')['scene'] == before['scene']
            c.call('redo'); assert c.call('inspect')['scene']['meshes'] == preview
            c.call('save_scene', path='examples/sculpt-brushes.scene.json')
            c.call('step', ticks=3)
            c.call('save_replay', path='artifacts/sculpt-brushes.replay.json')
            assert c.call('run_replay', path='artifacts/sculpt-brushes.replay.json')['matched']
            c.call('load_scene', path='examples/sculpt-brushes.scene.json')
            assert c.call('inspect')['scene']['meshes'] == preview
            outputs.append(preview)
            c.call('edit_mesh', edit=dict(kind='put_asset', asset=asset))
            reverse = copy.deepcopy(edit); reverse['stroke']['centers'] = list(reversed(centers))
            reversed_mesh = c.call('preview_mesh_edit', edit=reverse)['meshes']
            for p, q in zip(preview['assets'][0]['geometry']['positions'], reversed_mesh['assets'][0]['geometry']['positions']):
                assert math.dist(p, q) < 2e-6
            checks.append(f'{brush}: real deformation, identical preview/commit, single undo/redo, path reversal, save/load and replay')
        assert len({json.dumps(m, sort_keys=True) for m in outputs}) == 6
        before = c.call('inspect')
        for brush in ['flatten', 'relax']:
            c.call('edit_mesh', edit=dict(kind='sculpt', asset='mesh_0', stroke=dict(kind=brush, centers=centers, radius=.65, strength=-.25, direction=[0, 1, 0])), expected_error=True)
            assert c.call('inspect') == before
        checks.append('Six distinct kernels; unsupported inversion fails atomically')
        old = dict(brush='draw', radius=.4, draw_strength=.04, grab_strength=.7, smooth_strength=.5)
        c.call('edit_editor', action=dict(action='sculpt', settings=old))
        prefs = c.call('inspect_editor')['editor']['sculpt']
        for i, brush in enumerate(SHAPES):
            prefs[brush+'_strength'] = .1+i*.07
        for brush in SHAPES:
            prefs['brush'] = brush
            submitted = copy.deepcopy(prefs)
            c.call('edit_editor', action=dict(action='sculpt', settings=prefs))
            prefs = c.call('inspect_editor')['editor']['sculpt']
            assert prefs['brush'] == submitted['brush']
            assert all(math.isclose(prefs[k], v, abs_tol=1e-6) for k, v in submitted.items() if k not in ('brush', 'detail'))
            assert prefs['detail'] == submitted['detail']
            c.call('save_editor', path='artifacts/sculpt-brushes.layout.json')
            c.call('edit_editor', action=dict(action='sculpt', settings=old))
            c.call('load_editor', path='artifacts/sculpt-brushes.layout.json')
            assert c.call('inspect_editor')['editor']['sculpt'] == prefs
        checks.append('Old layout preferences load through defaults and all nine brushes retain independent validated strength')
        # Preference persistence is separate from scene/history. The public tool
        # must accept the same typed resolution that native pointer input freezes.
        before = c.call('inspect')
        assert prefs['detail'] == dict(mode='brush', percent=12.5)
        for detail in [dict(mode='brush', percent=25), dict(mode='constant', edge_length=.05)]:
            prefs.update(brush='draw', adaptive_detail=True, detail=detail)
            c.call('edit_editor', action=dict(action='sculpt', settings=prefs))
            prefs = c.call('inspect_editor')['editor']['sculpt']
            assert prefs['detail']['mode'] == detail['mode']
            field = 'percent' if detail['mode'] == 'brush' else 'edge_length'
            assert math.isclose(prefs['detail'][field], detail[field], abs_tol=1e-6)
            c.call('save_editor', path='artifacts/sculpt-brushes.layout.json')
            c.call('edit_editor', action=dict(action='sculpt', settings=old))
            c.call('load_editor', path='artifacts/sculpt-brushes.layout.json')
            assert c.call('inspect_editor')['editor']['sculpt'] == prefs
        bad = copy.deepcopy(prefs); bad['detail']['edge_length'] = -1
        c.call('edit_editor', action=dict(action='sculpt', settings=bad), expected_error=True)
        assert c.call('inspect_editor')['editor']['sculpt'] == prefs
        assert c.call('inspect') == before
        checks.append('Brush/constant detail survives save/load; invalid configuration is atomic and preferences never mutate the scene')
        c.call('capture_editor', path='artifacts/sculpt-brushes-current.png', width=1440, height=900)
        result = dict(passed=True, checks=checks, build_id=c.call('inspect')['build_id'], native_window=False)
        (ROOT/'artifacts/sculpt-brushes-verification.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({**result,'checks':len(checks)}))
    finally:
        c.close()

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=Path(__file__).resolve().parents[1]/'target/release/ignis')
    p.add_argument('--window', action='store_true')
    args = p.parse_args(); verify(args.binary, args.window)
