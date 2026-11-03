#!/usr/bin/env python3
"""Persistent identity/provenance through MCP; reuse remains an isolated Rust test."""
from verify_lab import SCENE_VERSION, REPLAY_VERSION
import argparse
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-identity-bindings-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        def ids(): return {i["authored_id"]:i["instance_id"] for i in c.call("inspect_instances")["instances"]}
        def quiet():
            edits=[]
            for e in c.call("inspect")["scene"]["entities"]:
                if e.get("npc"):
                    e["npc"]["enabled"]=False;edits.append({"op":"upsert","entity":e})
            if edits:c.call("edit_scene",edits=edits)
        def replay(name, prefix):
            path=f"artifacts/bindings-{name}.replay.json"
            c.call("save_replay",path=path);assert c.call("run_replay",path=path)["matched"]
            data=json.loads((ROOT/path).read_text());assert data["version"]==REPLAY_VERSION and data["scene"]["version"]==SCENE_VERSION
            snap=c.call("inspect_replay",path=path,tick=prefix["tick"])["snapshot"]
            for k in ["npcs","abilities","vitality","triggers"]:assert snap[k]==prefix[k],k
        c.call("load_scene",path="examples/npc-attributes.scene.json");quiet();identity=ids()
        c.call("activate_ability",actor="player",slot=1);c.call("step",ticks=1)
        buffs=c.call("inspect_abilities")["buffs"];b=buffs["active"][0]
        assert b["owner_instance"]==b["source_instance"]==identity["player"]
        assert buffs["receipts"][0]["instance"]["owner_instance"]==identity["player"]
        value=c.call("inspect_attribute",entity="player",attribute="ability.interruptible")
        assert value["effective"]=={"kind":"bool","value":False}
        assert value["contributions"][0]["source_instance"]==identity["player"]
        prefix=c.call("inspect");c.call("step",ticks=610);assert not c.call("inspect_abilities")["buffs"]["active"]
        replay("buff",prefix)
        checks.append("Buff owner/source and contribution provenance, expiration and full/prefix replay50 without scene schema change")
        c.call("load_scene",path="examples/npc-radio.scene.json");identity=ids();c.call("step",ticks=13)
        prefix=c.call("inspect");received=prefix["npcs"]["receiver"]["communication"]["last_received"]
        assert received["sender_instance"]==identity["sender"] and received["target_instance"]==identity["player"]
        assert received["source"]=="vision"
        c.call("capture",path="artifacts/bindings-radio-live.png",width=800,height=600)
        c.call("step",ticks=30);replay("radio",prefix)
        c.call("capture_replay",replay_path="artifacts/bindings-radio.replay.json",path="artifacts/bindings-radio-prefix.png",tick=13,width=800,height=600)
        assert (ROOT/"artifacts/bindings-radio-live.png").read_bytes()==(ROOT/"artifacts/bindings-radio-prefix.png").read_bytes()
        checks.append("Confirmed sight and frozen radio keep subject/sender identities; replay prefix regenerates identical GPU pixels")
        c.call("load_scene",path="examples/npc-periodic.scene.json");quiet();identity=ids()
        c.call("activate_ability",actor="player",slot=1,target="dummy");c.call("step",ticks=31)
        prefix=c.call("inspect");r=prefix["abilities"]["periodic"]["receipts"][0]
        assert r["actor_instance"]==identity["player"] and r["target_instance"]==identity["dummy"]
        c.call("step",ticks=61);a=c.call("inspect_abilities")
        pulse=next(r for r in a["receipts"] if r["outcome"]=="periodic_pulse")
        assert pulse["actor_instance"]==identity["player"] and pulse["effects"][0]["actor_instance"]==identity["dummy"]
        replay("periodic",prefix)
        checks.append("Periodic installation and damage receipts preserve original source and recipient through pulses and replay")
        c.call("load_scene",path="examples/npc-resurrection.scene.json");quiet();identity=ids()
        c.call("activate_ability",actor="player",slot=1,target="patient");c.call("step",ticks=1)
        death=c.call("inspect_vitality")["deaths"][0]
        assert death["actor_instance"]==identity["patient"] and death["source_instance"]==identity["player"]
        c.call("activate_ability",actor="player",slot=2,target="patient");prefix=c.call("inspect");c.call("step",ticks=121)
        revival=c.call("inspect_vitality")["revivals"][0]
        assert revival["actor_instance"]==identity["patient"] and revival["life_index"]==1
        replay("vital",prefix)
        checks.append("Death/revival histories preserve the same individual with a distinct life, shared MCP executor and replay")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/identity-bindings-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
