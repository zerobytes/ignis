# Build-time diffuse lighting experiment — EAF-208

Preflight 2026-11-04. This adds a persistent object-space surface atlas to the
existing one-bounce matte-box laboratory. It is a diagnostic bake pipeline,
not a production lightmap format, general UV unwrapping or PRT implementation.
The transport target/native direct raster remains the same as
[the RGB laboratory](lighting-one-bounce.md).

## Hypothesis and fixed comparison

Precompute incident one-bounce irradiance on six separate charts per box,
16/32/64 texels per chart edge,4096 stratified samples per texel. Camera is not
part of the bake. Store linear RGB32F plus validity32F, independently per
primitive/face; bilinear interpolation never crosses a face or primitive.
Interior chart samples covered by another box are invalid. Ignore invalid
interpolation taps, renormalize, and trace64 if no tap survives. Clamp chart
coordinates to texel centers; this is biased near chart boundaries.

Compare fresh direct64, the previous compact20 adaptive reconstruction, the
baked lookup and a guarded hybrid. Hybrid traces64 where distance to the edge
of the receiver's face or another box is <=1.5times the largest world-space
texel dimension on that face. The guard is geometric, not a reference-derived
radiance error bound; shadow discontinuities inside a chart can still fail.
The three atlas resolutions are calibration candidates; guard1.5 is fixed.

Use four query seeds and two independent4096-sample reference maps, and the
same global/near-wall/p95/worst-spatial-region mean-squared RGB criteria. The
last metric is the worst of an8×8partition of the image (64regions, not8pixel
square tiles), with at least32foreground pixels per region. Select the cheapest
eligible hybrid resolution across open/thin-wall calibration scenes before
running newly revealed camera/light/geometry cases. If none passes, report
failure rather than silently treating visual similarity as matched quality.

Fresh validation cases are fixed in source: camera(-5.8,5.2,10.6) looking at
(0,0.7,-0.3); RGB light scaling(0.45,0.8,1.1); sun direction(0.45,-0.8,-0.25);
last object translated(+1.7,0,-1.2); blue wall recolored(0.8,0.12,0.03).
For open, the last object is the small box; for thin-wall it is the partition.
These are changed static states, not continuous animation or temporal history.

## Validity and persistence

The bounded package has a version, kernel SHA256, exact geometry/material
fingerprint, normalized sun direction, original RGB light scale, atlas shape,
ray budget and payload checksum. Camera changes do not invalidate it. For this
one-bounce, fixed-geometry/fixed-material target, irradiance is channel-linear
in the sun scale, so RGB intensity/color changes can reuse and rescale the bake.
A previously zero channel becoming nonzero cannot be recovered and is rejected.
Direction, geometry or material changes invalidate the entire bake and hybrid
falls back to fresh64. Regional invalidation is intentionally not claimed.
An explicitly labeled stale control bypasses the validity policy to quantify
what would go wrong. Full fallback must match direct64 RGB exactly per seed.

Package writes use a stable temporary file then rename; the decoder checks
magic/version/shape/length, checksum and finite/nonnegative samples. Truncated
and corrupted bytes must be rejected. Stable current files16/32/64 support the
active scientific resolution comparison; they are distinct representations,
not historical executable/build snapshots. No timestamped executable copies.

## Costs and boundaries

Report GPU bake time, CPU submission, wall time, serialization/file bytes,
load/validation time and upload wall time separately. The runtime graph always
rebuilds the primary map, even with the bake; native raster + primary + query/
guard/fallback is measured end to end using fenced timestamps,64samples per
method in rotated order. Readback and PNGs are outside runtime timing. Probe
control/guide/list buffers retained by the shared laboratory are not minimal
production memory; disclose atlas bytes separately. No zero-cost GPU claim.

Build-time work uses all six charts including off-camera surfaces. During
execution, query only camera-visible receivers, while off-camera geometry
remains a valid light/shadow influencer. The bake is world/object-space data,
not reuse of a frozen screen image. Dynamic invalidation is correctness-first
and may eliminate the speed advantage until rebaking; no async/regional repair
or production asset streaming is implemented.

Background is exactly zero indirect RGB. Assess quality in scene-linear RGB,
not clamped PNGs. The finite reference has noise. Record kernel/source/executable
hashes, native profiling identity and exact invalidation checks. CPU independent
transport and interpolation checks should accompany GPU execution. No engine
scene format, default material, renderer behavior, GUI or dependency changes.

Execution note: the initial RGB scaling case used a blue factor1.2, which
exceeded the native light-color0..1 contract and was rejected before its
reference ran. It was corrected to1.1; the complete experiment is rerun with
the corrected source. No failed light-color run supplies reported timings.

## Follow-up selected before fresh E/F

The first guarded-hybrid policy passed initial camera/scaling/invalidation
states. Unexpectedly, valid pure32/64 atlas lookups were both cheaper and more
accurate there; injecting fresh64 near geometric edges added variance. Pure16
failed a local region despite good mean error. Promote pure lookup to an actual
policy candidate, not merely a control. Initial camera/scaling views are now
exploratory evidence for this follow-up, not fresh confirmation.

Select among baked/hybrid ×16/32/64 under the same four quality gates in base
calibration; among GPU ratios within5%of the fastest eligible worst-scene ratio,
prefer the smallest atlas. This explicit timing tie recognizes memory cost and
shared-workstation noise. Freeze before new E/F: E camera(-7,3.6,8.9), target
(-0.2,0.5,-1), RGB scale(0.8,0.4,0.9); F camera(8,3.5,-9), target(0,1,0), unchanged
light. Both reuse exactly the prebuilt object-space charts. No retuning on E/F.
Initial selection/summary are retained as scientific metadata; final selection
and fresh E/F results are reported separately. Invalid states retain the exact
full64 fallback already checked. This is not local repair or dynamic PRT.

## Interpretation and remaining scope

A bake represents this assembled scene, including transforms and other objects'
shadow/indirect influence. Procedural content generated only during gameplay
cannot be fully baked during application compilation. A future regional cook or
background cache needs actual generated content and dependency tracking. Baking
an isolated tree would not precompute its illumination in every future forest.

The field uses one fixed4096-ray bake seed. Four query snapshots vary fresh64
and compact20 samples, and hybrid correction; they do not supply four independent
bakes. Bright atlas texels on all six normals are checked against independent
CPU ray transport with f32 accumulation matching GPU summation precision.
This makes numerical comparisons conditional on these scenes and this bake.
Opaque Lambertian boxes and a single indirect bounce do not establish arbitrary
materials, characters, specular/transmission, motion stability or human artistic
acceptance. The production renderer remains unchanged.

This follows established light-baking practice, not a novelty claim; see
[Epic's GPU Lightmass description](https://dev.epicgames.com/documentation/en-us/unreal-engine/gpu-lightmass-global-illumination-in-unreal-engine).
[Precomputed radiance transfer](https://www.microsoft.com/en-us/research/publication/precomputed-radiance-transfer-real-time-rendering-dynamic-low-frequency-lighting-environments/)
precomputes a response to varying lighting; this experiment instead stores a
specific incident-light solution and only supports its exact RGB scaling case.
No directional transfer matrix is implemented.

## Final result — 2026-11-04

Selected policy: pure32×32-per-face atlas lookup while the package is valid;
otherwise fresh64. Guarded correction remains an independent measured control.
The16atlas fails local-region quality (worst-region MSE1.27×open/1.79×thin-wall),
so its faster lookup is not promoted.32passes all four criteria and offers a
smaller representation than64;64remains the higher-quality/larger option.
Fresh E/F pass without retuning, including newly revealed backside charts.

Apple M4 Pro / Metal,480×320,64complete runtime samples per method/case. Table
includes native raster + regenerated primary map + lookup or fresh transport;
CPU is preparation/submission only. GPU costs must be compared within each row:
the shared workstation's power/load state varied materially across cases.

| Scene | Direct64 GPU ms | Compact20 GPU ms | Selected bake GPU ms | Direct64 CPU ms | Selected CPU ms | Global MSE ratio | Worst-region MSE ratio |
|---|---:|---:|---:|---:|---:|---:|---:|
| open/base | 2.057 | 1.592 | 0.860 | 0.508 | 0.530 | 0.164 | 0.469 |
| thin-wall/base | 4.115 | 2.934 | 1.360 | 0.242 | 0.251 | 0.207 | 0.536 |
| open/reveal-e | 1.779 | 1.608 | 0.849 | 0.270 | 0.286 | 0.115 | 0.438 |
| open/reveal-f | 2.582 | 2.078 | 1.203 | 0.257 | 0.262 | 0.203 | 0.579 |
| thin-wall/reveal-e | 2.598 | 2.104 | 0.980 | 0.249 | 0.256 | 0.096 | 0.419 |
| thin-wall/reveal-f | 2.761 | 1.890 | 1.126 | 0.314 | 0.304 | 0.201 | 0.576 |

In these six comparisons GPU runtime falls52.3–67.0%against direct64, and
40.4–53.7%against compact20. Global MSE is79.3–90.4%lower than direct64. CPU
changes are small/descriptive (approximately−0.010..+0.022ms), not a demonstrated
CPU speedup. These are matched laboratory measurements, not engine/game FPS,
world-scale performance or superiority to Lumen.

Selected atlas payload is786432bytes open /884736bytes thin-wall (0.75/0.844MiB),
plus372bytes package header. Final validation allocates only the selected atlas
capacity; calibration retains capacity for its largest active candidate. The
shared diagnostic also allocates existing receiver/output/readback and H6/probe
control resources; these atlas figures are not total GPU/CPU memory.

Selected bake GPU work takes33.25/42.04ms respectively, measured separately from
runtime; wall35.29/45.19ms for dispatch/fence/timing, excluding tool/device/
pipeline initialization, atlas readback and serialization. Current local file
load/check and upload are recorded separately in JSON; they are not cold-disk
or device-startup benchmarks. This tiny8/9-box scene is not a build-time forecast
for a full game. Prebuilding changes when that cost is paid. Frequent invalidation
can erase the benefit until a new bake is ready.

The package survives camera changes and permitted RGB light scaling. Changed
sun direction, object position and material are rejected; fresh64 fallback RGB
must equal the baseline exactly for every saved seed. Stale controls expose
large errors (including roughly50×global MSE for changed sun in the first
control cohort). No regional rebuild or continuous animation is implied.

Validation:36bright nonzero atlas texels (six normals ×three resolutions ×two
scenes) independently CPU-traced with4096samples each, maxRGB difference
2.98e−8; all-pixel CPU interpolation/scaling and geometric fallback checks,
finite/nonnegative/zero-background maps, native profile identity, exact invalid
fallbacks, package roundtrip/corruption/truncation and zero-channel reuse guards.
Four query snapshots use one fixed bake seed, as disclosed above. Release build,
scoped release Clippy(-Dwarnings with pre-existing unrelated collapsible_if lint
isolated), Rustfmt2024 and Python F checks passed. A temporary final-check failure
in concurrent botany work was resolved by that work before the successful retry;
no botany source was changed or staged here. No all-workspace test claim.

Reproduce: `uv run scripts/run_lighting_bake.py`. The runner checks a successful
current build, manifests, initial hybrid selection/controls, final bake selection,
selected-resolution validity controls, then fresh E/F. Current artifacts:
`artifacts/lighting-bake/{review.html,summary.json,selection.json}`, initial cohort
metadata, per-scene packages/raw maps/manifests. One current example executable,
no GUI/app bundle or historical binary copies. EAF-208 remains In Progress.

Next implementation boundary: regional dependency/invalidation and replacement
of changed bake regions, followed by real triangle/UV surfaces, textured/curved
receivers and moving actors. This unit provides a measured persistent bake
prototype; it does not install baked lighting into the production renderer.
