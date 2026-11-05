"""Compile, calibrate once, freeze, validate independently, then repeat measurements."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
R=Path('artifacts/appearance-probe/tiled-field');R.mkdir(parents=True,exist_ok=True)
B=Path(os.environ.get('CARGO_TARGET_DIR','target'))/'release/examples/appearance_probe'
subprocess.run(['cargo','build','--locked','--release','--example','appearance_probe'],check=True)
paths=list(Path('examples/appearance_probe').glob('*tiled*'))+list(Path('examples/appearance_probe').glob('*hierarchical*'))+list(Path('examples/appearance_probe').glob('geometric_field.*'))+[Path('examples/appearance_probe/regional_scaling_render.rs'),Path('examples/appearance_probe/triangle_scaling.wgsl'),Path('examples/appearance_probe/scaling_depth.wgsl'),Path('artifacts/appearance-probe/automatic-shading/model.json')]
for d in range(4):
 paths.append(Path(f'artifacts/appearance-probe/geometric-field/f1_d{d}.igf'))
 paths.append(Path(f'artifacts/appearance-probe/hierarchical-field/f1_d{d}.ihf'))
 for ext in ['json','vertices','indices','normals']:paths.append(Path(f'artifacts/appearance-probe/hybrid-scaling/f1_d{d}.{ext}'))
 for variant in ['packed','spatial']:
  for ext in ['positions','indices']:paths.append(Path(f'artifacts/appearance-probe/near-plateau/f1_d{d}.{variant}.{ext}'))
paths.extend([Path('examples/appearance_probe/pixel_density.rs'),Path('examples/appearance_probe/regional_scaling.rs'),Path('examples/appearance_probe.rs')])
manifest={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
(R/'input-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
for stage in ['calibrate']:
 subprocess.run([B,f'--{stage}-tiled-field'],check=True)
frozen=hashlib.sha256((R/'policy.bin').read_bytes()).hexdigest()
(R/'frozen-policy.sha256').write_text(frozen+'\n')
for stage in ['capture','motion','depth','bench','repeat','audit','fixed']:
 subprocess.run([B,f'--{stage}-tiled-field'],check=True)
 assert hashlib.sha256((R/'policy.bin').read_bytes()).hexdigest()==frozen
for script in ['report_tiled_field.py','check_tiled_field.py']:subprocess.run([sys.executable,'scripts/'+script],check=True)


subprocess.run(['node','scripts/check_tiled_field_ui.mjs'],check=True)
