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
