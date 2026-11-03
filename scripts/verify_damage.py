#!/usr/bin/env python3
"""Game-defined component damage: authoring, transactions, inspection and replay via MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import definition, loadout
from verify_ability_policy import objective, policy, wait


def mixed(name):
    value = definition(name, "actor", "damage", 100)
    value["tags"] = ["theme"]
    value["effects"] = [{"kind": "damage_components", "recipient": "target", "components": [
        {"amount": 40, "tags": ["fire", "magical"]},
        {"amount": 60, "tags": ["ice", "magical"]}]}]
    return value


def resistance(name, multiplier):
    return {"id": name, "duration_ticks": 216000, "reapply": "refresh", "category": "resistance",
            "modifiers": [{"attribute": "combat.damage_received.multiplier", "tags_all": ["ice"],
                           "operation": {"kind": "multiply", "basis_points": multiplier}}]}


def apply(name, buff, own=False):
    value = definition(name, "self_actor" if own else "actor", "heal", 1)
    value["effects"] = [{"kind": "apply_buff", "recipient": "target", "buff": buff}]
    if own:
        value.update(range=0, cooldown_ticks=3600, costs=[])
    return value


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                "clientInfo": {"name": "ignis-damage-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        schema = json.dumps(c.request("tools/list"))
        assert "damage_components" in schema and "components" in schema
        c.call("generate", recipe={"width": 24, "depth": 20, "seed": 42, "obstacles": 0})
        shot = mixed("projectile")
        shot["delivery"] = {"kind": "projectile", "speed": 8, "half_extent": 0.1, "max_distance": 20}
        dot = mixed("periodic")
        dot["periodic"] = {"interval_ticks": 60, "pulses": 3, "reapply": "refresh"}
        clean = definition("clean", "actor", "heal", 1)
        clean["effects"] = [{"kind": "dispel", "recipient": "target", "category": "resistance",
                             "max_instances": 8, "require_match": True}]
        defs = [mixed("mixed"), shot, dot, apply("resist", "half_ice"), clean,
                definition("heal", "actor", "heal", 500),
                apply("own_half", "half_ice", True), apply("own_immune", "immune_ice", True)]
        catalog = {"resources": [{"id": "energy", "scale": 1}], "abilities": defs,
                   "attributes": {"tags": ["fire", "ice", "magical", "theme"],
                                  "buffs": [resistance("half_ice", 5000), resistance("immune_ice", 0)]}}
        player = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["transform"]["position"] = [0, 0.8, -5]
        player["footsteps"] = None
        player["vitality"] = {"initial": 1000, "maximum": 1000, "scale": 1}
        player["abilities"] = loadout([d["id"] for d in defs[:6]])
        dummy = actor("dummy", [0, 0.8, 1], [0.1, 0.5, 1])
        dummy["vitality"] = {"initial": 1000, "maximum": 1000, "scale": 1}
        dummy["npc"]["objectives"] = [wait()]
        c.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": catalog}] +
               [{"op": "upsert", "entity": e} for e in [player, dummy]])
        c.call("set_debug_view", npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)
        c.call("set_camera", camera={"position": [12, 13, 17], "target": [0, 0.8, 0], "fov_degrees": 48})

        def cast(slot, target="dummy"):
            c.call("activate_ability", actor="player", slot=slot, target=target)
            c.call("step", ticks=1)

        def hp(target="dummy"):
            return c.call("inspect_vitality")["actors"][target]["units"]

        cast(4)
        cast(1)
        result = c.call("inspect_abilities")["receipts"][-1]
        assert len(result["effects"]) == 1 and hp() == 930
        effect = result["effects"][0]
        assert effect["requested"] == 100 and effect["adjusted_amount"] == effect["applied"] == 70
        assert [p["adjusted"] for p in effect["damage_components"]] == [40, 30]
        assert [p["tags"] for p in effect["damage_components"]] == [["fire", "magical"], ["ice", "magical"]]
        prefix = c.call("inspect")
        c.call("capture", path="artifacts/damage-active.png", width=1280, height=900)
        c.call("capture", path="artifacts/damage-repeat.png", width=1280, height=900)
        assert (ROOT / "artifacts/damage-active.png").read_bytes() == (ROOT / "artifacts/damage-repeat.png").read_bytes()
        c.call("step", ticks=1)
        cast(1)
        c.call("save_replay", path="artifacts/damage.replay.json")
        assert c.call("run_replay", path="artifacts/damage.replay.json")["matched"]
        assert c.call("inspect_replay", path="artifacts/damage.replay.json", tick=2)["snapshot"]["abilities"] == prefix["abilities"]
        c.call("capture_replay", replay_path="artifacts/damage.replay.json", path="artifacts/damage-prefix.png", tick=2, width=1280, height=900)
        assert (ROOT / "artifacts/damage-active.png").read_bytes() == (ROOT / "artifacts/damage-prefix.png").read_bytes()
        checks.append("40 fire plus 60 ice becomes 70, one effect/payment, component tags and totals, full/prefix replay and identical GPU pixels")

        before = c.call("inspect")
        for site in ["component", "ability", "filter", "registry", "amount"]:
            bad = copy.deepcopy(catalog)
            if site == "component": bad["abilities"][0]["effects"][0]["components"][0]["tags"] = ["fier"]
            elif site == "ability": bad["abilities"][0]["tags"] = ["unknown"]
            elif site == "filter": bad["attributes"]["buffs"][0]["modifiers"][0]["tags_all"] = ["unknown"]
            elif site == "registry": bad["attributes"]["tags"].append("ice")
            else: bad["abilities"][0]["effects"][0]["components"][0]["amount"] = 0
            c.call("set_ability_catalog", catalog=bad, expected_error=True)
            assert c.call("inspect") == before
        reversed_catalog = copy.deepcopy(catalog)
        reversed_catalog["abilities"][0]["effects"][0]["components"].reverse()
        c.call("set_ability_catalog", catalog=reversed_catalog)
        reversed_saved = c.call("inspect")["scene"]["ability_catalog"]
        cast(4); cast(1)
        assert hp() == 930
        c.call("undo"); assert c.call("inspect")["scene"]["ability_catalog"] == before["scene"]["ability_catalog"]
        c.call("redo"); assert c.call("inspect")["scene"]["ability_catalog"] == reversed_saved
        c.call("set_ability_catalog", catalog=catalog)
        checks.append("Unknown/duplicate tag references and invalid amounts reject atomically; component permutation, undo/redo and explicit registry persist")

        cast(2)
        cast(4)
        c.call("step", ticks=90)
        assert hp() == 930
        impacts = [r for r in c.call("inspect_abilities")["receipts"] if r["outcome"] == "impacted"]
        assert len(impacts) == 1 and impacts[0]["effects"][0]["applied"] == 70
        c.call("reset"); cast(4); cast(3); c.call("step", ticks=181)
        assert hp() == 790
        pulses = [r for r in c.call("inspect_abilities")["receipts"] if r["outcome"] == "periodic_pulse"]
        assert len(pulses) == 3 and all(r["effects"][0]["applied"] == 70 for r in pulses)
        checks.append("Projectile evaluates resistance installed during flight; periodic pulses preserve packet totals and single launch/application cost")

        npcs = []
        for name, x, color, grant in [("unprotected", -5, [0.9, 0.2, 0.1], None),
                                      ("half_ice", 0, [0.1, 0.5, 1], "own_half"),
                                      ("immune_ice", 5, [0.7, 0.2, 0.9], "own_immune")]:
            n = actor(name, [x, 0.8, 1], color)
            n["vitality"] = {"initial": 1000, "maximum": 1000, "scale": 1}
            n["npc"]["objectives"] = [wait()]
            if grant:
                n["abilities"] = loadout([grant])
                n["npc"]["objectives"] = [objective(policy(margin=None)), wait()]
            npcs.append(n)
        c.call("edit_scene", edits=[{"op": "remove", "id": "dummy"}] + [{"op": "upsert", "entity": n} for n in npcs])
        c.call("save_scene", path="examples/npc-damage.scene.json")
        c.call("step", ticks=1)
        for target, expected in [("unprotected", 900), ("half_ice", 930), ("immune_ice", 960)]:
            cast(1, target)
            assert hp(target) == expected
        c.call("capture", path="artifacts/damage-demo.png", width=1280, height=900)
        cast(5, "half_ice"); cast(1, "half_ice"); assert hp("half_ice") == 830
        c.call("save_replay", path="artifacts/damage-demo.replay.json")
        assert c.call("run_replay", path="artifacts/damage-demo.replay.json")["matched"]
        c.call("load_scene", path="examples/npc-damage.scene.json")
        assert c.call("inspect")["scene"]["version"] == SCENE_VERSION and hp("half_ice") == 1000
        checks.append("MCP-authored comparison demo: NPC self-protection yields 100/70/40 damage, dispel restores100, save/load/reset and replay")
        result = {"passed": True, "checks": checks, "native_window": window,
                  "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT / "artifacts/damage-verification.json").write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": result["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
