#!/usr/bin/env python3
"""Author and run bounded verification documents exclusively through MCP."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    c=Client(binary,window)
    checks,samples=[],[]
    try:
        c.request("initialize",{"protocolVersion":"2025-11-25","capabilities":{},
                                "clientInfo":{"name":"ignis-scenario-verifier","version":"1"}})
        c.request("notifications/initialized",notification=True)
        schema=json.dumps(c.request("tools/list"))
        assert all(op in schema for op in ("save_scenario","inspect_scenario","run_scenario"))
        c.call("load_scene",path="examples/stealth.scene.json")
        quiet={"version":1,"name":"Quiet footsteps stay unheard behind the wall","budget_ticks":60,"steps":[
            {"op":"step","ticks":30},
            {"op":"set_input","input":{"x":1,"z":0,"mode":"sneak"}},
            {"op":"step","ticks":20},
            {"op":"assert","condition":{"kind":"npc_memory","id":"hunter","source":None}},
            {"op":"assert","condition":{"kind":"footsteps","min":1,"max":3}},
            {"op":"assert","condition":{"kind":"position_within","id":"player","center":[0.5,0.8,-2],"radius":0.1}},
            {"op":"until","condition":{"kind":"npc_observed","id":"hunter","expected":False},"max_ticks":1}]}
        c.call("save_scenario",path="examples/quiet.scenario.json",scenario=quiet)
        assert c.call("inspect_scenario",path="examples/quiet.scenario.json")["scenario"]["name"]==quiet["name"]
        c.call("set_input",input={"x":-1,"z":0,"mode":"run"})
        c.call("step",ticks=90)
        before=c.call("inspect")
        c.call("save_replay",path="artifacts/scenario-live.replay.json")
        live_replay=(ROOT/"artifacts/scenario-live.replay.json").read_bytes()
        result=c.call("run_scenario",path="examples/quiet.scenario.json",report_path="artifacts/quiet-report.json",replay_path="artifacts/quiet.replay.json")
        report=result["report"]
        assert report["passed"] and report["ticks"]==50 and report["completed_steps"]==7
        assert report["checks"][-1]["tick"]==50
        assert json.loads((ROOT/"artifacts/quiet-report.json").read_text())==report
        assert c.call("run_replay",path="artifacts/quiet.replay.json")["matched"]
        assert c.call("run_scenario",path="examples/quiet.scenario.json")["report"]==report
        after=c.call("inspect")
        assert after==before, {key:{"before":before[key],"after":after[key]} for key in before if before[key]!=after[key]}
        c.call("save_replay",path="artifacts/scenario-live.replay.json")
        assert (ROOT/"artifacts/scenario-live.replay.json").read_bytes()==live_replay
        samples.append({"quiet_ticks":report["ticks"],"checks":len(report["checks"])})
        checks.append("saved/inspected scenario runs from authoring, repeats exactly and exports identical report plus valid replay without touching live session")

        broken=copy.deepcopy(quiet)
        broken["steps"][3]["condition"]["source"]="sound"
        broken["name"]="Intentional failed expectation"
        c.call("save_scenario",path="examples/expected-failure.scenario.json",scenario=broken)
        result=c.call("run_scenario",path="examples/expected-failure.scenario.json",report_path="artifacts/expected-failure-report.json")["report"]
        assert not result["passed"] and result["failure"]["code"]=="assertion_failed"
        assert result["failure"]["step_index"]==3 and result["ticks"]==50
        assert result["checks"][-1]["actual"] is None
        broken["steps"]=[{"op":"until","condition":{"kind":"npc_observed","id":"hunter","expected":True},"max_ticks":3}]
        c.call("save_scenario",path="artifacts/timeout.scenario.json",scenario=broken)
        timed=c.call("run_scenario",path="artifacts/timeout.scenario.json")["report"]
        assert not timed["passed"] and timed["failure"]["code"]=="condition_timeout" and timed["ticks"]==3
        after=c.call("inspect")
        assert after==before, {key:{"before":before[key],"after":after[key]} for key in before if before[key]!=after[key]}
        samples.append({"intentional_failure":result["failure"],"timeout":timed["failure"]})
        checks.append("assertion failure and deadline return evidence at first failed step, as report rather than transport error")

        saved=(ROOT/"examples/quiet.scenario.json").read_bytes()
        bad=copy.deepcopy(quiet);bad["budget_ticks"]=50
        c.call("save_scenario",path="examples/quiet.scenario.json",scenario=bad,expected_error=True)
        assert (ROOT/"examples/quiet.scenario.json").read_bytes()==saved
        bad=copy.deepcopy(quiet);bad["steps"][3]["condition"]["id"]="missing"
        c.call("save_scenario",path="examples/quiet.scenario.json",scenario=bad,expected_error=True)
        for outputs in [{"report_path":"../escape.json"},{"report_path":"examples/quiet.scenario.json"},
                        {"report_path":"artifacts/same.json","replay_path":"artifacts/same.json"}]:
            c.call("run_scenario",path="examples/quiet.scenario.json",expected_error=True,**outputs)
        after=c.call("inspect")
        assert after==before, {key:{"before":before[key],"after":after[key]} for key in before if before[key]!=after[key]}
        assert (ROOT/"examples/quiet.scenario.json").read_bytes()==saved
        checks.append("budget/reference/path/output validation is atomic for both live state and saved input")

        c.call("load_scene",path="examples/patrol.scene.json")
        patrol={"version":1,"name":"Patrol resumes after investigating sound","budget_ticks":2400,"steps":[
            {"op":"step","ticks":30},
            {"op":"assert","condition":{"kind":"patrol_phase","id":"hunter","phase":"moving"}},
            {"op":"emit_noise","origin":{"kind":"position","position":[0,0.8,4]},"radius":12},
            {"op":"step","ticks":1},
            {"op":"assert","condition":{"kind":"npc_memory","id":"hunter","source":"sound"}},
            {"op":"assert","condition":{"kind":"patrol_phase","id":"hunter","phase":"suspended"}},
            {"op":"until","condition":{"kind":"patrol_phase","id":"hunter","phase":"moving"},"max_ticks":1000},
            {"op":"assert","condition":{"kind":"npc_memory","id":"hunter","source":None}},
            {"op":"until","condition":{"kind":"patrol_visits","id":"hunter","at_least":1},"max_ticks":1000}]}
        c.call("save_scenario",path="examples/patrol.scenario.json",scenario=patrol)
        before=c.call("inspect")
        report=c.call("run_scenario",path="examples/patrol.scenario.json",report_path="artifacts/patrol-report.json",replay_path="artifacts/scenario-patrol.replay.json")["report"]
        assert report["passed"] and report["checks"][3]["tick"]==922
        assert report["final_snapshot"]["patrols"]["hunter"]["visited"]>=1
        assert c.call("run_replay",path="artifacts/scenario-patrol.replay.json")["matched"]
        after=c.call("inspect")
        assert after==before, {key:{"before":before[key],"after":after[key]} for key in before if before[key]!=after[key]}
        samples.append({"patrol_ticks":report["ticks"],"checks":len(report["checks"])})
        checks.append("single-call patrol interruption/resume/arrival with bounded typed conditions and replay")
        output={"passed":True,"checks":checks,"samples":samples,"native_window":window,
                "slow_requests":c.slow_requests,"build_id":c.call("inspect")["build_id"]}
        (ROOT/"artifacts/scenarios-verification.json").write_text(json.dumps(output,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"samples":samples}))
    finally:
        c.close()

if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis")
    p.add_argument("--window",action="store_true")
    args=p.parse_args()
    verify(args.binary.resolve(),args.window)
