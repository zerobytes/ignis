# Procedural marble — reference-driven quality pass

## Reference analysis and contract before implementation

The three user-supplied images are visual targets, not inputs to the generated
pixels. Keep those reference files local, outside published source/assets.

| Reference | Structure to reproduce | Surface implications |
| --- | --- | --- |
| Dark green breccia | Angular mineral masses, branching calcite fractures with varying width, cloudy green inclusions and fine grains | Polished and nearly flush; white veins are mineral filling, not gaps between paving stones |
| Emerald / white / gold | Broad diagonal cloudy white veins, finer white networks, thin gold filaments, folded emerald bands and dark regions | Calcite stays dielectric; metallic response is an explicit decorative inlay choice |
| Emerald / broad gold | Folded mineral bands in several green values, broad irregular gold networks, tiny gold threads, granular gold and fine matrix speckles | Separate matrix, calcite, gold, micrograin, roughness and microscopic height |

Build a configurable native mineral program with seeded, multi-scale domain warp,
clouds/strata, irregular fracture networks, broad principal veins, fine branches,
gold filling and micrograin. It must generate the same channels for runtime GPU
and CPU fixed baking. Additional color palettes span white/gray, white/gold,
black/white, black/gold, brown, red, pink and blue decorative marble families.
These are artistic families inspired by stone names, not mineralogical
identification, every quarry variety or an empirical geological simulation.

Classical layered marble based on perturbed strata is described in
[PBRT's primary texture reference](https://www.pbr-book.org/3ed-2018/Texture/Noise).
Here, strata alone are insufficient: the supplied images also require branching
fractures, varying mineral domains and multiple frequencies of detail.

## Static performance and integration review

- Reuse the material recipe, mesh rest domain, shared instancing, immutable sample
  ownership and MCP/editor transaction. No species/asset-name shader switch.
- Add an optional versioned marble program. Legacy recipes retain their math;
  marble compiles its small parameter block once. Runtime work is bounded field
  evaluation per visible fragment; no image synthesis or file I/O in a frame.
- CPU and WGSL share the same deterministic integer hash and operator semantics.
  Periodic domain warp preserves tiling. Suppress subpixel octaves and grain;
  filtering is an approximation and must be visually checked in motion/distance.
- Raise fixed bake resolution only as needed for close inspection. A complete
  1024 float sample mip chain costs 67,108,848 bytes before headers; GPU binding
  limits still apply to the aggregate surface buffer; multiple fixed materials
  must fit the effective device binding limit.
- Avoid enormous textual float arrays in library files: support a bounded native
  binary envelope with lossless PNG compression of canonical little-endian float
  bytes. Preserve JSON library1 input and exact decoded samples. Bound dimensions,
  metadata, decompression and file sizes before allocating. No new dependency.
- High-resolution baking/encoding remains explicit cold command work, with one
  current output location. Authoring sources are small and remain editable; no
  texture reference image is embedded in the recipe.
- Polished stone has microscopic height. Broad veins change color and material,
  not centimetre-scale relief. Gold-colored mineral and metallic inlay remain
  independently configurable; emission and light visibility are separate.
- Validation must include native flat/close material views, changing illumination,
  CPU/GPU agreement, seed stability, mip/binary persistence/invalid-data atomicity
  and legacy material regressions. Human artistic acceptance remains distinct.

## Authoring and material reuse

In **Aparência → Biblioteca de materiais**, choose a marble family and press
**Gerar mármore**. **Estrutura do mármore** exposes four linear palettes,
fracture density, vein widths, matrix deformation, folded strata, white/gold
strength, gold flow, breccia, grain scale and polish. Standard recipe seed,
per-instance variation, domain scale and ordered overlay layers still apply.
The 12 families are Verde brechado, Esmeralda branco/ouro, Esmeralda ouro,
Carrara, Calacatta, Statuario, Nero Marquina, Portoro, Emperador, Rosso Levanto,
Rosa and Azul. The presets vary structure as well as color, and remain editable.

The same MCP transaction creates a reusable material on an existing mesh asset:

```json
{"command":{"op":"material","operation":{"kind":"marble","asset":"slab","family":"emerald_white_gold","seed":931,"unique_instances":false,"execution":"runtime"}}}
{"command":{"op":"material","operation":{"kind":"set_execution","asset":"slab","execution":{"fixed":{"resolution":1024}}}}}
{"command":{"op":"material","operation":{"kind":"save","asset":"slab","path":"materials/emerald.fixed.ignis-material"}}}
```

Compact runtime sources are in `assets/materials/marble/`. Loading one onto
another mesh uses the existing `material/load` operation. A fixed asset retains
its exact float mip samples and editable recipe. Return to runtime and edit or
regenerate explicitly. Runtime instances can share one recipe and geometry while
using stable ID-derived seeds; fixed samples are shared until explicitly rebaked.
The planar domain can use authored material coordinates (as on the gallery vase)
or the selected rest-coordinate plane. This is not automatic UV unwrapping.

`marble1` is an optional program inside Recipe1, followed by the existing bounded
ordered overlay layers. Its eight vec4 parameter records add 128 bytes in runtime;
fixed maps keep the existing sample layout. Marble materials require Scene115,
replay125 and Content12 in the current integration. Higher shared versions must
be preserved; protocol5, checkpoint3 and PSV1 envelopes are independent.

Library2 fixed files begin with `IGNISMAT2`, a little-endian metadata length and
resolution, bounded JSON metadata and the PNG float-byte payload. The decoder
checks metadata size, resolution, expanded dimensions/layout and sample values.
No color quantization is used. Runtime JSON uses library version2; existing
library1 JSON is still accepted. Scene/replay serialization retains its own
format; the compressed library does not enlarge the network/replay limits.

## Reproduce the native review

```sh
cargo build --release --bin ignis
python3 scripts/demo_marble_materials.py
```

The headless MCP script replaces the current `artifacts/marble/review.html`,
flat native captures, illuminated curved meshes, unique seeded slabs, reusable
libraries and compact runtime example scenes. Add up to three `--reference`
paths for local side-by-side comparison; those files never feed the generator
or enter source control. `--fixed-resolution` accepts 512 or 1024 (default 1024).
Fixed data is baked one material at a time to respect aggregate device limits.

## Validation and remaining limits

Generation is bounded
but synchronous; large fixed bake/save/load commands may take noticeable time.
Runtime remains fragment work, particularly on large surfaces and with relief.
No async compilation, texture streaming, volumetric mineral transport, geological
simulation, every quarry variety or universal FPS guarantee is implied. Height
is microscopic (the presets use 40 micrometres of budget); broad veins do not make
paving grooves. Real world shadows and reflections come from scene lighting.
Human artistic acceptance remains separate from automated correctness.


### Native evidence — EAF-214

- Seven CPU and seven native GPU checks passed across `marble_materials`,
  `material_recipes` and `material_transparency` (14 distinct tests). They cover
  all 12 family sources, periodicity/max32-bit seeds, invalid-data atomicity,
  old JSON input, malformed/truncated binary payloads, 512 mip persistence,
  version gates, CPU/GPU agreement, warm uploads, fixed/runtime filtering and
  existing opacity, emission, glass, bump, parallax and self-shadow routes.
- On Apple M4 Pro / Metal, mean RGB8 CPU/GPU albedo error at 512 was0.0070–0.0755
  across the three green families and Carrara. Runtime versus fixed 512 mean
  RGB8 difference was 0.6195–4.5387, reflecting the different sampling/filtering
  strategies; these two execution modes are not promised pixel-identical.
- The headless MCP gallery passed five checks: 12 runtime families exported and
  rendered; three fixed 1024 materials saved/loaded with byte-identical captured
  PNGs; warm map uploads unchanged; shared geometry with stable per-instance
  seeds; curved meshes under two light directions with reflections and shadows.
  Native flat, close, curved, seeded and collection images were visually reviewed.
  The local HTML compares the three supplied references with generated outputs.
  This is evidence for human artistic review, not an assertion of photographic
  equivalence or a geological classification certificate.
- The three explicit 1024 bake commands took 1.4631–1.4781 seconds each in this
  run (command round trip, excluding subsequent encoding). Each complete mip
  chain contains 67,108,848 sample bytes; its GPU program/map buffer used 67,108,944
  bytes. The compressed fixed files were 26,669,847 / 27,359,818 / 27,246,112 bytes.
  Runtime source parameters use 224 bytes per distinct program before shared
  material slots. These counts are not total GPU/CPU memory or FPS claims.
- Current evidence is in `artifacts/marble/verification.json`, `gpu-tests.log`,
  `tests.log` and `review.html`. Native build ID was
  `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|3297f675e8f3fd52`;
  the binary SHA256 and source hashes are recorded in the JSON. The release
  build succeeded at `target/release/ignis`; this task opened no window and
  preserved the user's active editor and concurrent work. Superseded numerical
  prototypes and their helper environment were removed after native review.
- Formatting passed. Strict Clippy stopped on two concurrent surface-program
  lints (`src/terrain/surface/domain.rs` needless range loop and
  `src/terrain/surface.rs` collapsible conditional), outside this unit. This
  does not establish a passing whole-tree lint run.
