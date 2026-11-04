"""Sequence-cluster statistics for shared lighting decision lifetime."""

from collections import defaultdict
import csv
import json

import numpy as np
import matplotlib.pyplot as plt

POLICIES = ["plain", "per_query", "periodic", "notified", "watched"]
SCENARIOS = ["stable", "geometry_step", "geometry_pulse", "lighting_step"]
LABELS = {
    "plain": "Plain MC",
    "per_query": "Fit each query",
    "periodic": "Shared TTL",
    "notified": "Shared + supplied revision",
    "watched": "Shared + paired sentinels",
}
COLORS = dict(zip(POLICIES, ["#475569", "#9333ea", "#d97706", "#0284c7", "#059669"]))


def read(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def build(work, config, baseline):
    trials = config["epoch_trials_effective"]
    steps = config["epoch_steps"]
    traces = read(work / "epoch_trace.csv")
    assert len(traces) == 4 * 5 * min(8, trials) * steps
    for trace in traces:
        budget = int(trace["budget"])
        counts = [
            int(trace[k]) for k in ["train", "checks", "snapshots", "evaluations"]
        ]
        assert sum(counts) == budget and counts[-1] > 0 and min(counts) >= 0
        assert 0 <= float(trace["alpha"]) <= 2
        assert np.isfinite([float(trace[c]) for c in "rgb"]).all()
        if trace["method"] == "notified":
            assert trace["target_revision"] == trace["trained_revision"]
        if trace["method"] != "watched" or int(trace["horizon"]) <= 4:
            assert int(trace["checks"]) == int(trace["snapshots"]) == 0
        allowed = {
            "plain": {0},
            "per_query": {1, 2},
            "periodic": {0, 1, 3},
            "notified": {0, 1, 3, 4},
            "watched": {0, 1, 3, 5},
        }
        assert int(trace["reason"]) in allowed[trace["method"]]
    groups = defaultdict(list)
    for r in read(work / "epoch_trials.csv"):
        groups[r["scenario"], int(r["budget"]), int(r["horizon"]), r["method"]].append(
            r
        )
    times = defaultdict(list)
    for r in read(work / "epoch_timings.csv"):
        times[r["scenario"], int(r["budget"]), int(r["horizon"]), r["method"]].append(
            float(r["ns_per_sequence"])
        )
    indices = np.random.default_rng(config["seed"] ^ 0x3C6EF372).integers(
        0, trials, (config["bootstrap_replicates"], trials)
    )
    rows = []
    setup = {
        (r["case"], r["grid"]): r
        for r in baseline["estimates"]
        if r["method"] == "corrected_exact"
    }
    for scenario in SCENARIOS:
        for budget in config["epoch_budgets"]:
            for horizon in config["epoch_horizons"]:
                selected = {m: groups[scenario, budget, horizon, m] for m in POLICIES}
                for m, records in selected.items():
                    assert [int(r["trial"]) for r in records] == list(range(trials))
                    for r in records:
                        assert (
                            sum(
                                int(r[c])
                                for c in [
                                    "train_calls",
                                    "check_calls",
                                    "snapshot_calls",
                                    "evaluation_calls",
                                ]
                            )
                            == budget * steps
                        )
                errors = {
                    m: np.array(
                        [[float(r[f"error_{c}"]) for c in "rgb"] for r in records]
                    )
                    for m, records in selected.items()
                }
                mse = {
                    m: np.array(
                        [sum(float(r[f"mse_{c}"]) for c in "rgb") / 3 for r in records]
                    )
                    for m, records in selected.items()
                }
                boot = {m: e[indices].mean(axis=1) for m, e in mse.items()}
                if horizon == 1:
                    for m in ["periodic", "notified", "watched"]:
                        assert np.array_equal(mse[m], mse["per_query"]), (
                            "H=1 must preserve per-query estimates exactly"
                        )
                for method in POLICIES:
                    records = selected[method]
                    latency = {}
                    for event in [1, 2]:
                        applicable = (
                            scenario != "stable"
                            and method != "plain"
                            and (event == 1 or scenario == "geometry_pulse")
                        )
                        values = np.array(
                            [int(r[f"event{event}_latency"]) for r in records]
                        )
                        detected = values[values >= 0]
                        latency[str(event)] = dict(
                            applicable=applicable,
                            missed_fraction=float(
                                np.mean(
                                    [int(r[f"event{event}_missed"]) for r in records]
                                )
                            )
                            if applicable
                            else None,
                            conditional_latency_quantiles=np.percentile(
                                detected, [0, 50, 95, 100]
                            ).tolist()
                            if detected.size
                            else None,
                            note="first retraining within the event's active phase; TTL response is not necessarily detection",
                        )
                    blocks = times[scenario, budget, horizon, method]
                    gain = boot["plain"] / np.maximum(boot[method], 1e-300)
                    counts = {
                        c: float(np.mean([int(r[c]) for r in records])) / steps
                        for c in [
                            "train_calls",
                            "check_calls",
                            "snapshot_calls",
                            "evaluation_calls",
                            "refreshes",
                            "stale_queries",
                        ]
                    }
                    window_results = {}
                    for name in ["early", "middle", "late"]:
                        a = np.array([float(r[f"{name}_mse"]) for r in records])
                        plain = np.array(
                            [float(r[f"{name}_mse"]) for r in selected["plain"]]
                        )
                        window_results[name] = dict(
                            mse=float(a.mean()),
                            mse_gain_over_plain=float(
                                plain.mean() / max(a.mean(), 1e-300)
                            ),
                            gain_ci95_cluster_bootstrap=np.percentile(
                                plain[indices].mean(axis=1)
                                / np.maximum(a[indices].mean(axis=1), 1e-300),
                                [2.5, 97.5],
                            ).tolist(),
                        )
                    assert (
                        np.isfinite(mse[method]).all()
                        and np.isfinite(errors[method]).all()
                    )
                    source_case = (
                        "hemisphere_open"
                        if scenario == "lighting_step"
                        else "hemisphere_occluded"
                    )
                    cold = setup[source_case, config["epoch_grid"]]
                    rows.append(
                        dict(
                            scenario=scenario,
                            budget=budget,
                            horizon=horizon,
                            method=method,
                            independent_sequences=trials,
                            queries_per_sequence=steps,
                            mse=float(mse[method].mean()),
                            mse_ci95_cluster_bootstrap=np.percentile(
                                boot[method], [2.5, 97.5]
                            ).tolist(),
                            gain_over_plain=float(
                                mse["plain"].mean() / max(mse[method].mean(), 1e-300)
                            ),
                            gain_ci95_cluster_bootstrap=np.percentile(
                                gain, [2.5, 97.5]
                            ).tolist(),
                            gain_over_per_query=float(
                                mse["per_query"].mean()
                                / max(mse[method].mean(), 1e-300)
                            ),
                            gain_over_per_query_ci95=np.percentile(
                                boot["per_query"] / np.maximum(boot[method], 1e-300),
                                [2.5, 97.5],
                            ).tolist(),
                            mean_sequence_error_rgb=errors[method]
                            .mean(axis=0)
                            .tolist(),
                            counts_per_query=counts,
                            cache_source_calls=config["epoch_grid"] ** 2
                            if method != "plain"
                            else 0,
                            cache_payload_bytes=cold["cache_payload_bytes"]
                            if method != "plain"
                            else 0,
                            cache_setup_median_ns=cold["cache_build_timing"][
                                "median_ns"
                            ]
                            if method != "plain"
                            else 0.0,
                            cold_cost_note="source cache creation excluded from query budget; setup is a separately measured median, not a directly timed cold sequence",
                            event_responses=latency,
                            windows=window_results,
                            timing=dict(
                                median_ns_per_sequence=float(np.median(blocks)),
                                median_ns_per_query=float(np.median(blocks)) / steps,
                                p95_block_average_ns_per_sequence=float(
                                    np.percentile(blocks, 95)
                                ),
                                block_average_ns=blocks,
                            ),
                        )
                    )
    result = dict(
        schema=1,
        config=config,
        estimates=rows,
        references=baseline["references"],
        uncertainty="bootstrap complete independent sequences; queries within one sequence are not independent replicates",
        windows=dict(early=[0, 17], middle=[17, 29], late=[29, steps]),
        notification="ideal supplied global scene revision; not inferred from illumination, not an implemented engine dependency graph",
        monitor="paired fixed directions, checked every four queries; finite coverage can miss a localized change until TTL",
        scope="one receiver, one fixed source cache; shared weights only; no temporal image accumulation or world-space sharing",
        trace_verification="selected trace count, call budgets, finite values/weights, notified revisions, monitor ownership and refresh reasons checked",
        limitations=baseline["limitations"]
        + [
            "MSE over an entire changing sequence can hide a dim phase; inspect each window and query curve",
            "sentinel comparison uses deterministic single-query transport; noisy shading/change detection not validated",
            "CPU sequence timings include preallocated output records; no reference/error/report IO is timed",
        ],
    )
    (work / "epoch-results.json").write_text(
        json.dumps(result, indent=2, sort_keys=True, allow_nan=False) + "\n"
    )
    plot(work, result)
    review(work, result)
    return result


def plot(work, report):
    budget = (
        256
        if 256 in report["config"]["epoch_budgets"]
        else report["config"]["epoch_budgets"][-1]
    )
    fig, axes = plt.subplots(2, 2, figsize=(12, 9))
    fig.subplots_adjust(
        left=0.09, right=0.98, bottom=0.17, top=0.85, wspace=0.25, hspace=0.37
    )
    for ax, scenario in zip(axes.flat, SCENARIOS):
        for method in POLICIES[1:]:
            selected = [
                r
                for r in report["estimates"]
                if r["scenario"] == scenario
                and r["budget"] == budget
                and r["method"] == method
            ]
            selected.sort(key=lambda r: r["horizon"])
            x = [r["horizon"] for r in selected]
            y = [r["gain_over_plain"] for r in selected]
            ci = np.array([r["gain_ci95_cluster_bootstrap"] for r in selected])
            ax.plot(x, y, "o-", color=COLORS[method], label=LABELS[method])
            ax.fill_between(x, ci[:, 0], ci[:, 1], color=COLORS[method], alpha=0.12)
        ax.axhline(1, color=COLORS["plain"], linestyle=":", label="Plain MC parity")
        ax.set(
            xscale="log",
            yscale="log",
            title=scenario.replace("_", " "),
            xlabel="Decision lifetime (queries)",
            ylabel="Plain sequence MSE / policy MSE",
        )
        ax.grid(alpha=0.2, which="both")
    handles, labels = axes.flat[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="lower center", bbox_to_anchor=(0.5, 0.01), ncols=2)
    fig.suptitle(
        f"Shared lighting decisions — total budget {budget} target calls per query\n95% intervals resample independent sequences",
        y=0.98,
        fontsize=14,
    )
    fig.savefig(work / "epoch-lifetime.svg", bbox_inches="tight")
    fig.savefig(work / "epoch-lifetime.png", dpi=135, bbox_inches="tight")
    plt.close(fig)

    horizons = report["config"]["epoch_horizons"]
    horizon = max(horizons)
    curves = defaultdict(list)
    for r in read(work / "epoch_curves.csv"):
        if int(r["budget"]) == budget and int(r["horizon"]) == horizon:
            curves[r["scenario"], r["method"]].append(r)
    fig, axes = plt.subplots(2, 2, figsize=(12, 9))
    fig.subplots_adjust(
        left=0.09, right=0.98, bottom=0.17, top=0.85, wspace=0.25, hspace=0.37
    )
    for ax, scenario in zip(axes.flat, SCENARIOS):
        baseline = np.array(
            [
                sum(float(r[f"mse_{c}"]) for c in "rgb") / 3
                for r in curves[scenario, "plain"]
            ]
        )
        for method in POLICIES[1:]:
            selected = curves[scenario, method]
            x = [int(r["step"]) for r in selected]
            error = np.array(
                [sum(float(r[f"mse_{c}"]) for c in "rgb") / 3 for r in selected]
            )
            ax.plot(
                x,
                baseline / np.maximum(error, 1e-300),
                color=COLORS[method],
                label=LABELS[method],
            )
        if scenario != "stable":
            ax.axvline(17, color="#dc2626", linestyle="--", alpha=0.6)
        if scenario == "geometry_pulse":
            ax.axvline(29, color="#dc2626", linestyle="--", alpha=0.6)
        ax.axhline(1, color=COLORS["plain"], linestyle=":", label="Plain MC parity")
        ax.set(
            yscale="log",
            title=scenario.replace("_", " "),
            xlabel="Query index (not game frames)",
            ylabel="Plain query MSE / policy query MSE",
        )
        ax.grid(alpha=0.2, which="both")
    handles, labels = axes.flat[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="lower center", bbox_to_anchor=(0.5, 0.01), ncols=2)
    fig.suptitle(
        f"Response to off-boundary changes — lifetime {horizon}, budget {budget}\nPer-query means across independent sequences; red lines mark supplied changes",
        y=0.98,
        fontsize=14,
    )
    fig.savefig(work / "epoch-response.svg", bbox_inches="tight")
    fig.savefig(work / "epoch-response.png", dpi=135, bbox_inches="tight")
    plt.close(fig)


def review(work, report):
    config = report["config"]
    budget = 256 if 256 in config["epoch_budgets"] else config["epoch_budgets"][-1]
    horizon = max(config["epoch_horizons"])
    cells = []
    for scenario in SCENARIOS:
        for method in POLICIES:
            r = next(
                r
                for r in report["estimates"]
                if (r["scenario"], r["budget"], r["horizon"], r["method"])
                == (scenario, budget, horizon, method)
            )
            lo, hi = r["gain_ci95_cluster_bootstrap"]
            e = r["event_responses"]["1"]
            delay = e["conditional_latency_quantiles"]
            lag = (
                "n/a"
                if not e["applicable"]
                else (f"{delay[1]:.1f}" if delay else "missed")
            )
            counts = r["counts_per_query"]
            cells.append(
                f"<tr><td>{scenario}</td><td>{LABELS[method]}</td><td>{r['gain_over_plain']:.3g} [{lo:.3g},{hi:.3g}]</td>"
                f"<td>{r['windows']['middle']['mse_gain_over_plain']:.3g}</td><td>{counts['train_calls']:.2f}/{counts['check_calls'] + counts['snapshot_calls']:.2f}</td>"
                f"<td>{lag}</td><td>{e['missed_fraction'] if e['applicable'] else 'n/a'}</td><td>{r['timing']['median_ns_per_query'] / 1000:.3f}</td></tr>"
            )
    page = f"""<!doctype html><html lang="en"><meta charset="utf-8"><title>Lumens: shared-decision lifetime</title>
<style>body{{font:16px system-ui;max-width:1300px;margin:40px auto;padding:0 24px;color:#172033;background:#fafafa}}img{{max-width:100%}}table{{border-collapse:collapse;font-size:13px}}td,th{{padding:8px;border-bottom:1px solid #ddd;text-align:left}}</style>
<h1>Amortizing the decision without hiding stale-state cost</h1>
<p>{config["epoch_trials_effective"]} independent sequences, {config["epoch_steps"]} queries each, cache {config["epoch_grid"]}×{config["epoch_grid"]}. Every query pays B calls including training, snapshots and checks. Y remains fresh and independent.</p>
<p>Shared means the scalar coefficient is reused at one receiver. No world-space cache, denoising, engine dependency tracking or GPU integration is demonstrated.</p>
<p>Notifications are explicitly supplied by the scripted environment. Sentinels compare a finite set of fixed directions every four queries and can miss local changes. Changes at17/29 do not align with TTL8/32.</p>
<img src="epoch-lifetime.svg" alt="Sequence error versus shared decision lifetime">
<img src="epoch-response.svg" alt="Per-query response to geometry and light changes">
<h2>Selected lifetime {horizon}, budget {budget}</h2><p>Gain = plain MSE / policy MSE; below one is worse. Intervals resample whole sequences. Middle window is queries17..28. Delay is conditional on refreshing before the event ended; misses are shown separately.</p>
<table><thead><tr><th>Sequence</th><th>Policy</th><th>Whole gain [95% CI]</th><th>Middle gain</th><th>Train/watch calls per query</th><th>Median response lag</th><th>Event1 misses</th><th>CPU μs/query</th></tr></thead><tbody>{"".join(cells)}</tbody></table>
<p>A whole-sequence MSE can conceal poor relative accuracy during dim lighting. Inspect middle/late windows and the time curve. These are CPU measurements of repeated estimates, not game frames or multi-bounce GI.</p>
<p><a href="epoch-results.json">Statistics and limitations</a> · <a href="epoch_trials.csv">Every sequence summary</a> · <a href="epoch_curves.csv">Population query curves</a> · <a href="epoch_trace.csv">Selected raw traces</a> · <a href="epoch_timings.csv">Timing blocks</a> · <a href="manifest.json">Provenance</a> · <a href="pilot-review.html">Previous policies</a></p></html>"""
    (work / "epoch-review.html").write_text(page)
    for name in ["review.html", "pilot-review.html"]:
        text = (work / name).read_text()
        (work / name).write_text(
            text.replace(
                "<h1>",
                '<p><a href="epoch-review.html">Stage 3: shared-decision lifetime and change response</a></p><h1>',
                1,
            )
        )
