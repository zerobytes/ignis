#!/usr/bin/env python3
"""Vital state, transactional effects, replay and health/corpse rendering through MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor


def vital(initial=100):
    return {"initial":initial,"maximum":100,"scale":1}


def definition(name, selection, kind, amount, cast=0, cooldown=0):
    return {"id":name,"selection":selection,"range":12,"cast_ticks":cast,
            "cooldown_ticks":cooldown,"costs":[{"resource":"energy","units":5}],
            "effects":[{"kind":kind,"recipient":"target","amount":amount}]}


def loadout(grants):
    return {"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":grants}


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-vitality-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert "inspect_vitality" in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":22,"depth":20,"seed":42,"obstacles":0})
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[
            definition("strike","actor","damage",35),definition("heal_self","self_actor","heal",35),definition("heal_ally","actor","heal",35)]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[0,0.8,-4];player["vitality"]=vital();player["abilities"]=loadout(["strike","heal_self","heal_ally"])
        dummy=actor("dummy",[0,0.8,0],[0.75,0.2,0.85]);dummy["vitality"]=vital()
        dummy["npc"]["objectives"]=[{"priority":1,"when":"always","action":{"kind":"wait"}}]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,dummy]])
        c.call("set_debug_view",npc_routes=True,npc_vision=True)
        c.call("activate_ability",actor="player",slot=1,target="dummy")
        c.call("save_replay",path="artifacts/vitality-pending.replay.json")
        pending=c.call("inspect_replay",path="artifacts/vitality-pending.replay.json")["snapshot"]
        assert pending["vitality"]["actors"]["dummy"]["units"]==100 and len(pending["abilities"]["pending_inputs"])==1
        c.call("step",ticks=1)
        prefix=c.call("inspect");assert prefix["vitality"]["actors"]["dummy"]["units"]==65
        c.call("capture",path="artifacts/vitality-alive.png",width=1280,height=900)
        c.call("capture",path="artifacts/vitality-alive-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/vitality-alive.png").read_bytes()==(ROOT/"artifacts/vitality-alive-repeat.png").read_bytes()
        for _ in range(2):c.call("activate_ability",actor="player",slot=1,target="dummy");c.call("step",ticks=1)
        state=c.call("inspect_vitality");assert state["actors"]["dummy"]["life"]=="dead" and len(state["deaths"])==1
        assert state["deaths"][0]["source"]=="player"
        receipt=c.call("inspect_abilities")["receipts"][-1]["effects"][0]
        assert receipt["applied"]==30 and receipt["vitality"]=={"kind":"damage","before":30,"after":0,"excess":5}
        assert c.call("inspect_npcs")["npcs"]["dummy"]["phase"]=="dead"
        c.call("capture",path="artifacts/vitality-dead.png",width=1280,height=900)
        assert (ROOT/"artifacts/vitality-alive.png").read_bytes()!=(ROOT/"artifacts/vitality-dead.png").read_bytes()
        c.call("save_replay",path="artifacts/vitality.replay.json")
        assert c.call("run_replay",path="artifacts/vitality.replay.json")["matched"]
        restored=c.call("inspect_replay",path="artifacts/vitality.replay.json",tick=1)["snapshot"]
        assert restored["vitality"]==prefix["vitality"]
        c.call("capture_replay",replay_path="artifacts/vitality.replay.json",path="artifacts/vitality-prefix.png",tick=1,width=1280,height=900)
        assert (ROOT/"artifacts/vitality-alive.png").read_bytes()==(ROOT/"artifacts/vitality-prefix.png").read_bytes()
        checks.append("MCP-only authoring, damage/excess/death receipts, pending/prefix/final replay and GPU health/corpse pixels")
        before=c.call("inspect_abilities")["actors"]["player"]["resources"]
        c.call("activate_ability",actor="player",slot=3,target="dummy");c.call("step",ticks=1)
        result=c.call("inspect_abilities");assert result["receipts"][-1]["outcome"]=="target_dead" and result["actors"]["player"]["resources"]==before
        c.call("activate_ability",actor="player",slot=2);c.call("step",ticks=1)
        result=c.call("inspect_abilities");assert result["receipts"][-1]["effects"][0]["applied"]==0 and result["actors"]["player"]["resources"][0]["units"]==980
        snapshot=c.call("inspect");bad=copy.deepcopy(dummy);bad["vitality"]["maximum"]=0
        c.call("upsert",entity=bad,expected_error=True);assert c.call("inspect")==snapshot
        c.call("reset");assert c.call("inspect_vitality")["actors"]["dummy"]["units"]==100
        changed=copy.deepcopy(dummy);changed["vitality"]["initial"]=20;c.call("upsert",entity=changed)
        assert c.call("inspect_vitality")["actors"]["dummy"]["units"]==20
        c.call("undo");assert c.call("inspect_vitality")["actors"]["dummy"]["units"]==100
        c.call("redo");assert c.call("inspect_vitality")["actors"]["dummy"]["units"]==20
        c.call("undo")
        checks.append("Dead target rejects without debit; full heal succeeds with cost; invalid editing atomic and reset/undo/redo reconstruct vital state")
        # Human demo: standing attacker, healer and passive target. No special behavior in the renderer.
        catalog["abilities"][0].update(cast_ticks=30,cooldown_ticks=45)
        catalog["abilities"][1].update(cast_ticks=45,cooldown_ticks=90)
        catalog["abilities"][2].update(cast_ticks=30,cooldown_ticks=60)
        hit=definition("enemy_strike","actor","damage",20,45,120);hit["range"]=6
        heal=definition("support","actor","heal",12,60,180);heal["range"]=8
        catalog["abilities"] += [hit,heal]
        enemy=actor("enemy",[4,0.8,0],[0.9,0.15,0.1],"player");enemy["vitality"]=vital();enemy["abilities"]=loadout(["enemy_strike"])
        healer=actor("healer",[-4,0.8,0],[0.15,0.5,1],"player");healer["vitality"]=vital();healer["abilities"]=loadout(["support"])
        for n in [enemy,healer]:
            n["npc"]["awareness"]["fov_degrees"]=360
            n["npc"]["objectives"]=[{"priority":2,"when":"visible_target","action":{"kind":"use_ability","slot":1,"retry_ticks":60}}, {"priority":1,"when":"always","action":{"kind":"wait"}}]
        dummy["transform"]["position"]=[0,0.8,3]
        player["transform"]["position"]=[0,0.8,-4]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,dummy,enemy,healer]])
        c.call("set_camera",camera={"position":[13,14,18],"target":[0,0.8,-1],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("select_entity",id="enemy")
        c.call("save_scene",path="examples/npc-vitality.scene.json")
        c.call("step",ticks=46);assert c.call("inspect_vitality")["actors"]["player"]["units"]==80
        c.call("step",ticks=15);assert c.call("inspect_vitality")["actors"]["player"]["units"]==92
        c.call("capture",path="artifacts/vitality-demo.png",width=1280,height=900)
        c.call("save_replay",path="artifacts/vitality-demo.replay.json");assert c.call("run_replay",path="artifacts/vitality-demo.replay.json")["matched"]
        checks.append("Demo uses shared NPC/player executor: enemy deals20, healer restores12, optional vitality on passive actor with no grants")
        c.call("reset")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/vitality-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
