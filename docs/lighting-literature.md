# Lighting at the research frontier: a program for Ignis

Targeted literature review, 2026-11-04, EAF-208 / Lumens. The objective is to
exceed competing real-time lighting systems on measured quality, temporal
response, GPU cost and memory. No superiority over Lumen has been established.
This review extends [the source audit](lighting-research.md) and
[the existing laboratory](runtime-visual-quality.md).

## Scope and evidence

Search covered primary author pages, papers, reference code and Epic's current
documentation, with particular attention to 2025-2026 developments. This is a
targeted engineering review, not an exhaustive systematic survey. Scientific
papers, production presentations and SDK implementations have different roles.
An arXiv posting alone is not evidence of peer review or production readiness.

The complete PDFs of Split Radiance Cascades v2, ReSTIR PT Enhanced and ReSTCV
were obtained. Relevant methods, evaluations and limitations were examined;
Split RC Table 2 (p9), Enhanced Table 1 (p12) and ReSTCV Figure 5 (p8) were
also rendered and visually checked. NIRC's full HTML methods/evaluation and the
LoD paper's mapping/limitations were examined. Other entries below are scoped
to their primary abstracts, author summaries or implementation documentation.
Their algorithms still require detailed reproduction before implementation.

## Lumen is a moving comparison target

Epic documents a hybrid system: screen traces, world representations, Surface
Cache and software distance-field or hardware triangle tracing. Missing cache
coverage and mismatches between representations are documented limitations.
The current performance guide also describes Lumen Lite's irradiance gather.
Therefore, simply combining probes and ray tracing is not a new contribution.
See [technical details](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-technical-details-in-unreal-engine).

The [performance guide](https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-performance-guide-for-unreal-engine)
gives console targets of 4/8 ms at 1080p internal resolution for GI/reflections
on opaque/translucent materials and volumetric fog, associated with 60/30 fps
profiles. These are published budgets, not our measurements or a universal
lighting cost. A 4K output reconstructed from 1080p is a distinct configuration
from native 4K. A cheaper diffuse-only pass would not establish superiority
over that combined system.

Production background: Daniel Wright's
[Radiance Caching for Real-Time Global Illumination, SIGGRAPH course 2021](https://advances.realtimerendering.com/s2021/index.html)
and the same course's GIBS presentation explain complementary screen/world
and surface-based caching choices. These are technical presentations, not
controlled head-to-head experiments against the 2026 papers.

## Correlated reading map

Each row connects a contribution to an Ignis question. "Candidate" means an
experiment is justified; it does not mean adoption or compatibility is proven.

| Work and primary source | Contribution and relationship | Ignis use and failure case |
| --- | --- | --- |
| [ReSTIR DI, Bitterli et al., TOG 2020](https://research.nvidia.com/publication/2020-07_spatiotemporal-reservoir-resampling-real-time-ray-tracing-dynamic-direct) | Reuses candidate light samples in space/time. Establishes the many-light sampling lineage. | Candidate for many dynamic lights/emissive triangles. One sun is not the workload that justifies this machinery; include reservoir traffic and shadow visibility cost. |
| [GRIS, Lin et al., TOG 2022](https://research.nvidia.com/publication/2022-07_generalized-resampled-importance-sampling-foundations-restir) | Extends resampling theory to correlated samples and path mappings. Underpins ReSTIR PT. | Mathematical prerequisite for reusing paths across procedural changes. A history counter is not an independent sample count or proof of convergence. |
| [DDGI Resampling, Majercik et al., CGF 2021](https://research.nvidia.com/publication/2021-12_dynamic-diffuse-global-illumination-resampling) | Combines sparse world probes with screen-space reservoirs. | Important prior art: the combination proposed here cannot be claimed novel merely because it mixes caches and resampling. Test local visibility at leaf/ground contact. |
| [GI-1.0, Boisse et al., 2023 manuscript](https://arxiv.org/abs/2310.19855) | Two-level radiance caching balances detail and update cost without content preprocessing. | Baseline for a procedural editor where users change geometry. Study screen/world cache coverage and newly revealed regions; benchmark newer implementation versions separately. |
| [Neural Radiance Caching, Mueller et al., SIGGRAPH 2021](https://research.nvidia.com/publication/2021-06_real-time-neural-radiance-caching-path-tracing) | Trains a radiance approximation online to truncate path tracing. | Candidate beyond conventional probes; training, inference and adaptation latency belong in the budget. Do not assume desktop accelerator costs transfer to Metal or other GPUs. |
| [Neural Two-Level Monte Carlo, Dereviannykh et al., EG/CGF 2025; preprint 2024](https://arxiv.org/html/2412.04634v1) | NIRC stores incident light and estimates a residual to correct approximation bias. Differs from simply terminating at a cache. | Strong conceptual basis for cheap broad illumination plus measured correction. The neural component is one representation choice; test a non-neural cache first as an Ignis hypothesis. |
| [SHaRC, NVIDIA reference implementation](https://github.com/NVIDIA-RTX/SHARC) | World-space hashed radiance lookup, distributed as shader code. | Practical non-neural cache comparator. It supplies cached outgoing light at hits; it is not identical to NIRC's incident field or Split RC's intervals. Audit its current guide and license before reuse. |
| [Histogram Stratification, Salaun et al., SIGGRAPH 2025](https://iribis.github.io/publication/2025_Stratified_Histogram_Resampling) | Organizes RIS candidates into local histograms and uses QMC/antithetic sampling. | Candidate for improving candidate selection after a correct sampler exists. Sampling-pattern benefits need tests during motion, not only static noise inspection. |
| [ReSTIR PT Enhanced, Lin/Kettunen/Wyman, I3D 2026](https://research.nvidia.com/labs/rtr/publication/lin2026restirptenhanced/) | Paired neighbors, footprint-based reconnection, duplication maps, unified DI/GI reservoirs and implementation improvements. | Main path-reuse baseline. Preserve separate unbiased and biased variants; evaluate bandwidth, divergence and chromatic/disocclusion noise. |
| [Stochastic Pairwise MIS, Hedstrom et al., EG/CGF 2026](https://research.nvidia.com/labs/rtr/publication/hedstrom2026stochastic/) | Enables unbiased spatial reuse from many neighbors, concentrating on contributing samples. | Candidate for freshly revealed regions with poor history. Compare improvement per extra neighbor query and memory access. |
| [Compatibility-Guided Neighbor Selection, Junkins et al., HPG 2026](https://research.nvidia.com/labs/rtr/publication/junkins2026compatibility/) | Selects neighbors with regard to path compatibility and the error/correlation tradeoff. | Important for overlapping foliage. Test jointly with paired reuse; their neighbor selection policies cannot simply be assumed additive. |
| [ReSTCV, Shi et al., SIGGRAPH 2026](https://hercier.github.io/restcv/) | Adds spatiotemporal control variates to reduce color variance left by scalar-target reservoirs. | Relevant to colored sunlight/sky and indirect color bleeding. Less benefit in nearly achromatic scenes; authors identify covariance-aware weighting and difference-target sampling as open directions. |
| [Split Radiance Cascades v2, Freeman/Sannikov/Pitt-Francis, arXiv revision 2026-10-11](https://arxiv.org/abs/2607.20384v2) | Sparse world probes and ray splitting distribute radiance into distance intervals, adapting spatial/angular resolution. | High-priority diffuse-GI prototype. Not a sharp-specular solution; interpolation leaks and sub-probe features remain. Publication status here is an arXiv preprint. |
| [Real-Time LoD Rendering with ReSTIR, Wang et al., SIGGRAPH 2026](https://research.nvidia.com/labs/rtr/publication/wang2026levelOfDetail/wang2026levelOfDetail.pdf) | Maps path vertices across geometry LoD changes to retain reuse. | Strong connection to procedural Ignis terrain/objects. Published method needs consistent texture mappings; mixed per-ray LoD and general topology changes are not solved by its implementation. |
| [MobileRC, Yu et al., CGF 2026](https://doi.org/10.1111/cgf.70537) | Locality-aware online plenoxel updates with SH angular coefficients, avoiding a neural MLP. | Hardware-oriented alternative to assuming fast matrix inference. The reported room-scale mobile experiments do not establish planetary or forest performance. |

## What the detailed results actually support

**Enhanced, Table 1 and Sections 7.1/7.4.** On RTX 5880 Ada, 1920x1080,
four-scene average: original code 35.73 ms; speed-oriented changes 13.04 ms;
all improvements 15.53 ms. The complete version is about 2.30x faster than that
baseline, not faster than Lumen by this factor. Reservoir storage changes from
208 to 128 bytes/pixel across two sets, about 431 to 265 decimal MB at 1080p.
Its decorrelation introduces bias; disabling it restores the paper's unbiased
variant. This is a reason to track bias and traffic explicitly.

**Split RC, Table 2 and Section 8.1.** RTX 3080 Laptop: 8.6 ms for Sponza and
11.5 ms for San Miguel, excluding G-buffer and shadow-ray work. The deposit,
merge and final shading cost matters alongside tracing. Most displayed
comparisons isolate a single indirect bounce. Remaining errors include leaks,
overblur and unresolved geometry below probe spacing; sharp mirrors are out
of scope. The temporal policy and ray allocation are identified improvement
opportunities. A fast 2D cascade demo cannot establish 3D forest performance.

**NIRC, Sections 4/6/7.** Tested at 1080p on RTX 3080; unbiased residual-corrected
and biased termination variants are distinct. Training and the angular field
representation are part of the method. Its residual principle is transferable
mathematically; a sparse non-neural version's performance is untested here.

**ReSTCV, Figure 5 and limitations.** Equal-time examples on RTX 5080, 1080p,
show cleaner chromatic estimates; some complete-frame examples remain tens
of milliseconds. Its strongest motivation is the mismatch between scalar
importance selection and RGB variance. It does not eliminate all noise or
prove a complete modern game's frame budget.

## Synthesis: the proposed research direction

Research hypothesis: **allocate light-transport work according to estimated
residual error, visibility discontinuities and temporal invalidation, while
retaining stable surface correspondence across procedural representations.**

This is a candidate architecture synthesized from the above work, not a claim
of a new algorithm. A novelty review and successful baselines are required
before attributing an original contribution to Ignis.

The layers should be independently replaceable:

1. Evaluate direct light and the material response with clear linear units.
   Establish HDR composition and a shared display transform for comparisons.
2. Approximate broad indirect light with a sparse world representation; compare
   a simple probe/hash baseline against Split RC. Choose incident versus outgoing
   radiance deliberately; their lookup and material-integration costs differ.
3. Trace corrective samples for high residuals, contact regions, thin blockers,
   disocclusions and changed light. Maintain nonzero exploration probability.
4. Reuse eligible samples with explicit path weights/mappings; study Enhanced,
   pairwise MIS, compatibility selection and ReSTCV as alternative ablations.
5. Map procedural surfaces across LoD/deformation or reject history explicitly.
   Surface IDs alone do not supply a valid invertible path mapping or Jacobian.

This intersects Ignis's procedural sources, content sharing and world residency.
Those systems may make useful cache identities available. They do not prove
transport reuse is valid. A local edit can change indirect light outside the
edited bounds; conservative dependency invalidation and periodic exploration
are necessary candidate mechanisms, not solved guarantees.

### A mathematical prototype before a renderer commitment

Let `f` denote a complete contribution integrand on a common direction/path
domain, including the appropriate BSDF, geometry and transport factors. Let
`g` be a frozen cheap approximation on that same domain. Then

`I = integral g(z) dz + integral (f(z)-g(z)) dz`.

A two-level estimator is

`I_hat = mean[g(X)/q(X)] + mean[(f(Y)-g(Y))/p(Y)]`.

The integrals must use the same domain/measure. Sampling densities must cover
the nonzero contributions; weights, path transformations and delta scattering
need their correct treatment. Freeze the cache for evaluation or establish
conditional correctness with independent samples. Updating a predictor with
the very residual samples used to evaluate it cannot be assumed unbiased.
Suppressing or clamping negative corrections also changes the estimator.

For independent batches with single-sample variances `Vg,Vr` and costs `cg,cr`,
`Var(I_hat) = Vg/ng + Vr/nr`, `C = ng*cg + nr*cr`.
Minimizing variance at fixed cost gives

`ng/nr = sqrt((Vg*cr)/(Vr*cg))`.

This identifies the actual opportunity: cheaper sampling helps only if the
approximation makes the expensive residual sufficiently predictable. This
allocation rule assumes independent batches and fixed variances/costs.
ReSTIR correlation, shared training and filtering invalidate that simple model;
measure covariance and derive the actual estimator before combining them.
Denoising/reconstruction bias is separate from estimator bias.

## Experiments that decide the architecture

| Experiment | Competing variants and evidence | Reject or revise when |
| --- | --- | --- |
| Residual proof of concept | Analytic integrals and a small converged transport scene; plain MC vs cache-only vs corrected cache, equal time and equal samples. | Correction fails to converge, variance exceeds plain MC, or prediction/update traffic consumes savings. |
| 3D indirect representation | Simple probes/hash vs Split RC under identical tracing/materials and final resolution; include every stage. | Leaks, sharp indirect structure or thin blockers stay unacceptable at the affordable density. |
| Surface/history validity | Static, wind deformation, topology regeneration, LoD switches, rebasing and camera cuts; mapping enabled vs history reset. | Reused light sticks to old geometry or mapped estimates drift from independently sampled references. |
| Colored light and reuse | Enhanced, ReSTCV and neighbor-selection variants; first evaluate separately, then controlled combinations. | Scalar error improves while chromatic noise, covariance, ghosting or GPU traffic worsens beyond budget. |
| Change response | Lights off/on, emissive object moved, door opened, newly exposed ground and dense canopy. | Mean static quality hides slow convergence or stale illumination. |
| Lumen comparison | Match engine version, content, material/light semantics, hardware, internal/output resolution, reconstruction and effect coverage. | A claimed gain depends on omitted reflections, lower resolution, absent update work or a different display transform. |

Keep the birch's geometry/materials/camera frozen in its lighting isolation
experiment. Add generic adversarial scenes: thin-wall room, colored emitters,
rough-to-mirror sweep, moving occluder, dense foliage and a terrain LoD boundary.
The approved warm/cool Horizon images remain art-direction references; they
are not radiometric ground truth or a matched Lumen benchmark.

Record native GPU stage timings, CPU preparation, bytes uploaded, peak memory,
cache occupancy/eviction and acceleration-structure build/update cost. Preserve
cold and steady-state results separately. Use repeated runs, warmup, sample
counts, median/p95, seeds and hardware/backend identities. Keep exploratory
rays so scheduling does not permanently miss newly relevant transport.

Report scene-linear relative errors, signed energy error, perceptual error and
temporal behavior. [FLIP](https://research.nvidia.com/publication/flip) provides
a perceptual difference map; the original version is LDR, so use the appropriate
HDR extension or a shared declared display transform. Static FLIP is not a
measure of ghosting or artistic approval. Measure settling time and errors on
freshly revealed regions; inspect moving native sequences and failure crops.

Dominance is a Pareto claim: improve one axis without an unacceptable regression
on others under a declared workload. Report separate quality-at-equal-time,
time-at-equal-quality and memory curves. No universal multiplier is inferred
from individual papers. We have not run Unreal or any head-to-head benchmark.

## Concrete continuation and implementation boundaries

First implement a small, independent estimator laboratory plus a machine-readable
measurement manifest. Reproduce plain MC and two-level correction before
integrating reservoirs. In parallel conceptually, instrument the existing native
lighting laboratory and establish the full HDR/visibility contract. These are
work packages, not instructions to launch additional agents or chats.

Then prototype the simplest sparse non-neural cache and compare Split RC on the
same toy workload. Only after its measured benefit survives thin geometry and
dynamic lighting should the architecture enter the production renderer. Audit
the current wgpu 27/Metal capability path before selecting a ray-query backend;
an algorithm needing ray intersections is not automatically an RTX-only
algorithm, and an available hardware feature is not an implemented Ignis API.

Pre-implementation cost review: bounded cache capacity/eviction, explicit bytes
per entry/reservoir, resource reuse, asynchronous instrumentation, no compulsory
GPU readback in gameplay, no per-frame whole-scene clones, and amortized
acceleration-structure updates. Shader languages, backend APIs and dependency
licenses are integration questions to resolve independently of paper quality.

This unit changes documentation only: no new render pass, asset, build, GPU
measurement or window interaction. Research artifacts do not complete EAF-208's
native implementation, movement validation or human acceptance criteria.

The subsequent [CPU estimator experiment](lighting-estimator-experiment.md)
implements the first numerical laboratory and records its findings and limits.
