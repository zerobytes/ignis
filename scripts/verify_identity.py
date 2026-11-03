#!/usr/bin/env python3
"""Instance references through MCP; no dynamic population or foreground window."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import definition, loadout, vital


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{}, "clientInfo":{"name":"ignis-identity-verifier","version":"1"}})
        c.request("notifications/initialized", notification=True)
        assert "inspect_instances" in json.dumps(c.request("tools/list"))
        c.call("generate", recipe={"width":24,"depth":20,"obstacles":0,"seed":42})
        catalog = {"resources":[{"id":"energy","scale":1}], "abilities":[definition("strike","actor","damage",100),definition("return","actor","resurrect",40,12)]}
        p = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"] == "player")
        p.update(vitality=vital(), abilities=loadout(["strike","return"]), blocks_vision=False)
        p.pop("footsteps", None);p["transform"]["position"]=[0,.8,-4]
        v = actor("patient", [0,.8,0], [.7,.2,.8], "player")
        v.update(vitality=vital(), abilities=loadout(["strike"]), blocks_vision=False)
        v["npc"]["enabled"]=False
        c.call("edit_scene", edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [p,v]])
        c.call("save_scene",path="artifacts/identity.scene.json")
        def ids():
            return {r["authored_id"]:r for r in c.call("inspect_instances")["instances"]}
        initial=ids();pr=initial["player"]["reference"];vr=initial["patient"]["reference"]
        assert c.call("inspect_instances")["despawn_supported"]
        assert c.call("inspect_instances")["spawn_supported"]
        before=c.call("inspect")
        fixed=next(r["reference"] for r in initial.values() if not r["actor"])
        for bad in ["ignis:bad", pr[:-1]+"Z", fixed]:
            c.call("activate_ability",actor=bad,slot=1,target=vr,expected_error=True)
            c.call("activate_ability",actor=pr,slot=1,target=bad,expected_error=True)
            assert c.call("inspect")==before and ids()==initial
        c.call("activate_ability",actor=pr,slot=1,target=vr)
        pending=c.call("inspect")
        c.call("save_replay",path="artifacts/identity.replay.json")
        assert c.call("run_replay",path="artifacts/identity.replay.json")["instances_matched"]
        c.call("step",ticks=1)
        assert c.call("inspect_vitality")["actors"]["patient"]["units"]==0
        c.call("activate_ability",actor=pr,slot=2,target=vr);c.call("step",ticks=13)
        current=ids();assert current["patient"]["reference"]==vr and current["patient"]["life_index"]==1
        assert current["patient"]["instance_id"]==initial["patient"]["instance_id"]
        assert c.call("inspect_vitality")["actors"]["patient"]["units"]==40
        c.call("save_replay",path="artifacts/identity.replay.json")
        assert c.call("run_replay",path="artifacts/identity.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/identity.replay.json",tick=0)["snapshot"]["abilities"]==pending["abilities"]
        c.call("capture",path="artifacts/identity-live.png",width=800,height=600)
        c.call("capture_replay",replay_path="artifacts/identity.replay.json",path="artifacts/identity-replay.png",width=800,height=600)
        assert (ROOT/"artifacts/identity-live.png").read_bytes()==(ROOT/"artifacts/identity-replay.png").read_bytes()
        checks.append("Scoped tokens, atomic malformed/static rejection, same individual through resurrection, pending/full/prefix replay and identical offscreen pixels")
        before=c.call("inspect");bad=copy.deepcopy(p);bad["transform"]["size"][0]=-1
        c.call("upsert",entity=bad,expected_error=True);assert ids()==current and c.call("inspect")==before
        old=pr
        for operation,kwargs in [("reset",{}),("upsert",{"entity":p}),("undo",{}),("redo",{}),("load_scene",{"path":"artifacts/identity.scene.json"})]:
            c.call(operation,**kwargs)
            fresh=ids()["player"]["reference"];assert fresh!=old
            before=c.call("inspect")
            c.call("activate_ability",actor=old,slot=1,target="patient",expected_error=True)
            c.call("cancel_ability",actor=old,execution_id=1,expected_error=True)
            assert c.call("inspect")==before
            old=fresh
        c.call("activate_ability",actor="player",slot=1,target="patient");c.call("step",ticks=1)
        assert c.call("inspect_vitality")["actors"]["patient"]["units"]==0
        checks.append("Invalid edits preserve scope; reset, edit, undo, redo and load invalidate prior tokens; authored aliases remain usable")
        c.call("load_scene",path="examples/free-aim.scene.json")
        pr=ids()["player"]["reference"]
        c.call("aim_ability",actor=pr,slot=1,aim={"kind":"direction","vector":[0,1,0]});c.call("step",ticks=1)
        cast=c.call("inspect_abilities")["actors"]["player"]["cast"]
        c.call("cancel_ability",actor=pr,execution_id=cast["execution_id"]);c.call("step",ticks=1)
        assert c.call("inspect_abilities")["actors"]["player"]["cast"] is None
        c.call("aim_ability_at",actor=pr,slot=1,x=400,y=300,width=800,height=600,plane_y=.8)
        c.call("save_replay",path="artifacts/identity-aim.replay.json")
        assert c.call("run_replay",path="artifacts/identity-aim.replay.json")["matched"]
        checks.append("Direction aim, cancellation and camera-plane aim share instance-token resolution through MCP")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/identity-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:
        c.close()

if __name__ == "__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
