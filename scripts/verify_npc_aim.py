#!/usr/bin/env python3
"""NPC spatial aiming from confirmed observations, exercised only through MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import vital, definition, loadout
from verify_ability_policy import objective, policy, wait
from verify_projectiles import delivery


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-npc-aim-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert "spatial_aim" in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        bolt=definition("observed_bolt","spatial","damage",20,60,300);bolt["range"]=0;bolt["delivery"]=delivery(6,0.14,18);bolt["movement"]={"mode":"lock"}
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[bolt]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player");player["transform"]["position"]=[-4,0.8,0];player["vitality"]=vital();player["footsteps"]=None
        shooter=actor("shooter",[4,0.8,0],[0.9,0.18,0.16],"player");shooter["vitality"]=vital();shooter["abilities"]=loadout(["observed_bolt"])
        shooter["npc"]["awareness"]["vision_range"]=20;shooter["npc"]["awareness"]["fov_degrees"]=360
        aim_policy=policy(margin=None);aim_policy["spatial_aim"]={"mode":"observed_point","range":12}
        shooter["npc"]["objectives"]=[objective(aim_policy),wait()]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,shooter]])
        c.call("set_camera",camera={"position":[10,12,15],"target":[0,0.8,0],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("step",ticks=1);initial=c.call("inspect_abilities")["actors"]["shooter"]["cast"]
        assert initial["aim"]["kind"]=="point" and all(abs(a-b)<1e-6 for a,b in zip(initial["aim"]["position"],[-4,0.8,0])) and initial["target"] is None, initial
        assert initial["aim_observation"]=={"actor":"player","tick":0,"source":"vision","life_index":0,"instance_id":next(i["instance_id"] for i in c.call("inspect_instances")["instances"] if i["authored_id"]=="player")}
        c.call("set_input",input={"x":0,"z":1});c.call("step",ticks=20);c.call("set_input",input={"x":0,"z":0})
        prefix=c.call("inspect");assert prefix["abilities"]["actors"]["shooter"]["cast"]["aim"]==initial["aim"]
        c.call("capture",path="artifacts/npc-aim-cast.png",width=1280,height=900)
        c.call("capture",path="artifacts/npc-aim-cast-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/npc-aim-cast.png").read_bytes()==(ROOT/"artifacts/npc-aim-cast-repeat.png").read_bytes()
        c.call("set_debug_view",npc_routes=False);assert c.call("inspect")["abilities"]==prefix["abilities"]
        c.call("capture",path="artifacts/npc-aim-hidden-guide.png",width=1280,height=900)
        assert (ROOT/"artifacts/npc-aim-hidden-guide.png").read_bytes()!=(ROOT/"artifacts/npc-aim-cast.png").read_bytes()
        c.call("set_debug_view",npc_routes=True)
        c.call("step",ticks=40);shot=c.call("inspect_abilities")["projectiles"]["active"][0]
        assert shot["aim"]==initial["aim"] and shot["aim_observation"]==initial["aim_observation"] and abs(shot["direction"][2])<1e-5
        c.call("step",ticks=190);assert c.call("inspect_vitality")["actors"]["player"]["units"]==100
        assert c.call("inspect_abilities")["actors"]["shooter"]["resources"][0]["units"]==995
        c.call("save_replay",path="artifacts/npc-aim.replay.json");assert c.call("run_replay",path="artifacts/npc-aim.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/npc-aim.replay.json",tick=21)["snapshot"]["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/npc-aim.replay.json",path="artifacts/npc-aim-prefix.png",tick=21,width=1280,height=900)
        assert (ROOT/"artifacts/npc-aim-prefix.png").read_bytes()==(ROOT/"artifacts/npc-aim-cast.png").read_bytes()
        checks.append("Observed point and provenance frozen at cast start; player dodges during preparation, cost once, guide presentation-only and repeated/prefix pixels identical")
        c.call("reset");c.call("step",ticks=145);assert c.call("inspect_vitality")["actors"]["player"]["units"]==80
        c.call("reset");cover={"id":"cover","name":"Cover","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[0,1.5,0.75],"size":[0.1,3,0.5]}}
        c.call("upsert",entity=cover);c.call("step",ticks=1);assert c.call("inspect_abilities")["actors"]["shooter"]["cast"]
        c.call("set_input",input={"x":0,"z":1})
        for _ in range(50):
            c.call("step",ticks=1)
            if c.call("inspect_abilities")["actors"]["shooter"]["cast"] is None:break
        a=c.call("inspect_abilities");assert a["actors"]["shooter"]["cast"] is None and not a["projectiles"]["active"] and a["actors"]["shooter"]["resources"][0]["units"]==1000
        assert any(r["outcome"]=="cancelled" for r in a["receipts"])
        c.call("save_replay",path="artifacts/npc-aim-hidden.replay.json");assert c.call("run_replay",path="artifacts/npc-aim-hidden.replay.json")["matched"]
        checks.append("Still target is hit, moving behind physical occluder during cast cancels without debit or flight, replay verifies cancellation")
        before=c.call("inspect");bad=copy.deepcopy(shooter);bad["npc"]["objectives"][0]["action"]["policy"]["spatial_aim"]["range"]=0
        c.call("upsert",entity=bad,expected_error=True);assert c.call("inspect")==before
        c.call("remove",id="cover");approach=copy.deepcopy(shooter);p=approach["npc"]["objectives"][0]["action"]["policy"];p["spatial_aim"]["range"]=3;p["approach"]={"margin":0.4}
        c.call("upsert",entity=approach);c.call("step",ticks=1);assert c.call("inspect_npcs")["npcs"]["shooter"]["objectives"]["ability_decisions"][0]["phase"]=="approaching"
        c.call("step",ticks=260);assert c.call("inspect_vitality")["actors"]["player"]["units"]==80
        c.call("undo");assert not c.call("inspect_abilities")["projectiles"]["active"]
        c.call("redo");c.call("reset")
        checks.append("Invalid policy is atomic; opt-in approach uses independent decision range, undo/redo/reset rebuild state")
        # Comparison scene: red freezes at start; blue tracks the actor until launch.
        bolt["cast_ticks"]=90;bolt["cooldown_ticks"]=180;bolt["effects"][0]["amount"]=10
        tracked=copy.deepcopy(bolt);tracked.update(id="tracked_bolt",selection="actor",range=18)
        heal=definition("self_heal","self_actor","heal",25,30,90);heal["range"]=0;heal["movement"]={"mode":"lock"}
        free=copy.deepcopy(bolt);free.update(id="player_bolt",cast_ticks=30,cooldown_ticks=45);free["effects"][0]["amount"]=20
        catalog["abilities"]=[bolt,tracked,heal,free]
        player["transform"]["position"]=[0,0.8,-4];player["abilities"]=loadout(["player_bolt","self_heal","tracked_bolt"])
        shooter["transform"]["position"]=[-4,0.8,5];shooter["npc"]["objectives"][0]["action"]["policy"]["spatial_aim"]["range"]=18
        blue=copy.deepcopy(shooter);blue.update(id="tracker",name="Tracking shooter",color=[0.15,0.5,1]);blue["transform"]["position"]=[4,0.8,5];blue["abilities"]=loadout(["tracked_bolt"]);blue["npc"]["objectives"]=[objective(policy(margin=None)),wait()]
        cover["transform"]={"position":[0,1.5,0],"size":[2,3,0.4]}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,shooter,blue,cover]])
        c.call("set_camera",camera={"position":[12,13,16],"target":[0,0.8,0],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("save_scene",path="examples/npc-aim.scene.json")
        c.call("step",ticks=1);c.call("set_input",input={"x":1,"z":0});c.call("step",ticks=15);c.call("set_input",input={"x":0,"z":0})
        c.call("capture",path="artifacts/npc-aim-demo.png",width=1280,height=900)
        c.call("save_replay",path="artifacts/npc-aim-demo.replay.json");assert c.call("run_replay",path="artifacts/npc-aim-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-aim.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION
        checks.append("Comparison demo authored through MCP: red observed-point, blue actor-targeted, player free shot/self heal/targeted shot, shared costs and collision")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/npc-aim-verification.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
