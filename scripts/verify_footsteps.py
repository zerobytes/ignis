#!/usr/bin/env python3
"""Real locomotion, hearing and bounded deterministic footsteps through MCP."""
from verify_lab import SCENE_VERSION
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
                                      "clientInfo": {"name": "ignis-footstep-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        assert "inspect_footsteps" in json.dumps(client.request("tools/list"))
        client.call("load_scene", path="examples/hearing.scene.json")
        assert client.call("inspect")["scene"]["version"] == SCENE_VERSION
        player = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["footsteps"] = {"surface": "floor", "stride": 0.6, "radius": 8, "min_speed": 0.1}
        client.call("upsert", entity=player)
        client.call("save_scene", path="examples/footsteps.scene.json")
        client.call("load_scene", path="examples/footsteps.scene.json")
        client.call("set_debug_view", npc_vision=True, npc_hearing=True, noise_events=True)
        client.call("step", ticks=60)
        assert client.call("inspect_footsteps")["footsteps"]["total_emitted"] == 0
        client.call("set_input", input={"x": 1, "z": 0})
        client.call("step", ticks=10)
        assert client.call("inspect_footsteps")["footsteps"]["total_emitted"] > 0
        client.call("step", ticks=1)
        snap = client.call("inspect")
        assert snap["recorded_noises"] == 0
        assert snap["npcs"]["hunter"]["memory"]["source"] == "sound"
        assert not snap["npcs"]["hunter"]["observed_this_tick"]
        samples.append({"footstep_heard": snap})
        client.call("capture", path="artifacts/footsteps-heard.png", width=1280, height=900)
        client.call("set_input", input={"x": 0, "z": 0})
        count = snap["generated_footsteps"]
        client.call("step", ticks=30)
        assert client.call("inspect")["generated_footsteps"] == count
        client.call("save_replay", path="artifacts/footsteps.replay.json")
        result = client.call("run_replay", path="artifacts/footsteps.replay.json")
        assert result["matched"] and result["footsteps_matched"]
        checks.append("configured scene, silence, physical stride, hidden hearing without N and exact replay")

        client.call("reset")
        assert client.call("inspect_footsteps")["footsteps"]["total_emitted"] == 0
        for field, value in [("stride", 0.1), ("radius", 0), ("min_speed", 0), ("surface", "player")]:
            bad = copy.deepcopy(player)
            bad["footsteps"][field] = value
            before = client.call("inspect")
            client.call("upsert", entity=bad, expected_error=True)
            assert client.call("inspect") == before
        silent = copy.deepcopy(player)
        silent["footsteps"] = None
        client.call("upsert", entity=silent)
        assert client.call("inspect_footsteps")["footsteps"]["states"] == {}
        client.call("undo")
        assert "player" in client.call("inspect_footsteps")["footsteps"]["states"]
        client.call("redo")
        assert client.call("inspect_footsteps")["footsteps"]["states"] == {}
        client.call("undo")
        checks.append("atomic validation and reset/undo/redo clear runtime accumulators")

        wall = copy.deepcopy(player)
        wall["transform"]["position"] = [0, 0.85, -0.65]
        client.call("upsert", entity=wall)
        client.call("set_input", input={"x": 0, "z": 1})
        client.call("step", ticks=180)
        assert client.call("inspect")["generated_footsteps"] == 0
        airborne = copy.deepcopy(player)
        airborne["transform"]["position"] = [0, 4, -2]
        client.call("upsert", entity=airborne)
        client.call("set_input", input={"x": 1, "z": 0})
        client.call("step", ticks=20)
        assert client.call("inspect")["generated_footsteps"] == 0
        checks.append("movement against a wall and airborne motion produce no false footsteps")

        client.call("generate", recipe={"width": 16, "depth": 14, "obstacles": 0, "seed": 42})
        walker = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "player")
        walker["footsteps"] = {"surface": "floor", "stride": 0.2, "radius": 3, "min_speed": 0.1}
        client.call("upsert", entity=walker)
        client.call("step", ticks=30)
        for _ in range(16):
            client.call("set_input", input={"x": 1, "z": 0})
            client.call("step", ticks=60)
            client.call("set_input", input={"x": -1, "z": 0})
            client.call("step", ticks=60)
        state = client.call("inspect_footsteps")["footsteps"]
        assert state["total_emitted"] > 256 and len(state["recent"]) == 256
        assert state["dropped_recent"] == state["total_emitted"] - 256
        assert len(state["pending"]) <= 1 and client.call("inspect_noises")["noises"] == []
        meta = client.call("capture", path="artifacts/footsteps-history.png", width=640, height=450)
        assert meta["noise_guide_count"] == 16
        client.call("save_replay", path="artifacts/footsteps-history.replay.json")
        assert client.call("run_replay", path="artifacts/footsteps-history.replay.json")["matched"]
        samples.append({"long_walk": {"emitted": state["total_emitted"], "retained": len(state["recent"]), "dropped": state["dropped_recent"]}})
        checks.append("bounded automatic history independent of manual budget, overlay merge and regeneration replay")
        client.call("load_scene", path="examples/footsteps.scene.json")
        client.call("capture", path="artifacts/footsteps-before.png", width=1280, height=900)
        report = {"passed": True, "checks": checks, "samples": samples, "native_window": window,
                  "slow_requests": client.slow_requests, "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/footsteps-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "native_window": window}))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
