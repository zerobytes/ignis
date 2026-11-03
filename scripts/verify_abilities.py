#!/usr/bin/env python3
"""Author shared abilities via MCP; inspect cast/resources/replay and offscreen guides."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def loadout(energy, focus, grants):
    return {"resources":[{"resource":"energy","initial":energy,"capacity":100},
                         {"resource":"focus","initial":focus,"capacity":100}],"grants":grants}


def ability(identifier, selection, cast, cost, effects, below=None):
    return {"id":identifier,"selection":selection,"range":8,"relation":"allied" if selection=="actor" else None,
            "cast_ticks":cast,"cooldown_ticks":120,"costs":[{"resource":"energy","units":cost}],
            "effects":effects,"requires_below":below}


def resource(name, delta, recipient="source"):
    return {"kind":"resource","recipient":recipient,"resource":name,"delta":delta}


def actor(identifier, position, color, target=None):
    return {"id":identifier,"name":identifier,"body":"npc","faction":"team","color":color,
            "transform":{"position":position,"size":[0.7,1.6,0.7]},
            "npc":{"target":target,"surface":"floor","speed":2,"stop_distance":1,"enabled":True,
                   "awareness":{"mode":"perceptive","vision_range":10,"fov_degrees":150,
                                "initial_facing_degrees":0,"memory_ticks":90,"search_ticks":120,
                                "search_turn_degrees_per_second":90,"hearing":{"range":12}}}}


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-abilities-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"));assert all(name in schema for name in ["activate_ability","cancel_ability","inspect_resources","set_ability_catalog"])
        c.call("generate",recipe={"width":20,"depth":18,"seed":42,"obstacles":0})
        catalog={"resources":[{"id":"energy","scale":1},{"id":"focus","scale":1}],"abilities":[
            ability("convert","self_actor",90,20,[resource("focus",30)]),
            ability("support","actor",60,15,[resource("focus",25,"target")]),
            ability("signal","self_actor",0,5,[{"kind":"noise","radius":12}]),
            ability("recover","self_actor",90,20,[resource("focus",30)],{"resource":"focus","units":60})]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[-2,0.8,-3];player["faction"]="team";player["abilities"]=loadout(100,0,["convert","support","signal"])
        converter=actor("converter",[-4,0.8,0],[0.75,0.2,0.85]);converter["abilities"]=loadout(100,0,["recover"])
        converter["npc"]["objectives"]=[{"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":60}}, {"priority":1,"when":"always","action":{"kind":"wait"}}]
        supporter=actor("supporter",[0,0.8,1],[0.15,0.55,1],"player");supporter["abilities"]=loadout(100,0,["support"])
        supporter["npc"]["objectives"]=[{"priority":2,"when":"visible_target","action":{"kind":"use_ability","slot":1,"retry_ticks":60}}, {"priority":1,"when":"always","action":{"kind":"wait"}}]
        listener=actor("listener",[4,0.8,2],[0.6,0.6,0.6]);listener["npc"]["awareness"]["fov_degrees"]=45
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,converter,supporter,listener]])
        c.call("set_camera",camera={"position":[10,11,14],"target":[0,0.8,-1],"fov_degrees":48})
        c.call("set_debug_view",npc_vision=False,npc_routes=True,npc_hearing=False,noise_events=True)
        c.call("select_entity",id="converter")
        c.call("save_scene",path="examples/npc-abilities.scene.json")
        c.call("activate_ability",actor="player",slot=1)
        pending=c.call("inspect_abilities");assert len(pending["pending_inputs"])==1
        c.call("save_replay",path="artifacts/abilities-pending.replay.json")
        assert c.call("inspect_replay",path="artifacts/abilities-pending.replay.json")["snapshot"]["abilities"]==pending
        c.call("step",ticks=45)
        prefix=c.call("inspect");a=prefix["abilities"]["actors"]
        assert a["player"]["cast"]["execute_at"]==90 and a["converter"]["cast"]
        assert a["player"]["resources"][0]["units"]==100
        c.call("capture",path="artifacts/abilities-cast.png",width=1280,height=900)
        c.call("capture",path="artifacts/abilities-cast-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/abilities-cast.png").read_bytes()==(ROOT/"artifacts/abilities-cast-repeat.png").read_bytes()
        c.call("step",ticks=46)
        a=c.call("inspect_abilities")["actors"]
        assert a["player"]["resources"][0]["units"]==80 and a["player"]["resources"][1]["units"]==55,a
        assert a["converter"]["resources"][1]["units"]==30
        c.call("capture",path="artifacts/abilities-complete.png",width=1280,height=900)
        c.call("save_replay",path="artifacts/abilities.replay.json")
        assert c.call("run_replay",path="artifacts/abilities.replay.json")["matched"]
        replay=c.call("inspect_replay",path="artifacts/abilities.replay.json",tick=45)
        assert replay["snapshot"]["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/abilities.replay.json",path="artifacts/abilities-prefix.png",tick=45,width=1280,height=900)
        assert (ROOT/"artifacts/abilities-cast.png").read_bytes()==(ROOT/"artifacts/abilities-prefix.png").read_bytes()
        assert (ROOT/"artifacts/abilities-cast.png").read_bytes()!=(ROOT/"artifacts/abilities-complete.png").read_bytes()
        checks.append("Shared player/converter/supporter casts commit exact integer costs/effects; pending/prefix/final replay and resource/cast pixels match")
        c.call("reset");c.call("activate_ability",actor="player",slot=1);c.call("step",ticks=90)
        current=c.call("inspect_abilities")["actors"]["player"]["cast"]
        c.call("cancel_ability",actor="player",execution_id=current["execution_id"]);c.call("step",ticks=1)
        a=c.call("inspect_abilities");assert a["actors"]["player"]["resources"][0]["units"]==100
        assert any(r["actor"]=="player" and r["outcome"]=="cancelled" for r in a["receipts"])
        c.call("activate_ability",actor="player",slot=3);c.call("step",ticks=1)
        assert not c.call("inspect_npcs")["npcs"]["listener"]["heard_this_tick"]
        c.call("step",ticks=1);assert c.call("inspect_npcs")["npcs"]["listener"]["heard_this_tick"]
        c.call("capture",path="artifacts/abilities-signal.png",width=1280,height=900)
        c.call("save_replay",path="artifacts/abilities-signal.replay.json");assert c.call("run_replay",path="artifacts/abilities-signal.replay.json")["matched"]
        checks.append("Cancel at execution deadline costs nothing; instant signal generates next-tick anonymous hearing and replayable yellow guide")
        before=c.call("inspect");bad=copy.deepcopy(catalog);bad["resources"][0]["scale"]=0
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        changed=copy.deepcopy(catalog);changed["abilities"][0]["cast_ticks"]=0
        c.call("set_ability_catalog",catalog=changed);assert c.call("inspect")["tick"]==0
        c.call("undo");assert c.call("inspect")["scene"]["ability_catalog"]==catalog
        c.call("redo");assert c.call("inspect")["scene"]["ability_catalog"]==changed
        c.call("load_scene",path="examples/npc-abilities.scene.json")
        c.call("step",ticks=400)
        a=c.call("inspect_abilities")["actors"];assert a["converter"]["resources"][1]["units"]==60
        c.call("step",ticks=240);assert c.call("inspect_abilities")["actors"]["converter"]==a["converter"]
        checks.append("Invalid authoring is atomic; undo/redo/load reset compiled state; NPC stops converting at own threshold and resumes wait")
        final=c.call("inspect")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":final["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/abilities-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":final["build_id"]}))
    finally:c.close()


if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
