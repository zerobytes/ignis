#!/usr/bin/env python3
"""Instant XYZ areas, atomic recipients and an authored comparison demo via MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_ability_policy import wait
from verify_vitality import definition, loadout


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                 "clientInfo": {"name": "ignis-area-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        assert "max_targets" in json.dumps(c.request("tools/list"))
        c.call("load_scene", path="examples/npc-outgoing.scene.json")
        scene = c.call("inspect")["scene"]
        catalog = scene["ability_catalog"]
        area = {"radius": 4.5, "max_targets": 9, "include_source": False, "occlusion": True}
        shot = copy.deepcopy(catalog["abilities"][1])
        shot.update(id="explosion", area=area, cast_ticks=15)
        point = copy.deepcopy(catalog["abilities"][0])
        point.update(id="point_area", selection="spatial", area=area, cast_ticks=15)
        own = copy.deepcopy(point)
        own.update(id="own_area", selection="self_actor", range=0)
        heal = definition("heal_area", "self_actor", "heal", 500)
        heal.update(range=0, area={**area, "radius": 8, "include_source": True})
        clean = copy.deepcopy(catalog["abilities"][4])
        clean.update(id="clean_area", selection="spatial", area=area)
        clean["effects"][0].update(category="resistance", require_match=False)
        catalog["abilities"][:6] = [shot, point, own, catalog["abilities"][3], heal, clean]
        byid = {e["id"]: e for e in scene["entities"]}
        player = byid["player"]
        player["abilities"] = loadout([d["id"] for d in catalog["abilities"][:6]])
        player["transform"]["position"] = [0, 0.8, -5]
        player["footsteps"] = None
        group = [byid[n] for n in ["unprotected", "half_ice", "immune_ice"]]
        for e, x in zip(group, [-2, 0, 2]):
            e["transform"]["position"] = [x, 0.8, 1]
        covered = actor("covered", [3.8, 0.8, 1], [0.45, 0.45, 0.48])
        covered["vitality"] = {"initial": 1000, "maximum": 1000, "scale": 1}
        covered["npc"]["objectives"] = [wait()]
        wall = {"id": "area_cover", "name": "Area cover", "body": "fixed", "color": [0.2, 0.55, 0.46],
                "transform": {"position": [2.8, 1, 1], "size": [0.2, 2, 5]}}
        c.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": catalog}] +
               [{"op": "upsert", "entity": e} for e in [player, *group, covered, wall]])
        c.call("set_camera", camera={"position": [12, 13, 17], "target": [0, 0.8, 0], "fov_degrees": 48})
        c.call("set_debug_view", npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)
        c.call("save_scene", path="examples/npc-area.scene.json")

        def state(): return c.call("inspect_abilities")
        def hp(name): return c.call("inspect_vitality")["actors"][name]["units"]
        def cast_point(slot=2, ticks=16):
            c.call("aim_ability", actor="player", slot=slot, aim={"kind": "point", "position": [0, 0.8, 1]})
            c.call("step", ticks=ticks)
        def energy(): return state()["actors"]["player"]["resources"][0]["units"]
        def damage(expected):
            assert [hp(n) for n in ["unprotected", "half_ice", "immune_ice", "covered"]] == expected

        c.call("step", ticks=1)
        c.call("aim_ability", actor="player", slot=2, aim={"kind": "point", "position": [0, 0.8, 1]})
        c.call("save_replay", path="artifacts/area-pending.replay.json")
        pending = c.call("inspect_replay", path="artifacts/area-pending.replay.json")["snapshot"]
        assert len(pending["abilities"]["pending_inputs"]) == 1
        c.call("step", ticks=1)
        preparing = c.call("inspect"); tick = preparing["tick"]
        c.call("capture", path="artifacts/area-preparing.png", width=1280, height=900)
        c.call("step", ticks=15)
        damage([900, 930, 960, 1000]); assert energy() == 995
        receipt = state()["areas"]["receipts"][-1]
        candidates = {v["actor"]: v for v in receipt["candidates"]}
        assert candidates["covered"]["status"] == "occluded"
        assert candidates["player"]["status"] == "source_excluded"
        applied = [r for r in state()["receipts"] if r["outcome"] == "area_applied"]
        assert len(applied) == 3 and all(not r["costs"] for r in applied)
        c.call("save_replay", path="artifacts/area.replay.json")
        assert c.call("run_replay", path="artifacts/area.replay.json")["matched"]
        assert c.call("inspect_replay", path="artifacts/area.replay.json", tick=tick)["snapshot"]["abilities"] == preparing["abilities"]
        c.call("capture_replay", replay_path="artifacts/area.replay.json", path="artifacts/area-prefix.png", tick=tick, width=1280, height=900)
        assert (ROOT / "artifacts/area-prefix.png").read_bytes() == (ROOT / "artifacts/area-preparing.png").read_bytes()
        c.call("capture", path="artifacts/area-impact.png", width=1280, height=900)
        c.call("capture", path="artifacts/area-repeat.png", width=1280, height=900)
        assert (ROOT / "artifacts/area-impact.png").read_bytes() == (ROOT / "artifacts/area-repeat.png").read_bytes()
        c.call("set_debug_view", npc_routes=False)
        before = c.call("inspect_abilities")
        c.call("capture", path="artifacts/area-hidden.png", width=800, height=600)
        assert state() == before
        c.call("set_debug_view", npc_routes=True)
        checks.append("XYZ point area applies100/70/40 behind one cost, wall excludes covered target, pending/preparing/full replay and identical prefix/impact pixels")

        c.call("activate_ability", actor="player", slot=4, target="player"); c.call("step", ticks=1)
        cast_point(); damage([790, 850, 910, 1000])
        c.call("activate_ability", actor="player", slot=1, target="half_ice"); c.call("step", ticks=70)
        damage([680, 770, 860, 1000])
        assert state()["projectiles"]["receipts"][-1]["outcome"] == "hit"
        c.call("activate_ability", actor="player", slot=5, target="player"); c.call("step", ticks=1)
        damage([1000, 1000, 1000, 1000])
        cast_point(6, 1); assert not any(b["buff"] in ["half_ice", "immune_ice"] for b in state()["buffs"]["active"])
        cast_point(); damage([890, 890, 890, 1000])
        checks.append("Shared outgoing/incoming modifiers, projectile contact area, self-centered group healing and spatial group dispel")

        c.call("reset"); c.call("step", ticks=1)
        changed = copy.deepcopy(catalog)
        changed["abilities"][1]["effects"].append({"kind": "resource", "recipient": "target", "resource": "energy", "delta": -1})
        victim = copy.deepcopy(group[0]); victim["abilities"] = None
        c.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": changed}, {"op": "upsert", "entity": victim}])
        c.call("step", ticks=1); cast_point()
        damage([1000, 930, 960, 1000]); assert energy() == 995
        failed = next(v for v in state()["areas"]["receipts"][-1]["candidates"] if v["actor"] == "unprotected")
        assert failed["status"] == "effect_failed" and failed["failure"] == "missing_resource"
        c.call("undo"); c.call("step", ticks=1); cast_point(); damage([900, 930, 960, 1000])
        c.call("redo"); c.call("step", ticks=1); cast_point(); damage([1000, 930, 960, 1000])
        c.call("load_scene", path="examples/npc-area.scene.json")
        checks.append("Per-recipient rollback preserves successful neighbors and one charge; undo/redo recompiles correct area behavior")

        baseline = c.call("inspect")
        for change in [{"radius": 0}, {"max_targets": 1025}]:
            bad = copy.deepcopy(catalog); bad["abilities"][1]["area"].update(change)
            c.call("set_ability_catalog", catalog=bad, expected_error=True)
            assert c.call("inspect") == baseline
        c.call("aim_ability", actor="player", slot=2, aim={"kind": "direction", "vector": [1, 0, 0]}, expected_error=True)
        assert c.call("inspect") == baseline
        c.call("step", ticks=1); cast_point()
        c.call("remove", id="area_cover"); c.call("step", ticks=1); cast_point(); assert hp("covered") == 900
        c.call("undo"); c.call("step", ticks=1); cast_point(); assert hp("covered") == 1000
        c.call("load_scene", path="examples/npc-area.scene.json")
        assert c.call("inspect")["scene"]["version"] == SCENE_VERSION
        c.call("step", ticks=1)
        cursor = c.call("aim_ability_at", actor="player", slot=2, x=640, y=450, width=1280, height=900, plane_y=0.8)
        assert abs(cursor["aim_point"][1] - 0.8) < 0.0001
        c.call("step", ticks=16); damage([900, 930, 960, 1000])
        c.call("capture", path="artifacts/area-demo.png", width=1280, height=900)
        c.call("save_replay", path="artifacts/area-demo.replay.json")
        assert c.call("run_replay", path="artifacts/area-demo.replay.json")["matched"]
        checks.append("Invalid config/aim atomic; cover removal/undo, save/load/reset, resize and final authored demo reproduce")
        result = {"passed": True, "checks": checks, "native_window": window,
                  "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT / "artifacts/area-verification.json").write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": result["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
