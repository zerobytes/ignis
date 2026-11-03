#!/usr/bin/env python3
"""Build and verify pursuit exclusively over MCP; write current evidence in artifacts/."""
import argparse
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def entity(id, position, size, body="fixed", color=None, npc=None):
    value = {"id": id, "name": id, "transform": {"position": position, "size": size},
             "body": body, "color": color or [0.15, 0.65, 0.45]}
    if npc is not None:
        value["npc"] = npc
    return value


def verify(binary, window=False):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    client = Client(binary, window)
    checks = []
    try:
        init = client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                             "clientInfo": {"name": "ignis-npc-verifier", "version": "1"}})
        assert init["result"]["serverInfo"]["name"] == "ignis-engine"
        client.request("notifications/initialized", notification=True)
        schema = client.request("tools/list")["result"]["tools"][0]["inputSchema"]
        assert "inspect_navigation" in json.dumps(schema) and "NpcConfig" in json.dumps(schema)
        (ROOT / "artifacts/command-schema.json").write_text(json.dumps(schema, indent=2))
        checks.append("MCP discovery exposes NPC configuration and navigation inspection")
        client.call("generate", recipe={"width": 16, "depth": 14, "seed": 42, "obstacles": 0})
        barrier = entity("barrier", [0, 1, 0], [6, 2, 0.6])
        hunter = entity("hunter", [0, 0.85, -5], [0.7, 1.6, 0.7], "npc", [0.75, 0.15, 0.8],
                        {"target": "player", "surface": "floor", "speed": 3, "stop_distance": 1.2, "enabled": True, "awareness": {"mode": "omniscient"}})
        client.call("upsert", entity=barrier)
        client.call("upsert", entity=hunter)
        authored = client.call("inspect")["scene"]
        client.call("save_scene", path="artifacts/npc.scene.json")
        before = client.call("inspect")
        client.call("remove", id="player", expected_error=True)
        bad = json.loads(json.dumps(hunter))
        bad["npc"]["target"] = "missing"
        client.call("upsert", entity=bad, expected_error=True)
        assert client.call("inspect") == before
        checks.append("invalid target/removal rejected atomically")
        client.call("capture", path="artifacts/npc-before.png", width=1280, height=900)
        samples = []
        for i in range(90):
            client.call("step", ticks=6)
            snap = client.call("inspect")
            p = snap["runtime_positions"]["hunter"]
            assert abs(p[0]) >= 3.33 or abs(p[2]) >= 0.63, p
            samples.append({"tick": snap["tick"], "position": p, "status": snap["navigation"]["hunter"]["status"]})
            if i == 35:
                client.call("capture", path="artifacts/npc-detour.png", width=1280, height=900)
        assert max(abs(s["position"][0]) for s in samples) > 3.4
        def distance(snapshot):
            h, p = [snapshot["runtime_positions"][id] for id in ("hunter", "player")]
            return math.hypot(h[0]-p[0], h[2]-p[2])
        assert distance(snap) < 1.25
        assert client.call("inspect_navigation")["npcs"]["hunter"]["status"] == "arrived"
        assert snap["scene"] == authored
        checks.append("body clearance, detour, arrival and unchanged authoring scene")
        client.call("capture", path="artifacts/npc-arrived.png", width=1280, height=900)
        client.call("set_input", input={"x": 1, "z": 0})
        client.call("step", ticks=60)
        client.call("set_input", input={"x": 0, "z": 0})
        client.call("step", ticks=180)
        assert distance(client.call("inspect")) < 1.25
        checks.append("moving target reacquired")
        client.call("save_replay", path="artifacts/npc.replay.json")
        replay = client.call("run_replay", path="artifacts/npc.replay.json")
        assert replay["matched"] and replay["navigation_matched"]
        assert client.call("run_replay", path="artifacts/npc.replay.json")["matched"]
        checks.append("exact replay of positions and navigation twice")
        barrier["transform"]["size"][0] = 16
        client.call("upsert", entity=barrier)
        client.call("step", ticks=120)
        unreachable = client.call("inspect_navigation")
        assert unreachable["npcs"]["hunter"]["status"] == "unreachable"
        assert abs(client.call("inspect")["runtime_positions"]["hunter"][2]+5) < 0.01
        client.call("undo")
        client.call("step", ticks=540)
        assert distance(client.call("inspect")) < 1.25
        client.call("redo")
        client.call("step", ticks=12)
        assert client.call("inspect_navigation")["npcs"]["hunter"]["status"] == "unreachable"
        checks.append("sealed route, stopped NPC, undo recovers and redo blocks again")
        client.call("load_scene", path="artifacts/npc.scene.json")
        assert client.call("inspect")["scene"] == authored
        hunter["npc"]["enabled"] = False
        client.call("upsert", entity=hunter)
        client.call("step", ticks=120)
        assert client.call("inspect_navigation")["npcs"]["hunter"]["status"] == "disabled"
        client.call("remove", id="hunter")
        assert client.call("inspect_navigation")["npcs"] == {}
        client.call("load_scene", path="artifacts/npc.scene.json")
        checks.append("scene persistence, disable and removal")
        # Reproduce the player's actual wall contact, not only a hand-placed target.
        wall_contact = []
        for hunter_z in (5, -5):
            client.call("load_scene", path="artifacts/npc.scene.json")
            wall_player = next(e for e in authored["entities"] if e["id"] == "player")
            wall_player = json.loads(json.dumps(wall_player))
            wall_player["transform"]["position"] = [0, 0.85, 2]
            wall_hunter = json.loads(json.dumps(hunter))
            wall_hunter["npc"]["enabled"] = True
            wall_hunter["transform"]["position"] = [0, 0.85, hunter_z]
            client.call("upsert", entity=wall_player)
            client.call("upsert", entity=wall_hunter)
            client.call("set_input", input={"x": 0, "z": -1})
            client.call("step", ticks=60)
            contact = client.call("inspect")
            assert ["barrier", "player"] in contact["contacts"]
            client.call("step", ticks=480)
            result = client.call("inspect")
            assert result["navigation"]["hunter"]["status"] == "arrived", result["navigation"]
            assert distance(result) < 1.25
            assert ["barrier", "player"] in result["contacts"]
            assert not any("hunter" in pair and "barrier" in pair for pair in result["contacts"])
            wall_contact.append({"hunter_start_z": hunter_z, "positions": result["runtime_positions"],
                                 "navigation": result["navigation"], "contacts": result["contacts"]})
        client.call("save_replay", path="artifacts/npc-wall.replay.json")
        assert client.call("run_replay", path="artifacts/npc-wall.replay.json")["matched"]
        client.call("capture", path="artifacts/npc-wall-contact.png", width=1280, height=900)
        checks.append("player touching wall: same-side approach, opposite-side detour and replay")
        client.call("load_scene", path="artifacts/npc.scene.json")
        report = {"passed": True, "checks": checks, "native_window": window, "slow_requests": client.slow_requests, "samples": samples,
                  "wall_contact": wall_contact, "unreachable": unreachable, "replay": replay, "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/npc-verification.json").write_text(json.dumps(report, indent=2)+"\n")
        print(json.dumps({"passed": True, "checks": len(checks), "native_window": window}, indent=2))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
