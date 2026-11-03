#!/usr/bin/env python3
"""Regenerable material frames and final-geometry finishing on the maintained tree."""
import copy
import hashlib
import json
import os
import subprocess
from pathlib import Path
PROJECT = Path(__file__).resolve().parents[1]
OUTPUT = PROJECT / 'artifacts/tree-material-recipe'
ID = 'branch-editable-1'
CAMS = {'neck': {'position':[1.8,2.9,4.0], 'target':[-.57,2.65,.30], 'fov_degrees':19},
        'profile': {'position':[-3,3.2,-3.5], 'target':[-.67,2.85,.25], 'fov_degrees':20},
        'fork': {'position':[3.5,3.2,4.7], 'target':[.55,2.95,.13], 'fov_degrees':25}}
def affine(offset):
    return {'columns':[[1,0,0],[0,1,0],[0,0,1]], 'offset':offset}
def options(offset):
    return {'reference':affine(offset), 'iterations':64, 'tolerance':.002, 'max_change':5, 'max_anisotropy':10000}
def recipe_frames(recipe, placement, main=True):
    a = copy.deepcopy(recipe)
    g = a['context_grafts'][0]
    g['source_material_frame'] = affine([-.4706738831428572,2.841677142857143,.32152964428571423])
    g['material_finish'] = options(g['options']['material_offset'])
    if main:
        a['plant_material_frame'] = affine(placement)
        a['material_finish'] = options(a['options']['material_offset'])
    return a

def run():
    (OUTPUT/'examples').mkdir(parents=True, exist_ok=True)
    (OUTPUT/'artifacts').mkdir(exist_ok=True)
    os.environ['IGNIS_VERIFY_ROOT'] = str(OUTPUT)
    from verify_lab import Client
    source = subprocess.run(['git','show','7d84460:examples/native-tree-study.scene.json'],cwd=PROJECT,check=True,stdout=subprocess.PIPE).stdout
    (OUTPUT/'examples/input.scene.json').write_bytes(source)
    c = Client(PROJECT/'target/release/ignis', False)
    call = c.call
    try:
        c.request('initialize',{'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'tree-material-recipe','version':'1'}})
        c.request('notifications/initialized',notification=True)
        call('load_scene',path='examples/input.scene.json')
        initial = call('inspect')['scene']
        old = call('inspect_plant',id=ID)['plant']
        placement = next(i['position'] for i in initial['meshes']['instances'] if i['asset']==ID)
        stages = {}
        for name, main in [('side',False),('combined',True)]:
            recipe = recipe_frames(old['assembly'], placement, main)
            edit = {'kind':'set_assembly','id':ID,'assembly':recipe}
            state = call('inspect')
            prediction = call('preview_plant',edit=edit)
            assert call('inspect') == state
            call('edit_plant',edit=edit)
            scene = call('inspect')['scene']
            evaluated = call('inspect_plant',id=ID)
            asset = call('inspect_mesh',asset=ID)['asset']
            before = next(a for a in initial['meshes']['assets'] if a['id']==ID)
            assert asset == next(a for a in prediction['assets'] if a['id']==ID)
            for key in ['positions','triangles']:
                assert asset['geometry'][key] == before['geometry'][key]
            for key in ['rig','clips','material','shading']:
                assert asset.get(key) == before.get(key)
            for key in ['source','reference','definition']:
                assert evaluated['plant'][key] == old[key]
            assert recipe['context'] == old['assembly']['context']
            assert recipe['context_grafts'][0]['source'] == old['assembly']['context_grafts'][0]['source']
            assert scene['meshes']['instances'] == initial['meshes']['instances']
            assert [a for a in scene['meshes']['assets'] if a['id']!=ID] == [a for a in initial['meshes']['assets'] if a['id']!=ID]
            call('save_scene',path=f'examples/{name}.scene.json')
            stages[name] = evaluated['report']['assembly']
            call('load_scene',path='examples/input.scene.json')
        call('save_scene',path='examples/before.scene.json')
        call('load_scene',path='examples/combined.scene.json')
        actual = call('inspect')['scene']
        recipe = call('inspect_plant',id=ID)['plant']['assembly']
        for key in ['plant_material_frame','material_finish']:
            assert key in recipe
        for _ in range(2):
            d = copy.deepcopy(old['definition'])
            d['trajectory']['control_points_xz'][3][0] += .001
            for definition in [d,old['definition']]:
                call('edit_plant',edit={'kind':'put','definition':definition,'detail_policy':'physical','animation_translation_policy':'physical'})
            assert call('inspect')['scene'] == actual
        changed = copy.deepcopy(recipe)
        changed['context_lofts'][0]['spans'] = 4
        call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':changed})
        call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':recipe})
        assert call('inspect')['scene'] == actual
        call('edit_plant',edit={'kind':'set_assembly','id':ID,'assembly':old['assembly']})
        assert call('inspect')['scene'] == initial
        call('undo'); assert call('inspect')['scene'] == actual
        call('redo'); assert call('inspect')['scene'] == initial
        for label in ['before','combined']:
            for name, cam in dict(CAMS,whole=actual['camera']).items():
                for neutral in [False,True]:
                    call('load_scene',path=f'examples/{label}.scene.json')
                    if neutral:
                        edits = [{'op':'mesh','edit':{'kind':'remove_instance','id':i['id']}} for i in actual['meshes']['instances'] if i['id'].startswith('crown-')]
                        edits.extend({'op':'mesh','edit':{'kind':'set_material','asset':a['id'],'material':{}}} for a in actual['meshes']['assets'] if a['id'] in ['wood',ID,'secondary-06','secondary-twigs'])
                        call('edit_scene',edits=edits)
                        call('set_light',light={'direction':[-.8,-1,-.4],'ambient':.35})
                    call('set_camera',camera=cam);call('step',ticks=60)
                    call('capture',path=f'artifacts/{label}-'+('neutral-' if neutral else '')+name+'.png',width=1100,height=1100)
        for name in dict(CAMS,whole=None):
            assert (OUTPUT/f'artifacts/before-neutral-{name}.png').read_bytes() == (OUTPUT/f'artifacts/combined-neutral-{name}.png').read_bytes()
        call('load_scene',path='examples/combined.scene.json');call('step',ticks=60)
        call('capture',path='artifacts/live.png',width=1100,height=1100)
        call('capture',path='artifacts/repeat.png',width=1100,height=1100)
        assert (OUTPUT/'artifacts/live.png').read_bytes() == (OUTPUT/'artifacts/repeat.png').read_bytes()
        call('save_replay',path='artifacts/tree.replay.json')
        assert call('run_replay',path='artifacts/tree.replay.json')['meshes_matched']
        call('capture_replay',replay_path='artifacts/tree.replay.json',path='artifacts/replay.png',tick=60,width=1100,height=1100)
        assert (OUTPUT/'artifacts/live.png').read_bytes() == (OUTPUT/'artifacts/replay.png').read_bytes()
        report = {'passed':True,'build_id':call('inspect')['build_id'],'baseline_sha256':hashlib.sha256(source).hexdigest(),
                  'geometry_topology_rig_clips_sources_instances_exact':True,'neutral_pixels_exact':True,
                  'stages':stages,'whole_tree_realism_accepted':False}
        (OUTPUT/'artifacts/report.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'passed':True,'build_id':report['build_id'],'neutral_pixels_exact':True}))
    finally:
        c.close()
if __name__ == '__main__':
    run()
