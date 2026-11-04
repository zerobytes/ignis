# Lighting research program: independent axes — EAF-208

Checkpoint 2026-11-04. The first native gallery was **not** a comparison of all
lighting architectures in the literature. Its nine variants study one transport
proxy (finite-radius ambient visibility) and the cache/control-variate hypothesis.
Visibility pruning is orthogonal work elimination, not a new GI estimator.
No complete GI architecture has been selected or shown superior to Lumen.

| Axis / hypothesis | Competing baselines | Evidence in Ignis | Required next falsification |
|---|---|---|---|
| H1: predictable residual reduces expensive estimation work | Plain MC, frozen cache, corrected cache, independent pilots, shared weights | CPU analytic/dynamic laboratories; native static GPU AO gallery | Matched RGB one-bounce transport, changed lighting/geometry; include construction/memory and prediction cost |
| H2: sample distribution reduces variance without prediction | IID vs randomized stratification at4/16/32/64 queries | Native GPU sampling experiment in this unit | Thin blockers/emissive features, camera motion, equal-time Pareto comparison; cannot infer all high-dimensional path sampling |
| H3: sparse representation amortizes indirect transport over receivers | Sparse probes/hash, angular incident cache, split distance intervals/cascades | Native one-bounce RGB baseline: two probe grids and visibility-gated interpolation; cascades/hash remain unimplemented | Match local error at thin blockers, build/update costs and directional coverage before adopting a representation |
| H4: spatial/temporal reservoir reuse improves path/light selection | Independent sampling vs ReSTIR/GRIS-compatible reuse | Literature only; none of the TTL policies is ReSTIR | Many lights/emissive transport, valid PDFs/mappings, disocclusion, duplication and reservoir traffic |
| H5: learned representation beats explicit caches | Small explicit field vs neural radiance/incident predictor | Literature only; no native training/inference | Same target/updates/device, include training/adaptation/memory, unbiased correction vs biased termination |
| H6: reconstruction buys quality with fewer samples | Full resolution direct16/64 vs geometry-guided spatial reconstruction and GPU-compacted adaptive budgets | Native one-bounce box laboratory; 20/24/28/32 low budgets,64 at geometric boundaries; full setup/dispatch/filter measured | Textures, curved surfaces, motion, disocclusion and bias beyond flat opaque boxes; temporal reconstruction remains unimplemented |
| H7: build-time transport amortizes stable content | Object-space surface bake vs fresh64/compact20 vs guarded lookup | Persistent six-face box atlases, three resolutions, RGB light rescaling and full invalidation; [bake experiment](lighting-bake.md) | General UV/triangle surfaces, budgeted regional repair, streamed/procedural content, dynamic characters and directional relighting |
| Shared infrastructure | Camera-visible receivers; conservative influencer selection; accelerated intersections | Native visibility masks; bounded AO influence experiment | BVH/build-update study, retained offscreen influences, dynamic scene costs |

The axes may compose, but their improvements cannot be assumed additive.
A cheaper AO pass does not replace the experiment needed to choose between
radiance cascades, reservoirs and a neural representation. Existing candidate
names should always identify their target and axis to avoid that conflation.

## This unit: H2, randomized stratification

PBRT's [stratified sampler](https://www.pbr-book.org/4ed/Sampling_and_Reconstruction/Stratified_Sampler)
partitions a sampling domain and jitters a uniform sample in every cell. Here
only the two-dimensional cosine-hemisphere domain is stratified:2×2,4×4,4×8,8×8.
The integral estimate is the equal-weight mean of all cells. Jitter is uniform
within each cell; no deterministic centre approximation, predictor, pilot,
history or sample rejection is used in these variants. This is prior art,
not a novel Ignis method or an implementation of the histogram-resampling paper.

Hold scenes, camera, ambient target, influencer filtering, raster and output
resolution fixed. Compare IID4/16/32/64, stratified4/16/32/64 and H1 fixed/learned/gated/
shared comparators, including their pilot queries. Capture four independently
seeded raw maps per method and a separate4096-query reference. Report foreground
AO error against this finite noisy reference, individual seeds, tiled paired
bootstrap intervals and total CPU/GPU timing. No denoiser or temporal accumulation.

Both existing scenes (gallery and offscreen-blocker guard) remain in scope.
All current limits remain: static receiver cache, analytic boxes, scalar AO,
no multi-bounce RGB transport, no engine lighting change or Lumen benchmark.
The next architecture-selection experiment must first establish a matched
one-bounce RGB oracle before choosing H3/H4/H5.

```sh
cargo build --locked --release --example lighting_gallery
./target/release/examples/lighting_gallery --sampling-audit
./target/release/examples/lighting_gallery --sampling-audit --offscreen-guard
uv run scripts/lighting_sampling_report.py
```

For N equal-area cells with independent jitter, write the per-cell mean as
`mu_k` and variance as `sigma_k^2`. Then
`Var(stratified)=sum_k sigma_k^2/N^2` and
`Var(IID)=Var(stratified)+Var_k(mu_k)/N`.
This is a statement about the ideal randomized estimator on this2D domain,
not a guaranteed wall-clock speedup, a claim about a particular four-seed image,
or a proof of higher-dimensional path-reuse correctness. GPU sampling overhead,
divergence and floating-point/PRNG behavior must still be checked.

A first H2 pass at4/16/64queries supported the equal-budget variance reduction
in both scenes. The same unit therefore adds a32-query4×8stratum test, with an
IID32 control, to directly test whether half the IID64 target-query count can
meet its approximate-reference quality. This is a follow-up selected by the
initial evidence; do not treat it as a preregistered global superiority claim.

## Measurement boundaries

Steady-state GPU measurements include native shadow/main passes and the AO
compute. Setup/cache construction, image readback and four quality snapshots
are outside frame timing, as disclosed in each raw record/report. Shared cache
storage is retained to run H1 controls, but the IID/stratified estimators do not
use predictions in their estimates or fit weights or require that cache to define their estimates.
This unit does not demonstrate a compact production memory implementation.

Quality is measured on camera-visible receivers only; background pixels are
checked separately rather than diluting MSE. The reference has4096 independent-
stream IID queries per receiver, so error against it includes reference noise.
Four query seeds per method are retained. Confidence intervals resample64spatial
tiles with all four seeds kept together, conditional on these two scenes.
They are not estimates of robustness across arbitrary worlds, motion or devices.
An independent CPU implementation checks80receivers for each of the eight
plain/stratified variants per scene, jitter-stratum coverage and exact budgets.

Execution audit: an early run used the prior executable while Cargo was still
building. Its images/maps matched prior full-scan evidence, but it refreshed
baseline timing output. That run is not used for the H2 comparison. Both H2
scenes and the ordinary gallery were rerun after confirmed compilation;
argument validation and result-mode/method-set assertions now reject accidental
experiment mismatches. `artifacts/lighting-gallery` is the refreshed current
baseline, while published historical numbers remain in their earlier documents.
No old executable or alternate app bundle is retained.

## H2 results and decision

gallery: IID64 versus stratified32 (4096-query reference, four independent evaluation seeds): MSE 0.0022726→0.0014910; ratio 1.524× with spatial-block CI95[1.429,1.622]. GPU compute 1.146→0.639ms, native+compute 2.079→1.567ms. CPU medians 0.417→0.365ms are descriptive only; shared-machine CPU is not a demonstrated speedup.

guard: IID64 versus stratified32 (4096-query reference, four independent evaluation seeds): MSE 0.0024780→0.0013116; ratio 1.889× with spatial-block CI95[1.775,2.009]. GPU compute 0.953→0.587ms, native+compute 2.140→1.770ms. CPU medians 0.487→0.489ms are descriptive only; shared-machine CPU is not a demonstrated speedup.

At16queries, stratification approximately halves MSE in both scenes; in this
experiment it is more accurate than every same-budget H1 comparator. Independent
CPU code matched640sample checks per scene across eight plain/stratified methods,
including complete jittered stratum coverage and exact query counts. Saved
quality maps are finite, and background is exactly1. Profiled/default native
and all-visible composite PNG checks passed on both runs. Actual repeated GPU
runs use fenced/monotonic timestamps; all methods have128timing samples.

Decision: promote randomized stratification to a comparison baseline, not a
production GI architecture or novelty claim. Keep H1 available for harder RGB
transport/cache tasks. Treat H3 sparse representations, H4 reservoir reuse and
H5 learned representations as untested choices. The next architecture experiment
must add a shared one-bounce RGB transport oracle and compare a plain sampled
baseline against sparse probes before growing into cascades or reservoirs.
Do not infer animation robustness, scene-general quality thresholds, many-light
sampling, memory reduction, equal-time denoised quality or superiority to Lumen.

Artifact: `artifacts/lighting-sampling/review.html`, comparison.png, summary.json,
raw timings, four current quality maps per method and4096-query reference per
scene. The current example executable remains at the same Cargo path. No GUI,
production shader/renderer, scene format, asset, app installer or Cargo dependency
was changed by this unit. Raw maps are retained in bounded current experiment
outputs; no binary snapshots were created. Strict unknown-argument rejection
was checked before resource creation. The final release example build, both GPU runs, Rustfmt/diff/Python F checks
and final release example Clippy -Dwarnings with the documented unrelated
collapsible_if lint isolated passed. A temporary recheck failure in concurrent
botany edits was resolved by that work's own ongoing changes before the final
retry; no botany file was edited or staged by this unit. No all-workspace test
success is claimed, and no source lint allowance was added.

## H3 checkpoint

The [one-bounce RGB experiment](lighting-one-bounce.md) supplies a shared
sun-lit secondary-surface transport target and finite high-sample reference.
Direct H2 sampling is compared with two simple six-axis probe grids, with and
without receiver-to-probe visibility gating. This is the first sparse-spatial
comparison in this program; it is not an implementation of Split RC, DDGI,
SHaRC or ReSTIR, and the map above must not treat those architectures as tested.
The native direct raster remains unchanged; the extra bounce is an explicit
headless diagnostic composite, not a production renderer feature.

## H3b / H6 checkpoint

The [adaptive allocation experiment](lighting-hybrid.md) compares geometry and
probe-disagreement heuristics with independent spatial reconstruction. Mean
error alone is insufficient: sparse-probe hybrids can improve global error yet
worsen a local tile. GPU lane divergence can also consume nominal ray savings.
The experiment therefore reports global/near-wall/p95/worst-tile error and
measures primary-map setup, guide construction, GPU list compaction, indirect
dispatch and filtering. Policy selection is frozen before fresh heldouts.
This remains a diagnostic one-bounce experiment, not production GI, ReSTIR,
DDGI, a novel estimator or a comparison demonstrating superiority to Lumen.

## H7 checkpoint

Build-time irradiance is independent of the camera, but depends on the assembled
scene geometry/materials and light direction. A single asset bake does not
encode arbitrary future placement or world occlusion. The diagnostic persists
and verifies the full-scene dependency, reuses camera changes and channel-linear
sun scaling, and falls back exactly on invalidation. Static bake and geometric
correction are competing policies; correction can add more sampling variance
than the spatial bake error it was intended to avoid. Numerical quality gates,
local failure at low resolution, memory and new views decide that tradeoff.
This is surface light baking, not PRT or a new transport algorithm. Regional
invalidation and world-scale streaming remain separate implementation work.

## H7 regional repair checkpoint

The [regional repair diagnostic](lighting-regional.md) records sampled-path
support and conservatively classifies changed charts/texels, including new
occlusion and off-camera blockers. GPU-produced lists repair a private atlas;
pending dirty support uses fresh64, then publishes atomically in queue order.
All24sequential events match full atlas/dependency rebuild and published pixels.
This preserves the existing finite sampled bake; it is not a continuous-radiance
certificate, async engine scheduler or frame-budgeted dynamic GI. Material edits
save more than broad geometry changes; global sun direction still invalidates
all. Dependency storage and conservative false positives are explicit costs.
