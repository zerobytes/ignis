#!/usr/bin/env python3
"""Native material-only junction proof on frozen final geometry, no product-scene edits."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--asset', type=Path, required=True)
    parser.add_argument('--union', type=Path, required=True)
    parser.add_argument('--scene', type=Path, required=True)
    parser.add_argument('--materials-from', type=Path)
    parser.add_argument('--material-range', type=int, nargs=2, metavar=('START', 'COUNT'))
    parser.add_argument('--material-offset', type=float, nargs=3)
    parser.add_argument('--max-change', type=float, default=1)
    args = parser.parse_args()
    assert bool(args.material_range) == bool(args.material_offset)
    root = Path(__file__).resolve().parents[1]/'artifacts'/('tree-material-frame' if args.material_range else 'tree-material-finish')
    (root/'artifacts').mkdir(parents=True, exist_ok=True)
    (root/'examples').mkdir(exist_ok=True)
    os.environ['IGNIS_VERIFY_ROOT'] = str(root)
    from verify_lab import Client
    asset_data = args.asset.read_bytes()
    original = json.loads(asset_data)
    asset = original.get('asset', original)
    display_before = copy.deepcopy(asset)
    union_data = args.union.read_bytes()
    union = json.loads(union_data)
    source_geometry = union['source']['geometry']
    n = len(source_geometry['positions'])
    end = len(asset['geometry']['positions'])-sum(map(len, union['ports']))
    pairs = []
    cursor = end
    for port in union['ports']:
        for index in reversed(port):
            pairs.append([index, cursor])
            cursor += 1
    assert asset['geometry']['triangles'][:len(source_geometry['triangles'])] == source_geometry['triangles']
    # These exact source->guard pairs were written by the union, not fitted by distance.
    for s, t in pairs:
        assert asset['geometry']['material_coordinates'][s] == asset['geometry']['material_coordinates'][t]
    if args.material_range:
        start, count = args.material_range
        assert count > 0 and 0 <= start < start+count <= n
        for m in asset['geometry']['material_coordinates'][start:start+count]:
            for k in range(3):
                m[k] += args.material_offset[k]
    plan = {'retained_vertices': n, 'retained_faces': len(source_geometry['triangles']),
            'iso_vertices': end-n, 'pairs': pairs}
    options = {'reference': {'columns': [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
                            'offset': union['options']['material_offset']},
               'iterations': 64, 'tolerance': 0.002, 'max_change': args.max_change, 'max_anisotropy': 10000}
    client = Client(args.binary.resolve(), False)
    call = client.call
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                       'clientInfo': {'name': 'tree-material-finish', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        # Explicit authoring in this isolated study root canonicalizes f32 values;
        # the input/canonical files on disk are never edited.
        call('edit_mesh', edit={'kind': 'put_asset', 'asset': asset})
        asset = call('inspect_mesh', asset=asset['id'])['asset']
        state = call('inspect')
        result = call('preview_mesh_material_finish', source=asset, plan=plan, options=options)
        assert call('inspect') == state
        assert call('preview_mesh_material_finish', source=asset, plan=plan, options=options) == result
        output = result['asset']
        for channel in ['positions', 'triangles']:
            assert output['geometry'][channel] == asset['geometry'][channel]
        assert output['geometry']['material_coordinates'][:n] == asset['geometry']['material_coordinates'][:n]
        for channel in ['rig', 'clips', 'material', 'shading']:
            assert output.get(channel) == asset.get(channel)
        (root/'artifacts/input.json').write_text(json.dumps({'source': asset, 'plan': plan, 'options': options}))
        (root/'artifacts/output.json').write_text(json.dumps(result))
        regions = {name: [] for name in ['source', 'guard', 'zipper', 'iso']}
        for face, triangle in enumerate(asset['geometry']['triangles']):
            kinds = tuple(sorted({0 if i<n else 1 if i<end else 2 for i in triangle}))
            name = {(0,): 'source', (0, 2): 'guard', (1, 2): 'zipper', (1,): 'iso'}[kinds]
            regions[name].append(face)
        report = {'build_id': state['build_id'], 'asset_sha256': hashlib.sha256(asset_data).hexdigest(),
                  'union_sha256': hashlib.sha256(union_data).hexdigest(), 'options': options,
                  'geometry_and_retained_material_exact': True, 'finish': result['report'], 'xy': {}, 'frequency': {}}
        report['retained_material_exact_reference'] = 'explicitly_authored_input' if args.material_range else 'original_input'
        if args.material_range:
            report['authoring_layer'] = {'range': args.material_range, 'translation': args.material_offset,
                                        'original_retained_material_intentionally_changed': count,
                                        'canonical_input_file_unchanged': hashlib.sha256(args.asset.read_bytes()).hexdigest() == report['asset_sha256']}
            report['original_domain_before_authoring'] = {
                name: call('preview_mesh_material_domain', source=display_before['geometry'], faces=faces)['report']
                for name, faces in regions.items() if faces}
        scene = json.loads(args.scene.read_bytes())
        assert scene['version'] <= state['scene']['version'], 'Do not validate a future authoring format'
        material = None
        if args.materials_from:
            material_data = args.materials_from.read_bytes()
            tree = json.loads(material_data)
            material = next(a['material'] for a in tree['meshes']['assets'] if a['id'] == 'branch-editable-1')
            report['material_scene_sha256'] = hashlib.sha256(material_data).hexdigest()
        for label, a in [('before', display_before), ('after', output)]:
            report['xy'][label] = {name: call('preview_mesh_material_domain', source=a['geometry'], faces=faces,
                                 options={'domain': 'xy'})['report'] for name, faces in regions.items() if faces}
            if material and material.get('pattern'):
                report['frequency'][label] = {name: call('preview_mesh_material_domain', source=a['geometry'], faces=faces,
                                              options={'scale': material['pattern']['frequency']})['report']
                                              for name, faces in regions.items() if faces}
            for mode in ['neutral', 'authored']:
                s = copy.deepcopy(scene)
                rendered = copy.deepcopy(a)
                rendered['material'] = {} if mode == 'neutral' else material or a.get('material', {})
                s['meshes']['assets'] = [rendered]
                assert len(s['meshes']['instances']) == 1
                s['meshes']['instances'][0]['asset'] = a['id']
                (root/f'examples/{label}-{mode}.scene.json').write_text(json.dumps(s))
                call('load_scene', path=f'examples/{label}-{mode}.scene.json')
                call('capture', path=f'artifacts/{label}-{mode}.png', width=1100, height=1100)
        assert (root/'artifacts/before-neutral.png').read_bytes() == (root/'artifacts/after-neutral.png').read_bytes()
        report['neutral_pixels_exact'] = True
        report['authored_pixels_changed'] = (root/'artifacts/before-authored.png').read_bytes() != (root/'artifacts/after-authored.png').read_bytes()
        assert args.asset.read_bytes() == asset_data and args.union.read_bytes() == union_data
        if args.materials_from:
            assert args.materials_from.read_bytes() == material_data
        report['canonical_input_files_unchanged'] = True
        report['passed'] = True
        (root/'report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'report': str(root/'report.json'), 'build_id': state['build_id'],
                          'guard_singular_before': result['report']['before']['guard']['near_singular_faces'],
                          'guard_singular_after': result['report']['after']['guard']['near_singular_faces']}))
    finally:
        client.close()


if __name__ == '__main__':
    main()
