#!/usr/bin/env python3
"""Independent player controls through the shared dispatcher; headless by default."""
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
                               "clientInfo": {"name": "ignis-actor-controls", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        schema = json.dumps(c.request("tools/list"))
        assert all(op in schema for op in ["bind_control", "set_control_input", "release_control", "inspect_controls"])
        c.call("generate", recipe={"width": 40, "depth": 36, "obstacles": 0, "seed": 122})
        p = next(e for e in c.call("inspect")["scene"]["entities"] if e["body"] == "player")
        p.update(blocks_vision=False, vitality={"initial": 100, "maximum": 100, "scale": 1},
                 footsteps={"surface": "floor", "stride": .4, "radius": 8, "min_speed": .01},
                 abilities={"resources": [{"resource": "energy", "initial": 100, "capacity": 100}],
                            "grants": ["lock", "kill", "return"]})
        p["transform"]["position"] = [-6, .8, -3]
        p["color"] = [1, .4, .1]
        q = copy.deepcopy(p)
        q.update(id="partner", name="partner", color=[.15, .55, 1],
                 locomotion={"walk": {"speed": 2, "noise_multiplier": .5},
                             "sneak": {"speed": .75, "noise_multiplier": .1},
                             "run": {"speed": 5, "noise_multiplier": 1.2}})
        q["transform"]["position"] = [-6, .8, 3]
        def ability(id_, selection, kind, amount):
            return {"id": id_, "selection": selection, "range": 30,
                    "cast_ticks": 0, "cooldown_ticks": 0, "costs": [],
                    "effects": [{"kind": kind, "recipient": "target", "amount": amount}]}
        catalog = {"resources": [{"id": "energy", "scale": 1}], "abilities": [
            {"id": "lock", "selection": "self_actor", "range": 0, "movement": {"mode": "lock"},
             "cast_ticks": 12, "cooldown_ticks": 0,
             "costs": [{"resource": "energy", "units": 7}], "effects": [{"kind": "noise", "radius": 1}]},
            ability("kill", "actor", "damage", 100), ability("return", "actor", "resurrect", 50)]}
        c.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": catalog}]
               + [{"op": "upsert", "entity": e} for e in [p, q]])
        c.call("save_scene", path="artifacts/actor-controls.scene.json")
        def state():
            return {p["actor"]: p for p in c.call("inspect_controls")["players"]}
        def drive(handle, x=0, z=0, mode="walk"):
            return c.call("set_control_input", control=handle, input={"x": x, "z": z, "mode": mode})
        refs = {p["authored_id"]: p["reference"] for p in c.call("inspect_instances")["instances"]}
        a = c.call("bind_control", actor=refs["player"])["control"]
        b = c.call("bind_control", actor="partner")["control"]
        before = c.call("inspect")
        for op, kwargs in [("bind_control", {"actor": "player"}), ("bind_control", {"actor": "floor"}),
                           ("set_input", {"input": {"x": 0, "z": 0}}),
                           ("set_control_input", {"control": a, "input": {"x": 2, "z": 0}}),
                           ("release_control", {"control": "ignis-control:foreign:0000000000000001"})]:
            c.call(op, expected_error=True, **kwargs)
            assert c.call("inspect") == before
        drive(a, x=1, mode="run")
        drive(b, x=1, mode="sneak")
        before = c.call("inspect")["runtime_positions"]
        c.call("step", ticks=60)
        after = c.call("inspect")
        assert abs(after["runtime_positions"]["player"][0] - before["player"][0] - 6) < .05
        assert abs(after["runtime_positions"]["partner"][0] - before["partner"][0] - .75) < .05
        assert state()["partner"]["motion"]["speed"] == .75
        assert after["footsteps"]["player"]["emitted"] > after["footsteps"]["partner"]["emitted"] > 0
        checks.append("Real MCP schema, instance binding, independent profiles/movement/steps and atomic input/exclusivity rejection")

        c.call("release_control", control=a)
        fresh = c.call("bind_control", actor="player")["control"]
        assert fresh != a
        before = c.call("inspect")
        c.call("release_control", control=a, expected_error=True)
        assert c.call("inspect") == before
        a = fresh
        drive(a, x=1)
        drive(b, z=1)
        c.call("activate_ability", actor="player", slot=1, target=None)
        start = c.call("inspect")["runtime_positions"]
        c.call("step", ticks=6)
        after = c.call("inspect")["runtime_positions"]
        assert abs(after["player"][0] - start["player"][0]) < .01
        assert after["partner"][2] > start["partner"][2]
        c.call("step", ticks=10)
        assert c.call("inspect")["runtime_positions"]["player"][0] > start["player"][0]
        c.call("activate_ability", actor="partner", slot=2, target="player")
        c.call("step", ticks=1)
        assert state()["player"]["dead"] and state()["player"]["input"]["x"] == 0
        assert state()["partner"]["input"]["z"] == 1
        c.call("set_control_input", control=a, input={"x": 1, "z": 0}, expected_error=True)
        c.call("activate_ability", actor="partner", slot=3, target="player")
        c.call("step", ticks=2)
        assert not state()["player"]["dead"] and state()["player"]["life"] == 1
        assert state()["player"]["input"]["x"] == 0
        checks.append("Release/rebind cannot reuse handles, cast lock is individual, death clears only victim input and revival preserves neutral control")

        drive(a, z=-1, mode="sneak")
        replay = "artifacts/actor-controls.replay.json"
        c.call("save_replay", path=replay)
        live = c.call("inspect")
        assert c.call("run_replay", path=replay)["matched"]
        assert c.call("run_replay", path=replay)["controls_matched"]
        prefix = c.call("inspect_replay", path=replay, tick=live["tick"])["snapshot"]
        prefix_players = {p["actor"]: p for p in prefix["controls"]["players"]}
        assert all(prefix_players[id_]["input"] == state()[id_]["input"] for id_ in ["player", "partner"])
        assert prefix["runtime_positions"] == live["runtime_positions"]
        c.call("capture", path="artifacts/actor-controls-live.png", width=800, height=600)
        c.call("capture_replay", replay_path=replay, path="artifacts/actor-controls-replay.png", width=800, height=600)
        assert (ROOT / "artifacts/actor-controls-live.png").read_bytes() == (ROOT / "artifacts/actor-controls-replay.png").read_bytes()
        bad = copy.deepcopy(json.loads((ROOT / replay).read_text()))
        bad["control_events"][0]["sequence"] = 9
        (ROOT / "artifacts/actor-controls-invalid.replay.json").write_text(json.dumps(bad))
        before = c.call("inspect")
        c.call("run_replay", path="artifacts/actor-controls-invalid.replay.json", expected_error=True)
        assert c.call("inspect") == before
        checks.append("Final-tick pending input, prefix/full control replay, rejection of tampered sequence and exact offscreen replay pixels")

        bad_entity = copy.deepcopy(p)
        bad_entity["transform"]["size"][0] = -1
        before = c.call("inspect")
        c.call("upsert", entity=bad_entity, expected_error=True)
        assert c.call("inspect") == before
        for op, kwargs in [("reset", {}), ("upsert", {"entity": p}), ("undo", {}), ("redo", {}),
                           ("load_scene", {"path": "artifacts/actor-controls.scene.json"})]:
            old = a
            c.call(op, **kwargs)
            before = c.call("inspect")
            c.call("set_control_input", control=old, input={"x": 1, "z": 0}, expected_error=True)
            assert c.call("inspect") == before
            a = c.call("bind_control", actor="player")["control"]
            assert a != old
        checks.append("Invalid authoring preserves controls; reset/edit/undo/redo/load invalidate old handles without persisting control in Scene")

        c.call("generate", recipe={"width": 20, "depth": 20, "obstacles": 0, "seed": 122})
        c.call("set_input", input={"x": 1, "z": 0, "mode": "walk"})
        c.call("step", ticks=15)
        a = c.call("bind_control", actor="player")["control"]
        assert c.call("inspect")["input"]["x"] == 0
        c.call("set_input", input={"x": 1, "z": 0}, expected_error=True)
        c.call("release_control", control=a)
        c.call("set_input", input={"x": 0, "z": 1})
        c.call("step", ticks=15)
        c.call("save_replay", path="artifacts/actor-controls-legacy.replay.json")
        assert c.call("run_replay", path="artifacts/actor-controls-legacy.replay.json")["matched"]
        checks.append("Single-player legacy remains usable before/after explicit control and shares physics/replay")
        result = {"passed": True, "checks": checks, "native_window": window,
                  "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT / "artifacts/actor-controls-verification.json").write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": result["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    p.add_argument("--window", action="store_true")
    a = p.parse_args()
    verify(a.binary.resolve(), a.window)
