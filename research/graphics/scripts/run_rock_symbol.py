"""Run the current surface-symbol experiment without altering the source rock."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

root = Path('artifacts/appearance-probe/rock-symbol')
root.mkdir(parents=True, exist_ok=True)
paths = [Path('artifacts/appearance-probe/assets/rock')/f for f in ['vertices.bin','indices.bin','asset.json']]
paths += [Path('examples/appearance_probe')/f for f in ['rock_symbol.wgsl','rock_symbol_build.rs']]
manifest = {str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
(root/'input-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
subprocess.run(['cargo','run','--locked','--release','--example','appearance_probe','--','--rock-symbol'],check=True)
for script in ['check_rock_symbol.py','report_rock_symbol.py']:
    subprocess.run([sys.executable,f'scripts/{script}'],check=True)
subprocess.run(['node','scripts/verify_rock_symbol.mjs'],check=True)
