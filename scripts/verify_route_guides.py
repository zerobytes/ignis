#!/usr/bin/env python3
"""Bounded navigation and authored-patrol overlays through MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{},
                                 "clientInfo":{"name":"ignis-route-verifier","version":"1"}})
        c.request("notifications/initialized", notification=True)
        assert "npc_routes" in json.dumps(c.request("tools/list"))
        c.call("load_scene", path="examples/patrol.scene.json")
        def capture(name="routes-cache", width=640, height=450):
            path = f"artifacts/{name}.png"
            meta = c.call("capture",path=path,width=width,height=height)
            return (ROOT/path).read_bytes(),meta
        off,meta=capture()
        assert meta["route_guide_count"]==0
        before=c.call("inspect")
        c.call("save_replay",path="artifacts/routes.replay.json")
        replay=(ROOT/"artifacts/routes.replay.json").read_bytes()
        changed=c.call("set_debug_view",npc_routes=True)
        assert c.call("set_debug_view",npc_routes=True)==changed
        on,meta=capture("routes-authored")
        assert on!=off and capture()[0]==on
        assert meta["route_guide_count"]==12 and meta["truncated_navigation_paths"]==0
        after=c.call("inspect")
        assert all(after[k]==v for k,v in before.items() if k not in ("debug_view","view_revision"))
        c.call("save_replay",path="artifacts/routes.replay.json")
        assert (ROOT/"artifacts/routes.replay.json").read_bytes()==replay
        c.call("set_debug_view",npc_routes=False)
        assert capture()[0]==off
        c.call("set_debug_view",npc_routes=True)
        assert capture()[0]==on
        checks.append("paused patch/idempotence, authored geometry and identical simulation/replay")

        c.call("step",ticks=30)
        snap=c.call("inspect")
        planned,meta=capture("routes-planned",1280,900)
        assert meta["route_guide_count"]>12
        assert snap["navigation"]["hunter"]["target_position"]==c.call("inspect_patrols")["routes"]["hunter"][0]
        c.call("emit_noise",origin={"kind":"position","position":[0,0.8,4]},radius=12)
        c.call("step",ticks=1)
        snap=c.call("inspect")
        assert snap["patrols"]["hunter"]["phase"]=="suspended"
        assert snap["navigation"]["hunter"]["target_position"]==snap["npcs"]["hunter"]["memory"]["position"]
        c.call("set_debug_view",npc_vision=True,npc_hearing=True,noise_events=True)
        combined,meta=capture("routes-investigation",1280,900)
        assert combined!=planned and meta["route_guide_count"]>12
        assert meta["vision_guide_count"]==1 and meta["hearing_guide_count"]==1 and meta["noise_guide_count"]==1
        c.call("save_replay",path="artifacts/routes.replay.json")
        assert c.call("run_replay",path="artifacts/routes.replay.json")["matched"]
        checks.append("planned path/known destination, sound interruption and combined guides without replay changes")

        hunter=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="hunter")
        c.call("set_debug_view",npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("reset")
        assert capture()[0]==on
        c.call("remove",id="hunter")
        assert capture()[1]["route_guide_count"]==0
        c.call("undo")
        assert capture()[0]==on
        disabled=copy.deepcopy(hunter)
        disabled["npc"]["enabled"]=False
        c.call("upsert",entity=disabled)
        assert capture()[1]["route_guide_count"]==0
        c.call("undo")
        assert capture()[0]==on
        resized,_=capture(width=900,height=640)
        assert resized!=on and capture()[0]==on
        checks.append("reset/remove/undo/disable and resize clear or restore exact route pixels")

        blocked=copy.deepcopy(hunter)
        blocked["npc"]["patrol"]["route"]={"kind":"points","points":[[0,0]]}
        c.call("upsert",entity=blocked)
        c.call("step",ticks=1)
        assert c.call("inspect")["navigation"]["hunter"]["status"]=="unreachable"
        _,meta=capture("routes-unreachable",1280,900)
        assert meta["route_guide_count"]==4
        c.call("load_scene",path="examples/patrol.scene.json")
        assert c.call("inspect")["debug_view"]["npc_routes"]
        assert capture()[0]==on
        checks.append("unreachable destination marker and load preserve preference without stale geometry")
        report={"passed":True,"checks":checks,"native_window":window,"slow_requests":c.slow_requests,
                "build_id":c.call("inspect")["build_id"]}
        (ROOT/"artifacts/routes-verification.json").write_text(json.dumps(report,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"native_window":window}))
    finally:
        c.close()

if __name__ == "__main__":
    p=argparse.ArgumentParser()
    p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis")
    p.add_argument("--window",action="store_true")
    args=p.parse_args()
    verify(args.binary.resolve(),args.window)
