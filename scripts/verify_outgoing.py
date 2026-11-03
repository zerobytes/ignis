#!/usr/bin/env python3
"""Outgoing component modifiers, dynamic delivery and inspectable demo through MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_damage import apply


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                "clientInfo": {"name": "ignis-outgoing-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        c.call("load_scene", path="examples/npc-damage.scene.json")
        catalog = c.call("inspect")["scene"]["ability_catalog"]
        catalog["attributes"]["buffs"].append({"id": "power", "duration_ticks": 600,
            "reapply": "refresh", "category": "power", "modifiers": [
                {"attribute": "combat.damage_dealt.multiplier", "tags_all": ["fire"],
                 "operation": {"kind": "multiply", "basis_points": 12500}}]})
        boost = apply("power_self", "power", True)
        boost.update(cooldown_ticks=0, costs=[{"resource": "energy", "units": 5}])
        catalog["abilities"][3] = boost
        catalog["abilities"][4]["effects"][0]["category"] = "power"
        player = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["abilities"]["grants"][3] = "power_self"
        c.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": catalog}, {"op": "upsert", "entity": player}])
        c.call("set_debug_view", npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)

        def cast(slot, target="half_ice"):
            c.call("activate_ability", actor="player", slot=slot, target="player" if slot == 4 else target)
            c.call("step", ticks=1)

        def hp(target="half_ice"):
            return c.call("inspect_vitality")["actors"][target]["units"]

        def state():
            return c.call("inspect_abilities")

        def power(tags):
            return c.call("inspect_attribute", entity="player", attribute="combat.damage_dealt.multiplier",
                          ability="mixed", tags=tags)["effective"]["units"]

        # Existing NPCs first install their self-resistances.
        c.call("step", ticks=1)
        for target, value in [("unprotected", 100), ("half_ice", 70), ("immune_ice", 40)]:
            cast(1, target)
            assert hp(target) == 1000 - value
        cast(4)
        assert power(["fire"]) == 12500 and power(["ice"]) == power([]) == 10000
        for target, value in [("unprotected", 210), ("half_ice", 150), ("immune_ice", 90)]:
            cast(1, target)
            assert hp(target) == 1000 - value
        r = state()["receipts"][-1]["effects"][0]
        assert r["damage_components"][0]["dealt_multiplier"] == 12500
        assert r["damage_components"][1]["received_multiplier"] == 0
        prefix = c.call("inspect")
        tick = prefix["tick"]
        c.call("capture", path="artifacts/outgoing-active.png", width=1280, height=900)
        c.call("capture", path="artifacts/outgoing-repeat.png", width=1280, height=900)
        assert (ROOT / "artifacts/outgoing-active.png").read_bytes() == (ROOT / "artifacts/outgoing-repeat.png").read_bytes()
        # Separate future input from the captured prefix; same-tick replay includes pending inputs.
        c.call("step", ticks=1)
        cast(5, "player")
        assert power(["fire"]) == 10000
        cast(1)
        assert hp() == 780
        c.call("save_replay", path="artifacts/outgoing.replay.json")
        assert c.call("run_replay", path="artifacts/outgoing.replay.json")["matched"]
        assert c.call("inspect_replay", path="artifacts/outgoing.replay.json", tick=tick)["snapshot"]["abilities"] == prefix["abilities"]
        c.call("capture_replay", replay_path="artifacts/outgoing.replay.json", path="artifacts/outgoing-prefix.png", tick=tick, width=1280, height=900)
        assert (ROOT / "artifacts/outgoing-prefix.png").read_bytes() == (ROOT / "artifacts/outgoing-active.png").read_bytes()
        checks.append("100/70/40 becomes110/80/50, explicit outgoing/incoming context, dispel, full/prefix replay and identical offscreen pixels")

        before = c.call("inspect")
        invalid = copy.deepcopy(catalog)
        invalid["attributes"]["definitions"] = [{"key": "combat.damage_dealt.multiplier", "default": {"kind": "number", "units": 10000}}]
        c.call("set_ability_catalog", catalog=invalid, expected_error=True)
        assert c.call("inspect") == before
        invalid = copy.deepcopy(catalog)
        invalid["attributes"]["bases"] = [{"entity": "player", "attribute": "combat.damage_dealt.multiplier", "value": {"kind": "number", "units": -1}}]
        c.call("set_ability_catalog", catalog=invalid, expected_error=True)
        assert c.call("inspect") == before
        changed = copy.deepcopy(catalog)
        changed["attributes"]["buffs"][-1]["modifiers"][0]["operation"]["basis_points"] = 20000
        c.call("set_ability_catalog", catalog=changed)
        c.call("step", ticks=1); cast(4); cast(1)
        assert hp() == 890
        c.call("undo")
        assert c.call("inspect")["scene"]["ability_catalog"] == before["scene"]["ability_catalog"]
        c.call("redo"); cast(4); c.call("step", ticks=1); cast(1)
        assert hp() == 890
        c.call("set_ability_catalog", catalog=catalog)
        checks.append("Reserved key and invalid base reject atomically; changing power, undo/redo recompiles without stale factors")

        c.call("step", ticks=1); cast(2); cast(4); c.call("step", ticks=90)
        assert hp() == 920
        impact = next(r for r in state()["receipts"] if r["outcome"] == "impacted")
        assert impact["effects"][0]["damage_components"][0]["dealt_multiplier"] == 12500
        c.call("reset"); c.call("step", ticks=1); cast(3); c.call("step", ticks=60)
        assert hp() == 930
        cast(4); c.call("step", ticks=60)
        assert hp() == 850
        cast(5, "player"); c.call("step", ticks=60)
        assert hp() == 780
        c.call("reset"); cast(4); c.call("step", ticks=598)
        assert power(["fire"]) == 12500
        c.call("step", ticks=1)
        assert power(["fire"]) == 10000
        checks.append("Projectile gains power during flight, periodic70/80/70 with gain/dispel and exact600-tick expiration")

        c.call("reset")
        c.call("save_scene", path="examples/npc-outgoing.scene.json")
        c.call("load_scene", path="examples/npc-outgoing.scene.json")
        assert c.call("inspect")["scene"]["version"] == SCENE_VERSION
        assert power(["fire"]) == 10000 and hp() == 1000
        c.call("step", ticks=1); cast(4); cast(1)
        assert hp() == 920
        c.call("capture", path="artifacts/outgoing-demo.png", width=1280, height=900)
        c.call("save_replay", path="artifacts/outgoing-demo.replay.json")
        assert c.call("run_replay", path="artifacts/outgoing-demo.replay.json")["matched"]
        checks.append("MCP-authored self-buff demo saved/loaded/reset with600 ticks,6 slots, shared NPC protections and reproducible build")
        result = {"passed": True, "checks": checks, "native_window": window,
                  "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT / "artifacts/outgoing-verification.json").write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": result["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
