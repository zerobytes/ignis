#!/usr/bin/env python3
"""Point-noise perception through the same MCP dispatcher as native controls."""
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
                                      "clientInfo": {"name": "ignis-hearing-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        schema = json.dumps(client.request("tools/list"))
        assert all(field in schema for field in ["emit_noise", "inspect_noises", "hearing", "sound_transmission"])
        client.call("load_scene", path="examples/search.scene.json")
        entities = {e["id"]: e for e in client.call("inspect")["scene"]["entities"]}
        player, hunter, barrier = (entities[k] for k in ["player", "hunter", "barrier"])
        player["transform"]["position"] = [0, 0.85, -2]
        hunter["transform"]["position"] = [0, 0.85, 2]
        hunter["npc"]["awareness"].update(hearing={"range": 12}, initial_facing_degrees=0,
                                             vision_range=6, area_search={"radius": 2, "points": 4, "scan_ticks": 120, "seed": 0})
        barrier["sound_transmission"] = 0.65
        for entity in [player, hunter, barrier]:
            client.call("upsert", entity=entity)
        configured = client.call("inspect")["scene"]
        client.call("save_scene", path="examples/hearing.scene.json")
        client.call("load_scene", path="examples/hearing.scene.json")
        assert client.call("inspect")["scene"] == configured
        for field, value in [("sound_transmission", -1), ("sound_transmission", 1.1)]:
            bad = copy.deepcopy(barrier)
            bad[field] = value
            before = client.call("inspect")
            client.call("upsert", entity=bad, expected_error=True)
            assert client.call("inspect") == before
        checks.append("schema, scene roundtrip and atomic configuration")

        client.call("step", ticks=30)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["phase"] == "unaware"
        player_position = client.call("inspect")["runtime_positions"]["player"]
        queued = client.call("emit_noise", origin={"kind": "entity", "id": "player"}, radius=12)["queued"]
        assert queued["position"] == player_position and queued["tick"] == 30
        assert client.call("inspect_noises")["pending"] == 1
        client.call("save_replay", path="artifacts/hearing-pending.replay.json")
        assert client.call("run_replay", path="artifacts/hearing-pending.replay.json")["matched"]
        client.call("step", ticks=1)
        heard = client.call("inspect_npcs")["npcs"]["hunter"]
        assert heard["heard_this_tick"] and not heard["observed_this_tick"]
        assert heard["phase"] == "investigating" and heard["memory"]["source"] == "sound"
        assert heard["memory"]["position"] == queued["position"]
        client.call("set_input", input={"x": 1, "z": 0})
        client.call("step", ticks=15)
        client.call("set_input", input={"x": 0, "z": 0})
        snap = client.call("inspect")
        assert snap["npcs"]["hunter"]["memory"] == heard["memory"]
        assert snap["runtime_positions"]["player"] != queued["position"]
        assert client.call("inspect_noises")["pending"] == 0
        client.call("capture", path="artifacts/hearing-investigating.png", width=1280, height=900)
        client.call("save_replay", path="artifacts/hearing.replay.json")
        assert client.call("run_replay", path="artifacts/hearing.replay.json")["matched"]
        samples.append({"investigation": snap, "noise": queued})
        checks.append("silence, hidden emission, frozen memory, pending/consumed replay")
        for _ in range(300):
            client.call("step", ticks=6)
            state = client.call("inspect_npcs")["npcs"]["hunter"]
            if state["observed_this_tick"]:
                assert state["memory"]["source"] == "vision" and "noise_id" not in state["memory"]
                break
        else:
            raise AssertionError("NPC never visually acquired the hidden player after investigating the sound")
        client.call("capture", path="artifacts/hearing-acquired.png", width=1280, height=900)
        client.call("save_replay", path="artifacts/hearing.replay.json")
        assert client.call("run_replay", path="artifacts/hearing.replay.json")["matched"]
        samples.append({"visual_acquisition": client.call("inspect")})
        checks.append("sound investigation routes around wall, real visual acquisition, exact replay")

        client.call("reset")
        assert not client.call("inspect_noises")["noises"]
        blocked = copy.deepcopy(barrier)
        blocked["sound_transmission"] = 0
        client.call("upsert", entity=blocked)
        client.call("emit_noise", origin={"kind": "entity", "id": "player"}, radius=100)
        client.call("step", ticks=1)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["phase"] == "unaware"
        client.call("undo")
        assert client.call("inspect")["scene"] == configured
        assert not client.call("inspect_noises")["noises"]
        client.call("redo")
        assert next(e for e in client.call("inspect")["scene"]["entities"] if e["id"] == "barrier")["sound_transmission"] == 0
        client.call("undo")
        decoy = client.call("emit_noise", origin={"kind": "position", "position": [-4, 0.85, 2]}, radius=12)["queued"]
        client.call("step", ticks=30)
        state = client.call("inspect_npcs")["npcs"]["hunter"]
        assert state["memory"]["source"] == "sound" and state["memory"]["position"] == decoy["position"], state
        checks.append("acoustic blocking independent of vision, reset/history, anonymous decoy")
        client.call("reset")
        for _ in range(16):
            client.call("emit_noise", origin={"kind": "position", "position": [0, 1, 0]}, radius=12)
        before = client.call("inspect_noises")
        client.call("emit_noise", origin={"kind": "position", "position": [0, 1, 0]}, radius=12, expected_error=True)
        assert client.call("inspect_noises") == before
        client.call("reset")
        client.call("capture", path="artifacts/hearing-before.png", width=1280, height=900)
        checks.append("bounded queue rejects overflow atomically")
        # Reproduce the reported contact at the wall end, outside the NPC's FOV.
        contact_hunter, contact_player = copy.deepcopy(hunter), copy.deepcopy(player)
        contact_hunter["transform"]["position"] = [3.3489742, 0.79893064, 0.43535045]
        contact_hunter["npc"]["awareness"]["initial_facing_degrees"] = 81.55771
        contact_player["transform"]["position"] = [2.2461648, 0.7989311, 0.6489972]
        for entity in [contact_hunter, contact_player]:
            client.call("upsert", entity=entity)
        client.call("save_scene", path="artifacts/hearing-contact.scene.json")
        client.call("set_debug_view", npc_vision=True)
        client.call("step", ticks=1)
        assert client.call("inspect_npcs")["npcs"]["hunter"]["phase"] == "unaware"
        client.call("emit_noise", origin={"kind": "entity", "id": "player"}, radius=12)
        client.call("step", ticks=1)
        contact = client.call("inspect")
        assert contact["npcs"]["hunter"]["heard_this_tick"]
        assert contact["navigation"]["hunter"]["status"] == "recovering"
        client.call("capture", path="artifacts/hearing-contact.png", width=1280, height=900)
        acquired = False
        for _ in range(120):
            client.call("step", ticks=1)
            snap = client.call("inspect")
            state = snap["npcs"]["hunter"]
            pos = snap["runtime_positions"]["hunter"]
            assert pos[0] >= 3.345 or abs(pos[2]) >= 0.645, pos
            assert state["phase"] != "lost" and state["skipped_search_points"] == 0, state
            acquired |= state["observed_this_tick"]
        assert acquired
        client.call("capture", path="artifacts/hearing-contact-acquired.png", width=1280, height=900)
        client.call("save_replay", path="artifacts/hearing-contact.replay.json")
        assert client.call("run_replay", path="artifacts/hearing-contact.replay.json")["matched"]
        samples.append({"contact_recovery": contact, "contact_acquired": snap})
        checks.append("wall-contact noise: recovery, no premature forgetting, real sight, collision and replay")
        client.call("load_scene", path="examples/hearing.scene.json")
        report = {"passed": True, "checks": checks, "samples": samples, "native_window": window,
                  "slow_requests": client.slow_requests, "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/hearing-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "native_window": window}))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
