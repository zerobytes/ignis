#!/usr/bin/env python3
"""Build the website's pinned catalog snapshot using the native Ignis exporter."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def refresh_meshes(catalog, data, revision, cache):
    """Preserve a deployed material/shader snapshot while adding mesh releases."""
    destination=ROOT/'website/dist/catalog/data'
    previous=json.loads((destination/'catalog.json').read_text())
    old={a['id']:a for a in previous['assets']}
    fresh_materials={a['id'] for a in data['assets'] if a['kind']=='material'}
    if fresh_materials!={a['id'] for a in previous['assets'] if a['kind']=='material'}:
        raise RuntimeError('Material set changed; use a full catalog build')
    records=[];runtime={}
    for asset in data['assets']:
        if asset['kind']=='material':
            prior=old[asset['id']].copy();prior.pop('preview',None)
            if prior!=asset:raise RuntimeError('Material metadata changed; use a full catalog build')
            records.append(old[asset['id']]);continue
        if asset['kind']!='mesh' or asset['native_preview']['format']!='IGMP1+gzip':
            raise RuntimeError('Unsupported mesh-only catalog record')
        packet=next(f for f in asset['files'] if f['media_type']=='application/vnd.ignis.mesh-preview+gzip')
        thumb=next(f for f in asset['files'] if f['media_type']=='image/jpeg')
        payload=(catalog/thumb['path']).read_bytes()
        if len(payload)!=thumb['bytes'] or sha(payload)!=thumb['sha256']:raise RuntimeError('Thumbnail integrity failure')
        (destination/'thumbs'/f"{asset['id']}.jpg").write_bytes(payload)
        relative=f"catalog/data/meshes/{asset['id']}.igmpz"
        runtime[relative]={k:packet[k] for k in ['url','bytes','sha256']}
        asset['preview']=dict(path=f"data/meshes/{asset['id']}.igmpz",packet_bytes=packet['bytes'],packet_sha256=packet['sha256'],decoded_bytes=asset['native_preview']['decoded_bytes'],decoded_sha256=asset['native_preview']['decoded_sha256'],thumbnail_sha256=thumb['sha256'])
        if cache and (cache/f"{asset['id']}.igmpz").is_file():
            local=cache/f"{asset['id']}.igmpz"
            if local.stat().st_size!=packet['bytes'] or sha(local.read_bytes())!=packet['sha256']:raise RuntimeError('Mesh cache integrity failure')
            (destination/'meshes').mkdir(exist_ok=True)
            shutil.copyfile(local,destination/'meshes'/local.name)
        records.append(asset)
    previous.update(assets=records,source_revision=revision,source_repository='https://github.com/zerobytes/ignis-catalog')
    (destination/'runtime-files.json').write_text(json.dumps(dict(version=1,files=runtime),indent=2)+'\n')
    (destination/'catalog.json').write_text(json.dumps(previous,indent=2,ensure_ascii=False)+'\n')
    print(f'Refreshed {len(runtime)} mesh records; existing material packets and renderer shader preserved')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--catalog', type=Path, required=True)
    parser.add_argument('--mesh-cache', type=Path, help='Verified local release packets; otherwise hydrate from public Releases')
    parser.add_argument('--exporter', type=Path, default=ROOT / 'target/release/examples/export_web_materials')
    parser.add_argument('--meshes-only',action='store_true',help='Refresh mesh releases while preserving deployed material packets and shader')
    args = parser.parse_args()
    catalog = args.catalog.resolve()
    subprocess.run(['python3', str(catalog / 'tools/catalog.py'), 'build', '--check'], check=True)
    if subprocess.check_output(['git', 'status', '--porcelain'], cwd=catalog).strip():
        raise RuntimeError('Catalog checkout must be committed and clean')
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=catalog, text=True).strip()
    data = json.loads((catalog / 'catalog.json').read_text())
    records = data['assets']
    if not 1 <= len(records) <= 500:
        raise RuntimeError('Unsupported catalog size')
    if args.meshes_only:
        refresh_meshes(catalog,data,revision,args.mesh_cache)
        return
    stage = ROOT / 'website/.catalog-stage'
    stage.mkdir()  # Stable staging path also refuses overlapping builders.
    try:
        output = stage / 'data'
        (output / 'sources').mkdir(parents=True)
        (output / 'thumbs').mkdir()
        inputs = []
        runtime = {}
        for asset in records:
            if asset['license'] != 'CC-BY-4.0':
                raise RuntimeError('Explicit CC BY asset license required')
            if asset['kind'] == 'mesh':
                if asset['native_preview']['format'] != 'IGMP1+gzip':
                    raise RuntimeError('Unsupported mesh preview')
                preview = next(f for f in asset['files'] if f['media_type'] == 'application/vnd.ignis.mesh-preview+gzip')
                thumbnail = next(f for f in asset['files'] if f['media_type'] == 'image/jpeg')
                image = (catalog / thumbnail['path']).resolve()
                if not image.is_relative_to(catalog) or len(image.read_bytes()) != thumbnail['bytes'] or sha(image.read_bytes()) != thumbnail['sha256']:
                    raise RuntimeError('Mesh thumbnail integrity failure')
                shutil.copyfile(image, output / 'thumbs' / f'{asset["id"]}.jpg')
                relative = f'catalog/data/meshes/{asset["id"]}.igmpz'
                runtime[relative] = {key: preview[key] for key in ['url', 'bytes', 'sha256']}
                asset['preview'] = dict(path=f'data/meshes/{asset["id"]}.igmpz', packet_bytes=preview['bytes'], packet_sha256=preview['sha256'], decoded_bytes=asset['native_preview']['decoded_bytes'], decoded_sha256=asset['native_preview']['decoded_sha256'], thumbnail_sha256=thumbnail['sha256'])
                if args.mesh_cache:
                    cached = args.mesh_cache / f'{asset["id"]}.igmpz'
                    if cached.stat().st_size != preview['bytes'] or sha(cached.read_bytes()) != preview['sha256']:
                        raise RuntimeError('Local release packet differs from catalog')
                    (output / 'meshes').mkdir(exist_ok=True)
                    shutil.copyfile(cached, output / 'meshes' / cached.name)
                continue
            if asset['kind'] != 'material':
                raise RuntimeError('Preview1 requires a supported, explicitly licensed material')
            source_record = next(f for f in asset['files'] if f['role'] == 'source')
            source = (catalog / source_record['path']).resolve()
            if not source.is_relative_to(catalog):
                raise RuntimeError('Escaping catalog source')
            payload = source.read_bytes()
            if len(payload) != source_record['bytes'] or sha(payload) != source_record['sha256']:
                raise RuntimeError('Catalog source integrity failure')
            asset_id = asset['id']
            source_copy = output / 'sources' / f'{asset_id}.ignis-material'
            source_copy.write_bytes(payload)
            inputs.append({'id': asset_id, 'path': str(source_copy)})
            family = next(t for t in asset['tags'] if t in {'basic', 'marble', 'rock'})
            stem = source.stem
            if family == 'basic':
                image = ROOT / f'artifacts/material-recipes/artifacts/detail-{stem.replace("-", "_")}.png'
                gallery_source = ROOT / f'artifacts/material-recipes/library/{stem.replace("-", "_")}-0.ignis-material'
            else:
                image = ROOT / f'artifacts/{family}/artifacts/{stem}.png'
                gallery_source = ROOT / f'artifacts/{family}/library/{source.name}'
            if json.loads(gallery_source.read_bytes()) != json.loads(payload):
                raise RuntimeError(f'Gallery source differs from catalog: {asset_id}')
            target = output / 'thumbs' / f'{asset_id}.jpg'
            subprocess.run(['sips', '-Z', '320', '-s', 'format', 'jpeg', '-s', 'formatOptions', '82', str(image), '--out', str(target)], check=True, stdout=subprocess.DEVNULL)
            recipe = json.loads(payload)['material']['procedural']['recipe']
            asset['preview'] = {'plane': recipe['plane'], 'thumbnail_source': str(image.relative_to(ROOT)), 'thumbnail_source_sha256': sha(image.read_bytes()), 'thumbnail_sha256': sha(target.read_bytes())}
        input_file = stage / 'input.json'
        input_file.write_text(json.dumps(inputs))
        subprocess.run([str(args.exporter.resolve()), str(input_file), str(output)], check=True)
        shader = (ROOT / 'src/scene.wgsl').read_text() + '\n' + (ROOT / 'src/particles.wgsl').read_text()
        if (output / 'ignis.wgsl').read_text() != shader:
            raise RuntimeError('Exporter was compiled with a different shader; rebuild it')
        for asset in records:
            if asset['kind'] != 'material':
                continue
            packet = (output / f'{asset["id"]}.bin').read_bytes()
            asset['preview'].update(packet_sha256=sha(packet), packet_bytes=len(packet))
        (output / 'packets.json').unlink()
        native_files = ['src/meshes/preview.rs', 'src/materials.rs', 'src/materials/preview.rs', 'src/materials/recipe.rs', 'src/materials/marble.rs', 'src/materials/rock.rs', 'src/scene.wgsl', 'src/particles.wgsl']
        data.update(source_revision=revision, source_repository='https://github.com/zerobytes/ignis-catalog', shader_sha256=sha(shader.encode()), renderer_source={'repository': 'https://github.com/zerobytes/ignis', 'local_snapshot': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'source_state': 'working-tree', 'source_sha256': {p: sha((ROOT / p).read_bytes()) for p in native_files}, 'scope': 'native-material-and-static-mesh-shader-webgpu-host'})
        (output / 'runtime-files.json').write_text(json.dumps({'version': 1, 'files': runtime}, indent=2) + '\n')
        (output / 'catalog.json').write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')
        shutil.copyfile(catalog / 'LICENSES/CC-BY-4.0.txt', output / 'LICENSE.txt')
        shutil.copyfile(ROOT / 'LICENSE', output / 'engine-license.txt')
        destination = ROOT / 'website/dist/catalog/data'
        if destination.exists():
            shutil.rmtree(destination)
        output.rename(destination)
        total = sum(p.stat().st_size for p in destination.rglob('*') if p.is_file())
        print(f'Built {len(records)} assets from catalog {revision}: {total:,} bytes')
    finally:
        shutil.rmtree(stage)


if __name__ == '__main__':
    main()
