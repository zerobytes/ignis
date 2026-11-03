#!/usr/bin/env python3
"""Transactional categorized dispels, using the shared MCP dispatcher only."""
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
from verify_periodic import schedule


def dispel(required=True,limit=2):
    return {"kind":"dispel","recipient":"target","category":"poison","max_instances":limit,"require_match":required}


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-dispel-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"));assert "periodic_category" in schema and "require_match" in schema and "dispellable" in schema
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        poison=definition("poison","actor","damage",5);poison.update(periodic=schedule(reapply="independent"),periodic_category="poison")
        regen=definition("regeneration","actor","heal",5);regen.update(periodic=schedule(reapply="independent"),periodic_category="regeneration")
        clean=definition("antidote","actor","heal",1);clean["effects"]=[dispel()]
        own=copy.deepcopy(clean);own.update(id="self_antidote",selection="self_actor",range=0)
        bolt=copy.deepcopy(clean);bolt.update(id="cleanse_bolt",delivery=delivery(6,0.14,18))
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[poison,own,clean,regen,bolt]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[-3,0.8,0];player["vitality"]=vital();player["abilities"]=loadout([d["id"] for d in catalog["abilities"]]);player["footsteps"]=None
        target=actor("dummy",[3,0.8,0],[0.75,0.2,0.85]);target["vitality"]=vital();target["npc"]["objectives"]=[wait()]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,target]])
        c.call("set_camera",camera={"position":[10,12,15],"target":[0,0.8,0],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        def state():return c.call("inspect_abilities")
        def cast(slot,target=None):
            args={"actor":"player","slot":slot}
            if target:args["target"]=target
            c.call("activate_ability",**args);c.call("step",ticks=1)
        for _ in range(3):cast(1,"dummy")
        cast(4,"dummy");prefix=c.call("inspect");original=state()["periodic"]["active"]
        assert [e["category"] for e in original]==["poison"]*3+["regeneration"]
        c.call("capture",path="artifacts/dispel-before.png",width=1280,height=900)
        c.call("step",ticks=1);cast(3,"dummy");after=state()
        assert [e["execution_id"] for e in after["periodic"]["active"]]==[e["execution_id"] for e in original[2:]]
        removed=[r for r in after["periodic"]["receipts"] if r["outcome"]=="dispelled"]
        assert [r["execution_id"] for r in removed]==[e["execution_id"] for e in original[:2]]
        assert all(r["removed_by"]["actor"]=="player" for r in removed)
        assert after["receipts"][-1]["effects"][0]["dispel"]=={"category":"poison","removed":2}
        c.call("save_replay",path="artifacts/dispel.replay.json");assert c.call("run_replay",path="artifacts/dispel.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/dispel.replay.json",tick=4)["snapshot"]["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/dispel.replay.json",path="artifacts/dispel-prefix.png",tick=4,width=1280,height=900)
        assert (ROOT/"artifacts/dispel-before.png").read_bytes()==(ROOT/"artifacts/dispel-prefix.png").read_bytes()
        checks.append("MCP schema/categories, oldest bounded removals preserve regeneration, exact removal provenance and prefix/final replay/pixels")
        c.call("reset");cast(2);assert state()["receipts"][-1]["outcome"]=="no_matching_effect" and state()["actors"]["player"]["resources"][0]["units"]==1000
        before=c.call("inspect");bad=copy.deepcopy(catalog);bad["abilities"][0]["periodic_category"]="has space"
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        bad=copy.deepcopy(catalog);bad["abilities"][2]["effects"][0]["max_instances"]=65
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        optional=copy.deepcopy(catalog);optional["abilities"][2]["effects"][0]["require_match"]=False
        c.call("set_ability_catalog",catalog=optional);cast(3,"dummy");assert state()["actors"]["player"]["resources"][0]["units"]==995 and state()["receipts"][-1]["effects"][0]["dispel"]["removed"]==0
        c.call("undo");c.call("redo");c.call("set_ability_catalog",catalog=catalog)
        cast(1,"dummy");before=c.call("inspect");mixed=copy.deepcopy(catalog);mixed["abilities"][2]["effects"].append({"kind":"resource","recipient":"target","resource":"energy","delta":5})
        c.call("set_ability_catalog",catalog=mixed);cast(1,"dummy");cast(3,"dummy");assert len(state()["periodic"]["active"])==1 and state()["actors"]["player"]["resources"][0]["units"]==995 and state()["receipts"][-1]["outcome"]=="missing_resource"
        c.call("reset");assert not state()["periodic"]["active"]
        checks.append("Required absence has no cost; optional absence pays; invalid authoring atomic, mixed transaction failure does not remove; undo/redo/reset")
        c.call("set_ability_catalog",catalog=catalog);cast(1,"dummy");cast(5,"dummy");assert len(state()["periodic"]["active"])==1
        c.call("step",ticks=80);assert not state()["periodic"]["active"] and state()["actors"]["player"]["resources"][0]["units"]==990
        assert state()["projectiles"]["receipts"][-1]["outcome"]=="hit"
        c.call("capture",path="artifacts/dispel-cleared.png",width=1280,height=900)
        c.call("capture",path="artifacts/dispel-cleared-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/dispel-cleared.png").read_bytes()==(ROOT/"artifacts/dispel-cleared-repeat.png").read_bytes()
        c.call("save_replay",path="artifacts/dispel-impact.replay.json");assert c.call("run_replay",path="artifacts/dispel-impact.replay.json")["matched"]
        c.call("capture_replay",replay_path="artifacts/dispel-impact.replay.json",path="artifacts/dispel-cleared-replay.png",width=1280,height=900)
        assert (ROOT/"artifacts/dispel-cleared.png").read_bytes()==(ROOT/"artifacts/dispel-cleared-replay.png").read_bytes()
        checks.append("Projectile dispels on impact with single launch charge, last-duration marker clears, repeated/final pixels and replay match")
        # Protected effects keep their category and do not consume the removal limit.
        protected=copy.deepcopy(poison);protected.update(id="protected_poison",dispellable=False)
        protected_catalog=copy.deepcopy(catalog);protected_catalog["abilities"].append(protected)
        protected_player=copy.deepcopy(player);protected_player["abilities"]=loadout([d["id"] for d in protected_catalog["abilities"]])
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":protected_catalog},{"op":"upsert","entity":protected_player}])
        assert state()["actors"]["player"]["grants"][-1]["dispellable"] is False
        for slot in [6,1,6,1,1]:cast(slot,"dummy")
        original=state()["periodic"]["active"];prefix_tick=c.call("inspect")["tick"];prefix_state=state()
        # Put the next order after the saved prefix boundary (pending inputs are observable).
        c.call("step",ticks=1);cast(3,"dummy")
        assert [e["execution_id"] for e in state()["periodic"]["active"]]==[original[i]["execution_id"] for i in [0,2,4]]
        assert [e["dispellable"] for e in state()["periodic"]["active"]]==[False,False,True]
        assert all(e["category"]=="poison" for e in state()["periodic"]["active"])
        c.call("step",ticks=1);cast(3,"dummy");c.call("step",ticks=1)
        energy=state()["actors"]["player"]["resources"][0]["units"]
        cast(3,"dummy");assert state()["receipts"][-1]["outcome"]=="no_matching_effect"
        assert len(state()["periodic"]["active"])==2 and state()["actors"]["player"]["resources"][0]["units"]==energy
        c.call("save_replay",path="artifacts/dispel-protected.replay.json");assert c.call("run_replay",path="artifacts/dispel-protected.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/dispel-protected.replay.json",tick=prefix_tick)["snapshot"]["abilities"]==prefix_state
        before=c.call("inspect")
        for invalid in ["false",None]:
            bad=copy.deepcopy(protected_catalog);bad["abilities"][-1]["dispellable"]=invalid
            c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        bad=copy.deepcopy(protected_catalog);bad["abilities"][2]["dispellable"]=False
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        c.call("save_scene",path="examples/dispel-protected-verification.scene.json")
        c.call("load_scene",path="examples/dispel-protected-verification.scene.json")
        assert c.call("inspect")["scene"]["ability_catalog"]["abilities"][-1]["dispellable"] is False
        editable=copy.deepcopy(protected_catalog);editable["abilities"][-1]["dispellable"]=True
        c.call("set_ability_catalog",catalog=editable);c.call("undo")
        assert c.call("inspect")["scene"]["ability_catalog"]["abilities"][-1]["dispellable"] is False
        c.call("redo");assert "dispellable" not in c.call("inspect")["scene"]["ability_catalog"]["abilities"][-1]
        checks.append("Protected poison skipped before limit/require_match, no charge for protected-only, schema/inspection/replay prefix, atomic errors, save/load/undo/redo")
        # Manual scene: active shooter, passive target, NPC with self-antidote.
        for d in [poison,own,clean,regen]:d.update(cast_ticks=30,cooldown_ticks=30)
        venom=copy.deepcopy(poison);venom.update(dispellable=False,id="enemy_venom",cast_ticks=60,cooldown_ticks=480,delivery=delivery(6,0.14,18),movement={"mode":"lock"});venom["periodic"]=schedule(60,4);venom["effects"][0]["amount"]=4
        auto=copy.deepcopy(own);auto.update(id="npc_antidote",cast_ticks=90,cooldown_ticks=180,movement={"mode":"lock"})
        catalog["abilities"]=[poison,own,clean,regen,venom,auto]
        player["abilities"]=loadout(["poison","self_antidote","antidote","regeneration"]);player["transform"]["position"]=[0,0.8,-4]
        target["transform"]["position"]=[-4,0.8,1]
        blue=actor("defender",[0,0.8,3],[0.15,0.5,1]);blue["vitality"]=vital();blue["abilities"]=loadout(["npc_antidote"]);blue["npc"]["objectives"]=[objective(policy(margin=None)),wait()]
        shooter=actor("shooter",[4,0.8,5],[0.9,0.18,0.16],"player");shooter["vitality"]=vital();shooter["abilities"]=loadout(["enemy_venom"]);shooter["npc"]["awareness"].update(vision_range=18,fov_degrees=360);shooter["npc"]["objectives"]=[objective(policy(margin=None)),wait()]
        # Low cover off the player's direct line to the blue NPC.
        cover={"id":"cover","name":"Cover","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[3,1.5,0],"size":[1.6,3,0.4]}}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,target,blue,shooter,cover]])
        c.call("save_scene",path="examples/npc-dispel.scene.json")
        c.call("step",ticks=35);assert state()["actors"]["defender"]["resources"][0]["units"]==1000 and state()["actors"]["defender"]["cast"] is None
        cast(1,"defender");c.call("step",ticks=70);assert state()["actors"]["defender"]["cast"] is not None
        c.call("capture",path="artifacts/dispel-demo.png",width=1280,height=900)
        c.call("step",ticks=100);assert not any(e["target"]=="defender" for e in state()["periodic"]["active"]) and state()["actors"]["defender"]["resources"][0]["units"]==995
        c.call("save_replay",path="artifacts/dispel-demo.replay.json");assert c.call("run_replay",path="artifacts/dispel-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-dispel.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION
        checks.append("Demo authored/saved/reloaded via MCP; blue self-antidote starts only with matching own effect, single charge/removal and replay")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/dispel-verification.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
