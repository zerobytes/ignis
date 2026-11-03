#!/usr/bin/env python3
"""Resurrection authored/executed through MCP, isolated replay and GPU evidence."""
from verify_lab import SCENE_VERSION
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
        c.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{}, "clientInfo":{"name":"ignis-resurrection-verifier","version":"1"}})
        c.request("notifications/initialized", notification=True)
        assert 'resurrect' in json.dumps(c.request("tools/list"))
        c.call("generate", recipe={"width":24,"depth":20,"obstacles":0,"seed":42})
        kill = definition("strike", "actor", "damage", 100)
        revive = definition("return", "actor", "resurrect", 40, 12, 30)
        heal = definition("heal", "actor", "heal", 25)
        spend = definition("signal", "self_actor", "heal", 1, 0, 600)
        spend.update(range=0, effects=[{"kind":"noise","radius":1}], costs=[{"resource":"energy","units":17}])
        catalog = {"resources":[{"id":"energy","scale":1}], "abilities":[kill,revive,heal,spend]}
        p = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"] == "player")
        p.update(vitality=vital(), abilities=loadout(["strike","return","heal","signal"]), blocks_vision=False)
        p.pop("footsteps", None);p["transform"]["position"]=[0,.8,-4]
        v = actor("patient", [0,.8,0], [.7,.2,.8], "player")
        v.update(vitality=vital(), abilities=loadout(["signal"]), blocks_vision=False)
        wait = {"priority":1,"when":"always","action":{"kind":"wait"}}
        v["npc"]["objectives"]=[wait]
        c.call("edit_scene", edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [p,v]])
        c.call("set_debug_view", npc_routes=True, npc_vision=False, npc_hearing=False, noise_events=False)
        c.call("activate_ability", actor="patient", slot=1);c.call("step", ticks=1)
        initial = c.call("inspect_abilities")["actors"]["patient"]
        c.call("activate_ability", actor="player", slot=1, target="patient");c.call("step", ticks=1)
        c.call("capture", path="artifacts/resurrection-dead.png", width=1280, height=900)
        c.call("activate_ability", actor="player", slot=2, target="patient");c.call("step", ticks=1)
        prefix=c.call("inspect");prefix_tick=prefix["tick"]
        c.call("capture", path="artifacts/resurrection-prefix-live.png", width=1280, height=900)
        c.call("step", ticks=12)
        st=c.call("inspect");h=st["vitality"]["actors"]["patient"]
        assert (h["units"],h["life_index"],h["life"]) == (40,1,"alive")
        current=st["abilities"]["actors"]["patient"]
        assert current["resources"]==initial["resources"]
        assert current["grants"][0]["ready_at"]==initial["grants"][0]["ready_at"]
        assert len(st["vitality"]["deaths"])==len(st["vitality"]["revivals"])==1
        c.call("capture",path="artifacts/resurrection-alive.png",width=1280,height=900)
        assert (ROOT/"artifacts/resurrection-dead.png").read_bytes() != (ROOT/"artifacts/resurrection-alive.png").read_bytes()
        c.call("save_replay",path="artifacts/resurrection.replay.json")
        assert c.call("run_replay",path="artifacts/resurrection.replay.json")["matched"]
        replay_prefix=c.call("inspect_replay",path="artifacts/resurrection.replay.json",tick=prefix_tick)["snapshot"]
        assert replay_prefix["vitality"]==prefix["vitality"] and replay_prefix["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/resurrection.replay.json",path="artifacts/resurrection-prefix.png",tick=prefix_tick,width=1280,height=900)
        assert (ROOT/"artifacts/resurrection-prefix-live.png").read_bytes()==(ROOT/"artifacts/resurrection-prefix.png").read_bytes()
        checks.append("MCP schema/authoring, same individual returns with preserved energy/cooldown, events and full/prefix replay with identical GPU pixels")
        c.call("step",ticks=40);balance=c.call("inspect_abilities")["actors"]["player"]["resources"]
        c.call("activate_ability",actor="player",slot=2,target="patient");c.call("step",ticks=1)
        assert c.call("inspect_abilities")["receipts"][-1]["outcome"]=="target_alive"
        assert c.call("inspect_abilities")["actors"]["player"]["resources"]==balance
        before=c.call("inspect");bad=copy.deepcopy(catalog);bad["abilities"][1]["effects"][0]["amount"]=0
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        c.call("reset");assert c.call("inspect_vitality")["actors"]["patient"]["life_index"]==0
        changed=copy.deepcopy(v);changed["vitality"]["initial"]=0;c.call("upsert",entity=changed)
        c.call("undo");assert c.call("inspect_vitality")["actors"]["patient"]["units"]==100
        c.call("redo");assert c.call("inspect_vitality")["actors"]["patient"]["units"]==0
        c.call("activate_ability",actor="player",slot=3,target="patient");c.call("step",ticks=1)
        assert c.call("inspect_abilities")["receipts"][-1]["outcome"]=="target_dead"
        checks.append("Living target rejected without payment; invalid schema atomic, reset/undo/redo and ordinary healing still cannot revive")
        # Human demo: explicit visible patient, inaccessible corpse and independent observer.
        catalog["abilities"][1].update(cast_ticks=120,cooldown_ticks=180,range=8)
        v["npc"]["objectives"]=[{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":60}}]
        healer=actor("healer",[-3,.8,0],[.2,.85,.4],"patient")
        healer.update(vitality=vital(),abilities=loadout(["return"]),blocks_vision=False)
        healer["npc"]["awareness"].update(fov_degrees=360,vision_range=10)
        healer["npc"]["objectives"]=[{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":30}}]
        observer=actor("observer",[3,.8,0],[.2,.5,1],"patient");observer.update(vitality=vital(),blocks_vision=False)
        observer["npc"]["awareness"].update(fov_degrees=360,vision_range=10);observer["npc"]["objectives"]=[wait]
        hidden=actor("hidden",[0,.8,6],[.6,.6,.6]);hidden.update(vitality=vital(0),blocks_vision=False);hidden["npc"]["objectives"]=[wait]
        wall={"id":"screen","name":"Occlusion wall","body":"fixed","color":[.25,.6,.55],"transform":{"position":[0,1.5,4],"size":[7,3,.4]}}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [p,v,healer,observer,hidden,wall]])
        c.call("select_entity",id="patient")
        c.call("save_scene",path="examples/npc-resurrection.scene.json")
        c.call("capture",path="artifacts/resurrection-demo.png",width=1280,height=900)
        c.call("step",ticks=1)
        c.call("activate_ability",actor="player",slot=1,target="patient");c.call("step",ticks=32)
        cast=c.call("inspect_abilities")["actors"]["healer"]["cast"];assert cast and cast["target"]=="patient"
        c.call("step",ticks=120);st=c.call("inspect")
        assert st["vitality"]["actors"]["patient"]["units"]==40
        assert st["vitality"]["actors"]["hidden"]["units"]==0
        assert st["abilities"]["actors"]["patient"]["resources"][0]["units"]==983
        assert st["abilities"]["actors"]["patient"]["grants"][0]["ready_at"]==600
        c.call("step",ticks=1);assert "patient" not in c.call("inspect_npcs")["npcs"]["observer"]["known_dead"]
        c.call("save_replay",path="artifacts/resurrection-demo.replay.json");assert c.call("run_replay",path="artifacts/resurrection-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-resurrection.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION
        checks.append("Saved MCP demo: NPC resurrects observed explicit patient, preserves its spent resource/cooldown, observer rediscovers and hidden corpse remains dead")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/resurrection-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:
        c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
