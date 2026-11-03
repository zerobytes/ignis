#!/usr/bin/env python3
"""Local write sets and indexed buffs through the executable; macOS offline gate."""
import argparse
import copy
import json
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from verify_lab import Client, ROOT


def offline_binary(binary):
    sandbox = shutil.which("sandbox-exec") if sys.platform == "darwin" else None
    network_denied = False
    if sandbox:
        profile = "(version 1)(allow default)(deny network*)"
        for action in ['s.bind(("127.0.0.1",0))', 's.connect(("127.0.0.1",9))']:
            probe = subprocess.run([sandbox, "-p", profile, sys.executable, "-c",
                                    "import socket; s=socket.socket(); " + action],
                                   capture_output=True, text=True, timeout=15)
            assert probe.returncode != 0 and "Operation not permitted" in probe.stderr, probe.stderr
        launcher = ROOT / "artifacts/offline-ignis.sh"
        launcher.write_text("#!/bin/sh\nexec " + shlex.quote(sandbox) + " -p "
                            + shlex.quote(profile) + " " + shlex.quote(str(binary)) + ' "$@"\n')
        launcher.chmod(0o755)
        binary = launcher
        network_denied = True
    return binary, network_denied


def verify(binary, window):
    checks = []
    binary, network_denied = offline_binary(binary)
    c = Client(binary, window)
    try:
        c.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                 "clientInfo": {"name": "ignis-local-transactions", "version": "1"}})
        c.request("notifications/initialized", notification=True)
        c.call("generate", recipe={"width": 32, "depth": 28, "obstacles": 0, "seed": 123})
        player = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"] == "player")
        player["transform"]["position"] = [-4, .8, 0]
        player["footsteps"] = None
        player["vitality"] = {"initial": 100, "maximum": 100, "scale": 1}
        def definition(id_, effects, costs=None):
            return {"id": id_, "selection": "actor", "range": 30, "cast_ticks": 0,
                    "cooldown_ticks": 0, "costs": costs or [], "effects": effects}
        costs = [{"resource": "energy", "units": 7}]
        definitions = [
            definition("mixed", [
                {"kind": "resource", "recipient": "source", "resource": "energy", "delta": 20},
                {"kind": "resource", "recipient": "target", "resource": "energy", "delta": -11},
                {"kind": "damage", "recipient": "source", "amount": 40},
                {"kind": "heal", "recipient": "target", "amount": 10}], costs),
            definition("boost", [{"kind": "apply_buff", "recipient": "target", "buff": "boost"}]),
            definition("hit", [{"kind": "damage", "recipient": "target", "amount": 10}]),
            definition("failure", [
                {"kind": "resource", "recipient": "source", "resource": "energy", "delta": -13},
                {"kind": "damage", "recipient": "target", "amount": 100},
                {"kind": "apply_buff", "recipient": "source", "buff": "boost"},
                {"kind": "resource", "recipient": "target", "resource": "energy", "delta": 1}], costs),
            definition("clean", [{"kind": "dispel", "recipient": "target", "category": "boost",
                                  "max_instances": 8, "require_match": True}])]
        catalog = {"resources": [{"id": "energy", "scale": 1}], "abilities": definitions,
                   "attributes": {"buffs": [{"id": "boost", "duration_ticks": 60, "reapply": "refresh",
                       "category": "boost", "modifiers": [{"attribute": "combat.damage_dealt.multiplier",
                           "operation": {"kind": "multiply", "basis_points": 20000}}]}]}}
        player["abilities"] = {"resources": [{"resource": "energy", "initial": 100, "capacity": 100}],
                               "grants": [d["id"] for d in definitions]}
        c.call("edit_scene", edits=[{"op": "set_ability_catalog", "catalog": catalog},
                                    {"op": "upsert", "entity": player}])
        def cast(slot, target="player"):
            c.call("activate_ability", actor="player", slot=slot, target=target)
            c.call("step", ticks=1)
        def health(id_="player"):
            return c.call("inspect_vitality")["actors"][id_]["units"]
        def energy():
            return c.call("inspect_resources")["actors"]["player"]["resources"][0]["units"]
        def attr(id_="player"):
            return c.call("inspect_attribute", entity=id_, attribute="combat.damage_dealt.multiplier")
        c.call("set_input", input={"x": 1, "z": 0})
        c.call("step", ticks=15)
        assert c.call("inspect")["runtime_positions"]["player"][0] > -4
        c.call("set_input", input={"x": 0, "z": 0})
        cast(1)
        assert health() == 70 and energy() == 89
        cast(2)
        cast(3)
        assert health() == 50 and attr()["effective"]["units"] == 20000
        c.call("save_scene", path="artifacts/local-transactions.scene.json")
        c.call("save_replay", path="artifacts/local-transactions-offline.replay.json")
        assert c.call("run_replay", path="artifacts/local-transactions-offline.replay.json")["matched"]
        checks.append("Executable single player: legacy movement, cost+self effects, buffs/damage, disk save and replay; network denied on macOS")

        partner = copy.deepcopy(player)
        partner.update(id="partner", name="partner")
        partner["transform"]["position"] = [1, .8, 0]
        c.call("upsert", entity=partner)
        cast(2, "partner")
        cast(2)
        cast(2, "partner")
        assert attr()["effective"]["units"] == attr("partner")["effective"]["units"] == 20000
        assert len(c.call("inspect_abilities")["buffs"]["active"]) == 2
        cast(5)
        assert attr()["effective"]["units"] == 10000
        assert attr("partner")["effective"]["units"] == 20000
        snapshot = c.call("inspect")
        c.call("save_replay", path="artifacts/local-transactions-index.replay.json")
        raw = json.loads((ROOT / "artifacts/local-transactions-index.replay.json").read_text())
        assert isinstance(raw["expected_abilities"]["buffs"]["active"], list)
        assert "by_owner_buff" not in json.dumps(raw)
        assert c.call("run_replay", path="artifacts/local-transactions-index.replay.json")["matched"]
        replayed = c.call("inspect_replay", path="artifacts/local-transactions-index.replay.json")
        assert replayed["snapshot"]["abilities"] == snapshot["abilities"]
        checks.append("Two local players: independent owner contributions, refresh, compaction after dispel, unchanged JSON list and reconstructed replay index")

        partner["abilities"] = None
        c.call("upsert", entity=partner)
        cast(4, "partner")
        assert c.call("inspect_abilities")["receipts"][-1]["outcome"] == "missing_resource"
        assert energy() == 100 and health("partner") == 100
        assert not c.call("inspect_abilities")["buffs"]["active"]
        assert not c.call("inspect_vitality")["deaths"]
        c.call("undo")
        cast(2, "partner")
        assert attr("partner")["effective"]["units"] == 20000
        c.call("reset")
        assert attr("partner")["effective"]["units"] == 10000
        c.call("load_scene", path="artifacts/local-transactions.scene.json")
        assert c.call("inspect_controls")["legacy_input_available"]
        cast(1)
        assert health() == 70 and energy() == 89
        checks.append("Late missing resource rolls back cost/health/buff; undo/reset/load rebuild storage and restore offline single player")
        result = {"passed": True, "checks": checks, "native_window": window,
                  "network_denied": network_denied, "build_id": c.call("inspect")["build_id"],
                  "slow_requests": c.slow_requests}
        (ROOT / "artifacts/local-transactions-verification.json").write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "network_denied": network_denied,
                          "build_id": result["build_id"]}))
    finally:
        c.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
