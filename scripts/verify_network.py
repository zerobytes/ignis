#!/usr/bin/env python3
"""Critical cross-process native/MCP QUIC integration; no window or remote service."""
import argparse
import copy
import json
import time
from pathlib import Path
from verify_lab import Client, ROOT


def wait(client, op, predicate, **args):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        state = client.call(op, **args)
        if predicate(state):
            return state
        time.sleep(.02)
    raise AssertionError(state)


def verify(binary, window=False):
    assert not window, "Network gate runs headless"
    host, client = Client(binary), Client(binary)
    checks = []
    try:
        for c in (host, client):
            c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                     "clientInfo": {"name": "ignis-network", "version": "1"}})
            c.request("notifications/initialized", notification=True)
        schema = json.dumps(client.request("tools/list"))
        assert all(op in schema for op in ["start_multiplayer_host", "connect_multiplayer", "send_multiplayer_operation", "reconnect_multiplayer"])
        assert host.call("inspect_multiplayer_host")["state"] == "offline"
        host.call("generate", recipe={"width": 20, "depth": 20, "obstacles": 0, "seed": 142})
        p = next(e for e in host.call("inspect")["scene"]["entities"] if e.get("body") == "player")
        p.update(vitality={"initial": 100, "maximum": 100, "scale": 1},
                 abilities={"resources": [{"resource": "energy", "initial": 100, "capacity": 100}], "grants": ["hit", "bolt"]})
        p["transform"]["position"] = [-3, .8, 0]
        q = copy.deepcopy(p)
        q.update(id="partner", name="partner")
        q["transform"]["position"] = [3, .8, 0]
        catalog = {"resources": [{"id": "energy", "scale": 1}], "abilities": [
            {"id": "hit", "selection": "actor", "range": 20, "cast_ticks": 0, "cooldown_ticks": 0,
             "costs": [{"resource": "energy", "units": 1}],
             "effects": [{"kind": "damage", "recipient": "target", "amount": 1}]}]}
        catalog["abilities"].append({**copy.deepcopy(catalog["abilities"][0]), "id": "bolt", "delivery": {"kind": "projectile", "speed": 1, "half_extent": .1, "max_distance": 20}})
        props = [{"id": "ramp", "name": "Ramp", "body": "fixed", "geometry": {"kind": "wedge", "rise": "positive_x"}, "transform": {"position": [0, 1, -5], "size": [3, 2, 3]}, "color": [.2, .3, .7]},
                 {"id": "crate", "name": "Crate", "body": "dynamic", "transform": {"position": [0, 3, 4], "size": [1, 1, 1]}, "color": [.8, .5, .2]}]
        host.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": catalog}] + [{"op": "upsert", "entity": e} for e in [p, q, *props]])
        host.call("set_camera", camera={"position": [12, 10, 14], "target": [0, 1, 0], "fov_degrees": 50})
        host.call("edit_mesh", edit={"kind": "put_asset", "asset": {"id": "decoration", "geometry": {"positions": [[-1, 0, 0], [1, 0, 0], [0, 2, 0]], "triangles": [[0, 1, 2]]}}})
        host.call("edit_mesh", edit={"kind": "put_instance", "instance": {"id": "decoration_instance", "asset": "decoration", "position": [0, 1.5, 3], "rotation": [0, 0, 0, 1], "scale": 1, "color": [.2, .8, .3], "playback": None}})
        host.call("edit_mesh", edit={"kind": "grid", "id": "dense_tile", "segments": 31, "size": 3})
        host.call("edit_mesh", edit={"kind": "put_instance", "instance": {"id": "dense_instance", "asset": "dense_tile", "position": [-6, .05, -5], "rotation": [0, 0, 0, 1], "scale": 1, "color": [.4, .6, .9], "playback": None}})
        host.call("start_multiplayer_host", address="127.0.0.1:0", capacity=2)
        server = wait(host, "inspect_multiplayer_host", lambda s: s["transport"]["state"] == "listening")
        public = server["transport"]
        assert "secret" not in json.dumps(server)
        client.call("connect_multiplayer", address=public["address"], certificate_der=public["certificate_der"])
        state = wait(client, "inspect_multiplayer_client", lambda s: s.get("actor_count") == 2 and s.get("owner") and s.get("content"), actors=True)
        target = next(a for a in state["actors"].values() if a["appearance"]["kind"] in ("player", "npc") and a["id"] != state["actor"]["actor"])
        operation = {"id": 1, "life": state["actor"]["life"], "intent": {"kind": "activate", "slot": 1,
                     "target": {"actor": target["id"], "life": target["life"]}}}
        client.call("send_multiplayer_operation", operation=operation)
        done = lambda s: s.get("owner") and s["owner"]["resources"][0][1] == 99 and any(r["status"].get("outcome") == "completed" for r in s["results"])
        state = wait(client, "inspect_multiplayer_client", done, actors=True)
        assert state["actors"][str(target["id"])]["health"]["units"] == 99
        assert "secret" not in json.dumps(state)
        checks.append("Two real processes: shared schema/Service clock, pinned TLS, public roster, own resources and committed combat")
        invalid = copy.deepcopy(operation)
        invalid["id"] = 9999
        client.call("send_multiplayer_operation", operation=invalid)
        state = wait(client, "inspect_multiplayer_client", lambda s: any(r["id"] == 9999 and r["status"].get("reason") == "sequence" for r in s.get("results", [])), actors=True)
        assert state["last_operation"] == 1
        old_generation = state["connection"]
        client.call("reconnect_multiplayer")
        state = wait(client, "inspect_multiplayer_client", lambda s: s.get("connection", 0) > old_generation and s.get("owner"), actors=True)
        client.call("send_multiplayer_operation", operation=operation)
        state = wait(client, "inspect_multiplayer_client", done, actors=True)
        assert state["actors"][str(target["id"])]["health"]["units"] == 99
        client.call("send_multiplayer_movement", movement={"sequence": 1, "life": state["actor"]["life"], "input": {"x": 1, "z": 0}})
        wait(client, "inspect_multiplayer_client", lambda s: s.get("owner") and s["owner"]["input_ack"] == 1)
        checks.append("Native handle retains private resume ticket, fences old transport and deduplicates retry without second charge")
        assert client.call("inspect")["tick"] == 0
        client.call("play", expected_error=True)
        client.call("step", ticks=1, expected_error=True)
        bolt = copy.deepcopy(operation)
        bolt["id"] = 2
        bolt["intent"]["slot"] = 2
        client.call("send_multiplayer_operation", operation=bolt)
        state = wait(client, "inspect_multiplayer_client", lambda s: s.get("projectile_count", 0) > 0 and s.get("owner") and s["owner"]["resources"][0][1] == 98, actors=True)
        host.call("pause")
        tick = host.call("inspect")["tick"]
        align = (1-tick) % 6
        if align:
            host.call("step", ticks=align)
        tick = host.call("inspect")["tick"]
        state = wait(client, "inspect_multiplayer_client", lambda s: s.get("replica_tick") == tick, actors=True)
        assert any(a["appearance"]["kind"] == "dynamic" for a in state["actors"].values())
        assert any(a["appearance"]["geometry"]["kind"] == "wedge" for a in state["actors"].values())
        local_before = client.call("inspect")
        remote = client.call("capture_multiplayer", path="artifacts/network-remote.png", width=640, height=480)
        reference = host.call("capture", path="artifacts/network-host.png", width=640, height=480)
        assert remote["mesh_vertex_count"] == reference["mesh_vertex_count"] == 3 + 31*31*6
        assert remote["entity_count"] == reference["entity_count"] == state["entity_count"] + state["projectile_count"]
        metrics = host.call("compare_images", reference_path="artifacts/network-host.png", candidate_path="artifacts/network-remote.png", difference_path="artifacts/network-difference.png")["report"]["metrics"]
        assert metrics["ssim_luma"] > .999 and metrics["rgb_mae"] < .001, metrics
        assert client.call("inspect") == local_before
        checks.append("Remote public meshes/materials, wedge, dynamic prop, actors and projectile use the native renderer; GPU matches host at quantized pose and client physics remains paused")
        host.call("reset")
        wait(host, "inspect_multiplayer_host", lambda s: s.get("closed"))
        wait(client, "inspect_multiplayer_client", lambda s: s["state"] == "disconnected")
        client.call("capture_multiplayer", path="artifacts/network-frozen.png", width=640, height=480)
        assert host.call("compare_images", reference_path="artifacts/network-remote.png", candidate_path="artifacts/network-frozen.png")["report"]["metrics"]["exact_rgb_match"]
        client.call("disconnect_multiplayer")
        client.call("capture_multiplayer", path="artifacts/network-no-client.png", width=640, height=480, expected_error=True)
        client.call("set_input", input={"x": .5, "z": 0})
        client.call("step", ticks=6)
        assert client.call("inspect")["tick"] == 6
        host.call("stop_multiplayer_host")
        host.call("set_execution_mode", mode="recorded")
        host.call("set_input", input={"x": 0, "z": 0}, expected_error=True)  # multiple Players require explicit controls
        control = host.call("bind_control", actor=p["id"])["control"]
        host.call("set_control_input", control=control, input={"x": .5, "z": 0})
        host.call("step", ticks=6)
        assert host.call("inspect_multiplayer_host")["state"] == "offline"
        assert host.call("inspect")["recorded_ticks"] == 6
        checks.append("World rebuild closes epoch/connections; same process returns to recorded offline simulation with no endpoint")
        result = {"passed": True, "checks": checks, "build_id": host.call("inspect")["build_id"], "remote_metrics": metrics}
        (ROOT / "artifacts/network-verification.json").write_text(json.dumps(result, indent=2) + "\n")
        return result
    finally:
        client.close()
        host.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    result = verify(args.binary.resolve(), args.window)
    print(json.dumps({**result, "checks": len(result["checks"]), "native_window": False}))
