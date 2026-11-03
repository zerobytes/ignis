#!/usr/bin/env python3
"""Exercise bounded local connectors and sensory pursuit through real MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def actor(identifier, position, color, goal, fly=False):
    return {"id": identifier, "name": identifier, "body": "npc", "color": color,
            "transform": {"position": position, "size": [0.6, 1.6, 0.6]},
            "npc": {"surface": "floor", "speed": 3, "stop_distance": 0.8, "enabled": True,
                    "mobility": {"walk": not fly, "jump_speed": None, "fly_speed": 3 if fly else None, "glide": None},
                    "local_connections": {"walk": not fly, "fly": fly, "max_distance": 6 if fly else 4.5},
                    "objectives": [{"priority": 1, "when": "always", "action": {"kind": "guard_3d", "point": goal, "arrival_distance": 0.2}}]}}


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "ignis-local-connections-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        assert "local_connections" in json.dumps(c.request("tools/list"))
        c.call("generate", recipe={"width": 24, "depth": 20, "obstacles": 0, "seed": 42})
        player = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["transform"]["position"] = [-8, 0.8, 7]
        walker = actor("walker", [-5, 0.8, 0], [0.7, 0.2, 0.8], [5, 0.8, 0])
        flyer = actor("flyer", [-5, 0.8, 2], [0.2, 0.7, 0.9], [5, 3.8, 0], True)
        wall = {"id": "barrier", "name": "barrier", "body": "fixed", "color": [0.3, 0.7, 0.6], "transform": {"position": [0, 2, 0], "size": [1, 4, 6]}}
        network = {"nodes": [[-3, 0, -4], [3, 0, -4], [-3, 0, 4], [3, 0, 4], [-3, 5, 0], [3, 5, 0]], "links": []}
        for a, b in [(0, 1), (2, 3), (0, 2), (1, 3), (4, 5)]:
            for source, target in [(a, b), (b, a)]:
                network["links"].append({"from": source, "to": target, "mode": "fly" if a == 4 else "walk"})
        edits = [{"op": "set_spatial_network", "network": network}]
        edits += [{"op": "upsert", "entity": e} for e in [player, wall, walker, flyer]]
        edits += [{"op": "set_camera", "camera": {"position": [-15, 16, 18], "target": [0, 2, 0], "fov_degrees": 48}}]
        c.call("edit_scene", edits=edits)
        c.call("set_debug_view", npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)
        c.call("select_entity", id="walker")
        c.call("save_scene", path="artifacts/connectors-guard.scene.json")
        c.call("step", ticks=1)
        first = c.call("inspect_mobility")
        assert all(first["states"][name]["connector"]["kind"] == "entry" for name in ["walker", "flyer"]), first
        c.call("step", ticks=59)
        live = c.call("inspect")
        c.call("capture", path="artifacts/connectors-traversing.png", width=1280, height=900)
        c.call("save_replay", path="artifacts/connectors-traversing.replay.json")
        assert c.call("run_replay", path="artifacts/connectors-traversing.replay.json")["matched"]
        c.call("capture_replay", replay_path="artifacts/connectors-traversing.replay.json", path="artifacts/connectors-traversing-replay.png", width=1280, height=900)
        assert (ROOT / "artifacts/connectors-traversing.png").read_bytes() == (ROOT / "artifacts/connectors-traversing-replay.png").read_bytes()
        assert c.call("inspect_replay", path="artifacts/connectors-traversing.replay.json", tick=1)["snapshot"]["mobility"] == first["states"]
        c.call("step", ticks=540)
        reached = c.call("inspect")
        assert all(reached["mobility"][name]["phase"] == "arrived" for name in ["walker", "flyer"]), reached["mobility"]
        assert reached["runtime_positions"]["flyer"][1] > 3.5
        assert all(reached["mobility"][name]["completed"] >= 3 for name in ["walker", "flyer"]), reached["mobility"]
        checks.append("Off-network start and goal use physical entry/network/exit; walk detour and XYZ flight; exact replay/prefix/PNG")

        before = c.call("inspect")
        invalid = copy.deepcopy(walker)
        invalid["npc"]["local_connections"]["max_distance"] = 8.1
        c.call("upsert", entity=invalid, expected_error=True)
        assert c.call("inspect") == before
        limited = copy.deepcopy(walker)
        limited["npc"]["local_connections"]["max_distance"] = 0.25
        c.call("upsert", entity=limited)
        c.call("step", ticks=1)
        assert c.call("inspect")["mobility"]["walker"]["reason"] == "no_local_route"
        c.call("undo")
        assert c.call("inspect_mobility")["local_connections"]["walker"]["max_distance"] == 4.5
        c.call("redo")
        c.call("load_scene", path="artifacts/connectors-guard.scene.json")
        assert c.call("inspect")["mobility"]["walker"]["connector"] is None
        checks.append("MCP range limits, capability schema, atomic error, save/load/undo/redo and reset")

        walker["transform"]["position"] = [-5, 0.8, 4.5]
        walker["npc"].update(target="player", objectives=[{"priority": 1, "when": "always", "action": {"kind": "respond"}}],
                             awareness={"mode": "perceptive", "vision_range": 16, "fov_degrees": 360, "initial_facing_degrees": 90,
                                        "search_turn_degrees_per_second": 90, "memory_ticks": 180, "search_ticks": 60, "hearing": {"range": 16}})
        player["transform"]["position"] = [4, 0.8, 4.5]
        c.call("edit_scene", edits=[{"op": "upsert", "entity": e} for e in [walker, player]])
        c.call("set_debug_view", npc_routes=True, npc_vision=True, npc_hearing=False, noise_events=False)
        c.call("save_scene", path="examples/npc-connectors.scene.json")
        c.call("step", ticks=1)
        observed = c.call("inspect")
        assert observed["npcs"]["walker"]["observed_this_tick"]
        assert observed["mobility"]["walker"]["connector"] is not None
        c.call("step", ticks=89)
        c.call("set_input", input={"x": 1, "z": 0})
        c.call("step", ticks=30)
        c.call("set_input", input={"x": 0, "z": 0})
        c.call("step", ticks=300)
        pursued = c.call("inspect")
        assert abs(pursued["runtime_positions"]["walker"][0] - pursued["runtime_positions"]["player"][0]) < 1.2, pursued["mobility"]
        assert pursued["npcs"]["walker"]["observed_this_tick"]
        c.call("save_replay", path="artifacts/connectors-pursuit.replay.json")
        assert c.call("run_replay", path="artifacts/connectors-pursuit.replay.json")["matched"]
        checks.append("Perceptive NPC pursues a moving player at free ground positions through the shared spatial motor; replay matches")

        c.call("reset")
        c.call("step", ticks=60)
        c.call("capture", path="artifacts/connectors-demo.png", width=1280, height=900)
        before = c.call("inspect")
        c.call("set_debug_view", npc_routes=False)
        c.call("capture", path="artifacts/connectors-no-routes.png", width=1280, height=900)
        after = c.call("inspect")
        assert before["mobility"] == after["mobility"] and before["runtime_positions"] == after["runtime_positions"]
        assert (ROOT / "artifacts/connectors-demo.png").read_bytes() != (ROOT / "artifacts/connectors-no-routes.png").read_bytes()
        c.call("set_debug_view", npc_routes=True)
        c.call("reset")
        checks.append("Cyan connector endpoints and paths use MCP route guides without changing simulation; demo persists and resets")
        report = {"passed": True, "checks": checks, "first": first, "traversing": live["mobility"], "reached": reached["mobility"],
                  "pursued": pursued["mobility"], "native_window": window, "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT / "artifacts/local-connections-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": report["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
