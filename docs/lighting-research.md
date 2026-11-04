# Lighting research — Lumens, EAF-208

Initial source audit: 2026-11-04. This is an investigation and experimental
contract, not a claim of improved runtime images or measured speedups.
Continue the existing laboratory in `runtime-visual-quality.md`.

## Mathematical target

For opaque surfaces, outgoing radiance is

`Lo(x,wo) = Le(x,wo) + integral_Omega fr(x,wi,wo) Li(x,wi) |n.wi| dwi`.

Visibility belongs in incident transport. Separate direct illumination,
indirect illumination, surface scattering and the display transform. Thin
transmission and participating media require their respective transport terms.
For an importance-sampled integral, estimate `I = mean(f(X)/p(X))`, with
nonzero sampling support wherever the integrand contributes. Report bias and
variance separately; spatial/temporal reuse requires valid weights and history
rejection. A low sample count alone proves neither accuracy nor low total cost.

Primary foundations: [PBRT surface transport](https://www.pbr-book.org/4ed/Light_Transport_I_Surface_Reflection)
and [Monte Carlo integration](https://www.pbr-book.org/4ed/Monte_Carlo_Integration).
[ReSTIR](https://research.nvidia.com/publication/2020-07_spatiotemporal-reservoir-resampling-real-time-ray-tracing-dynamic-direct)
is a candidate for many-light sampling with spatial/temporal reuse. Its paper's
performance results do not predict Ignis performance. One directional light
does not establish a need for a many-light reservoir implementation.

## Observations from the owning source

- `src/scene.wgsl::shade_surface` already evaluates directional visibility,
  GGX/dielectric response, metallic response, thin transmission, emission,
  procedural surface response and optional environment lighting. Investigate
  these implementations before proposing replacements.
- The initial diffuse expression is `base*(a+(1-a)*diffuse*visibility)`;
  metallic/dielectric branches also use `(1-a)` on direct lighting. Ambient
  therefore changes direct-light strength. At a=0.24 the coefficient is 0.76;
  at a=0.40 it is 0.60 (21.05% lower). This algebra is a configuration coupling,
  not proof that it dominates the rejected appearance.
- Uniform ambient is independent of local geometric occlusion; procedural AO
  is a material input. Direct shadow visibility does not by itself occlude
  ambient illumination. Test canopy cavities and ground contact separately.
- `src/render.rs` defaults offscreen output to Rgba8UnormSrgb; `fs_main`
  writes shaded color directly. This opaque output cannot retain HDR values
  above its normalized range for a later exposure operation. Audit transparency,
  atmosphere and every intermediate before changing the complete pipeline.
- Inspected directional/main render passes use `timestamp_writes: None`.
  Existing draw-plus-wait timings are not GPU pass timestamps. Establish
  adapter capabilities before introducing optional GPU queries.
- Birch demo sets ambient 0.24, direct intensity 1.15 and a 2048 shadow map
  with extent 12. Geometry and materials remain frozen during isolation.

## First experiments and falsifiable decisions

1. Preserve a baseline scene, camera, resolution, executable identity and input
   hashes in one stable output directory. Use front/side/back views and a fixed
   camera/light motion sequence. Warm resources before steady-state timings.
2. Sweep ambient 0/0.12/0.24/0.40 with IBL independently off/on. In a companion
   experiment compensate directional intensity by `(1-a0)/(1-a)` within its
   validated range. This isolates the direct attenuation coupling without
   modifying production shading. Record both parameter sets explicitly.
3. Sweep shadow off/on and receiver-plane correction independently. Inspect
   contact, detachment, grazing acne and temporal shimmer. Compare uniform
   neutral materials as a separate control; retain the original-material run.
4. Test bright neutral/emissive patches over several intensities to locate
   clipping in each composition path. Preserve scene-linear reference values;
   compare display images only through the same documented display transform.
5. Capture CPU preparation, upload bytes, memory and render/readback costs
   separately. Add capability-gated timestamps for shadow/main/composition
   before making per-pass GPU cost claims. Record device/backend/resolution,
   warmup, sample count, median and p95; repeat without competing GPU jobs.

Accept a change only with a quality/cost comparison, adversarial cases and
human review of native images. Numerical agreement does not establish artistic
acceptance. No numeric image-error threshold is selected before a matched
reference exists. Compare linear error against converged references where
available; report convergence and highlight/silhouette/temporal errors.

## Implementation order and cost review

First implement the controlled ablation runner and measurement manifest using
existing scene controls. Then choose independent direct/ambient controls,
linear HDR/display, or local indirect visibility according to the evidence.
Keep legacy rendering selectable and maintain shared MCP/editor semantics.
Ray queries need an explicit backend/capability audit, acceleration structure
build/update/memory budgets and a supported fallback. Evaluate shadows,
reflections and indirect transport as separate consumers.

The first runner should share immutable scene geometry, vary only Light/camera,
and reuse stable output paths. No per-frame scene cloning, new geometry,
texture allocation, file I/O or synchronization is authorized by a measurement
requirement alone. Timestamp queries must be asynchronous and optional; do not
force queue completion in normal gameplay. This document adds no runtime cost.

Status: source audit and experimental design complete; controlled native
captures, instrumentation and runtime improvements remain pending. No build,
window interaction or performance measurement was performed for this audit.
