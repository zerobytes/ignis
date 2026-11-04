# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy>=2,<3", "pillow>=11,<13"]
# ///
"""Run calibration, freeze a policy, then evaluate heldout scenes without retuning."""
from pathlib import Path
import argparse
import json
import importlib
import hashlib
import subprocess
import lighting_hybrid_report as report

parser=argparse.ArgumentParser();parser.add_argument("--use-calibration",action="store_true",help="Use already measured calibration files with this source; still freeze before heldout")
args=parser.parse_args();root=Path(__file__).resolve().parents[1];out=report.OUT;out.mkdir(parents=True,exist_ok=True)
if not args.use_calibration:
    with (out/"build.log").open("w") as log:subprocess.run(["cargo","build","--locked","--release","--example","lighting_bounce"],cwd=root,stdout=log,stderr=subprocess.STDOUT,check=True)

report=importlib.reload(report)

def run(label,extra):
    with (out/f"{label}.log").open("w") as log:
        subprocess.run([str(root/"target/release/examples/lighting_bounce"),"--hybrid-audit",*extra],cwd=root,stdout=log,stderr=subprocess.STDOUT,check=True)
    manifest=dict(source_sha256=report.hashes(),executable_sha256=hashlib.sha256((root/"target/release/examples/lighting_bounce").read_bytes()).hexdigest())
    (out/label/"run-manifest.json").write_text(json.dumps(manifest,indent=2))
    print("Completed",label,flush=True)
if not args.use_calibration:
    run("open",[]);run("thin-wall",["--thin-wall"])
if args.use_calibration:
    for label in ["open","thin-wall"]:
        manifest=json.loads((out/label/"run-manifest.json").read_text())
        assert manifest["source_sha256"]==report.hashes(),"Calibration source changed"
        assert manifest["executable_sha256"]==hashlib.sha256((root/"target/release/examples/lighting_bounce").read_bytes()).hexdigest(),"Calibration executable changed"
calibration={label:report.scene_report(label) for label in ["open","thin-wall"]}
selection=report.freeze_selection(calibration)
print("Frozen policy",selection["selected_mode"],selection["selected_name"],flush=True)
run("validation-c",["--validation-c"]);run("validation-d",["--validation-d"])
reports={**calibration,**{label:report.scene_report(label) for label in ["validation-c","validation-d"]}}
summary=report.write_report(reports,selection)
print(json.dumps(dict(selected_mode=selection["selected_mode"],heldout=summary["heldout_quality_pass"]),indent=2))
if summary["effective_mode"] is None:raise SystemExit("Selected and predeclared fallback policies failed heldout; do not silently retune")
