#!/usr/bin/env python3
"""MCP-only authoring and comparison of walking, sneaking and running."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT

PROFILES = {"walk": {"speed": 4, "noise_multiplier": 1},
            "sneak": {"speed": 1.5, "noise_multiplier": 0.2},
            "run": {"speed": 6, "noise_multiplier": 1.8}}


def verify(binary, window):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    client = Client(binary, window)
    checks, samples = [], []
    try:
        client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                      "clientInfo": {"name": "ignis-locomotion-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        assert all(mode in json.dumps(client.request("tools/list")) for mode in PROFILES)
        client.call("load_scene", path="examples/hearing.scene.json")
        player = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["locomotion"] = copy.deepcopy(PROFILES)
        player["footsteps"] = {"surface": "floor", "stride": 0.3, "radius": 8, "min_speed": 0.1}
        client.call("upsert", entity=player)
        client.call("save_scene", path="examples/stealth.scene.json")
        client.call("set_debug_view", npc_vision=True, npc_hearing=True, noise_events=True)
        for mode, profile in PROFILES.items():
            client.call("reset")
            client.call("step", ticks=30)
            before = client.call("inspect")
            client.call("set_input", input={"x": 1, "z": 0, "mode": mode})
            assert client.call("inspect")["tick"] == before["tick"]
            actual_profile = client.call("inspect")["player_motion"]
            assert all(abs(actual_profile[k] - v) < 0.00001 for k, v in profile.items())
            client.call("step", ticks=20)
            snap = client.call("inspect")
            heard = snap["npcs"]["hunter"]["memory"] is not None
            assert heard == (mode != "sneak")
            assert not snap["npcs"]["hunter"]["observed_this_tick"]
            distance = snap["runtime_positions"]["player"][0] - before["runtime_positions"]["player"][0]
            assert abs(distance - profile["speed"] / 3) < 0.03
            assert snap["generated_footsteps"] > 0 and snap["recorded_noises"] == 0
            noise = client.call("inspect_footsteps")["footsteps"]["recent"][-1]
            assert abs(noise["radius"] - 8 * profile["noise_multiplier"]) < 0.001
            client.call("capture", path=f"artifacts/locomotion-{mode}.png", width=1280, height=900)
            client.call("save_replay", path="artifacts/locomotion.replay.json")
            result = client.call("run_replay", path="artifacts/locomotion.replay.json")
            assert result["matched"] and result["footsteps_matched"]
            samples.append({"mode": mode, "distance": distance, "heard": heard,
                            "noise_radius": noise["radius"], "emitted": snap["generated_footsteps"]})
        checks.append("MCP-authored scene, mode inspection while paused, physical speed, hidden hearing and replay")

        before = client.call("inspect")
        client.call("set_input", input={"x": 1, "z": 0, "mode": "fly"}, expected_error=True)
        assert client.call("inspect") == before
        for field, value in [("speed", 0), ("speed", 1001), ("noise_multiplier", -1), ("noise_multiplier", 5)]:
            bad = copy.deepcopy(player)
            bad["locomotion"]["run"][field] = value
            client.call("upsert", entity=bad, expected_error=True)
            assert client.call("inspect") == before
        client.call("set_input", input={"x": 0, "z": 0})
        assert client.call("inspect")["input"]["mode"] == "walk"
        checks.append("invalid input/profile rejection is atomic; missing mode preserves legacy walk default")

        silent = copy.deepcopy(player)
        silent["locomotion"]["sneak"]["noise_multiplier"] = 0
        client.call("upsert", entity=silent)
        client.call("step", ticks=30)
        client.call("set_input", input={"x": 1, "z": 0, "mode": "sneak"})
        client.call("step", ticks=60)
        assert client.call("inspect")["generated_footsteps"] == 0
        assert client.call("inspect")["npcs"]["hunter"]["memory"] is None
        client.call("save_replay", path="artifacts/locomotion.replay.json")
        assert client.call("run_replay", path="artifacts/locomotion.replay.json")["matched"]
        client.call("undo")
        assert client.call("inspect")["input"]["mode"] == "walk"
        assert client.call("inspect")["scene"]["version"] == SCENE_VERSION
        restored = next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "player")
        assert all(abs(restored["locomotion"][mode][key] - value) < 0.00001
                   for mode, profile in PROFILES.items() for key, value in profile.items())
        checks.append("explicit silence, replay and undo reset runtime while restoring authored profiles")
        client.call("load_scene", path="examples/stealth.scene.json")
        client.call("capture", path="artifacts/stealth-before.png", width=1280, height=900)
        report = {"passed": True, "checks": checks, "samples": samples, "native_window": window,
                  "build_id": client.call("inspect")["build_id"], "slow_requests": client.slow_requests}
        (ROOT / "artifacts/locomotion-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "samples": samples}))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
