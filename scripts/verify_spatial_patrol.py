#!/usr/bin/env python3
"""Exercise spatial routines and author their demonstration through headless MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{},
                                "clientInfo":{"name":"ignis-spatial-patrol-verifier","version":"1"}})
        c.request("notifications/initialized", notification=True)
        schema = json.dumps(c.request("tools/list"))
        assert "points_3d" in schema and "ring_3d" in schema
        c.call("generate", recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        player = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"] = [-8,0.8,7]
        player.pop("footsteps", None)
        def patrol(points):
            return {"route":{"kind":"points_3d","points":points},"traversal":"loop",
                    "wait_ticks":30,"travel_ticks":1200,"retry_ticks":120,"arrival_distance":0.2}
        def npc(identifier, position, color, capabilities, points):
            return {"id":identifier,"name":identifier,"body":"npc","color":color,
                    "transform":{"position":position,"size":[0.6,1.6,0.6]},
                    "npc":{"surface":"floor","speed":4,"stop_distance":0.2,"enabled":True,
                           "mobility":capabilities,"patrol":patrol(points)}}
        flyer = npc("flyer",[0,4.5,-7.5],[0.1,0.6,1],{"walk":False,"fly_speed":3},[])
        flyer["npc"]["local_connections"] = {"walk":False,"fly":True,"max_distance":8}
        flyer["npc"]["patrol"]["route"] = {"kind":"ring_3d","center":[0,4.5,-5],"normal":[1,2,0],"radius":2.5,"points":6,"seed":0}
        flyer["npc"]["awareness"] = {"mode":"perceptive","vision_range":0.1,"fov_degrees":90,
            "initial_facing_degrees":0,"memory_ticks":30,"search_ticks":1,"search_turn_degrees_per_second":0,"hearing":{"range":12}}
        jumper = npc("jumper",[-6,0.8,0],[0.75,0.15,0.85],{"walk":False,"jump_speed":10},[[-2,1.8,0],[-6,0.8,0]])
        glider = npc("glider",[2,5.8,4],[0.15,0.9,0.35],{"walk":False,"glide":{"speed":4,"sink_speed":2}},[[8,0.8,4],[2,5.8,4]])
        walker = npc("walker",[-6,0.8,3],[0.7,0.7,0.7],{"walk":True},[[-2,1.8,0]])
        walker["npc"]["local_connections"] = {"walk":True,"fly":False,"max_distance":8}
        platform = {"id":"platform","name":"Jump landing","body":"fixed","color":[0.45,0.38,0.25],
                    "transform":{"position":[-1.5,0.5,0],"size":[3,1,3]}}
        network = {"nodes":[[-6,0,0],[-2,1,0],[2,5,4],[8,0,4]],
                   "links":[{"from":0,"to":1,"mode":"jump"},{"from":1,"to":0,"mode":"jump"},{"from":2,"to":3,"mode":"glide"}]}
        c.call("edit_scene", edits=[{"op":"set_spatial_network","network":network}]+
               [{"op":"upsert","entity":e} for e in [player,platform,flyer,jumper,glider,walker]]+
               [{"op":"set_camera","camera":{"position":[15,17,21],"target":[0,2,0],"fov_degrees":50}}])
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("select_entity",id="flyer")
        c.call("save_scene",path="examples/npc-spatial-patrol.scene.json")
        routes = c.call("inspect_patrols")["routes"]
        assert len(routes["flyer"])==6 and max(p[1] for p in routes["flyer"])>5
        c.call("step",ticks=60)
        prefix = c.call("inspect")
        c.call("capture",path="artifacts/spatial-patrol.png",width=1280,height=900)
        c.call("step",ticks=1740)
        end = c.call("inspect")
        for identifier, minimum in [("flyer",6),("jumper",4)]:
            assert end["patrols"][identifier]["visited"]>=minimum, end["patrols"]
            assert end["patrols"][identifier]["skipped"]==0, end["patrols"]
        assert end["patrols"]["glider"]["visited"]>=1 and end["patrols"]["glider"]["skipped"]>0
        assert end["patrols"]["walker"]["visited"]==0 and end["patrols"]["walker"]["skipped"]>0
        assert end["runtime_positions"]["glider"][1]<1
        c.call("save_replay",path="artifacts/spatial-patrol.replay.json")
        assert c.call("run_replay",path="artifacts/spatial-patrol.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/spatial-patrol.replay.json",tick=60)["snapshot"]["patrols"]==prefix["patrols"]
        c.call("capture_replay",replay_path="artifacts/spatial-patrol.replay.json",path="artifacts/spatial-patrol-prefix.png",tick=60,width=1280,height=900)
        assert (ROOT/"artifacts/spatial-patrol.png").read_bytes()==(ROOT/"artifacts/spatial-patrol-prefix.png").read_bytes()
        checks.append("Spatial ring, repeated jumps, descending glide and incapable walker; full/prefix replay and exact GPU image")

        before = c.call("inspect")
        invalid = copy.deepcopy(flyer); invalid["npc"]["patrol"]["route"]["normal"]=[0,0,0]
        c.call("upsert",entity=invalid,expected_error=True)
        assert c.call("inspect")==before
        changed = copy.deepcopy(flyer); changed["npc"]["patrol"]["route"]["seed"]=40
        c.call("upsert",entity=changed)
        changed_routes = c.call("inspect_patrols")["routes"]
        assert changed_routes!=routes
        c.call("undo"); assert c.call("inspect_patrols")["routes"]==routes
        c.call("redo"); assert c.call("inspect_patrols")["routes"]==changed_routes
        c.call("load_scene",path="examples/npc-spatial-patrol.scene.json")
        assert c.call("inspect_patrols")["routes"]==routes
        checks.append("Bounded procedural authoring exposed in MCP schema; atomic validation, seed, undo/redo and persistence")

        c.call("step",ticks=36)
        before = c.call("inspect")
        p = before["runtime_positions"]["flyer"]
        c.call("emit_noise",origin={"kind":"position","position":[p[0]+1,p[1],p[2]]},radius=12)
        c.call("step",ticks=1)
        interrupted = c.call("inspect")
        assert interrupted["patrols"]["flyer"]["phase"]=="suspended"
        assert interrupted["mobility"]["flyer"]["committed"]
        c.call("step",ticks=40)
        resumed = c.call("inspect")
        assert resumed["patrols"]["flyer"]["phase"]=="moving"
        assert resumed["patrols"]["flyer"]["point_index"]==before["patrols"]["flyer"]["point_index"]
        c.call("save_replay",path="artifacts/spatial-patrol-interruption.replay.json")
        assert c.call("run_replay",path="artifacts/spatial-patrol-interruption.replay.json")["matched"]
        checks.append("Anonymous sound suspends route without cancelling committed flight; same point resumes after memory expires")

        c.call("load_scene",path="examples/npc-spatial-patrol.scene.json")
        c.call("step",ticks=60)
        before = c.call("inspect")
        c.call("set_debug_view",npc_routes=False)
        c.call("capture",path="artifacts/spatial-patrol-no-guides.png",width=1280,height=900)
        after = c.call("inspect")
        assert before["patrols"]==after["patrols"] and before["runtime_positions"]==after["runtime_positions"]
        assert (ROOT/"artifacts/spatial-patrol.png").read_bytes()!=(ROOT/"artifacts/spatial-patrol-no-guides.png").read_bytes()
        c.call("reset")
        assert all(p["visited"]==0 for p in c.call("inspect_patrols")["patrols"].values())
        checks.append("XYZ debug routes are observational; reset clears routine counters")
        result={"passed":True,"native_window":window,"checks":checks,"build_id":before["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/spatial-patrol-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":result["build_id"]}))
    finally:
        c.close()


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis")
    parser.add_argument("--window",action="store_true")
    args=parser.parse_args();verify(args.binary.resolve(),args.window)
