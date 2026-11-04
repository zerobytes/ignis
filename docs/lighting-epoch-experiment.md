# Shared-decision lifetime experiment — EAF-208 / Lumens

Preflight, 2026-11-04. Builds on [independent pilot selection](lighting-pilot-experiment.md).
The scope is repeated independent estimates at one receiver. It shares a
coefficient, not rendered radiance, geometry, or a cache between spatial points.

## Fixed design before evaluation

Each query has B expensive target evaluations, including training, sentinel
snapshots and monitoring. Remaining Y samples are fresh and independent of
all control streams. `a*G + mean_Y(f-a*g)` remains conditionally unbiased even
if a is stale; the failure being measured is variance/cost, not missing light
due to deleting sampling support. Y never updates the decision or sentinel set.

Compare plain MC, per-query training, periodic shared training, notified shared
training, and sentinel-watched shared training. Shared coefficients expire
after H queries. Training uses `min(pilot_size,B/4)` samples. All source-cache
values stay fixed to isolate coefficient lifetime from rebuilding the cache.

Notified reuse receives a scripted scene revision supplied by the environment.
This is an explicit ideal notification baseline, not inferred from pixels or
a claim that Ignis already tracks every lighting dependency. Other policies
must not inspect that revision when deciding when to retrain.

Watched reuse snapshots up to eight independent sentinel directions when
training. Every four queries it re-evaluates those same directions, stopping
at the first changed value, then trains independently and replaces sentinels.
TTL refresh takes precedence. The comparison is deterministic ray visibility/
lighting at those directions, with `|a-b| > 1e-9*max(1,|a|,|b|)` per channel. A finite
sentinel set can miss a localized change until TTL; no detection guarantee is
assumed. If H<=4, TTL already refreshes at least as often as monitoring, so
sentinel snapshots/checks are omitted.

Four sequences: stable blockers, blocker moved permanently at query17, blocker
moved at17 and restored at29, and environment intensity reduced to 10% at17.
Default length48, lifetimes1/8/32, budgets64/256, cache8x8, 128 independent
sequences. Changes do not align with lifetimes8/32. Both transient and lasting
changes matter. The transport integrand and refined references are reused;
scaled illumination uses the corresponding scaled reference by linearity.

Report sequence-cluster bootstrap intervals, pre/change/recovery errors,
per-query curves, actual call counts, refresh counts, change detection latency
and missed events. Do not treat correlated queries within one sequence as
independent trials. A refresh after a transient event has ended does not count
as detecting that event. Store limited raw traces plus every sequence summary.

## Static performance review before implementation

- Extend the current standalone Rust executable by one private module. Reuse
  pilot moments and weighted integration; add no engine or external dependency.
- Bounded f64 state: one coefficient, refresh time/revision, reused train,
  sentinel and evaluation buffers. Prepare target state once per query, outside
  per-sample arithmetic. No allocation per query or monitor sample.
- Charge sentinel snapshots/checks/retraining to the current query; stop an
  already-positive sentinel check early. A known revision update and TTL need
  no extra ray query. Scene notifications are a supplied baseline, not free
  general world dependency analysis.
- Measure complete-sequence CPU cost with preallocated output records and
  no reference/error computation or CSV I/O in timed work. Cache construction
  remains separate. No GPU, GUI, locks, jobs, game formats or runtime changes.
- Keep one executable and current report directory. Append epoch artifacts to
  the comparison set, record all source/output hashes, and remove staging/replay
  after publication. Bound total work and trace volume; stream/aggregate trial
  summaries instead of retaining every sample/query as Python objects.
- Protect budget accounting, independent evaluation, H=1 equivalence, exact
  notification response, transient-event accounting, monitor blind spots and
  pure illumination scaling with focused numerical oracles. Freeze thresholds
  before reading outcomes. Scientific results need not favor shared reuse.

## Reproduction and evidence

```sh
uv run scripts/run_lighting_estimator.py --verify-reproducibility
```

The runner now also executes the epoch module with `--epoch-trials` (default
128, capped by `--trials`), `--epoch-steps` (48), and `--epoch-horizons` (1,8,32).
Epoch budgets are 64/256 when those exist in `--samples`, otherwise its largest
budget. The grid is8 when present in `--cache-grids`, otherwise its first grid.
All effective values are recorded in the manifest/configuration. Bounded work
rejects combinations above150M target evaluations for the epoch experiment.

Current outputs share `artifacts/lighting-estimator/` and its one executable:
`epoch-review.html`, `epoch-results.json`, per-sequence `epoch_trials.csv`,
population `epoch_curves.csv`, selected raw `epoch_trace.csv`, and CPU timing
blocks in `epoch_timings.csv`. Raw traces retain the first8 trials at the largest
epoch budget/lifetime. Error statistics retain every independent sequence;
omitted traces can be reproduced from source/configuration/seeds.

The default epoch study has128 independent sequences per configuration, not
128×48 independent trials. The bootstrap samples whole sequences, preserving
shared-pilot correlation. Query curves show ratios of population mean MSEs;
they are not individually bootstrapped time-series confidence bands.

## Results: default grid8, budget256, lifetime32

Gain below means plain MC MSE / policy MSE over all48 queries. Values below
one are regressions. These are repeated estimates at one receiver in CPU,
not game frames, GPU timing or spatial/global-cache performance.

| Sequence | Train every query | Shared TTL | Supplied revision | Paired sentinels |
| --- | --- | --- | --- | --- |
| Stable geometry | 1.361x | 1.548x | 1.548x | 1.530x |
| Geometry step | 1.053x | 0.974x | 1.219x | 1.114x |
| Geometry pulse | 1.293x | 1.196x | 1.451x | 1.305x |
| Light reduced to10% | 2.213x | 1.022x | 2.592x | 1.958x |

In the stable sequence, TTL/notified reuse fits twice (queries0/32):64 training
calls across48 queries, or1.333/query, versus32/query for per-query fitting.
This is24x fewer training calls, not24x fewer total calls or24x faster execution.
The same total target budget instead leaves more independent Y samples.
Sequence MSE improves about12% relative to per-query fitting. CPU timings remain
similar because B target calls are still spent; no speedup multiplier is inferred.
The paired sequence-MSE gain over per-query fitting is1.137x, with a95%
cluster-bootstrap interval of[1.098,1.175].

### Response and missed events

- Geometry step at17: supplied revision refreshes immediately. TTL32 waits15
  queries. Paired sentinels have median delay3 but a95th percentile of15,
  because missed directions rely on the later TTL refresh.
- Geometry pulse17..28: TTL32 never refreshes while the pulse is active (100%
  missed). Sentinels miss14/128 pulses (10.94%). Conditional median delay is3
  for detected events; reporting only that delay would hide the misses.
  Sentinels also miss12/128 restoration responses before the sequence ends.
- Light change at17: sentinels respond at20, three queries later; supplied
  revision responds at17. TTL waits until32. Fixed sentinel checks are effective
  here because all unoccluded environment values change.

Whole-sequence scores can conceal severe temporary error. During queries17..28
of the dim-light sequence, TTL has gain about0.014 (about73x more MSE than
plain MC) and watched reuse about0.056 (about18x more MSE), despite whole-
sequence gains above one. Inspect both the middle/late windows and query curves;
bright early queries otherwise dominate an absolute-MSE average. No temporal
image accumulation is implemented, so this is a variance response, not a tested
ghosting/afterimage result. The corrected estimator still has fresh support.

The notified baseline performs best across these four sequences, but it assumes
every relevant change is explicitly supplied. It demonstrates the value of
dependency information, not a solved invalidation algorithm. A global source
revision does not establish a correct/efficient affected-region map, offscreen
dependency tracking, continuous animation handling or multi-bounce invalidation.

## Decision and continuation

Shared training is supported for a stable receiver/field epoch. Fixed lifetime
alone is rejected as a safe default for changing lighting, and sparse sentinels
are not an adequate substitute for known source changes. Prefer explicit
dependency/revision invalidation where available, with TTL/sentinels as measured
fallbacks rather than certificates that nothing changed.

The next missing evidence is spatial transfer: a coefficient trained at one
receiver need not suit nearby receivers even in the same unchanged scene.
Test multiple receivers across visibility boundaries and define valid receiver/
source identities before proposing a world-space decision cache or GPU adapter.
This unit changes no engine renderer or authoring/persistence contract.

## Validation and final performance review

- Oracles cover every-query budget conservation, H=1 equivalence, immediate
  supplied-version refresh, pulse censoring, pure light scaling, deterministic
  stable sentinels and a real directional blind spot (two targets agree at a
  probe while differing elsewhere). Existing sampling/signed-correction/pilot
  independence oracles still pass.
- Python checks complete sequence counts, trial identities, all per-query call
  budgets in selected traces, finite statistics and H=1 equality. Query-local
  refresh metadata is diagnostic; only the notified policy consults revisions
  for control. Final evaluation samples never feed the next decision.
- Preallocated query records are retained with `black_box` in timed sequences.
  Reference comparison, phase errors, curve aggregation and CSV output are
  outside measured kernel work. No per-query allocation or dependency added.
- Cold cache setup/source calls/payload are reported separately. They are not
  part of the warm B target-call budget. Summing separate medians is a cost
  estimate; p95 is across block averages, not per-query tail latency.
- The original four-file and five-file numerical digests remain recorded in
  the manifest. Replay now checks eight numerical CSVs, excluding timing blocks.
  One current executable and comparison report set remains; staging/replay is
  removed. No Cargo, native app build, GUI/GPU or asset change is involved.
- Engine integration, spatial transfer, multi-bounce GI, artistic acceptance
  and any comparison to Lumen remain unvalidated; EAF-208 stays In Progress.
- All eight numerical CSVs reproduce byte for byte (SHA256
  `5bffaf58e2deba7196053a253665b5491be42f5bc6172517819eb08d8668383f`).
  Both earlier-stage digests remain unchanged. Source/output checksums and
  staging cleanup pass. Invalid lengths, trial counts, lifetimes and excess
  work are rejected without replacing the current report. Rust warnings/fmt,
  Python F checks and rendered plot inspection pass.
