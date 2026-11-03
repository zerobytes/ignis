#!/usr/bin/env python3
"""Author, exercise and inspect projectile delivery only through MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import vital, definition, loadout
from verify_ability_policy import objective, policy, wait


def delivery(speed=8, half_extent=0.12, max_distance=20):
    return {"kind":"projectile","speed":speed,"half_extent":half_extent,"max_distance":max_distance}


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-projectile-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert "max_distance" in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        bolt=definition("bolt","actor","damage",20,30,60);bolt["range"]=18;bolt["delivery"]=delivery()
        heal=definition("heal","self_actor","heal",20,30,120);heal["range"]=0;heal["movement"]={"mode":"lock"}
        healing=definition("healing_bolt","actor","heal",20,30,60);healing["range"]=18;healing["delivery"]=delivery()
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[bolt,heal,healing]}
        p=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        p["transform"]["position"]=[-4,0.8,0];p["footsteps"]=None;p["vitality"]=vital();p["abilities"]=loadout(["bolt","heal","healing_bolt"])
        target=actor("target",[4,0.8,0],[0.75,0.2,0.85],"player");target["vitality"]=vital();target["abilities"]=loadout(["bolt"])
        target["npc"]["awareness"]["fov_degrees"]=360;target["npc"]["awareness"]["vision_range"]=25;target["npc"]["objectives"]=[wait()]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [p,target]])
        c.call("set_camera",camera={"position":[10,10,12],"target":[0,0.8,0],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("activate_ability",actor="player",slot=1,target="target");c.call("step",ticks=31)
        a=c.call("inspect_abilities");assert len(a["projectiles"]["active"])==1 and a["actors"]["player"]["resources"][0]["units"]==995
        assert a["receipts"][-1]["outcome"]=="launched" and a["receipts"][-1]["costs"]==[{"resource":"energy","units":5}]
        assert c.call("inspect_vitality")["actors"]["target"]["units"]==100
        c.call("step",ticks=15);prefix=c.call("inspect")
        c.call("capture",path="artifacts/projectile-flight.png",width=1280,height=900)
        c.call("capture",path="artifacts/projectile-flight-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/projectile-flight.png").read_bytes()==(ROOT/"artifacts/projectile-flight-repeat.png").read_bytes()
        c.call("step",ticks=70);a=c.call("inspect_abilities");assert not a["projectiles"]["active"] and a["projectiles"]["receipts"][-1]["outcome"]=="hit"
        assert c.call("inspect_vitality")["actors"]["target"]["units"]==80
        assert a["receipts"][-1]["outcome"]=="impacted" and not a["receipts"][-1]["costs"]
        c.call("save_replay",path="artifacts/projectile.replay.json");assert c.call("run_replay",path="artifacts/projectile.replay.json")["matched"]
        restored=c.call("inspect_replay",path="artifacts/projectile.replay.json",tick=46)["snapshot"]
        assert restored["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/projectile.replay.json",path="artifacts/projectile-prefix.png",tick=46,width=1280,height=900)
        assert (ROOT/"artifacts/projectile-flight.png").read_bytes()==(ROOT/"artifacts/projectile-prefix.png").read_bytes()
        checks.append("Launch pays once, delayed impact, inspect grants/costs/flight, deterministic prefix/final replay and identical offscreen pixels")
        wall={"id":"cover","name":"Cover","body":"fixed","blocks_vision":False,"color":[0.2,0.6,0.5],"transform":{"position":[0,1.5,0],"size":[0.05,3,2]}}
        c.call("upsert",entity=wall);c.call("activate_ability",actor="player",slot=1,target="target");c.call("step",ticks=120)
        a=c.call("inspect_abilities");assert a["projectiles"]["receipts"][-1]["outcome"]=="blocked" and a["projectiles"]["receipts"][-1]["entity"]=="cover"
        assert c.call("inspect_vitality")["actors"]["target"]["units"]==100 and a["actors"]["player"]["resources"][0]["units"]==995
        c.call("remove",id="cover");c.call("activate_ability",actor="target",slot=1,target="player");c.call("step",ticks=31)
        heading=c.call("inspect_abilities")["projectiles"]["active"][0]["direction"]
        c.call("set_input",input={"x":0,"z":1});c.call("step",ticks=20)
        assert c.call("inspect_abilities")["projectiles"]["active"][0]["direction"]==heading
        c.call("set_input",input={"x":0,"z":0});c.call("step",ticks=160)
        assert c.call("inspect_vitality")["actors"]["player"]["units"]==100
        c.call("save_replay",path="artifacts/projectile-dodge.replay.json");assert c.call("run_replay",path="artifacts/projectile-dodge.replay.json")["matched"]
        checks.append("Physical cover blocks despite visual transparency; NPC direction frozen after launch, player dodges, miss is paid and replayable")
        before=c.call("inspect");bad=copy.deepcopy(catalog);bad["abilities"][0]["delivery"]["speed"]=0
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        direct=copy.deepcopy(catalog);direct["abilities"][0]["delivery"]={"kind":"direct"};c.call("set_ability_catalog",catalog=direct)
        c.call("undo");assert c.call("inspect_abilities")["actors"]["player"]["grants"][0]["delivery"]["kind"]=="projectile"
        c.call("redo");assert c.call("inspect_abilities")["actors"]["player"]["grants"][0]["delivery"]["kind"]=="direct"
        c.call("set_ability_catalog",catalog=catalog)
        c.call("activate_ability",actor="player",slot=1,target="target");c.call("save_replay",path="artifacts/projectile-pending.replay.json")
        assert len(c.call("inspect_replay",path="artifacts/projectile-pending.replay.json")["snapshot"]["abilities"]["pending_inputs"])==1
        c.call("reset");assert not c.call("inspect_abilities")["projectiles"]["active"]
        checks.append("Invalid delivery atomic, undo/redo/direct legacy, pending input replay and reset clear runtime")
        # Final human scene, authored and saved with the same commands.
        bolt["cast_ticks"]=45;bolt["cooldown_ticks"]=90;bolt["delivery"]=delivery(6,0.14,18)
        healing["cast_ticks"]=45;healing["delivery"]=delivery(6,0.14,18)
        enemy=definition("enemy_bolt","actor","damage",10,60,120);enemy["range"]=18;enemy["delivery"]=delivery(5,0.14,18);enemy["movement"]={"mode":"lock"}
        catalog["abilities"].append(enemy)
        p["transform"]["position"]=[0,0.8,-4]
        target["transform"]["position"]=[-5,0.8,-4];target["vitality"]=vital(60);target["abilities"]=None
        red=actor("shooter",[0,0.8,6],[0.9,0.18,0.16],"player");red["vitality"]=vital();red["abilities"]=loadout(["enemy_bolt"])
        red["npc"]["awareness"]["fov_degrees"]=360;red["npc"]["awareness"]["vision_range"]=18;red["npc"]["objectives"]=[objective(policy(margin=None)),wait()]
        wall["blocks_vision"]=True;wall["transform"]={"position":[2,1.5,1],"size":[3,3,0.4]}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [p,target,red,wall]])
        c.call("set_camera",camera={"position":[12,12,16],"target":[0,0.8,0],"fov_degrees":48});c.call("select_entity",id="target")
        c.call("save_scene",path="examples/npc-projectiles.scene.json")
        c.call("activate_ability",actor="player",slot=1,target="target");c.call("step",ticks=80)
        assert len(c.call("inspect_abilities")["projectiles"]["active"])==2
        c.call("capture",path="artifacts/projectile-demo.png",width=1280,height=900)
        c.call("step",ticks=120);assert c.call("inspect_vitality")["actors"]["player"]["units"]==90
        assert c.call("inspect_vitality")["actors"]["target"]["units"]==40
        c.call("save_replay",path="artifacts/projectile-demo.replay.json");assert c.call("run_replay",path="artifacts/projectile-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-projectiles.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION
        checks.append("Playable NPC shooter/cover/damage/healing demo created and loaded through MCP; both player and NPC launch visible projectiles")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/projectiles-verification.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
