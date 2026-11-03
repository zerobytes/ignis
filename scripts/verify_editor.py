#!/usr/bin/env python3
"""Editor layout, scene controls and real UI composition through MCP; no window."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    if window:
        raise ValueError("Use --editor explicitly for desktop UI acceptance; this scenario is offscreen")
    (ROOT / "artifacts").mkdir(exist_ok=True)
    c = Client(binary, False)
    checks = []
    try:
        c.request("initialize", {"protocolVersion":"2025-11-25", "capabilities":{},
                                  "clientInfo":{"name":"ignis-editor-verifier","version":"1"}})
        c.request("notifications/initialized", notification=True)
        schema = json.dumps(c.request("tools/list"))
        assert all(op in schema for op in ["inspect_editor", "edit_editor", "save_editor", "load_editor", "capture_editor"])
        c.call("load_scene", path="examples/stealth-lab.scene.json")
        c.call("select_entity", id="hunter")
        before = c.call("inspect")
        original = c.call("inspect_editor")["editor"]
        c.call("save_replay", path="artifacts/editor.replay.json")
        replay = (ROOT / "artifacts/editor.replay.json").read_bytes()

        def edit(action):
            revision = c.call("inspect_editor")["editor"]["revision"]
            return c.call("edit_editor", action=action, expected_editor_revision=revision)["editor"]

        edit({"action":"resize","area":0,"ratio":0.72})
        edit({"action":"split","area":1,"axis":"vertical","panel":"events"})
        edit({"action":"maximize","area":1})
        edit({"action":"maximize","area":None})
        edit({"action":"move","from":3,"to":4,"panel":"templates"})
        edit({"action":"duplicate","id":"personal","name":"Pessoal"})
        edit({"action":"rename","name":"Depuração pessoal"})
        c.call("save_editor", path="artifacts/editor.layout.json")
        saved = c.call("inspect_editor")["editor"]
        edit({"action":"restore"})
        c.call("load_editor", path="artifacts/editor.layout.json")
        assert c.call("inspect_editor")["editor"]["workspaces"] == saved["workspaces"]
        assert c.call("inspect") == before
        c.call("save_replay", path="artifacts/editor.replay.json")
        assert (ROOT / "artifacts/editor.replay.json").read_bytes() == replay
        checks.append("typed layout/workspace operations, persistence and scene/runtime/replay invariance")

        for action in [{"action":"switch","workspace":"missing"}, {"action":"resize","area":0,"ratio":1.1},
                       {"action":"close","area":1,"panel":"viewport"}, {"action":"maximize","area":999},
                       {"action":"move","from":4,"to":999,"panel":"properties"}]:
            state = c.call("inspect_editor")
            c.call("edit_editor", action=action, expected_error=True)
            assert c.call("inspect_editor") == state and c.call("inspect") == before
        c.call("edit_editor", action={"action":"restore"}, expected_editor_revision=999, expected_error=True)
        c.call("save_editor", path="../escape.json", expected_error=True)
        checks.append("invalid layouts, stale revision and confined paths preserve session and world")

        edit({"action":"switch","workspace":"scene"})
        edit({"action":"restore"})
        def capture(path, width=1440, height=900):
            c.call("capture_editor", path=path, width=width, height=height)
            data=(ROOT / path).read_bytes()
            assert data[:8] == b"\x89PNG\r\n\x1a\n"
            return data
        pixels=capture("artifacts/editor-current.png")
        assert capture("artifacts/editor-repeat.png") == pixels
        assert c.call("inspect") == before
        edit({"action":"switch","workspace":"inspect"})
        assert capture("artifacts/editor-inspect.png") != pixels
        edit({"action":"switch","workspace":"scene"})
        assert capture("artifacts/editor-current.png") == pixels
        camera=copy.deepcopy(before["scene"]["camera"])
        camera["position"][0] += 3
        edit({"action":"camera","camera":camera})
        assert capture("artifacts/editor-camera.png") != pixels
        assert c.call("inspect") == before
        edit({"action":"camera","camera":None})
        assert capture("artifacts/editor-current.png") == pixels
        capture("artifacts/editor-compact.png", 900, 600)
        c.call("capture_editor", path="artifacts/invalid.png", width=320, height=240, expected_error=True)
        checks.append("real offscreen editor GPU composition, repeat/switch/restore pixels and compact viewport")

        hunter=copy.deepcopy(next(e for e in before["scene"]["entities"] if e["id"] == "hunter"))
        hunter["transform"]["position"][0] += 1
        c.call("upsert", entity=hunter)
        changed=capture("artifacts/editor-edited.png")
        assert changed != pixels
        c.call("undo")
        # Revision/status text is expected to differ; scene pixels are covered by render/selection.
        assert c.call("inspect")["scene"] == before["scene"]
        c.call("redo")
        c.call("step", ticks=3)
        assert c.call("inspect")["tick"] == 3
        c.call("reset")
        c.call("save_scene", path="artifacts/editor.scene.json")
        assert "workspaces" not in json.loads((ROOT / "artifacts/editor.scene.json").read_text())
        assert original["version"] == 3
        checks.append("existing authoring undo/redo, stepping/reset and scene persistence remain separate")

        # Exercise the editor compositor with actual authoring from the merged fronts
        # and functional population in the same document, not only legacy boxes.
        c.call("load_scene", path="examples/native-curved-cylinder.scene.json")
        authoring = c.call("inspect")["scene"]
        assert authoring["meshes"]["assets"] and authoring["plants"]["entries"]
        c.call("load_scene", path="examples/npc-generator-activation.scene.json")
        combined = c.call("inspect")["scene"]
        combined["meshes"] = authoring["meshes"]
        combined["plants"] = authoring["plants"]
        # Weighted base is inherited by the first wave; preserve the existing
        # spatial activation fixture while testing the new compiled choices.
        generator = combined["npc_generators"][0]
        base_template = generator.pop("template")
        generator["templates"] = [{"template":base_template, "weight":3}, {"template":"elite", "weight":1}]
        for instance in combined["meshes"]["instances"]:
            instance.update(position=[-2, 2, 0], scale=8, color=[0.2, 0.8, 1])
        next(e for e in combined["entities"] if e["id"] == "player")["transform"]["position"][0] = 3
        combined_path = "artifacts/editor-combined.scene.json"
        (ROOT / combined_path).write_text(json.dumps(combined))
        c.call("load_scene", path=combined_path)
        combined_before = c.call("inspect")
        combined_pixels = capture("artifacts/editor-combined.png")
        assert capture("artifacts/editor-combined-repeat.png") == combined_pixels
        assert c.call("inspect") == combined_before
        without_mesh = copy.deepcopy(combined)
        without_mesh["meshes"]["instances"] = []
        (ROOT / "artifacts/editor-without-mesh.scene.json").write_text(json.dumps(without_mesh))
        c.call("load_scene", path="artifacts/editor-without-mesh.scene.json")
        capture("artifacts/editor-without-mesh.png")
        region = [{"id":"viewport", "x":0, "y":100, "width":1000, "height":700}]
        def viewport_difference(candidate):
            return c.call("compare_images", reference_path="artifacts/editor-combined.png",
                          candidate_path=candidate, regions=region)["report"]["regions"][0]["metrics"]
        visible_mesh_pixels = viewport_difference("artifacts/editor-without-mesh.png")["changed_pixels"]
        assert visible_mesh_pixels > 200
        c.call("load_scene", path=combined_path)
        capture("artifacts/editor-combined-restored.png")
        assert viewport_difference("artifacts/editor-combined-restored.png")["exact_rgb_match"]
        c.call("step", ticks=60)
        assert c.call("inspect_generators")["state"]["generators"][0]["births"] > 0
        assert c.call("inspect_generators")["controls"][0]["template_draws"] > 0
        capture("artifacts/editor-combined-live.png")
        c.call("save_replay", path="artifacts/editor-combined.replay.json")
        report = c.call("run_replay", path="artifacts/editor-combined.replay.json")
        assert report["matched"] and report["meshes_matched"] and report["generators_matched"]
        checks.append("merged plants/meshes render inside the editor alongside weighted live generators and matching replay")
        # New native constructor must reach the same editor renderer and replay.
        c.call("edit_mesh", edit={"kind":"panel", "id":"editor-panel",
               "dimensions":[6, 4, 0.3], "openings":[{"min":[-1, -2], "max":[1, 0.5]}]})
        capture("artifacts/editor-panel-baseline.png")
        c.call("edit_mesh", edit={"kind":"put_instance", "instance":{
               "id":"editor-panel", "asset":"editor-panel", "position":[3, 2, 0],
               "rotation":[0, 0, 0, 1], "scale":1, "color":[1, 0.35, 0.1], "playback":None}})
        panel_state = c.call("inspect")
        capture("artifacts/editor-panel-visible.png")
        panel_metrics = c.call("compare_images", reference_path="artifacts/editor-panel-baseline.png",
                               candidate_path="artifacts/editor-panel-visible.png",
                               regions=region)["report"]["regions"][0]["metrics"]
        visible_panel_pixels = panel_metrics["changed_pixels"]
        assert visible_panel_pixels > 200
        assert c.call("inspect") == panel_state
        c.call("save_replay", path="artifacts/editor-panel.replay.json")
        panel_replay = c.call("run_replay", path="artifacts/editor-panel.replay.json")
        assert panel_replay["matched"] and panel_replay["meshes_matched"] and panel_replay["generators_matched"]
        c.call("undo")
        capture("artifacts/editor-panel-undone.png")
        assert c.call("compare_images", reference_path="artifacts/editor-panel-baseline.png",
                      candidate_path="artifacts/editor-panel-undone.png",
                      regions=region)["report"]["regions"][0]["metrics"]["exact_rgb_match"]
        checks.append("native solid panel reaches editor GPU composition, replay and exact undo with plants/generators")
        from editor_integration_probe import verify_combined
        checks.extend(verify_combined(c))
        result={"passed":True,"checks":len(checks),"native_window":False,
                "native_input_tested":False,"visible_mesh_pixels":visible_mesh_pixels,
                "visible_panel_pixels":visible_panel_pixels,
                "slow_requests":c.slow_requests}
        (ROOT / "artifacts/editor-verification.json").write_text(json.dumps(result,indent=2)+"\n")
        print(json.dumps(result))
    finally:
        c.close()


if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary",type=Path,default=ROOT/"target/release/ignis")
    parser.add_argument("--window",action="store_true")
    args=parser.parse_args()
    verify(args.binary,args.window)
