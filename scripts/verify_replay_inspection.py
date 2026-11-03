#!/usr/bin/env python3
"""Reconstruct and render recorded ticks through the real MCP protocol."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    c = Client(binary, window)
    checks, samples = [], []
    try:
        c.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{},
                                 "clientInfo":{"name":"ignis-replay-inspector","version":"1"}})
        c.request("notifications/initialized", notification=True)
        schema = json.dumps(c.request("tools/list"))
        assert all(op in schema for op in ("inspect_replay", "capture_replay"))
        c.call("load_scene", path="examples/patrol.scene.json")
        c.call("set_debug_view", npc_vision=True, npc_hearing=True, noise_events=True, npc_routes=True)
        def cue(x):
            c.call("emit_noise",origin={"kind":"position","position":[x,0.8,4]},radius=12)
        def capture_live():
            meta=c.call("capture",path="artifacts/replay-live.png",width=640,height=450)
            return c.call("inspect"),(ROOT/"artifacts/replay-live.png").read_bytes(),meta
        recorded={}
        cue(0)
        recorded[0]=capture_live()
        c.call("step",ticks=30)
        cue(1)
        recorded[30]=capture_live()
        c.call("set_input",input={"x":1,"z":0,"mode":"run"})
        c.call("step",ticks=1)
        recorded[31]=capture_live()
        c.call("step",ticks=19)
        cue(2)
        recorded[50]=capture_live()
        path="artifacts/inspection.replay.json"
        c.call("save_replay",path=path)
        source=(ROOT/path).read_bytes()
        assert recorded[50][0]["generated_footsteps"]>0
        # Deliberately use another scene/camera in the live session.
        c.call("generate",recipe={"width":10,"depth":10,"seed":19,"obstacles":1})
        c.call("step",ticks=90)
        before,live_png,_=capture_live()
        c.call("save_replay",path="artifacts/inspection-live.replay.json")
        live_replay=(ROOT/"artifacts/inspection-live.replay.json").read_bytes()
        for tick,(expected,pixels,meta) in recorded.items():
            result=c.call("inspect_replay",path=path,tick=tick)
            actual=result["snapshot"]
            for key in ("tick","scene","runtime_positions","navigation","npcs","contacts","footsteps","patrols",
                        "generated_footsteps","recorded_ticks","recorded_noises","pending_noises"):
                assert actual[key]==expected[key],(tick,key)
            info=result["replay"]
            assert info["at_end"]==(tick==50)
            assert info["final_verification"]["matched"] if tick==50 else info["final_verification"] is None
            out="artifacts/replay-selected.png"
            rendered=c.call("capture_replay",replay_path=path,path=out,tick=tick,width=640,height=450)
            assert rendered["tick"]==tick and rendered["replay"]==info
            assert (ROOT/out).read_bytes()==pixels
            for key in ("camera","entity_count","vision_guide_count","hearing_guide_count","noise_guide_count","route_guide_count"):
                assert rendered[key]==meta[key],(tick,key)
            samples.append({"tick":tick,"png_sha256":hashlib.sha256(pixels).hexdigest(),"pending_noises":actual["pending_noises"]})
        assert c.call("inspect_replay",path=path)==c.call("inspect_replay",path=path,tick=50)
        checks.append("zero/intermediate/final state and PNG exactly match original live ticks, including pending noise and footsteps")

        # Alternating replay/live frames, viewport and preferences must invalidate shared GPU caches.
        assert capture_live()[1]==live_png
        c.call("capture_replay",replay_path=path,path="artifacts/replay-selected.png",tick=30,width=900,height=640)
        assert capture_live()[1]==live_png
        c.call("set_debug_view",npc_routes=False,noise_events=False,npc_hearing=False,npc_vision=False)
        plain=c.call("capture_replay",replay_path=path,path="artifacts/replay-selected.png",tick=50,width=640,height=450)
        assert plain["route_guide_count"]==plain["noise_guide_count"]==0
        assert (ROOT/"artifacts/replay-selected.png").read_bytes()!=recorded[50][1]
        c.call("set_debug_view",npc_routes=True,noise_events=True,npc_hearing=True,npc_vision=True)
        c.call("capture_replay",replay_path=path,path="artifacts/replay-selected.png",width=640,height=450)
        assert (ROOT/"artifacts/replay-selected.png").read_bytes()==recorded[50][1]
        current=c.call("inspect")
        assert all(current[k]==v for k,v in before.items() if k!="view_revision")
        c.call("save_replay",path="artifacts/inspection-live.replay.json")
        assert (ROOT/"artifacts/inspection-live.replay.json").read_bytes()==live_replay
        checks.append("recorded camera/scene, current guide preferences and shared capture cache survive resize/live alternation without gameplay mutation")

        broken=json.loads(source); broken["expected_positions"]["player"][0]+=5
        bad="artifacts/inspection-tampered.replay.json"
        (ROOT/bad).write_text(json.dumps(broken))
        assert c.call("inspect_replay",path=bad)["replay"]["final_verification"]["matched"] is False
        assert c.call("inspect_replay",path=bad,tick=30)["replay"]["final_verification"] is None
        captured=c.call("capture_replay",replay_path=bad,path="artifacts/replay-selected.png",width=640,height=450)
        assert captured["replay"]["final_verification"]["matched"] is False
        assert (ROOT/"artifacts/replay-selected.png").read_bytes()==recorded[50][1]
        checks.append("tampered expected endpoint is reported as mismatch while reconstructed actual image remains inspectable")

        before=c.call("inspect")
        for tick in (51,-1,2**64-1):
            c.call("inspect_replay",path=path,tick=tick,expected_error=True)
        c.call("capture_replay",replay_path=path,path=path,width=640,height=450,expected_error=True)
        c.call("capture_replay",replay_path=path,path="../escape.png",width=640,height=450,expected_error=True)
        c.call("capture_replay",replay_path=path,path="artifacts/replay-selected.png",width=2049,height=450,expected_error=True)
        c.call("inspect_replay",path=path,expected_revision=999,expected_error=True)
        invalid=copy.deepcopy(broken);invalid["inputs"][-1]["x"]=2
        (ROOT/bad).write_text(json.dumps(invalid))
        c.call("inspect_replay",path=bad,tick=0,expected_error=True)
        invalid=copy.deepcopy(broken);invalid["build_id"]="different"
        (ROOT/bad).write_text(json.dumps(invalid))
        c.call("capture_replay",replay_path=bad,path="artifacts/replay-selected.png",tick=0,width=640,height=450,expected_error=True)
        assert c.call("inspect")==before and (ROOT/path).read_bytes()==source
        checks.append("invalid tick/build/suffix/path/viewport/revision rejected without changing live state or input file")
        # Keep the current useful diagnostic, rather than the intentionally corrupt fixture.
        (ROOT/bad).unlink()
        c.call("capture_replay",replay_path=path,path="artifacts/replay-selected.png",tick=31,width=1280,height=900)
        output={"passed":True,"checks":checks,"samples":samples,"native_window":window,
                "build_id":before["build_id"],"slow_requests":c.slow_requests}
        (ROOT/"artifacts/replay-inspection-verification.json").write_text(json.dumps(output,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"samples":samples}))
    finally:
        c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis")
    p.add_argument("--window",action="store_true")
    args=p.parse_args()
    verify(args.binary.resolve(),args.window)
