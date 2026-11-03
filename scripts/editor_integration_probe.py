"""Cross-front editor acceptance: geometry, environment, normals and authoring."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import copy
import json
from verify_lab import ROOT
from verify_layered_patterns import pattern


def verify_combined(c):
    checks = []
    c.call('load_scene', path='examples/npc-spawn-support.scene.json')
    assert c.call('inspect')['scene']['version'] == SCENE_VERSION
    sweep = json.loads((ROOT/'examples/profile-sweep.edit.json').read_text())
    state = c.call('inspect')
    plan = c.call('plan_mesh_sweep', sweeps=sweep['sweeps'])['plan']
    assert not plan['surface_validated'] and c.call('inspect') == state
    c.call('edit_mesh', edit=sweep)
    assert len(c.call('inspect_mesh', asset='molding')['asset']['geometry']['positions']) == plan['vertices']
    material = dict(pattern=pattern(), secondary_pattern=pattern(seed=71),
                    reflectance=dict(roughness=.7, f0=.04, metallic=.8))
    edit = dict(kind='set_material', asset='molding', material=material)
    before = c.call('inspect')
    preview = c.call('preview_mesh_edit', edit=edit)['meshes']
    assert c.call('inspect') == before
    c.call('edit_mesh', edit=edit)
    assert c.call('inspect')['scene']['meshes'] == preview
    c.call('undo')
    assert c.call('inspect')['scene'] == before['scene']
    c.call('redo')
    assert c.call('inspect')['scene']['meshes'] == preview
    c.call('edit_mesh', edit=dict(kind='put_instance', instance=dict(
        id='integrated-molding', asset='molding', position=[0, 2.5, 0],
        rotation=[0, 0, 0, 1], scale=1, color=[.6, .32, .12], playback=None)))
    center = [-.6875, 3.625, .078125]
    c.call('set_camera', camera=dict(position=[center[0]+4, center[1]+3, center[2]+8],
                                   target=center, fov_degrees=40))
    state = c.call('inspect')
    hit = c.call('pick_editor_mesh', x=400, y=300, width=800, height=600)['hit']
    assert hit and hit['instance'] == 'integrated-molding' and c.call('inspect') == state
    c.call('select_editor_mesh', instance='integrated-molding')
    assert c.call('inspect_editor')['selected_mesh'] == 'integrated-molding'
    checks.append('Combined support scene, pure Sweep plan/preview, layered metallic material, single undo/redo and editor triangle picking')

    wedge = dict(id='integrated-wedge', name='Integrated wedge', body='fixed',
                 color=[.2, .6, .5], transform=dict(position=[4, 1, -3], size=[4, 2, 2]),
                 geometry=dict(kind='wedge', rise='positive_x'))
    c.call('upsert', entity=wedge)
    state = c.call('inspect')
    assert c.call('query_geometry', entity=wedge['id'], start=[3, 1.5, -5], end=[3, 1.5, -1])['fraction'] is None
    assert c.call('query_geometry', entity=wedge['id'], start=[5, 1, -5], end=[5, 1, -1])['fraction'] is not None
    normals = c.call('inspect_mesh_normals', instance='integrated-molding', sample_limit=3)['report']
    assert normals['space'] == 'world' and normals['stage'] == 'pre_material' and len(normals['samples']) == 3
    assert c.call('inspect') == state
    light = copy.deepcopy(state['scene']['light'])
    light.update(direct_intensity=.7, environment=dict(sky=[.6, .75, .95],
        horizon=[.8, .75, .6], ground=[.05, .04, .025], axis=[0, 1, 0], intensity=.55))
    c.call('set_light', light=light)
    light = c.call('inspect')['scene']['light']
    c.call('edit_editor', action=dict(action='lighting', light=light))
    checks.append('Wedge void/solid and posed-normal queries are pure in the combined world; environment and editor work light share validation')

    state = c.call('inspect')
    result = c.call('query_spawn_support', template='guard', position=[-4, 8, 0], max_drop=10)
    assert result['report']['status'] == 'supported', result
    assert c.call('inspect') == state
    settings = copy.deepcopy(c.call('inspect_editor')['editor']['sculpt'])
    settings.update(radius=.47, draw_strength=.1)
    c.call('edit_editor', action=dict(action='sculpt', settings=settings))
    c.call('save_editor', path='artifacts/integrated.layout.json')
    c.call('load_editor', path='artifacts/integrated.layout.json')
    assert c.call('inspect_editor')['editor']['version'] == 3
    assert c.call('inspect_editor')['editor']['work_light'] == light
    assert abs(c.call('inspect_editor')['editor']['sculpt']['radius']-.47) < .00001
    assert c.call('inspect') == state
    c.call('save_scene', path='artifacts/integrated.scene.json')
    authored = c.call('inspect')['scene']
    c.call('capture_editor', path='artifacts/integrated-editor.png', width=1440, height=900)
    c.call('capture_editor', path='artifacts/integrated-editor-repeat.png', width=1440, height=900)
    assert (ROOT/'artifacts/integrated-editor.png').read_bytes() == (ROOT/'artifacts/integrated-editor-repeat.png').read_bytes()
    assert c.call('inspect')['scene'] == authored
    c.call('edit_editor', action=dict(action='lighting', light=None))
    c.call('capture_editor', path='artifacts/integrated-scene-light.png', width=1440, height=900)
    # Form controls change, but the rendered viewport must use the same light.
    from png_pixels import rgba8
    aw, ah, a=rgba8((ROOT/'artifacts/integrated-editor.png').read_bytes())
    bw, bh, b=rgba8((ROOT/'artifacts/integrated-scene-light.png').read_bytes())
    assert (aw, ah) == (bw, bh)
    assert all(a[(y*aw)*4:(y*aw+900)*4] == b[(y*bw)*4:(y*bw+900)*4]
               for y in range(125, 700))
    assert c.call('inspect')['scene'] == authored
    c.call('load_scene', path='artifacts/integrated.scene.json')
    assert c.call('inspect')['scene'] == authored
    c.call('step', ticks=241)
    controls = c.call('inspect_generators')
    assert controls['state']['generators'][0]['births'] > 0
    c.call('capture', path='artifacts/integrated-live.png', width=800, height=600)
    c.call('save_replay', path='artifacts/integrated.replay.json')
    replay = json.loads((ROOT/'artifacts/integrated.replay.json').read_text())
    assert replay['version'] == REPLAY_VERSION and replay['scene']['version'] == SCENE_VERSION
    from verify_materials import f32
    assert f32(replay['scene']['light']) == light
    assert next(e for e in replay['scene']['entities'] if e['id']==wedge['id'])['geometry'] == wedge['geometry']
    r = c.call('run_replay', path='artifacts/integrated.replay.json')
    assert r['matched'] and r['meshes_matched'] and r['generators_matched'], r
    c.call('capture_replay', replay_path='artifacts/integrated.replay.json',
           path='artifacts/integrated-replay.png', width=800, height=600)
    assert (ROOT/'artifacts/integrated-live.png').read_bytes() == (ROOT/'artifacts/integrated-replay.png').read_bytes()
    checks.append('Support query preserves the world; layout3/sculpt preferences, editor GPU repeat, scene83 and replay87 preserve all merged fronts')
    checks.extend(verify_controls(c))
    return checks


def verify_controls(c):
    c.call('load_scene', path='examples/actor-controls.scene.json')
    sweep=json.loads((ROOT/'examples/profile-sweep.edit.json').read_text())
    piece=sweep['sweeps'][0]
    piece['curves'].insert(0, [[-1,-1,0],[-1,-2/3,0],[-1,-1/3,0],[-1,0,0]])
    piece['spans']=[1,24]
    plan=c.call('plan_mesh_sweep', sweeps=sweep['sweeps'])['plan']
    assert plan['pieces'][0]['spans'][0] == 1
    piece['spans']=plan['pieces'][0]['spans']
    c.call('edit_mesh', edit=sweep)
    chamfer=json.loads((ROOT/'examples/chamfer-boxes.edit.json').read_text())
    state=c.call('inspect')
    preview=c.call('preview_mesh_edit', edit=chamfer)['meshes']
    assert c.call('inspect') == state
    c.call('edit_mesh', edit=chamfer)
    assert c.call('inspect')['scene']['meshes'] == preview
    for instance,asset,position in [('control-molding','molding',[0,2,0]),
                                    ('control-chamfer','beveled-blocks',[1,2,0])]:
        c.call('edit_mesh',edit=dict(kind='put_instance',instance=dict(
            id=instance,asset=asset,position=position,rotation=[0,0,0,1],scale=1,color=[.7,.4,.2])))
    light=copy.deepcopy(c.call('inspect')['scene']['light'])
    light.update(environment=dict(sky=[.6,.75,.95],horizon=[.8,.75,.6],
        ground=[.05,.04,.025],axis=[0,1,0],intensity=.4))
    c.call('set_light',light=light)
    c.call('select_editor_mesh',instance='control-chamfer')
    c.call('save_scene',path='artifacts/controls-editor.scene.json')
    for actor in ['player', 'partner']:
        entity=copy.deepcopy(next(e for e in c.call('inspect')['scene']['entities'] if e['id']==actor))
        entity['character_controller']={}
        c.call('upsert',entity=entity)
    a=c.call('bind_control',actor='player')['control']
    b=c.call('bind_control',actor='partner')['control']
    c.call('set_control_input',control=a,input=dict(x=1,z=0,mode='run'))
    c.call('set_control_input',control=b,input=dict(x=0,z=1,mode='sneak'))
    start=c.call('inspect')
    c.call('step',ticks=15)
    live=c.call('inspect')
    assert live['player_motion'] is None
    motors=c.call('inspect_character_controllers')['controllers']
    assert motors['player']['requested_velocity'][0]>0 and motors['player']['requested_velocity'][1]==0
    assert motors['partner']['requested_velocity'][0]==0 and motors['partner']['requested_velocity'][1]>0
    assert live['runtime_positions']['player'][0] > start['runtime_positions']['player'][0]
    assert live['runtime_positions']['partner'][2] > start['runtime_positions']['partner'][2]
    c.call('capture_editor',path='artifacts/controls-editor.png',width=1440,height=900)
    assert c.call('inspect') == live
    c.call('save_replay',path='artifacts/controls-editor.replay.json')
    replay=json.loads((ROOT/'artifacts/controls-editor.replay.json').read_text())
    assert replay['version']==REPLAY_VERSION and replay['scene']['version']==SCENE_VERSION
    result=c.call('run_replay',path='artifacts/controls-editor.replay.json')
    assert result['matched'] and result['controls_matched'] and result['meshes_matched'] and result['character_controllers_matched']
    c.call('capture',path='artifacts/controls-editor-live.png',width=800,height=600)
    c.call('capture_replay',replay_path='artifacts/controls-editor.replay.json',
           path='artifacts/controls-editor-replay.png',width=800,height=600)
    assert (ROOT/'artifacts/controls-editor-live.png').read_bytes()==(ROOT/'artifacts/controls-editor-replay.png').read_bytes()
    c.call('load_scene',path='artifacts/controls-editor.scene.json')
    before=c.call('inspect')
    c.call('set_control_input',control=a,input=dict(x=1,z=0),expected_error=True)
    assert c.call('inspect')==before
    return ['Two independently controlled terrestrial Player coexist with per-curve Sweep, chamfer preview, environment and editor mesh selection/capture; controls+character controllers+meshes replay and pixels exact; load expires handles atomically']
