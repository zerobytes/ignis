#!/usr/bin/env python3
"""Support effects filtered through the shared dispel selector; MCP and replay."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import definition, loadout, vital


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-dispel-triage-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert 'require_dispel_match' in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":24,"depth":20,"obstacles":0,"seed":42})
        cleanse=definition("cleanse","actor","heal",1,12,30)
        cleanse.update(range=8,relation="allied",effects=[{"kind":"dispel","recipient":"target","category":"ailment","max_instances":1,"require_match":False}])
        poison=definition("poison","actor","damage",2);poison.update(range=20,cast_ticks=0,cooldown_ticks=0,costs=[],periodic={"interval_ticks":60,"pulses":10,"reapply":"independent"},periodic_category="ailment")
        protected=copy.deepcopy(poison);protected.update(id="protected",dispellable=False)
        restore=copy.deepcopy(cleanse);restore.update(id="restore",cast_ticks=0,cooldown_ticks=0,range=20,costs=[],relation=None)
        hex_spell=definition("hex","actor","heal",1);hex_spell.update(range=20,cast_ticks=0,cooldown_ticks=0,costs=[],effects=[{"kind":"apply_buff","recipient":"target","buff":"hex"}])
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[cleanse,poison,protected,restore,hex_spell],"attributes":{"buffs":[{"id":"hex","duration_ticks":600,"reapply":"independent","category":"ailment","modifiers":[{"attribute":"combat.damage_received.multiplier","operation":{"kind":"multiply","basis_points":12000}}]}]}}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player.update(faction="team",vitality=vital(),abilities=loadout(["poison","protected","restore","hex"]),footsteps=None);player["transform"]["position"]=[0,.8,-6]
        wait={"priority":0,"when":"always","action":{"kind":"wait"}}
        healer=actor("healer",[0,.8,0],[.15,.9,.4],"enemy");healer.update(faction="team",vitality=vital(),abilities=loadout(["cleanse"]),blocks_vision=False)
        healer["npc"]["awareness"].update(fov_degrees=360,vision_range=20)
        policy={"allies":{"ranking":"lowest_health_ratio","include_self":True},"require_dispel_match":True}
        healer["npc"]["objectives"]=[{"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":1,"policy":policy}},wait]
        a=actor("ally_a",[-2,.8,-2],[.2,.5,1]);a.update(faction="team",vitality=vital(20),blocks_vision=False)
        b=actor("ally_b",[3,.8,-2],[.8,.2,.85]);b.update(faction="team",vitality=vital(),blocks_vision=False)
        enemy=actor("enemy",[-4,.8,3],[.9,.2,.15]);enemy.update(faction="foes",vitality=vital(),blocks_vision=False)
        for e in [a,b,enemy]:e["npc"]["objectives"]=[wait]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,healer,a,b,enemy]])
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("activate_ability",actor="player",slot=2,target="ally_a");c.call("step",ticks=1)
        c.call("activate_ability",actor="player",slot=1,target="ally_b");c.call("step",ticks=2)
        first=c.call("inspect");assert first["abilities"]["actors"]["healer"]["cast"]["target"]=="ally_b"
        assert first["npcs"]["healer"]["memory"]["actor_id"]=="enemy"
        c.call("capture",path="artifacts/dispel-triage-first.png",width=1280,height=900)
        c.call("save_replay",path="artifacts/dispel-triage.replay.json")
        assert c.call("run_replay",path="artifacts/dispel-triage.replay.json")["matched"]
        c.call("capture_replay",replay_path="artifacts/dispel-triage.replay.json",path="artifacts/dispel-triage-prefix.png",tick=3,width=1280,height=900)
        assert (ROOT/"artifacts/dispel-triage-first.png").read_bytes()==(ROOT/"artifacts/dispel-triage-prefix.png").read_bytes()
        c.call("step",ticks=12);st=c.call("inspect_abilities")
        assert st["actors"]["healer"]["resources"][0]["units"]==995 and len(st["periodic"]["active"])==1
        checks.append("Protected wounded ally excluded before health ranking; full-health affected ally selected; independent enemy memory and identical replay pixels")
        c.call("reset");c.call("activate_ability",actor="player",slot=4,target="ally_b");c.call("step",ticks=2)
        assert c.call("inspect_abilities")["actors"]["healer"]["cast"]["target"]=="ally_b"
        c.call("activate_ability",actor="player",slot=3,target="ally_b");c.call("step",ticks=2)
        st=c.call("inspect_abilities");assert st["actors"]["healer"]["cast"] is None and st["actors"]["healer"]["resources"][0]["units"]==1000
        checks.append("Buff shares category and selection; external removal cancels optional dispel before payment")
        wall={"id":"screen","name":"Occlusion screen","body":"fixed","color":[.25,.65,.5],"transform":{"position":[1.5,1,-1],"size":[.4,2,1.5]}}
        c.call("upsert",entity=wall);c.call("activate_ability",actor="player",slot=1,target="ally_b");c.call("step",ticks=5)
        hidden_state=c.call("inspect_abilities")
        assert hidden_state["actors"]["healer"]["cast"] is None and len(hidden_state["periodic"]["active"])==1
        before=c.call("inspect");bad=copy.deepcopy(catalog);bad["abilities"][0]["effects"]=[{"kind":"heal","recipient":"target","amount":1}]
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        c.call("undo");c.call("redo");c.call("reset")
        checks.append("Hidden status does not enter triage; incompatible definition edit is atomic; undo/redo/reset preserve policy")
        # Playable demo: one protected blue patient, one removable purple patient, hidden gray.
        self_poison=copy.deepcopy(poison);self_poison.update(id="self_poison",selection="self_actor",range=0,cooldown_ticks=3600)
        self_protected=copy.deepcopy(self_poison);self_protected.update(id="self_protected",dispellable=False)
        self_restore=copy.deepcopy(restore);self_restore.update(id="self_restore",selection="self_actor",range=0)
        catalog["abilities"] += [self_poison,self_protected,self_restore]
        player["abilities"]=loadout(["poison","protected","restore","hex","self_poison","self_restore"])
        # Separate player self-application cooldown from the autonomous patients.
        player_poison=copy.deepcopy(self_poison);player_poison.update(id="player_poison",cooldown_ticks=30)
        catalog["abilities"].append(player_poison);player["abilities"]["grants"][4]="player_poison"
        cleanse.update(cast_ticks=60,cooldown_ticks=120,range=4)
        policy["approach"]={"margin":.4};healer["npc"]["objectives"][0]["action"]["retry_ticks"]=15
        a["vitality"]=vital(50);a["transform"]["position"]=[-3,.8,-2]
        b["transform"]["position"]=[3,.8,-2]
        hidden=actor("hidden_ally",[4,.8,4],[.6,.6,.6]);hidden.update(faction="team",vitality=vital(),blocks_vision=False)
        for e,grant in [(a,"self_protected"),(b,"self_poison"),(hidden,"self_poison")]:
            e["abilities"]=loadout([grant]);e["npc"]["objectives"]=[{"priority":1,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":15}},wait]
        wall["transform"]={"position":[2,1.5,2],"size":[.4,3,3]}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,healer,a,b,hidden,enemy,wall]])
        c.call("set_camera",camera={"position":[12,18,-17],"target":[0,1,0],"fov_degrees":48})
        c.call("select_entity",id="ally_b");c.call("save_scene",path="examples/npc-dispel-triage.scene.json")
        c.call("step",ticks=2);assert c.call("inspect_abilities")["actors"]["healer"]["cast"]["target"]=="ally_b"
        c.call("capture",path="artifacts/dispel-triage-demo.png",width=1280,height=900)
        c.call("step",ticks=75);st=c.call("inspect_abilities");assert len(st["periodic"]["active"])==2
        c.call("save_replay",path="artifacts/dispel-triage-demo.replay.json");assert c.call("run_replay",path="artifacts/dispel-triage-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-dispel-triage.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION
        checks.append("Demo authored/saved by MCP; protected/hidden effects remain while visible removable effect is dispelled; replay and load")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/dispel-triage-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    args=p.parse_args();verify(args.binary.resolve(),args.window)
