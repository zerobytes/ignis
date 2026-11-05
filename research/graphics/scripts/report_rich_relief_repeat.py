#!/usr/bin/env python3
"""Audit fixed diagnostic cohorts without turning stable samples into a fix."""
import hashlib
import json
import math
from pathlib import Path

OUT = Path(__file__).resolve().parents[1] / "artifacts/appearance-probe/rich-relief"


def summarize(name, suffix):
    path = OUT / f"{name}-repeat.json"
    rows = json.loads(path.read_text())
    order = ["implicit", "explicit-lod", "manual-trilinear",
             "manual-trilinear", "explicit-lod", "implicit"]
    expected = {(width, case, round_, variant + suffix)
                for width in (1728, 3456) for case in (0, 1)
                for round_, variant in enumerate(order)}
    assert len(rows) == 24
    assert {(r["width"], r["case"], r["round"], r["variant"])
            for r in rows} == expected
    for row in rows:
        assert row["captures"] == 32
        assert 0 <= row["changed_captures"] <= row["captures"]
        assert (row["max_pixels"] == 0) == (row["changed_captures"] == 0)
        q = row["comparison_to_implicit"]
        assert all(math.isfinite(q[k]) for k in ("rmse", "p99", "max", "coverage_error"))
        assert q["pass"] == (q["rmse"] <= .005 and q["p99"] <= .02
                             and q["coverage_error"] <= .0005)
    variants = {}
    for variant in dict.fromkeys(order):
        group = [r for r in rows if r["variant"] == variant + suffix]
        variants[variant] = {
            "returns": sum(r["captures"] for r in group),
            "changed_returns": sum(r["changed_captures"] for r in group),
            "max_changed_pixels": max(r["max_pixels"] for r in group),
            "max_channel_change": max(r["max_channel_error"] for r in group),
            "material_comparisons_passed": sum(r["comparison_to_implicit"]["pass"] for r in group),
            "max_rmse_vs_control": max(r["comparison_to_implicit"]["rmse"] for r in group),
        }
        if suffix:
            variants[variant].update(
                derivative_pixel_events=sum(r["derivative_pixel_changes"] for r in group),
                color_events_with_identical_derivatives=sum(
                    r["color_changes_with_same_derivatives"] for r in group),
            )
    return {
        "file": path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "instrumented": bool(suffix),
        "variants": variants,
        "all_returns_equal": all(r["changed_captures"] == 0 for r in rows),
        "all_material_comparisons_pass": all(r["comparison_to_implicit"]["pass"] for r in rows),
    }


def main():
    summary = {
        "cohorts_complete": True,
        "promotion": False,
        "performance_measured": False,
        "cohorts": [summarize("texel", ""), summarize("inputs", "-inputs")],
        "limitation": "Instrumentation can change execution; correlation does not establish a driver defect or a universal fix.",
    }
    (OUT / "texel-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
