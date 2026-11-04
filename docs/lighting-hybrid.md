# Geometry-guided RGB allocation — EAF-208

Preflight 2026-11-04. Objective: find a practical quality/cost point, not a
universal minimum or a production GI architecture. Use direct stratified64 as
the practical quality baseline. A candidate must not worsen its mean RGB error
or the pre-existing near-partition region; also expose tail/tile errors, query
cost, decision cost, primary-map cost and probe reconstruction cost.

## Prediction independent of evaluation samples

Keep the prior one-bounce target, grids and references. The classifier uses
only geometry: distance from the receiver to another AABB, or to the edge of
its own planar face, whichever is smaller. Near geometry changes use fresh64
rays; elsewhere query the0.5m visibility-gated probe field. If no probe survives,
use64rays, not the old16-ray zero-weight fallback. No reference, target query,
material label, object ID heuristic or fixed problem-region coordinate controls
allocation. The near-wall region is only a reporting mask.

Store the geometric feature in the receiver's unused rho.w slot. Compute it
once in the primary-map setup and reuse only while camera/geometry are fixed.
A simple compare chooses the shading branch. Primary map with/without feature
gets a paired32-sample study, so moving-camera models include decision setup.
Do not claim a new per-frame CPU culling pass or zero decision overhead.

Thresholds0.25/0.5/0.75m are calibrated on the existing open/thin-wall scenes.
These are0.5/1/1.5fine-grid spacings. Initial read-only recomposition of the four
saved H3 seeds suggests0.25m misses thin-wall local error,0.5m improves global
and local MSE with about half of receivers traced, and0.75m spends more rays.
This is a calibration forecast, not measured GPU speed or held-out evidence.
Actual GPU runs must charge divergence, field lookup, fallback and comparison.

After calibration, freeze the cheapest eligible policy before any held-out
reference is evaluated. Store the policy, code hashes and calibration decisions.
Two additional scenes change geometry/lighting/materials/view; their parameters
are fixed before inspecting reference errors. If validation fails, reject or
revise and explicitly mark that data as used for calibration; do not silently
retune against the same holdout and claim independent validation.

## Limits and checks

The hybrid is biased wherever it uses an approximate probe field. A geometry
threshold is an empirical reliability policy, not a certified radiance-error
bound. Direct64 is a noisy practical baseline, not ground truth. Keep four
quality seeds and the mean of two independent4096 references. Test all-pixels-
direct and all-pixels-probe limiting cases with identical random streams;
CPU-check geometry features and exact branch composition. Preserve native
pixels, channel/energy checks and offscreen influencers. Geometry/light changes
still require full probe rebuild; no temporal history or dynamic invalidation
is added. Construction amortization is a model, not measured gameplay FPS.

No production shader, material, scene format, asset, app GUI or dependency change.
Stable output root is `artifacts/lighting-hybrid`; the current diagnostic binary
stays at `target/release/examples/lighting_bounce` without historical copies.

## Additional calibration and independent H6 comparator

A read-only tail audit found that geometry0.5m alone still worsens an image
block, despite improved global/near-wall means. Extend the calibration set to
geometry1m and geometry0.25m plus frozen probe disagreement0.02/0.03/0.04/0.08,
normalized by sun RGB scale. Disagreement loads the eight neighboring probe
values; its cost is included in the query, even if it rejects the lookup.
It is a heuristic, not a radiance-error certificate.

Also test H6 spatial reconstruction, independent of probes. A geometric guide
admits3×3neighbors only on the same primitive instance and normal, within
2.5times the nearest compatible cardinal-pixel distance. Gaussian image weights
are1/0.5/0.25. No current radiance value controls neighbor admission or ray count.
The effective neighbor count `(sum w)^2/sum(w^2)` is a stationary-noise proxy,
not an error bound. Compare filtered16 without adaptive rays, adaptive16 with
at least6effective neighbors, and adaptive32 (8×4strata) with at least4; otherwise
trace64 and preserve the result without filtering. Read raw samples from one
buffer and write reconstruction to another, avoiding in-place GPU races.

Guides require a separate per-camera/geometry pass and4bytes/pixel; filtered RGB
requires16bytes/pixel. These resources and guide time are disclosed. H6 does
not need a probe field or a rebuild after a light-only change, but actual dynamic
scene behavior is not yet established. This opaque flat-color box guide does
not establish behavior on textures, curved surfaces, transparency or specular.

Freeze acceptance before heldout: mean RGBMSE, near-partition RGBMSE, pixel-error
p95 and worst of8×8image-tile MSE must all be no worse than direct64 on both
calibration scenes. Expose every seed and candidate failure. Choose the cheapest
eligible policy under a disclosed moving-camera + field-rebuild/16frames model;
also provide the best choice over rebuild periods1/2/4/8/16/32/infinite. Field
construction is charged only to algorithms that actually require it. No claim
that16frames is the user's eventual game requirement. Heldout remains separate.

Heldout parameters are fixed in source before their references run. A moves the
partition tox=-0.63/z=0.4, shifts the roof, swaps wall colors, changes sun RGB/
direction and views from(-5.5,6,10). B uses a perpendicular0.06m partition at
z=0.37, raises/moves the small box, changes sun direction and uses camera
(6.5,5.5,11). The reporting band follows each partition geometrically, never
controls the runtime policy, and is not changed after validation. These are
static newly revealed views and fresh fields, not a continuous-motion or
stale-history validation. Dynamic update/animation acceptance remains separate.

## Phase 1 result and phase 2, fixed before fresh holdouts

The initial frozen policy was H6 adaptive32, not a probe hybrid. It passed all
four error criteria on A/B, but full-camera setup modeling left only a small
speed benefit. Original selection/results are retained as bounded scientific
metadata in `frozen-initial/` (no executable copies).

Phase2 compacts camera-visible pixels into low-budget and64-query lists on GPU,
using workgroup prefix sums and global reservations. GPU-produced indirect
arguments drive separate homogeneous-budget dispatches. RNG remains keyed by
pixel and query seed, not list order. Compare compact32 against the original32
policy with the same maps; zero-low, zero-high and both-empty cases are checked.
No CPU counter readback schedules a frame. Extra resources are two uint lists,
two counters, dispatch arguments and a small non-indirect marker buffer.

Test compact20/24/28/32, all with the same previously frozen geometric guide,
filter and64-ray edge fallback. This revises only sampling budget/execution;
A/B are now historical exploratory views, not fresh validation for these budgets.
Require10%calibration quality margin on all four metrics, and no degradation on
fresh C/D. Predeclare compact32 (equivalent to the original validated policy) as
fallback if the cheaper selected budget fails; report that rejection explicitly.
No parameter is retuned on C/D.

C uses a thinner partition atx=0.83, changed roof, blue/orange walls, changed sun
and camera(8,4.5,9). D uses a perpendicular partition atz=-0.43, moved roof/box,
changed sun and camera(5.5,8,9). Both were fixed in source before their references.

In addition to128steady query samples, measure32complete GPU graphs per compact
variant and direct64: native raster + primary-map regeneration + guide + GPU
compaction/arguments + tracing + filter. The camera is fixed during a case but
all its dependent maps are recomputed each frame; this is measured setup work,
not a sum of separately favorable timings or a moving gameplay/FPS benchmark.
Only algorithms that actually need probes are charged probe construction.

## Measured result — 2026-11-04

The frozen selected policy is compact20:5×4 randomized strata where the guide
has at least4effective neighbors, otherwise64(8×8). Both fresh C/D scenes passed
all four criteria; the predeclared compact32 fallback was not needed. This is
the lowest measured GPU-cost candidate among20/24/28/32 under the declared
calibration quality margin; it is not a global minimum. Low-budget16 variants
were exploratory controls, not independently validated production choices.

Apple M4 Pro / Metal,480×320. Medians below include native raster and the entire
recomputed primary/guide/compaction/tracing/filter graph,32samples per variant.
CPU is preparation and submission, excluding synchronization/readback/PNG.
GPU timestamps are fenced before resolution; no stale timestamp samples are
accepted. Measurements are on a shared workstation, not isolated game FPS.

| Scene | Direct64 GPU ms | Selected GPU ms | GPU reduction | Direct64 CPU ms | Selected CPU ms | RGB MSE ratio | Worst-tile MSE ratio |
|---|---:|---:|---:|---:|---:|---:|---:|
| open | 1.475 | 1.158 | 21.5% | 0.253 | 0.335 | 0.799 | 0.858 |
| thin-wall | 1.549 | 1.152 | 25.6% | 0.169 | 0.234 | 0.805 | 0.810 |
| validation-c | 1.894 | 1.430 | 24.5% | 0.195 | 0.269 | 0.813 | 0.848 |
| validation-d | 2.036 | 1.476 | 27.5% | 0.165 | 0.221 | 0.728 | 0.778 |

The CPU overhead increases by0.056–0.082ms in these medians; this is a GPU
tradeoff, not a CPU optimization. A CPU-bound application may prefer direct64
or require pass fusion before adopting it. Only5.6–17.7%of visible receivers
retain64samples; nominal ray reduction alone does not predict elapsed time.

Compacted32 and the previous mixed32 implementation produce exactly equal
saved quality maps on all four scenes. Independent CPU code checks256transport
samples total, guide masks exactly, compacted partitions/dispatch counts,
all-pixel reconstruction and branch composition. Empty-low/empty-high/both-empty
lists, zero energy, isolated channels and unchanged native raster checks pass.
Four query seeds and two independent4096-sample references per scene are retained;
reference noise remains. Metrics use foreground scene-linear RGB, not tone-mapped
PNG or background dilution. Full per-seed/tail/timing records are in the JSON.

Additional allocation is28bytes/pixel plus56bytes for guides, filtered output,
work lists, counters and dispatch/marker buffers:4.10MiB at this resolution.
This is incremental over existing receiver/raw-output buffers. The diagnostic
also retains probe fields and readback resources for competing controls; it is
not a minimal production memory implementation. H6 itself does not need probes.

Reproduce with `uv run scripts/run_lighting_hybrid.py`. The runner waits for a
successful current release build, writes source/executable manifests, calibrates,
freezes selection, then evaluates C/D without retuning. `--use-calibration`
requires matching manifests. Current evidence is `artifacts/lighting-hybrid/`
(`review.html`, `summary.json`, `selection.json`, comparison images and raw maps).
One current example executable is retained; no app window or production renderer
was changed. Scoped release Clippy with the pre-existing unrelated
`collapsible_if` lint isolated, Rustfmt2024, Python F checks and GPU/CPU oracles
passed; no all-workspace test result is claimed.

Next falsification is textured/curved receivers and continuous camera/light/
geometry changes, followed by general triangle intersections and cost scaling.
Within-primitive radiance discontinuities can still be blurred: the guide is a
geometric heuristic, not an unbiased estimator or an error certificate. Static
boxes do not establish motion stability, multi-bounce transport, transparency,
specular quality, artistic acceptance or superiority to Lumen. EAF-208 remains
In Progress; no production GI integration is implied.
