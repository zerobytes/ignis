#!/usr/bin/env python3
"""Geological profiles through native MCP; real holes, authoring, undo and replay."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window=False):
    c = Client(binary, window)
    checks = []
    try:
        c.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'landforms', 'version': '1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result'])
        assert all(name in schema for name in ['query_landform', 'publish_landform', 'save_landform', 'load_landform'])
        recipe = json.loads((ROOT / 'examples/landforms.recipe.json').read_text())
        before = c.call('inspect')
        docs = {}
        for form in recipe['forms']:
            if 'preset' in form:
                doc = c.call('query_landform', query={'kind': 'preset', 'id': form['id'], 'origin': recipe['origin'], 'preset': form['preset']})['document']
            else:
                doc = c.call('query_landform', query={'kind': 'inspect', 'document': form['document']})['document']
            piece = copy.deepcopy(doc['pieces'][0])
            piece['anchor'] = form['anchor']
            edits = [{'kind': 'replace', 'id': piece['id'], 'piece': piece}]
            if 'sections' in form:
                edits.append({'kind': 'set_sections', 'id': piece['id'], 'sections': form['sections']})
            doc = c.call('query_landform', query={'kind': 'edit', 'document': doc, 'expected_revision': doc['revision'], 'edits': edits})['document']
            docs[form['id']] = doc
        gallery = {'version': 1, 'space_version': 1, 'id': 'landform-gallery', 'revision': 1, 'origin': recipe['origin'], 'pieces': [doc['pieces'][0] for doc in docs.values()]}
        compiled = c.call('query_landform', query={'kind': 'compile', 'document': gallery})
        assert len(compiled['parts']) == 8 and all(p['closed_oriented'] and p['signed_volume_cubic_meters'] > 0 for p in compiled['parts'])
        assert not compiled['boolean_union'] and compiled['visual_only']
        c.call('save_landform', path='examples/landforms.ignis-landform', document=gallery)
        assert c.call('load_landform', path='examples/landforms.ignis-landform')['document'] == gallery
        c.call('load_landform', path='examples/terrain-tools.ignis-terrain', expected_error=True)
        bad = copy.deepcopy(gallery)
        bad['pieces'][0]['profile'] = [[-1, -1], [1, 1], [-1, 1], [1, -1]]
        c.call('query_landform', query={'kind': 'compile', 'document': bad}, expected_error=True)
        c.call('query_landform', query={'kind': 'edit', 'document': gallery, 'expected_revision': 1, 'edits': [{'kind': 'remove', 'id': 'ridge'}, {'kind': 'remove', 'id': 'missing'}]}, expected_error=True)
        assert c.call('load_landform', path='examples/landforms.ignis-landform')['document'] == gallery
        assert c.call('inspect') == before
        checks.append('Manual ridge and parameterized profiles share editable source; eight closed oriented components, bounded binary source, late errors preserve file/live state')

        def ray(form, at, direction):
            doc = docs[form]
            anchor = doc['pieces'][0]['anchor']
            origin = [a+b for a, b in zip(at, anchor)]
            result = c.call('query_landform', query={'kind': 'raycast', 'document': doc, 'rays': [{'origin': origin, 'direction': direction, 'max_distance_meters': 100}]})
            assert result['visual_query_only']
            return result['hits'][0]
        wave = ray('wave', [2, 40, 5], [0, -1, 0])
        assert len(wave) == 2 and wave[1]['point'][1] > 1
        assert ray('canyon', [0, 30, 6], [0, -1, 0])[0]['point'][1] == 0
        assert abs(ray('canyon', [0, 5, 7], [1, 0, 0])[0]['point'][0]+11) < 1e-6
        assert ray('canyon', [8, 30, 6], [0, -1, 0])[0]['point'][1] == 10
        assert not ray('arch', [0, 1, -2], [0, 0, 1])
        assert len(ray('arch', [0, 8, -2], [0, 0, 1])) == 2
        assert not ray('tunnel', [0, 0, -2], [0, 0, 1])
        assert len(ray('tunnel', [10, 0, 6], [-1, 0, 0])) == 4
        assert c.call('inspect') == before
        checks.append('Triangle queries prove wave overhang, vertical canyon walls/flat floor/top, unobstructed arch and hollow tunnel; no gameplay collider inferred')

        c.call('generate', recipe={'width': 12, 'depth': 12, 'seed': 166, 'obstacles': 0})
        c.call('edit_scene', edits=[{'op': 'remove', 'id': e['id']} for e in c.call('inspect')['scene']['entities']])
        native_before = c.call('inspect')['scene']
        first = True
        for form in recipe['forms']:
            publication = {'document': docs[form['id']], 'asset': form['id'], 'instance': form['id'], 'frame_origin': recipe['origin'], 'color': form['color'], 'shading': 'flat'}
            if first:
                bad = copy.deepcopy(publication)
                bad['instance'] = ''
                c.call('publish_landform', publication=bad, expected_error=True)
                assert c.call('inspect')['scene'] == native_before
            result = c.call('publish_landform', publication=publication)
            assert result['visual_only'] and not result['source_embedded_in_scene']
            if first:
                after = c.call('inspect')['scene']
                c.call('undo')
                assert c.call('inspect')['scene'] == native_before
                c.call('redo')
                assert c.call('inspect')['scene'] == after
                first = False
        ground = {'id': 'ground', 'geometry': {'positions': [[-59, -2.05, -24], [57, -2.05, -24], [-59, -2.05, 31], [57, -2.05, 31]], 'triangles': [[0, 2, 1], [1, 2, 3]]}}
        instance = {'id': 'ground', 'asset': 'ground', 'position': [0, 0, 0], 'rotation': [0, 0, 0, 1], 'scale': 1, 'color': [.30, .34, .29]}
        c.call('edit_scene', edits=[{'op': 'mesh', 'edit': {'kind': 'put_asset', 'asset': ground}}, {'op': 'mesh', 'edit': {'kind': 'put_instance', 'instance': instance}}])
        c.call('set_camera', camera=recipe['views'][0]['camera'])
        c.call('save_scene', path='artifacts/landforms.scene.json')
        scene = c.call('inspect')['scene']
        c.call('capture', path='artifacts/landforms-gallery.png', width=1280, height=800)
        pixels = (ROOT / 'artifacts/landforms-gallery.png').read_bytes()
        c.call('step', ticks=2)
        c.call('save_replay', path='artifacts/landforms.replay.json')
        assert c.call('run_replay', path='artifacts/landforms.replay.json')['meshes_matched']
        c.call('capture_replay', replay_path='artifacts/landforms.replay.json', path='artifacts/landforms-replay.png', tick=2, width=1280, height=800)
        assert (ROOT / 'artifacts/landforms-replay.png').read_bytes() == pixels
        c.call('load_scene', path='artifacts/landforms.scene.json')
        assert c.call('inspect')['scene'] == scene
        c.call('capture', path='artifacts/landforms-loaded.png', width=1280, height=800)
        assert (ROOT / 'artifacts/landforms-loaded.png').read_bytes() == pixels
        for view in recipe['views'][1:]:
            c.call('set_camera', camera=view['camera'])
            c.call('capture', path=f"artifacts/landforms-{view['name']}.png", width=960, height=720)
        # Leave the reusable scene at the gallery camera, not at a diagnostic view.
        c.call('load_scene', path='artifacts/landforms.scene.json')
        checks.append('Native eight-form gallery, atomic publication/one undo, exact save/load and replay pixels; offscreen detail views, no window or audio')
        report = {'passed': True, 'build_id': c.call('inspect')['build_id'], 'native_window': window, 'checks': checks, 'vertices': len(compiled['geometry']['positions']), 'triangles': len(compiled['geometry']['triangles']), 'scope': {'terrain_physics': False, 'erosion_simulated': False, 'boolean_union': False, 'streaming': False}, 'images': [f"artifacts/landforms-{v['name']}.png" for v in recipe['views']]}
        (ROOT / 'artifacts/landforms-verification.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'groups': len(checks), 'build_id': report['build_id']}))
    finally:
        c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/ignis')
    p.add_argument('--window', action='store_true')
    a = p.parse_args()
    verify(a.binary.resolve(), a.window)
