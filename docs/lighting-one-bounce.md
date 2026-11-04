# One-bounce RGB transport and sparse probe baseline — EAF-208

Experiment completed 2026-11-04. H3 begins with a simple sparse spatial field, not a
radiance-cascade, DDGI, SHaRC, ReSTIR or neural implementation. H2 cosine-domain
stratification remains the direct-sampling comparator. No production shader,
scene format, GUI, materials, app binary or dependencies are changed.

## Shared transport contract

For matte RGB boxes, let the directly lit secondary surface be
`B(y)=rho(y)*D*max(dot(n_y,sun),0)*V_sun(y)`.
D is the native directional-light radiance scale, equivalently sun irradiance
pi*D under the Lambertian convention. For the camera-visible receiver,
`I(x,n)=integral B(first_hit(x,w))*cos(n,w)/pi dw`, and the added outgoing
radiance is `L1=rho(x)*I`. A miss returns zero (black environment). This is the
sun→secondary-surface→receiver→camera contribution: one diffuse indirect bounce.
No emission, extra bounces, specular transport, transmission or participating
medium. See [PBRT LTE](https://www.pbr-book.org/4ed/Light_Transport_I_Surface_Reflection/The_Light_Transport_Equation).

Every estimator uses identical analytic ray/box geometry, secondary shading and
unbounded shadow/transport visibility. The prior4m AO cutoff is NOT reused.
The first positive primary hit creates each receiver; unseen geometry remains
eligible as an indirect/shadow influence. Origins use0.002m normal offsets in
this bounded laboratory. The mathematical target is the offset-ray transport.

Direct-only display is the actual native Ignis raster with ambient/environment
zero, unchanged direct light/shadow settings. The added RGB term is composited
onto its decoded linear output; native shadow-map differences/quantization are
common to all variants, not errors charged to the GI estimator. Store raw
unclamped L1 separately. Display uses the same SDR sRGB transform/clamping for
all images; assess physical error before display clipping.

## Comparators and scope

- Cosine-stratified16 and64 target rays per visible receiver, fresh every frame.
- Two independent4096-ray stratified maps; their mean supplies a finite noisy
  reference, and their difference supplies a convergence/noise-floor diagnostic.
- Coarse1m and fine0.5m world grids, fixed vertical phase0.125m above the floor.
  Each active probe stores six cosine integrals for the six axis normals, each
  built with256stratified target rays. This represents the complete normal set
  of these boxes; it is not an arbitrary-direction/general-mesh representation.
- Trilinear interpolation of eight probes, ignoring solid-interior probes,
  normalizing surviving weights. Lookup coordinates clamp to the explicit grid
  domain; this introduces approximation near its boundary and must be disclosed.
- Visibility-gated interpolation rejects probes occluded from the actual
  receiver offset. It adds up to eight analytic segment traces per query.
  Zero surviving weight falls back to the16-ray direct estimator; charge it.

There is no ray culling by camera frustum. Scene A is an open colored-wall room;
scene B adds an opaque thin partition. A roof/overhang provides shaded receivers.
Probes are frozen during performance samples. Geometry/light changes require a
rebuild; dynamic invalidation and camera movement are not claimed by this unit.

## Measurement and validation plan

Reuse targets, uniform/storage/query buffers and two probe fields. Keep shader/
resource construction, primary-map creation, each grid build, and camera-frame
queries separate. Native raster and compute use fenced pass timestamps. CPU
frame preparation/encode/upload/submission excludes waiting/readback/PNG.
Rotate method order and reverse the second round,128 samples per method. Record
all samples, median/p95, probe bytes and invalid count; no FPS claim.

Use four independent query/probe-build seeds for quality. Report foreground
RGB MSE/signed error against the mean reference, reference-pair difference,
near-partition error, fallback frequency and display clipping separately.
CPU independent ray transport must match sampled pixels. Check direct-only
passthrough, nonnegative finite energy, zero sun after rebuilding probes,
channel isolation, and constant-field interpolation/normal selection. Retain
cold build costs alongside query costs and show a rebuild-frequency break-even
instead of hiding construction in a static warm-frame result.

This is a bounded architecture experiment. A cheaper probe query is acceptable
only together with its bias/leak/detail/update evidence. No production adoption,
world-scale efficiency, general GI solution or Lumen superiority is inferred.

The thin partition is deliberately placed atx=0.27m, between both grids' nodes;
a wall aligned exactly with grid columns could hide interpolation leakage by
invalidating that column. This is an adversarial baseline, not a claim that
visibility-gated trilinear interpolation solves all thin-wall transport.

Probe construction receives a separate32-rebuild-per-grid timing study, with
coarse/fine order alternated. The four quality seeds are not used as the timing
population for reconstruction. Retain cold/quality records as well as calibration
median/p95/min/max; model per-frame and multi-frame amortization explicitly.
CPU probe interpolation and visibility are independently checked against the
last quality field read back before any calibration/oracle mutation. Six
synthetic ±axis receivers supplement the actual camera's visible normal set in
the constant-field check. Thus unobserved normal directions are genuinely tested.

## Measured result

M4Pro/Metal480×320,128query timings/method/scene; query median below excludes
field construction. Four independent field/query seeds measure quality against
the mean of two4096-ray references. RGBMSE averages visible pixels/channels.

| Scene | Method | CPU ms | GPU raster+query ms | GPU query ms | RGBMSE |
|---|---|---:|---:|---:|---:|
| open | Somente direta | 0.390 | 2.174 | 0.000 | 0.00217223 |
| open | RGB · 16 raios | 0.640 | 3.311 | 1.141 | 0.00017033 |
| open | RGB · 64 raios | 0.670 | 6.261 | 4.068 | 0.00002544 |
| open | Probes 1 m | 0.666 | 2.315 | 0.144 | 0.00017424 |
| open | Probes 1 m + visibilidade | 0.594 | 2.530 | 0.334 | 0.00012228 |
| open | Probes 0,5 m | 0.634 | 2.328 | 0.141 | 0.00003698 |
| open | Probes 0,5 m + visibilidade | 0.632 | 2.530 | 0.336 | 0.00002783 |
| thin-wall | Somente direta | 0.370 | 2.191 | 0.000 | 0.00234570 |
| thin-wall | RGB · 16 raios | 0.593 | 3.455 | 1.251 | 0.00018570 |
| thin-wall | RGB · 64 raios | 0.588 | 6.649 | 4.441 | 0.00002763 |
| thin-wall | Probes 1 m | 0.570 | 2.343 | 0.113 | 0.00028714 |
| thin-wall | Probes 1 m + visibilidade | 0.594 | 2.621 | 0.400 | 0.00018660 |
| thin-wall | Probes 0,5 m | 0.566 | 2.348 | 0.151 | 0.00007116 |
| thin-wall | Probes 0,5 m + visibilidade | 0.587 | 2.586 | 0.368 | 0.00003558 |

Construction context is material: values from tightly repeated rebuilds must
not be substituted for interleaved per-frame update costs.

open: fine field 372.09KiB (3969nodes,334invalid). Fine-grid build first=7.052ms;32-rebuild median=1.461ms/p95=1.547ms; three builds interleaved with quality queries median=6.801ms/range[6.746,6.808]ms. Summing that interleaved build and warm raster+query models 9.331ms if rebuilt every frame, versus direct64 6.261ms. This is a sum model, not a measured dynamic end-to-end frame or confidence interval. At these costs, about 1.82frames reuse amortizes build against direct64, before accounting for unequal quality.

Fine+visibility MSE/direct64=1.09× overall and 2.67× in the fixed near-partition band. The open scene uses that same spatial band for comparison, even though no partition is present. Reference-pair RGBMSE=1.4e-07; estimated mean-reference noise MSE=3.49e-08. This checks finite-reference noise, not exact convergence.

thin-wall: fine field 372.09KiB (3969nodes,334invalid). Fine-grid build first=1.749ms;32-rebuild median=1.601ms/p95=1.616ms; three builds interleaved with quality queries median=7.447ms/range[7.396,7.483]ms. Summing that interleaved build and warm raster+query models 10.032ms if rebuilt every frame, versus direct64 6.649ms. This is a sum model, not a measured dynamic end-to-end frame or confidence interval. At these costs, about 1.83frames reuse amortizes build against direct64, before accounting for unequal quality.

Fine+visibility MSE/direct64=1.29× overall and 2.53× in the fixed near-partition band. The open scene uses that same spatial band for comparison, even though no partition is present. Reference-pair RGBMSE=1.49e-07; estimated mean-reference noise MSE=3.72e-08. This checks finite-reference noise, not exact convergence.

## Decision and limits

Sparse probes make this static query much cheaper, but trilinear irradiance
interpolation introduces spatial error. Visibility gating helps the thin-wall
case without matching direct64 local accuracy. A visible receiver-to-probe
segment does not imply equivalent incoming radiance/occlusion in every sampled
direction. A tiny whole-image error can hide a persistent local defect.
Do not adopt this probe representation as the sole production GI path.

The next bounded question is a visibility-aware validity/reconstruction or
measured hybrid allocation policy near discontinuities. Keep direct stratified
sampling as fallback and retain a bias/error comparison; adding probe samples
alone does not remove interpolation bias. Any H1 residual correction needs a
compatible angular incident representation: six cosine integrals are not a
per-direction predictor with an automatically known correction integral.

Independent CPU checks passed128transport samples per scene (maxRGB difference
4.67e-8), plus48independently evaluated probe-build lobes per scene (maxRGB difference
1.27e-6), and256open/264thin-wall probe interpolation/visibility samples
(maxRGB difference2.97e-8), including sampled zero-weight fallbacks. CPU probe
occupancy matches both GPU fields. Six synthetic normals and all actual visible
normals pass the constant-field/albedo oracle (max1.35e-8). Energy bounds,
nonnegativity/finiteness, background zero, zero-sun/rebuilt-field checks, channel
isolation, profiled/native and passthrough pixel identity passed. None of the
captured candidates clips the SDR display in these two scenes.

Final release build, actual repeated GPU runs, Rustfmt/diff/PythonF and release
example Clippy -Dwarnings with the previously documented unrelated
collapsible_if lint isolated passed. No source lint allowance, Cargo dependency,
production renderer/material/scene version/GUI/asset change was introduced.
The current diagnostic executable is `target/release/examples/lighting_bounce`;
no historical binary copies were created. Preserve unrelated game assets.

Reproduce:

```sh
cargo build --locked --release --example lighting_bounce
./target/release/examples/lighting_bounce
./target/release/examples/lighting_bounce --thin-wall
uv run scripts/lighting_bounce_report.py
```

Current evidence: `artifacts/lighting-bounce/review.html`, both comparison/error
plates, raw RGB/receiver/field maps, all query/build/reference timestamps,
CPU oracles, source and executable SHA256 in summary.json. Timing is on a shared
machine; no FPS, dynamic validity, world scaling, multi-bounce or Lumen claim.
