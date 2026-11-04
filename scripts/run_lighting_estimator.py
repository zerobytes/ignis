# /// script
# requires-python = ">=3.11"
# dependencies = ["matplotlib>=3.9,<4", "numpy>=2,<3"]
# ///
"""Run the standalone CPU experiment: uv run scripts/run_lighting_estimator.py.

This is a transport estimator experiment, not an engine GPU benchmark.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import html
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time
from collections import defaultdict
from fractions import Fraction as F

import numpy as np
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "artifacts/lighting-estimator"
SOURCE = ROOT / "scripts/lighting_estimator.rs"
CASES = [
    "smooth_polynomial",
    "missed_emitter",
    "stale_analytic",
    "hemisphere_open",
    "hemisphere_occluded",
    "hemisphere_changed",
]
METHODS = ["plain", "cache_only", "corrected_exact", "two_level"]
COLORS = dict(zip(METHODS, ["#475569", "#ca8a04", "#0284c7", "#9333ea"]))
LABELS = {
    "plain": "Plain MC",
    "cache_only": "Cache only",
    "corrected_exact": "Cache + residual (exact G)",
    "two_level": "Two-level (4n cheap + n residual)",
}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_csv(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")


def run(command, **kwargs):
    return subprocess.run(
        command, check=True, text=True, capture_output=True, **kwargs
    ).stdout.strip()


def int_list(value):
    result = [int(v) for v in value.split(",")]
    if not result or len(result) > 8 or sorted(set(result)) != result:
        raise argparse.ArgumentTypeError(
            "use an increasing list of at most eight integers"
        )
    return result


def analytic_moments(case, side):
    """Independent rational integration, including exact cache/residual moments.

    No trial samples or measured means enter these oracles. Coefficients are
    the declared analytic controls, not fitted representations of run outputs.
    """
    polynomials = {
        "smooth_polynomial": [
            [(F(1, 10), 0, 0), (F(4, 5), 2, 0), (F(3, 10), 0, 2)],
            [(F(1, 5), 0, 0), (F(7, 10), 1, 1), (F(1, 4), 2, 0)],
            [(F(3, 10), 0, 0), (F(3, 10), 1, 0), (F(2, 5), 0, 3)],
        ],
        "stale_analytic": [
            [(F(1, 10), 0, 0), (F(1, 20), 1, 0)],
            [(F(2, 25), 0, 0), (F(1, 25), 0, 1)],
            [(F(1, 25), 0, 0), (F(3, 100), 1, 0)],
        ],
    }
    if case not in polynomials and case != "missed_emitter":
        return None

    def integ(poly, x0=F(0), x1=F(1), y0=F(0), y1=F(1)):
        return sum(
            c
            * (x1 ** (a + 1) - x0 ** (a + 1))
            / (a + 1)
            * (y1 ** (b + 1) - y0 ** (b + 1))
            / (b + 1)
            for c, a, b in poly
        )

    base = [F(1, 10), F(2, 25), F(1, 25)]
    amplitude = [F(120), F(45), F(10)]
    source = polynomials.get(case)
    if case == "stale_analytic":
        source = [
            [(F(1, 10), 0, 0), (F(5), 1, 0)],
            [(F(2, 25), 0, 0), (F(4), 0, 1)],
            [(F(1, 25), 0, 0), (F(3), 1, 0)],
        ]
    result = []
    for channel in range(3):
        if case == "missed_emitter":
            mean = base[channel] + amplitude[channel] / 2500
            second = (
                base[channel] ** 2
                + (2 * base[channel] * amplitude[channel] + amplitude[channel] ** 2)
                / 2500
            )
        else:
            poly = polynomials[case][channel]
            mean = integ(poly)
            second = integ(
                [(a * b, u + x, v + y) for a, u, v in poly for b, x, y in poly]
            )
        eg = eg2 = efg = F(0)
        for y in range(side):
            for x in range(side):
                x0, x1, y0, y1 = F(x, side), F(x + 1, side), F(y, side), F(y + 1, side)
                u, v = (x0 + x1) / 2, (y0 + y1) / 2
                if case == "missed_emitter":
                    hit = F(501, 1000) <= u < F(503, 1000) and F(101, 1000) <= v < F(
                        301, 1000
                    )
                    g = base[channel] + amplitude[channel] * hit
                    overlap = max(
                        F(0), min(x1, F(503, 1000)) - max(x0, F(501, 1000))
                    ) * max(F(0), min(y1, F(301, 1000)) - max(y0, F(101, 1000)))
                    integral_f = base[channel] / side**2 + amplitude[channel] * overlap
                else:
                    g = sum(c * u**a * v**b for c, a, b in source[channel])
                    integral_f = integ(polynomials[case][channel], x0, x1, y0, y1)
                eg += g / side**2
                eg2 += g * g / side**2
                efg += g * integral_f
        variance_f = second - mean * mean
        variance_g = eg2 - eg * eg
        variance_residual = second - 2 * efg + eg2 - (mean - eg) ** 2
        assert min(variance_f, variance_g, variance_residual) >= 0
        result.append(
            dict(
                mean=float(mean),
                cache_mean=float(eg),
                variance_f=float(variance_f),
                variance_g=float(variance_g),
                variance_residual=float(variance_residual),
            )
        )
    return result


def summarize(work, config):
    references = defaultdict(list)
    for row in read_csv(work / "references.csv"):
        references[row["case"]].append(
            dict(grid=int(row["grid"]), rgb=[float(row[c]) for c in "rgb"])
        )
    ref_report = {}
    for case, levels in references.items():
        finest = np.array(levels[-1]["rgb"])
        deltas = [np.array(b["rgb"]) - a["rgb"] for a, b in zip(levels, levels[1:])]
        ref_report[case] = dict(
            levels=levels,
            rgb=finest.tolist(),
            exact=levels[-1]["grid"] == 0,
            refinement_differences_rgb=[d.tolist() for d in deltas],
            last_refinement_rms=float(np.sqrt(np.mean(deltas[-1] ** 2)))
            if deltas
            else 0.0,
            uncertainty="exact analytic integral"
            if not deltas
            else "refinement difference is diagnostic, not a rigorous error bound",
        )
    caches = {(r["case"], int(r["grid"])): r for r in read_csv(work / "caches.csv")}
    timings = defaultdict(list)
    for row in read_csv(work / "timings.csv"):
        timings[(row["case"], int(row["grid"]), int(row["n"]), row["method"])].append(
            float(row["ns_per_estimate"])
        )
    timing_summary = {
        key: dict(
            median_ns=float(np.median(values)),
            p95_block_average_ns=float(np.percentile(values, 95)),
            block_average_ns=values,
        )
        for key, values in timings.items()
    }
    groups = defaultdict(list)
    for row in read_csv(work / "trials.csv"):
        groups[(row["case"], int(row["grid"]), int(row["n"]))].append(row)
    # Fixed bootstrap indices affect uncertainty reporting only. They never
    # enter fitting, sampling, timing or estimator parameter selection.
    indices = np.random.default_rng(config["seed"] ^ 0x6A09E667).integers(
        0, config["trials"], (config["bootstrap_replicates"], config["trials"])
    )
    summaries = []
    theory = {
        (case, grid): analytic_moments(case, grid)
        for case in CASES
        for grid in config["cache_grids"]
    }
    for (case, grid, n), trials in groups.items():
        assert [int(t["trial"]) for t in trials] == list(range(config["trials"]))
        truth = np.array(ref_report[case]["rgb"])
        cache = caches[case, grid]
        g = np.array([float(cache[c]) for c in "rgb"])
        values = {
            m: np.array([[float(t[f"{m}_{c}"]) for c in "rgb"] for t in trials])
            for m in ["plain", "corrected_exact", "two_level"]
        }
        values["cache_only"] = np.repeat(g[None, :], config["trials"], axis=0)
        errors = {m: a - truth for m, a in values.items()}
        per_trial_mse = {m: np.mean(e * e, axis=1) for m, e in errors.items()}
        bootstrap = {m: err[indices].mean(axis=1) for m, err in per_trial_mse.items()}
        for method in METHODS:
            a, error = values[method], errors[method]
            bias = error.mean(axis=0)
            variance = a.var(axis=0, ddof=1)
            sem = np.sqrt(variance / config["trials"])
            mse_rgb = (error * error).mean(axis=0)
            assert np.isfinite(a).all()
            # Protect the report's scientific meaning: sample MSE decomposition
            # must agree with population variance + squared empirical bias.
            assert np.allclose(
                mse_rgb, a.var(axis=0) + bias * bias, rtol=1e-9, atol=1e-12
            )
            row = dict(
                case=case,
                grid=grid,
                n_expensive=n if method != "cache_only" else 0,
                comparison_n=n,
                method=method,
                trials=config["trials"],
                mean_rgb=a.mean(axis=0).tolist(),
                mean_error_rgb=bias.tolist(),
                mean_error_ci95_normal_rgb=np.stack(
                    [bias - 1.96 * sem, bias + 1.96 * sem], axis=1
                ).tolist(),
                mean_ci_note="normal approximation over independent trials; rare-event tails can invalidate coverage",
                variance_rgb=variance.tolist(),
                mse_rgb=mse_rgb.tolist(),
                mse_rgb_mean=float(mse_rgb.mean()),
                mse_ci95_bootstrap=np.percentile(
                    bootstrap[method], [2.5, 97.5]
                ).tolist(),
                cache_integral_rgb=g.tolist(),
                cache_payload_bytes=int(cache["payload_bytes"]),
                cache_source_calls=int(cache["source_calls"]),
                reference_exact=ref_report[case]["exact"],
                reference_last_refinement_rms=ref_report[case]["last_refinement_rms"],
            )
            if method != "cache_only":
                timing = timing_summary[case, grid, n, method]
                setup = (
                    timing_summary[case, grid, 0, "cache_build"]
                    if method != "plain"
                    else None
                )
                setup_ns = setup["median_ns"] if setup else 0.0
                row.update(
                    timing=timing,
                    cache_build_timing=setup,
                    cold_cost_estimate_ns=timing["median_ns"] + setup_ns,
                    amortized_16_cost_estimate_ns=timing["median_ns"] + setup_ns / 16,
                    amortized_256_cost_estimate_ns=timing["median_ns"] + setup_ns / 256,
                    warm_mse_times_ns=float(mse_rgb.mean()) * timing["median_ns"],
                    active_input_payload_bytes=(5 * n if method == "two_level" else n)
                    * 16,
                    allocated_sampling_payload_bytes=5 * n * 16,
                    input_generation_timing=timing_summary[
                        case,
                        grid,
                        n,
                        "input_generation_5n"
                        if method == "two_level"
                        else "input_generation_n",
                    ],
                    cached_lookups=(5 * n if method == "two_level" else n)
                    if method != "plain"
                    else 0,
                )
                paired_ratio = bootstrap["plain"] / np.maximum(
                    bootstrap[method], 1e-300
                )
                row.update(
                    mse_gain_over_plain=float(
                        per_trial_mse["plain"].mean() / max(mse_rgb.mean(), 1e-300)
                    ),
                    mse_gain_ci95_paired_bootstrap=np.percentile(
                        paired_ratio, [2.5, 97.5]
                    ).tolist(),
                )
            moments = theory[case, grid]
            if moments:
                assert np.allclose(
                    [c["mean"] for c in moments], truth, rtol=0, atol=1e-14
                )
                assert np.allclose(
                    [c["cache_mean"] for c in moments], g, rtol=0, atol=1e-12
                )
                expected_bias = [
                    c["cache_mean"] - c["mean"] if method == "cache_only" else 0.0
                    for c in moments
                ]
                expected_variance = [
                    0.0
                    if method == "cache_only"
                    else c["variance_f"] / n
                    if method == "plain"
                    else c["variance_residual"] / n
                    + (c["variance_g"] / (4 * n) if method == "two_level" else 0.0)
                    for c in moments
                ]
                row["analytic_theory"] = dict(
                    expected_bias_rgb=expected_bias,
                    expected_variance_rgb=expected_variance,
                    expected_mse_rgb_mean=float(
                        np.mean(np.array(expected_bias) ** 2 + expected_variance)
                    ),
                    note="exact rational moments for ideal continuous independent uniform samples; no fitting to trials",
                )
            if case == "missed_emitter":
                baseline = np.array([0.1, 0.08, 0.04])
                row["no_emitter_hit_fraction"] = float(
                    np.mean(
                        np.all(
                            np.isclose(values["plain"], baseline, rtol=0, atol=1e-12),
                            axis=1,
                        )
                    )
                )
                row["expected_no_emitter_hit_fraction"] = (1.0 - 0.0004) ** n
            summaries.append(row)
    return dict(
        schema=1,
        config=config,
        references=ref_report,
        estimates=summaries,
        timing_scope="single-threaded CPU wall time, including PRNG and fill, excluding scratch allocation, references and report IO",
        timing_p95_note="p95 across block averages, not per-estimate tail latency",
        setup_cost_note="build timing includes cache allocation and destruction; cold/amortized estimates sum separately measured medians, not a directly timed cold run",
        comparison="equal expensive evaluation count; MSE*time is an efficiency proxy, not an equal-time run",
        cache_note="G integrates the piecewise-constant g exactly, not f; two_level samples g independently",
        limitations=[
            "single receiver/environment visibility, no indirect bounces",
            "toy CPU costs do not predict GPU gains",
            "quadrature refinement differences are not certified transport reference error bounds",
            "pseudorandom independent streams; common Y pairs methods and cache resolutions",
            "normal mean confidence intervals may under-cover rare-event distributions",
            "cache reuse horizons are hypothetical repeats at this receiver, not measured frames or pixels",
        ],
    )


def plot(work, report):
    rows = report["estimates"]
    selected_grid = (
        8
        if 8 in report["config"]["cache_grids"]
        else report["config"]["cache_grids"][0]
    )
    for kind in ["convergence", "quality-cost"]:
        fig, axes = plt.subplots(2, 3, figsize=(15, 9))
        fig.subplots_adjust(
            left=0.07, right=0.99, bottom=0.17, top=0.85, wspace=0.28, hspace=0.40
        )
        for ax, case in zip(axes.flat, CASES):
            for method in METHODS:
                selected = [
                    r
                    for r in rows
                    if r["case"] == case
                    and r["grid"] == selected_grid
                    and r["method"] == method
                ]
                if kind == "quality-cost" and method == "cache_only":
                    continue
                selected.sort(key=lambda r: r["comparison_n"])
                x = [
                    r["comparison_n"]
                    if kind == "convergence"
                    else r["timing"]["median_ns"] / 1000
                    for r in selected
                ]
                y = [max(r["mse_rgb_mean"], 1e-30) for r in selected]
                ax.plot(
                    x, y, "o-", color=COLORS[method], label=LABELS[method], markersize=4
                )
                bounds = np.array([r["mse_ci95_bootstrap"] for r in selected])
                ax.fill_between(
                    x,
                    np.maximum(bounds[:, 0], 1e-30),
                    np.maximum(bounds[:, 1], 1e-30),
                    color=COLORS[method],
                    alpha=0.12,
                )
            reference = report["references"][case]
            if not reference["exact"]:
                ax.text(
                    0.03,
                    0.03,
                    f"Reference refinement RMS: {reference['last_refinement_rms']:.2g}\n(not an error bound)",
                    transform=ax.transAxes,
                    fontsize=8,
                    color="#dc2626",
                    bbox=dict(facecolor="white", alpha=0.8, edgecolor="none"),
                )
            ax.set(
                xscale="log",
                yscale="log",
                title=case.replace("_", " "),
                ylabel="Mean RGB squared error",
                xlabel="Expensive samples (cache-only: comparison budget)"
                if kind == "convergence"
                else "Warm CPU microseconds / estimate",
            )
            ax.grid(alpha=0.2, which="both")
        handles, labels = axes.flat[0].get_legend_handles_labels()
        fig.legend(
            handles, labels, loc="lower center", bbox_to_anchor=(0.5, 0.01), ncols=2
        )
        fig.suptitle(
            f"Lumens CPU estimator experiment — {report['config']['trials']} trials, {selected_grid}×{selected_grid} cache\n"
            + (
                "Equal expensive sample count; shaded bands: bootstrap 95% intervals"
                if kind == "convergence"
                else "Measured warm costs at sampled budgets; not a direct equal-time comparison"
            ),
            y=0.98,
            fontsize=14,
        )
        fig.savefig(work / f"{kind}.svg", bbox_inches="tight")
        fig.savefig(work / f"{kind}.png", dpi=135, bbox_inches="tight")
        plt.close(fig)
    maps = defaultdict(list)
    for row in read_csv(work / "maps.csv"):
        maps[row["case"]].append(row)
    fig, axes = plt.subplots(6, 3, figsize=(11, 17), layout="constrained")
    weight = np.array([0.2126, 0.7152, 0.0722])
    for i, case in enumerate(CASES):
        entries = maps[case]
        f = (
            np.array([[float(r[f"f_{c}"]) for c in "rgb"] for r in entries]).reshape(
                64, 64, 3
            )
            @ weight
        )
        g = (
            np.array([[float(r[f"g_{c}"]) for c in "rgb"] for r in entries]).reshape(
                64, 64, 3
            )
            @ weight
        )
        for j, values in enumerate([f, g, f - g]):
            maximum = max(float(np.max(np.abs(values))), 1e-12)
            kwargs = (
                dict(cmap="coolwarm", vmin=-maximum, vmax=maximum)
                if j == 2
                else dict(
                    cmap="magma",
                    vmin=0,
                    vmax=max(float(f.max()), float(g.max()), 1e-12),
                )
            )
            im = axes[i, j].imshow(
                values,
                origin="lower",
                extent=(0, 1, 0, 1),
                interpolation="nearest",
                **kwargs,
            )
            axes[i, j].set(
                title=[
                    "f (current integrand)",
                    "g (frozen approximation)",
                    "signed residual f−g",
                ][j]
                if i == 0
                else "",
                xlabel="u",
                ylabel=case.replace("_", " ") + "\nv" if j == 0 else "v",
            )
            fig.colorbar(im, ax=axes[i, j], shrink=0.7)
    fig.suptitle(
        "Directional integrands in the cosine-sampling domain; luminance, not native rendered images\n"
        "64² display samples can miss the narrow emitter; analytic truth and random evaluation still include it"
    )
    fig.savefig(work / "integrands.png", dpi=120, bbox_inches="tight")
    plt.close(fig)


def review(work, report):
    grid = (
        8
        if 8 in report["config"]["cache_grids"]
        else report["config"]["cache_grids"][0]
    )
    n = min(report["config"]["samples"], key=lambda x: abs(x - 256))
    cells = []
    for case in CASES:
        for method in METHODS:
            r = next(
                r
                for r in report["estimates"]
                if r["case"] == case
                and r["grid"] == grid
                and r["comparison_n"] == n
                and r["method"] == method
            )
            timing = (
                f"{r['timing']['median_ns'] / 1000:.3f}"
                if "timing" in r
                else "not timed (G lookup)"
            )
            gain = (
                f"{r['mse_gain_over_plain']:.3g}×"
                if "mse_gain_over_plain" in r
                else "—"
            )
            cells.append(
                f"<tr><td>{case}</td><td>{html.escape(LABELS[method])}</td><td>{r['mse_rgb_mean']:.5g}</td><td>{gain}</td><td>{timing}</td></tr>"
            )
    page = f"""<!doctype html><html lang="en"><meta charset="utf-8"><title>Lumens estimator experiment</title>
<style>body{{font:16px system-ui;max-width:1200px;margin:40px auto;padding:0 24px;color:#172033;background:#fafafa}}img{{max-width:100%;background:white}}table{{border-collapse:collapse;font-size:14px}}td,th{{padding:8px;border-bottom:1px solid #ddd;text-align:left}}code{{background:#eee;padding:2px 5px}}</style>
<h1>Lumens: approximate light + signed residual</h1>
<p>Independent CPU numerical experiment. No native GPU, multi-bounce GI or Lumen performance claim.</p>
<p>{report["config"]["trials"]} trials; samples {report["config"]["samples"]}; cache grids {report["config"]["cache_grids"]}; seed {report["config"]["seed"]}.</p>
<p>Plain MC evaluates f. Cache-only integrates g. Exact-G correction adds mean(f−g). Two-level independently estimates g with 4n cheap samples and uses n residual samples.</p>
<p>Cache construction is separate. Timings include input generation into reusable scratch; allocation is excluded from warm timings. The JSON reports input generation separately and estimates cold/amortized costs by adding separately measured medians. Cache setup timings include allocation and destruction.</p>
<img src="convergence.svg" alt="Error convergence across six controlled cases">
<img src="quality-cost.svg" alt="Error against measured warm CPU time">
<h2>Selected budget: n={n}, {grid}×{grid} cache</h2>
<p>Gain = plain MSE / method MSE at equal expensive sample count; values below one are regressions. Cache-only does zero expensive evaluation after setup.</p>
<table><thead><tr><th>Case</th><th>Method</th><th>Mean RGB MSE</th><th>MSE gain</th><th>Warm CPU μs</th></tr></thead><tbody>{"".join(cells)}</tbody></table>
<h2>Directional fields and failure cases</h2><img src="integrands.png" alt="Current integrands, frozen cache and signed residuals">
<p>Refinement differences are diagnostics, not rigorous reference error bounds. Rare events can invalidate normal confidence coverage; observed compatibility with zero does not prove unbiasedness. Timing p95 describes block averages, not per-call tail latency.</p>
<p><a href="results.json">Results and uncertainty</a> · <a href="manifest.json">Provenance/checksums</a> · <a href="trials.csv">Raw independent trials</a> · <a href="timings.csv">Raw timing blocks</a> · <a href="references.csv">Reference refinements</a> · <a href="caches.csv">Cache integrals/payload</a></p></html>"""
    (work / "review.html").write_text(page)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--trials", type=int, default=256)
    parser.add_argument("--seed", type=int, default=20261104)
    parser.add_argument("--samples", type=int_list, default=[16, 64, 256, 1024])
    parser.add_argument("--cache-grids", type=int_list, default=[4, 8, 16])
    parser.add_argument("--reference-grid", type=int, default=2048)
    parser.add_argument("--timing-blocks", type=int, default=9)
    parser.add_argument("--bootstrap-replicates", type=int, default=1000)
    parser.add_argument("--verify-reproducibility", action="store_true")
    args = parser.parse_args()
    if not (
        32 <= args.trials <= 2048
        and 0 <= args.seed < 2**64
        and all(1 <= n <= 4096 for n in args.samples)
        and all(1 <= g <= 64 for g in args.cache_grids)
        and 128 <= args.reference_grid <= 2048
        and args.reference_grid & (args.reference_grid - 1) == 0
        and 5 <= args.timing_blocks <= 31
        and 100 <= args.bootstrap_replicates <= 2000
        and args.trials * sum(args.samples) * len(args.cache_grids) * 6 <= 100_000_000
    ):
        parser.error("configuration outside bounded experiment limits")
    config = vars(args)
    OUTPUT.mkdir(parents=True, exist_ok=True)
    work = OUTPUT / ".pending"
    # Creation is also an exclusive run lock. Never remove another run's stage.
    work.mkdir()
    started = time.perf_counter()
    try:
        compiler = run(["rustc", "--version"])
        compile_flags = ["--edition=2024", "-O", "-D", "warnings"]
        run(["rustc", *compile_flags, str(SOURCE), "-o", str(work / "estimator")])
        verification = run([str(work / "estimator"), "--verify"])
        print(verification, flush=True)
        command = [
            str(work / "estimator"),
            str(work),
            str(args.trials),
            str(args.seed),
            ",".join(map(str, args.samples)),
            ",".join(map(str, args.cache_grids)),
            str(args.reference_grid),
            str(args.timing_blocks),
        ]
        run(command)
        report = summarize(work, config)
        # Scientific red/green controls: these check inference risks rather
        # than mirroring implementation details or demanding random CI passes.
        selected = [
            r for r in report["estimates"] if r["comparison_n"] == args.samples[-1]
        ]
        for r in selected:
            if r["case"] == "missed_emitter" and r["method"] == "corrected_exact":
                assert abs(r["mse_gain_over_plain"] - 1.0) < 1e-8, (
                    "missed constant cache fabricated variance reduction"
                )
        report["verification"] = verification
        write_json(work / "results.json", report)
        plot(work, report)
        review(work, report)
        numerical_digest = hashlib.sha256(
            b"".join(
                (work / n).read_bytes()
                for n in ["references.csv", "caches.csv", "trials.csv", "maps.csv"]
            )
        ).hexdigest()
        reproduction = "not requested"
        if args.verify_reproducibility:
            replay = work / "replay"
            replay.mkdir()
            repeated_command = command.copy()
            repeated_command[1] = str(replay)
            run(repeated_command)
            replay_digest = hashlib.sha256(
                b"".join(
                    (replay / n).read_bytes()
                    for n in ["references.csv", "caches.csv", "trials.csv", "maps.csv"]
                )
            ).hexdigest()
            assert replay_digest == numerical_digest, (
                "numerical replay differs with identical inputs"
            )
            reproduction = "all numerical CSVs byte-identical; timing measurements deliberately excluded"
            shutil.rmtree(replay)
        manifest = dict(
            schema=1,
            config=config,
            numerical_sha256=numerical_digest,
            source_sha256={
                str(p.relative_to(ROOT)): digest(p) for p in [SOURCE, Path(__file__)]
            },
            provenance=dict(
                rustc=compiler,
                rustc_flags=compile_flags,
                cpu=run(["sysctl", "-n", "machdep.cpu.brand_string"])
                if sys.platform == "darwin"
                else platform.processor(),
                python=sys.version,
                numpy=np.__version__,
                matplotlib=matplotlib.__version__,
                platform=platform.platform(),
                machine=platform.machine(),
                processor=platform.processor(),
                git_head_at_run=run(["git", "rev-parse", "HEAD"], cwd=ROOT),
                command=[
                    "uv",
                    "run",
                    "scripts/run_lighting_estimator.py",
                    *sys.argv[1:],
                ],
                timing_processes_note="no competing-workload exclusion or hardware counter isolation enforced",
            ),
            elapsed_seconds=time.perf_counter() - started,
            reproducibility=reproduction,
            outputs={p.name: digest(p) for p in work.iterdir() if p.is_file()},
            publication="files replaced individually, manifest last; consumers must verify all checksums",
        )
        write_json(work / "manifest.json", manifest)
        for p in sorted(work.iterdir(), key=lambda p: p.name == "manifest.json"):
            os.replace(p, OUTPUT / p.name)
        print(
            json.dumps(
                dict(
                    report=str(OUTPUT / "review.html"),
                    numerical_sha256=numerical_digest,
                    elapsed_seconds=manifest["elapsed_seconds"],
                )
            ),
            flush=True,
        )
    finally:
        shutil.rmtree(work)


if __name__ == "__main__":
    main()
