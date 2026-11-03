#!/usr/bin/env python3
"""NPC ability policies, execution evidence and playable demo through shared MCP commands."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import vital, definition, loadout


def objective(policy, slot=1, priority=2):
    return {"priority":priority,"when":"always","action":{"kind":"use_ability","slot":slot,"retry_ticks":30,"policy":policy}}


def wait():
    return {"priority":0,"when":"always","action":{"kind":"wait"}}


def policy(subject=None, below=7000, margin=0.4):
    return {"vitality_below":None if subject is None else {"subject":subject,"basis_points":below},
            "approach":None if margin is None else {"margin":margin}}


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-policy-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert "basis_points" in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        heal=definition("heal","actor","heal",30,12,60);heal["range"]=3
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[heal]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[0,0.8,-6];player["vitality"]=vital(40);player["footsteps"]=None
        healer=actor("healer",[0,0.8,0],[0.15,0.5,1],"player");healer["vitality"]=vital();healer["abilities"]=loadout(["heal"])
        healer["npc"]["speed"]=3;healer["npc"]["awareness"]["fov_degrees"]=360
        healer["npc"]["objectives"]=[objective(policy("target")),wait()]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,healer]])
        c.call("set_debug_view",npc_routes=True,npc_vision=False)
        c.call("step",ticks=1)
        before=c.call("inspect")
        assert before["scene"]["version"]==SCENE_VERSION
        assert before["npcs"]["healer"]["objectives"]["ability_decisions"][0]["phase"]=="approaching"
        assert not before["abilities"]["receipts"]
        c.call("capture",path="artifacts/policy-approaching.png",width=1280,height=900)
        c.call("step",ticks=240)
        after=c.call("inspect")
        assert after["vitality"]["actors"]["player"]["units"]==70
        assert after["npcs"]["healer"]["objectives"]["ability_decisions"][0]["phase"]=="vitality_satisfied"
        assert [r["outcome"] for r in after["abilities"]["receipts"]]==["started","completed"]
        assert after["abilities"]["actors"]["healer"]["resources"][0]["units"]==995
        c.call("capture",path="artifacts/policy-healed.png",width=1280,height=900)
        c.call("capture",path="artifacts/policy-healed-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/policy-healed.png").read_bytes()==(ROOT/"artifacts/policy-healed-repeat.png").read_bytes()
        c.call("save_replay",path="artifacts/policy.replay.json");assert c.call("run_replay",path="artifacts/policy.replay.json")["matched"]
        prefix=c.call("inspect_replay",path="artifacts/policy.replay.json",tick=1)["snapshot"]
        assert prefix["npcs"]==before["npcs"]
        c.call("capture_replay",replay_path="artifacts/policy.replay.json",path="artifacts/policy-prefix.png",tick=1,width=1280,height=900)
        assert (ROOT/"artifacts/policy-approaching.png").read_bytes()==(ROOT/"artifacts/policy-prefix.png").read_bytes()
        checks.append("Approach without rejected attempts, exactly one heal below70%, threshold stop, costs, final/prefix replay and repeated GPU captures")
        bad=copy.deepcopy(healer);bad["npc"]["objectives"][0]["action"]["policy"]["approach"]["margin"]=4
        c.call("upsert",entity=bad,expected_error=True);assert c.call("inspect")==after
        changed=copy.deepcopy(healer);changed["npc"]["objectives"][0]["action"]["policy"]["approach"]=None
        c.call("upsert",entity=changed);c.call("step",ticks=60)
        assert c.call("inspect_npcs")["npcs"]["healer"]["objectives"]["ability_decisions"][0]["phase"]=="out_of_range"
        assert not c.call("inspect_abilities")["receipts"]
        c.call("undo");c.call("step",ticks=1)
        assert c.call("inspect_npcs")["npcs"]["healer"]["objectives"]["ability_decisions"][0]["phase"]=="approaching"
        c.call("redo");c.call("reset")
        checks.append("Atomic invalid policy, opt-in approach versus wait, undo/redo/reset reconstruct decision state")
        # Demo: red approaches/attacks; blue heals only a visible wounded player; purple self-heals.
        strike=definition("strike","actor","damage",35,15,30);strike["range"]=6
        self_heal=definition("self_heal","self_actor","heal",35,30,60);self_heal["range"]=0
        ally_heal=definition("ally_heal","actor","heal",35,30,60);ally_heal["range"]=6
        enemy_hit=definition("enemy_strike","actor","damage",15,30,120);enemy_hit["range"]=2.5
        support=definition("support","actor","heal",20,45,180);support["range"]=4
        recover=definition("recover","self_actor","heal",30,45,120);recover["range"]=0
        catalog["abilities"]=[strike,self_heal,ally_heal,enemy_hit,support,recover]
        player["transform"]["position"]=[0,0.8,-3];player["vitality"]=vital();player["abilities"]=loadout(["strike","self_heal","ally_heal"])
        enemy=actor("enemy",[5,0.8,2],[0.9,0.15,0.1],"player");enemy["vitality"]=vital();enemy["abilities"]=loadout(["enemy_strike"])
        enemy["npc"]["speed"]=2.5;enemy["npc"]["awareness"]["fov_degrees"]=360
        enemy["npc"]["awareness"]["vision_range"]=14
        enemy["npc"]["objectives"]=[objective(policy()),{"priority":1,"when":"evidence","action":{"kind":"respond"}},wait()]
        # During cooldown stay at range; respond is only for lost visual evidence.
        enemy["npc"]["objectives"][1]["when"]="visual_memory"
        enemy["npc"]["objectives"].insert(1,{"priority":1,"when":"visible_target","action":{"kind":"wait"}})
        healer["transform"]["position"]=[-4,0.8,2];healer["abilities"]=loadout(["support"])
        healer["npc"]["awareness"]["vision_range"]=14
        healer["npc"]["objectives"]=[objective(policy("target",7000,0.4)),wait()]
        dummy=actor("recoverer",[-4,0.8,-4],[0.75,0.2,0.85]);dummy["vitality"]=vital();dummy["abilities"]=loadout(["recover"])
        dummy["npc"]["objectives"]=[objective(policy("source",5000,None)),wait()]
        wall={"id":"screen","name":"Occlusion screen","body":"fixed","color":[0.25,0.65,0.5],"transform":{"position":[0,1.5,4],"size":[4,3,0.6]}}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,healer,enemy,dummy,wall]])
        c.call("set_camera",camera={"position":[10,12,15],"target":[0,0.8,-1],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("select_entity",id="enemy")
        c.call("save_scene",path="examples/npc-ability-policy.scene.json")
        c.call("step",ticks=1);assert not c.call("inspect_abilities")["receipts"]
        assert c.call("inspect_npcs")["npcs"]["healer"]["objectives"]["ability_decisions"][0]["phase"]=="vitality_satisfied"
        c.call("step",ticks=600)
        receipts=c.call("inspect_abilities")["receipts"]
        assert any(r["actor"]=="enemy" and r["outcome"]=="completed" for r in receipts)
        assert any(r["actor"]=="healer" and r["outcome"]=="completed" for r in receipts)
        assert not any(r["actor"]=="recoverer" for r in receipts)
        c.call("capture",path="artifacts/policy-demo.png",width=1280,height=900)
        c.call("save_replay",path="artifacts/policy-demo.replay.json");assert c.call("run_replay",path="artifacts/policy-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-ability-policy.scene.json")
        c.call("activate_ability",actor="player",slot=1,target="recoverer");c.call("step",ticks=46)
        assert c.call("inspect_vitality")["actors"]["recoverer"]["units"]==65
        c.call("activate_ability",actor="player",slot=1,target="recoverer");c.call("step",ticks=62)
        assert c.call("inspect_vitality")["actors"]["recoverer"]["units"]==60
        checks.append("Playable MCP demo: attacker approaches, healer waits for visible health below70%, own recovery below50%, all using shared executor")
        c.call("reset")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/ability-policy-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
