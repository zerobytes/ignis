"""Run one current fixed-content experiment; reuse the existing hybrid source bake."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

root = Path('artifacts/appearance-probe/pixel-density')
root.mkdir(parents=True, exist_ok=True)
inputs = [Path(f'artifacts/appearance-probe/hybrid-scaling/f1_d2.{ext}') for ext in ['json', 'vertices', 'indices', 'normals']]
inputs += [Path(f'examples/appearance_probe/{name}.wgsl') for name in ['triangle_scaling', 'scaling_depth']]
manifest = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
(root / 'input-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
subprocess.run(['cargo', 'run', '--locked', '--release', '--example', 'appearance_probe', '--', '--pixel-density'], check=True)
for script in ['check_pixel_density.py', 'report_pixel_density.py']:
    subprocess.run([sys.executable, f'scripts/{script}'], check=True)
subprocess.run(['node', 'scripts/verify_pixel_density.mjs'], check=True)
