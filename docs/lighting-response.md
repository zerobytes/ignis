# Scalar color-response coefficients — EAF-208

Preflight2026-11-05. Separate from the native shadow-query experiment: optimize
UPDATES of the added baked GI feature, not the original GI-off renderer.
Specialize GPU Gems3 ch9's transfer idea to fixed-geometry, fixed-sun-direction,
one-bounce Lambertian boxes with constant per-object albedos. For each atlas
texel/source object store `w_ij=mean(first-hit-j * sun-cosine * sun-visibility)`.
Then `I_i=D*sum_j(w_ij*rho_j)`; receiver albedo is multiplied at lookup as before.
Colors and RGB sun scale can change; geometry/direction/BRDF/topology cannot.

Build weights once from the same4096cosine-stratified paths/offsets/seed317 used
by the full bake. Recombine from CURRENT colors, not accumulated deltas. Store
dense scalar weights padded to vec4rows; max12stable object slots in this bounded
prototype. Validity is separate from zero illumination. All-black source/light
must remain a valid chart, and zero-to-positive light channels must work.

Fixed20case sequence per original open/thin scene: base palette, all black,
four isolated white objects, all white, zero sun RGB, colored sun at intensity2,
then11deterministic hashed palettes. Rotate among front/E/F cameras. No geometry
or sun-direction edit is silently accepted. No new optical/material phenomenon
or multi-bounce transport is assumed.

Before evaluation require nonnegative finite coefficients, row sum<=1.0001,
identical chart validity and maxabsolute scene-linear RGB error<=2e−4 against
full4096ray bake and its displayed receiver values. Reassociation in f32 means
bit equality is not assumed. Independent CPU basis-color transport verifies
coefficients, CPU recombination verifies all atlas rows, and PNG maxchannel
difference must be<=1. Report actual errors, not only threshold pass/fail.

Measure cold coefficient construction/storage separately.16rotated update
samples per method/case charge recombination/full bake PLUS GPU publication;
geometry/color upload submission logged separately and its GPU execution,
allocation/init/readback/fence/file I/O excluded from timed intervals. Same
unchanged native direct raster is used for image comparison, no new engine
lighting is installed. The ordinary baked query cost remains extra versus GI-off.

Stable outputs artifacts/lighting-response; existing lighting_bounce executable
is replaced in its usual Cargo location. No historical binary copies or GUI.
This does not implement sparse/general transfer, runtime regional coefficient
repair, async scheduling or physically general material changes.

## Result2026-11-05

Executed40cases (20per scene),16alternated samples/method/case on the local
Apple M4 Pro GPU. Below are medians across per-case medians; CPU is submission only,
GPU includes calculation plus atlas publication, with the exclusions above.

| Scene | Full rebake GPU | Recombine GPU | CPU full / recombine | Weight storage | Cold coefficient GPU |
|---|---:|---:|---:|---:|---:|
| Open,8objects |33.853ms|0.02075ms|0.14717 /0.13701ms|1.500MiB|59.647ms|
| Thin wall,9objects |40.770ms|0.02678ms|0.14957 /0.13249ms|2.53125MiB|39.969ms|

This is about99.93–99.94% less GPU time for this fixed-scene COLOR UPDATE,
not a whole-frame speedup. CPU submission does not fall by the same factor.
Cold weights cost more than an ordinary bake and consume additional memory;
weight build is a one-shot observation, not a statistically calibrated estimate.
Weights are dense/padded:9logical scalars consume12f32 values per texel.
Storage scales quadratically with object count here, so these small-box results
are not a scalability claim for a game world.

All40fields and displayed receiver values passed the preregistered2e-4 bound.
Maximum atlas RGB error2.37823e-5; maximum field MSE8.52e-12. CPU recombination
across every atlas row differed by at most1.15e-7. Four independent nonzero
basis-color/row ray checks (two per scene) matched the GPU coefficients within
5e-6 (observed0 at these chosen samples). Chart validity was identical; both
all-black sources and zero-light RGB stayed exactly zero, with successful
zero-to-positive recovery. All40PNG comparisons have maximum channel error<=1.
The full4096ray reference has its own sampling/spatial approximation error:
matching it does not establish ground-truth lighting or artistic acceptance.

Review: `artifacts/lighting-response/review.html`; full timings, binary fields,
weights, source/executable hashes and independent audit in `summary.json`.
Reproduce with `uv run scripts/run_lighting_response.py`; re-audit retained
inputs with `--report-only` (requires matching executable/source hashes).
The final retained dataset was reproduced by the complete runner: release build,
both sequential GPU scenes, source/executable pinning and independent CPU/PNG
audits. No GUI was opened or runtime default changed.

Release example build, scoped release Clippy (`--no-deps -D warnings
-A clippy::collapsible_if`), scoped rustfmt, Python F checks, diff checks and all
GPU/CPU audits pass. An intermediate Cargo revalidation was temporarily blocked
by unrelated simultaneous inspection-flight changes (E0499); the owning work
resolved it before the successful final build. Those parallel changes were
preserved and are excluded from this unit's commit. No full-workspace test or
production/runtime integration claim is made.

Next research contract: retain these coefficients for color-only edits; study
regional coefficient repair for geometry/sun changes and bounded asynchronous
publication. The current prototype deliberately rejects unsupported edits; it
has no persistent response-file loader/invalidation integration or production
scheduler. Ordinary baked lookup remains the previous added per-frame GI cost.
