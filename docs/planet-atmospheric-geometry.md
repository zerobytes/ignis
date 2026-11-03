# Atmospheric geometry in the planetary atlas

The atlas partitions each tile into 4×4 blocks and may replace the **interior**
of each block with a fan. A whole tile often crosses the limb, so its complete
projection cannot safely be simplified as one disk.
Every block/tile boundary vertex, its high/low position, normal and material coordinates
remain byte-identical to the current source tile. The atlas quadtree, physical
grid, terrain sampler, atmosphere density and authored scene are unchanged.
This is a derivative of the resident visual mesh, not a change to the source.

`SetPlanetView.settings.atmosphere_geometry:true` enables this opt-in selection; `false` (the default)
retains the complete visual source as an oracle. `atmosphere_optimization:false`
disables both atmospheric material substitution and geometry selection. Existing
settings documents receive the defaults through the session settings reader;
no Scene, replay, content or network version changes. The existing demo keeps
its original geometry path unless this new setting is explicitly enabled.

## Admission and coverage

Extinction alone cannot justify moving a silhouette. Selection requires both:

1. The finite spherical medium bounds color contribution below 5% of the
   existing `color_error`. Its radial envelope uses the **global** terrain
   upper bound, including terrain that a depth change might expose behind the
   tile. Optical endpoint displacement and reconstruction guards are charged
   before evaluating the bound; vacuum is never charged as air.
2. The complete source disk projects with positive triangle orientation, every
   point of its bounding box is in front of a virtual camera aimed at the disk, and the replacement
   fan also projects positively with winding exactly one around its center.

The fan condition establishes a simple star-shaped projected perimeter. The
source is a triangulated disk with that same boundary and positive local
orientation, so its projected coverage equals the fan's coverage in exact
arithmetic. This preserves the terrain/sky silhouette and inter-tile seams.
The depth inside the disk may change. Any surface newly exposed behind it is
at least as far as the certified source envelope, with the global radial bound
covering its finite-air path. Color changes are bounded by the attenuated
surface/air contrast. This is not a certificate of identical depth buffers.

Failure of either condition retains the original tile. Clear air, uncertain
folds, terrain near the limb, near-plane crossings, procedural material
recipes, detail/surface layers, transparency, specular reflection, shadows,
environment light and atmosphere edge/layer effects use the original geometry.
There is no species rule, fixed distance cutoff or reduction of physics.

## Motion, retention and cost

The source plane test records an eye-centered ball in which all orientation
inequalities remain strictly positive. Reuse outside this ball requires a new
test; a failed plane also provides a safe rejection ball. The small fan and
its winding are checked for every selection in ray space, including ordinary
movement that does not replan the atlas. Rotating the real camera does not
invalidate this ray-coverage proof. Light changes are checked immediately. Cached
derivatives are keyed by the complete resource key and retain the actual source
Arc, preventing pointer reuse from authorizing a different source.

Only current resident sources and the one in-flight atlas retain candidates;
retiring or changing a source retires its candidate. Source arrays stay resident for fallback. A derivative
retains the boundary/center of admitted blocks and the original triangles of
other blocks. Only one derivative is cached per current source, rather than a history of masks. There is no worker, new GPU target/pass,
readback, JSON, disk access or lock in selection. Source plane tests are linear
in triangles when their certificate expires, fan checks are linear in boundary
vertices, and the warm selection allocates a small list of resource references.
Submission groups and GPU payloads change only when the selected Arcs change.

Patch membership/bounds are prepared with the source batch, but boundary edge
maps and fans are built only after extinction and source projection pass.
Preparation also primes the target-view derivative inside that batch. A rejected
tile retains the exact source and retries after 32 m of eye motion; explicit
settings/light changes invalidate those negative results immediately. This
amortizes unsuccessful optimization attempts, not visibility: the source stays
fully rendered. Positive candidates still revalidate their contribution, fan
and coverage for every eye.

The inspection object `atmospheric_geometry` separates source counts from
selected triangles/vertices, derivative builds, projection tests and current
cache size. These are not total RAM/VRAM or FPS measurements. Source generation during movement is separately paced as described below;
reducing GPU geometry alone does not bound region-generation pauses.

## Validation

`planet_orbit_atmospheric_*` checks exact seam attributes, oriented boundary
edges, source retention, warm reuse, explicit oracle restoration and light/
finite-vacuum invalidation. The ignored native GPU oracle separately compares
atmospheric RGB8 output and a neutral terrain/space mask over camera changes and
viewport shapes. It must exercise actual reduction; an empty selection cannot
pass as evidence. The integrated release gates include all 19 orbital tests and three explicit
native GPU oracles. Final build identity and movement limitations are recorded
below and in the generated reports. Human presentation/FPS acceptance is separate.

See [planet-pacing.md](planet-pacing.md) for the CPU movement methodology and
[planet-runtime.md](planet-runtime.md) for physics and residence boundaries.

## Generation pacing

Movement and `LookPlanetView` may prepare one candidate atlas in bounded batches
(default 8 tiles or 4 ms per update). The time limit is soft: a tile, sampler
budget and its patch partition are indivisible. An explicit `SetPlanetView`,
zoom or initial activation completes synchronously. `incremental_generation:
false` retains eager generation for comparison; `generation_tiles_per_update`
and `generation_time_ms` configure the batch limit through the same settings.

The installed atlas remains complete and conforming while the candidate is
prepared. No mixture of two seam registries is rendered. Resources shared with
the current atlas reuse their Arcs; only one candidate is retained, within the
existing 2M-vertex source limit in addition to the current source. Publication
swaps the complete set and retires old source/cache entries. An explicit set
cancels an obsolete candidate. A queued plan is rebased to the current eye/look
before publication, reusing ready resources only by the complete seam key. An
urgent fallback finishes the current target, never an obsolete queued view;
rebase also retires cache entries outside the installed and candidate sets.

**Quality takes priority over the batch budget.** Deferral is admitted only
while the installed source's geometric pixel bound remains within the current
settings budget for the current eye. Otherwise generation completes eagerly;
`forced_synchronous_for_quality` exposes this fallback. A closed old mesh alone
would avoid holes but could lose visible detail, which is not acceptable.
Pending tiles/age, generated tiles and batch counts are exposed in inspection.
Planning, group publication, uploads and physical residency remain separate
costs; this is not an asynchronous worker or a hard frame-time guarantee.

## Native coverage and submission oracle

The six-view Metal oracle produced exact RGB8 output and equal nonempty neutral
terrain/space masks. Counts below separate resident atlas reduction from actual
post-frustum submission; fewer resident triangles do not necessarily reduce a
particular draw. Draw counts include the unchanged player body.

| Camera distance | Atlas triangles, source → selected | Drawn triangles, source → selected |
| --- | ---: | ---: |
| 4 m | 297,088 → 294,016 | 108,576 → 108,000 |
| 1 km | 297,088 → 217,664 | 24,896 → 24,896 |
| 20 km | 183,232 → 130,592 | 25,088 → 18,816 |
| 100 km | 107,328 → 107,328 | 25,216 → 25,216 |

The 20 km pair selected 870,288 → 682,128 GPU geometry bytes, with the same 12
draw calls. These are selected payload counts, not allocated VRAM, RAM totals,
measured GPU time or presentation FPS. The dense/folded and thin-shell fallback
remains part of the contract. `artifacts/planet-pacing/geometry.json` records
build identity, per-view atlas inspection and both renderer counters.

## Consumed position precision

The source pixel bound charges analytic deviation from the canonical reference
mesh plus twice the quantization error of its **consumed high/low position**.
The former bound charged local f32 proxy roundoff even though planetary clip
positions bypass that proxy. This could reject deferral without an equivalent
error in the actual input stream. Local proxy roundoff remains reported as
`max_source_cast_error_meters`; the separate
`max_consumed_coordinate_cast_error_meters` covers high/low input reconstruction.
Copied samples retain a conservative bound from their original resource.

This changes neither coordinates nor the 8 px default policy. It does not claim
a universal bound on GPU arithmetic, rasterization or presentation. The native
precision/coverage oracles remain separate, as do physical geometry and local
proxy use for material projection.

## Movement boundary discovered during validation

At 4 m and 2248×1580, the unchanged 768-tile Earth preset reports roughly
35 px of geometric bound for an 8 px request. Correcting consumed-position
quantization does not remove that analytic/capacity limitation. Consequently the
safe queue uses its urgent synchronous fallback in this preset; the unit does
**not** establish that its original motion hitches or presented FPS are fixed.
Source bounds/planning coverage must be improved before deferral can help those
frames without relaxing quality.

The first controlled six-run comparison exposed unnecessary CPU work in the
prototype's eager perimeter construction (roughly 140–144 ms run maximum versus
117 ms with geometry selection disabled). This motivated selective perimeter
construction and negative-result amortization. The earlier measured state is
kept as `horizon-motion-before.json`; the current run is `horizon-motion.json`.
The reports compare exact KCC poses/ticks/path across 276 samples per run,
separate geometry-off and synchronous-generation controls, and record sampled
compiler/benchmark interference. CPU time is not presentation FPS.

The derivative is **opt-in by default**: this preset currently reduces no tiles,
so its additional CPU validation cost does not earn a GPU saving. The new option
is not silently promoted as a universal performance improvement. The CPU example
and MCP verification explicitly enable it; `--geometry-reference` / `--geometry-off`
retain the original geometry. The final timing comparison was marked provisional
because unrelated compilation remained active; its physics equality and work
counts are valid, but no new CPU/FPS gain is claimed from those times.

Release validation for this unit: 19 orbital tests including three native Metal
oracles passed; source/normal/coordinate payloads, seams, finite vacuum,
reference restoration, delayed atlas publication and obsolete-target fallback
were covered. The final default is opt-in. MCP movement exercised 264 poses and
an explicit visual sync preserving tick/path; six CPU runs compared 276 exact
physical poses each, with 2,392.273523662746 m during the run segment. The earlier
2,402.94 m figure belongs to the preceding snapshot, not this measurement.

Final timing report build: `44c1e136894c4f95` (explicit geometry-on/off controls,
before the default-only opt-in change). It detected 80 samples with concurrent
compilation: times are provisional. Geometry-on run maxima were approximately
131–132 ms and geometry-off 125 ms; those observations are not evidence of a CPU
gain. The code removes eager rejected fans and warm seam-signature copies, but
retains source fallback and this limitation openly. The native geometry oracle
is rebuilt with the final settings and identifies its exact snapshot in JSON.

Retained local executable: `target/release/ignis`; diagnostic executable:
`target/release/examples/planet_motion_cost`. Both use stable replacement paths,
without a timestamped checkout/build or copied app. No human window or authored
scene was opened, restarted or modified by this unit. No publication is implied.
