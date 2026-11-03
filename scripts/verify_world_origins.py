#!/usr/bin/env python3
"""World/authoring boundary and source leases through the shared MCP dispatcher."""
from verify_lab import REPLAY_VERSION
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "ignis-world-origins-verifier", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        c.call("load_scene", path="examples/npc-outgoing.scene.json")
        authored = c.call("inspect")["scene"]
        world = c.call("inspect_world")
        assert world["dynamic_population"] is True
        assert world["authored_entities"] == len(authored["entities"])
        assert set(world["positions"]) == {e["id"] for e in authored["entities"]}
        c.call("step", ticks=1)
        assert c.call("inspect")["scene"] == authored
        assert c.call("inspect_world")["positions"] == c.call("inspect")["runtime_positions"]
        checks.append("Present runtime placements exposed separately from unchanged authoring, including nonphysical entities")

        def cast(slot, target=None):
            args = {"actor": "player", "slot": slot}
            if target is not None:
                args["target"] = target
            c.call("activate_ability", **args)
            c.call("step", ticks=1)

        origins = c.call("inspect_origins")
        assert origins["per_actor_capacity"] == 32 and origins["reserved"] == []
        assert origins["allocated"] == 0 and origins["capacity"] >= 32
        cast(2, "half_ice")
        cast(3, "half_ice")
        leases = c.call("inspect_origins")["reserved"]
        assert len(leases) == 1 and leases[0]["detached"] is False
        identity = next(i for i in c.call("inspect_instances")["instances"] if i["authored_id"] == "player")["instance_id"]
        assert leases[0]["source_instance"] == identity
        cast(4)
        prefix = c.call("inspect")
        tick = prefix["tick"]
        c.call("capture", path="artifacts/world-live.png", width=800, height=600)
        c.call("step", ticks=190)
        state = c.call("inspect_abilities")
        assert c.call("inspect_origins")["reserved"] == []
        assert state["receipts"] and state["projectiles"]["receipts"] and state["periodic"]["receipts"]
        impact = next(r for r in state["receipts"] if r["outcome"] == "impacted")
        assert impact["effects"][0]["damage_components"][0]["dealt_multiplier"] == 12500
        checks.append("Projectile and periodic share a reserved source; live buffs remain dynamic; history does not retain contexts")

        path = "artifacts/world-origins.replay.json"
        c.call("save_replay", path=path)
        assert json.loads((ROOT/path).read_text())["version"] == REPLAY_VERSION
        assert c.call("run_replay", path=path)["matched"]
        replay = c.call("inspect_replay", path=path, tick=tick)["snapshot"]
        for key in ["runtime_positions", "abilities", "vitality", "npcs"]:
            assert prefix[key] == replay[key], key
        c.call("capture_replay", replay_path=path, path="artifacts/world-prefix.png", tick=tick, width=800, height=600)
        assert (ROOT/"artifacts/world-live.png").read_bytes() == (ROOT/"artifacts/world-prefix.png").read_bytes()
        checks.append("Replay50 reproduces world and leases at full/prefix execution and identical offscreen pixels")

        before = c.call("inspect")
        c.call("activate_ability", actor="missing", slot=1, expected_error=True)
        assert c.call("inspect") == before
        c.call("reset")
        assert c.call("inspect_origins")["reserved"] == []
        assert c.call("inspect")["scene"] == authored
        assert c.call("inspect_world")["positions"] == world["positions"]
        checks.append("Invalid source is atomic; reset reconstructs authored placements and clears leases")
        result = {"passed": True, "checks": checks, "native_window": window, "build_id": c.call("inspect")["build_id"], "slow_requests": c.slow_requests}
        (ROOT/"artifacts/world-origins-verification.json").write_text(json.dumps(result, indent=2)+"\n")
        print(json.dumps({"passed": True, "checks": len(checks), "build_id": result["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--binary", type=Path, default=ROOT/"target/release/ignis")
    p.add_argument("--window", action="store_true")
    args = p.parse_args()
    verify(args.binary.resolve(), args.window)
