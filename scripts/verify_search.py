#!/usr/bin/env python3
"""Bounded spatial NPC search through MCP, including a player hiding behind a wall."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    client = Client(binary, window)
    checks, samples = [], []
    try:
        client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                      "clientInfo": {"name": "ignis-search-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        assert "area_search" in json.dumps(client.request("tools/list"))
        # Build an open test room from the versioned example via MCP.
        for radius in [2, 20]:
            client.call("load_scene", path="examples/perception.scene.json")
            client.call("remove", id="barrier")
            hunter = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "hunter")
            hunter["npc"]["awareness"].update(fov_degrees=360, memory_ticks=None, search_ticks=None,
                max_investigation_distance=100, area_search={"radius": radius, "points": 4, "scan_ticks": 12, "seed": 90})
            client.call("upsert", entity=hunter)
            configured = client.call("inspect")["scene"]
            client.call("save_scene", path="artifacts/search.scene.json")
            client.call("load_scene", path="artifacts/search.scene.json")
            assert client.call("inspect")["scene"] == configured
            bad = copy.deepcopy(hunter)
            bad["npc"]["awareness"]["area_search"]["points"] = 9
            before = client.call("inspect")
            client.call("upsert", entity=bad, expected_error=True)
            assert client.call("inspect") == before
            client.call("step", ticks=1)
            client.call("set_input", input={"x": 1, "z": 0})
            client.call("step", ticks=135)
            client.call("set_input", input={"x": 0, "z": 0})
            explored, scanned = False, False
            for _ in range(420):
                client.call("step", ticks=6)
                snap = client.call("inspect")
                state = snap["npcs"]["hunter"]
                explored |= state["phase"] == "exploring"
                scanned |= state["reason"] == "scan_search_point"
                if state["phase"] == "lost":
                    break
            assert state["reason"] == "search_points_exhausted", state
            assert explored and state["search_point_index"] == 4
            assert state["skipped_search_points"] == (4 if radius == 20 else 0)
            assert scanned == (radius == 2)
            client.call("save_replay", path="artifacts/search.replay.json")
            assert client.call("run_replay", path="artifacts/search.replay.json")["matched"]
            samples.append({"radius": radius, "tick": snap["tick"], "state": state})
        checks.append("bounded visits, unreachable points skipped, exact replay")
        checks.append("schema, atomic invalid config and save/load")

        # Author the interactive wall scenario, keeping the previous example unchanged.
        client.call("load_scene", path="examples/perception.scene.json")
        hunter = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "hunter")
        player = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["transform"]["position"] = [-5, 0.85, 4]
        client.call("upsert", entity=player)
        hunter["transform"]["position"] = [-2, 0.85, 4]
        hunter["npc"]["speed"] = 1.5
        hunter["npc"]["awareness"].update(initial_facing_degrees=-90, memory_ticks=1800, search_ticks=1200,
            max_investigation_distance=40, area_search={"radius": 3.5, "points": 4, "scan_ticks": 120, "seed": 0})
        client.call("upsert", entity=hunter)
        hunter = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "hunter")
        client.call("save_scene", path="examples/search.scene.json")
        client.call("step", ticks=1)
        client.call("set_input", input={"x": 0, "z": -1})
        client.call("step", ticks=100)
        client.call("set_input", input={"x": 1, "z": 0})
        client.call("step", ticks=52)
        client.call("set_input", input={"x": 0, "z": 0})
        escaped = client.call("inspect")
        remembered = escaped["npcs"]["hunter"]["memory"]
        assert remembered is not None and not escaped["npcs"]["hunter"]["observed_this_tick"], escaped
        explored = False
        for _ in range(300):
            snap = client.call("inspect")
            state = snap["npcs"]["hunter"]
            if state["phase"] == "exploring" and not explored:
                explored = True
                samples.append({"wall_exploration": snap})
                client.call("capture", path="artifacts/search-exploring.png", width=1280, height=900)
                client.call("save_replay", path="artifacts/search-exploring.replay.json")
                assert client.call("run_replay", path="artifacts/search-exploring.replay.json")["matched"]
            if state["observed_this_tick"]:
                assert explored and state["memory"]["tick"] > remembered["tick"]
                assert state["search_point"] is None and state["search_point_index"] == 0
                assert state["investigation_distance"] == 0
                break
            assert state["memory"] == remembered, state
            client.call("step", ticks=6)
        else:
            raise AssertionError("did not visually reacquire the player after exploring")
        client.call("capture", path="artifacts/search-reacquired.png", width=1280, height=900)
        client.call("save_replay", path="artifacts/search.replay.json")
        assert client.call("run_replay", path="artifacts/search.replay.json")["matched"]
        samples.append({"wall_reacquired": snap})
        checks.append("hidden player behind wall, frozen memory, exploration, visual reacquisition and replay")
        client.call("reset")
        clean = client.call("inspect_npcs")["npcs"]["hunter"]
        assert clean["search_point"] is None and clean["search_point_index"] == 0
        client.call("undo")
        prior = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "hunter")
        assert "area_search" not in prior["npc"]["awareness"]
        client.call("redo")
        restored = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "hunter")
        assert restored == hunter
        client.call("capture", path="artifacts/search-before.png", width=1280, height=900)
        checks.append("reset and undo/redo restore configuration and clear episode")
        report = {"passed": True, "checks": checks, "samples": samples, "native_window": window,
                  "slow_requests": client.slow_requests, "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/search-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "native_window": window}))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
