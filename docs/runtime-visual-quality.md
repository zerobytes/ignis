# Native runtime visual quality — EAF-208

The goal is excellent nearby objects and scalable scenes. The offline shrub
render is an art reference, not evidence of native quality or performance.
Asset production remains EAF-204; transparency is EAF-203; spatial LOD and
planetary rendering remain EAF-181/199. This milestone preserves their source assets, existing render features, world
and running window. Its opt-in shadow extension is versioned below.

## First diagnosis and implementation boundary

The reference is `artifacts/shrub-hero/hero.png`, a Cycles render. The existing
`native.png` uses different framing, illumination and materials. Its directional
shadow covers 36 metres at 2048 pixels: 17.58 mm per texel, for a shrub about
1.2 metres across. The normalized depth bias 0.001 corresponds to about 72 mm.
Ambient 0.40 plus an unoccluded distant probe fills the canopy cavities. Neither
image pair isolates missing renderer features.

Ignis already has GGX reflectance, diffuse leaf transmission, procedural bump,
surface detail and split-sum IBL. The offline scene additionally authors leaf
coordinates/variation, cuticle/veins, several area lights, local indirect light,
soft shadows, AgX and lens effects. These have not automatically become native
features. Glass transmission is a separate material model.

First milestone: a reproducible native look-development laboratory, controlled
light/shadow/material ablations and complete scene lighting controls in the
editor. Use existing `set_light` and `edit_mesh/set_material` contracts. Fix
configuration before choosing a new render pass. The lighting controls reuse
the existing format and shader; no promise of Cycles equivalence.

The controlled close-up then exposed PCF self-shadow stripes on oblique leaves
and the ground. Extend this milestone with opt-in receiver-plane depth bias:
derive depth per shadow UV from geometric screen derivatives, correct each PCF
tap, and bound the uncertainty inside the hardware comparison footprint. Keep
the old arithmetic when disabled. This is a planar local approximation, not soft
area lighting. [Microsoft's primary description](https://learn.microsoft.com/en-us/windows/desktop/DxTechArts/cascaded-shadow-maps#calculating-a-per-texel-depth-bias-with-ddx-and-ddy-for-large-pcfs)
explains per-texel derivative correction; the implementation here is native WGSL.

Static extension review: reuse the unused shadow uniform component, with no ABI
growth, new targets, extra texture reads or geometry changes. Derivatives run
before shadow-frustum branches. Degenerate/ill-conditioned Jacobians use the
constant-bias fallback; bound depth corrections for grazing receivers. The
additional arithmetic is opt-in and must be timed, not assumed free. Persist
the opt-in through the existing Light/MCP/editor contract, with explicit gates;
default false omits the field and retains the legacy Light encoding.

## Static performance review before implementation

- UI works on one small cloned Light, emitting existing validated commands only
  after an edit. No geometry copies, renderer resources, locks or I/O in drawing.
- Directional shadow diagnostics are constant arithmetic in the UI, in metres;
  do not allocate or resize a shadow map merely to display a metric.
- The laboratory uses isolated headless processes, stable output paths and one
  current build. Reuse geometry/instances; no species-specific shader or asset
  mutation. Native captures include readback and encoding, so their wall time
  must not be presented as GPU time or runtime FPS.
- Separate cold generation/load/upload from repeated rendering. Any shader,
  pass, target or per-vertex channel extension needs a fresh review and its own
  measured quality/cost checkpoint.

## Subsequent milestones and gates

| Milestone | Decision and acceptance gate |
| --- | --- |
| Lighting baseline | Same geometry, camera and resolution; isolate ambient, IBL, shadows and reflectance; close/front/side/back and repeatable motion; preserve legacy images. |
| Surface authoring | Carry generic leaf/surface coordinates and stable per-element variation only if current channels fail; verify regeneration, deformation and distance filtering; no species switches. |
| Contact and indirect visibility | Evaluate screen-space ambient occlusion against probe/local visibility alternatives; protect silhouettes, thin leaves and temporal stability; separate indirect visibility from direct shadows. |
| Shadow quality | Fit stable coverage, depth/normal bias, filtering and later cascades/contact refinement; test grazing angles, moving light/camera and scale; quantify acne, detachment and shimmer. |
| Linear HDR and display | Scene-linear accumulation through transparency/atmosphere, exposure and a documented display transform after composition; preserve opt-out SDR and test bright emissive/glass/sky values. |
| Antialiasing and motion | Establish spatial sampling baseline, then evaluate temporal reconstruction with motion/disocclusion; inspect leaf shimmer, ghosting, resize and camera cuts. |
| Scene scaling | Hero, library specimen, small garden and progressive density using shared resources; preserve appearance before approximating LOD/coverage. Record CPU preparation, uploads, GPU, present cadence and memory separately. |

Raster and compute solutions with fallbacks are the initial scope. Ray tracing
may later be an optional measured path, never a prerequisite for using the
engine. Every milestone ends in native artifacts the user can judge; artistic
acceptance, automated validation, physical hardware coverage and delivery are
separate results.

## Reproduce this milestone

```sh
cargo build --release --bin ignis --example runtime_lookdev_cost
python3 scripts/runtime_lookdev.py
target/release/examples/runtime_lookdev_cost artifacts/runtime-lookdev/hero.scene.json artifacts/runtime-lookdev/hero-cost.json 1
target/release/examples/runtime_lookdev_cost artifacts/runtime-lookdev/library.scene.json artifacts/runtime-lookdev/library-cost.json 1 4 16 64
```

The hero input comes from `scripts/prepare_shrub_hero.py`; the laboratory reads
it and the existing library sources, without altering their curves or settings.
The saved scene's camera predates its original capture: the laboratory sets an
explicit matched pose instead. It converts the Cycles camera to Ignis Y-up,
using the same 85 mm / 36 mm square pinhole gate. Lights, lens effects, display
transform and materials are deliberately not claimed equivalent to Cycles.

Current native outputs are in `artifacts/runtime-lookdev/review.html`: original
lighting, shadow-only change, light/environment change, legacy versus corrected
PCF under the same material, close/side/back, no-shadow/no-IBL controls, eight
discrete camera samples per specimen, and 4/16/64-copy gardens. The slider is a
frame inspection tool, not a continuous gameplay or temporal AA measurement.
`hero.scene.json`, `library.scene.json` and `garden.scene.json` are editable
native scenes. One current executable is retained at `target/release/ignis`;
the profiling executable is the current Cargo example, without app snapshots.

The shadow member used in the hero and library look is:

```json
{"center":[0,0.52,0],"extent":1.15,"resolution":2048,"bias":0.00006,"receiver_plane_bias":true}
```

This gives about 1.12 mm per texel and 0.28 mm constant depth bias; the additional
receiver correction depends on geometry. The laboratory also reduces uniform
ambient from 0.40 to 0.015, the probe multiplier from 0.50 to 0.12, changes direct
lighting to 1.8 at direction [0.6,-1,-0.8], and assigns leaf transmission 0.12,
roughness 0.48 and F0 0.035. These are authored presentation choices, not new
defaults for games. The editor now exposes the existing full scene light and
the new shadow option, independently of the existing work-light override.

## Observations and evidence

- The dense native before/after uses exactly the same geometry and camera.
  Lower ambient and appropriately scaled shadows recover canopy depth. The
  optional receiver correction removes the obvious stripes in the controlled
  ground and leaf close-up. It does not supply the offline surface variation,
  indirect transport or area-light penumbra.
- Two planar GPU controls at 384² had 32,334 and 27,499 false-shadow pixels with
  constant bias; the opt-in correction reduced each to zero under the test's
  RGB8 tolerance. Vertical light remained zero; a separate elevated plate still
  cast a shadow. This is a bounded oracle, not a universal acne certificate.
- The legacy original-light hero and library captures remain SHA256 exact
  against the pre-change executable. Native save/load and return to the same
  camera after the motion sequence also reproduce the final PNG byte for byte.
  Provenance, source hashes and renderer counters are in `verification.json`.
- Visual inspection still shows hard shadows, uniform surfaces, thin-feature
  aliasing and an unoccluded environment. Library tessellation is visibly more
  angular in close-up than the hero. These are remaining quality limits, not
  evidence that geometry must always use the maximum tessellation.

The current cost run used Apple M4 Pro/Metal, 1200², build `9f157a7a6ef8b26f`, with
24 paired camera samples per case. Median draw plus blocking GPU completion,
legacy → receiver-plane, in milliseconds: hero 20.377 → 21.163; library one
7.703 → 8.934; four 10.365 → 10.311; sixteen 15.474 → 17.736; sixty-four
36.534 → 35.781. The correction is not free: this run includes increases of
about 4% for the hero and 15–16% for some library cases. Other cases are within
noise or lower; this does not establish a speedup or a universal overhead. CPU
preparation and submission medians were about 0.16–0.44 ms. The 64-copy case retains about
58.65 million source triangles before culling, with shared geometry.

The correction allocates no new render targets/geometry. The hero selected
geometry stream is about 497 MB; the one-copy library about 66 MB. Repeated
library assets can use shared indexed streams (~22.24 MB vertices + 10.56 MB
indices), while the benchmark's retained earlier representation raises allocated
geometry to ~88.22 MB. Selected, allocated and cumulative cache counters are
distinct. These are logical buffers, not total VRAM/RSS. Existing depth, IBL,
occlusion and driver allocations remain; the directional map alone is 16 MiB.

Reports separate JSON load, Engine construction, frame preparation, first draw,
warm CPU submit and warm draw/completion. There are no GPU timestamps, present
cadence or gameplay frames in this harness. No new window was opened and the
user's planetary process was preserved; other workloads on shared hardware are
not excluded. Cold frame/preparation costs must not be hidden in a warm number.

The current cost example scales the ground plate with garden size, matching the
gallery. The 43 native captures were also repeated on this build: both legacy
baselines, native save/load, camera return and source hashes still pass.

Validation checkpoint: the six directional-shadow tests (the native GPU
oracle is run explicitly with `--include-ignored`), material contract, two mesh persistence/catalog tests and public-content
boundary test passed on the integrated checkout. All nine existing GPU regressions explicitly marked ignored by the default
harness were also run serially and passed: three transparency, five mesh/cache/
visibility and one occlusion oracle. Total: 19 distinct tests, including ten
that exercise native GPU rendering. To reproduce, run the five targets listed
below normally, then with `-- --ignored --test-threads=1`; the new GPU oracle is
also ignored by default, preserving CI without a native GPU.

```sh
cargo test --release --test directional_shadows --test material_transparency --test mesh_resources --test render_occlusion --test network_content
cargo test --release --test directional_shadows --test material_transparency --test mesh_resources --test render_occlusion -- --ignored --test-threads=1
```

Release builds, all-target Clippy with warnings denied, formatting of the owned
files and Python compilation passed. Planetary work changed the shared checkout
during final builds; transient adapter/grouping errors were left with EAF-199,
and the targeted suite plus a final all-target Clippy passed once integrated.
The gallery/cost reports identify their actual successful build above. Native
UI interaction and artistic acceptance remain human review gates; EAF-208 stays
In Progress.

Next quality milestone: antialiasing and stable surface variation, judged in the
same nearby and garden poses before adding local indirect visibility. The
current eight-sample motion review shows why a sharp static screenshot alone
is insufficient. HDR/exposure and softer directional/area lighting remain
separate following milestones with compatibility and cost gates above.
