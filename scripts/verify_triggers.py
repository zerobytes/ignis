#!/usr/bin/env python3
"""Author gameplay volumes, cross them and verify events entirely via MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    (ROOT/"artifacts").mkdir(exist_ok=True)
    c=Client(binary,window)
    checks,samples=[],[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},
                                "clientInfo":{"name":"ignis-trigger-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        assert "inspect_triggers" in json.dumps(c.request("tools/list"))
        c.call("generate",recipe={"width":16,"depth":14,"seed":73,"obstacles":0})
        player=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="player")
        player["transform"]["position"]=[-3,0.85,2]
        c.call("upsert",entity=player)
        zone={"id":"exit_zone","name":"Exit region","body":None,"color":[0.2,0.8,0.4],
              "transform":{"position":[0,1,2],"size":[2,2,2]},"visible":False,
              "trigger":{"enabled":True,"targets":["player"]}}
        c.call("upsert",entity=zone)
        marker={"id":"exit_marker","name":"Exit floor marker","body":None,"color":[0.2,0.8,0.4],
                "transform":{"position":[0,0.025,2],"size":[2,0.05,2]},"blocks_vision":False}
        c.call("upsert",entity=marker)
        crate={"id":"unselected_crate","name":"Unselected occupant","body":"dynamic","color":[0.5,0.6,0.7],
               "transform":{"position":[-0.5,0.85,2.9],"size":[0.3,1.6,0.3]}}
        c.call("upsert",entity=crate)
        c.call("save_scene",path="examples/triggers.scene.json")
        def capture():
            meta=c.call("capture",path="artifacts/triggers-current.png",width=640,height=450)
            return (ROOT/"artifacts/triggers-current.png").read_bytes(),meta
        hidden,meta=capture()
        assert meta["entity_count"]==len(c.call("inspect")["scene"]["entities"])-1
        visible=copy.deepcopy(zone);visible["visible"]=True
        c.call("upsert",entity=visible)
        assert capture()[0]!=hidden
        c.call("undo")
        assert capture()[0]==hidden
        c.call("step",ticks=30)
        assert c.call("inspect_triggers")["triggers"]["total_events"]==0
        checks.append("MCP authors current-version invisible sensor plus independent marker; visibility cache/undo exact and unselected occupant ignored")

        c.call("set_input",input={"x":1,"z":0})
        c.call("step",ticks=60)
        entered=c.call("inspect_triggers")["triggers"]
        assert entered["volumes"]["exit_zone"]["occupants"]==["player"]
        assert entered["volumes"]["exit_zone"]["entered"]==1 and entered["total_events"]==1
        c.call("capture",path="artifacts/triggers-entered.png",width=1280,height=900)
        c.call("step",ticks=60)
        left=c.call("inspect_triggers")["triggers"]
        assert left["volumes"]["exit_zone"]["occupants"]==[]
        assert left["volumes"]["exit_zone"]["exited"]==1 and left["total_events"]==2
        assert [event["kind"] for event in left["events"]]==["enter","exit"]
        samples.append({"events":left["events"]})
        c.call("save_replay",path="artifacts/triggers.replay.json")
        assert c.call("run_replay",path="artifacts/triggers.replay.json")["triggers_matched"]
        tick=left["events"][0]["tick"]
        prefix=c.call("inspect_replay",path="artifacts/triggers.replay.json",tick=tick)
        assert prefix["snapshot"]["triggers"]["volumes"]["exit_zone"]["occupants"]==["player"]
        c.call("capture_replay",replay_path="artifacts/triggers.replay.json",path="artifacts/triggers-prefix.png",tick=tick,width=1280,height=900)
        checks.append("post-physics enter/stay/exit, bounded evidence and replay/prefix inspection follow the same volume state")

        scenario={"version":2,"name":"Enter and leave a gameplay region","budget_ticks":200,"steps":[
            {"op":"set_input","input":{"x":1,"z":0}},
            {"op":"until","condition":{"kind":"trigger_occupied","id":"exit_zone","target":"player","expected":True},"max_ticks":100},
            {"op":"assert","condition":{"kind":"trigger_entries","id":"exit_zone","at_least":1}},
            {"op":"until","condition":{"kind":"trigger_occupied","id":"exit_zone","target":"player","expected":False},"max_ticks":100}]}
        c.call("save_scenario",path="examples/triggers.scenario.json",scenario=scenario)
        before=c.call("inspect")
        report=c.call("run_scenario",path="examples/triggers.scenario.json",report_path="artifacts/triggers-report.json")["report"]
        assert report["passed"] and report["final_snapshot"]["triggers"]["volumes"]["exit_zone"]["exited"]==1
        after=c.call("inspect")
        assert after==before,{k:[before[k],after[k]] for k in before if before[k]!=after[k]}
        samples.append({"scenario_ticks":report["ticks"]})
        bad=copy.deepcopy(scenario);bad["version"]=1
        c.call("save_scenario",path="examples/triggers.scenario.json",scenario=bad,expected_error=True)
        for targets in ([],["missing"],["player","player"],["player"]*33):
            bad=copy.deepcopy(zone);bad["trigger"]["targets"]=targets
            c.call("upsert",entity=bad,expected_error=True)
        c.call("remove",id="player",expected_error=True)
        after=c.call("inspect")
        assert after==before,{k:[before[k],after[k]] for k in before if before[k]!=after[k]}
        c.call("reset")
        assert c.call("inspect_triggers")["triggers"]["total_events"]==0
        disabled=copy.deepcopy(zone);disabled["trigger"]["enabled"]=False
        c.call("upsert",entity=disabled)
        c.call("set_input",input={"x":1,"z":0});c.call("step",ticks=120)
        assert c.call("inspect_triggers")["triggers"]["total_events"]==0
        c.call("load_scene",path="examples/triggers.scene.json")
        assert capture()[0]==hidden
        checks.append("typed scenario2 checks entry/exit in isolation; invalid IDs/budgets/versions atomic; reset/disable/load restore authored state")

        # A hidden wall remains a physical wall; visual presence is not collision policy.
        wall=next(e for e in c.call("inspect")["scene"]["entities"] if e["id"]=="wall_north")
        wall["visible"]=False;c.call("upsert",entity=wall)
        assert capture()[1]["entity_count"]==meta["entity_count"]-1
        c.call("set_input",input={"x":0,"z":-1});c.call("step",ticks=300)
        assert c.call("inspect")["runtime_positions"]["player"][2]>-6.5
        c.call("load_scene",path="examples/triggers.scene.json")
        assert capture()[0]==hidden
        checks.append("hidden physical wall still blocks motion; load/capture clears visibility changes without geometry residue")
        result={"passed":True,"checks":checks,"samples":samples,"native_window":window,
                "slow_requests":c.slow_requests,"build_id":c.call("inspect")["build_id"]}
        (ROOT/"artifacts/triggers-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"samples":samples}))
    finally:
        c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis")
    p.add_argument("--window",action="store_true");args=p.parse_args();verify(args.binary.resolve(),args.window)
