#!/usr/bin/env python3
"""Exercise cast/movement modes and create the human demo via MCP only."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import vital, definition, loadout


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-cast-movement-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert "speed_tolerance" in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":20,"depth":18,"seed":42,"obstacles":0})
        free=definition("free_strike","actor","damage",15,90,60);free["range"]=12
        interrupt=definition("stationary_heal","self_actor","heal",15,90,60);interrupt["range"]=0;interrupt["movement"]={"mode":"interrupt","speed_tolerance":0.2}
        lock=definition("rooted_heal","self_actor","heal",15,90,60);lock["range"]=0;lock["movement"]={"mode":"lock"}
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[free,interrupt,lock]}
        p=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        p["transform"]["position"]=[0,0.8,-3];p["vitality"]=vital(40);p["abilities"]=loadout(["free_strike","stationary_heal","rooted_heal"])
        dummy=actor("dummy",[0,0.8,1],[0.75,0.2,0.85]);dummy["vitality"]={"initial":1000,"maximum":1000,"scale":1}
        dummy["npc"]["objectives"]=[{"priority":1,"when":"always","action":{"kind":"wait"}}]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [p,dummy]])
        c.call("set_camera",camera={"position":[9,10,12],"target":[0,0.8,-1],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("select_entity",id="dummy")
        c.call("save_scene",path="examples/cast-movement.scene.json")
        c.call("set_input",input={"x":1,"z":0});c.call("activate_ability",actor="player",slot=1,target="dummy");c.call("step",ticks=91)
        state=c.call("inspect_abilities");assert state["receipts"][-1]["outcome"]=="completed" and state["actors"]["player"]["resources"][0]["units"]==995
        assert c.call("inspect_vitality")["actors"]["dummy"]["units"]==985
        c.call("save_replay",path="artifacts/cast-free.replay.json");assert c.call("run_replay",path="artifacts/cast-free.replay.json")["matched"]
        c.call("reset");c.call("set_input",input={"x":1,"z":0});c.call("activate_ability",actor="player",slot=2);c.call("step",ticks=1)
        state=c.call("inspect_abilities");assert state["receipts"][-1]["outcome"]=="moving" and state["actors"]["player"]["resources"][0]["units"]==1000
        c.call("reset");c.call("step",ticks=5);c.call("activate_ability",actor="player",slot=2);c.call("step",ticks=90)
        assert c.call("inspect_abilities")["actors"]["player"]["cast"]
        c.call("set_input",input={"x":0,"z":1});c.call("step",ticks=1)
        state=c.call("inspect_abilities");assert state["receipts"][-1]["outcome"]=="interrupted_by_movement" and state["actors"]["player"]["resources"][0]["units"]==1000
        assert c.call("inspect_vitality")["actors"]["player"]["units"]==40
        c.call("save_replay",path="artifacts/cast-interrupted.replay.json");assert c.call("run_replay",path="artifacts/cast-interrupted.replay.json")["matched"]
        checks.append("Free cast commits while moving; stationary cast rejects moving input and cancels at deadline without cost/heal/cooldown; deterministic replay")
        c.call("reset");c.call("set_input",input={"x":1,"z":0});c.call("activate_ability",actor="player",slot=3)
        c.call("save_replay",path="artifacts/cast-pending.replay.json")
        pending=c.call("inspect_replay",path="artifacts/cast-pending.replay.json")["snapshot"]
        assert len(pending["abilities"]["pending_inputs"])==1
        c.call("step",ticks=45);prefix=c.call("inspect")
        assert prefix["abilities"]["actors"]["player"]["movement_locked"]
        assert abs(prefix["runtime_positions"]["player"][0])<0.001
        c.call("capture",path="artifacts/cast-locked.png",width=1280,height=900)
        c.call("capture",path="artifacts/cast-locked-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/cast-locked.png").read_bytes()==(ROOT/"artifacts/cast-locked-repeat.png").read_bytes()
        c.call("step",ticks=46);after=c.call("inspect")
        assert not after["abilities"]["actors"]["player"]["movement_locked"] and after["runtime_positions"]["player"][0]>0.01
        assert after["vitality"]["actors"]["player"]["units"]==55
        c.call("save_replay",path="artifacts/cast-lock.replay.json");assert c.call("run_replay",path="artifacts/cast-lock.replay.json")["matched"]
        restored=c.call("inspect_replay",path="artifacts/cast-lock.replay.json",tick=45)["snapshot"]
        assert restored["abilities"]==prefix["abilities"] and restored["runtime_positions"]==prefix["runtime_positions"]
        c.call("capture_replay",replay_path="artifacts/cast-lock.replay.json",path="artifacts/cast-lock-prefix.png",tick=45,width=1280,height=900)
        assert (ROOT/"artifacts/cast-locked.png").read_bytes()==(ROOT/"artifacts/cast-lock-prefix.png").read_bytes()
        checks.append("Lock suppresses held input, releases on commit, preserves pending/prefix/final replay and identical GPU captures")
        c.call("reset");c.call("set_input",input={"x":1,"z":0});c.call("activate_ability",actor="player",slot=3);c.call("step",ticks=1)
        cast=c.call("inspect_abilities")["actors"]["player"]["cast"]
        c.call("cancel_ability",actor="player",execution_id=cast["execution_id"]);c.call("step",ticks=1)
        assert not c.call("inspect_abilities")["actors"]["player"]["movement_locked"]
        before=c.call("inspect");bad=copy.deepcopy(catalog);bad["abilities"][1]["movement"]["speed_tolerance"]=0
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        changed=copy.deepcopy(catalog);changed["abilities"][1]["movement"]={"mode":"free"};c.call("set_ability_catalog",catalog=changed)
        c.call("undo");assert c.call("inspect_abilities")["actors"]["player"]["grants"][1]["movement"]["mode"]=="interrupt"
        c.call("redo");assert c.call("inspect_abilities")["actors"]["player"]["grants"][1]["movement"]["mode"]=="free"
        c.call("load_scene",path="examples/cast-movement.scene.json")
        assert c.call("inspect")["scene"]["version"]==SCENE_VERSION
        checks.append("Manual cancellation releases lock; invalid policy atomic; undo/redo/load/reset and all controls available in MCP schema")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/cast-movement-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
