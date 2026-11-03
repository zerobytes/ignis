#!/usr/bin/env python3
"""Atomic multi-entity authoring and one-step undo through MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    (ROOT/"artifacts").mkdir(exist_ok=True)
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},
                                "clientInfo":{"name":"ignis-scene-edit-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert "edit_scene" in json.dumps(c.request("tools/list"))
        c.call("load_scene",path="examples/patrol.scene.json")
        c.call("set_debug_view",npc_vision=True,npc_hearing=True,noise_events=True,npc_routes=True)
        def capture(path="artifacts/batch-current.png"):
            meta=c.call("capture",path=path,width=1280,height=900)
            return (ROOT/path).read_bytes(),meta
        original=c.call("inspect");original_pixels,_=capture()
        player=copy.deepcopy(next(e for e in original["scene"]["entities"] if e["id"]=="player"))
        player["id"]="hero";player["name"]="Hero"
        hunter=copy.deepcopy(next(e for e in original["scene"]["entities"] if e["id"]=="hunter"))
        hunter["npc"]["target"]="hero"
        zone={"id":"exit_zone","name":"Exit region","body":None,"color":[0.2,0.8,0.4],"visible":False,
              "transform":{"position":[4,1,-4],"size":[2,2,2]},"trigger":{"enabled":True,"targets":["hero"]}}
        marker={"id":"exit_marker","name":"Exit floor marker","body":None,"color":[0.2,0.8,0.4],
                "transform":{"position":[4,0.025,-4],"size":[2,0.05,2]},"blocks_vision":False}
        edits=[{"op":"remove","id":"player"},{"op":"upsert","entity":hunter},{"op":"upsert","entity":zone},
               {"op":"upsert","entity":marker},{"op":"upsert","entity":player}]
        c.call("step",ticks=20);c.call("emit_noise",origin={"kind":"position","position":[0,0.8,4]},radius=12)
        before=c.call("inspect");c.call("save_replay",path="artifacts/batch-live.replay.json")
        replay=(ROOT/"artifacts/batch-live.replay.json").read_bytes()
        for invalid in [edits[:-1],edits+[{"op":"remove","id":"absent"}],[],[{"op":"remove","id":"absent"}]*513]:
            c.call("edit_scene",edits=invalid,expected_error=True)
        c.call("edit_scene",edits=edits,expected_revision=999,expected_error=True)
        assert c.call("inspect")==before
        c.call("save_replay",path="artifacts/batch-live.replay.json")
        assert (ROOT/"artifacts/batch-live.replay.json").read_bytes()==replay
        checks.append("local/final/empty/oversized/stale batch errors preserve live inputs, cues, scene, history and replay")

        c.call("edit_scene",edits=edits,expected_revision=before["revision"])
        changed=c.call("inspect")
        assert changed["revision"]==before["revision"]+1 and changed["tick"]==0 and not changed["playing"]
        assert changed["recorded_ticks"]==changed["recorded_noises"]==0
        assert changed["history"]["undo"]==before["history"]["undo"]+1
        assert not any(e["id"]=="player" for e in changed["scene"]["entities"])
        assert c.call("inspect_triggers")["triggers"]["volumes"]["exit_zone"]["targets"]==["hero"]
        combined_pixels,_=capture("artifacts/batch-combined.png")
        c.call("undo");assert c.call("inspect")["scene"]==original["scene"] and capture()[0]==original_pixels
        c.call("redo");assert c.call("inspect")["scene"]==changed["scene"] and capture()[0]==combined_pixels
        c.call("save_scene",path="examples/stealth-lab.scene.json")
        c.call("load_scene",path="examples/stealth-lab.scene.json")
        assert capture()[0]==combined_pixels
        checks.append("forward references and rename commit once; single undo/redo restores exact scene/pixels; combined scene saved and loaded")

        c.call("set_input",input={"x":1,"z":0,"mode":"sneak"});c.call("step",ticks=90)
        moving=c.call("inspect")
        assert moving["runtime_positions"]["hero"][0]>changed["runtime_positions"]["hero"][0]+1
        assert moving["generated_footsteps"]>0
        c.call("save_replay",path="artifacts/batch.replay.json")
        assert c.call("run_replay",path="artifacts/batch.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/batch.replay.json")["replay"]["final_verification"]["matched"]
        c.call("reset");assert capture()[0]==combined_pixels
        checks.append("renamed player remains controllable with footsteps/patrol/triggers and exact replay; reset restores authoring presentation")
        result={"passed":True,"checks":checks,"native_window":window,"slow_requests":c.slow_requests,
                "build_id":c.call("inspect")["build_id"]}
        (ROOT/"artifacts/scene-edits-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks)}))
    finally:
        c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis")
    p.add_argument("--window",action="store_true");args=p.parse_args();verify(args.binary.resolve(),args.window)
