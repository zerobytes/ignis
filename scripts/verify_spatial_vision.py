#!/usr/bin/env python3
"""Author and verify volumetric NPC vision through MCP, without a window by default."""
from verify_lab import SCENE_VERSION
import argparse
import copy
import json
import math
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{},
                                "clientInfo":{"name":"ignis-spatial-vision-verifier","version":"1"}})
        c.request("notifications/initialized", notification=True)
        assert "spatial_vision" in json.dumps(c.request("tools/list"))
        c.call("generate", recipe={"width":18,"depth":16,"seed":42,"obstacles":0})
        player = next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"] = [0,4.8,0]
        player.pop("footsteps",None)
        c.call("upsert",entity=player)
        platform = {"id":"platform","name":"Elevated platform","body":"fixed","color":[0.4,0.5,0.5],
                    "transform":{"position":[0,2,0],"size":[2,4,1]}}
        c.call("upsert",entity=platform)
        for identifier,x,pitch,color in [("horizontal",-2,0,[0.2,0.6,1]),("upward",2,35,[0.7,0.2,0.8])]:
            c.call("upsert",entity={"id":identifier,"name":identifier,"body":"npc","color":color,
                "transform":{"position":[x,0.8,5],"size":[0.6,1.6,0.6]},
                "npc":{"target":"player","surface":"floor","speed":2,"stop_distance":0.8,"enabled":True,
                    "spatial_vision":{"initial_pitch_degrees":pitch,"follow_movement":False},
                    "awareness":{"mode":"perceptive","vision_range":9,"fov_degrees":45,
                        "initial_facing_degrees":math.degrees(math.atan2(-x,5)),
                        "memory_ticks":180,"search_ticks":60,"search_turn_degrees_per_second":0},
                    "objectives":[{"priority":10,"when":"always","action":{"kind":"wait"}}]}})
        c.call("set_camera",camera={"position":[12,12,17],"target":[0,3,1],"fov_degrees":50})
        c.call("set_debug_view",npc_vision=True,npc_hearing=False,noise_events=False,npc_routes=False)
        c.call("select_entity",id="upward")
        c.call("save_scene",path="examples/npc-spatial-vision.scene.json")
        c.call("step",ticks=1)
        seen = c.call("inspect")
        assert seen["scene"]["version"] == SCENE_VERSION
        assert not seen["npcs"]["horizontal"]["observed_this_tick"], seen["npcs"]
        assert seen["npcs"]["upward"]["observed_this_tick"], seen["npcs"]
        assert seen["npcs"]["upward"]["vision_direction"][1] > 0.5
        c.call("capture",path="artifacts/spatial-vision.png",width=1280,height=900)
        prefix = copy.deepcopy(seen["npcs"])
        c.call("step",ticks=59)
        c.call("save_replay",path="artifacts/spatial-vision.replay.json")
        assert c.call("run_replay",path="artifacts/spatial-vision.replay.json")["matched"]
        c.call("capture_replay",replay_path="artifacts/spatial-vision.replay.json",path="artifacts/spatial-vision-prefix.png",tick=1,width=1280,height=900)
        assert (ROOT/"artifacts/spatial-vision.png").read_bytes()==(ROOT/"artifacts/spatial-vision-prefix.png").read_bytes()
        checks.append("Elevation gates target acquisition; MCP direction and GPU replay prefix match")

        # Invalid edits must leave both scene and running state intact.
        entity=next(e for e in seen["scene"]["entities"] if e["id"]=="upward")
        before=c.call("inspect")
        invalid=copy.deepcopy(entity); invalid["npc"]["spatial_vision"]["initial_pitch_degrees"]=91
        c.call("upsert",entity=invalid,expected_error=True)
        assert c.call("inspect")==before
        horizontal=copy.deepcopy(entity); horizontal["npc"]["spatial_vision"]["initial_pitch_degrees"]=0
        c.call("upsert",entity=horizontal); c.call("step",ticks=1)
        assert not c.call("inspect")["npcs"]["upward"]["observed_this_tick"]
        c.call("undo"); c.call("step",ticks=1)
        assert c.call("inspect")["npcs"]==prefix
        c.call("redo"); c.call("step",ticks=1)
        assert not c.call("inspect")["npcs"]["upward"]["observed_this_tick"]
        c.call("load_scene",path="examples/npc-spatial-vision.scene.json")
        checks.append("Pitch validation is atomic; save/load and undo/redo restore sensor orientation")

        # A visual-only ceiling blocks both modes; a broad FOV is not omniscience.
        ceiling={"id":"ceiling","name":"Visual occluder","body":None,"blocks_vision":True,
                 "color":[0.4,0.4,0.4],"transform":{"position":[0,3,2.5],"size":[8,0.3,4]}}
        c.call("upsert",entity=ceiling); c.call("step",ticks=1)
        assert not c.call("inspect")["npcs"]["upward"]["observed_this_tick"]
        ceiling["blocks_vision"]=False;c.call("upsert",entity=ceiling);c.call("step",ticks=1)
        assert c.call("inspect")["npcs"]["upward"]["observed_this_tick"]
        c.call("load_scene",path="examples/npc-spatial-vision.scene.json")
        checks.append("Volumetric line of sight respects independent visual occluders")

        # A vertical gaze uses real geometry, not a degenerate floor arrow.
        vertical=copy.deepcopy(entity)
        vertical["npc"]["spatial_vision"]["initial_pitch_degrees"]=90
        c.call("upsert",entity=vertical)
        c.call("capture",path="artifacts/spatial-vision-vertical.png",width=1280,height=900)
        before=c.call("inspect")
        c.call("set_debug_view",npc_vision=False)
        c.call("capture",path="artifacts/spatial-vision-no-guides.png",width=1280,height=900)
        after=c.call("inspect")
        assert before["npcs"]==after["npcs"] and before["runtime_positions"]==after["runtime_positions"]
        assert (ROOT/"artifacts/spatial-vision-vertical.png").read_bytes()!=(ROOT/"artifacts/spatial-vision-no-guides.png").read_bytes()
        c.call("load_scene",path="examples/npc-spatial-vision.scene.json")
        c.call("set_debug_view",npc_vision=True)
        checks.append("Vertical guides render through MCP; debug controls do not mutate simulation")
        report={"passed":True,"native_window":window,"checks":checks,"build_id":seen["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/spatial-vision-verification.json").write_text(json.dumps(report,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":seen["build_id"]}))
    finally:
        c.close()


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis")
    parser.add_argument("--window",action="store_true")
    args=parser.parse_args();verify(args.binary.resolve(),args.window)
