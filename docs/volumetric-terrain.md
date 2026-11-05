# Volumetric terrain — GPU Gems 3 reference

## Construction inventory before geometry

Reference: GPU Gems 3 chapter 1, figure 1-15, user-supplied 250×186 image.
The crop artifacts/volumetric-terrain/reference-ledges.png is enlarged 4× for
inspection, without claiming additional detail. No metric scale is available.

- V01: connected earth mass filling the lower view; not separate floating mesas.
- V02: several vertically separated irregular shelves. Exact count is occluded;
  at least three elevation bands are distinguishable in the foreground/centre.
- V03: relatively flat upper surfaces, with irregular angular/rounded contours.
- V04: darker recessed cliff faces under projecting edges; multiple intersections
  along some vertical lines require a volumetric field rather than a height map.
- V05: lateral breaks and erosion-like detail at smaller scale than shelf spacing.
- V06: larger undulations and raised background masses; shelves remain part of
  the same continuous field as ordinary terrain.
- V07: green upper faces, warmer exposed walls and dark cavities make structure
  legible; image colour alone does not identify rock type or erosion physics.
- V08: oblique elevated view with visible horizon/background. Backside and
  underground geometry are unknown; no invented exact reconstruction.

All items initially not constructed / model verification not executed.

## Source study

Ryan Geiss, Generating Complex Procedural Terrains Using the GPU:
https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-1-generating-complex-procedural-terrains-using-gpu

The official companion ZIP was inspected with HTTP byte ranges, downloading only
its directory and density.h/tables.nma, not the 373 MB archive or executables.
The density.h shader actually contains smooth_snap, crusty shelf, flat and
spherical terraces, domain warp and nine noise octaves. The web chapter omits
these terrace formulas. Native code will implement this mathematics with our
own seeded noise; this is not byte-equivalent NVIDIA texture sampling.

Positive density means solid; zero is the surface. Density is not a distance.
Shelf height uses a mixture of original and warped altitude. A snapped periodic
altitude contributes both terrace density and an overhang displacement. Do not
replace this with stacked extrusions or quantized output height.

## Performance review before implementation

- Validate recipe and grid/work/output budgets before allocating or sampling.
- Compile recipe once; no JSON, strings, allocation or graph lookup per sample.
- Sample lattice corners once; use dense edge-index arrays instead of per-edge
  ordered maps. Share mesh vertices and index triangles.
- Retain only one current generated asset per publication; no frame regeneration.
- Cold generation and physics publication may cost time; never infer viewport FPS
  from mesh counts or capture wall time.
- Marching Cubes ambiguity, grid resolution, finite domain boundaries and future
  GPU/readback/streaming costs must remain explicit.

## Implemented contract (first native increment)

`QueryVolumeTerrain`, `PublishVolumeTerrain`, `QueryVolumeTerrainGpu`,
`PublishVolumeTerrainGpu`, `SaveVolumeTerrain`, `LoadVolumeTerrain` are available
through the shared MCP command schema. No new visual-only controls exist.
The source is compact typed CBOR in a versioned `IGNISVOL` envelope, extension
`.ignis-volume`; JSON is only command/authoring transport, not the sampling loop.
Publication uses the existing atomic scene edits, undo/redo and optional Rapier
mesh collision. Save the source separately: a Scene contains the baked mesh,
not its procedural recipe. No global Scene/replay version was changed.

A single sampler handles plane altitude and spherical radial altitude. Warp,
noise, terraces, shelf and floor are common operations. Authoring uses f64;
the current GPU backend explicitly rejects coordinates/base scale beyond8192
grid steps instead of silently losing the lattice. It is a local authoring tool,
not integration into Planet1 or the continuously resident planetary runtime.

For altitude h, warped altitude hw and terrace phase φ:

```
u = -(h + (hw-h)*warp)/spacing + phase
f = u-floor(u)
s = 0.5*(2*f)^m                  if f<=0.5
s = 1-0.5*(2*(1-f))^m            otherwise
density += (floor(u)+s-phase)*strength + (s-f)*overhang
```

The `overhang` term changes the density itself: it can produce several solid/air
crossings in one vertical. It does not move overlapping slabs. `shelf` blends
in a positive-density altitude band, which can also create excessive projections;
the final demonstration leaves this optional operation off.

Noise frequency is in native lattice cells/metre. To translate the NVIDIA
16³ normalized noise-volume coordinates, the demonstration multiplies its
published frequencies by16. Quintic native value noise, seeds and rotations
are intentionally different from the original texture data/interpolation.
The demonstration omits the three finest octaves unresolved by its0.5m lattice;
this is a declared sampling choice, not a complete reconstruction of the DVD demo.

## Pipeline and performance review after implementation

- GPU evaluates density with a cached WGSL compute pipeline; no generation per
  rendered frame. One explicit readback feeds CPU indexed Marching Cubes and
  the existing static mesh renderer/collider. CPU reference remains available.
- Each lattice corner is evaluated once. Dense per-axis edge caches share
  vertices; exact-zero endpoints share corner indices. No per-sample allocation,
  JSON or string lookup. Octave rotations/seeds are prepared once.
- At most2,200,000samples,600,000vertices and1,200,000triangles per cold operation.
  These are work/memory envelopes, not a world-size claim. Extent, finite input,
  device storage/dispatch and output checks precede publication; output overflow
  aborts without changing the scene. Empty blocks can be inspected; publishing
  an empty asset is explicitly refused.
- At the sample cap, CPU density/edge/corner scratch is about50.4MiB, plus mesh
  output, allocator capacity and renderer/physics costs. GPU sampling adds two
  f32 lattice buffers (about16.8MiB maximum). No RAM/FPS guarantee is inferred.
- Geometry is extracted once and retained as an Arc. There is no full-scene clone
  in the sampling loop. Scene transaction costs remain those of existing editing.
- Normals queried from density use six central samples. Published mesh shading
  currently uses the renderer's mesh normals; gradient normal streams and the
  chapter's directional density AO are not yet integrated.

## Evidence and visual review

Reproduce with `python3 scripts/volume_terrain_demo.py` (headless MCP, no window).
Current recipe: `examples/volume-terrain.recipe.json`. Stable outputs are under
`artifacts/volumetric-terrain/`, replacing prior attempts at the same paths.
The current patch is64×64m,48m vertical sampling range,0.5m cells;
1,614,177samples,35,265vertices and69,742triangles in the reviewed recipe.
49 vertical probes found three columns with three solid/air transitions,
confirming real reentrances. Source roundtrip, atomic invalid input, undo/redo,
Scene reload pixel equality, deterministic saved replay and grounded collider
contact were checked via MCP. Empty GPU grids were also inspected successfully.

Reference/render review: V01–V04 are supported by one extracted field, with
visible tiers, connected mass and recessed/projecting walls in both oblique
views. The finite sampling boundary is open and visible in the overview; it is
not an authored cliff or a complete world. Small isolated solid islands are
possible with a general density field and are not silently removed.
V05 remains too soft/coarse compared with the reference's finely crumbled edges;
V06 composition and V07 slope-dependent green/rock appearance are not reproduced.
The neutral render demonstrates the geometry, not artistic acceptance or an
identical copy of figure1-15. V08 camera is an inspection hypothesis, not solved
reference-camera alignment. No image-similarity percentage is asserted.

Remaining chapter/runtime work: fully GPU-resident indexed extraction, block
pool/empty-block caching and scheduling, compatible LOD transitions, gradient
normal streams, density AO, richer material blending, control maps/local flattening influences, continuous world/planet
adapters and visual refinement. Classic Marching Cubes has ambiguous face cases:
this increment exposes that limitation and does not certify arbitrary field
manifoldness, seam closure or sub-cell feature preservation. Collision represents
the extracted resolution, not exact analytic density. No erosion, hydrology or
tectonic simulation is implemented by these noise/terrace functions.

Validation of this increment:7focused Rust release tests passed (including real
headless GPU/reference comparisons for all density operators on plane and sphere,
maximum tested density error below0.001). Release lib/bin Clippy with warnings
as errors, focused rustfmt, Python syntax and the release executable build passed.
The real MCP demonstration passed source/baked Scene/replay checks, undo/redo,
atomic invalid-publication refusal, empty GPU queries and grounded physics.
This is not a claim that the entire repository test suite passed. The current
binary is target/release/ignis; no engine window was launched for this work.

## Larger regional experiment and timing interpretation

`scripts/volume_terrain_region.py` schedules a common source field over a4×4
layout of52m blocks (208×208m,10.5625times the previous area). Cell size remains
0.5m; each block samples104×192×104cells, including a96m vertical interval.
This grows the sampled domain, not the scale of the existing rocks. All geometry
comes from the native MCP publication. Read-only mesh inspection compares shared
boundary vertex sets and checks that no surface reaches the vertical cut planes.
Matching vertex sets alone does not certify ambiguous-face connectivity or
normal continuity. It remains a fully resident finite authoring experiment.

Performance checklist for this extension: reuse the GPU pipeline and source;
keep per-block sample buffers bounded; keep only boundary sets from read-only
inspection; do not regenerate geometry per frame. Timing adds a few `Instant`
reads per cold operation, not per sample/frame. No code rebuild is needed merely
to change seed or region size. Scene publication currently reuses the existing
transaction/physics path; repeated cold publications can cost more as the resident
scene grows. This is not asynchronous world streaming.

The MCP report now exposes wall times for combined density setup/dispatch,
synchronization/readback/conversion, CPU extraction/validation, and scene
publication including collision. These are host wall times, not GPU timestamps;
density execution and readback are intentionally not presented as isolated
measurements. The client additionally records full MCP latency, serialization,
captures and saving separately. Captures include rendering/readback/PNG work and
are not frame-rate measurements. A first query can include renderer/pipeline
initialization; subsequent queries still regenerate the field and mesh.

Measured on this Mac for the208×208m run:397,237vertices/782,522triangles,
24shared boundaries with identical vertex sets, no upper/lower clipping,
grounded contact and pixel-identical Scene reload. Original64m patch benchmark:
first MCP call98.05ms, warmed median15.87ms (still regenerates, not cached mesh).
Across16larger blocks:225.47ms density/readback,195.73ms CPU extraction,
3377.13ms scene publication including collision. Total publication MCP calls
3802.11ms; including separate read-only geometry inspections4363.86ms. Saving
89.78ms; captures148.53–224.47ms each. These are one observed run, not universal
budgets or GPU timestamps. Most cold authoring cost here is the current separate
scene transaction/physics publication per block. A future batch/incremental
publication path should address that before claiming streaming readiness.

Review of overview and ledges captures: the extended region contains different
features from the same global source, with tiers and reentrances at the original
scale. Neutral materials, open outer boundaries, occasional disconnected density
islands and soft detail remain visible; larger area is not artistic acceptance.
Recipe:examples/volume-terrain-region.recipe.json; current captures and timing
report:artifacts/volumetric-terrain-region/. Static review, release lib/bin Clippy,
focused rustfmt, Python syntax and the real MCP run validate this extension;
no terrain mathematics or persisted Scene/source format changed.

## Feature scale correction: kilometre landscape

The208m experiment expanded the area without correcting the geological feature
scale. The user's correction sets this map at approximately10×10km (100km²).
Feature width, altitude and extraction resolution are separate controls.

Volume source2 adds optional `noise_domain_scale` (positive XYZ coefficients,
0.0001..10000, default1). Source1 remains readable and rejects non-unit values;
unit values are omitted on serialization, preserving legacy source encoding.
Both noise and warp use the scaled domain; warp displacement uses a prepared
inverse scale. Plane/sphere base geometry remains defined in world coordinates,
not turned into an ellipsoid. XYZ scales are Cartesian, not automatic planetary
surface coordinates. All controls exist in the typed MCP API first.

The kilometre recipe maps the208m reference field into10km horizontally
(48.0769×) and8× vertically. It sets domain coefficients to
[0.0208,0.125,0.0208], scales vertical density strength, terrace spacing and floor
altitude/transition consistently. This preserves the density under the stated
plane-coordinate mapping rather than merely increasing the count of small hills.
Walls are tens of metres high; broad formations span hundreds of metres or more.
Existing material frequency remains a separate metre-based appearance control.
A new independently authored fine geometry layer is not implied by this mapping.

Grid `vertical_spacing` defaults to the existing isotropic spacing. A50m
horizontal/8m vertical grid supplies the landscape overview without sampling
100km² every0.5m. It is a deliberately coarse overview, not full near-player
resolution or dynamic LOD. All grid/cell/output budgets remain per operation;
the96m laboratory and the10km landscape use the same extractor.

Pre-code performance review: domain vectors/reciprocals are prepared once,
no per-sample allocation/division/JSON; only fixed vector multiplications are
added. GPU parameters grow by32bytes, not per-vertex data. Anisotropic lattice
positions retain shared indices and use the same CPU/GPU steps. Grid count
multiplication is checked, including32-bit hosts. GPU rejects noise coordinates
beyond its local precision envelope rather than casting oversized lattices.

The10km inspection also exposed legacy perspective depth precision: a5cm near
plane is unsuitable for an overview kilometres from the terrain. `capture` now
accepts an optional validated `clip` (near/far/reverse_z), using the existing
renderer projection support. Omission preserves the previous capture path. The
option affects only that offscreen capture; it does not mutate Scene camera,
physics, gameplay viewport or saved state. Planet runtime overrides are refused
to preserve its projection/atmospheric contract. The overview uses100m near,
local landscape inspection20m near, both30km far and ordinary depth. A runtime
large-world camera clipping policy is still separate work, not silently enabled.

Validation:9focused release Rust tests passed, including exact mapped-plane
values under independent horizontal/vertical scale, anisotropic plane extraction,
source1 byte-stable encoding and real GPU/reference agreement for scaled plane
and sphere. The headless10km MCP fixture produced94,272vertices/182,714triangles,
with elevation range-216.79..130.43m,24matching boundary vertex sets, no vertical
cutting, grounded contact and pixel-identical clipped Scene reload. Legacy source1
was loaded/re-encoded byte-exact through MCP. Invalid capture clipping was refused;
explicit legacy near/far matched default capture pixels; capture did not change
Scene camera. Current artefacts live in artifacts/volumetric-terrain-landscape/;
recipe in examples/volume-terrain-landscape.recipe.json.

Visual review: macro contours now span the intended geographical area rather
than adding more small hills. The oblique view shows broad terraces with roughly
62m altitude spacing in the source. Neutral materials and coarse overview sampling
still limit appearance; local high-resolution terrain and automatic transitions
have not been delivered by this experiment. No screenshot is artistic acceptance.

Final validation for this scale increment: release executable build, release
lib/bin Clippy with warnings denied, focused rustfmt, Python syntax and the
headless fixture passed. The final run repeated the same geometry, grounded
contact, legacy-source bytes and clipped capture controls;16block publications
including collision took816.28ms, or955.92ms including separate read-only mesh
checks. This is the coarse overview result, not uniform metre-resolution100km²
or a near-player streaming/performance claim. No engine window was opened.
