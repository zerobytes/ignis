#!/usr/bin/env python3
"""Native finite solid subtraction through MCP; no window or Python triangulation."""
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from open_earth_walk import init

OUT = ROOT / 'artifacts/rock-chipping'
PREFIX = 'artifacts/rock-chipping'


def box_points(lo, hi):
    return [[hi[a] if i & (1 << a) else lo[a] for a in range(3)] for i in range(8)]


def volume(asset):
    points = asset['geometry']['positions']
    total = 0
    for f in asset['geometry']['triangles']:
        a, b, c = [points[i] for i in f]
        total += (a[0]*(b[1]*c[2]-b[2]*c[1]) + a[1]*(b[2]*c[0]-b[0]*c[2])
                  + a[2]*(b[0]*c[1]-b[1]*c[0]))/6
    return total


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    c = Client(ROOT/'target/release/ignis', log_path=OUT/'native.log')
    checks = []
    try:
        init(c)
        assert 'subtract_convex' in json.dumps(c.request('tools/list')['result'])
        c.call('generate', recipe=dict(width=12, depth=12, seed=221, obstacles=0))
        c.call('create_editor_mesh', primitive='cube', resolution=4, size=2)
        instance = c.call('inspect_editor')['selected_mesh']

        def asset():
            return c.call('inspect_mesh', asset=instance)['asset']

        def reference():
            return c.call('inspect_mesh_topology', asset=instance)['reference']

        def apply(operation, preview=False, ref=None, **extra):
            return c.call('model_editor_mesh', instance=instance, reference=ref or reference(),
                          operation=operation, preview=preview, **extra)

        first = dict(kind='subtract_convex', points=box_points([.2,-.7,.4],[1.2,.6,1.3]))
        second = dict(kind='subtract_convex', points=box_points([-.8,-.4,.7],[.6,.3,1.4]))
        original = c.call('inspect')
        old_ref = reference()
        preview = apply(first, preview=True)
        assert c.call('inspect') == original
        apply(first)
        assert c.call('inspect')['scene'] == preview['scene']
        assert abs(volume(asset())-7.376)<2e-5
        apply(second)
        assert abs(volume(asset())-7.166)<2e-5
        final = c.call('inspect')
        assert final['history']['undo'] == original['history']['undo']+2
        topology = c.call('inspect_mesh_topology', asset=instance)['topology']
        assert topology['boundary_edges'] == topology['non_manifold_edges'] == 0
        c.call('undo'); c.call('undo')
        assert c.call('inspect')['scene'] == original['scene']
        c.call('redo'); c.call('redo')
        assert c.call('inspect')['scene'] == final['scene']
        checks.append('Typed MCP cuts have analytic volumes 7.376 and 7.166, pure preview parity, closed topology and exact undo/redo')
        frozen = c.call('inspect')
        apply(first, ref=old_ref, expected_error=True)
        for points in [box_points([-2]*3,[2]*3), box_points([-.1,-2,-2],[.1,2,2]),
                       box_points([-.1]*3,[.1]*3), [[0,0,0]]*4]:
            apply(dict(kind='subtract_convex', points=points), expected_error=True)
            assert c.call('inspect') == frozen
        edit = dict(kind='model_selection', asset=instance, reference=reference(),
                    operation=dict(kind='subtract_convex', points=box_points([-1.2,-.6,-1.2],[-.5,.4,-.5])))
        c.call('edit_scene', edits=[dict(op='mesh',edit=edit),dict(op='mesh',edit=edit)], expected_error=True)
        assert c.call('inspect') == frozen
        c.call('play')
        frozen = c.call('inspect')
        apply(first, expected_error=True)
        assert c.call('inspect') == frozen
        c.call('pause')
        checks.append('Stale references, empty/disconnected/cavity/degenerate cuts, a late failed batch and edits during play are atomic')

        # Actual KCC settling on a cut collider, separate from geometry-only queries.
        c.call('generate', recipe=dict(width=12,depth=12,seed=221,obstacles=0))
        c.call('create_editor_mesh', primitive='cube', resolution=4, size=2)
        instance = c.call('inspect_editor')['selected_mesh']
        inst = next(i for i in c.call('inspect_meshes')['instances'] if i['id']==instance)
        inst['position']=[0,0,0]
        c.call('edit_mesh',edit=dict(kind='put_instance',instance=inst))
        player = copy.deepcopy(next(e for e in c.call('inspect')['scene']['entities'] if e['id']=='player'))
        player['transform']=dict(position=[0,3,0],size=[.2,.4,.2])
        player['character_controller']={}
        c.call('edit_scene', edits=[dict(op='upsert',entity=player),dict(op='set_mesh_collision',instance=instance,enabled=True)])
        c.call('step',ticks=90)
        before = c.call('inspect')
        assert abs(before['runtime_positions']['player'][1]-1.2)<.03
        apply(dict(kind='subtract_convex',points=box_points([-.6,.2,-.6],[.6,1.3,.6])))
        c.call('step',ticks=90)
        after = c.call('inspect')
        assert abs(after['runtime_positions']['player'][1]-.4)<.03,after['runtime_positions']['player']
        assert after['character_controllers']['player']['grounded']
        c.call('save_replay',path=f'{PREFIX}/collision.replay.json')
        replay = c.call('run_replay',path=f'{PREFIX}/collision.replay.json')
        assert replay['matched'] and replay['meshes_matched']
        c.call('save_scene',path=f'{PREFIX}/collision.scene.json')
        scene = c.call('inspect')['scene']
        c.call('load_scene',path=f'{PREFIX}/collision.scene.json')
        assert c.call('inspect')['scene'] == scene
        checks.append('Native mesh collider rebuild: real KCC settles at Y1.2 before and Y0.4 after a top chip, grounded; replay and scene persistence match')

        # Editable recipe contains native source and cutter points, never a Python mesh.
        recipe=json.loads((ROOT/'examples/rock-chipping.recipe.json').read_text())
        c.call('generate',recipe=dict(width=20,depth=20,seed=221,obstacles=0))
        c.call('edit_scene',edits=[dict(op='remove',id=e['id']) for e in c.call('inspect')['scene']['entities']])
        c.call('publish_landform',publication=dict(document=recipe['base'],asset='rock',instance='rock',
              frame_origin=recipe['base']['origin'],color=[1,1,1],shading='flat',collision=True))
        instance='rock'
        volumes=[volume(asset())]
        reports=[]
        for operation in recipe['operations']:
            result=apply(operation)
            reports.append(result['modeling_report'])
            volumes.append(volume(asset()))
        assert all(b<a for a,b in zip(volumes,volumes[1:]))
        material=copy.deepcopy(json.loads((ROOT/'assets/materials/rock/alpine.ignis-material').read_text())['material'])
        rock=material['procedural']['recipe']
        rock['tile_size']=[12,12]
        rock['rock'].update(dark=[.12,.13,.14],light=[.43,.44,.45],contrast=.8,
                            oxidation=.04,grain=.4,grain_frequency=240,joint_depth=.03,detail_height=.1,weathering=.3)
        c.call('edit_mesh',edit=dict(kind='set_material',asset=instance,material=material))
        geometry=asset()['geometry']
        c.call('edit_mesh',edit=dict(kind='set_material_coordinates',asset=instance,
               coordinates=[[x+.4*z,y+.3*z,z] for x,y,z in geometry['positions']]))
        c.call('set_light',light=dict(direction=[.35,-.7,.8],ambient=.26,direct_intensity=1.1,
               direct_color=[1,.95,.86],shadow=dict(center=[0,8,0],extent=40,resolution=2048,bias=.0005,receiver_plane_bias=True)))
        for name,camera in recipe['views'].items():
            c.call('set_camera',camera=camera)
            c.call('capture',path=f'{PREFIX}/{name}.png',width=1280,height=900)
        c.call('set_camera',camera=recipe['views']['oblique'])
        c.call('save_scene',path=f'{PREFIX}/rock.scene.json')
        c.call('load_scene',path=f'{PREFIX}/rock.scene.json')
        c.call('capture',path=f'{PREFIX}/reloaded.png',width=1280,height=900)
        assert (OUT/'reloaded.png').read_bytes()==(OUT/'oblique.png').read_bytes()
        topology=c.call('inspect_mesh_topology',asset=instance)['topology']
        assert topology['boundary_edges']==topology['non_manifold_edges']==0
        checks.append('Editable native block plus finite cutters, material presentation, three offscreen views and scene reload PNG equality')
        report=dict(build=c.call('inspect')['build_id'],checks=checks,passed=True,native_window=False,
                    volumes=volumes,operations=reports,views=recipe['views'],
                    vertices=len(geometry['positions']),triangles=len(geometry['triangles']),
                    scope=dict(tool=True,reference_fidelity=False,world_integration=False,human_acceptance=False))
        (OUT/'verification.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2))
    finally:
        c.close()


if __name__=='__main__':
    main()
