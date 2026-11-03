#!/usr/bin/env python3
"""External interruption, protection and shared NPC executor through headless MCP."""
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


def interruption(name, projectile=False):
    d=definition(name,"actor","heal",1)
    d["effects"]=[{"kind":"interrupt","recipient":"target","require_active":True}]
    if projectile:d["delivery"]={"kind":"projectile","speed":6,"half_extent":0.1,"max_distance":20}
    return d


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-interrupt-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"));assert "interruptible" in schema and "require_active" in schema
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        long=definition("long","self_actor","heal",5,120,120);long["range"]=0
        cut=interruption("cut");shot=interruption("shot",True)
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[long,cut,shot]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[0,0.8,-4];player["vitality"]=vital(50);player["abilities"]=loadout(["cut","shot","long"]);player["footsteps"]=None
        target=actor("dummy",[0,0.8,0],[0.75,0.2,0.85]);target["vitality"]=vital(50);target["npc"]["objectives"]=[wait()];target["abilities"]=loadout(["long"])
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,target]])
        c.call("set_camera",camera={"position":[10,12,15],"target":[0,0.8,0],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        def state():return c.call("inspect_abilities")
        def active(id="dummy"):return state()["actors"][id]["cast"]
        def energy(id="player"):return state()["actors"][id]["resources"][0]["units"]
        def cast(who="player",slot=1,target="dummy"):
            args={"actor":who,"slot":slot}
            if target is not None:args["target"]=target
            c.call("activate_ability",**args)
        cast("dummy",target=None);c.call("step",ticks=20);victim=active()["execution_id"]
        prefix=c.call("inspect");c.call("capture",path="artifacts/interrupt-before.png",width=1280,height=900)
        c.call("step",ticks=1);cast();c.call("step",ticks=1)
        assert active() is None and energy()==995 and energy("dummy")==1000
        receipt=next(r for r in state()["receipts"] if r["outcome"]=="interrupted_by_ability")
        assert receipt["execution_id"]==victim and receipt["interrupted_by"]["actor"]=="player"
        c.call("capture",path="artifacts/interrupt-after.png",width=1280,height=900)
        c.call("capture",path="artifacts/interrupt-after-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/interrupt-after.png").read_bytes()==(ROOT/"artifacts/interrupt-after-repeat.png").read_bytes()
        assert (ROOT/"artifacts/interrupt-before.png").read_bytes()!=(ROOT/"artifacts/interrupt-after.png").read_bytes()
        c.call("save_replay",path="artifacts/interrupt.replay.json");assert c.call("run_replay",path="artifacts/interrupt.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/interrupt.replay.json",tick=20)["snapshot"]["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/interrupt.replay.json",path="artifacts/interrupt-prefix.png",tick=20,width=1280,height=900)
        assert (ROOT/"artifacts/interrupt-before.png").read_bytes()==(ROOT/"artifacts/interrupt-prefix.png").read_bytes()
        checks.append("MCP schema, active execution ID and reciprocal provenance; bar removal, repeated GPU pixels, full/prefix replay and exact prefix PNG")
        protected=copy.deepcopy(catalog);protected["abilities"][0]["interruptible"]=False
        c.call("set_ability_catalog",catalog=protected);cast("dummy",target=None);c.call("step",ticks=1);cast();c.call("step",ticks=1)
        assert active()["interruptible"] is False and energy()==1000 and state()["receipts"][-1]["outcome"]=="no_interruptible_execution"
        before=c.call("inspect")
        for edit in [dict(recipient="source"),dict(require_active="yes")]:
            bad=copy.deepcopy(protected);bad["abilities"][1]["effects"][0].update(edit);c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        bad=copy.deepcopy(protected);bad["abilities"][1]["channel"]=channel();c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        c.call("set_ability_catalog",catalog=catalog);c.call("undo");assert state()["actors"]["dummy"]["grants"][0]["interruptible"] is False
        c.call("redo");assert state()["actors"]["dummy"]["grants"][0]["interruptible"] is True
        checks.append("Protection rejects without payment, inspection/defaults, invalid composition preserves runtime/history, undo/redo")
        cast("dummy",target=None);c.call("step",ticks=1);cast(slot=2);c.call("step",ticks=1);execution=active()["execution_id"]
        c.call("cancel_ability",actor="dummy",execution_id=execution);c.call("step",ticks=50)
        assert energy()==995 and state()["receipts"][-1]["outcome"]=="no_interruptible_execution" and active() is None
        c.call("save_replay",path="artifacts/interrupt-impact.replay.json");assert c.call("run_replay",path="artifacts/interrupt-impact.replay.json")["matched"]
        checks.append("Projectile charges launch and revalidates current execution at impact without refund; replay")
        # Playable scene: casts to interrupt, protected comparison and a stationary NPC interrupter.
        own=definition("own_channel","self_actor","heal",2,30,60);own.update(range=0,channel=channel("on_start",30,10))
        shield=copy.deepcopy(own);shield.update(id="protected_channel",interruptible=False)
        redcut=interruption("enemy_interrupt");redcut.update(range=4,cooldown_ticks=120)
        cut["cooldown_ticks"]=30;shot["cooldown_ticks"]=30
        catalog["abilities"]=[long,cut,shot,own,shield,redcut]
        player["abilities"]=loadout(["cut","shot","own_channel","protected_channel"])
        target["transform"]["position"]=[-4,0.8,0];target["npc"]["objectives"]=[objective(policy(margin=None)),wait()]
        blue=actor("protected",[3,0.8,3],[0.15,0.5,1]);blue["vitality"]=vital(50);blue["abilities"]=loadout(["protected_channel"]);blue["npc"]["objectives"]=[objective(policy(margin=None)),wait()]
        red=actor("interrupter",[5,0.8,-4],[0.9,0.18,0.16],"player");red["vitality"]=vital();red["abilities"]=loadout(["enemy_interrupt"]);red["npc"]["awareness"].update(vision_range=8,fov_degrees=360);red["npc"]["objectives"]=[objective(policy(margin=None)),wait()]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,target,blue,red]])
        c.call("save_scene",path="examples/npc-interrupt.scene.json");c.call("step",ticks=60)
        assert active() is not None and active("protected")["phase"]=="channeling"
        c.call("capture",path="artifacts/interrupt-demo.png",width=1280,height=900)
        cast();c.call("step",ticks=1);assert active() is None
        c.call("set_input",input={"x":1,"z":0});c.call("step",ticks=35);c.call("set_input",input={"x":0,"z":0})
        cast(slot=3,target=None);c.call("step",ticks=60);assert active("player") is None
        assert any(r["outcome"]=="interrupted_by_ability" and r["actor"]=="player" and r["interrupted_by"]["actor"]=="interrupter" for r in state()["receipts"])
        cast(slot=4,target=None);c.call("step",ticks=180);assert active("player")["interruptible"] is False
        c.call("save_replay",path="artifacts/interrupt-demo.replay.json");assert c.call("run_replay",path="artifacts/interrupt-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-interrupt.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION and active("player") is None
        checks.append("Demo authored/saved/reloaded via MCP, purple interrupted, blue protected, red interrupts player under sight/range and respects protected channel")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/interrupt-verification.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
