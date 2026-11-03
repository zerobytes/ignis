#!/usr/bin/env python3
"""Author and verify straight/curved/holding flight through real MCP."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def actor(identifier, z, color, mode):
    profile = {"mode": "straight"} if mode == "straight" else {
        "mode": "curved", "acceleration": 3, "deceleration": 3,
        "turn_degrees_per_second": 45, "smoothing_seconds": 0.4,
        "minimum_speed": 1 if mode == "holding" else 0,
        "can_hover": mode != "holding", "initial_direction": [0, 0, -1]}
    return {"id": identifier, "name": identifier, "body": "npc", "color": color,
            "transform": {"position": [-5, 3.8, z], "size": [0.6, 1.6, 0.6]},
            "npc": {"surface": "floor", "speed": 3, "stop_distance": 0.3, "enabled": True,
                    "mobility": {"walk": False, "jump_speed": None, "fly_speed": 3, "glide": None},
                    "flight": profile, "local_connections": {"walk": False, "fly": True, "max_distance": 8},
                    "objectives": [{"priority": 1, "when": "always", "action": {"kind": "guard_3d", "point": [2, 5.8, z], "arrival_distance": 0.3}}]}}


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "ignis-flight-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        schema = json.dumps(c.request("tools/list"))
        assert all(p in schema for p in ["turn_degrees_per_second", "smoothing_seconds", "minimum_speed", "can_hover"])
        c.call("generate", recipe={"width": 24, "depth": 20, "obstacles": 0, "seed": 42})
        player = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["transform"]["position"] = [-7, 0.8, 7]
        actors = [actor("straight", -4, [0.2, 0.6, 1], "straight"), actor("curved", 0, [0.8, 0.2, 0.9], "curved"), actor("holding", 7, [0.3, 0.85, 0.4], "holding")]
        pillar = {"id": "pillar", "name": "pillar", "body": "fixed", "color": [0.5, 0.6, 0.65], "transform": {"position": [0, 1.5, 0], "size": [1.5, 3, 1.5]}}
        edits = [{"op": "upsert", "entity": e} for e in [player, pillar] + actors]
        edits.append({"op": "set_camera", "camera": {"position": [-12, 14, 18], "target": [0, 3, 0], "fov_degrees": 50}})
        c.call("edit_scene", edits=edits)
        c.call("set_debug_view", npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)
        c.call("select_entity", id="curved")
        c.call("save_scene", path="examples/npc-flight.scene.json")
        c.call("step", ticks=80)
        middle = c.call("inspect")
        assert middle["mobility"]["straight"]["flight"] is None
        assert middle["mobility"]["curved"]["flight"] is not None, middle["mobility"]
        assert middle["runtime_positions"]["curved"][2] < -0.3, middle["runtime_positions"]
        assert abs(middle["runtime_positions"]["straight"][2] + 4) < 1e-5
        c.call("capture", path="artifacts/flight-curves.png", width=1280, height=900)
        c.call("save_replay", path="artifacts/flight-curves.replay.json")
        assert c.call("run_replay", path="artifacts/flight-curves.replay.json")["matched"]
        c.call("capture_replay", replay_path="artifacts/flight-curves.replay.json", path="artifacts/flight-curves-replay.png", width=1280, height=900)
        assert (ROOT / "artifacts/flight-curves.png").read_bytes() == (ROOT / "artifacts/flight-curves-replay.png").read_bytes()
        assert c.call("inspect_replay", path="artifacts/flight-curves.replay.json", tick=80)["snapshot"]["mobility"] == middle["mobility"]
        checks.append("MCP authors profiles, curved XYZ motion differs from straight, full replay/prefix/PNG match")

        c.call("step", ticks=720)
        end = c.call("inspect")
        assert all(end["mobility"][name]["phase"] == "arrived" for name in ["straight", "curved", "holding"]), end["mobility"]
        assert end["mobility"]["holding"]["flight"]["holding"]
        parked = end["runtime_positions"]["curved"]
        old_orbit = end["runtime_positions"]["holding"]
        c.call("step", ticks=60)
        after = c.call("inspect")
        assert after["runtime_positions"]["curved"] == parked
        assert sum((a-b)**2 for a,b in zip(old_orbit, after["runtime_positions"]["holding"])) > 0.1
        assert sum(v*v for v in after["mobility"]["holding"]["flight"]["velocity"]) >= 0.99
        orbit_ticks = math.ceil(2 * math.pi / after["mobility"]["holding"]["flight"]["orbit_angle_per_tick"])
        remaining = orbit_ticks * 2
        while remaining:
            ticks = min(remaining, 3600)
            c.call("step", ticks=ticks)
            remaining -= ticks
        stable = c.call("inspect")
        assert stable["mobility"]["holding"]["flight"]["holding"], stable["mobility"]
        assert stable["mobility"]["holding"]["completed"] == 1
        assert stable["runtime_positions"]["curved"] == parked, {"parked": parked, "current": stable["runtime_positions"], "events": c.call("events")}
        c.call("capture", path="artifacts/flight-holding.png", width=1280, height=900)
        c.call("save_replay", path="artifacts/flight-holding.replay.json")
        assert c.call("run_replay", path="artifacts/flight-holding.replay.json")["matched"]
        checks.append("Hover profile brakes and remains parked; non-hover reaches then keeps validated orbit above minimum speed")

        invalid = copy.deepcopy(actors[1]); invalid["npc"]["flight"]["minimum_speed"] = 4
        before = c.call("inspect")
        c.call("upsert", entity=invalid, expected_error=True)
        assert c.call("inspect") == before
        changed = copy.deepcopy(actors[1]); changed["npc"]["flight"]["turn_degrees_per_second"] = 90
        c.call("upsert", entity=changed)
        assert c.call("inspect_mobility")["flight_profiles"]["curved"]["turn_degrees_per_second"] == 90
        c.call("undo")
        assert c.call("inspect_mobility")["flight_profiles"]["curved"]["turn_degrees_per_second"] == 45
        c.call("redo")
        c.call("load_scene", path="examples/npc-flight.scene.json")
        assert c.call("inspect_mobility")["states"]["curved"]["flight"] is None
        checks.append("MCP validation is atomic; profile edits, undo/redo, save/load and reset preserve configuration and clear runtime")

        c.call("step", ticks=80)
        before = c.call("inspect")
        c.call("set_debug_view", npc_routes=False)
        c.call("capture", path="artifacts/flight-no-guides.png", width=1280, height=900)
        after = c.call("inspect")
        assert before["mobility"] == after["mobility"] and before["runtime_positions"] == after["runtime_positions"]
        assert (ROOT / "artifacts/flight-curves.png").read_bytes() != (ROOT / "artifacts/flight-no-guides.png").read_bytes()
        c.call("set_debug_view", npc_routes=True)
        c.call("reset")
        checks.append("Cached flight curves and direction guides use the MCP route toggle without mutating simulation")
        report = {"passed": True, "checks": checks, "build_id": c.call("inspect")["build_id"], "middle": middle["mobility"], "end": end["mobility"], "native_window": window, "slow_requests": c.slow_requests}
        (ROOT / "artifacts/flight-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": report["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
