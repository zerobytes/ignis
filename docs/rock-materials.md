# Mountain rock materials

## Visual contract before implementation — EAF-217

The references test the tool's capabilities. Their pixels are never source data
for a material. Retain the supplied images only in the local review directory.

| Reference | Required structure | Material versus geometry |
| --- | --- | --- |
| Alpine gray faces | Large angular planes, intersecting joints, smaller chipped planes and fine crystalline grain | A material supplies local surface relief; mountain peaks and overhangs require source geometry |
| Dark fractured rock | Unequal interlocking blocks, deep narrow recesses, worn edges, pale inclusions and rusty deposits | Correlated height, mineral color, roughness and indirect cavity attenuation; shadows follow the light |
| Ochre cliff | Oblique plates and fractures, broad warm mineral variation, granular abrasion | Orientation and block aspect are controls; a palette alone is insufficient |
| Bedded sediment | Many thin wavy layers of unequal thickness/hardness, broken ledges and cross fractures | Layer structure affects height and color together; local relief remains a single-valued surface |

Extend the existing native material recipe with an optional `rock1` program.
Use seeded periodic fields, angular mineral domains, inclined face planes,
variable-width joints, stepped/wavy strata, erosion, pits and fine mineral grain.
Four authoring families share configurable operators rather than image-specific
branches. Preserve the accepted marble program and its exact numerical path.

The CPU bake and WGSL runtime evaluate the same fields. Existing fixed mip maps,
library2 files, per-instance seeds, editor/MCP transactions and explicit geometry
displacement remain the execution and reuse paths. Color is mineral albedo;
directional lighting is not stored in it. Indirect cavity attenuation is an
authored approximation, not a visibility or GI solver.

Relief budgets are in metres and stay within the existing 0.25m contract. Bump
changes normals; optional parallax adds bounded view/light sampling. Explicit
geometry baking is needed for a silhouette/world-shadow demonstration and uses
the existing source-preserving transaction. No automatic tessellation, geological
simulation, cave/overhang generation or terrain/planet adapter is implied.

## Static performance review before code

| Question | Decision |
| --- | --- |
| Frequency | Program parameters compile on authoring changes; runtime field work is per visible fragment or explicit relief sample; CPU baking is a cold command |
| Allocation | Fixed stack intermediates in evaluators; immutable parameter/sample Arcs reuse existing caches and history; no field allocation in a fragment/evaluation |
| Representation | Small typed parameter block; existing 48-byte surface samples, material ABI, vertex/instance layout and GPU binding remain unchanged |
| Repeated work | Share hash/periodic-noise semantics and existing map/program packing; reuse field values across height, albedo, roughness and cavities |
| Limits | Fixed bounded cell neighborhoods and noise octaves; integer frequencies and finite 0..1 controls; fixed 16..1024 and aggregate device binding limits unchanged |
| GPU/I/O | One existing procedural buffer; no new pass, readback, file/network access or geometry generation in a frame |
| Concurrency | No worker, lock, dependency or shared mutable random stream |
| Invalidation | Source/material edits rebuild existing immutable program cache once; camera/light changes preserve material identity; fixed data remains authoritative |
| Correctness | Test periods/seeds, source conflicts/version gates, channels, lossless library persistence, CPU/GPU oracles, fixed/runtime and legacy materials; inspect native images under two lights |
| Retained costs | Synchronous high-resolution baking/encoding and explicit mesh displacement; expensive optional parallax; approximate footprint filtering and cavity attenuation; no FPS guarantee |

Implementation, current native evidence, controls and remaining limits follow
after the visual review. Preserve concurrent planet, character and renderer work.

### Native-review correction: per-pixel relief normals

The first native captures exposed 2x2 derivative-block artifacts on steep height
changes that were absent in the CPU finite-difference preview. Add a reusable
`NormalFilter::Pixel` option: two extra neighboring height evaluations supply
per-pixel forward differences instead of sharing a quad's gradient. Keep the
previous screen-derivative mode as the default for every old recipe/marble.
Rock presets choose the new mode; authors can choose the quality/cost tradeoff.
The existing relief header has a spare word, so no GPU ABI or buffer is added.
The shifted parallax coordinate, when active, is also the normal lookup domain.
Fixed maps reuse their filtered reads; runtime repeats bounded field evaluation.
Both the rock program and this opt-in normal mode require Scene117. This is
local normal sampling, not temporal antialiasing or geometry displacement.

## Authoring and reuse

Select a mesh and open **Aparência → Biblioteca de materiais**. Choose Rocha
alpina, Rocha escura fraturada, Paredão ocre or Estratos sedimentares and press
**Gerar rocha**. **Estrutura da rocha** exposes the mineral palettes, block
frequencies/aspect, integer direction, joint width/depth and secondary fractures,
faceting, layer density/warp/disorder, pits, erosion, grain and inclusions.
The usual seed, per-instance variation, planar rest/material domain, metre-scale
relief budget and ordered overlay layers remain available. **Normais rápidas**
uses the previous quad derivative path; **Normais suaves** selects per-pixel
height differences for a higher GPU cost. Both work with fixed and runtime data.

```json
{"command":{"op":"material","operation":{"kind":"rock","asset":"wall","family":"fractured","seed":4271,"unique_instances":true,"execution":"runtime"}}}
{"command":{"op":"material","operation":{"kind":"set_execution","asset":"wall","execution":{"fixed":{"resolution":1024}}}}}
{"command":{"op":"material","operation":{"kind":"save","asset":"wall","path":"materials/fractured.ignis-material"}}}
```

`generate` accepts a complete recipe for custom blends. Four compact source files
live under `assets/materials/rock/`; no photograph is embedded. `load` applies
one to another mesh. Fixed samples retain all linear channels and the editable
recipe; they remain authoritative until explicit regeneration. Shared runtime
instances can derive distinct stable seeds from their IDs.

Rock and marble are mutually exclusive base programs. Either supports up to
eight existing ordered overlays. The rock block is nine vec4 records (144 bytes),
following the unchanged 96-byte recipe header; fixed samples keep their 48-byte
layout and lossless library2 envelope. The opt-in normal flag uses the spare
fourth word in the relief header. Scene117/replay127/Content13 protect the new
source semantics; later shared versions must be preserved. Protocol5,
checkpoint3, portable-save1, terrain/planet source formats and physics are
separate contracts and are not modified by these materials.

## Native review workflow

```sh
cargo build --locked --release --bin ignis
python3 scripts/demo_rock_materials.py
```

Stable current outputs are in `artifacts/rock/`: `review.html`, native material
plates under two lights and ambient illumination, fixed libraries, parameter
variants, ID-derived instances, a geometry-displacement example and reusable
scene files. Add up to four `--reference` paths for local side-by-side review.
`--quick` only regenerates runtime views for iteration. The full run checks four
fixed 1024 library round trips and unchanged warm uploads. The 512x512 grid demo
explicitly invokes the existing displacement command, preserving the source and
turning off duplicate shader displacement on the generated mesh. Its world
shadows follow scene lighting; it is not a closed mountain/cave generator.

The dense authored grid is loaded through the existing buffered local-scene
reader, keeping the 2MiB MCP request envelope unchanged. Mesh density limits
the narrowest geometric joints; shadow bias is an explicit light parameter.

The full harness also compares the accepted green marble captures when their
prior local gallery is available. It never overwrites that gallery. No native
window or user's world is opened or replaced.

## Validation and retained costs — EAF-217

- Nine CPU and eight native GPU tests passed across `rock_materials`,
  `marble_materials`, `material_recipes` and `material_transparency` (17 distinct
  tests). Coverage includes seeded periodic sources, finite physical channels,
  conflicting programs/invalid controls rejected atomically, source-version
  gates, exact library round trips, overlays after both mineral program blocks,
  CPU/GPU color/cavity oracles, fixed 512 filtering, warm uploads, the independent
  normal-mode switch, light response and prior transparency/parallax routes.
- For four rock families at 512, native CPU/GPU mean RGB8 difference was
  0.01459–0.02416. Runtime/fixed 512 mean RGB8 difference was 0.91628–1.59546;
  different field/map filtering remains intentional and is not pixel equivalence.
- The full native gallery passed six checks. Four fixed 1024 files reloaded to
  byte-identical captured PNGs; subsequent warm captures kept uploads unchanged.
  Parameter variants share palette/seed while changing structure. Four ID-derived
  pieces share one source geometry. The explicit 512x512 mesh bake preserves the
  source and produces 263,169 vertices/524,288 triangles on the generated surface.
  Its shader height is zero afterward, avoiding duplicate displacement. Native
  flat, opposing-light, parameter, seeded, fixed and geometric views were reviewed.
- All three accepted green marble reference PNGs remained byte identical. The
  accepted marble gallery and sources were preserved.
- Current run: build `b11b867816ecade1`, binary SHA256
  `d02f8279189ccf6caa87cc91b901cbc3f2f40eacd97891ca6b676bf5cea418c3`.
  The shared native tree was Scene118 because concurrent particle work advances
  the global source; this feature's minimum stays 117. Source hashes and exact
  measurements are in `artifacts/rock/verification.json`.
- Explicit 1024 bake command round trips took 1.1384–1.1916 seconds in this run,
  excluding subsequent encoding. Each map/mip chain has 67,108,848 sample bytes
  and used 67,108,944 GPU program/map bytes. Fixed file sizes were 28,820,291 /
  29,059,427 /30,436,728 /31,036,698 bytes. Runtime uses 240 bytes per distinct
  program, before material-table slots. These are not total memory or FPS.
- The shared release build succeeded at `target/release/ignis`. MCP render-cache
  regression passed 5 checks, the general laboratory 11 and replay inspection 4.
  The broader selection scenario failed its hidden-trigger outline assertion:
  both views had 9 boxes/10 entities and one selected guide, but selecting the hidden
  trigger did not change the image. Its diagnostic scene has zero procedural
  materials. The failure and native metadata are retained under
  `artifacts/rock/regressions/interaction`; this unit does not alter selection.
- Whole-tree tests/strict Clippy were attempted but stopped during concurrent
  render/character/particle edits: an undefined `a` and missing internal
  `position_coordinates` initializer in `render_instances`, missing `field_blends`
  in a character test initializer, and an unused mutable particle variable.
  Whole-tree formatting also found a concurrent meadow-example diff. The owned
  material modules, UI and test formatting passed. These results do not claim a
  passing whole-repository build/test/lint state or pristine isolated-commit run.

Final static review keeps the ten preflight decisions: evaluators allocate no
heap or mutable RNG state; nine parameter vectors and the existing header/sample
layout reuse one procedural buffer and caches. Pixel normals add two bounded
neighbor evaluations only when selected and depth is nonzero. Baking, file
encoding/loading and geometry generation remain explicit synchronous command
costs. No extra render pass, worker, dependency, per-instance stride, frame I/O,
collision adapter or readback was added. The first oversized inline grid request
was corrected to use local scene loading; the 2MiB MCP envelope was preserved.
Only the current release location and current review outputs are retained;
superseded numerical prototypes are removed after native inspection.

These are authored rock families. Runtime octave/lamination filtering and cavity
attenuation are approximations, with no temporal stability/FPS guarantee. Dense
heightfield geometry still samples finite vertices and cannot express overhangs;
shadow-map resolution and bias remain light settings. Fixed maps retain their
real memory/read cost. GUI input, cross-platform/hardware coverage, geologic
calibration and human artistic acceptance are separate from these headless tests.
