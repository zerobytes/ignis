#!/usr/bin/env python3
"""Patrol authoring, perception interruption and recovery through the actual MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    client = Client(binary, window)
    checks, samples = [], []
    try:
        client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                      "clientInfo": {"name": "ignis-patrol-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        assert "inspect_patrols" in json.dumps(client.request("tools/list"))
        client.call("load_scene", path="examples/stealth.scene.json")
        hunter = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "hunter")
        hunter["npc"]["patrol"] = {
            "route": {"kind": "points", "points": [[-4, 2], [-4, -2], [4, -2], [4, 2]]},
            "traversal": "loop", "wait_ticks": 60, "travel_ticks": 1800,
            "retry_ticks": 180, "arrival_distance": 0.25}
        client.call("upsert", entity=hunter)
        client.call("save_scene", path="examples/patrol.scene.json")
        client.call("set_debug_view", npc_vision=True, npc_hearing=True, noise_events=True)
        client.call("step", ticks=30)
        before = client.call("inspect")
        assert before["npcs"]["hunter"]["memory"] is None
        assert before["patrols"]["hunter"]["phase"] == "moving"
        client.call("capture", path="artifacts/patrol-moving.png", width=1280, height=900)
        interrupted_index = before["patrols"]["hunter"]["point_index"]
        client.call("emit_noise", origin={"kind": "position", "position": [0, 0.8, 4]}, radius=12)
        client.call("step", ticks=1)
        assert client.call("inspect")["patrols"]["hunter"]["phase"] == "suspended"
        client.call("capture", path="artifacts/patrol-interrupted.png", width=1280, height=900)
        resumed = False
        for _ in range(900):
            client.call("step", ticks=1)
            snap = client.call("inspect")
            if snap["patrols"]["hunter"]["phase"] == "moving":
                assert snap["npcs"]["hunter"]["memory"] is None
                assert snap["patrols"]["hunter"]["point_index"] == interrupted_index
                samples.append({"resumed_tick": snap["tick"], "point_index": interrupted_index})
                resumed = True
                break
        assert resumed, {"tick":snap["tick"],"input":snap["input"],"npc":snap["npcs"]["hunter"],"patrol":snap["patrols"]["hunter"],"navigation":snap["navigation"]["hunter"]}
        client.call("capture", path="artifacts/patrol-resumed.png", width=1280, height=900)
        client.call("save_replay", path="artifacts/patrol.replay.json")
        result = client.call("run_replay", path="artifacts/patrol.replay.json")
        assert result["matched"] and result["patrols_matched"]
        checks.append("MCP-authored patrol, sound interruption, same-point resume and replay")

        routine = copy.deepcopy(hunter)
        routine["npc"]["awareness"]["vision_range"] = 0.1
        routine["npc"]["awareness"]["hearing"] = None
        routine["npc"]["patrol"]["route"] = {"kind": "ring", "center": [0, 3], "radius": 1, "points": 4, "seed": 27}
        routine["npc"]["patrol"]["wait_ticks"] = 10
        routine["npc"]["patrol"]["traversal"] = "ping_pong"
        client.call("upsert", entity=routine)
        route = client.call("inspect_patrols")["routes"]
        client.call("step", ticks=600)
        state = client.call("inspect_patrols")["patrols"]["hunter"]
        assert state["visited"] >= 5 and state["skipped"] == 0
        samples.append({"ring_route": route, "ring_state": state})
        client.call("save_replay", path="artifacts/patrol.replay.json")
        assert client.call("run_replay", path="artifacts/patrol.replay.json")["matched"]
        client.call("reset")
        assert client.call("inspect_patrols")["routes"] == route
        alternate = copy.deepcopy(routine)
        alternate["npc"]["patrol"]["route"]["seed"] = 28
        client.call("upsert", entity=alternate)
        assert client.call("inspect_patrols")["routes"] != route
        client.call("undo")
        assert client.call("inspect_patrols")["routes"] == route
        checks.append("seeded route, repeated visits, ping-pong, reset and undo regenerate deterministically")

        before = client.call("inspect")
        for field, value in [("travel_ticks", 0), ("retry_ticks", 0), ("wait_ticks", 3601), ("arrival_distance", 0)]:
            bad = copy.deepcopy(routine)
            bad["npc"]["patrol"][field] = value
            client.call("upsert", entity=bad, expected_error=True)
            assert client.call("inspect") == before
        bad = copy.deepcopy(routine)
        bad["npc"]["patrol"]["route"] = {"kind": "points", "points": [[0, 0]] * 17}
        client.call("upsert", entity=bad, expected_error=True)
        assert client.call("inspect") == before
        blocked = copy.deepcopy(routine)
        blocked["npc"]["patrol"]["route"] = {"kind": "points", "points": [[100,100], [101,101]]}
        client.call("upsert", entity=blocked)
        client.call("step", ticks=4)
        state = client.call("inspect_patrols")["patrols"]["hunter"]
        assert state["phase"] == "cooldown" and state["skipped"] == 2
        client.call("step", ticks=60)
        assert client.call("inspect_patrols")["patrols"]["hunter"] == state
        assert client.call("inspect_navigation")["npcs"]["hunter"]["target_position"] is None
        client.call("save_replay", path="artifacts/patrol.replay.json")
        assert client.call("run_replay", path="artifacts/patrol.replay.json")["matched"]
        checks.append("atomic limits and unreachable-route cooldown without repeated navigation")
        client.call("load_scene", path="examples/patrol.scene.json")
        client.call("capture", path="artifacts/patrol-before.png", width=1280, height=900)
        report = {"passed": True, "checks": checks, "samples": samples, "native_window": window,
                  "slow_requests": client.slow_requests, "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/patrol-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "samples": samples}))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
