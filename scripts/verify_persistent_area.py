#!/usr/bin/env python3
"""Author and verify fixed persistent volumes through the shared MCP dispatcher."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                 "clientInfo": {"name": "ignis-persistent-area-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        assert "end_on_source_death" in json.dumps(c.request("tools/list"))
        c.call("load_scene", path="examples/npc-area.scene.json")
        scene = c.call("inspect")["scene"]
        catalog = copy.deepcopy(scene["ability_catalog"])
        schedule = {"duration_ticks": 360, "interval_ticks": 60, "apply_on_spawn": True,
                    "reapply": "refresh", "end_on_source_death": False}
        for index in [0, 1, 2]:
            catalog["abilities"][index]["area"]["persistence"] = dict(schedule)
        catalog["abilities"][0]["area"]["persistence"]["reapply"] = "independent"
        own = catalog["abilities"][2]
        own.update(cast_ticks=0)
        own["area"].update(radius=2, include_source=True)
        own["area"]["persistence"].update(reapply="independent", end_on_source_death=True)
        c.call("set_ability_catalog", catalog=catalog)
        c.call("set_debug_view", npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)
        c.call("save_scene", path="examples/npc-persistent-area.scene.json")

        def state(): return c.call("inspect_abilities")
        def active(): return state()["persistent_areas"]["active"]
        def hp(name): return c.call("inspect_vitality")["actors"][name]["units"]
        def energy(): return state()["actors"]["player"]["resources"][0]["units"]
        def damage(expected):
            actual = [hp(n) for n in ["unprotected", "half_ice", "immune_ice", "covered"]]
            assert actual == expected, (actual, expected)
        def cast_point(position=None):
            c.call("aim_ability", actor="player", slot=2,
                   aim={"kind": "point", "position": position or [0, 0.8, 1]})
            c.call("step", ticks=16)

        c.call("step", ticks=1)
        c.call("aim_ability", actor="player", slot=2, aim={"kind": "point", "position": [0, 0.8, 1]})
        c.call("save_replay", path="artifacts/persistent-area-pending.replay.json")
        assert len(c.call("inspect_replay", path="artifacts/persistent-area-pending.replay.json")["snapshot"]["abilities"]["pending_inputs"]) == 1
        c.call("step", ticks=16)
        damage([900, 930, 960, 1000]); assert energy() == 995
        before = c.call("inspect"); tick = before["tick"]
        original = active()[0]; assert original["pulses"] == 1 and original["expires_at"] - original["created_tick"] == 360
        c.call("capture", path="artifacts/persistent-area-active.png", width=1280, height=900)
        c.call("capture", path="artifacts/persistent-area-repeat.png", width=1280, height=900)
        assert (ROOT / "artifacts/persistent-area-active.png").read_bytes() == (ROOT / "artifacts/persistent-area-repeat.png").read_bytes()
        c.call("step", ticks=59); damage([900, 930, 960, 1000])
        c.call("step", ticks=1); damage([800, 860, 920, 1000])
        c.call("step", ticks=300); damage([400, 580, 760, 1000]); assert not active() and energy() == 995
        assert state()["persistent_areas"]["receipts"][-1]["outcome"] == "expired"
        c.call("save_replay", path="artifacts/persistent-area.replay.json")
        assert c.call("run_replay", path="artifacts/persistent-area.replay.json")["matched"]
        assert c.call("inspect_replay", path="artifacts/persistent-area.replay.json", tick=tick)["snapshot"]["abilities"] == before["abilities"]
        c.call("capture_replay", replay_path="artifacts/persistent-area.replay.json", path="artifacts/persistent-area-prefix.png", tick=tick, width=1280, height=900)
        assert (ROOT / "artifacts/persistent-area-prefix.png").read_bytes() == (ROOT / "artifacts/persistent-area-active.png").read_bytes()
        checks.append("Pending/prepared/active lifecycle, six pulses100/70/40, wall exclusion, expiration without extra pulse, unique cost and deterministic replay/prefix/pixels")

        c.call("reset"); c.call("step", ticks=1); cast_point()
        first = active()[0]["execution_id"]
        cast_point([0, 0.8, 2]); assert len(active()) == 1 and all(abs(a-b) < 1e-5 for a,b in zip(active()[0]["center"], [0, 0.8, 2])), active()
        assert any(r["execution_id"] == first and r["outcome"] == "refreshed" and r["replaced_by"] == active()[0]["execution_id"]
                   for r in state()["persistent_areas"]["receipts"])
        assert energy() == 990
        c.call("reset"); c.call("step", ticks=1)
        c.call("activate_ability", actor="player", slot=3, target="player"); c.call("step", ticks=1)
        fixed = active()[0]["center"]; assert hp("player") == 900
        c.call("set_input", input={"x": -1, "z": 0}); c.call("step", ticks=61)
        assert hp("player") == 900 and active()[0]["center"] == fixed
        c.call("set_input", input={"x": 0, "z": 0})
        c.call("activate_ability", actor="player", slot=3, target="player"); c.call("step", ticks=1)
        assert len(active()) == 2 and active()[0]["center"] != active()[1]["center"]
        assert hp("player") == 800 and energy() == 990
        c.call("save_replay", path="artifacts/persistent-area-movement.replay.json")
        assert c.call("run_replay", path="artifacts/persistent-area-movement.replay.json")["matched"]
        checks.append("Refresh replaces/repositions with linked IDs; independent self-centered volumes stay fixed after walking out, later pulses exclude escaped player")

        c.call("reset"); c.call("step", ticks=1)
        c.call("activate_ability", actor="player", slot=1, target="half_ice"); c.call("step", ticks=16)
        assert not active() and len(state()["projectiles"]["active"]) == 1 and energy() == 995
        for _ in range(90):
            c.call("step", ticks=1)
            if active(): break
        assert active() and active()[0]["created_tick"] > 16
        assert active()[0]["expires_at"] - active()[0]["created_tick"] == 360
        assert state()["projectiles"]["receipts"][-1]["outcome"] == "hit"
        damage([900, 930, 960, 1000])
        c.call("activate_ability", actor="player", slot=4, target="player"); c.call("step", ticks=1)
        c.call("step", ticks=59); damage([790, 850, 910, 1000]); assert energy() == 990
        c.call("save_replay", path="artifacts/persistent-area-projectile.replay.json")
        assert c.call("run_replay", path="artifacts/persistent-area-projectile.replay.json")["matched"]
        checks.append("Projectile travel precedes volume lifetime; shared outgoing buff acquired after placement affects next pulse dynamically")

        c.call("reset"); baseline = c.call("inspect")
        for change in [{"duration_ticks": 0}, {"interval_ticks": 0}, {"interval_ticks": 3601},
                       {"duration_ticks": 60, "interval_ticks": 60, "apply_on_spawn": False}]:
            bad = copy.deepcopy(catalog); bad["abilities"][1]["area"]["persistence"].update(change)
            c.call("set_ability_catalog", catalog=bad, expected_error=True)
            assert c.call("inspect") == baseline
        changed = copy.deepcopy(catalog); changed["abilities"][1]["area"]["persistence"]["apply_on_spawn"] = False
        c.call("set_ability_catalog", catalog=changed); c.call("step", ticks=1); cast_point(); damage([1000, 1000, 1000, 1000])
        c.call("undo"); c.call("step", ticks=1); cast_point(); damage([900, 930, 960, 1000])
        c.call("redo"); c.call("step", ticks=1); cast_point(); damage([1000, 1000, 1000, 1000])
        c.call("load_scene", path="examples/npc-persistent-area.scene.json")
        assert c.call("inspect")["scene"]["version"] == SCENE_VERSION and not active()
        c.call("step", ticks=1)
        c.call("aim_ability_at", actor="player", slot=2, x=640, y=450, width=1280, height=900, plane_y=0.8)
        c.call("step", ticks=16); damage([900, 930, 960, 1000])
        before = state(); c.call("set_debug_view", npc_routes=False)
        c.call("capture", path="artifacts/persistent-area-hidden.png", width=800, height=600)
        assert state() == before
        c.call("set_debug_view", npc_routes=True)
        c.call("capture", path="artifacts/persistent-area-demo.png", width=1280, height=900)
        c.call("save_replay", path="artifacts/persistent-area-demo.replay.json")
        assert c.call("run_replay", path="artifacts/persistent-area-demo.replay.json")["matched"]
        checks.append("Schema failures preserve active scene/history; edit/undo/redo/load/reset, cursor command, guide toggles and final demo remain reproducible")
        result = {"passed": True, "checks": checks, "native_window": window,
                  "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT / "artifacts/persistent-area-verification.json").write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": result["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
