#!/usr/bin/env python3
"""Inspect one continuously shaped rock body through the native MCP, without a window."""
import copy
import json
import shutil
from pathlib import Path
from verify_lab import Client, ROOT
from open_earth_walk import init
from relief_compositions import position, ZERO

OUT = ROOT / 'artifacts/plateau-relief'
PREFIX = 'artifacts/plateau-relief'
VIEWS = {
    'oblique': {'position': [65, 67, -160], 'target': [0, 23, -10], 'fov_degrees': 48},
    'front': {'position': [0, 32, -170], 'target': [0, 21, -10], 'fov_degrees': 48},
    'side': {'position': [175, 55, -15], 'target': [0, 20, -5], 'fov_degrees': 48},
    'rear': {'position': [110, 75, 165], 'target': [0, 20, 0], 'fov_degrees': 48},
    'top': {'position': [0, 220, 40], 'target': [0, 0, 0], 'fov_degrees': 48},
}


def verify():
    OUT.mkdir(parents=True, exist_ok=True)
    authored = json.loads((ROOT / 'examples/plateau-wall.recipe.json').read_text())
    baseline = copy.deepcopy(authored)
    baseline['version'] = 2
    shape = baseline['pieces'][0].pop('shaping')
    c = Client(ROOT / 'target/release/ignis', log_path=OUT / 'mcp.log')
    try:
        init(c)
        schema = json.dumps(c.request('tools/list')['result'])
        assert 'set_shaping' in schema and 'flat_core' in schema
        build = c.call('inspect')['build_id']
        doc = c.call('query_landform', query={'kind': 'edit', 'document': baseline,
            'expected_revision': 1, 'edits': [{'kind': 'set_shaping', 'id': 'continuous_rock_body', 'shaping': shape}]})['document']
        canonical = c.call('query_landform', query={'kind': 'inspect', 'document': authored})['document']
        assert doc['version'] == 3 and doc['pieces'] == canonical['pieces']
        compiled = c.call('query_landform', query={'kind': 'compile', 'document': doc})
        assert len(compiled['parts']) == 1 and compiled['parts'][0]['closed_oriented']
        assert max(p[1] for p in compiled['geometry']['positions']) == 40
        c.call('save_landform', path=f'{PREFIX}/plateau.ignis-landform', document=doc)
        assert c.call('load_landform', path=f'{PREFIX}/plateau.ignis-landform')['document'] == doc
        rays = [{'origin': [x, y, -120], 'direction': [0, 0, 1], 'max_distance_meters': 200}
                for y in [8, 20, 30] for x in [-60, -40, -20, 0, 20, 40, 60]]
        old_hits = c.call('query_landform', query={'kind': 'raycast', 'document': baseline, 'rays': rays})['hits']
        hits = c.call('query_landform', query={'kind': 'raycast', 'document': doc, 'rays': rays})['hits']
        relief = [a[0]['distance_meters']-b[0]['distance_meters'] for a, b in zip(old_hits, hits)]
        assert max(relief) > 4 and max(relief)-min(relief) > 4, relief
        overhang = c.call('query_landform', query={'kind': 'raycast', 'document': doc, 'rays': [
            {'origin': [18, 80, -32], 'direction': [0, -1, 0], 'max_distance_meters': 100}]})['hits'][0]
        assert len(overhang) >= 2 and any(h['slope_degrees'] > 90 for h in overhang), overhang
        # The same source also compiles on Earth before any publication.
        sphere = copy.deepcopy(doc)
        sphere['origin'] = position(0, 6371000, 0)
        sphere['domain'] = {'kind': 'sphere', 'center': ZERO, 'radius_meters': 6371000, 'forward': [0, 0, 1]}
        mapped = c.call('query_landform', query={'kind': 'compile', 'document': sphere})
        assert mapped['parts'][0]['closed_oriented']
        c.call('generate', recipe={'width': 20, 'depth': 20, 'seed': 221, 'obstacles': 0})
        scene = c.call('inspect')['scene']
        player = copy.deepcopy(next(e for e in scene['entities'] if e['id'] == 'player'))
        c.call('edit_scene', edits=[{'op': 'remove', 'id': e['id']} for e in scene['entities']])
        ground = {'version': 1, 'space_version': 1, 'id': 'floor', 'revision': 1, 'datum': ZERO,
                  'base': {'kind': 'constant', 'height_meters': -.03}, 'operations': []}
        c.call('publish_terrain', publication={'document': ground,
            'patch': {'origin': position(-256, 0, -256), 'cells': [8, 8], 'spacing_meters': 64},
            'stride': 1, 'asset': 'floor', 'instance': 'floor', 'frame_origin': ZERO,
            'color': [.2, .23, .25], 'collision': True})
        light = {'direction': [.35, -.35, 1], 'ambient': .24, 'direct_color': [1, .96, .9],
                 'direct_intensity': 1.2, 'environment': None,
                 'shadow': None,
                 'atmosphere': {'density': 0, 'reduce_details': False, 'color': [.5, .6, .7],
                                'sun_color': [1, 1, 1], 'sun_strength': 0}}
        c.call('set_light', light=light)
        publication = {'document': baseline, 'asset': 'rock', 'instance': 'rock',
                       'frame_origin': ZERO, 'color': [.58, .6, .62], 'shading': 'flat', 'collision': True}
        c.call('publish_landform', publication=publication)
        c.call('set_camera', camera=VIEWS['oblique'])
        c.call('capture', path=f'{PREFIX}/unshaped.png', width=1100, height=720)
        before = c.call('inspect')['scene']
        publication['document'] = doc
        c.call('publish_landform', publication=publication)
        after = c.call('inspect')['scene']
        c.call('undo'); assert c.call('inspect')['scene'] == before
        c.call('redo'); assert c.call('inspect')['scene'] == after
        bad = copy.deepcopy(publication)
        bad['document']['pieces'][0]['shaping']['stamps'][-1]['half_extents'][0] = 0
        c.call('publish_landform', publication=bad, expected_error=True)
        assert c.call('inspect')['scene'] == after
        late = copy.deepcopy(publication)
        late['document']['pieces'][0]['shaping']['stamps'][-1]['displacement'] = [0, -100, 0]
        c.call('publish_landform', publication=late, expected_error=True)
        assert c.call('inspect')['scene'] == after
        islands = c.call('inspect_mesh_islands', asset='rock')['topology']['islands']
        assert len(islands) == 1
        for name, camera in VIEWS.items():
            c.call('set_camera', camera=camera)
            c.call('capture', path=f'{PREFIX}/{name}.png', width=1100, height=720)
            print(json.dumps({'captured': name}), flush=True)
        c.call('set_camera', camera=VIEWS['oblique'])
        c.call('save_scene', path=f'{PREFIX}/plateau.scene.json')
        c.call('load_scene', path=f'{PREFIX}/plateau.scene.json')
        c.call('capture', path=f'{PREFIX}/reloaded.png', width=1100, height=720)
        assert (OUT / 'reloaded.png').read_bytes() == (OUT / 'oblique.png').read_bytes()
        # Separate reference appearance diagnostic: no alignment, geometry-only
        # neutral candidate vs coloured contextual image. Never an artistic pass.
        comparison = None
        ref = OUT / 'reference.json'
        if ref.exists() and Path(json.loads(ref.read_text())['path']).exists():
            shutil.copyfile(json.loads(ref.read_text())['path'], OUT / 'reference-current.png')
            c.call('capture', path=f'{PREFIX}/reference-view.png', width=882, height=472)
            comparison = c.call('compare_images', reference_path=f'{PREFIX}/reference-current.png',
                candidate_path=f'{PREFIX}/reference-view.png', difference_path=f'{PREFIX}/reference-difference.png')['report']
        # Test actual published collision at the shaped foot, not just query output.
        player.update(transform={'position': [-40, .85, -80], 'size': [.6, 1.6, .6]},
            character_controller={'max_slope_degrees': 45, 'slide_slope_degrees': 50,
                                  'offset': .01, 'snap_distance': .2, 'gravity': 9.81})
        c.call('edit_scene', edits=[{'op': 'upsert', 'entity': player}])
        c.call('step', ticks=60)
        c.call('set_input', input={'x': 0, 'z': 1}); c.call('step', ticks=1000)
        c.call('set_input', input={'x': 0, 'z': 0}); c.call('step', ticks=20)
        end = c.call('inspect')['runtime_positions']['player']
        assert -40 < end[2] < -30 and end[1] < 2, end
        c.call('save_replay', path=f'{PREFIX}/collision.replay.json')
        assert c.call('run_replay', path=f'{PREFIX}/collision.replay.json')['matched']
        report = {'passed': True, 'build': build, 'views': VIEWS,
                  'vertices': len(compiled['geometry']['positions']), 'triangles': len(compiled['geometry']['triangles']),
                  'islands': len(islands), 'relief_meters': relief, 'collision_end': end,
                  'overhang_hits': overhang,
                  'sphere_sampled_error_meters': mapped['maximum_sampled_domain_error_meters'],
                  'reference_appearance_diagnostic': comparison,
                  'native_window': False, 'artistic_acceptance': False, 'planet_runtime_integration': False}
        (OUT / 'verification.json').write_text(json.dumps(report, indent=2))
        print(json.dumps({k: v for k, v in report.items() if k not in ['reference_appearance_diagnostic', 'relief_meters', 'views']}), flush=True)
    finally:
        c.close()


if __name__ == '__main__':
    verify()
