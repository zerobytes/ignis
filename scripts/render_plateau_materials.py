#!/usr/bin/env python3
"""Native material, all-around strata and orbit reviews through MCP offscreen."""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT
from open_earth_walk import init
from relief_compositions import position, ZERO
from verify_plateau_shaping import VIEWS

OUT = ROOT / 'artifacts/plateau-relief/materials'
PREFIX = 'artifacts/plateau-relief/materials'


def fingerprint(geometry):
    shape = {k: geometry[k] for k in ['positions', 'triangles']}
    return hashlib.sha256(json.dumps(shape, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def grass(seed, ground=False):
    return {'pattern': {'frequency': [.03, .03, .03] if ground else [.4, .4, .4],
        'low': [.07, .14, .025] if ground else [.045, .16, .025],
        'high': [.15, .23, .055] if ground else [.17, .32, .045],
        'bump': .0003 if ground else .004,
        'warp': .35, 'octaves': 3, 'seed': seed, 'contrast': .6 if ground else 1.1, 'kind': 'noise'},
        'reflectance': {'roughness': .95, 'f0': .025}, 'transmission': 0}


def main():
    global OUT, PREFIX
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--strata', action='store_true')
    parser.add_argument('--orbit-frames', type=int, default=0)
    parser.add_argument('--shading', choices=['flat', 'smooth'], default='flat')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if not 0 <= args.orbit_frames <= 48:
        parser.error('Orbit supports0..48 frames')
    if args.output or args.strata:
        OUT = (args.output or ROOT / 'artifacts/plateau-strata').resolve()
        PREFIX = str(OUT.relative_to(ROOT))
    # Static performance review: explicit cold authoring only; existing materials,
    # lighting and shadow pass. Grass reuses selected roof faces through native
    # RemoveFaces, not a new procedural mesh or a per-frame geometry generator.
    # Runtime rock is per visible fragment; no fixed1024 maps or extra builds.
    OUT.mkdir(parents=True, exist_ok=True)
    c = Client(ROOT / 'target/release/ignis', log_path=OUT / 'native.log')
    try:
        init(c)
        schema = json.dumps(c.request('tools/list')['result'])
        assert all(x in schema for x in ['publish_landform', 'set_material', 'remove_faces'])
        c.call('generate', recipe={'width': 20, 'depth': 20, 'seed': 221, 'obstacles': 0})
        c.call('edit_scene', edits=[{'op': 'remove', 'id': e['id']} for e in c.call('inspect')['scene']['entities']])
        if args.strata:
            from plateau_strata import document
            params = json.loads((ROOT / 'examples/plateau-strata.recipe.json').read_text())
            doc = document(params)
            assert doc == document(params)
        else:
            params = None
            doc = json.loads((ROOT / 'examples/plateau-wall.recipe.json').read_text())
        doc = c.call('query_landform', query={'kind': 'inspect', 'document': doc})['document']
        c.call('save_landform', path=f'{PREFIX}/plateau.ignis-landform', document=doc)
        assert c.call('load_landform', path=f'{PREFIX}/plateau.ignis-landform')['document'] == doc
        c.call('publish_landform', publication={'document': doc, 'asset': 'rock', 'instance': 'rock',
            'frame_origin': ZERO, 'color': [1, 1, 1], 'shading': args.shading, 'collision': True})
        original = c.call('inspect_mesh', asset='rock')['asset']
        rock_hash = fingerprint(original['geometry'])
        topology = c.call('inspect_mesh_topology', asset='rock')['topology']
        assert topology['boundary_edges'] == topology['non_manifold_edges'] == 0
        assert len(c.call('inspect_mesh_islands', asset='rock')['topology']['islands']) == 1
        probes = None
        if args.strata:
            rays = []
            sides = [('front',[0,0,-160],[0,0,1]), ('rear',[0,0,160],[0,0,-1]),
                     ('left',[-160,0,0],[1,0,0]), ('right',[160,0,0],[-1,0,0])]
            for _, origin, direction in sides:
                for y in [5,11,17,23,29,35]:
                    rays.append({'origin':[origin[0],y,origin[2]],'direction':direction,'max_distance_meters':320})
            hits = c.call('query_landform', query={'kind':'raycast','document':doc,'rays':rays})['hits']
            probes = {}
            for i,(side,_,_) in enumerate(sides):
                distances = [h[0]['distance_meters'] for h in hits[i*6:(i+1)*6]]
                assert max(distances)-min(distances) > .75, (side,distances)
                probes[side] = distances
            sphere = copy.deepcopy(doc)
            sphere['origin'] = position(0,6371000,0)
            sphere['domain'] = {'kind':'sphere','center':ZERO,'radius_meters':6371000,'forward':[0,0,1]}
            mapped = c.call('query_landform', query={'kind':'compile','document':sphere})
            assert all(p['closed_oriented'] for p in mapped['parts'])
        # Apply an editable copy of the existing Rock2 source, at landscape scale.
        material = copy.deepcopy(json.loads((ROOT / 'assets/materials/rock/alpine.ignis-material').read_text())['material'])
        recipe = material['procedural']['recipe']
        recipe['tile_size'] = [24, 24]
        recipe['rock'].update(dark=[.09, .11, .13], light=[.35, .38, .4],
                              contrast=1.1, oxidation=.05, grain=.55, grain_frequency=384,
                              joint_depth=.14, detail_height=.35, weathering=.45)
        c.call('edit_mesh', edit={'kind': 'set_material', 'asset': 'rock', 'material': material})
        # Explicit oblique planar mapping avoids a rank-one XY projection on
        # end faces. This changes material coordinates only, not surface shape.
        coordinates = [[x+.55*z, y+.2*z, z] for x,y,z in original['geometry']['positions']]
        c.call('edit_mesh', edit={'kind': 'set_material_coordinates', 'asset': 'rock', 'coordinates': coordinates})
        # A separate thin visual skin is a native subset of existing flat top
        # faces. The rock positions, topology and collision remain untouched.
        turf = copy.deepcopy(original)
        turf['id'] = 'turf'
        turf['material'] = grass(221)
        c.call('edit_mesh', edit={'kind': 'put_asset', 'asset': turf})
        geometry = original['geometry']
        removed = [i for i, face in enumerate(geometry['triangles'])
                   if not all(geometry['positions'][v][1] >= 39.999 for v in face)]
        assert 0 < len(removed) < len(geometry['triangles'])
        reference = c.call('inspect_mesh_topology', asset='turf')['reference']
        c.call('edit_mesh', edit={'kind': 'remove_faces', 'asset': 'turf', 'reference': reference, 'faces': removed})
        instance = copy.deepcopy(next(i for i in c.call('inspect_meshes')['instances'] if i['id'] == 'rock'))
        instance.update(id='turf', asset='turf', position=[0, .045, 0])
        c.call('edit_mesh', edit={'kind': 'put_instance', 'instance': instance})
        floor = {'version': 1, 'space_version': 1, 'id': 'floor', 'revision': 1, 'datum': ZERO,
                 'base': {'kind': 'constant', 'height_meters': -.03}, 'operations': []}
        c.call('publish_terrain', publication={'document': floor,
            'patch': {'origin': position(-4096, 0, -4096), 'cells': [16, 16], 'spacing_meters': 512},
            'stride': 1, 'asset': 'floor', 'instance': 'floor', 'frame_origin': ZERO,
            'color': [1, 1, 1], 'collision': False})
        c.call('edit_mesh', edit={'kind': 'set_material', 'asset': 'floor', 'material': grass(934, ground=True)})
        light = {'direction': [.4, -.65, .8], 'ambient': .22, 'direct_color': [1, .94, .82],
            'direct_intensity': 1.15,
            'environment': {'sky': [.52, .66, .86], 'horizon': [.64, .69, .65],
                            'ground': [.2, .22, .13], 'axis': [0, 1, 0], 'intensity': .65},
            'shadow': {'center': [0, 20, -5], 'extent': 130, 'resolution': 2048,
                       'bias': .0006, 'receiver_plane_bias': True},
            'atmosphere': {'density': .00025, 'reduce_details': False, 'color': [.52, .68, .84],
                           'sun_color': [1, .92, .78], 'sun_strength': .1}}
        c.call('set_light', light=light)
        views = {'overview': VIEWS['oblique'],
                 'detail': {'position': [90, 42, -110], 'target': [18, 23, -28], 'fov_degrees': 44}}
        if args.orbit_frames:
            views.update(front=VIEWS['front'], left={'position':[-175,65,-15],'target':[0,20,0],'fov_degrees':48},
                         right=VIEWS['side'], rear=VIEWS['rear'])
        for name, camera in views.items():
            c.call('set_camera', camera=camera)
            c.call('capture', path=f'{PREFIX}/{name}.png', width=1600, height=1000)
            print('captured', name, flush=True)
        orbit = []
        frames = OUT / 'frames'
        frames.mkdir(exist_ok=True)
        # Replace this task's generated frame set when the requested count changes.
        for old in frames.glob('frame-*.png'):
            old.unlink()
        for i in range(args.orbit_frames):
            a = math.tau*i/args.orbit_frames
            camera = {'position':[180*math.sin(a),76,-180*math.cos(a)],
                      'target':[0,19,0],'fov_degrees':44}
            c.call('set_camera', camera=camera)
            c.call('capture', path=f'{PREFIX}/frames/frame-{i:03}.png', width=960, height=600)
            orbit.append(camera)
            if i%6 == 0: print('orbit',i,'/',args.orbit_frames,flush=True)
        c.call('set_camera', camera=views['overview'])
        assert fingerprint(c.call('inspect_mesh', asset='rock')['asset']['geometry']) == rock_hash
        c.call('save_scene', path=f'{PREFIX}/plateau.scene.json')
        c.call('load_scene', path=f'{PREFIX}/plateau.scene.json')
        c.call('capture', path=f'{PREFIX}/reloaded.png', width=1600, height=1000)
        assert (OUT / 'reloaded.png').read_bytes() == (OUT / 'overview.png').read_bytes()
        report = {'build': c.call('inspect')['build_id'], 'rock_shape_unchanged': True,
                  'rock_shape_sha256': rock_hash, 'views': views, 'light': light,
                  'rock_material': material, 'grass_faces': len(geometry['triangles'])-len(removed),
                  'material_mapping': 'u=x+0.55z, v=y+0.2z; explicit planar coordinates, not triplanar',
                  'grass_vertical_offset_meters': .045, 'native_window': False,
                  'save_reload_pixels_equal': True, 'orbit':orbit,'parameters':params,
                  'vertices':len(original['geometry']['positions']),
                  'triangles':len(original['geometry']['triangles']), 'side_probes':probes,
                  'shading':args.shading}
        (OUT / 'render.json').write_text(json.dumps(report, indent=2))
        print(json.dumps({'rendered': True, 'shape_unchanged': True, 'build': report['build']}))
    finally:
        c.close()


if __name__ == '__main__':
    main()
