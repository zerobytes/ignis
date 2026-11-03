#!/usr/bin/env python3
"""Critical remote lease/ownership/life and shared game-command integration, no window."""
import argparse
import copy
import json
import math
import time
from pathlib import Path
from verify_lab import Client, ROOT
from verify_network import wait
from verify_selection import project


def verify(binary, window=False):
    assert not window
    host, client = Client(binary), Client(binary)
    checks = []
    try:
        for c in (host, client):
            c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "remote-controls", "version": "1"}})
            c.request("notifications/initialized", notification=True)
        assert "release_multiplayer_input" in json.dumps(client.request("tools/list"))
        host.call("generate", recipe={"width": 32, "depth": 32, "obstacles": 0, "seed": 144})
        p = next(e for e in host.call("inspect")["scene"]["entities"] if e.get("body") == "player")
        p.update(footsteps=None, vitality={"initial": 100, "maximum": 100, "scale": 1}, abilities={"resources": [{"resource": "energy", "initial": 100, "capacity": 100}], "grants": ["hit", "long", "spatial", "kill", "revive", "self"]})
        p["transform"]["position"] = [-4, .8, 0]
        q = copy.deepcopy(p)
        q.update(id="partner", name="partner")
        q["transform"]["position"] = [4, .8, 0]
        def ability(id_, amount, cost=0, cast=0, kind="damage"):
            return {"id": id_, "selection": "actor", "range": 20, "cast_ticks": cast, "cooldown_ticks": 0, "costs": [{"resource": "energy", "units": cost}] if cost else [], "effects": [{"kind": kind, "recipient": "target", "amount": amount}]}
        spatial = ability("spatial", 3, 1)
        spatial.update(selection="spatial", range=0, delivery={"kind": "projectile", "speed": 1, "half_extent": .1, "max_distance": 20})
        catalog = {"resources": [{"id": "energy", "scale": 1}], "abilities": [ability("hit", 5, 1), ability("long", 7, 3, 120), spatial, ability("kill", 1000), ability("revive", 50, kind="resurrect"), {"id": "self", "selection": "self_actor", "range": 0, "cast_ticks": 0, "cooldown_ticks": 0, "costs": [], "effects": [{"kind": "noise", "radius": 1}]}]}
        host.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": catalog}] + [{"op": "upsert", "entity": e} for e in [p, q]])
        camera = {"position": [14, 12, 16], "target": [0, 1, 0], "fov_degrees": 50}
        host.call("set_camera", camera=camera)
        host.call("start_multiplayer_host", address="127.0.0.1:0", capacity=2)
        server = wait(host, "inspect_multiplayer_host", lambda s: s["transport"]["state"] == "listening")["transport"]
        client.call("connect_multiplayer", address=server["address"], certificate_der=server["certificate_der"])
        def state():
            return client.call("inspect_multiplayer_client", actors=True)
        s = wait(client, "inspect_multiplayer_client", lambda s: s.get("owner") and s.get("content") and s.get("actor_count") == 2, actors=True)
        own_id = s["actor"]["actor"]
        own_name = s["actors"][str(own_id)]["appearance"]["label"]
        target = next(a for a in s["actors"].values() if a["appearance"]["kind"] == "player" and a["id"] != own_id)
        other_name = target["appearance"]["label"]
        target_ref = {"actor": target["id"], "life": target["life"]}
        def hold(x=0, z=0, mode="walk"):
            return client.call("set_multiplayer_input", input={"x": x, "z": z, "mode": mode})["control"]
        def pos():
            return state()["owner"]["position"]
        def release(token):
            return client.call("release_multiplayer_input", control=token)["released"]
        start = pos()
        old = hold(z=1)
        time.sleep(.65)
        after = pos()
        assert after[2] > start[2] + 1.4, (start, after)
        newer = hold(x=-1)
        assert newer != old and not release(old)
        start = pos()
        time.sleep(.4)
        after = pos()
        assert after[0] < start[0] - .7, (start, after)
        assert release(newer)
        time.sleep(.3)
        start = pos()
        time.sleep(.3)
        assert math.dist(start, pos()) < .03
        # First GPU initialization holds Service; the I/O heartbeat still renews input.
        token = hold(z=.5)
        start = pos()
        client.call("capture_multiplayer", path="artifacts/controls-held.png", width=640, height=480)
        time.sleep(.4)
        assert pos()[2] > start[2] + .6
        assert release(token)
        time.sleep(.2)
        assert client.call("inspect")["tick"] == 0
        checks.append("Persistent hold exceeds server lease, I/O heartbeat survives GPU capture, old focus token preserves newer MCP input, current release neutralizes without local simulation")

        s = state()
        target_now = s["actors"][str(target["id"])]
        x, y = project(camera, [v/1000 for v in target_now["millimeters"]], 800, 600)
        assert client.call("pick_multiplayer", x=x, y=y, width=800, height=600)["target"] == target_ref
        assert client.call("capture_multiplayer", path="artifacts/controls-selected.png", width=640, height=480)["selection_guide_count"] == 1
        client.call("select_multiplayer_target", target={**target_ref, "life": target_ref["life"]+1}, expected_error=True)
        assert client.call("inspect_multiplayer_controls")["selected"] == target_ref
        def resolved(id_, outcome):
            return wait(client, "inspect_multiplayer_client", lambda s: any(r["id"] == id_ and r["status"].get("outcome") == outcome for r in s.get("results", [])), actors=True)
        hit = client.call("use_multiplayer_ability", slot=1)
        s = resolved(hit["operation"], "completed")
        wait(client, "inspect_multiplayer_client", lambda s: s.get("owner") and s["owner"]["resources"][0][1] == 99)
        assert state()["actors"][str(target["id"])]["health"]["units"] == 95
        duplicate = {"id": hit["operation"], "life": s["actor"]["life"], "intent": {"kind": "activate", "slot": 4, "target": target_ref}}
        client.call("send_multiplayer_operation", operation=duplicate)
        time.sleep(.15)
        assert state()["actors"][str(target["id"])]["health"]["units"] == 95
        long = client.call("use_multiplayer_ability", slot=2)
        wait(client, "inspect_multiplayer_client", lambda s: s.get("owner") and s["owner"].get("cast_execution"))
        cancel = client.call("cancel_multiplayer_ability")
        resolved(long["operation"], "cancelled")
        wait(client, "inspect_multiplayer_client", lambda s: s.get("owner") and s["owner"].get("cast_execution") is None)
        assert state()["owner"]["resources"][0][1] == 99
        s = state()
        point = [v/1000 for v in s["actors"][str(target["id"])]["millimeters"]]
        x, y = project(camera, point, 800, 600)
        shot = client.call("use_multiplayer_ability_at", slot=3, x=x, y=y, width=800, height=600, plane_y=point[1])
        resolved(shot["operation"], "launched")
        wait(client, "inspect_multiplayer_client", lambda s: s.get("projectile_count", 0) > 0 and s.get("owner") and s["owner"]["resources"][0][1] == 98)
        resolved(client.call("use_multiplayer_ability", slot=6)["operation"], "completed")
        checks.append("Public ray/selection and stale-life refusal, own grant metadata, auto-ID actor/self/spatial aim and cancel share dispatcher; duplicate payload cannot kill or charge twice")

        stale = hold(x=1)
        life = state()["actor"]["life"]
        host.call("activate_ability", actor=other_name, slot=4, target=own_name)
        wait(client, "inspect_multiplayer_client", lambda s: s.get("owner") and not s["owner"]["alive"])
        client.call("set_multiplayer_input", input={"x": 1, "z": 0}, expected_error=True)
        host.call("activate_ability", actor=other_name, slot=5, target=own_name)
        wait(client, "inspect_multiplayer_client", lambda s: s.get("owner") and s["owner"]["alive"] and s["actor"]["life"] == life+1)
        assert not release(stale)
        time.sleep(.25)
        start = pos()
        time.sleep(.3)
        assert math.dist(start, pos()) < .03
        fresh = hold(x=1)
        assert not release(stale)
        start = pos()
        time.sleep(.3)
        assert pos()[0] > start[0] + .65
        generation = state()["connection"]
        client.call("reconnect_multiplayer")
        wait(client, "inspect_multiplayer_client", lambda s: s.get("owner") and s.get("content") and s.get("connection", 0) > generation and s.get("actor_count") == 2)
        assert not release(fresh)
        time.sleep(.25)
        start = pos()
        time.sleep(.3)
        assert math.dist(start, pos()) < .03
        final = hold(x=-1)
        assert final != fresh
        assert release(final)
        checks.append("Death/revive and resumed connection fence stale holds/tokens; no automatic input in the new life/transport, fresh control works, old release cannot erase it")
        assert client.call("inspect")["tick"] == 0
        client.call("disconnect_multiplayer")
        client.call("set_input", input={"x": .5, "z": 0})
        client.call("step", ticks=6)
        assert client.call("inspect")["tick"] == 6
        result = {"passed": True, "checks": checks, "build_id": host.call("inspect")["build_id"]}
        (ROOT/"artifacts/remote-controls-verification.json").write_text(json.dumps(result, indent=2)+"\n")
        return result
    finally:
        client.close()
        host.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT/"target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    result = verify(args.binary.resolve(), args.window)
    print(json.dumps({**result, "checks": len(result["checks"]), "native_window": False}))
