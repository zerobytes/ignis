#!/usr/bin/env python3
"""Exercise the real executable through MCP stdio; no in-process shortcuts."""
import argparse
import base64
import json
import os
import re
from pathlib import Path
import select
import subprocess
import time

ROOT = Path(os.environ.get("IGNIS_VERIFY_ROOT", Path(__file__).resolve().parents[1])).resolve()
SOURCE_ROOT = Path(__file__).resolve().parents[1]
# Check the current source contract; preserve explicit old-version rejection fixtures.
def source_version(path, name):
    declaration = re.search(r"pub const " + name + r": u32 = (\d+);", (SOURCE_ROOT / path).read_text())
    if declaration is None:
        raise RuntimeError(f"Missing source version {name}")
    return int(declaration.group(1))

SCENE_VERSION = source_version("src/model.rs", "SCENE_VERSION")
REPLAY_VERSION = source_version("src/engine.rs", "REPLAY_VERSION")



class Client:
    def __init__(self, binary, window=False, log_path=None, startup_args=(), detached=False):
        self.serial = 0
        self.transport_failed = False
        self.startup_timeout = int(os.environ.get("IGNIS_MCP_STARTUP_TIMEOUT", "120"))
        if not 1 <= self.startup_timeout <= 600:
            raise ValueError("IGNIS_MCP_STARTUP_TIMEOUT requires 1..600 seconds")
        self.slow_requests = []
        self.log = open(log_path or ROOT / "artifacts/runtime.log", "w")
        command = [str(binary), "--root", str(ROOT), "--mcp"]
        if window:
            command.append("--window")
        command.extend(startup_args)
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=self.log,
                                        start_new_session=detached)

    def request(self, method, params=None, notification=False):
        self.serial += 1
        payload = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            payload["params"] = params
        if not notification:
            payload["id"] = self.serial
        self.process.stdin.write(json.dumps(payload).encode() + b"\n")
        self.process.stdin.flush()
        if notification:
            return None
        op = (params or {}).get("arguments", {}).get("command", {}).get("op")
        # Startup and durable writes on external volumes can exceed the normal command budget.
        timeout = 120 if method == "initialize" or op in {"capture_editor", "save_editor", "load_editor", "capture", "save_scene", "save_replay", "load_scene", "run_replay", "run_scenario", "inspect_replay", "capture_replay", "load_planet_runtime", "set_planet_runtime"} else 45
        if method == "initialize":
            timeout = self.startup_timeout
        started = time.monotonic()
        ready, _, _ = select.select([self.process.stdout], [], [], timeout)
        if not ready:
            self.transport_failed = True
            raise RuntimeError(f"MCP timeout after {timeout}s: {method}/{op}; see artifacts/runtime.log")
        line = self.process.stdout.readline()
        if not line:
            self.transport_failed = True
            raise RuntimeError(f"MCP closed: {method}; see artifacts/runtime.log")
        response = json.loads(line)
        assert response["id"] == self.serial, f"Expected response {self.serial}, received {response.get('id')}"
        elapsed = time.monotonic() - started
        if elapsed > 45:
            self.slow_requests.append({"method": method, "op": op, "elapsed_seconds": round(elapsed, 3)})
        return response

    def call(self, op, expected_error=False, expected_revision=None, **kwargs):
        args = {"command": {"op": op, **kwargs}}
        if expected_revision is not None:
            args["expected_revision"] = expected_revision
        result = self.request("tools/call", {"name": "engine_command", "arguments": args})
        assert "error" not in result, result
        result = result["result"]
        assert result["isError"] == expected_error, result
        if op in {"capture", "capture_replay", "capture_editor", "capture_multiplayer"} and not expected_error:
            image = next(c for c in result["content"] if c["type"] == "image")
            image_bytes = base64.b64decode(image["data"])
            assert image_bytes[:8] == b"\x89PNG\r\n\x1a\n"
            assert image_bytes == (ROOT / kwargs["path"]).read_bytes()
        return result.get("structuredContent", result)

    def close(self):
        failed = self.transport_failed
        try:
            if self.process.poll() is None and not failed:
                self.call("exit")
        finally:
            failed |= self.transport_failed
            if self.process.poll() is None:
                if failed:
                    self.process.terminate()
                try:
                    self.process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait(timeout=15)
            self.process.stdin.close()
            self.process.stdout.close()
            self.log.close()
        if not failed:
            assert self.process.returncode == 0


def verify(binary, window, hold):
    (ROOT / "artifacts").mkdir(exist_ok=True)
    client = Client(binary, window)
    try:
        init = client.request("initialize", {
            "protocolVersion": "2025-11-25", "capabilities": {},
            "clientInfo": {"name": "ignis-laboratory-verifier", "version": "1"}})
        assert init["result"]["protocolVersion"] == "2025-11-25"
        client.request("notifications/initialized", notification=True)
        assert client.request("ping")["result"] == {}
        tools = client.request("tools/list")["result"]["tools"]
        assert len(tools) == 1 and tools[0]["name"] == "engine_command"
        (ROOT / "artifacts/command-schema.json").write_text(json.dumps(tools[0]["inputSchema"], indent=2))

        recipe = {"width": 16, "depth": 14, "seed": 42, "obstacles": 0}
        client.call("generate", recipe=recipe)
        initial = client.call("inspect")
        north = next(e for e in initial["scene"]["entities"] if e["id"] == "wall_north")
        invalid = json.loads(json.dumps(north))
        invalid["transform"]["size"][0] = -1
        client.call("upsert", entity=invalid, expected_error=True)
        assert client.call("inspect") == initial
        client.call("remove", id="wall_north", expected_revision=9999, expected_error=True)
        assert client.call("inspect") == initial
        # Exercise independent creation and removal, preserving the authored room.
        marker = {**north, "id": "temporary_marker", "name": "temporary marker", "body": None}
        client.call("upsert", entity=marker)
        assert any(e["id"] == marker["id"] for e in client.call("inspect")["scene"]["entities"])
        client.call("remove", id=marker["id"])

        client.call("remove", id="wall_north")
        client.call("set_input", input={"x": 0, "z": -1})
        client.call("step", ticks=300)
        broken = client.call("inspect")
        assert broken["runtime_positions"]["player"][2] < -7, broken
        client.call("undo")
        client.call("set_input", input={"x": 0, "z": -1})
        client.call("step", ticks=300)
        repaired = client.call("inspect")
        assert -6.49 < repaired["runtime_positions"]["player"][2] < -6.3, repaired
        events = client.call("events")
        assert any({event["a"], event["b"]} == {"player", "wall_north"}
                   and event["kind"] == "contact_started" for event in events["events"])
        client.call("save_scene", path="artifacts/lab.scene.json")
        saved = json.loads((ROOT / "artifacts/lab.scene.json").read_text())
        assert {e["id"] for e in saved["entities"]} == {e["id"] for e in repaired["scene"]["entities"]}
        assert next(e for e in saved["entities"] if e["id"] == "player")["transform"]["position"][2] == 5
        client.call("save_replay", path="artifacts/lab.replay.json")
        replay = client.call("run_replay", path="artifacts/lab.replay.json")
        assert replay["matched"], replay
        assert client.call("run_replay", path="artifacts/lab.replay.json") == replay
        client.call("capture", path="artifacts/collision.png", width=1280, height=900)
        client.call("load_scene", path="artifacts/lab.scene.json")
        reloaded = client.call("inspect")
        assert reloaded["tick"] == 0
        assert reloaded["scene"] == repaired["scene"]
        client.call("generate", recipe={**recipe, "obstacles": 4})
        client.call("step", ticks=30)
        capture = client.call("capture", path="artifacts/lab.png", width=1280, height=900)
        client.call("save_scene", path="artifacts/lab.scene.json")
        report = {
            "passed": True, "transport": "MCP stdio", "native_window": window,
            "checks": ["initialize/ping/discovery", "atomic invalid edit", "revision conflict",
                       "entity create/remove", "fault detected", "undo repaired collision",
                       "contact events", "authoring/runtime isolation", "save/load",
                       "replay exact positions twice", "GPU capture returned through MCP"],
            "broken_player": broken["runtime_positions"]["player"],
            "repaired_player": repaired["runtime_positions"]["player"],
            "replay": replay, "capture": capture,
            "binary": str(binary), "slow_requests": client.slow_requests,
        }
        (ROOT / "artifacts/verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(report["checks"]), "capture": capture}, indent=2), flush=True)
        if hold:
            signal = ROOT / "artifacts/close-window"
            signal.unlink(missing_ok=True)
            client.call("play")
            print("Window available for review. Create artifacts/close-window to close through MCP.", flush=True)
            while not signal.exists() and client.process.poll() is None:
                time.sleep(0.25)
            signal.unlink(missing_ok=True)
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    parser.add_argument("--hold", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window, args.hold)
