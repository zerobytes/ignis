# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3", "pillow>=11,<13"]
# ///
"""Run the fixed regional bake dependency/repair experiment."""
from pathlib import Path
import hashlib
import importlib
import json
import subprocess
import lighting_regional_report as report
root=Path(__file__).resolve().parents[1];out=report.OUT;out.mkdir(parents=True,exist_ok=True)
with (out/'build.log').open('w') as log:subprocess.run(['cargo','build','--locked','--release','--example','lighting_bounce'],cwd=root,stdout=log,stderr=subprocess.STDOUT,check=True)
report=importlib.reload(report)
manifest=dict(source_sha256=report.hashes(),executable_sha256=hashlib.sha256((root/'target/release/examples/lighting_bounce').read_bytes()).hexdigest())
for label,extra in [('open',[]),('thin-wall',['--thin-wall'])]:
    with (out/f'{label}.log').open('w') as log:subprocess.run([str(root/'target/release/examples/lighting_bounce'),'--regional-audit',*extra],cwd=root,stdout=log,stderr=subprocess.STDOUT,check=True)
    (out/label/'manifest.json').write_text(json.dumps(manifest,indent=2));print('Completed',label,flush=True)
report.write_report({k:report.scene_report(k) for k in ['open','thin-wall']})
