#!/usr/bin/env python3
"""Export catalog previews from the exact native scenes in published source ZIPs.

No recipe regeneration, tessellation override or representative-color conversion.
Only the preview records are updated; published editable source archives stay pinned.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import zipfile

ROOT=Path(__file__).resolve().parents[1]

def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
    return h.hexdigest()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--catalog',type=Path,required=True)
    parser.add_argument('--only')
    parser.add_argument('--jobs',type=int,choices=range(1,5),default=1)
    parser.add_argument('--release-tag',default='native-previews-v2')
    args=parser.parse_args()
    root=args.catalog.resolve();out=ROOT/'artifacts/catalog-fidelity';out.mkdir(exist_ok=True)
    exporter=ROOT/'target/release/examples/export_catalog_assets';build=sha(exporter)
    stamp=out/'exporter.sha256'
    if not stamp.exists() or stamp.read_text().strip()!=build:
        (out/'ignis.wgsl').unlink(missing_ok=True)
        stamp.write_text(build+'\n')
    def export_one(manifest):
        asset=json.loads(manifest.read_text())
        if asset['kind']!='mesh' or args.only and asset['id']!=args.only:return
        ident=asset['id'];source=next(f for f in asset['files'] if f['role']=='source' and f['media_type']=='application/zip')
        archive=next((p for p in [ROOT/'artifacts/catalog-meshes'/(ident+'.zip'),ROOT/'artifacts/catalog-ancestral'/(ident+'.zip')] if p.is_file() and sha(p)==source['sha256']),None)
        if archive is None:raise ValueError(f'Pinned source ZIP not in current local cache: {ident}')
        receipt=out/(ident+'.json')
        if receipt.exists():
            result=json.loads(receipt.read_text())
            if result['source_sha256']==source['sha256'] and result['exporter_sha256']==build and all((out/p['file']).is_file() and sha(out/p['file'])==p['packet_sha256'] for p in result['parts']):
                print(f'Reusing exact preview {ident}',flush=True)
            else:result=None
        else:result=None
        if result is None:
            stage=out/(ident+'.export');stage.mkdir() # Refuse to overlap another exporter.
            try:
                scene=stage/'tree.scene.json'
                with zipfile.ZipFile(archive) as z,z.open('tree.scene.json') as src,scene.open('wb') as dst:shutil.copyfileobj(src,dst)
                subprocess.run([str(exporter),str(scene),str(stage)],check=True)
                metas=json.loads((stage/'parts.json').read_text());parts=[]
                low=[min(m['bounds']['min'][i] for m in metas) for i in range(3)];high=[max(m['bounds']['max'][i] for m in metas) for i in range(3)]
                for i,meta in enumerate(metas):
                    raw=stage/f'part-{i}.igmp';name=f'{ident}.native-part-{i}.igmpz';packed=out/name
                    with raw.open('rb') as src,packed.open('wb') as dst,gzip.GzipFile(fileobj=dst,mode='wb',mtime=0,compresslevel=6) as gz:shutil.copyfileobj(src,gz)
                    parts.append(dict(file=name,packet_bytes=packed.stat().st_size,packet_sha256=sha(packed),decoded_bytes=raw.stat().st_size,decoded_sha256=sha(raw),vertices=meta['parts'][0]['vertices'],triangles=meta['parts'][0]['indices']//3))
                shader=stage/'ignis.wgsl'
                if (out/'ignis.wgsl').exists() and sha(out/'ignis.wgsl')!=sha(shader):raise ValueError('Mixed native shader builds')
                if not (out/'ignis.wgsl').exists():shutil.copyfile(shader,out/'ignis.wgsl')
                for file in stage.glob('environment-*.bin'):
                    if not (out/file.name).exists():shutil.copyfile(file,out/file.name)
                result=dict(source_sha256=source['sha256'],exporter_sha256=build,parts=parts,vertices=sum(p['vertices'] for p in parts),triangles=sum(p['triangles'] for p in parts),bounds=dict(min=low,max=high))
                if result['triangles']!=asset['mesh']['triangles']:raise ValueError(f'Saved scene triangle count differs from catalog: {ident}')
                receipt.write_text(json.dumps(result,indent=2)+'\n')
            finally:shutil.rmtree(stage)
        result['shader_sha256']=sha(out/'ignis.wgsl')
        receipt.write_text(json.dumps(result,indent=2)+'\n')
        base=f'https://github.com/zerobytes/ignis-catalog/releases/download/{args.release_tag}/'
        asset['native_preview']=dict(format='IGMP2+gzip-parts',quality='full-native',material_appearance='native-materials',parts=result['parts'],vertices=result['vertices'],triangles=result['triangles'],bounds=result['bounds'],source_sha256=source['sha256'],shader_sha256=result['shader_sha256'])
        asset['files']=[f for f in asset['files'] if f['media_type']!='application/vnd.ignis.mesh-preview+gzip']
        for part in result['parts']:asset['files'].append(dict(role='preview',media_type='application/vnd.ignis.mesh-preview+gzip',bytes=part['packet_bytes'],sha256=part['packet_sha256'],url=base+part['file']))
        manifest.write_text(json.dumps(asset,indent=2,ensure_ascii=False)+'\n')
        print(f"Exact {ident}: {result['triangles']:,} triangles; {sum(p['packet_bytes'] for p in result['parts'])/1024**2:.1f} MiB",flush=True)

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        list(pool.map(export_one,sorted((root/'assets').rglob('asset.json'))))

if __name__=='__main__':main()
