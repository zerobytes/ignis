#!/usr/bin/env python3
"""Build, verify or open the native Land of Survivors character front end."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

PROJECT = Path(__file__).resolve().parent
ENGINE = PROJECT.parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", action="store_true")
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--open", action="store_true")
    parser.add_argument(
        "--play", action="store_true",
        help="Open the selected character directly in the offline stage",
    )
    parser.add_argument("--export-animation-scene", metavar="RELATIVE_PATH",
                        help="Export a native scene for editing clips through the Ignis editor/MCP")
    parser.add_argument("--animation-preview",action="store_true",help="Open the unsaved animation stage with live clip reload")
    args = parser.parse_args()
    binary = ENGINE / "target/release/examples/land_of_survivors_app"
    out = PROJECT / "artifacts/app"
    out.mkdir(parents=True, exist_ok=True)
    if args.build:
        source = ENGINE / "artifacts/character-base/human-nose.ignis-character"
        if not source.is_file():
            for script in [
                "prepare_source", "extend_source", "refine_source",
                "smooth_source", "cupid_source", "nose_source",
            ]:
                subprocess.run(
                    [sys.executable, str(ENGINE / f"scripts/character/{script}.py")],
                    cwd=ENGINE, check=True,
                )
        subprocess.run(
            ["cargo", "build", "--locked", "--release", "--example", "land_of_survivors_app"],
            cwd=ENGINE, check=True,
        )
    if not binary.is_file():
        parser.error("Build first: python3 games/land-of-survivors/app.py --build")
    if args.export_animation_scene:
        subprocess.run([str(binary), "--export-animation-scene", args.export_animation_scene],cwd=ENGINE,check=True)
        return
    if args.verify:
        subprocess.run([str(binary), "--verify"], cwd=ENGINE, check=True)
        return
    if args.build and not (args.open or args.play or args.animation_preview):
        return
    with (out / "window.log").open("w") as log:
        process = subprocess.Popen(
            [str(binary)] + (["--animation-preview"] if args.animation_preview else ["--play"] if args.play else []), cwd=ENGINE,
            stdin=subprocess.DEVNULL, stdout=log, stderr=log, start_new_session=True,
        )
    (out / "launch.json").write_text(json.dumps({
        "pid": process.pid,
        "binary": str(binary),
        "profile_override": os.environ.get("LOS_PROFILE_PATH"),
        "entry_scene": "scenes/Inicio.scene.json",
        "play_selected": args.play,
    }, indent=2) + "\n")
    print(f"Land of Survivors opened (PID {process.pid}).")


if __name__ == "__main__":
    main()
