#!/usr/bin/env python3
"""Author modular walkways and verify continuous body support through MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def lane(identifier, z, color, gap=0, bridge_depth=2):
    """Parametric modular geometry; generated surfaces remain individually editable."""
    tiles=[]
    for part,x,width,depth in [("left",-4,4,2),("bridge",0,4-gap,bridge_depth),("right",4,4,2)]:
        tiles.append({"id":f"{identifier}_{part}","name":f"{identifier} {part}","body":"fixed",
                      "color":[c*0.55+0.2 for c in color],
                      "transform":{"position":[x,1,z],"size":[width,2,depth]}})
    npc={"id":identifier,"name":identifier,"body":"npc","color":color,
         "transform":{"position":[-4,2.8,z],"size":[0.6,1.6,0.6]},
         "npc":{"surface":"floor","speed":2,"stop_distance":0.2,"enabled":True,
                "mobility":{"walk":True},"local_connections":{"walk":True,"fly":False,"max_distance":8},
                "patrol":{"route":{"kind":"points_3d","points":
                    [[4,2.8,z]] if gap or bridge_depth<0.6 else [[4,2.8,z],[-4,2.8,z]]},
                          "traversal":"loop","wait_ticks":30,"travel_ticks":1200,"retry_ticks":120,"arrival_distance":0.2}}}
    return tiles,npc


def verify(binary,window):
    c=Client(binary,window);checks=[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"ignis-walk-support-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        c.call("generate",recipe={"width":24,"depth":20,"seed":42,"obstacles":0})
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[-8,0.8,7];player.pop("footsteps",None)
        edits=[{"op":"upsert","entity":player}];nodes=[];links=[]
        for identifier,z,color,gap,depth in [("connected",-5,[0.1,0.6,1],0,2),
                                            ("gap",0,[0.75,0.2,0.85],1,2),
                                            ("narrow",5,[0.65,0.65,0.65],0,0.4)]:
            tiles,npc=lane(identifier,z,color,gap,depth)
            edits.extend({"op":"upsert","entity":e} for e in [*tiles,npc])
            i=len(nodes);nodes.extend([[-4,2,z],[4,2,z]])
            links.extend([{"from":i,"to":i+1,"mode":"walk"},{"from":i+1,"to":i,"mode":"walk"}])
        edits.extend([{"op":"set_spatial_network","network":{"nodes":nodes,"links":links}},
                      {"op":"set_camera","camera":{"position":[14,17,21],"target":[0,1.5,0],"fov_degrees":50}}])
        c.call("edit_scene",edits=edits)
        c.call("set_debug_view",npc_routes=True,npc_vision=False,npc_hearing=False,noise_events=False)
        c.call("select_entity",id="connected")
        c.call("save_scene",path="examples/npc-walk-surfaces.scene.json")
        compiled=c.call("inspect_mobility")
        assert compiled["rejection_reasons"]["connected"][0] is None,compiled
        assert compiled["rejection_reasons"]["gap"][2]=="walk_support_gap"
        assert compiled["rejection_reasons"]["narrow"][4]=="walk_support_gap"
        c.call("step",ticks=120)
        prefix=c.call("inspect")
        assert prefix["runtime_positions"]["connected"][0]>-0.2,prefix["runtime_positions"]
        assert prefix["runtime_positions"]["connected"][1]>2.7
        for identifier in ["gap","narrow"]:
            assert prefix["runtime_positions"][identifier][0]<-3.9
            assert prefix["patrols"][identifier]["visited"]==0
            assert prefix["patrols"][identifier]["skipped"]>0
        c.call("capture",path="artifacts/walk-surfaces.png",width=1280,height=900)
        c.call("step",ticks=1080)
        final=c.call("inspect")
        assert final["patrols"]["connected"]["visited"]>=4 and final["patrols"]["connected"]["skipped"]==0,final["patrols"]
        c.call("save_replay",path="artifacts/walk-surfaces.replay.json")
        assert c.call("run_replay",path="artifacts/walk-surfaces.replay.json")["matched"]
        assert c.call("inspect_replay",path="artifacts/walk-surfaces.replay.json",tick=120)["snapshot"]["runtime_positions"]==prefix["runtime_positions"]
        c.call("capture_replay",replay_path="artifacts/walk-surfaces.replay.json",path="artifacts/walk-surfaces-prefix.png",tick=120,width=1280,height=900)
        assert (ROOT/"artifacts/walk-surfaces.png").read_bytes()==(ROOT/"artifacts/walk-surfaces-prefix.png").read_bytes()
        checks.append("Modular walking patrols cross seams physically; gaps/narrow bridges reject; replay/prefix and GPU pixels match")

        c.call("remove",id="connected_bridge");c.call("step",ticks=240)
        broken=c.call("inspect")
        assert broken["runtime_positions"]["connected"][0]<-3.9 and broken["patrols"]["connected"]["skipped"]>0
        assert c.call("inspect_mobility")["rejection_reasons"]["connected"][0]=="walk_support_gap"
        c.call("undo");c.call("step",ticks=300)
        assert c.call("inspect")["patrols"]["connected"]["visited"]>=1
        c.call("redo");c.call("step",ticks=1)
        assert c.call("inspect")["patrols"]["connected"]["visited"]==0
        c.call("load_scene",path="examples/npc-walk-surfaces.scene.json")
        checks.append("Removing a bridge invalidates support; undo restores it and redo/load rebuild deterministically")

        entity=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="connected")
        explicit=copy.deepcopy(entity);explicit["npc"].pop("local_connections")
        c.call("upsert",entity=explicit);c.call("step",ticks=300)
        assert c.call("inspect")["patrols"]["connected"]["visited"]>=1
        # A sound cue is a controlled test here; the manual demo intentionally isolates support.
        reactive=copy.deepcopy(entity)
        reactive["npc"]["awareness"]={"mode":"perceptive","vision_range":0.1,"fov_degrees":90,
            "initial_facing_degrees":0,"memory_ticks":30,"search_ticks":1,"search_turn_degrees_per_second":0,"hearing":{"range":12}}
        c.call("upsert",entity=reactive);c.call("step",ticks=60)
        c.call("emit_noise",origin={"kind":"position","position":[-3,2.8,-5]},radius=12)
        c.call("step",ticks=1)
        assert c.call("inspect")["patrols"]["connected"]["phase"]=="suspended"
        c.call("step",ticks=40)
        assert c.call("inspect")["patrols"]["connected"]["point_index"]==0
        c.call("step",ticks=300)
        assert c.call("inspect")["patrols"]["connected"]["visited"]>=1
        c.call("save_replay",path="artifacts/walk-surfaces-interruption.replay.json")
        assert c.call("run_replay",path="artifacts/walk-surfaces-interruption.replay.json")["matched"]
        c.call("load_scene",path="examples/npc-walk-surfaces.scene.json")
        checks.append("Authored links and local connectors share support; sound interruption resumes modular patrol and replays")
        result={"passed":True,"checks":checks,"native_window":window,"build_id":final["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/walk-surfaces-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":final["build_id"]}))
    finally:c.close()


if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis");p.add_argument("--window",action="store_true")
    a=p.parse_args();verify(a.binary.resolve(),a.window)
