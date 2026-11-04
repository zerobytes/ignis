"""Statistics and standalone plots for independent-pilot lighting policies."""

from collections import Counter, defaultdict
import csv
import html
import json

import numpy as np
import matplotlib.pyplot as plt

METHODS = ["plain", "fixed", "learned", "gated"]
LABELS = {
    "plain": "Plain MC",
    "fixed": "Fixed weight 1",
    "learned": "Independent learned weight",
    "gated": "Independent validated gate",
}
COLORS = dict(zip(METHODS, ["#475569", "#0284c7", "#9333ea", "#059669"]))


def read(path):
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))


def build(work, config, baseline, analytic_moments):
    groups = defaultdict(list)
    for row in read(work / "pilot_trials.csv"):
        groups[row["case"], int(row["grid"]), int(row["budget"]), row["method"]].append(
            row
        )
    times = defaultdict(list)
    for r in read(work / "pilot_timings.csv"):
        times[
            r["case"], int(r["grid"]), int(r["budget"]), r["method"], r["phase"]
        ].append(float(r["ns_per_estimate"]))
    indices = np.random.default_rng(config["seed"] ^ 0xBB67AE85).integers(
        0, config["trials"], (config["bootstrap_replicates"], config["trials"])
    )
    cases = list(baseline["references"])
    theory = {
        (case, grid): analytic_moments(case, grid)
        for case in cases
        for grid in config["cache_grids"]
    }
    setup = {
        (r["case"], r["grid"]): r
        for r in baseline["estimates"]
        if r["method"] == "corrected_exact"
    }
    rows = []
    for case in cases:
        truth = np.array(baseline["references"][case]["rgb"])
        for grid in config["cache_grids"]:
            for budget in config["samples"]:
                entries = {m: groups[case, grid, budget, m] for m in METHODS}
                assert all(
                    [int(r["trial"]) for r in entries[m]]
                    == list(range(config["trials"]))
                    for m in METHODS
                )
                values = {
                    m: np.array([[float(r[c]) for c in "rgb"] for r in entries[m]])
                    for m in METHODS
                }
                squared = {
                    m: np.mean((value - truth) ** 2, axis=1)
                    for m, value in values.items()
                }
                bootstrap = {m: v[indices].mean(axis=1) for m, v in squared.items()}
                for method in METHODS:
                    data = entries[method]
                    alpha = np.array([float(r["alpha"]) for r in data])
                    counts = {
                        field: np.array([int(r[field]) for r in data])
                        for field in ["train", "validation", "evaluation"]
                    }
                    assert np.all(sum(counts.values()) == budget)
                    assert np.all(counts["evaluation"] > 0) and np.all(
                        (alpha >= 0) & (alpha <= 2)
                    )
                    assert np.isfinite(values[method]).all()
                    timing = {
                        phase: dict(
                            median_ns=float(
                                np.median(times[case, grid, budget, method, phase])
                            ),
                            p95_block_average_ns=float(
                                np.percentile(
                                    times[case, grid, budget, method, phase], 95
                                )
                            ),
                            block_average_ns=times[case, grid, budget, method, phase],
                        )
                        for phase in ["decision", "total"]
                    }
                    ratio = bootstrap["plain"] / np.maximum(bootstrap[method], 1e-300)
                    error = values[method] - truth
                    variance = values[method].var(axis=0, ddof=1)
                    bias = error.mean(axis=0)
                    sem = np.sqrt(variance / config["trials"])
                    row = dict(
                        case=case,
                        grid=grid,
                        budget=budget,
                        method=method,
                        mean_rgb=values[method].mean(axis=0).tolist(),
                        mean_error_rgb=bias.tolist(),
                        mean_error_ci95_normal_rgb=np.stack(
                            [bias - 1.96 * sem, bias + 1.96 * sem], axis=1
                        ).tolist(),
                        variance_rgb=variance.tolist(),
                        mse_rgb_mean=float(squared[method].mean()),
                        mse_ci95_bootstrap=np.percentile(
                            bootstrap[method], [2.5, 97.5]
                        ).tolist(),
                        mse_gain_over_plain=float(
                            squared["plain"].mean()
                            / max(squared[method].mean(), 1e-300)
                        ),
                        gain_ci95_paired_bootstrap=np.percentile(
                            ratio, [2.5, 97.5]
                        ).tolist(),
                        mse_gain_over_fixed=float(
                            squared["fixed"].mean()
                            / max(squared[method].mean(), 1e-300)
                        ),
                        gain_over_fixed_ci95_paired_bootstrap=np.percentile(
                            bootstrap["fixed"] / np.maximum(bootstrap[method], 1e-300),
                            [2.5, 97.5],
                        ).tolist(),
                        alpha_quantiles=np.percentile(
                            alpha, [0, 25, 50, 75, 100]
                        ).tolist(),
                        cache_used_fraction=float(np.mean(alpha > 0)),
                        decision_reasons=dict(Counter(r["reason"] for r in data)),
                        mean_counts={
                            name: float(v.mean()) for name, v in counts.items()
                        },
                        timing=timing,
                        warm_mse_times_ns=float(squared[method].mean())
                        * timing["total"]["median_ns"],
                        reference_exact=baseline["references"][case]["exact"],
                        reference_last_refinement_rms=baseline["references"][case][
                            "last_refinement_rms"
                        ],
                        allocated_input_payload_bytes=(
                            budget + 2 * min(config["pilot_samples"], budget // 4)
                        )
                        * 16,
                    )
                    build_cost = (
                        setup[case, grid]["cache_build_timing"]["median_ns"]
                        if method != "plain"
                        else 0.0
                    )
                    row.update(
                        cache_payload_bytes=setup[case, grid]["cache_payload_bytes"]
                        if method != "plain"
                        else 0,
                        cache_source_calls=grid * grid if method != "plain" else 0,
                        cache_setup_median_ns=build_cost,
                        cold_cost_estimate_ns=timing["total"]["median_ns"] + build_cost,
                        setup_note="setup allocation/destruction and target evaluation measured separately; warm budget excludes cache creation",
                    )
                    moments = theory[case, grid]
                    if moments:
                        vf = np.array([c["variance_f"] for c in moments])
                        vg = np.array([c["variance_g"] for c in moments])
                        covariance = (
                            vf
                            + vg
                            - np.array([c["variance_residual"] for c in moments])
                        ) / 2
                        conditional_variance = (
                            vf[None, :]
                            - 2 * alpha[:, None] * covariance
                            + alpha[:, None] ** 2 * vg
                        ) / counts["evaluation"][:, None]
                        assert np.all(conditional_variance > -1e-12)
                        row["conditional_analytic_theory"] = dict(
                            mse_averaged_over_observed_pilots=float(
                                conditional_variance.mean()
                            ),
                            expected_mean_rgb=[c["mean"] for c in moments],
                            note="exact rational moments, averaged over observed pilot outcomes; final Y is not used",
                        )
                    if case == "missed_emitter" and method in ["learned", "gated"]:
                        assert (
                            np.all(alpha == 0)
                            and np.all(counts["train"] == 0)
                            and np.all(counts["validation"] == 0)
                        )
                        assert np.array_equal(values[method], values["plain"]), (
                            "constant-cache bypass must preserve plain MC exactly"
                        )
                    rows.append(row)
    result = dict(
        schema=1,
        config=config,
        estimates=rows,
        policy=dict(
            alpha_range=[0, 2],
            validation_margin=0.9,
            p="min(requested pilot samples, B//4)",
            acceptance="validation residual variance / plain variance < 0.9 * remaining_evaluation_count / B",
            warning="heuristic, no dominance guarantee; failed pilots consume budget and are not recycled",
        ),
        comparison="equal total expensive f calls, including pilot calls; not equal CPU time",
        estimator="alpha*G + mean_Y(f-alpha*g); alpha/counts selected without Y",
        limitations=baseline["limitations"]
        + [
            "no cache invalidation implementation; trust is evaluated only for these frozen cases",
            "scalar RGB trace-variance weights may not optimize each color channel",
            "per-estimate fresh pilots; no amortization across receivers or frames is established",
            "normal confidence intervals can under-cover rare events; bootstrap is not a performance guarantee",
        ],
    )
    (work / "pilot-results.json").write_text(
        json.dumps(result, indent=2, sort_keys=True, allow_nan=False) + "\n"
    )
    plot(work, result, cases)
    review(work, result, cases)
    return result


def plot(work, report, cases):
    grid = (
        8
        if 8 in report["config"]["cache_grids"]
        else report["config"]["cache_grids"][0]
    )
    for kind in ["pilot-gain", "pilot-quality-cost"]:
        fig, axes = plt.subplots(2, 3, figsize=(15, 9))
        fig.subplots_adjust(
            left=0.07, right=0.99, bottom=0.17, top=0.85, wspace=0.28, hspace=0.4
        )
        for ax, case in zip(axes.flat, cases):
            limits = []
            for method in METHODS:
                selected = [
                    r
                    for r in report["estimates"]
                    if r["case"] == case and r["grid"] == grid and r["method"] == method
                ]
                if kind == "pilot-gain" and method == "plain":
                    continue
                selected.sort(key=lambda r: r["budget"])
                x = [
                    r["budget"]
                    if kind == "pilot-gain"
                    else r["timing"]["total"]["median_ns"] / 1000
                    for r in selected
                ]
                y = [
                    r["mse_gain_over_plain"]
                    if kind == "pilot-gain"
                    else r["mse_rgb_mean"]
                    for r in selected
                ]
                ci = np.array(
                    [
                        r["gain_ci95_paired_bootstrap"]
                        if kind == "pilot-gain"
                        else r["mse_ci95_bootstrap"]
                        for r in selected
                    ]
                )
                limits.extend(ci.ravel().tolist())
                ax.plot(
                    x, y, "o-", color=COLORS[method], label=LABELS[method], markersize=4
                )
                ax.fill_between(
                    x,
                    np.maximum(ci[:, 0], 1e-30),
                    np.maximum(ci[:, 1], 1e-30),
                    color=COLORS[method],
                    alpha=0.12,
                )
            if kind == "pilot-gain":
                ax.axhline(1, color="#475569", linestyle=":", label="Plain MC parity")
            ax.set(
                xscale="log",
                yscale="log",
                title=case.replace("_", " "),
                xlabel="Total expensive calls, including pilots"
                if kind == "pilot-gain"
                else "Warm CPU microseconds / full policy",
                ylabel="Plain MSE / policy MSE (higher is better)"
                if kind == "pilot-gain"
                else "Mean RGB squared error",
            )
            # Do not visually amplify roundoff around exact MC parity.
            if kind == "pilot-gain" and min(limits) > 0.9 and max(limits) < 1.1:
                ax.set_ylim(0.5, 2.0)
            ax.grid(alpha=0.2, which="both")
        handles, labels = axes.flat[0].get_legend_handles_labels()
        fig.legend(
            handles, labels, loc="lower center", bbox_to_anchor=(0.5, 0.01), ncols=2
        )
        fig.suptitle(
            f"Independent-pilot cache trust — {report['config']['trials']} trials, {grid}×{grid} cache\n"
            + (
                "Same total f-call budget; shaded bands: paired bootstrap 95% intervals"
                if kind == "pilot-gain"
                else "Actual policy CPU cost includes pilot generation, fitting, validation and evaluation"
            ),
            y=0.98,
            fontsize=14,
        )
        fig.savefig(work / f"{kind}.svg", bbox_inches="tight")
        fig.savefig(work / f"{kind}.png", dpi=135, bbox_inches="tight")
        plt.close(fig)


def review(work, report, cases):
    grid = (
        8
        if 8 in report["config"]["cache_grids"]
        else report["config"]["cache_grids"][0]
    )
    budget = min(report["config"]["samples"], key=lambda n: abs(n - 256))
    cells = []
    for case in cases:
        for method in METHODS:
            r = next(
                r
                for r in report["estimates"]
                if r["case"] == case
                and r["grid"] == grid
                and r["budget"] == budget
                and r["method"] == method
            )
            lo, hi = r["gain_ci95_paired_bootstrap"]
            counts = r["mean_counts"]
            cells.append(
                f"<tr><td>{case}</td><td>{html.escape(LABELS[method])}</td><td>{r['mse_gain_over_plain']:.3g} [{lo:.3g}, {hi:.3g}]</td>"
                f"<td>{r['alpha_quantiles'][2]:.4g}</td><td>{r['cache_used_fraction']:.0%}</td>"
                f"<td>{counts['train']:.1f}/{counts['validation']:.1f}/{counts['evaluation']:.1f}</td>"
                f"<td>{r['timing']['decision']['median_ns'] / 1000:.3f}</td><td>{r['timing']['total']['median_ns'] / 1000:.3f}</td></tr>"
            )
    page = f"""<!doctype html><html lang="en"><meta charset="utf-8"><title>Lumens: independent-pilot cache trust</title>
<style>body{{font:16px system-ui;max-width:1400px;margin:40px auto;padding:0 24px;background:#fafafa;color:#172033}}img{{max-width:100%}}td,th{{padding:8px;border-bottom:1px solid #ddd;text-align:left;font-size:13px}}table{{border-collapse:collapse}}</style>
<h1>Choosing how much to trust a frozen light cache</h1>
<p>Same total expensive evaluation budget B for every policy, including pilot samples. No GPU or game FPS claim.</p>
<p>Fit the coefficient on a separate training stream. The gate uses a second independent validation stream. Neither observes final evaluation Y.
Rejected pilot samples are never recycled: fallback is statistically valid, but its already-spent pilot budget can hurt efficiency.</p>
<img src="pilot-gain.svg" alt="MSE gain over plain MC including pilot costs">
<img src="pilot-quality-cost.svg" alt="MSE against total measured CPU policy cost">
<h2>Selected budget B={budget}, {grid}×{grid} cache</h2><p>Gain = plain MSE / policy MSE; below one is worse. Counts are mean train/validation/evaluation calls per trial. Timing p95, raw blocks and conditional analytic moments are in JSON.</p>
<table><thead><tr><th>Case</th><th>Policy</th><th>MSE gain [95% CI]</th><th>Median weight</th><th>Cache used</th><th>Calls train/validate/eval</th><th>Decision μs</th><th>Total μs</th></tr></thead><tbody>{"".join(cells)}</tbody></table>
<p>Transport references are refined quadratures, not certified exact truth. Mean-error intervals use a normal approximation that may fail for rare events. The gate is a heuristic; a small pilot can miss a newly important feature.</p>
<p><a href="pilot-results.json">Results and limitations</a> · <a href="pilot_trials.csv">Raw trial decisions/counts/estimates</a> · <a href="pilot_timings.csv">Raw timing blocks</a> · <a href="manifest.json">Provenance</a> · <a href="review.html">Original estimators</a></p></html>"""
    (work / "pilot-review.html").write_text(page)
    previous = (work / "review.html").read_text()
    (work / "review.html").write_text(
        previous.replace(
            "<h1>",
            '<p><a href="pilot-review.html">Stage 2: independent-pilot cache trust</a></p><h1>',
            1,
        )
    )
