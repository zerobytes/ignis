#!/usr/bin/env python3
"""Source-anchored pulses, real locomotion, spatial NPC aura and offscreen replay."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import loadout


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                 "clientInfo": {"name": "ignis-aura-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        assert '"anchor"' in json.dumps(c.request("tools/list"))
        c.call("load_scene", path="examples/npc-persistent-area.scene.json")
        scene = c.call("inspect")["scene"]
        catalog = copy.deepcopy(scene["ability_catalog"])
        harm = copy.deepcopy(catalog["abilities"][2])
        harm.update(id="harm_aura", selection="self_actor", range=0, relation="hostile", cast_ticks=0)
        harm["area"].update(radius=3, include_source=False)
        harm["area"]["persistence"].update(anchor="source", duration_ticks=600, reapply="refresh", end_on_source_death=True)
        heal = copy.deepcopy(harm)
        heal.update(id="heal_aura", relation="allied", effects=[{"kind": "heal", "recipient": "target", "amount": 60}])
        heal["area"].update(radius=4, include_source=True)
        catalog["abilities"][0:2] = [harm, heal]
        npc_heal = copy.deepcopy(heal); npc_heal.update(id="npc_heal_aura", cooldown_ticks=600)
        catalog["abilities"].append(npc_heal)
        byid = {e["id"]: e for e in scene["entities"]}
        player = byid["player"]; player["faction"] = "team"
        player["abilities"] = loadout([d["id"] for d in catalog["abilities"][:6]])
        targets = [byid[n] for n in ["unprotected", "half_ice", "immune_ice", "covered"]]
        for e in targets: e["faction"] = "enemies"
        flyer = actor("aura_flyer", [-6, 2.8, -4], [0.15, 0.85, 0.45])
        flyer["abilities"] = loadout(["npc_heal_aura"])
        flyer["vitality"] = {"initial": 1000, "maximum": 1000, "scale": 1}
        flyer["npc"].update(mobility={"walk": False, "fly_speed": 3},
            local_connections={"walk": False, "fly": True, "max_distance": 8},
            patrol={"route": {"kind": "points_3d", "points": [[-6, 2.8, -4], [-6, 5.8, -4], [-2, 4.8, -4]]},
                    "traversal": "loop", "wait_ticks": 12, "travel_ticks": 1200, "retry_ticks": 60, "arrival_distance": 0.2},
            objectives=[{"priority": 2, "when": "always", "action": {"kind": "use_ability", "slot": 1, "retry_ticks": 60}},
                        {"priority": 0, "when": "always", "action": {"kind": "patrol"}}])
        relations = [{"from": "team", "to": "enemies", "attitude": "hostile"},
                     {"from": "enemies", "to": "team", "attitude": "hostile"}]
        c.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": catalog},
                                    {"op": "set_faction_relations", "relations": relations}] +
               [{"op": "upsert", "entity": e} for e in [player, *targets, flyer]])
        c.call("set_camera", camera={"position": [13, 15, 19], "target": [-1, 2, -1], "fov_degrees": 48})
        c.call("set_debug_view", npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)
        c.call("save_scene", path="examples/npc-aura.scene.json")

        def state(): return c.call("inspect_abilities")
        def active(owner): return [i for i in state()["persistent_areas"]["active"] if i["actor"] == owner]
        def hp(name): return c.call("inspect_vitality")["actors"][name]["units"]
        def energy(): return state()["actors"]["player"]["resources"][0]["units"]
        def activate(n):
            c.call("activate_ability", actor="player", slot=n, target="player"); c.call("step", ticks=1)
        def move(x, z, ticks):
            c.call("set_input", input={"x": x, "z": z}); c.call("step", ticks=ticks)
        def healths(): return [hp(n) for n in ["unprotected", "half_ice", "immune_ice", "covered"]]
        def matching_center(owner, snapshot):
            i = next(i for i in snapshot["abilities"]["persistent_areas"]["active"] if i["actor"] == owner)
            assert i["center"] == snapshot["runtime_positions"][owner]
            assert i["anchor_actor"] == owner

        c.call("step", ticks=1)
        c.call("activate_ability", actor="player", slot=1, target="player")
        c.call("save_replay", path="artifacts/aura-pending.replay.json")
        assert len(c.call("inspect_replay", path="artifacts/aura-pending.replay.json")["snapshot"]["abilities"]["pending_inputs"]) == 1
        c.call("step", ticks=1); first = active("player")[0]
        assert healths() == [1000]*4
        move(0, 1, 60); assert healths() == [900, 930, 960, 1000], healths()
        live = c.call("inspect"); tick = live["tick"]
        matching_center("player", live); matching_center("aura_flyer", live)
        current = active("player")[0]
        assert (current["execution_id"], current["expires_at"]) == (first["execution_id"], first["expires_at"])
        assert energy() == 995
        c.call("capture", path="artifacts/aura-active.png", width=1280, height=900)
        c.call("capture", path="artifacts/aura-repeat.png", width=1280, height=900)
        assert (ROOT/"artifacts/aura-active.png").read_bytes() == (ROOT/"artifacts/aura-repeat.png").read_bytes()
        c.call("step", ticks=1)
        move(0, -1, 60)
        c.call("save_replay", path="artifacts/aura.replay.json")
        assert c.call("run_replay", path="artifacts/aura.replay.json")["matched"]
        assert c.call("inspect_replay", path="artifacts/aura.replay.json", tick=tick)["snapshot"]["abilities"] == live["abilities"]
        c.call("capture_replay", replay_path="artifacts/aura.replay.json", path="artifacts/aura-prefix.png", tick=tick, width=1280, height=900)
        assert (ROOT/"artifacts/aura-prefix.png").read_bytes() == (ROOT/"artifacts/aura-active.png").read_bytes()
        assert healths() == [900, 930, 960, 1000]
        checks.append("Player aura physically moves into and out of hostile group;100/70/40, same lifetime/ID and one cost; post-physics centers, pending/full/prefix replay and identical pixels")

        c.call("reset"); c.call("step", ticks=1); activate(3)
        fixed = active("player")[0]["center"]; assert hp("player") == 900
        move(0, 1, 60); assert healths() == [1000]*4 and active("player")[0]["center"] == fixed
        c.call("set_input", input={"x": 0, "z": 0}); activate(2)
        assert hp("player") == 960 and energy() == 990
        assert healths() == [1000]*4
        old = next(i["execution_id"] for i in active("player") if i["anchor_actor"] == "player")
        activate(2)
        moving = [i for i in active("player") if i["anchor_actor"] == "player"]
        assert len(moving) == 1 and moving[0]["execution_id"] != old
        assert hp("player") == 1000
        checks.append("Fixed zone stays behind while source aura heals its carrier; same definition refresh replaces execution; faction filter remains shared")

        c.call("reset"); c.call("step", ticks=1)
        assert active("aura_flyer") and active("aura_flyer")[0]["anchor_actor"] == "aura_flyer"
        height = 0
        for _ in range(18):
            c.call("step", ticks=10); snap = c.call("inspect")
            matching_center("aura_flyer", snap)
            height = max(height, snap["runtime_positions"]["aura_flyer"][1])
        assert height > 5, height
        assert c.call("inspect")["npcs"]["aura_flyer"]["memory"] is None
        c.call("save_replay", path="artifacts/aura-flight.replay.json")
        assert c.call("run_replay", path="artifacts/aura-flight.replay.json")["matched"]
        checks.append("NPC automatically activates the same executor then patrols in XYZ; aura follows actual flight without gaining target knowledge")

        before = c.call("inspect")
        for selection in ["actor", "spatial"]:
            bad = copy.deepcopy(catalog); bad["abilities"][0].update(selection=selection, range=12)
            c.call("set_ability_catalog", catalog=bad, expected_error=True); assert c.call("inspect") == before
        bad = copy.deepcopy(catalog); bad["abilities"][0]["delivery"] = {"kind": "projectile", "speed": 8, "half_extent": 0.1, "max_distance": 20}
        c.call("set_ability_catalog", catalog=bad, expected_error=True); assert c.call("inspect") == before
        fixed_catalog = copy.deepcopy(catalog); fixed_catalog["abilities"][0]["area"]["persistence"]["anchor"] = "fixed"
        c.call("set_ability_catalog", catalog=fixed_catalog); activate(1); p = active("player")[0]["center"]
        move(1, 0, 20); assert active("player")[0]["center"] == p
        c.call("undo"); activate(1); move(1, 0, 20); matching_center("player", c.call("inspect"))
        c.call("redo"); activate(1); p = active("player")[0]["center"]; move(1, 0, 20); assert active("player")[0]["center"] == p
        c.call("load_scene", path="examples/npc-aura.scene.json")
        assert c.call("inspect")["scene"]["version"] == SCENE_VERSION and not state()["persistent_areas"]["active"]
        c.call("step", ticks=1); activate(1); move(0, 1, 60)
        before = state(); c.call("set_debug_view", npc_routes=False)
        c.call("capture", path="artifacts/aura-hidden.png", width=800, height=600); assert state() == before
        c.call("set_debug_view", npc_routes=True)
        c.call("capture", path="artifacts/aura-demo.png", width=1280, height=900)
        c.call("save_replay", path="artifacts/aura-demo.replay.json")
        assert c.call("run_replay", path="artifacts/aura-demo.replay.json")["matched"]
        checks.append("Invalid anchor/delivery edits atomic; fixed/source undo/redo, load/reset and guide toggles preserve state and deterministic demo")
        result = {"passed": True, "checks": checks, "native_window": window,
                  "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT/"artifacts/aura-verification.json").write_text(json.dumps(result, indent=2)+"\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": result["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    p = argparse.ArgumentParser(); p.add_argument("--binary", type=Path, default=ROOT/"target/release/ignis"); p.add_argument("--window", action="store_true")
    args = p.parse_args(); verify(args.binary.resolve(), args.window)
