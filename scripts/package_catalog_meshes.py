#!/usr/bin/env python3
"""Export canonical trees into verified, full-source release packages (no binary Git files)."""
import argparse
import gzip
import hashlib
import json
import re
from pathlib import Path
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT.parent / 'ignis-catalog'
OUTPUT = ROOT / 'artifacts/catalog-meshes'
TAG = 'trees-v1.0.0'


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def record(path, role, media):
    return dict(role=role, media_type=media, bytes=path.stat().st_size, sha256=sha(path))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inputs', type=Path, default=ROOT / 'examples/catalog-trees.json')
    parser.add_argument('--only', help='Export only this stable asset id')
    parser.add_argument('--asset-version', default='1.0.0')
    parser.add_argument('--release-tag', default=TAG)
    args = parser.parse_args()
    if not re.fullmatch(r'\d+\.\d+\.\d+', args.asset_version):
        parser.error('Expected a semantic asset version')
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]*', args.release_tag):
        parser.error('Expected an immutable release tag')
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    entries = json.loads(args.inputs.read_text())
    if args.only:
        entries = [item for item in entries if item['id'] == args.only]
        if not entries:
            parser.error('Asset id is not in the input manifest')
    for item in entries:
        asset_id = item['id']
        item['path'] = str((ROOT / item['path']).resolve())
        item['materials'] = {k: str((ROOT / p).resolve()) for k, p in item['materials'].items()}
        source = Path(item['path'])
        originals = [source, *map(Path, item['materials'].values())]
        before = {str(p): sha(p) for p in originals}
        for p in originals:
            rel = p.relative_to(ROOT).as_posix()
            subprocess.run(['git', 'ls-files', '--error-unmatch', rel], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
            if subprocess.check_output(['git', 'status', '--porcelain', '--', rel], cwd=ROOT).strip():
                raise RuntimeError('Canonical source must be tracked and clean')
        single = OUTPUT / 'one.json'
        single.write_text(json.dumps([item]))
        subprocess.run([str(ROOT / 'target/release/examples/export_web_meshes'), str(single), str(OUTPUT / 'current')], check=True)
        directory = OUTPUT / 'current' / asset_id
        full = json.loads((directory / 'mesh.json').read_text())
        preview = json.loads((directory / 'preview.json').read_text())
        target = CATALOG / 'assets/meshes/trees' / asset_id.removeprefix('ignis.mesh.tree.')
        target.mkdir(parents=True, exist_ok=True)
        subprocess.run(['sips', '-Z', '320', '-s', 'format', 'jpeg', '-s', 'formatOptions', '85', str(directory / 'thumbnail.png'), '--out', str(target / 'thumbnail.jpg')], check=True, stdout=subprocess.DEVNULL)
        packed = OUTPUT / f'{asset_id}.igmpz'
        raw = directory / 'preview.igmp'
        decoded = record(raw, 'preview', 'application/vnd.ignis.mesh-preview')
        with raw.open('rb') as src, packed.open('wb') as dst, gzip.GzipFile(fileobj=dst, mode='wb', compresslevel=6, mtime=0) as compressed:
            shutil.copyfileobj(src, compressed)
        preview_record = record(packed, 'preview', 'application/vnd.ignis.mesh-preview+gzip')
        base = f'https://github.com/zerobytes/ignis-catalog/releases/download/{args.release_tag}/'
        preview_record['url'] = base + packed.name
        files = {p.name: p for p in originals}
        files.update({p: directory / p for p in ['mesh.glb', 'tree.scene.json']})
        files['LICENSE.txt'] = CATALOG / 'LICENSES/CC-BY-4.0.txt'
        package_hashes = {name: record(p, 'source', 'application/octet-stream') for name, p in files.items()}
        readme = f'''# {item['name']}

Copyright Guilherme Oliveira / Zerobytes. CC BY 4.0.
Credit: "{item['name']} by Guilherme Oliveira / Zerobytes, CC BY 4.0 — https://github.com/zerobytes/ignis-catalog".

Open tree.scene.json in a compatible Ignis editor for the full native mesh,
botanical source binding, original settings and native materials. The separate
.ignis-botany file is the original editable recipe and organ graph, unchanged.
Birch includes its original .ignis-material files. No reference images or vendor
geometry are included. These are authored procedural assets, not botanical simulations.

mesh.glb contains full positions, smooth normals, indices, transforms and simple
portable PBR colors. Native procedural surface patterns remain in the Ignis scene
and material files; GLB colors approximate them. The browser preview uses lower
tessellation (axis8 / sides6 / blade4), retaining all source organs. Full download
geometry: {sum(p['vertices'] for p in full['parts']):,} vertices / {sum(p['indices']//3 for p in full['parts']):,} triangles.

Source repository https://github.com/zerobytes/ignis
Snapshot {revision}
Source {source.relative_to(ROOT).as_posix()}
Botanical recipe format {json.loads(source.read_text())['version']}; use an Ignis editor
that supports this format to regenerate or edit the native botanical binding.
Browser preview and portable GLB do not execute botanical growth.
The code/editor has its own license; this asset's CC BY license does not relicense it.
'''
        archive = OUTPUT / f'{asset_id}.zip'
        with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, compresslevel=6) as out:
            for name, p in files.items():
                out.write(p, name)
            out.writestr('README.md', readme)
            out.writestr('checksums.json', json.dumps(package_hashes, indent=2) + '\n')
        with zipfile.ZipFile(archive) as check:
            if check.testzip() is not None:
                raise RuntimeError('Invalid ZIP')
            for name, expected in package_hashes.items():
                h = hashlib.sha256()
                with check.open(name) as stream:
                    for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                        h.update(chunk)
                if h.hexdigest() != expected['sha256']:
                    raise RuntimeError('ZIP member integrity failure')
        if before != {str(p): sha(p) for p in originals}:
            raise RuntimeError('Source changed during export')
        download = record(archive, 'source', 'application/zip')
        download['url'] = base + archive.name
        thumb = record(target / 'thumbnail.jpg', 'preview', 'image/jpeg')
        thumb['path'] = (target / 'thumbnail.jpg').relative_to(CATALOG).as_posix()
        def stats(metadata):
            return dict(vertices=sum(p['vertices'] for p in metadata['parts']), triangles=sum(p['indices']//3 for p in metadata['parts']), bounds=metadata['bounds'])
        manifest = dict(format_version=1, id=asset_id, version=args.asset_version, kind='mesh', name=item['name'], summary='Editable native botanical tree with full mesh, source recipe and materials.', author=dict(name='Guilherme Oliveira / Zerobytes', url='https://github.com/zerobytes'), license='CC-BY-4.0', attribution=f"{item['name']} by Guilherme Oliveira / Zerobytes — CC BY 4.0 — https://github.com/zerobytes/ignis-catalog", tags=['trees','mesh','botany',asset_id.rsplit('.',1)[-1]], provenance=dict(repository='https://github.com/zerobytes/ignis',commit=revision,path=source.relative_to(ROOT).as_posix(),source_sha256=before[str(source)],availability='local-unpublished',note='The tracked source snapshot was local at transfer time; this release includes its complete licensed source. No reference images included.'), compatibility=dict(engine='ignis',runtime_validation='Native bake, scene validation and native renderer capture. Static preview only; botanical physics is not run in the browser.',botany_recipe_version=json.loads(source.read_text())['version']), mesh=stats(full), native_preview=dict(format='IGMP1+gzip',quality='reduced-tessellation',tessellation=preview['tessellation'],decoded_bytes=decoded['bytes'],decoded_sha256=decoded['sha256'],**stats(preview)), files=[download,preview_record,thumb])
        (target / 'asset.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n')
        print(f'Packaged {asset_id}: zip={download["bytes"]:,} preview={preview_record["bytes"]:,} full={manifest["mesh"]} preview_mesh={stats(preview)}', flush=True)
        shutil.rmtree(directory)  # Retain one current ZIP and web packet; no redundant raw build copies.


if __name__ == '__main__':
    main()
