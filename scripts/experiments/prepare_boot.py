"""Build one current boot study; no global installation or browser launch."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root',type=Path,default=Path('/Volumes/Zero/img2threejs/work/ignis-boot'))
    p.add_argument('--reference-crop',type=Path,help='Existing 200x470 reference crop')
    args=p.parse_args();root=args.root.resolve();root.mkdir(parents=True,exist_ok=True)
    project=Path(__file__).resolve().parents[2]
    if args.reference_crop and args.reference_crop.resolve()!=root/'reference.png':
        shutil.copyfile(args.reference_crop,root/'reference.png')
    if not (root/'reference.png').is_file():
        p.error('supply --reference-crop; image evidence is required')
    esbuild=root/'node_modules/.bin/esbuild';three=root/'node_modules/three'
    if not esbuild.is_file() or not three.is_dir():
        p.error(f'Install pinned experiment dependencies: npm install --prefix {root} --save-exact three@0.169.0 esbuild@0.25.12 --no-audit --no-fund')
    if json.loads((three/'package.json').read_text())['version']!='0.169.0':
        p.error('This study is pinned to Three.js 0.169.0')
    aliases=[f'--alias:three={three}/build/three.module.js',f'--alias:three/addons/controls/OrbitControls.js={three}/examples/jsm/controls/OrbitControls.js']
    shutil.copyfile(project/'scripts/experiments/boot.html',root/'index.html')
    subprocess.run([str(esbuild),str(project/'scripts/experiments/boot_viewer.ts'),'--bundle','--format=esm','--target=es2022',*aliases,f'--outfile={root}/viewer.js'],check=True)
    files=['scripts/experiments/boot_model.ts','scripts/experiments/boot_viewer.ts','scripts/vendor/img2threejs/tapered_sweep.ts']
    manifest={'upstream':'img2threejs@3e7099e1b7fe93e187e06db4bdbe1e1906c21a28','three':'0.169.0','esbuild':'0.25.12',
      'sources':{f:hashlib.sha256((project/f).read_bytes()).hexdigest() for f in files},
      'reference_crop_sha256':hashlib.sha256((root/'reference.png').read_bytes()).hexdigest(),
      'scope':'Adapted image-only construction study; not full upstream skill certification.'}
    (root/'build-provenance.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(f'python3 {project}/scripts/experiments/serve_boot.py {root}')


if __name__=='__main__':main()
