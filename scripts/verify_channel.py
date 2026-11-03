#!/usr/bin/env python3
"""Sustained actions via MCP: costs, interruptions, replay, guides and NPC demo."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT
from verify_abilities import actor
from verify_vitality import vital, definition, loadout
from verify_ability_policy import objective, policy, wait
from verify_periodic import schedule


def channel(mode="on_start", interval=10, pulses=3):
    return {"interval_ticks":interval,"pulses":pulses,"cost_mode":mode}


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-channel-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"));assert "channel" in schema and "cost_mode" in schema and "per_pulse" in schema
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        beam=definition("beam","actor","damage",5,5,0);beam["channel"]=channel()
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[beam]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[0,0.8,-4];player["vitality"]=vital();player["abilities"]=loadout(["beam"]);player["footsteps"]=None
        target=actor("dummy",[-4,0.8,1],[0.75,0.2,0.85]);target["vitality"]=vital();target["npc"]["objectives"]=[wait()]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,target]])
        c.call("set_camera",camera={"position":[10,12,15],"target":[0,0.8,0],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        def state():return c.call("inspect_abilities")
        def hp(id):return c.call("inspect_vitality")["actors"][id]["units"]
        def energy(id="player"):return state()["actors"][id]["resources"][0]["units"]
        def cast(slot=1,target="dummy"):
            args={"actor":"player","slot":slot}
            if target is not None:args["target"]=target
            c.call("activate_ability",**args)
        cast();c.call("step",ticks=5);assert state()["actors"]["player"]["cast"]["phase"]=="preparing" and energy()==1000
        c.call("step",ticks=1);active=state()["actors"]["player"]["cast"]
        assert active["phase"]=="channeling" and active["channel"]=={"started":5,"next_tick":15,"finishes_at":35,"remaining":3}
        assert hp("dummy")==100 and energy()==995
        prefix=c.call("inspect");c.call("capture",path="artifacts/channel-active.png",width=1280,height=900)
        c.call("capture",path="artifacts/channel-active-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/channel-active.png").read_bytes()==(ROOT/"artifacts/channel-active-repeat.png").read_bytes()
        c.call("step",ticks=30);assert hp("dummy")==85 and energy()==995 and state()["actors"]["player"]["cast"] is None
        assert len([r for r in state()["receipts"] if r["outcome"]=="channel_pulse"])==3
        c.call("save_replay",path="artifacts/channel.replay.json");assert c.call("run_replay",path="artifacts/channel.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/channel.replay.json",tick=6)["snapshot"]["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/channel.replay.json",path="artifacts/channel-prefix.png",tick=6,width=1280,height=900)
        assert (ROOT/"artifacts/channel-active.png").read_bytes()==(ROOT/"artifacts/channel-prefix.png").read_bytes()
        checks.append("MCP schema, preparation/channel phases, delayed pulses/single charge, full/prefix replay and cyan guide repeated/replayed pixels")
        c.call("reset");cast();c.call("step",ticks=25);execution=state()["actors"]["player"]["cast"]["execution_id"]
        c.call("cancel_ability",actor="player",execution_id=execution);c.call("step",ticks=1)
        assert hp("dummy")==95 and energy()==995 and state()["actors"]["player"]["cast"] is None
        c.call("save_replay",path="artifacts/channel-cancel.replay.json");assert c.call("run_replay",path="artifacts/channel-cancel.replay.json")["matched"]
        per=copy.deepcopy(catalog);per["abilities"][0]["channel"]["cost_mode"]="per_pulse"
        poor=copy.deepcopy(player);poor["abilities"]["resources"][0]["initial"]=11
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":per},{"op":"upsert","entity":poor}]);cast();c.call("step",ticks=36)
        assert energy()==1 and hp("dummy")==90 and state()["actors"]["player"]["cast"] is None and state()["receipts"][-1]["outcome"]=="insufficient_resource"
        checks.append("Cancel before due pulse preserves earlier effects/payment; per-pulse costs stop atomically without remaining energy")
        before=c.call("inspect")
        for invalid in [{"interval_ticks":0,"pulses":1,"cost_mode":"on_start"},{"interval_ticks":1,"pulses":1,"cost_mode":"unknown"},{"interval_ticks":3600,"pulses":61,"cost_mode":"per_pulse"}]:
            bad=copy.deepcopy(per);bad["abilities"][0]["channel"]=invalid;c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        bad=copy.deepcopy(per);bad["abilities"][0]["periodic"]=schedule();c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        c.call("set_ability_catalog",catalog=catalog);c.call("undo");assert c.call("inspect")["scene"]["ability_catalog"]["abilities"][0]["channel"]["cost_mode"]=="per_pulse"
        c.call("redo");assert c.call("inspect")["scene"]["ability_catalog"]["abilities"][0]["channel"]["cost_mode"]=="on_start"
        c.call("reset");assert state()["actors"]["player"]["cast"] is None
        checks.append("Invalid timing/mode/composition preserves running state/history; catalog undo/redo/reset reconstruct channel")
        # Human demo uses the same definitions/executor and four existing slots.
        beam.update(cast_ticks=30,cooldown_ticks=60,channel=channel("per_pulse",30,6))
        own=definition("self_channel","self_actor","heal",5,30,60);own.update(range=0,channel=channel("on_start",30,6),movement={"mode":"interrupt","speed_tolerance":0.2});own["costs"][0]["units"]=15
        support=definition("support_channel","actor","heal",5,30,60);support.update(channel=channel("per_pulse",30,6),movement={"mode":"lock"})
        poison=definition("poison","actor","damage",3,30,60);poison.update(periodic=schedule(60,4),periodic_category="poison")
        enemy=definition("enemy_channel","actor","damage",3,30,240);enemy.update(range=8,channel=channel("on_start",45,4),movement={"mode":"lock"})
        healing=definition("npc_channel","self_actor","heal",5,30,240);healing.update(range=0,channel=channel("on_start",30,6),movement={"mode":"lock"});healing["costs"][0]["units"]=10
        catalog["abilities"]=[beam,own,support,poison,enemy,healing]
        player["abilities"]=loadout(["beam","self_channel","support_channel","poison"]);player["vitality"]=vital(70)
        blue=actor("healer",[0,0.8,3],[0.15,0.5,1]);blue["vitality"]=vital(50);blue["abilities"]=loadout(["npc_channel"]);blue["npc"]["objectives"]=[objective(policy(subject="source",below=10000,margin=None)),wait()]
        red=actor("shooter",[4,0.8,5],[0.9,0.18,0.16],"player");red["vitality"]=vital();red["abilities"]=loadout(["enemy_channel"]);red["npc"]["awareness"].update(vision_range=18,fov_degrees=360);red["npc"]["objectives"]=[objective(policy(margin=0.5)),wait()]
        cover={"id":"cover","name":"Cover","body":"fixed","color":[0.2,0.6,0.5],"transform":{"position":[3,1.5,0],"size":[1.6,3,0.4]}}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,target,blue,red,cover]])
        c.call("save_scene",path="examples/npc-channel.scene.json");cast();c.call("step",ticks=40)
        assert state()["actors"]["player"]["cast"]["phase"]=="channeling" and state()["actors"]["healer"]["cast"]["phase"]=="channeling"
        c.call("capture",path="artifacts/channel-demo.png",width=1280,height=900)
        c.call("step",ticks=180);assert hp("healer")==80 and energy("healer")==990
        c.call("save_replay",path="artifacts/channel-demo.replay.json");assert c.call("run_replay",path="artifacts/channel-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-channel.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION and state()["actors"]["player"]["cast"] is None
        checks.append("Demo authored/saved/reloaded through MCP, NPC/player channel with same executor, NPC single payment/pulses and replay")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/channel-verification.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
