#!/usr/bin/env python3
"""Material-domain authoring on a live plant: purity, source preservation and persistence."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from demo_tree_material_frames import recipe_frames, CAMS
ID = 'branch-editable-1'

def verify(binary, window=False):
    c = Client(binary, window)
    call = c.call
    checks = []
    try:
        c.request('initialize', {'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'plant-material-frames','version':'1'}})
        c.request('notifications/initialized', notification=True)
        schema = json.dumps(c.request('tools/list')['result'])
        assert all(k in schema for k in ['source_material_frame','plant_material_frame','material_finish'])
        call('load_scene', path='examples/native-tree-study.scene.json')
        old = call('inspect_plant', id=ID)['plant']
        # This verifier keeps the committed recipe as its baseline, whether it
        # already contains these optional layers or not.
        baseline = copy.deepcopy(old['assembly'])
        baseline.pop('plant_material_frame', None)
        baseline.pop('material_finish', None)
        for graft in baseline['context_grafts']:
            graft.pop('source_material_frame', None)
            graft.pop('material_finish', None)
        call('edit_plant', edit={'kind':'set_assembly','id':ID,'assembly':baseline})
        before = call('inspect')['scene']
        asset0 = call('inspect_mesh', asset=ID)['asset']
        placement = next(i['position'] for i in before['meshes']['instances'] if i['asset']==ID)
        recipe = recipe_frames(baseline, placement)
        edit = {'kind':'set_assembly','id':ID,'assembly':recipe}
        state = call('inspect')
        pred = call('preview_plant', edit=edit)
        assert call('inspect') == state
        call('edit_plant', edit=edit)
        after = call('inspect')['scene']
        current = call('inspect_plant', id=ID)
        asset1 = call('inspect_mesh', asset=ID)['asset']
        assert asset1 == next(a for a in pred['assets'] if a['id']==ID)
        for key in ['positions','triangles']:
            assert asset1['geometry'][key] == asset0['geometry'][key]
        for key in ['rig','clips','material','shading']:
            assert asset1.get(key) == asset0.get(key)
        for key in ['source','reference','definition']:
            assert current['plant'][key] == old[key]
        assert recipe['context'] == baseline['context']
        assert recipe['context_grafts'][0]['source'] == baseline['context_grafts'][0]['source']
        report = current['report']['assembly']
        for f in [report['material_finish'], report['context_grafts'][0]['material_finish']]:
            assert f['after']['guard']['near_singular_faces'] == 0
            assert f['residual_max'] <= .002
        assert call('inspect_mesh_material_domain', asset=ID)['report']['near_singular_faces'] == 0
        assert after['meshes']['instances'] == before['meshes']['instances']
        assert [a for a in after['meshes']['assets'] if a['id']!=ID] == [a for a in before['meshes']['assets'] if a['id']!=ID]
        checks.append('Native preview/publication repairs both final collars with canonical sources/positions/topology/rig/clips/instances exact')
        for case in range(5):
            a = copy.deepcopy(recipe)
            g = a['context_grafts'][0]
            if case == 0:
                g['source_material_frame']['offset'][1] = 10001
            elif case == 1:
                g['source_material_frame']['columns'][1][1] = 10000
            elif case == 2:
                g['material_finish']['max_change'] = .000001
            elif case == 3:
                g['source_material_frame']['columns'] = [[0,0,0]]*3
            else:
                a['plant_material_frame']['unknown'] = 1
            state = call('inspect')
            call('preview_plant', edit={'kind':'set_assembly','id':ID,'assembly':a}, expected_error=True)
            call('edit_plant', edit={'kind':'set_assembly','id':ID,'assembly':a}, expected_error=True)
            assert call('inspect') == state
        legacy = copy.deepcopy(after)
        legacy['version'] = 69
        (ROOT/'artifacts/material-frame-old.scene.json').write_text(json.dumps(legacy))
        state = call('inspect')
        assert 'version70' in str(call('load_scene', path='artifacts/material-frame-old.scene.json', expected_error=True))
        assert call('inspect') == state
        checks.append('Malformed/schema/bounds/singular-support/solver and declared old-format errors preserve complete state')
        call('save_scene', path='examples/material-frame.scene.json')
        call('undo'); assert call('inspect')['scene'] == before
        call('redo'); assert call('inspect')['scene'] == after
        call('load_scene', path='examples/material-frame.scene.json'); assert call('inspect')['scene'] == after
        d = copy.deepcopy(old['definition'])
        d['trajectory']['control_points_xz'][3][0] += .001
        for definition in [d, old['definition']]:
            call('edit_plant', edit={'kind':'put','definition':definition,'detail_policy':'physical','animation_translation_policy':'physical'})
        assert call('inspect')['scene'] == after
        checks.append('Recipe save/load/undo/redo and trajectory return regenerate exact current geometry and authored domain')
        call('set_camera', camera=CAMS['neck']);call('step', ticks=60)
        call('capture', path='artifacts/material-frame.png', width=600, height=600)
        call('capture', path='artifacts/material-frame-repeat.png', width=600, height=600)
        call('save_replay', path='artifacts/material-frame.replay.json')
        assert call('run_replay', path='artifacts/material-frame.replay.json')['meshes_matched']
        call('capture_replay', replay_path='artifacts/material-frame.replay.json', path='artifacts/material-frame-replay.png', tick=60, width=600, height=600)
        assert len({(ROOT/f'artifacts/{p}.png').read_bytes() for p in ['material-frame','material-frame-repeat','material-frame-replay']}) == 1
        checks.append('Native headless captures repeat and match strict-build replay after material-only regeneration')
        return {'passed':True, 'checks':len(checks), 'details':checks, 'build_id':call('inspect')['build_id']}
    finally:
        c.close()
if __name__ == '__main__':
    p = argparse.ArgumentParser(); p.add_argument('--binary',type=Path,required=True);p.add_argument('--window',action='store_true');args=p.parse_args()
    print(json.dumps(verify(args.binary,args.window)))
