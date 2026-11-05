# GPU Gems 3 applied to Ignis lighting — EAF-208

Correlated investigation,2026-11-05. Located the Codex chat **Ler e documentar
GPU Gems 3**, id`01a1a9ae-16ea-76e3-86db-d55c4a43981b`. That is the41-chapter,
six-part series the user referenced. This review targets PartII(ch8–13) and
PartIII(ch14–20), with ch24 as a color-space cross-check. It verifies selected
primary NVIDIA chapters and owning Ignis sources; it is not a claim that this
turn repeated the other chat's complete book reading. No renderer feature or
new GPU speedup is implemented by this review.

The strongest findings are **direct-to-indirect transfer(ch9)** for avoiding
material-only ray rebuilds, and **contribution-based work rejection(ch19's
pipeline principle)** for making existing lighting cheaper without adding GI.
Importance sampling(ch20), shadow resolution/filtering(ch10/8), vegetation(ch16)
and skin(ch14) are useful under different contracts. This is historical research,
not evidence that any technique is current frontier or faster than Lumen.

## What is already present

At this source checkpoint:

- `src/scene.wgsl` shades surfaces in the main geometry pipeline. It already has
  direct GGX/dielectric/metal response, thin backlighting, emission and IBL.
  `environment_light` reads prefiltered diffuse/specular cubemaps plus a DFG LUT.
- `src/environment.rs::sample/ggx_samples/cosine_samples` already uses the
  Hammersley-style `(i/N,reverse_bits(i))` construction, GGX directions and
  cosine-weighted directions during environment precomputation. Ch20 is not
  an entirely missing capability to add from scratch.
- `visibility` evaluates a3×3PCF loop,9comparison-filter calls. The main shading
  path requests visibility before deciding the final direct response. Native
  shadow projection is one explicit orthographic light volume in
  `src/render.rs::shadow_view_projection`; it is not the PSSM cascade of ch10.
- The RGB laboratory already uses cosine-weighted randomized stratification.
  Recommending cosine importance sampling as a new diffuse breakthrough would
  misdescribe the existing baseline.
- [Bake](lighting-bake.md) stores one incident-light solution, and
  [regional repair](lighting-regional.md) tracks dependency masks/bounds but
  retraces4096samples in dirty texels. It does not store transfer weights.
- Thin transmission is already present; that is different from spatial skin
  diffusion. The main opaque target is sRGB UNORM; do not confuse an existing
  linear shading calculation with an HDR-capable complete composition chain.

## Chapter-by-chapter relevance

Each application/priority below is our engineering judgment, not a speedup
reported for Ignis by the source.

| Chapter and primary source | Useful mechanism | Application / boundary |
|---|---|---|
| [8 — Summed-Area Variance Shadow Maps](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-8-summed-area-variance-shadow-maps) | Filter depth moments; summed-area filtering for large footprints | Conditional shadow-quality candidate. Our current9PCF calls are a small kernel; extra moment/SAT construction, precision and memory can lose. Light bleeding across thin blockers is a required rejection case. Filtering is not a cure for badly allocated shadow resolution. |
| [9 — Interactive Cinematic Relighting](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-9-interactive-cinematic-relighting-global) | Separate direct illumination from precomputed indirect transfer; sparse/wavelet representation | Highest relevance to expensive cache updates. Adapt rows to world-space atlas texels, not fixed camera pixels. Geometry/material response assumptions and storage limits remain. Detailed derived experiment below. |
| [10 — Parallel-Split Shadow Maps](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-10-parallel-split-shadow-maps-programmable-gpus) | Allocate shadow sampling by view-depth partitions | High relevance to large worlds. Compare at fixed aggregate shadow-texel budget and quality, with movement/stability/caster coverage. More cascades can add passes and memory; this is not an automatic cost reduction. |
| [11 — Robust Shadow Volumes](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-11-efficient-and-robust-shadow-volumes-using) | Robust visibility volumes and hierarchical rejection | Keep robustness/culling lessons. Wholesale stencil-volume replacement has low immediate priority for dense foliage/complex geometry and soft shadows; it is a different visibility representation with its own costs. |
| [12 — High-Quality Ambient Occlusion](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-12-high-quality-ambient-occlusion) | Hierarchical surface approximation with near-detail correction | Useful for compact static AO/appearance and representation-error tests. Disk approximation artifacts/interpolation are not fixed merely by more samples. Scalar AO cannot substitute our RGB one-bounce transport or exact sun visibility. |
| [13 — Volumetric Light Scattering](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-13-volumetric-light-scattering-post-process) | Image-space light-shaft approximation | Optional visual effect, not an optimization of current lighting. Screen information cannot certify off-camera volumetric occlusion or replace global atmospheric transport. |
| [14 — Skin Rendering](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-14-advanced-techniques-realistic-real-time-skin) | Surface/specular response plus spatial subsurface diffusion | High visual relevance to characters. More GI rays do not provide the missing skin-scattering mechanism. Treat diffusion/extra passes as an added material feature, not free quality or a lighting-cost reduction. |
| [15 — Playable Universal Capture](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-15-playable-universal-capture) | Compress and reproduce captured facial appearance | Lower priority for a procedural lighting architecture; needs captured data and does not provide general editable-world transport. Compression ideas remain useful separately. |
| [16 — Crysis Vegetation](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-16-vegetation-procedural-animation-and-shading-crysis) | Cheap backlighting/thickness appearance, ambient/AO channels and shading-frequency choices | High visual/cost relevance to Ignis botany. Existing transmission is a starting point. Test thickness/normal/leaf-scale variation and vertex-vs-fragment error on moving close-up leaves; generic screen blur or global tint is not equivalent. Do not overwrite current per-vertex channel semantics. |
| [17 — Multiple Reflections/Refractions](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-17-robust-multiple-specular-reflections-and-refractions) | Secondary ray intersections in layered distance-map geometry | Relevant to the separate reflection/transmission track. Layer resolution/coverage/update costs and missing thin/hidden surfaces limit the representation; extra ray-march steps cannot restore absent layers. Not an immediate replacement for general geometric queries. |
| [18 — Relaxed Cone Stepping](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-18-relaxed-cone-stepping-relief-mapping) | Precomputed support accelerates height-field intersection/refinement | Useful for bark/stone relief and material self-shadow work. Ignis already has a parallax/material path. Add only if precomputed cone data and parameter invalidation beat that path; it does not automatically replace world geometry, collision or macro silhouette. |
| [19 — Deferred Shading](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-19-deferred-shading-tabula-rasa) | Separate material/visibility preparation from lighting; avoid shading hidden opaque fragments per light | High relevance as a work-selection principle. Our single-directional-light path is not evidence that a full G-buffer migration wins. Audit local zero-contribution work first; evaluate deferred/clustered organization separately when many local lights justify bandwidth/transparency costs. |
| [20 — GPU Importance Sampling](https://developer.nvidia.com/gpugems/gpugems3/part-iii-rendering/chapter-20-gpu-based-importance-sampling) | BRDF-aware PDFs, low-discrepancy distributions, filtered environment samples | Useful for glossy/environment transport and future directional predictors. Diffuse cosine and environment GGX/quasirandom sampling already exist. The chapter's environment formulation omits local occlusion; it does not solve our thin-blocker problem. Mip filtering trades bias for noise/aliasing. |

[Ch24 — The Importance of Being Linear](https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-24-importance-being-linear)
reinforces linear-light sums/filtering and correct display encoding. Our raw
GI errors are already scene-linear. Audit every texture/intermediate/alpha/HDR
boundary; applying gamma correction a second time would be a regression. Adding
an HDR target/tone mapper can improve range but changes memory/pass costs.

## Priority 0: optimize current lighting, without new GI

Ch19 motivates eliminating lighting work that cannot affect the result. A local
source candidate is the unconditional `visibility(in.world)` in the shaded
surface path. `dielectric` already rejects `N·L<=0`. For opaque, nontransmitting
matte surfaces with no positive direct lobe, the sun shadow result cannot change
their direct contribution. This is a derived optimization opportunity, not code
from ch19 and not a measured GPU saving.

Read-only counts from retained analytic receiver maps:

| View | Visible receivers | N·L<=0 | Fraction |
|---|---:|---:|---:|
| Original front |67168|0|0.0%|
| Reveal E |85714|33961|39.62%|
| Reveal F |81186|44847|55.24%|

These counts are eligibility for that flat matte diagnostic. They are NOT
counts of executed native shader invocations/texture reads or expected time
savings. Native overdraw, derivative/normal perturbation, GPU lane divergence,
branch cost and compiler optimization can change the outcome. The front view
also shows why one favorable camera is insufficient.

Experiment contract: exact same current native image with and without the
visibility shortcut, GI absent in both. Include zero/direct/ambient/emission/
IBL, dielectric/metal, normal perturbation, leaves with backlighting, glass and
front/back poses. The guard must follow all supported direct lobes; blindly
skipping shadow when the geometric normal faces away would break translucent
leaves. Receiver-plane shadow bias currently uses `dpdx/dpdy` inside visibility. Keep
those derivatives in valid uniform control flow (for example, prepare them
before a divergent rejection and guard only the sampling); do not silence
derivative-uniformity checks to force the shortcut. Preserve caster rendering
for offscreen objects. Measure shadow/main
GPU separately, CPU, total resources and moving-camera pixels. Retain the old
path as an oracle; reject if the complete pipeline does not improve.

This directly addresses the user's question about being cheaper than doing
nothing: it targets work already performed by the original lighting. Reducing
the extra GI cost is a different optimization axis.

## Priority 1: store response, not only the illuminated answer

Ch9's direct-to-indirect transfer is particularly relevant to our material
updates. The original system assumes a fixed view and fixed scene response;
its sparse matrices and wavelet truncation introduce representation tradeoffs.
It must not be imported as unrestricted camera/material/geometry support.
The [original SIGGRAPH2006 project](https://www.cs.cornell.edu/~kb/projects/directToIndirectRelighting/)
is also explicit about fixed viewpoint. Our proposed adaptation below is a
smaller ONE-BOUNCE specialization, not that full multibounce method.

For current constant-albedo Lambert boxes, fixed geometry, fixed sun direction,
fixed visibility and the existing N cosine-sampled paths, define

`w[i,j] = (1/N) sum_{k:first_hit(i,k)=j} max(n_hit(k)·sun,0) V_sun(hit(k))`.

Then the incident field and outgoing indirect radiance are

`I[i] = D ⊙ sum_j w[i,j] rho[j]`,

`L1[i] = rho[i] ⊙ I[i]`.

Here D is the sun RGB scale, rho[j] the secondary object's constant RGB albedo,
and w is scalar. Receiver cosine/PDF cancels because the existing sampler is
cosine-weighted. All ray offsets/miss behavior stay those of the laboratory.
For valid normalized normals/directions, `w>=0` and `sum_j w[i,j]<=1`
in exact arithmetic. These provide useful independent row/energy checks. A
coefficient error obeys `|delta I_c| <= |D_c| sum_j |delta w_j| |rho[j]_c|`;
with declared color/light bounds this can support a real quantization budget.

This algebra is our specialization of the transport target: with this contract,
color-only edits can recombine stored weights without tracing4096paths again.
Geometry or sun-direction changes invalidate affected weights; no free moving
occluders/day-night solution is claimed. Spatially varying/procedural materials,
changed roughness/BRDF/transmission or multiple bounces need a richer model.

Use atlas texels as response rows, preserving our camera independence. A later
generalization can use secondary surface gather samples and recompute their
direct lighting, but changing sun direction still requires correct new shadow
visibility and sufficient source sampling. Compressing away a thin occluder is
not made correct by a faster matrix multiply.

Storage planning from existing first-hit masks (not a constructed matrix):

| Scene | Mean hit objects/texel | Dense F32 scalar weights | Sparse planning upper bound |
|---|---:|---:|---:|
| Open,49152texels/8objects |2.720|1572864B(1.500MiB)|1266052B(1.207MiB)|
| Thin-wall,55296texels/9objects |2.929|1990656B(1.898MiB)|1517036B(1.447MiB)|

Dense is `texels*objects*4`. Sparse is `(texels+1)*4 + hit_entries*8`, for a U32
row-offset table and U32object/F32weight entries. The hit mask includes unlit or
shadowed hits, so it bounds rather than proves nonzero coefficient count. This
excludes allocator padding, other caches, source data and existing dependencies.
Do not extrapolate tiny8/9-object storage to dense world-wide all-pairs matrices.

Falsifiable next experiment: dense scalar baseline first, then sparse storage if
needed; compare coefficient recombination against full4096sample rebuilds across
random RGB edits, zero channels, repeated edit cycles, receiver/source edits and
new camera views. Floating-point reassociation prevents assuming bit equality;
use an independently accumulated reference and explicit max/RGB/local error
limits. Recompute from current colors instead of accumulating unbounded deltas.
Charge construction, coefficient memory/reads, update latency, publication and
steady lookup. Geometry repair must regenerate both response and dependency
records; stale versions cannot publish over newer edits. No new performance
prediction is made by this investigation.

## Other useful choices and rejection rules

- Transfer compression(ch9) is worth studying AFTER a correct response baseline.
  Lossy radiance coefficients need an error budget, for example bounding the
  contribution of discarded weights using a declared input-light envelope.
  Dependency bounds are different: shrinkage could miss influences. Quantize
  those outward or compress losslessly, accepting extra false positives.
- PSSM(ch10) can improve close/far shadow allocation. First compare total texels,
  pass/caster work and temporal stability. A cascade's receivers may be camera
  visible while their casters are not; use light influence volumes, not camera
  visibility alone. Existing planetary restrictions are separate contracts.
- Filtered importance sampling(ch20) is a comparator for glossy HDR environments,
  not a promise of fewer rays for already cosine-sampled diffuse boxes. Keep
  support/PDF weighting, seams, thin blockers and highlight/temporal error tests.
- Vegetation(ch16) and skin(ch14) may offer more visible improvement than adding
  diffuse GI samples when the missing phenomenon is thin transmission or skin
  diffusion. They are quality features with cost, and require native material
  ablations/artistic references rather than only box-scene MSE.

## Experimental order and baseline accounting

1. Current renderer unchanged, new GI absent: establish original cost/image.
2. Native zero-contribution rejection: compare to1, exact pixels and full cost.
3. Current baked GI and regional repair: measure incremental cost above1/2.
4. Response coefficients for supported color edits: compare to3at matched target.
5. Larger-world shadow allocation and material-specific quality experiments.

A faster GI implementation still adds some cost over its feature-off baseline.
Require feature-off paths to avoid unnecessary allocations/passes. Distinguish
native raster, added GI query, changed-region update, offline construction,
CPU preparation, GPU intervals and actually reserved memory. GPU Gems is not a
substitute for measurements on our current wgpu/Metal path; its historical API
code, hardware timings and sample counts are not Ignis performance guarantees.

Reproduce the read-only counts/storage estimates:
`uv run scripts/review_gpu_gems_lighting.py` →
`artifacts/lighting-gems-review/evidence.json`, including source/input hashes.
The script uses retained datasets, not a new renderer build. Runtime, scene
formats, assets and app windows are unchanged. No new GPU test or new speedup is
claimed. EAF-208 remains In Progress; this is the completed applicability review.
