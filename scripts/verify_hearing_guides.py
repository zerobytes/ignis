#!/usr/bin/env python3
"""Acoustic diagnostic overlays through the real MCP dispatcher and GPU."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    client = Client(binary, window)
    checks = []
    try:
        client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                      "clientInfo": {"name": "ignis-acoustic-guide-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        assert all(k in json.dumps(client.request("tools/list")) for k in ["npc_hearing", "noise_events"])
        client.call("load_scene", path="examples/hearing.scene.json")
        # Wide view makes a twelve-metre nominal hearing circle fully visible.
        client.call("set_camera", camera={"position": [24, 28, 28], "target": [0, 0, 0], "fov_degrees": 48})

        def capture(name="acoustic-cache", width=640, height=450):
            path = f"artifacts/{name}.png"
            meta = client.call("capture", path=path, width=width, height=height)
            return (ROOT / path).read_bytes(), meta

        def noise_count(expected):
            meta = capture()[1]
            if meta["noise_guide_count"] != expected:
                diagnostic = {"expected": expected, "capture": meta, "snapshot": client.call("inspect"), "noises": client.call("inspect_noises")}
                (ROOT / "artifacts/acoustic-count-failure.json").write_text(json.dumps(diagnostic, indent=2) + "\n")
                raise AssertionError(diagnostic)

        off, _ = capture()
        before = client.call("inspect")
        client.call("save_replay", path="artifacts/acoustic-guides.replay.json")
        replay = (ROOT / "artifacts/acoustic-guides.replay.json").read_bytes()
        changed = client.call("set_debug_view", npc_hearing=True, noise_events=True)
        assert client.call("set_debug_view") == changed
        on, meta = capture()
        assert meta["hearing_guide_count"] == 1 and meta["noise_guide_count"] == 0
        assert meta["vision_guide_count"] == 0 and on != off and capture()[0] == on
        after = client.call("inspect")
        for key in ["scene", "tick", "revision", "runtime_positions", "npcs", "navigation"]:
            assert before[key] == after[key]
        client.call("save_replay", path="artifacts/acoustic-guides.replay.json")
        assert (ROOT / "artifacts/acoustic-guides.replay.json").read_bytes() == replay
        client.call("set_debug_view", npc_hearing=False)
        assert capture()[0] == off
        client.call("set_debug_view", npc_hearing=True)
        assert capture()[0] == on
        checks.append("partial/idempotent MCP controls, paused toggle, exact pixels and replay independence")

        rev = client.call("inspect")["view_revision"]
        client.call("emit_noise", origin={"kind": "entity", "id": "player"}, radius=4)
        snap = client.call("inspect")
        assert snap["tick"] == 0 and snap["view_revision"] == rev + 1
        noisy, meta = capture("acoustic-emission", 1280, 900)
        assert meta["noise_guide_count"] == 1
        assert noisy == capture("acoustic-emission", 1280, 900)[0]
        client.call("set_debug_view", noise_events=False)
        assert capture()[0] == on
        client.call("set_debug_view", noise_events=True)
        for _ in range(15):
            client.call("emit_noise", origin={"kind": "position", "position": [-3, 0.8, 2]}, radius=2)
        noise_count(16)
        client.call("step", ticks=1)
        client.call("emit_noise", origin={"kind": "position", "position": [3, 0.8, 2]}, radius=3)
        noise_count(16)
        client.call("step", ticks=89)
        noise_count(1)
        client.call("step", ticks=1)
        noise_count(0)
        client.call("save_replay", path="artifacts/acoustic-guides.replay.json")
        assert client.call("run_replay", path="artifacts/acoustic-guides.replay.json")["matched"]
        checks.append("paused noise redraw, frozen impulse, bounded history, exact expiration and gameplay replay")

        client.call("reset")
        assert capture()[0] == on
        hunter = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "hunter")
        for variant in ["disabled", "deaf", "omniscient"]:
            entity = copy.deepcopy(hunter)
            if variant == "disabled":
                entity["npc"]["enabled"] = False
            elif variant == "deaf":
                entity["npc"]["awareness"]["hearing"] = None
            else:
                entity["npc"]["awareness"] = {"mode": "omniscient"}
            client.call("upsert", entity=entity)
            assert capture()[1]["hearing_guide_count"] == 0
            client.call("undo")
            assert capture()[0] == on
        client.call("remove", id="hunter")
        assert capture()[1]["hearing_guide_count"] == 0
        client.call("undo")
        capture(width=320, height=240)
        assert capture()[0] == on
        checks.append("disabled/deaf/omniscient, remove/undo, reset and resize clear stale guides")
        client.call("set_debug_view", npc_vision=True)
        view = client.call("inspect")["debug_view"]
        assert all(view[k] for k in ("npc_vision", "npc_hearing", "noise_events"))
        assert not view["npc_routes"]
        client.call("load_scene", path="examples/hearing.scene.json")
        meta = capture("acoustic-before", 1280, 900)[1]
        assert meta["hearing_guide_count"] == 1 and meta["vision_guide_count"] == 1
        assert meta["noise_guide_count"] == 0
        checks.append("old vision control composes with hearing, load retains preferences")
        report = {"passed": True, "checks": checks, "native_window": window,
                  "slow_requests": client.slow_requests, "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/acoustic-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "native_window": window}))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
