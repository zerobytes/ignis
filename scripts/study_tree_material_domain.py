#!/usr/bin/env python3
"""Read-only snapshot study. All metrics come from the native MCP diagnostic."""
import argparse
import hashlib
import json
import os
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--after', type=Path, required=True)
    parser.add_argument('--junction-before', type=Path)
    parser.add_argument('--junction-after', type=Path)
    parser.add_argument('--junction-source', type=Path,
                        help='Union input with canonical retained source and three ports; enables provenance partitions')
    parser.add_argument('--load-after', action='store_true', help='Also validate the after Scene with this binary in the isolated study process')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]/'artifacts/tree-material-domain'
    (root/'artifacts').mkdir(parents=True, exist_ok=True)
    os.environ['IGNIS_VERIFY_ROOT'] = str(root)
    from verify_lab import Client
    client = Client(args.binary.resolve(), False)
    call = client.call
    report = {'snapshots': {}, 'junction': {}, 'note': 'Local-space derivatives; original scene/material/geometry files unchanged. Optional load uses only the isolated study process.'}
    try:
        client.request('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                       'clientInfo': {'name': 'tree-material-study', 'version': '1'}})
        client.request('notifications/initialized', notification=True)
        state = call('inspect')
        report['build_id'] = state['build_id']
        for label, path in [('before', args.before), ('after', args.after)]:
            data = path.read_bytes()
            scene = json.loads(data)
            snapshot = {'sha256': hashlib.sha256(data).hexdigest(), 'path': str(path.resolve()), 'assets': {}}
            for asset in scene['meshes']['assets']:
                if asset['id'] not in ['branch-editable-1', 'wood', 'secondary-06']:
                    continue
                result = {}
                options = [('spatial', {}), ('detail_xy', {'domain': 'xy'})]
                pattern = asset.get('material', {}).get('pattern')
                if pattern:
                    options.append(('pattern_frequency', {'scale': pattern['frequency']}))
                for name, option in options:
                    result[name] = call('preview_mesh_material_domain', source=asset['geometry'], options=option)['report']
                result['vertices'] = len(asset['geometry']['positions'])
                result['triangles'] = len(asset['geometry']['triangles'])
                snapshot['assets'][asset['id']] = result
            report['snapshots'][label] = snapshot
        for label, path in [('before', args.junction_before), ('after', args.junction_after)]:
            if path:
                data = path.read_bytes()
                asset = json.loads(data)
                if 'asset' in asset:
                    asset = asset['asset']
                metrics = {domain: call('preview_mesh_material_domain', source=asset['geometry'],
                           options={'domain': domain})['report'] for domain in ['spatial', 'xy']}
                report['junction'][label] = {'sha256': hashlib.sha256(data).hexdigest(), 'metrics': metrics}
                if args.junction_source:
                    source_data = args.junction_source.read_bytes()
                    fixture = json.loads(source_data)
                    source = fixture['source']['geometry']
                    n = len(source['positions'])
                    guard_count = sum(map(len, fixture['ports']))
                    iso_end = len(asset['geometry']['positions']) - guard_count
                    assert iso_end > n
                    assert asset['geometry']['triangles'][:len(source['triangles'])] == source['triangles']
                    regions = {name: [] for name in ['retained_source', 'iso', 'first_guard', 'zipper']}
                    for face, triangle in enumerate(asset['geometry']['triangles']):
                        kinds = {0 if v < n else 1 if v < iso_end else 2 for v in triangle}
                        name = {(0,): 'retained_source', (1,): 'iso', (0, 2): 'first_guard', (1, 2): 'zipper'}.get(tuple(sorted(kinds)))
                        assert name, (face, triangle, kinds)
                        regions[name].append(face)
                    report['junction'][label]['source_sha256'] = hashlib.sha256(source_data).hexdigest()
                    report['junction'][label]['regions'] = {
                        name: {domain: call('preview_mesh_material_domain', source=asset['geometry'], faces=faces,
                                           options={'domain': domain})['report'] for domain in ['spatial', 'xy']}
                        for name, faces in regions.items()}
        assert call('inspect') == state
        if args.load_after:
            examples = root/'examples'
            examples.mkdir(exist_ok=True)
            (examples/'source.scene.json').write_bytes(args.after.read_bytes())
            call('load_scene', path='examples/source.scene.json')
            loaded = call('inspect')
            for asset, expected in report['snapshots']['after']['assets'].items():
                actual = call('inspect_mesh_material_domain', asset=asset)['report']
                assert actual == expected['spatial']
            assert call('inspect') == loaded
            report['loaded_scene_version'] = loaded['scene']['version']
            report['asset_inspection_matches_preview'] = True
        (root/'report.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps({'passed': True, 'report': str(root/'report.json'), 'build_id': report['build_id']}))
    finally:
        client.close()


if __name__ == '__main__':
    main()
