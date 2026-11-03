#!/usr/bin/env python3
"""Observed ally triage, independent combat memory, MCP authoring and replay."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import definition, loadout, vital


def verify(binary, window):
    c=Client(binary,window); checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-triage-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert 'lowest_health_ratio' in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":24,"depth":20,"obstacles":0,"seed":42})
        heal=definition("support","actor","heal",30,12,30);heal.update(range=8,relation="allied")
        harm=definition("injure","actor","damage",20);harm["range"]=20
        restore=definition("restore","actor","heal",1000);restore["range"]=20
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[heal,harm,restore]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player.update(faction="foes",vitality=vital(),abilities=loadout(["injure","restore"]),footsteps=None)
        player["transform"]["position"]=[0,.8,-6]
        wait={"priority":0,"when":"always","action":{"kind":"wait"}}
        healer=actor("healer",[0,.8,0],[.15,.9,.4],"player")
        healer.update(faction="team",vitality=vital(),abilities=loadout(["support"]),blocks_vision=False)
        healer["npc"]["awareness"].update(fov_degrees=360,vision_range=20)
        policy={"allies":{"ranking":"lowest_health_ratio","include_self":False},"vitality_below":{"subject":"target","basis_points":7000}}
        healer["npc"]["objectives"]=[{"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":1,"policy":policy}},wait]
        a=actor("ally_a",[-2,.8,-2],[.2,.5,1]);a.update(faction="team",vitality=vital(40),blocks_vision=False)
        b=actor("ally_b",[2,.8,-2],[.8,.2,.85]);b.update(faction="team",vitality={"initial":60,"maximum":200,"scale":1},blocks_vision=False)
        for e in [a,b]:e["npc"]["objectives"]=[wait]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,healer,a,b]])
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("step",ticks=1)
        first=c.call("inspect");ns=first["npcs"]["healer"]
        assert ns["memory"]["actor_id"]=="player" and ns["objectives"]["ability_decisions"][0]["target_id"]=="ally_b"
        assert {o["actor_id"] for o in ns["support"]["observed_allies"]}=={"ally_a","ally_b"}
        c.call("capture",path="artifacts/triage-first.png",width=1280,height=900)
        c.call("activate_ability",actor="player",slot=1,target="ally_a");c.call("step",ticks=1)
        assert c.call("inspect_abilities")["actors"]["healer"]["cast"]["target"]=="ally_b"
        c.call("step",ticks=11)
        assert c.call("inspect_vitality")["actors"]["ally_b"]["units"]==90
        c.call("save_replay",path="artifacts/triage.replay.json")
        assert c.call("run_replay",path="artifacts/triage.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/triage.replay.json",tick=1)["snapshot"]["npcs"]==first["npcs"]
        c.call("capture_replay",replay_path="artifacts/triage.replay.json",path="artifacts/triage-prefix.png",tick=1,width=1280,height=900)
        assert (ROOT/"artifacts/triage-first.png").read_bytes()==(ROOT/"artifacts/triage-prefix.png").read_bytes()
        checks.append("Exact ratios, independent enemy memory and observed ally IDs; cast stable after competing injury; full/prefix replay and identical GPU pixels")
        c.call("reset");c.call("step",ticks=1)
        c.call("activate_ability",actor="player",slot=2,target="ally_b");c.call("step",ticks=2)
        st=c.call("inspect_abilities");assert st["actors"]["healer"]["cast"] is None
        assert st["actors"]["healer"]["resources"][0]["units"]==1000
        assert any(r["actor"]=="healer" and r["outcome"]=="cancelled" for r in st["receipts"])
        checks.append("External restoration invalidates support cast without charge or silent retarget")
        wall={"id":"screen","name":"Occlusion screen","body":"fixed","color":[.25,.65,.5],"transform":{"position":[1,1,-1],"size":[.3,2,1.5]}}
        c.call("upsert",entity=wall);c.call("step",ticks=1)
        assert c.call("inspect_abilities")["actors"]["healer"]["cast"]["target"]=="ally_a"
        before=c.call("inspect");bad=copy.deepcopy(healer);bad["npc"]["objectives"][0]["action"]["policy"]["allies"]["ranking"]="hidden_health"
        c.call("upsert",entity=bad,expected_error=True);assert c.call("inspect")==before
        c.call("undo");c.call("step",ticks=1);assert c.call("inspect_abilities")["actors"]["healer"]["cast"]["target"]=="ally_b"
        c.call("redo");c.call("reset")
        checks.append("Hidden more urgent ally excluded; invalid policy atomic; undo/redo/reset rebuild observation and selection")
        # Playable demo: green healer supports wounded teammates while watching red enemy.
        player.update(faction="team",abilities=loadout(["injure","restore","self_heal"]));player["vitality"]=vital()
        self_heal=definition("self_heal","self_actor","heal",30,15,30);self_heal["range"]=0
        harm.update(cast_ticks=0,cooldown_ticks=15,range=14)
        restore.update(effects=[{"kind":"heal","recipient":"target","amount":30}],cast_ticks=15,cooldown_ticks=30,range=14)
        heal.update(range=4,cast_ticks=45,cooldown_ticks=120)
        catalog["abilities"].append(self_heal)
        healer["npc"]["target"]="enemy";healer["npc"]["speed"]=3
        policy["approach"]={"margin":.4};policy["allies"]["include_self"]=True
        healer["npc"]["objectives"][0]["action"]["retry_ticks"]=15
        a["transform"]["position"]=[-3,.8,-2];b["transform"]["position"]=[3,.8,-2]
        a["vitality"]=vital(50);b["vitality"]=vital(35)
        hidden=actor("hidden_ally",[4,.8,4],[.6,.6,.6]);hidden.update(faction="team",vitality=vital(10),blocks_vision=False);hidden["npc"]["objectives"]=[wait]
        enemy=actor("enemy",[-4,.8,3],[.9,.2,.15]);enemy.update(faction="foes",vitality=vital());enemy["npc"]["objectives"]=[wait]
        wall["transform"]={"position":[2,1.5,2],"size":[.4,3,3]}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,healer,a,b,hidden,enemy,wall]])
        c.call("set_camera",camera={"position":[12,18,-17],"target":[0,1,0],"fov_degrees":48})
        c.call("select_entity",id="ally_b")
        c.call("save_scene",path="examples/npc-triage.scene.json")
        c.call("step",ticks=1);st=c.call("inspect")
        assert st["npcs"]["healer"]["memory"]["actor_id"]=="enemy"
        assert st["abilities"]["actors"]["healer"]["cast"]["target"]=="ally_b"
        assert "hidden_ally" not in [o["actor_id"] for o in st["npcs"]["healer"]["support"]["observed_allies"]]
        c.call("capture",path="artifacts/triage-demo.png",width=1280,height=900)
        c.call("step",ticks=600);assert c.call("inspect_vitality")["actors"]["hidden_ally"]["units"]==10
        c.call("save_replay",path="artifacts/triage-demo.replay.json");assert c.call("run_replay",path="artifacts/triage-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-triage.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION
        checks.append("Playable support demo authored by MCP: threshold/approach, hidden wounded ally and separate enemy; reload and deterministic replay")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/triage-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    args=p.parse_args();verify(args.binary.resolve(),args.window)
