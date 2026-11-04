# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3", "pillow>=11,<13"]
# ///
"""Build, calibrate and freeze before independently evaluating bake reuse."""
from pathlib import Path
import argparse
import hashlib
import importlib
import json
import subprocess
import lighting_bake_report as report
parser=argparse.ArgumentParser();parser.add_argument('--use-calibration',action='store_true');args=parser.parse_args()
root=Path(__file__).resolve().parents[1];out=report.OUT;out.mkdir(parents=True,exist_ok=True)
if not args.use_calibration:
    with (out/'build.log').open('w') as log:subprocess.run(['cargo','build','--locked','--release','--example','lighting_bounce'],cwd=root,stdout=log,stderr=subprocess.STDOUT,check=True)
report=importlib.reload(report)
def manifest():return dict(source_sha256=report.hashes(),executable_sha256=hashlib.sha256((root/'target/release/examples/lighting_bounce').read_bytes()).hexdigest())
def run(label,validation=False,final=False):
    extra=['--thin-wall'] if label=='thin-wall' else []
    if validation:extra+=['--bake-validation']
    if final:extra+=['--bake-final']
    phase='final' if final else 'validation' if validation else 'calibration'
    with (out/f'{label}-{phase}.log').open('w') as log:subprocess.run([str(root/'target/release/examples/lighting_bounce'),'--bake-audit',*extra],cwd=root,stdout=log,stderr=subprocess.STDOUT,check=True)
    (out/label/f'{phase}-manifest.json').write_text(json.dumps(manifest(),indent=2));print('Completed',label,phase,flush=True)
if not args.use_calibration:
    for label in ['open','thin-wall']:run(label)
for label in ['open','thin-wall']:assert json.loads((out/label/'calibration-manifest.json').read_text())==manifest(),'Calibration code/binary changed'
calibration={k:report.report_scene(k) for k in ['open','thin-wall']};selection=report.freeze(calibration);print('Frozen',selection,flush=True)
if selection['side'] is None:
    report.write_report(calibration,{},selection)
    raise SystemExit('No baked hybrid meets matched-quality gates; revise calibration and keep heldout untouched')
for label in ['open','thin-wall']:run(label,True)
validation={k:report.report_scene(k,True) for k in ['open','thin-wall']};summary=report.write_report(calibration,validation,selection)
if not all(next(t for t in r['timings'] if t['method']=='hybrid')['eligible'] for d in validation.values() for r in d['reports']):raise SystemExit('Frozen hybrid failed first validation')
summary['heldout_quality_pass']=True
summary['heldout_cohort']='initial camera/scaling/invalidation; exploratory for final bake selection'
(out/'initial-selection.json').write_text(json.dumps(selection,indent=2))
(out/'initial-summary.json').write_text(json.dumps(summary,indent=2))
selection=report.freeze_final(calibration);print('Frozen final policy',selection,flush=True)
# Repeat reuse/invalidation controls with the final selected atlas resolution.
for label in ['open','thin-wall']:run(label,True)
validation={k:report.report_scene(k,True) for k in ['open','thin-wall']}
for label in ['open','thin-wall']:run(label,True,True)
final={k+'-final':report.report_scene(k,True,True) for k in ['open','thin-wall']}
summary=report.write_report(calibration,{**validation,**final},selection)
if not summary['heldout_quality_pass']:raise SystemExit('Frozen final policy failed fresh E/F; no retuning')
print('Bake reuse, scaling and invalidation passed; final E/F passed',flush=True)
