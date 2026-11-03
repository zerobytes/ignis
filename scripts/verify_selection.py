#!/usr/bin/env python3
"""Selection projection, presentation, lifecycle and replay through the real MCP process."""
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def project(camera, point, width, height):
    def sub(a,b): return [x-y for x,y in zip(a,b)]
    def dot(a,b): return sum(x*y for x,y in zip(a,b))
    def norm(a): return [x/math.sqrt(dot(a,a)) for x in a]
    def cross(a,b): return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
    forward=norm(sub(camera["target"],camera["position"]));right=norm(cross(forward,[0,1,0]));up=cross(right,forward)
    delta=sub(point,camera["position"]);z=dot(delta,forward);t=math.tan(math.radians(camera["fov_degrees"])/2)
    return (1+dot(delta,right)/(z*t*width/height))*width/2,(1-dot(delta,up)/(z*t))*height/2


def verify(binary, window):
    (ROOT/"artifacts").mkdir(exist_ok=True);c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-selection-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"));assert all(op in schema for op in ["select_entity","select_at","inspect_selection"])
        c.call("load_scene",path="examples/stealth-lab.scene.json")
        def capture(path="artifacts/selection-current.png",w=1280,h=900):
            meta=c.call("capture",path=path,width=w,height=h);return (ROOT/path).read_bytes(),meta
        original=c.call("inspect");plain,_=capture()
        c.call("save_replay",path="artifacts/selection.replay.json");replay=(ROOT/"artifacts/selection.replay.json").read_bytes()
        c.call("select_entity",id="hunter");selected=c.call("inspect_selection")["selection"]
        assert selected["id"]=="hunter" and selected["npc"] is not None and selected["patrol"] is not None
        pixels,meta=capture("artifacts/selection-npc.png")
        assert meta["selection"]=="hunter" and meta["selection_guide_count"]==1 and pixels!=plain
        assert capture()[0]==pixels
        stable=c.call("inspect");c.call("select_entity",id="hunter");assert c.call("inspect")==stable
        for op,args in [("select_entity",{"id":"missing"}),("select_at",{"x":1280,"y":0,"width":1280,"height":900}),("select_at",{"x":0,"y":0,"width":0,"height":900})]:
            c.call(op,**args,expected_error=True);assert c.call("inspect")==stable
        c.call("select_entity",id="hero",expected_revision=999,expected_error=True);assert c.call("inspect")==stable
        c.call("save_replay",path="artifacts/selection.replay.json");assert (ROOT/"artifacts/selection.replay.json").read_bytes()==replay
        c.call("select_entity",id=None);assert capture()[0]==plain
        checks.append("MCP schema, component inspection, idempotence/errors, exact contour removal and replay invariance")

        c.call("set_input",input={"x":1,"z":0});c.call("step",ticks=45);state=c.call("inspect")
        assert state["runtime_positions"]["hero"][0]>original["runtime_positions"]["hero"][0]+1
        for w,h in [(1280,900),(700,1000),(1600,500)]:
            x,y=project(state["scene"]["camera"],state["runtime_positions"]["hero"],w,h)
            hit=c.call("select_at",x=x,y=y,width=w,height=h);assert hit["selection"]=="hero" and hit["hit"]["distance"]>0
            assert capture(w=w,h=h)[1]["selection_guide_count"]==1
        assert c.call("inspect_selection")["selection"]["world_position"]==state["runtime_positions"]["hero"]
        c.call("select_at",x=0,y=0,width=1280,height=900);assert c.call("inspect_selection")["selection"] is None
        c.call("reset");c.call("select_entity",id="exit_zone")
        hidden,meta=capture("artifacts/selection-region.png");assert hidden!=plain and meta["entity_count"]==len(original["scene"]["entities"])-1
        assert c.call("inspect_selection")["selection"]["trigger"]["occupants"]==[]
        x,y=project(original["scene"]["camera"],[4,1,-4],1280,900)
        assert c.call("select_at",x=x,y=y,width=1280,height=900)["selection"]!="exit_zone"
        checks.append("current physics selection at three viewports; miss clears; hidden trigger explicit outline but excluded from picking")

        c.call("select_entity",id="hunter");c.call("reset");assert c.call("inspect")["selection"]=="hunter"
        saved=c.call("inspect");extra={"id":"aaa","name":"marker","body":None,"color":[1,0,0],"transform":{"position":[5,3,5],"size":[1,1,1]}}
        c.call("upsert",entity=extra);assert c.call("inspect_selection")["selection"]["id"]=="hunter"
        c.call("remove",id="aaa");assert c.call("inspect_selection")["selection"]["world_position"]==saved["runtime_positions"]["hunter"]
        hunter=copy.deepcopy(next(e for e in saved["scene"]["entities"] if e["id"]=="hunter"));hunter["id"]="guard"
        c.call("edit_scene",edits=[{"op":"remove","id":"hunter"},{"op":"upsert","entity":hunter}]);assert c.call("inspect")["selection"] is None
        c.call("undo");assert c.call("inspect")["selection"] is None
        c.call("select_entity",id="hunter");c.call("redo");assert c.call("inspect")["selection"] is None
        c.call("undo");c.call("select_entity",id="hunter");assert capture()[0]==pixels
        checks.append("reset/reindex/rename/remove/undo/redo selection lifecycle with exact restored pixels")

        c.call("load_scene",path="examples/stealth-lab.scene.json");c.call("set_input",input={"x":1,"z":0});c.call("step",ticks=30)
        c.call("save_replay",path="artifacts/selection.replay.json");c.call("select_entity",id="hero")
        live=c.call("inspect");live_pixels,_=capture()
        inspected=c.call("inspect_replay",path="artifacts/selection.replay.json");assert inspected["snapshot"]["selection"]=="hero" and inspected["replay"]["final_verification"]["matched"]
        meta=c.call("capture_replay",replay_path="artifacts/selection.replay.json",path="artifacts/selection-replay.png",width=1280,height=900)
        assert meta["selection"]=="hero" and (ROOT/"artifacts/selection-replay.png").read_bytes()==live_pixels
        assert c.call("inspect")==live
        c.call("upsert",entity=extra);c.call("select_entity",id="aaa");before=c.call("inspect")
        assert c.call("inspect_replay",path="artifacts/selection.replay.json")["snapshot"]["selection"] is None
        meta=c.call("capture_replay",replay_path="artifacts/selection.replay.json",path="artifacts/selection-replay.png",width=1280,height=900)
        assert meta["selection"] is None and meta["selection_guide_count"]==0 and c.call("inspect")==before
        checks.append("replay inherits existing selected ID and exact live pixels; absent ID omitted and live preserved")
        result={"passed":True,"checks":checks,"native_window":window,"native_click_tested":False,"slow_requests":c.slow_requests,"build_id":c.call("inspect")["build_id"]}
        (ROOT/"artifacts/selection-verification.json").write_text(json.dumps(result,indent=2)+"\n");print(json.dumps({"passed":True,"checks":len(checks)}))
    finally: c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    args=p.parse_args();verify(args.binary.resolve(),args.window)
