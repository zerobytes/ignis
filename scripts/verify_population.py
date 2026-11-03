#!/usr/bin/env python3
"""Transactional NPC retirement over MCP, isolated and offscreen by default."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "ignis-population-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        scene = json.loads((ROOT/"examples/npc-outgoing.scene.json").read_text())
        player = next(e for e in scene["entities"] if e["id"] == "player")
        source = next(e for e in scene["entities"] if e["id"] == "half_ice")
        source["abilities"] = copy.deepcopy(player["abilities"])
        for e in scene["entities"]:
            if "npc" in e:
                e["npc"]["enabled"] = False
        source["npc"]["enabled"] = True
        source["npc"]["target"] = "player"
        source["npc"]["awareness"] = {"mode": "omniscient"}
        source["npc"].pop("objectives", None)
        source["transform"]["position"] = [0, 0.8, -2]
        player["transform"]["position"] = [-4, 0.8, -2]
        zone = {"id": "population_zone", "name": "Population zone", "color": [0.2, 0.7, 0.4], "visible": False, "transform": {"position": [0, 1, -2], "size": [2, 3, 2]}, "trigger": {"enabled": True, "targets": ["half_ice"]}}
        scene["entities"].append(zone)
        (ROOT/"artifacts/population.scene.json").write_text(json.dumps(scene))
        c.call("load_scene", path="artifacts/population.scene.json")
        c.call("step", ticks=2)
        authored = c.call("inspect")["scene"]
        identities = c.call("inspect_instances")["instances"]
        source_id = next(i for i in identities if i["authored_id"] == "half_ice")
        player_id = next(i for i in identities if i["authored_id"] == "player")
        for actor in ["player", "floor", "missing", "ignis:wrong:0000000000000001"]:
            before = c.call("inspect")
            c.call("despawn_actor", actor=actor, expected_error=True)
            assert c.call("inspect") == before
        checks.append("Invalid/player/fixed/foreign requests are atomic; no population or authoring changes")

        c.call("activate_ability", actor="half_ice", slot=3, target="player")
        c.call("step", ticks=1)
        c.call("activate_ability", actor="half_ice", slot=2, target="player")
        c.call("step", ticks=1)
        c.call("select_entity", id="half_ice")
        before = c.call("inspect")
        for actor in [source_id["reference"], "half_ice"]:
            assert c.call("despawn_actor", actor=actor)["phase"] == "pending"
        pending_tick = c.call("inspect")["tick"]
        assert c.call("inspect_world")["positions"] == before["runtime_positions"]
        assert len(c.call("inspect_population")["pending"]) == 2
        c.call("save_replay", path="artifacts/population-pending.replay.json")
        assert c.call("run_replay", path="artifacts/population-pending.replay.json")["matched"]
        c.call("step", ticks=1)
        pop = c.call("inspect_population")
        assert [r["outcome"] for r in pop["receipts"]] == ["despawned", "instance_unavailable"]
        assert all(r["instance_id"] == source_id["instance_id"] for r in pop["receipts"])
        state = c.call("inspect")
        assert state["scene"] == authored and state["selection"] is None
        assert "half_ice" not in state["runtime_positions"] and "half_ice" not in state["npcs"]
        assert next(i for i in c.call("inspect_instances")["instances"] if i["authored_id"] == "player") == player_id
        assert c.call("inspect_origins")["reserved"][0]["detached"]
        events = state["triggers"]["events"]
        assert any(r["kind"] == "exit" and r["reason"] == "removed" and r["target_instance"] == source_id["instance_id"] for r in events)
        checks.append("Paused queue applies at tick boundary; duplicate has rejection receipt; source retires without resetting survivor or authoring; trigger exits with removed reason")

        c.call("set_debug_view", npc_vision=True, npc_hearing=True, noise_events=True, npc_routes=True)
        prefix = c.call("inspect")
        c.call("capture", path="artifacts/population-live.png", width=800, height=600)
        c.call("step", ticks=190)
        abilities = c.call("inspect_abilities")
        assert any(r["outcome"] == "impacted" and r["actor_instance"] == source_id["instance_id"] for r in abilities["receipts"])
        assert c.call("inspect_origins")["reserved"] == []
        c.call("save_replay", path="artifacts/population.replay.json")
        assert c.call("run_replay", path="artifacts/population.replay.json")["matched"]
        replay = c.call("inspect_replay", path="artifacts/population.replay.json", tick=prefix["tick"])["snapshot"]
        for key in ["runtime_positions", "abilities", "vitality", "npcs", "population", "triggers"]:
            assert prefix[key] == replay[key], key
        queued = c.call("inspect_replay", path="artifacts/population.replay.json", tick=pending_tick)["snapshot"]
        assert len(queued["population"]["pending"]) == 2 and "half_ice" in queued["runtime_positions"]
        c.call("capture_replay", replay_path="artifacts/population.replay.json", path="artifacts/population-prefix.png", tick=prefix["tick"], width=800, height=600)
        assert (ROOT/"artifacts/population-live.png").read_bytes() == (ROOT/"artifacts/population-prefix.png").read_bytes()
        checks.append("Paid projectile/periodic preserve detached source; leases release; replay50 full, pending boundary, prefix and offscreen pixels agree")

        before = c.call("inspect")
        c.call("activate_ability", actor=source_id["reference"], slot=1, target="player", expected_error=True)
        c.call("despawn_actor", actor=source_id["reference"], expected_error=True)
        assert c.call("inspect") == before
        c.call("reset")
        assert c.call("inspect_population")["pending"] == [] and c.call("inspect_population")["processed"] == 0
        assert "half_ice" in c.call("inspect_world")["positions"]
        c.call("despawn_actor", actor=source_id["reference"], expected_error=True)
        checks.append("Retired references fail atomically; reset restores authored actors and invalidates previous execution tokens")
        result = {"passed": True, "checks": checks, "native_window": window, "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT/"artifacts/population-verification.json").write_text(json.dumps(result, indent=2)+"\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": result["build_id"]}))
    finally:
        c.close()

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--binary", type=Path, default=ROOT/"target/release/ignis")
    p.add_argument("--window", action="store_true")
    args = p.parse_args()
    verify(args.binary.resolve(), args.window)
