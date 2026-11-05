#!/usr/bin/env python3
"""Generate shuffled mesh files, compile in isolated processes, validate offscreen."""
import argparse
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "artifacts/appearance-probe/geometry-compiler"
EXE = ROOT / "target/release/examples/geometry_compiler_probe"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    subprocess.run(["cargo", "build", "--locked", "--release", "--example", "geometry_compiler_probe"], cwd=ROOT, check=True)
    subprocess.run([str(EXE), "--generate", str(OUT), "16" if args.smoke else "64"], cwd=ROOT, check=True)
    for name in ("stone", "folded"):
        for factor in (1, 2, 10, 100):
            output = OUT / f"{name}-{factor}.igc"
            # Peak RSS belongs to this entire compiler process, including validation/encoding.
            process = subprocess.run(["/usr/bin/time", "-l", str(EXE), "--compile", str(OUT / f"{name}-{factor}.igm"), str(output)], cwd=ROOT, text=True, capture_output=True, check=True)
            print(process.stdout, end="", flush=True)
            metrics = json.loads(output.with_suffix(".igc.json").read_text())
            for line in process.stderr.splitlines():
                if "maximum resident set size" in line:
                    metrics["compiler_peak_rss_bytes"] = int(line.split()[0])
            output.with_suffix(".igc.json").write_text(json.dumps(metrics, indent=2) + "\n")
    subprocess.run([str(EXE), "--smoke" if args.smoke else "--run", str(OUT)], cwd=ROOT, check=True)
    print("Results:", OUT)


if __name__ == "__main__":
    main()
