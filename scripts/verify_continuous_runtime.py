#!/usr/bin/env python3
"""Critical executable guards: offline continuity, bounded history and safe replay writes."""
import argparse
import json
import tempfile
from pathlib import Path
from verify_lab import Client, ROOT, SCENE_VERSION
from verify_local_transactions import offline_binary


def verify(binary, window):
    binary, network_denied = offline_binary(binary)
    c = Client(binary, window)
    checks = []
    try:
        c.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{},
                                "clientInfo":{"name":"ignis-continuous-runtime", "version":"1"}})
        c.request("notifications/initialized", notification=True)
        c.call("generate", recipe={"width":16,"depth":14,"obstacles":0,"seed":17})
        assert c.call("inspect")["scene"]["version"] == SCENE_VERSION
        c.call("save_scene", path="artifacts/continuous.scene.json")
        c.call("set_execution_mode", mode="continuous")
        c.call("load_scene", path="artifacts/continuous.scene.json")
        assert c.call("inspect_execution")["mode"] == "continuous"
        control = c.call("bind_control", actor="player")["control"]
        c.call("set_control_input", control=control, input={"x":0,"z":0})
        c.call("emit_noise", origin={"kind":"position","position":[0,0,0]}, radius=1)
        for _ in range(11):
            c.call("step", ticks=3600)
        accepted = c.call("set_control_input", control=control, input={"x":0,"z":0})
        assert accepted["sequence"] == 2
        state = c.call("inspect")
        assert state["tick"] == 39600
        assert all(n == 0 for n in state["execution"]["retained"].values())
        assert state["execution"]["admitted"] == {"controls":3,"abilities":0,"population":0,"noises":1}
        protected = ROOT / "artifacts/continuous-protected.replay.json"
        protected.write_text("preserve existing bytes\n")
        c.call("save_replay", path="artifacts/continuous-protected.replay.json", expected_error=True)
        assert protected.read_text() == "preserve existing bytes\n"
        c.call("set_execution_mode", mode="recorded", expected_error=True)
        assert c.call("inspect") == state
        checks.append("Offline simulation passes39600ticks with bounded retained input and still accepts control; replay export fails before overwriting a file")

        c.call("reset")
        c.call("set_control_input", control=control, input={"x":0,"z":0}, expected_error=True)
        assert c.call("inspect_execution")["mode"] == "continuous"
        c.call("set_execution_mode", mode="recorded")
        c.call("step", ticks=2)
        c.call("save_replay", path="artifacts/continuous-recorded.replay.json")
        assert c.call("run_replay", path="artifacts/continuous-recorded.replay.json")["matched"]
        checks.append("Reset invalidates control handles and preserves policy; explicit recorded mode retains valid legacy replay")

        c.call("reset")
        c.call("set_execution_mode", mode="continuous")
        with tempfile.TemporaryDirectory(prefix="ignis-continuous-project-",dir=ROOT/"artifacts") as directory:
            project = str(Path(directory)/"game")
            c.call("create_project",path=project,name="Continuous",discard_changes=True)
            assert c.call("inspect_execution")["mode"] == "continuous"
            c.call("new_project_scene",name="Other",discard_changes=True)
            assert c.call("inspect_execution")["mode"] == "continuous"
            c.call("open_project",path=project,discard_changes=True)
            assert c.call("inspect_execution")["mode"] == "continuous"
            build = c.call("inspect")["build_id"]
            # Close the engine before the temporary project is removed.
            c.close()
        checks.append("Project and scene replacements preserve the selected execution policy without retaining old command ownership")
        result={"passed":True,"checks":checks,"build_id":build,"network_denied":network_denied,"native_window":window}
        (ROOT/"artifacts/continuous-runtime-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps({"passed":True,"checks":len(checks),"build_id":build,"network_denied":network_denied}))
    finally:
        if c.process.poll() is None:
            c.close()


if __name__ == "__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--binary",type=Path,default=Path(__file__).resolve().parents[1]/"target/release/ignis")
    parser.add_argument("--window",action="store_true")
    args=parser.parse_args()
    verify(args.binary.resolve(),args.window)
