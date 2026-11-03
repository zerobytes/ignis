#!/usr/bin/env python3
"""Free XYZ intent and cursor mapping through the shared MCP dispatcher."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c=Client(binary,window); checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-free-aim-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"));assert "aim_ability_at" in schema and "plane_y" in schema
        c.call("load_scene",path="examples/npc-projectiles.scene.json")
        scene=c.call("inspect")["scene"];catalog=scene["ability_catalog"]
        free=copy.deepcopy(catalog["abilities"][0]);free.update(id="free_bolt",selection="spatial",range=0,cast_ticks=30,cooldown_ticks=30)
        catalog["abilities"].append(free)
        p=next(e for e in scene["entities"] if e["id"]=="player");p["abilities"]["grants"]=["free_bolt","heal","bolt"]
        shooter=next(e for e in scene["entities"] if e["id"]=="shooter");shooter["npc"]["enabled"]=False
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog},{"op":"upsert","entity":p},{"op":"upsert","entity":shooter}])
        c.call("select_entity",id=None)
        before=c.call("inspect")
        for aim in [{"kind":"direction","vector":[0,0,0]},{"kind":"point","position":[10001,0,0]}]:
            c.call("aim_ability",actor="player",slot=1,aim=aim,expected_error=True);assert c.call("inspect")==before
        c.call("activate_ability",actor="player",slot=1,target="target",expected_error=True);assert c.call("inspect")==before
        c.call("aim_ability_at",actor="player",slot=1,x=800,y=300,width=800,height=600,plane_y=0.8,expected_error=True);assert c.call("inspect")==before
        c.call("aim_ability",actor="player",slot=1,aim={"kind":"direction","vector":[-2,0,0]})
        c.call("save_replay",path="artifacts/free-aim-pending.replay.json");assert c.call("run_replay",path="artifacts/free-aim-pending.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/free-aim-pending.replay.json")["snapshot"]["abilities"]["pending_inputs"][0]["operation"]["aim"]["vector"]==[-2,0,0]
        c.call("step",ticks=50);prefix=c.call("inspect");a=prefix["abilities"];shot=a["projectiles"]["active"][0]
        assert shot["intended_target"] is None and shot["direction"]==[-1,0,0]
        assert a["actors"]["player"]["resources"][0]["units"]==995
        assert c.call("inspect_vitality")["actors"]["target"]["units"]==60
        c.call("capture",path="artifacts/free-aim-flight.png",width=1280,height=900)
        c.call("capture",path="artifacts/free-aim-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/free-aim-flight.png").read_bytes()==(ROOT/"artifacts/free-aim-repeat.png").read_bytes()
        c.call("step",ticks=100);assert c.call("inspect_vitality")["actors"]["target"]["units"]==40
        c.call("save_replay",path="artifacts/free-aim.replay.json");assert c.call("run_replay",path="artifacts/free-aim.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/free-aim.replay.json",tick=50)["snapshot"]["abilities"]==a
        c.call("capture_replay",replay_path="artifacts/free-aim.replay.json",path="artifacts/free-aim-prefix.png",tick=50,width=1280,height=900)
        assert (ROOT/"artifacts/free-aim-flight.png").read_bytes()==(ROOT/"artifacts/free-aim-prefix.png").read_bytes()
        checks.append("Typed free direction without selection, atomic errors, launch/impact costs, pending/prefix/final replay and identical offscreen pixels")
        c.call("reset");c.call("aim_ability",actor="player",slot=1,aim={"kind":"point","position":[-1,0.8,-4]});c.call("step",ticks=120)
        assert c.call("inspect_vitality")["actors"]["target"]["units"]==40
        c.call("reset");c.call("aim_ability",actor="player",slot=1,aim={"kind":"direction","vector":[0,1,0]});c.call("step",ticks=300)
        a=c.call("inspect_abilities");assert a["projectiles"]["receipts"][-1]["outcome"]=="expired" and a["actors"]["player"]["resources"][0]["units"]==995
        checks.append("Point guides direction without stopping the projectile; vertical miss spends once and expires at travel limit")
        c.call("reset");wall={"id":"free_cover","name":"Free cover","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[-2.5,1.5,-4],"size":[0.1,3,2]}}
        c.call("upsert",entity=wall);c.call("aim_ability",actor="player",slot=1,aim={"kind":"direction","vector":[-1,0,0]});c.call("step",ticks=120)
        a=c.call("inspect_abilities");assert a["projectiles"]["receipts"][-1]["entity"]=="free_cover" and a["projectiles"]["receipts"][-1]["outcome"]=="blocked"
        assert c.call("inspect_vitality")["actors"]["target"]["units"]==60
        c.call("undo");assert not c.call("inspect_abilities")["projectiles"]["active"]
        c.call("redo");assert any(e["id"]=="free_cover" for e in c.call("inspect")["scene"]["entities"])
        c.call("remove",id="free_cover")
        checks.append("Free fire does not require visibility; first physical cover blocks, with undo/redo clearing runtime")
        response=c.call("aim_ability_at",actor="player",slot=1,x=400,y=300,width=800,height=600,plane_y=0.8)
        point=response["aim_point"];assert abs(point[0])<1e-4 and abs(point[1]-0.8)<1e-4 and abs(point[2])<1e-4
        c.call("save_replay",path="artifacts/free-aim-cursor.replay.json");assert c.call("run_replay",path="artifacts/free-aim-cursor.replay.json")["matched"]
        c.call("step",ticks=31);assert c.call("inspect_abilities")["projectiles"]["active"][0]["aim"]["position"]==point
        checks.append("Cursor command resolves current camera/viewport to a frozen world point, same input used by native hotkeys")
        # Preserve the targeted NPC while giving the player free and targeted options.
        shooter["npc"]["enabled"]=True
        c.call("upsert",entity=shooter);c.call("select_entity",id=None)
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("save_scene",path="examples/free-aim.scene.json")
        c.call("aim_ability",actor="player",slot=1,aim={"kind":"point","position":[-5,0.8,-4]});c.call("step",ticks=80)
        assert c.call("inspect_vitality")["actors"]["target"]["units"]==40
        c.call("capture",path="artifacts/free-aim-demo.png",width=1280,height=900)
        c.call("save_replay",path="artifacts/free-aim-demo.replay.json");assert c.call("run_replay",path="artifacts/free-aim-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/free-aim.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION
        checks.append("Playable free-fire/targeted-fire/self-heal demo saved via MCP; NPC retains perceived-target projectile behavior")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/free-aim-verification.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
