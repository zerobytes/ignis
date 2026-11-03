#!/usr/bin/env python3
"""Extensible attributes, temporary buffs and conditional gameplay through MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import vital, definition, loadout
from verify_ability_policy import objective, policy, wait
from verify_channel import channel
from verify_periodic import schedule


def modifier(key,operation,tags=(),abilities=(),priority=0):
    return {"attribute":key,"operation":operation,"tags_all":list(tags),"abilities":list(abilities),"priority":priority}


def protection(key,tags=()):return modifier(key,{"kind":"set","value":{"kind":"bool","value":False}},tags)


def multiplier(value,tags=()):return modifier("combat.damage_received.multiplier",{"kind":"multiply","basis_points":value},tags)


def buff(id,mods,duration=600):
    return {"id":id,"duration_ticks":duration,"reapply":"refresh","category":"enhancement","dispellable":True,"modifiers":mods}


def apply(id,buff_id,selection="self_actor"):
    a=definition(id,selection,"heal",1);a["range"]=0 if selection=="self_actor" else 12
    a["effects"]=[{"kind":"apply_buff","recipient":"target","buff":buff_id}];return a


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-attribute-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"));assert all(k in schema for k in ["inspect_attribute","apply_buff","tags_all","basis_points"])
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        wards=[buff("focus",[protection("ability.interruptible"),protection("effect.dispellable",["poison"])]),buff("polar",[multiplier(0,["ice"]),multiplier(15000,["fire"])]),buff("immune",[multiplier(0)])]
        own=definition("channel","self_actor","heal",10,30,60);own.update(range=0,channel=channel("on_start",60,10))
        poison=definition("poison","actor","damage",5,0,30);poison.update(tags=["poison"],periodic=schedule(60,12),periodic_category="poison")
        clean=definition("clean","self_actor","heal",1);clean.update(range=0,effects=[{"kind":"dispel","recipient":"target","category":"enhancement","max_instances":8,"require_match":True}])
        fire=definition("fire","actor","damage",10);fire["tags"]=["fire"];fire["effects"]=[{"kind":"damage_components","recipient":"target","components":[{"amount":10,"tags":["fire"]}]}]
        ice=definition("ice","actor","damage",10);ice["tags"]=["ice"];ice["effects"]=[{"kind":"damage_components","recipient":"target","components":[{"amount":10,"tags":["ice"]}]}]
        cut=definition("cut","actor","heal",1);cut["effects"]=[{"kind":"interrupt","recipient":"target","require_active":True}]
        antidote=definition("antidote","self_actor","heal",1);antidote.update(range=0,effects=[{"kind":"dispel","recipient":"target","category":"poison","max_instances":2,"require_match":True}])
        defs=[apply("focus","focus"),apply("polar","polar"),apply("immune","immune"),own,poison,clean,fire,ice,cut,antidote]
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":defs,"attributes":{"tags":["fire","ice","poison"],"buffs":wards,"definitions":[{"key":"craft.temperature","default":{"kind":"number","units":10000},"minimum":0,"maximum":100000}],"bases":[{"entity":"floor","attribute":"craft.temperature","value":{"kind":"number","units":20000}}]}}
        # A custom key uses the identical evaluator, with no gameplay branch for its name.
        wards[0]["modifiers"].append(modifier("craft.temperature",{"kind":"add","units":5000}))
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[0,0.8,-5];player["vitality"]=vital(50);player["footsteps"]=None;player["abilities"]=loadout([d["id"] for d in defs])
        target=actor("dummy",[0,0.8,0],[0.6,0.2,0.8],"player");target["vitality"]=vital();target["abilities"]=loadout(["fire","ice","cut"]);target["npc"]["awareness"].update(vision_range=20,fov_degrees=360);target["npc"]["objectives"]=[wait()]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,target]])
        c.call("set_camera",camera={"position":[11,13,17],"target":[0,0.8,0],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        def state():return c.call("inspect_abilities")
        def attr(key,entity="player",ability=None):
            args={"entity":entity,"attribute":key}
            if ability is not None:args["ability"]=ability
            return c.call("inspect_attribute",**args)
        def cast(slot=1,who="player",target=None):
            args={"actor":who,"slot":slot}
            if target is not None:args["target"]=target
            c.call("activate_ability",**args)
        def hp(id="player"):return c.call("inspect_vitality")["actors"][id]["units"]
        cast();c.call("step",ticks=10)
        v=attr("ability.interruptible",ability="channel");assert v["base"]["value"] is True and v["effective"]["value"] is False and v["contributions"][0]["source"]=="player"
        assert attr("craft.temperature")["effective"]["units"]==15000 and attr("craft.temperature","floor")["effective"]["units"]==20000
        prefix=c.call("inspect");c.call("capture",path="artifacts/attributes-active.png",width=1280,height=900);c.call("capture",path="artifacts/attributes-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/attributes-active.png").read_bytes()==(ROOT/"artifacts/attributes-repeat.png").read_bytes()
        c.call("step",ticks=1);cast(4);c.call("step",ticks=40);cast(3,"dummy","player");c.call("step",ticks=1)
        assert state()["actors"]["player"]["cast"]["phase"]=="channeling" and state()["receipts"][-1]["outcome"]=="no_interruptible_execution"
        c.call("save_replay",path="artifacts/attributes.replay.json");assert c.call("run_replay",path="artifacts/attributes.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/attributes.replay.json",tick=10)["snapshot"]["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/attributes.replay.json",path="artifacts/attributes-prefix.png",tick=10,width=1280,height=900)
        assert (ROOT/"artifacts/attributes-active.png").read_bytes()==(ROOT/"artifacts/attributes-prefix.png").read_bytes()
        checks.append("Open key registry/custom numeric attribute and non-actor base, MCP base/effective/provenance, cast protection, full/prefix replay and exact repeated/replayed lime-bar PNG")
        c.call("reset");cast(2);c.call("step",ticks=1);cast(2,"dummy","player");c.call("step",ticks=1);assert hp()==50
        cast(1,"dummy","player");c.call("step",ticks=1);assert hp()==35 and state()["receipts"][-1]["effects"][0]["adjusted_amount"]==15
        cast(3);c.call("step",ticks=1);cast(1,"dummy","player");c.call("step",ticks=1);assert hp()==35
        cast(6);c.call("step",ticks=1);assert not state()["buffs"]["active"]
        cast(2,"dummy","player");c.call("step",ticks=1);assert hp()==25
        checks.append("Ice immunity/fire vulnerability, general immunity, buff dispel and restoration through the same damage executor")
        before=c.call("inspect")
        for key,value in [("duration_ticks",0),("duration_ticks",216001)]:
            bad=copy.deepcopy(catalog);bad["attributes"]["buffs"][0][key]=value;c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        bad=copy.deepcopy(catalog);bad["attributes"]["buffs"][0]["modifiers"][0]["attribute"]="missing";c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        c.call("inspect_attribute",entity="player",attribute="missing",expected_error=True);assert c.call("inspect")==before
        bad=copy.deepcopy(catalog);bad["attributes"]["buffs"][0]["modifiers"][0]["operation"]={"kind":"add","units":10};c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        short=copy.deepcopy(catalog);short["attributes"]["buffs"][0]["duration_ticks"]=2;c.call("set_ability_catalog",catalog=short);cast();c.call("step",ticks=2);assert not state()["buffs"]["active"] and attr("ability.interruptible")["effective"]["value"] is True
        c.call("undo");assert c.call("inspect")["scene"]["ability_catalog"]["attributes"]["buffs"][0]["duration_ticks"]==600
        c.call("redo");assert c.call("inspect")["scene"]["ability_catalog"]["attributes"]["buffs"][0]["duration_ticks"]==2
        checks.append("Invalid keys/types/timing and diagnostic queries preserve scene/history; exact expiration, undo/redo/reset")
        # Save a six-slot comparison demo. All runtime actions remain shared MCP commands.
        fire.update(cast_ticks=30,cooldown_ticks=90,delivery={"kind":"projectile","speed":6,"half_extent":0.1,"max_distance":20});fire["effects"][0]["components"][0]["amount"]=3
        ice.update(cast_ticks=30,cooldown_ticks=90,delivery={"kind":"projectile","speed":6,"half_extent":0.1,"max_distance":20});ice["effects"][0]["components"][0]["amount"]=3
        cut["cooldown_ticks"]=120
        player["abilities"]=loadout([d["id"] for d in defs[:6]]);player["vitality"]={"initial":100,"maximum":200,"scale":1}
        c.call("edit_scene",edits=[{"op":"remove","id":"dummy"},{"op":"set_ability_catalog","catalog":catalog},{"op":"upsert","entity":player}])
        npcs=[]
        for id,pos,color,grant,aim in [("fire_caster",[4,0.8,3],[0.9,0.15,0.1],"fire","player"),("ice_caster",[-4,0.8,3],[0.1,0.5,1],"ice","player"),("interrupter",[0,0.8,3],[0.7,0.2,0.9],"cut","player"),("cleanser",[-5,0.8,-2],[0.2,0.85,0.3],"antidote",None)]:
            n=actor(id,pos,color,aim);n["vitality"]={"initial":1000,"maximum":1000,"scale":1};n["abilities"]=loadout([grant]);n["npc"]["awareness"].update(vision_range=20,fov_degrees=360);n["npc"]["objectives"]=[objective(policy(margin=None)),wait()];npcs.append(n)
        c.call("edit_scene",edits=[{"op":"upsert","entity":n} for n in npcs]);c.call("save_scene",path="examples/npc-attributes.scene.json")
        cast(4);c.call("step",ticks=90);assert any(r["actor"]=="player" and r["outcome"]=="interrupted_by_ability" for r in state()["receipts"])
        c.call("reset");cast();c.call("step",ticks=1);cast(4);c.call("step",ticks=90);assert state()["actors"]["player"]["cast"]["phase"]=="channeling"
        c.call("capture",path="artifacts/attributes-demo.png",width=1280,height=900)
        c.call("cancel_ability",actor="player",execution_id=state()["actors"]["player"]["cast"]["execution_id"]);c.call("step",ticks=1);cast(5,target="cleanser");c.call("step",ticks=60)
        assert any(p["target"]=="cleanser" and p["dispellable"] is False for p in state()["periodic"]["active"])
        c.call("step",ticks=510);assert not any(p["target"]=="cleanser" for p in state()["periodic"]["active"])
        c.call("save_replay",path="artifacts/attributes-demo.replay.json");assert c.call("run_replay",path="artifacts/attributes-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-attributes.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION and not state()["buffs"]["active"]
        checks.append("Demo authored/saved/reloaded via MCP: NPC interruption prevented by focus, poison protects by author until buff expiry then cleanser removes it; same NPC executor and replay")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/attributes-verification.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
