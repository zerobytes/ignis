# Lighting estimator experiment — EAF-208 / Lumens

Preflight contract, 2026-11-04. See [research synthesis](lighting-literature.md).
The first experiment is an independent CPU numerical laboratory, not a new
renderer, a complete GI solver or a Lumen benchmark.

## Hypothesis and estimators

On a common uniform domain `[0,1)^2`, freeze a midpoint piecewise-constant
cache `g` before evaluation. Compare plain Monte Carlo `mean(f)`, the exact
cache integral `G`, `G + mean(f-g)` and a two-level variant
`mean(g(X)) + mean(f(Y)-g(Y))` with independent X/Y streams and `|X|=4|Y|`.
The exact-cache variant is a control variate special case, not a finite-sample
two-level estimator. Both corrected variants retain signed corrections.

Analytic controls have exact integrals: smooth RGB polynomial, a narrow bright
rectangle missed by coarse caches, and a deliberately stale high-variation
cache. The transport controls integrate Lambertian outgoing light at a single
receiver from an environment with optional opaque boxes. Cosine-weighted
hemisphere sampling maps to the uniform domain: `f = albedo * Li * visibility`.
This cancels `cos(theta)/pi` with the sampling PDF. It does not simulate
multiple bounces. Current and stale blocker layouts are explicit.

Use exact analytic truth where available. Transport truth uses nested
deterministic quadratures, with reported refinement differences and a finest
grid retained as a numerical reference, not declared exact. Record independent
trial distributions, signed mean error, variance, MSE, RGB errors, uncertainty
of the estimated mean and paired MSE comparisons. Statistical compatibility
with zero is not proof of unbiasedness; the estimator identity supplies the
mathematical argument under the stated sampling assumptions.

## Static performance review before coding

- Frequency: standalone batch tool only. No per-frame runtime work or engine
  changes. A dependency-free Rust kernel avoids compiling or locking the shared
  engine; Python orchestrates provenance, statistics and standard plots.
- Allocation/data: f64 RGB values, bounded dense cache, reusable sampling
  buffers. Prepare inputs outside sampling timings. Report their generation
  cost separately and include it in end-to-end estimator timings.
- Repetition: construct each cache once, retain its exact integral, and reuse
  it across sample counts/trials. References and independent validation are
  outside estimator timings. Warm benchmark kernels before repeated blocks.
- GPU/I/O: no GPU, GUI or new textures. Batch CSV output after measurements;
  no filesystem writes inside timed estimator kernels.
- Synchronization: single-threaded kernel, no shared random generator, locks,
  background jobs or engine resources. Freeze approximation and seeds before
  evaluating trials; common random numbers pair plain/corrected estimates.
- Budgets: bounded trials, sample counts, cache resolutions and quadrature
  resolution; reject invalid/excessive CLI values. Record cost of cache
  construction and payload bytes. Report warm and cold/amortized costs distinctly.
- Outputs: one stable directory `artifacts/lighting-estimator/`; replace the
  current kernel and reports, with no versioned/timestamped binary history.
  Publish each result file by atomic replacement; a complete manifest records
  source/config hashes and output checksums. Preserve unrelated outputs.
- Scientific risks: wrong PDF/Jacobian, residual clipping, contaminated random
  streams, stale-cache bias, reference error and misleading time labels.
  Protect these with focused numerical oracles and inspect experiment failures.

## Delivery and interpretation

Deliver an editable/reproducible runner, kernel, numerical report, plots and
review page. Equal expensive-sample comparisons must show actual total CPU
time and cache overhead. MSE-times-time is an efficiency proxy, not a direct
equal-time benchmark. Renderer GPU performance, dynamic GI, artistic acceptance
and Lumen comparisons remain separate, untested claims.

## Reproduction and retained artifacts

```sh
uv run scripts/run_lighting_estimator.py --verify-reproducibility
```

The script declares its plotting dependencies using PEP 723. It compiles only
`scripts/lighting_estimator.rs` using `rustc --edition=2024 -O -D warnings`;
it does not run Cargo or replace `target/release/ignis`. The retained current
executable is `artifacts/lighting-estimator/estimator`. Compilation/evaluation
uses a private `.pending` stage, removed on completion/failure. Numerical replay
uses a temporary subdirectory that is also removed; no old executable is kept.

`review.html` contains the plots and selected results. `results.json` contains
RGB statistics, paired bootstrap intervals, analytic expected moments, reference
refinements, per-method costs and limitations. Raw `trials.csv`, `timings.csv`,
`references.csv`, `caches.csv` and `maps.csv` allow independent inspection.
`manifest.json` records seed/configuration, CPU, compiler/runtime versions,
source SHA256s, numerical replay digest and output checksums. Check every output
checksum before consuming a run; the manifest is replaced last.

Default configuration: six cases, 256 independent PRNG trial streams, sample
counts 16/64/256/1024 and 4x4/8x8/16x16 caches. The X stream uses four times
as many points as Y. References use exact analytic integrals or 512x512,
1024x1024 and 2048x2048 midpoint quadrature in the cosine-sampling domain.
The three transport cases have opaque box visibility and a directional sky
plus a bright environment lobe. They evaluate one receiver and zero secondary
surface bounces; the directional fields are not native engine images.

## Initial results on Apple M4 Pro

At n=256 expensive evaluations and an 8x8 cache, measured RGB MSE ratios are
below. Gain means `plain MC MSE / corrected MSE`, not a frame-rate multiplier.
Intervals are paired 95% bootstrap intervals across the 256 trials.

| Transport case | Exact cache integral + residual | Finite two-level estimate (4n cheap) |
| --- | --- | --- |
| Open environment | 3.483x [2.658, 4.483] | 1.859x [1.459, 2.374] |
| Current opaque blockers | 1.416x [1.109, 1.780] | 0.893x [0.695, 1.145] |
| Blocker moved, old cache retained | 0.347x [0.285, 0.415] | 0.289x [0.238, 0.344] |

The smooth analytic control benefits strongly (60.98x for exact-G correction,
4.67x for finite two-level sampling), but it is an intentionally favorable
polynomial, not evidence of a comparable game gain. The narrow emitter has
identical variance with/without exact-G correction because every tested cache
misses it. Many short trials miss it too: empirical error need not decrease
monotonically as the first rare contributions are discovered. Its expectation
and variance are available exactly; normal confidence intervals can fail here.

The deliberately stale, over-scaled analytic predictor makes exact-G correction
about 9,575 times worse in measured MSE at this budget. This is a failure
control, not a representative dynamic-scene average. Even the more realistic
moved-blocker case worsens MSE by about 2.88x. Correct expectation alone does
not make a noisy correction efficient.

The finest-grid refinement RMS is approximately 5.11e-7 (open), 2.68e-4
(occluded) and 2.83e-5 (changed). These are differences between numerical
references, not certified bounds on truth. Reported physical MSE/intervals
are relative to that finest reference.

## Decision from the experiment

Continue with a representation whose integral is cheap to evaluate, plus
independent residual evaluation. The finite 4:1 allocation is not accepted as
a general policy: its extra cache-sampling variance/cost can erase the benefit.
Study analytic/low-order integrated representations and pilot-based weighting
or rejection of a poor predictor before adding reservoirs or a world cache.
Use independent pilot data for any learned weights/allocation; keep exploratory
samples so a narrow missed contribution cannot disappear permanently.

The stale controls reject unconditional reuse with unit correction weight.
They motivate change detection/invalidation and covariance-aware control
weights, but this experiment implements neither mechanism. Cache density alone
does not repair a predictor for the wrong scene.

## Validation and final cost review

- Numerical oracles cover reproducible [0,1) sampling, slab visibility (including
  parallel/behind rays), unit hemisphere directions and cosine moments, exact
  cache measure, the missed feature, and a signed stale correction. Independent
  rational integration supplies exact means and second moments for analytic
  controls, including each cache's residual variance. No statistical CI pass is
  used as proof of unbiasedness.
- Identical-input replay checks all four numerical CSVs byte for byte; timing
  values are deliberately excluded. The current numerical SHA256 is
  `6eec81465885e0f0886ec583f6bc592cf6912cde1aadc9820cf827e713ce3971`.
- Rust warnings/fmt and Python F-class static checks pass. Generated plots are
  inspected. This isolated numerical change does not require engine/GPU tests.
- The cache payload is 24 bytes/cell: 384/1536/6144 bytes at the tested grids,
  excluding allocator/struct overhead. Active and allocated input scratch are
  reported separately; the kernel retains both X and Y buffers. The current
  experiment does not establish a cache shared between receivers. Multiplying
  a per-receiver cache by screen pixels is not a viable inferred architecture.
- Timings include PRNG/fill into reusable scratch; input generation is also
  measured separately. CSV/report/reference work is outside timed kernels.
  Cache setup timing includes allocation/destruction. Cold/amortized costs sum
  separately measured medians and are estimates, not directly timed cold runs.
- Wall-time blocks are not hardware counters, GPU measurements or isolated
  machine benchmarks. p95 is across block averages, not per-estimate tail
  latency. No exclusion of competing CPU work is enforced. No new engine
  allocation, I/O, lock, render pass, dependency or serialization format exists.
- One current lab executable/report set remains in the stable output directory;
  task staging and replay outputs are removed. Unrelated sources, builds,
  processes, windows and the birch are preserved. EAF-208 remains In Progress.
