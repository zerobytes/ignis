#!/usr/bin/env python3
"""Bounded periodic effects, authoring, inspection and replay through MCP only."""
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


def schedule(interval=60, pulses=6, reapply="refresh"):
    return {"interval_ticks":interval,"pulses":pulses,"reapply":reapply}


def verify(binary, window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-periodic-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"));assert "periodic" in schema and "interval_ticks" in schema
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        poison=definition("poison","actor","damage",5);poison["periodic"]=schedule(10,3)
        regen=definition("regeneration","self_actor","heal",5);regen["range"]=0;regen["periodic"]=schedule(10,3,"independent")
        bolt=copy.deepcopy(poison);bolt["id"]="venom_bolt";bolt["delivery"]=delivery(6,0.14,18)
        catalog={"resources":[{"id":"energy","scale":1}],"abilities":[poison,regen,bolt]}
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[-3,0.8,0];player["vitality"]=vital(60);player["abilities"]=loadout(["poison","regeneration","venom_bolt"]);player["footsteps"]=None
        dummy=actor("dummy",[3,0.8,0],[0.75,0.2,0.85]);dummy["vitality"]=vital();dummy["npc"]["objectives"]=[wait()]
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,dummy]])
        c.call("set_camera",camera={"position":[10,12,15],"target":[0,0.8,0],"fov_degrees":48})
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        def hp(who):return c.call("inspect_vitality")["actors"][who]["units"]
        def state():return c.call("inspect_abilities")
        def cast(slot,target=None):
            args={"actor":"player","slot":slot}
            if target:args["target"]=target
            c.call("activate_ability",**args);c.call("step",ticks=1)
        cast(1,"dummy");prefix=c.call("inspect")
        assert hp("dummy")==100 and state()["actors"]["player"]["resources"][0]["units"]==995
        a=state()["periodic"]["active"][0];assert a["next_tick"]==10 and a["last_tick"]==30 and a["remaining_pulses"]==3
        c.call("capture",path="artifacts/periodic-active.png",width=1280,height=900)
        c.call("capture",path="artifacts/periodic-repeat.png",width=1280,height=900)
        assert (ROOT/"artifacts/periodic-active.png").read_bytes()==(ROOT/"artifacts/periodic-repeat.png").read_bytes()
        before=state();c.call("set_debug_view",npc_routes=False);assert state()==before
        c.call("capture",path="artifacts/periodic-no-guide.png",width=1280,height=900)
        assert (ROOT/"artifacts/periodic-active.png").read_bytes()!=(ROOT/"artifacts/periodic-no-guide.png").read_bytes()
        c.call("set_debug_view",npc_routes=True);c.call("step",ticks=9);assert hp("dummy")==100
        c.call("step",ticks=1);assert hp("dummy")==95
        c.call("step",ticks=20);assert hp("dummy")==85 and not state()["periodic"]["active"]
        pulse=[r for r in state()["receipts"] if r["outcome"]=="periodic_pulse"]
        assert [r["tick"] for r in pulse]==[10,20,30] and all(not r["costs"] for r in pulse)
        c.call("save_replay",path="artifacts/periodic.replay.json");assert c.call("run_replay",path="artifacts/periodic.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/periodic.replay.json",tick=1)["snapshot"]["abilities"]==prefix["abilities"]
        c.call("capture_replay",replay_path="artifacts/periodic.replay.json",path="artifacts/periodic-prefix.png",tick=1,width=1280,height=900)
        assert (ROOT/"artifacts/periodic-active.png").read_bytes()==(ROOT/"artifacts/periodic-prefix.png").read_bytes()
        checks.append("Delayed first/last pulse, one charge, lifecycle inspection, guide is presentation-only, repeated and prefix pixels/replay match")
        c.call("reset");cast(1,"dummy");c.call("step",ticks=4);cast(1,"dummy")
        assert len(state()["periodic"]["active"])==1 and state()["periodic"]["active"][0]["next_tick"]==15
        assert any(r["outcome"]=="refreshed" and r["replaced_by"] for r in state()["periodic"]["receipts"])
        cast(2);cast(2);assert len(state()["periodic"]["active"])==3
        c.call("step",ticks=30);assert hp("dummy")==85 and hp("player")==90
        assert state()["actors"]["player"]["resources"][0]["units"]==980
        c.call("save_replay",path="artifacts/periodic-refresh.replay.json");assert c.call("run_replay",path="artifacts/periodic-refresh.replay.json")["matched"]
        checks.append("Refresh restarts interval/count and identifies replacement; independent regeneration stacks, no per-pulse cost, replay")
        c.call("reset");cast(3,"dummy");assert not state()["periodic"]["active"]
        for _ in range(80):
            c.call("step",ticks=1)
            if state()["periodic"]["active"]:break
        active=state()["periodic"]["active"];assert len(active)==1 and hp("dummy")==100
        assert active[0]["next_tick"]==active[0]["applied_tick"]+10
        c.call("step",ticks=30);assert hp("dummy")==85 and state()["actors"]["player"]["resources"][0]["units"]==995
        # Solid transparent cover lets launch succeed but physically blocks its delivery.
        cover={"id":"cover","name":"Cover","body":"fixed","blocks_vision":False,"color":[0.2,0.6,0.5],"transform":{"position":[0,1.5,0],"size":[0.2,3,2]}}
        c.call("upsert",entity=cover);cast(3,"dummy");c.call("step",ticks=80)
        assert not state()["periodic"]["active"] and hp("dummy")==100
        assert state()["projectiles"]["receipts"][-1]["outcome"]=="blocked"
        checks.append("Projectile installs only on actual contact with delayed pulses; physical cover prevents installation, launch cost remains")
        before=c.call("inspect");bad=copy.deepcopy(catalog);bad["abilities"][0]["periodic"]["interval_ticks"]=0
        c.call("set_ability_catalog",catalog=bad,expected_error=True);assert c.call("inspect")==before
        c.call("remove",id="cover");cast(1,"dummy");assert state()["periodic"]["active"]
        changed=copy.deepcopy(dummy);changed["color"]=[0.4,0.2,0.9];c.call("upsert",entity=changed);assert not state()["periodic"]["active"]
        c.call("undo");c.call("redo");assert not state()["periodic"]["active"]
        cast(1,"dummy");c.call("remove",id="dummy");assert not state()["periodic"]["active"]
        c.call("undo");assert hp("dummy")==100
        checks.append("Invalid authoring preserves full state/history; edit/remove/undo/redo rebuild schedules atomically")
        # Final demonstration, authored entirely through the shared MCP dispatcher.
        poison.update(cast_ticks=30,cooldown_ticks=30);poison["periodic"]=schedule()
        regen.update(cast_ticks=30,cooldown_ticks=30);regen["periodic"]=schedule(reapply="independent")
        bolt.update(cast_ticks=30,cooldown_ticks=30);bolt["periodic"]=schedule()
        enemy=copy.deepcopy(bolt);enemy.update(id="enemy_venom",cast_ticks=60,cooldown_ticks=480);enemy["effects"][0]["amount"]=4;enemy["periodic"]=schedule(60,4);enemy["movement"]={"mode":"lock"}
        catalog["abilities"]=[poison,regen,bolt,enemy]
        player["transform"]["position"]=[0,0.8,-4];dummy["transform"]["position"]=[-4,0.8,1]
        shooter=actor("shooter",[4,0.8,5],[0.9,0.18,0.16],"player");shooter["vitality"]=vital();shooter["abilities"]=loadout(["enemy_venom"])
        shooter["npc"]["awareness"].update(vision_range=18,fov_degrees=360);shooter["npc"]["objectives"]=[objective(policy(margin=None)),wait()]
        cover["blocks_vision"]=True;cover["transform"]={"position":[0,1.5,0],"size":[2,3,0.4]}
        c.call("edit_scene",edits=[{"op":"set_ability_catalog","catalog":catalog}]+[{"op":"upsert","entity":e} for e in [player,dummy,shooter,cover]])
        c.call("save_scene",path="examples/npc-periodic.scene.json")
        cast(1,"dummy");c.call("step",ticks=31);assert any(x["target"]=="dummy" for x in state()["periodic"]["active"])
        # Cast60 + ~90 travel ticks + first interval60; inspect after tick240.
        c.call("step",ticks=210);assert hp("dummy")<100 and hp("player")<60, {"health":c.call("inspect_vitality"),"abilities":state()}
        assert any(x["actor"]=="shooter" and x["target"]=="player" for x in state()["periodic"]["active"])
        c.call("capture",path="artifacts/periodic-demo.png",width=1280,height=900)
        c.call("save_replay",path="artifacts/periodic-demo.replay.json");assert c.call("run_replay",path="artifacts/periodic-demo.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-periodic.scene.json");assert c.call("inspect")["scene"]["version"]==SCENE_VERSION and not state()["periodic"]["active"]
        checks.append("Demo authored/saved/reloaded via MCP, player poison and NPC venom share executor, live periodic state and replay")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":c.call("inspect")["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/periodic-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
