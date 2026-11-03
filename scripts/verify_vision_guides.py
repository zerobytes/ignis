#!/usr/bin/env python3
"""Inspect NPC orientation/nominal vision guides using real MCP and GPU captures."""
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
                                      "clientInfo": {"name": "ignis-vision-guide-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        assert "set_debug_view" in json.dumps(client.request("tools/list"))
        client.call("load_scene", path="examples/hearing.scene.json")

        def capture(path="artifacts/vision-guide-cache.png", width=640, height=450):
            meta = client.call("capture", path=path, width=width, height=height)
            return (ROOT / path).read_bytes(), meta

        off, meta = capture()
        assert meta["vision_guide_count"] == 0
        before = client.call("inspect")
        client.call("save_replay", path="artifacts/vision-guides.replay.json")
        replay_before = (ROOT / "artifacts/vision-guides.replay.json").read_bytes()
        changed = client.call("set_debug_view", npc_vision=True)
        assert client.call("set_debug_view", npc_vision=True) == changed
        on, meta = capture()
        on_state = client.call("inspect")
        assert meta["vision_guide_count"] == 1 and meta["debug_view"]["npc_vision"]
        assert on != off and capture()[0] == on
        after = client.call("inspect")
        for key in ["scene", "tick", "revision", "runtime_positions", "npcs", "navigation", "history", "recorded_ticks"]:
            assert before[key] == after[key], key
        client.call("save_replay", path="artifacts/vision-guides.replay.json")
        assert (ROOT / "artifacts/vision-guides.replay.json").read_bytes() == replay_before
        client.call("set_debug_view", npc_vision=False)
        assert capture()[0] == off
        client.call("set_debug_view", npc_vision=True)
        assert capture()[0] == on, {"baseline": on_state, "current": client.call("inspect")}
        checks.append("MCP toggle while paused, idempotence, exact capture restoration, no gameplay/replay mutation")

        hunter = next(e for e in after["scene"]["entities"] if e["id"] == "hunter")
        for yaw, fov in [(90, 120), (-90, 120), (0, 360), (0, 1)]:
            rotated = copy.deepcopy(hunter)
            rotated["npc"]["awareness"].update(initial_facing_degrees=yaw, fov_degrees=fov)
            client.call("upsert", entity=rotated)
            assert client.call("inspect_npcs")["npcs"]["hunter"]["facing_degrees"] == yaw
            assert capture()[0] != on
            client.call("undo")
            assert capture()[0] == on, {"baseline": on_state, "current": client.call("inspect")}
        client.call("remove", id="hunter")
        assert capture()[1]["vision_guide_count"] == 0
        client.call("undo")
        assert capture()[0] == on, {"baseline": on_state, "current": client.call("inspect")}
        capture(width=320, height=240)
        assert capture()[0] == on, {"baseline": on_state, "current": client.call("inspect")}
        checks.append("yaw sign, narrow/360 FOV, removal/undo and resize invalidate guides")

        client.call("step", ticks=30)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["phase"] == "unaware"
        capture("artifacts/vision-unaware.png", 1280, 900)
        client.call("emit_noise", origin={"kind": "entity", "id": "player"}, radius=12)
        client.call("step", ticks=1)
        client.call("set_input", input={"x": 1, "z": 0})
        client.call("step", ticks=15)
        client.call("set_input", input={"x": 0, "z": 0})
        state = client.call("inspect_npcs")["npcs"]["hunter"]
        assert state["memory"]["source"] == "sound" and not state["observed_this_tick"]
        capture("artifacts/vision-memory.png", 1280, 900)
        samples.append({"memory": state})
        for _ in range(300):
            client.call("step", ticks=6)
            state = client.call("inspect_npcs")["npcs"]["hunter"]
            if state["observed_this_tick"]:
                assert state["memory"]["source"] == "vision"
                break
        else:
            raise AssertionError("No visual acquisition")
        capture("artifacts/vision-seeing.png", 1280, 900)
        samples.append({"seeing": state})
        client.call("save_replay", path="artifacts/vision-guides.replay.json")
        assert client.call("run_replay", path="artifacts/vision-guides.replay.json")["matched"]
        client.call("reset")
        assert capture()[0] == on, {"baseline": on_state, "current": client.call("inspect")}
        checks.append("live orientation, hidden memory and true visual acquisition, exact replay/reset")

        disabled = copy.deepcopy(hunter)
        disabled["npc"]["enabled"] = False
        client.call("upsert", entity=disabled)
        assert capture()[0] != on
        client.call("undo")
        omni = copy.deepcopy(hunter)
        omni["npc"]["awareness"] = {"mode": "omniscient"}
        client.call("upsert", entity=omni)
        assert capture()[0] != on
        client.call("undo")
        assert capture()[0] == on, {"baseline": on_state, "current": client.call("inspect")}
        client.call("load_scene", path="examples/hearing.scene.json")
        assert capture()[0] == on, {"baseline": on_state, "current": client.call("inspect")}
        capture("artifacts/vision-before.png", 1280, 900)
        checks.append("disabled/omniscient differ visually, load preserves view preference")
        report = {"passed": True, "checks": checks, "samples": samples, "native_window": window,
                  "slow_requests": client.slow_requests, "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/vision-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "native_window": window}))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
