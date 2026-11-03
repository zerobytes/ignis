#!/usr/bin/env python3
"""Run MCP regressions serially in an isolated workspace; no window by default."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
# Dependency order matters: several scenarios load earlier authored examples.
SCENARIOS = (
    'wind',
    'wind_vortex',
    'planet',
    'planet_reference',
    'player_perspectives',
    'terrain_region',
    'terrain_collision',
    'landforms',
    'terrain',
    'world_regions',
    'world_space',
    'npc_air_circulation',
    'portable_save',
    'npc_checkpoint',
    'npc_population_scale',
    'npc_presentation',
    'npc_presentation_complete',
    'npc_routines_interactions',
    'npc_tactics_cooperation',
    'npc_circulation',
    'projects',
    'project_browser',
    'editor_mesh',
    'sculpt_brushes',
    'editor_authoring',
    'editor_mesh_selection',
    'editor_transform',
    'mesh_modeling',
    'editor',
    'mesh_material_domain',
    'mesh_material_finish',
    'triplanar_details',
    'detail_domains',
    'lab',
    'npc',
    'perception',
    'render',
    'search',
    'hearing',
    'vision_guides',
    'hearing_guides',
    'footsteps',
    'locomotion',
    'patrol',
    'route_guides',
    'scenarios',
    'replay_inspection',
    'triggers',
    'scene_edits',
    'selection',
    'npc_autonomy',
    'targeting',
    'recognition',
    'sound_attention',
    'objectives',
    'territory',
    'nav_recovery',
    'passages',
    'mobility',
    'partial_routes',
    'local_connections',
    'flight',
    'spatial_vision',
    'spatial_patrol',
    'walk_support',
    'communication',
    'abilities',
    'vitality',
    'ability_policy',
    'cast_movement',
    'projectiles',
    'free_aim',
    'npc_aim',
    'periodic',
    'dispel',
    'channel',
    'interrupt',
    'attributes',
    'damage',
    'outgoing',
    'area',
    'persistent_area',
    'aura',
    'triage',
    'dispel_triage',
    'resurrection',
    'identity',
    'identity_bindings',
    'world_origins',
    'population',
    'spawn',
    'generators',
    'waves',
    'generator_control',
    'generator_activation',
    'spawn_regions',
    'template_selection',
    'spawn_support',
    'geometry',
    'actor_controls',
    'local_transactions',
    'continuous_runtime',
    'network',
    'remote_controls',
    'character_controller',
    'ground_navigation',
    'ground_local_connections',
    'npc_mind',
    'npc_hybrid_mobility',
    'npc_navigation_generation',
    'mesh_sweep',
    'chamfer_boxes',
    'meshes',
    'mesh_panels',
    'mesh_storage',
    'mesh_surface_tools',
    'plants',
    'mesh_topology',
    'plant_sections',
    'mesh_shading',
    'directional_shadows',
    'materials',
    'layered_patterns',
    'metallic',
    'environment',
    'atmosphere',
    'mesh_normals',
    'landmark_projection',
    'mesh_organic',
    'curved_plants',
    'smooth_plants',
    'mesh_attachments',
    'mesh_islands',
    'mesh_island_transform',
    'surface_details',
    'plant_trajectories',
    'reflectance',
    'mesh_bridges',
    'mesh_junction_union',
    'plant_adoption',
    'plant_assemblies',
    'plant_material_frames',
    'plant_connections',
    'image_similarity',
)
WORKSPACE = ROOT / "artifacts/mcp-validation"


def save_report(report):
    path = WORKSPACE / "report.json"
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(report, indent=2) + "\n")
    temporary.replace(path)


def run(binary, selected, window):
    discovered = {p.stem.removeprefix("verify_") for p in (ROOT / "scripts").glob("verify_*.py")}
    if discovered != set(SCENARIOS):
        raise RuntimeError("Update SCENARIOS dependency order for added/removed verification scripts")
    shutil.copytree(ROOT / "examples", WORKSPACE / "examples", dirs_exist_ok=True)
    (WORKSPACE / "artifacts").mkdir(exist_ok=True)
    logs = WORKSPACE / "logs"
    logs.mkdir(exist_ok=True)
    env = {**os.environ, "IGNIS_VERIFY_ROOT": str(WORKSPACE)}
    report = {"status": "running", "passed": False, "native_window": window,
              "binary": str(binary), "workspace": str(WORKSPACE),
              "selected": selected, "results": [], "checks": 0}
    save_report(report)
    try:
        for name in selected:
            command = [sys.executable, str(ROOT / f"scripts/verify_{name}.py"),
                       "--binary", str(binary)]
            if window:
                command.append("--window")
            print(f"Running {name} ({'window' if window else 'headless'})", flush=True)
            started = time.monotonic()
            output = logs / f"{name}.stdout.log"
            error = logs / f"{name}.stderr.log"
            with output.open("w") as stdout, error.open("w") as stderr:
                result = subprocess.run(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
            # The client uses one current runtime log; retain it per scenario, not per attempt.
            runtime = WORKSPACE / "artifacts/runtime.log"
            if runtime.exists():
                shutil.copyfile(runtime, logs / f"{name}.runtime.log")
            entry = {"scenario": name, "returncode": result.returncode,
                     "seconds": round(time.monotonic() - started, 3),
                     "stdout": str(output), "stderr": str(error), "passed": False}
            if result.returncode == 0:
                try:
                    if output.stat().st_size > 1024 * 1024:
                        raise ValueError("Scenario summary exceeds 1 MiB")
                    summary = json.loads(output.read_text())
                    count = summary["checks"]
                    if summary.get("passed") is not True or type(count) is not int or count <= 0:
                        raise ValueError("Invalid scenario result")
                    entry.update(passed=True, checks=count)
                    report["checks"] += count
                except (ValueError, KeyError, TypeError) as exc:
                    entry["error"] = str(exc)
            report["results"].append(entry)
            save_report(report)
            print(f"  {'PASS' if entry['passed'] else 'FAIL'} {name}", flush=True)
            if not entry["passed"]:
                report["status"] = "failed"
                break
        else:
            report.update(status="completed", passed=True)
    except BaseException as exc:
        report.update(status="interrupted", error=str(exc))
        raise
    finally:
        save_report(report)
    print(f"Report: {WORKSPACE / 'report.json'}", flush=True)
    return 0 if report["passed"] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--only", nargs="+", choices=SCENARIOS,
                        help="Run selected scenarios in dependency order; seed from checked-in examples")
    parser.add_argument("--window", action="store_true",
                        help="Explicit desktop integration run; requires --only and may take focus")
    args = parser.parse_args()
    if args.window and not args.only:
        parser.error("--window requires --only; normal full regression runs without windows")
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"Build the executable first: {binary}")
    selected = [name for name in SCENARIOS if not args.only or name in args.only]
    WORKSPACE.mkdir(parents=True, exist_ok=True)
    with (WORKSPACE / ".lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            parser.error("Another verification suite is using artifacts/mcp-validation")
        return run(binary, selected, args.window)


if __name__ == "__main__":
    sys.exit(main())
