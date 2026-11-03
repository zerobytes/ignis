#!/usr/bin/env python3
"""Author one-hop radio guards and verify provenance, history and replay over MCP."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def guard(identifier, position, color, channel, observer=False):
    npc = {"target":"player","surface":"floor","speed":2,"stop_distance":0.6,"enabled":True,
           "communication":{"mode":"radio","channel":channel,"range":14,"cooldown_ticks":60,
                            "max_age_ticks":180,"send":observer,"receive":not observer},
           "awareness":{"mode":"perceptive","vision_range":7 if observer else 5,"fov_degrees":360 if observer else 120,
                        "initial_facing_degrees":90,"memory_ticks":240,"search_ticks":120,"search_turn_degrees_per_second":90}}
    if observer:
        npc["objectives"]=[{"priority":1,"when":"always","action":{"kind":"wait"}}]
        npc["recognition"]={"acquire_ticks":12,"decay_ticks":1}
    return {"id":identifier,"name":identifier,"body":"npc","faction":"guards","color":color,
            "transform":{"position":position,"size":[0.6,1.6,0.6]},"npc":npc}


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-radio-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert "communication" in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":22,"depth":20,"seed":42,"obstacles":0})
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[-4,0.8,-3];player.pop("footsteps",None)
        wall={"id":"screen","name":"Sight screen","body":"fixed","color":[0.25,0.65,0.5],
              "transform":{"position":[0,1.5,0],"size":[0.6,3,8]}}
        sender=guard("sender",[-4,0.8,0],[0.75,0.2,0.85],1,True)
        receiver=guard("receiver",[3,0.8,0],[0.15,0.55,1],1)
        other=guard("other_channel",[6,0.8,1],[0.6,0.6,0.6],2)
        c.call("edit_scene",edits=[{"op":"upsert","entity":e} for e in [player,wall,sender,receiver,other]])
        c.call("set_camera",camera={"position":[15,16,20],"target":[0,0.5,0],"fov_degrees":50})
        c.call("set_debug_view",npc_vision=True,npc_routes=True,npc_hearing=False,noise_events=False)
        c.call("select_entity",id="receiver")
        c.call("save_scene",path="examples/npc-radio.scene.json")
        c.call("step",ticks=12)
        assert c.call("inspect_npcs")["npcs"]["receiver"]["memory"] is None
        c.call("step",ticks=1)
        prefix=c.call("inspect")
        r=prefix["npcs"]["receiver"]
        assert prefix["scene"]["version"]==SCENE_VERSION
        assert r["memory"]["source"]=="report" and r["memory"]["tick"]==11,r
        assert not r["observed_this_tick"] and not r["heard_this_tick"]
        receipt=r["communication"]["last_received"]
        assert receipt["sender_id"]=="sender" and receipt["target_id"]=="player" and receipt["received_tick"]==12,receipt
        assert prefix["npcs"]["other_channel"]["memory"] is None
        c.call("capture",path="artifacts/radio.png",width=1280,height=900)
        c.call("step",ticks=180)
        final=c.call("inspect")
        assert sum((a-b)**2 for a,b in zip(final["runtime_positions"]["receiver"],prefix["runtime_positions"]["receiver"]))>1,final["navigation"]
        assert final["npcs"]["other_channel"]["memory"] is None
        c.call("save_replay",path="artifacts/radio.replay.json")
        assert c.call("run_replay",path="artifacts/radio.replay.json")["matched"]
        replay=c.call("inspect_replay",path="artifacts/radio.replay.json",tick=13)
        assert replay["snapshot"]["npcs"]==prefix["npcs"]
        c.call("capture_replay",replay_path="artifacts/radio.replay.json",path="artifacts/radio-prefix.png",tick=13,width=1280,height=900)
        assert (ROOT/"artifacts/radio.png").read_bytes()==(ROOT/"artifacts/radio-prefix.png").read_bytes()
        checks.append("Confirmed sighting reaches allied radio after one tick; receiver investigates without seeing; other channel stays unaware; replay/pixels match")

        before=c.call("inspect")
        invalid=copy.deepcopy(receiver);invalid["npc"]["communication"]["range"]=0
        c.call("upsert",entity=invalid,expected_error=True);assert c.call("inspect")==before
        disabled=copy.deepcopy(receiver);disabled["npc"]["communication"]["receive"]=False
        c.call("upsert",entity=disabled);c.call("step",ticks=13)
        assert c.call("inspect_npcs")["npcs"]["receiver"]["memory"] is None
        c.call("undo");c.call("step",ticks=13)
        assert c.call("inspect_npcs")["npcs"]==prefix["npcs"]
        c.call("redo");c.call("step",ticks=13)
        assert c.call("inspect_npcs")["npcs"]["receiver"]["memory"] is None
        c.call("load_scene",path="examples/npc-radio.scene.json")
        c.call("remove",id="sender");c.call("step",ticks=30)
        assert c.call("inspect_npcs")["npcs"]["receiver"]["communication"]["last_received"] is None
        c.call("undo");c.call("step",ticks=13)
        assert c.call("inspect_npcs")["npcs"]==prefix["npcs"]
        checks.append("MCP validation is atomic; disable/undo/redo/remove/load reset compiled radio state and preserve provenance")

        c.call("load_scene",path="examples/npc-radio.scene.json")
        faction=copy.deepcopy(receiver);faction["faction"]="outsiders"
        c.call("upsert",entity=faction);c.call("step",ticks=13)
        assert c.call("inspect_npcs")["npcs"]["receiver"]["memory"] is None
        c.call("set_faction_relations",relations=[{"from":"guards","to":"outsiders","attitude":"allied"}]);c.call("step",ticks=13)
        assert c.call("inspect_npcs")["npcs"]["receiver"]["memory"] is None
        c.call("set_faction_relations",relations=[{"from":a,"to":b,"attitude":"allied"} for a,b in [("guards","outsiders"),("outsiders","guards")]])
        c.call("step",ticks=13)
        assert c.call("inspect_npcs")["npcs"]["receiver"]["memory"]["source"]=="report"
        c.call("load_scene",path="examples/npc-radio.scene.json")
        checks.append("Directional alliances must be mutual; channel/faction policy is independent of hearing and line of sight")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":final["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/radio-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":final["build_id"]}))
    finally:c.close()


if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
