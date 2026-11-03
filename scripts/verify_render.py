#!/usr/bin/env python3
"""Check retained GPU geometry through pixel-exact MCP captures, without profiling."""
import argparse
import copy
import json
from pathlib import Path
from verify_lab import Client, ROOT


def verify(binary, window):
    client = Client(binary, window)
    checks = []
    try:
        client.request("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                                      "clientInfo": {"name": "ignis-render-verifier", "version": "1"}})
        client.request("notifications/initialized", notification=True)
        client.call("generate", recipe={"width": 16, "depth": 14, "seed": 42, "obstacles": 0})

        def capture(width=320, height=240):
            client.call("capture", path="artifacts/render-cache.png", width=width, height=height)
            return (ROOT / "artifacts/render-cache.png").read_bytes()

        before = capture()
        assert capture() == before
        checks.append("unchanged capture is pixel exact")
        scene = client.call("inspect")["scene"]
        player = next(e for e in scene["entities"] if e["id"] == "player")
        for field, value in [("color", [0, 1, 0]), ("size", [2, 3, 2]), ("position", [3, 0.85, 2])]:
            modified = copy.deepcopy(player)
            if field == "color":
                modified[field] = value
            else:
                modified["transform"][field] = value
            client.call("upsert", entity=modified)
            assert capture() != before
            client.call("undo")
            assert capture() == before
        checks.append("color, size and position invalidate geometry; undo restores exact pixels")
        # Exceed the initial buffer capacity, then shrink without leaving stale geometry.
        for i in range(12):
            marker = copy.deepcopy(player)
            marker.update(id=f"marker_{i}", name="Marker", body=None, color=[0, 0.8, 1])
            marker["transform"]["position"] = [-5 + i % 6 * 2, 1, -2 + i // 6 * 3]
            client.call("upsert", entity=marker)
        assert capture() != before
        for i in range(12):
            client.call("remove", id=f"marker_{i}")
        assert capture() == before
        checks.append("GPU buffer growth and shrinking scene have no stale boxes")
        client.call("set_input", input={"x": 1, "z": 0})
        client.call("step", ticks=30)
        assert capture() != before
        client.call("reset")
        assert capture() == before
        checks.append("runtime motion and reset invalidate geometry")
        capture(640, 360)
        assert capture() == before
        camera = copy.deepcopy(scene["camera"])
        camera["position"][0] *= -1
        client.call("set_camera", camera=camera)
        assert capture() != before
        client.call("undo")
        assert capture() == before
        checks.append("viewport and camera changes preserve retained geometry")
        report = {"passed": True, "checks": checks, "native_window": window,
                  "slow_requests": client.slow_requests, "build_id": client.call("inspect")["build_id"]}
        (ROOT / "artifacts/render-verification.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps({"passed": True, "checks": len(checks), "native_window": window}))
    finally:
        client.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/ignis")
    parser.add_argument("--window", action="store_true")
    args = parser.parse_args()
    verify(args.binary.resolve(), args.window)
