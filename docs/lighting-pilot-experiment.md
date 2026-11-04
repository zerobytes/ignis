# Independent-pilot cache trust experiment — EAF-208 / Lumens

Preflight, 2026-11-04. Extends the [initial estimator experiment](lighting-estimator-experiment.md).
This remains a single-receiver CPU experiment; no renderer change is proposed.

## Estimator and frozen policies

For a pilot-selected scalar `a`, evaluate on fresh, independent points Y:

`I_hat = a*G + mean_Y(f-a*g)`.

Conditioned on the pilot, `a` is fixed and `E[I_hat | pilot] = I`. Any bounded
pilot-selected coefficient (including zero) preserves this identity. Learning
the coefficient or selecting the result using Y would invalidate this argument.
Keep signed corrections; do not clamp the resulting radiance estimate.

The RGB trace-variance optimum for a fixed coefficient is
`a* = sum_channels Cov(f,g) / sum_channels Var(g)`. Estimate these quantities
with online centered moments on a training pilot, then clamp `a` to [0,2].
The clamp is a variance safeguard, not an unbiasedness requirement. Compare:

1. Plain MC, no pilot or cache evaluation.
2. Fixed `a=1`, the previous exact-G correction.
3. Learned coefficient, with an independent training pilot.
4. Learned coefficient accepted/rejected using a second, independent validation
   pilot. Accept only if its estimated residual variance can beat plain MC by
   10% after accounting for the reduced evaluation count. This is a heuristic,
   not a statistical performance guarantee.

Total expensive evaluation budget B includes pilots. Let
`p=min(requested_pilot_size, floor(B/4))`. Learned uses p training samples and
B-p evaluation samples; gated uses p training + p validation + B-2p evaluation
samples when validation is needed. A zero training coefficient skips validation.
A globally constant cache is detected from its complete entries at construction
and bypasses pilots: it cannot reduce variance. All remaining samples use fresh
uniform exploration; narrow emitters retain nonzero sampling support.

Rejected pilots are never recycled into the final estimate. A fallback therefore
can be slower or noisier than plain MC because its pilot cost is already spent.
Record actual train/validation/evaluation counts per trial and reject bookkeeping
that hides this cost. Conditional sampling counts depend only on pilot data.

## Static performance review before implementation

- Reuse the existing standalone Rust executable, scene functions, references
  and dense cache; add a private experiment module, not an engine API/dependency.
- One-pass centered f64 RGB moments and a scalar coefficient: bounded stack
  state, no per-sample fitting allocations or stored training matrices.
- Reuse train/validation/evaluation buffers. Exact constant-cache detection runs
  during construction, never as a whole-cache scan per sample or estimate.
- Measure full policy CPU wall time including PRNG, fitting, validation, fallback
  and final evaluation. Measure the decision phase separately; cache construction
  is a separate cold cost. Equal evaluation count is not equal wall time.
- All policy and statistical output is outside timed kernels. No GPU/readback,
  network, GUI, engine I/O, locks, worker jobs or format/version changes.
- Preserve original numerical CSVs and append pilot CSVs/plots to the same
  current output directory. One executable, one current report set; staging and
  replay files are removed after publication. Include every new source/output
  in provenance/checksums and numerical reproduction.
- Protect independence, total-budget accounting, coefficient scale recovery,
  exact fallback equivalence and signed corrections with focused oracles.
  Evaluate failures as results, without tuning policy thresholds on trial output.

For analytic controls, independent exact moments give conditional variance
`(Vf - 2*a*Cov(f,g) + a*a*Vg)/N`. Average this over pilot outcomes as an
additional check; this prediction uses no final evaluation samples. Continue
reporting numerical reference uncertainty on transport cases and distinguish
error reduction from measured CPU cost or hypothetical GPU/game performance.

## Reproduction and result locations

```sh
uv run scripts/run_lighting_estimator.py --verify-reproducibility
```

`--pilot-samples` sets the requested training/validation batch size (default
32, bounded 4..256). Budgets must be at least 16. The same current executable
`artifacts/lighting-estimator/estimator` runs both the original and pilot
experiments; no engine build or GPU work is involved.

`pilot-review.html` contains standalone plots and a result table;
`pilot-results.json` records all policies, uncertainty, coefficient quantiles,
decision reasons, sample counts, full/decision timings and analytic checks.
`pilot_trials.csv` retains every decision and output, and `pilot_timings.csv`
retains repeated timing blocks. The shared manifest hashes both Rust and both
Python source files, all outputs, and all five numerical CSVs for reproduction.

## Results and decision — initial default run

Apple M4 Pro; 256 trials; caches 4/8/16 and total budgets 16/64/256/1024.
Below is the selected 8x8 cache, B=256. Every policy spends exactly 256 current
target evaluations including pilots. Gain is plain MC MSE / policy MSE;
values below one are regressions. Intervals are paired bootstrap 95% intervals.

| Case | Fixed weight 1 | Learned weight | Validated gate |
| --- | --- | --- | --- |
| Smooth analytic control | 60.98x | 48.02x | 43.86x |
| Missed narrow emitter | 1.00x | 1.00x, no pilots | 1.00x, no pilots |
| Deliberately over-scaled analytic cache | 0.000104x | 54.58x | 46.38x |
| Open environment | 3.483x | 2.344x [1.872, 2.904] | 1.594x [1.301, 1.907] |
| Current blockers | 1.416x | 1.436x [1.216, 1.667] | 1.067x [0.917, 1.250] |
| Moved blocker, old cache | 0.347x | 0.869x [0.799, 0.942] | 0.830x [0.757, 0.906] |

The over-scaled control recovers a median coefficient close to 0.01, matching
its known 100x scale mismatch. This is deliberately favorable to coefficient
correction; it does not establish similar gains in a dynamic game. The
independent exact moment prediction and empirical MSE agree to ordinary
finite-trial accuracy for the smooth/stale controls; the rare-event control
remains noisy and illustrates why normal intervals are insufficient alone.

For the moved blocker, learned weight is zero in about 43% of trials and has
a median of about 0.0296. Validation rejects the cache in every trial. The gate
reduces the fixed policy's 2.88x MSE regression to 1.20x, but cannot recover
the already-spent pilot budget. It averages 32 training, 18.25 validation and
205.75 final evaluations; zero training weights skip the validation batch.
The result is **not** parity with plain MC or a guarantee against regressions.

On open/current-blocker cases, training normally consumes 32 calls, leaving
224 for the learned estimator. Validation consumes another 32, leaving 192.
The gate's additional cost and false rejections reduce useful cache gains.
The current-blocker gate interval includes parity; evidence does not support a
reliable improvement there. Full CPU and separate decision medians are in JSON;
these are single-thread wall times, not isolated hardware or GPU measurements.

**Decision:** retain independent coefficient fitting as a useful mechanism,
but reject fresh training/validation per estimate as a general optimized policy.
Before proposing a world cache, test amortizing a decision over multiple
independent queries during a declared stable epoch, then test how a change
invalidates that decision. A shared decision must never observe the evaluation
samples it controls. This experiment implements neither sharing nor geometric
cache invalidation and makes no Lumen or game-frame claim.

## Validation and retained costs

- Numerical oracles check centered coefficient scale recovery, exact zero/unit
  coefficient behavior, frozen decisions, distinct training/validation/Y seed
  streams, constant-cache bypass, signed corrections and per-policy call budgets.
  All branches retain nonzero fresh uniform evaluation support.
- Constant-cache detection examines complete entries during construction. It
  avoids statistically pointless pilots; it does not claim the true target is
  constant or that the narrow emitter is absent. Both adaptive policies reproduce
  plain MC byte for byte on that control.
- For analytic controls, the report integrates moments independently and averages
  `(Vf-2*a*Cov(f,g)+a²*Vg)/N` over observed pilot decisions. No final Y sample
  enters this prediction or coefficient choice. Confidence intervals are not
  used as a proof of unbiasedness or as acceptance criteria tuned after the run.
- Full CPU timings include PRNG, online moments, validation, branch decisions
  and final integration. Decision timing is independently measured; subtracting
  its median from the total is not an exact stage decomposition. p95 describes
  block averages, not per-estimate tail latency.
- Cache creation remains a separate cold cost (and source-call count), including
  allocation/destruction. Summed cold medians are estimates, not directly timed
  cold runs. A constant-cache fallback still needs existing cache metadata; its
  construction cost has not disappeared. Scratch is reused and reported.
- No per-sample allocation, stored fitting matrices, new dependencies, locks,
  threads, GUI/GPU work or engine format changes. Cache payload remains
  24 bytes/cell; the cached constant flag adds only struct metadata.
- Numerical replay covers the original four CSVs plus the pilot trial CSV;
  timing blocks are intentionally excluded. One current executable/report set
  is retained; `.pending`/replay outputs are removed after success or failure.
  Original baseline data stays current as comparison input, not a historical
  duplicate binary. Native integration and human visual acceptance remain open.
- The full five-file numerical SHA256 is
  `5a4a79957074a5bc3d735a08078e9bef172ecb5eac72240557e75ebdec916d3b`;
  the original baseline digest remains unchanged. Source/output checksums and
  rejected configurations (pilot size zero, budget below 16) were verified to
  preserve the published report. Rust warnings/fmt and Python F checks pass.
  Pilot plots were visually inspected, including exact-parity axis scaling.

The next [shared-decision lifetime experiment](lighting-epoch-experiment.md)
tests amortization and response to changes. The same runner/executable now
includes it; this stage's five-file digest is retained separately in the
manifest as `pilot_numerical_sha256`.
