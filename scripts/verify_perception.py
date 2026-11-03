#!/usr/bin/env python3
"""Exercise perception, memory and explicit omniscience through the real MCP transport."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_npc import entity


def verify(binary, window=False):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    client = Client(binary, window)
    checks, samples = [], []
    try:
        client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                       "clientInfo": {"name": "ignis-perception-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        schema = client.request("tools/list")["result"]["tools"][0]["inputSchema"]
        assert "inspect_npcs" in json.dumps(schema) and "blocks_vision" in json.dumps(schema)
        client.call("load_scene", path="examples/npc.scene.json")
        migrated = client.call("inspect")["scene"]
        assert migrated["version"] == SCENE_VERSION
        assert next(e for e in migrated["entities"] if e["id"] == "hunter")["npc"]["awareness"]["mode"] == "omniscient"
        client.call("step", ticks=1)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["memory"]["source"] == "omniscient"
        checks.append("v2 migration preserves explicit omniscient behavior")

        client.call("generate", recipe={"width": 16, "depth": 14, "seed": 42, "obstacles": 0})
        hunter = entity("hunter", [0, 0.85, -4], [0.7, 1.6, 0.7], "npc", [0.75, 0.15, 0.8],
                        {"target": "player", "surface": "floor", "speed": 1, "stop_distance": 1.2, "enabled": True})
        barrier = entity("barrier", [0, 1, 0], [6, 2, 0.6])
        client.call("upsert", entity=barrier)
        client.call("upsert", entity=hunter)
        current = client.call("inspect")
        default = next(e for e in current["scene"]["entities"] if e["id"] == "hunter")["npc"]["awareness"]
        assert default["mode"] == "perceptive"
        # Face the hidden player to distinguish occlusion from the FOV test.
        hunter["npc"]["awareness"] = dict(default, initial_facing_degrees=180)
        client.call("upsert", entity=hunter)
        client.call("step", ticks=120)
        blocked = client.call("inspect")
        assert blocked["npcs"]["hunter"]["phase"] == "unaware"
        assert blocked["npcs"]["hunter"]["memory"] is None
        assert blocked["navigation"]["hunter"]["target_position"] is None
        assert abs(blocked["runtime_positions"]["hunter"][2] + 4) < 0.01
        barrier["blocks_vision"] = False
        client.call("upsert", entity=barrier)
        client.call("step", ticks=1)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["observed_this_tick"]
        client.call("undo")
        client.call("step", ticks=1)
        assert not client.call("inspect_npcs")["npcs"]["hunter"]["observed_this_tick"]
        client.call("redo")
        client.call("step", ticks=1)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["observed_this_tick"]
        checks.append("perceptive default, hidden target unknown, sight independent of collider, undo/redo")

        client.call("generate", recipe={"width": 16, "depth": 14, "seed": 42, "obstacles": 0})
        player = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["transform"]["position"] = [-2, 0.85, 4]
        client.call("upsert", entity=player)
        hunter["transform"]["position"] = [-5, 0.85, 4]
        hunter["npc"]["awareness"] = {"mode": "perceptive", "vision_range": 4, "fov_degrees": 360,
            "initial_facing_degrees": 90, "memory_ticks": 600, "search_ticks": 60,
            "search_turn_degrees_per_second": 90}
        client.call("upsert", entity=hunter)
        authored = client.call("inspect")["scene"]
        before = client.call("inspect")
        for key, value in [("memory_ticks", 0), ("fov_degrees", 361), ("vision_range", -1)]:
            invalid = copy.deepcopy(hunter)
            invalid["npc"]["awareness"][key] = value
            client.call("upsert", entity=invalid, expected_error=True)
            assert client.call("inspect") == before
        checks.append("invalid perception edits are atomic")
        client.call("save_scene", path="artifacts/perception.scene.json")
        client.call("step", ticks=1)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["phase"] == "pursuing"
        client.call("set_input", input={"x": 1, "z": 0})
        client.call("step", ticks=135)
        client.call("set_input", input={"x": 0, "z": 0})
        escaped = client.call("inspect")
        assert escaped["npcs"]["hunter"]["phase"] == "investigating"
        remembered = escaped["npcs"]["hunter"]["memory"]
        assert remembered["position"] != escaped["runtime_positions"]["player"]
        captured = set()
        for _ in range(120):
            snap = client.call("inspect")
            state = snap["npcs"]["hunter"]
            samples.append({"tick": snap["tick"], "state": state, "navigation": snap["navigation"]["hunter"],
                            "hunter": snap["runtime_positions"]["hunter"], "player": snap["runtime_positions"]["player"]})
            if state["memory"] is not None:
                assert state["memory"] == remembered
            if state["phase"] in {"searching", "lost"} and state["phase"] not in captured:
                client.call("capture", path=f'artifacts/perception-{state["phase"]}.png', width=1280, height=900)
                captured.add(state["phase"])
                if state["phase"] == "searching":
                    client.call("save_replay", path="artifacts/perception-search.replay.json")
                    assert client.call("run_replay", path="artifacts/perception-search.replay.json")["npcs_matched"]
            if state["phase"] == "lost":
                assert state["reason"] == "search_budget_exhausted"
                assert snap["navigation"]["hunter"]["target_position"] is None
                break
            client.call("step", ticks=6)
        assert captured == {"searching", "lost"}, samples[-1]
        client.call("save_replay", path="artifacts/perception.replay.json")
        replay = client.call("run_replay", path="artifacts/perception.replay.json")
        assert replay["matched"] and replay["npcs_matched"]
        checks.append("input-driven escape, frozen memory, investigation, bounded search, give-up and exact replay")
        client.call("set_input", input={"x": -1, "z": 0})
        client.call("step", ticks=90)
        client.call("set_input", input={"x": 0, "z": 0})
        assert client.call("inspect_npcs")["npcs"]["hunter"]["phase"] == "pursuing"
        client.call("save_replay", path="artifacts/perception-reacquired.replay.json")
        assert client.call("run_replay", path="artifacts/perception-reacquired.replay.json")["npcs_matched"]
        checks.append("target reacquired from a new visual observation and replayed")
        client.call("load_scene", path="artifacts/perception.scene.json")
        assert client.call("inspect")["scene"] == authored
        client.call("step", ticks=5)
        client.call("reset")
        assert client.call("inspect_npcs")["npcs"]["hunter"]["memory"] is None
        hunter["npc"]["enabled"] = False
        client.call("upsert", entity=hunter)
        client.call("step", ticks=5)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["phase"] == "disabled"
        client.call("remove", id="hunter")
        assert client.call("inspect_npcs")["npcs"] == {}
        checks.append("save/load, reset, disable and removal preserve knowledge lifecycle")
        budgets = []
        for memory, distance, reason in [(None, 1, "travel_budget_exhausted"),
                                         (600, 1, "travel_budget_exhausted"),
                                         (5, 100, "memory_expired")]:
            client.call("load_scene", path="artifacts/perception.scene.json")
            budget_hunter = copy.deepcopy(hunter)
            budget_hunter["npc"]["enabled"] = True
            budget_hunter["npc"]["awareness"].update(memory_ticks=memory, search_ticks=None,
                                                    max_investigation_distance=distance)
            client.call("upsert", entity=budget_hunter)
            configured = client.call("inspect")["scene"]
            client.call("save_scene", path="artifacts/perception-budget.scene.json")
            client.call("load_scene", path="artifacts/perception-budget.scene.json")
            assert client.call("inspect")["scene"] == configured
            invalid = copy.deepcopy(budget_hunter)
            invalid["npc"]["awareness"].update(memory_ticks=None, search_ticks=None,
                                             max_investigation_distance=None)
            before = client.call("inspect")
            client.call("upsert", entity=invalid, expected_error=True)
            assert client.call("inspect") == before
            client.call("step", ticks=1)
            client.call("set_input", input={"x": 1, "z": 0})
            client.call("step", ticks=135)
            client.call("set_input", input={"x": 0, "z": 0})
            state = client.call("inspect_npcs")["npcs"]["hunter"]
            assert state["phase"] == "lost" and state["reason"] == reason, state
            assert 0 < state["investigation_distance"] <= distance + 0.0001
            if reason == "travel_budget_exhausted":
                assert abs(state["investigation_distance"] - distance) < 0.001
            assert client.call("inspect_navigation")["npcs"]["hunter"]["target_position"] is None
            client.call("save_replay", path="artifacts/perception-budget.replay.json")
            assert client.call("run_replay", path="artifacts/perception-budget.replay.json")["matched"]
            budgets.append({"memory_ticks": memory, "distance_limit": distance, "state": state})
            client.call("reset")
            assert client.call("inspect_npcs")["npcs"]["hunter"]["investigation_distance"] == 0
        checks.append("distance-only and combined limits: first exhausted, atomic validation, save/load, reset, replay")
        client.call("load_scene", path="artifacts/perception.scene.json")
        # Author the interactive example exclusively with MCP operations.
        hunter["npc"]["enabled"] = True
        hunter["npc"]["awareness"]["fov_degrees"] = 120
        hunter["npc"]["awareness"]["search_ticks"] = 180
        client.call("upsert", entity=hunter)
        barrier.pop("blocks_vision", None)
        client.call("upsert", entity=barrier)
        client.call("save_scene", path="examples/perception.scene.json")
        client.call("capture", path="artifacts/perception-before.png", width=1280, height=900)
        report = {"passed": True, "checks": checks, "samples": samples, "replay": replay, "budgets": budgets,
                  "native_window": window, "slow_requests": client.slow_requests,
                  "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/perception-verification.json").write_text(json.dumps(report, indent=2)+"\n")
        print(json.dumps({"passed": True, "checks": len(checks), "native_window": window}, indent=2))
    finally:
        client.close()

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
