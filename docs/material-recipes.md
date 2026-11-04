# Reusable procedural materials

## Contract before implementation

A versioned surface recipe is the common source for two execution modes:
`runtime` evaluates deterministic fields in the fragment shader; `fixed` stores
precomputed, periodic surface samples and their mip chain. A `.ignis-material`
library file contains the material, including its recipe and optional fixed data.
Applying one uses the normal mesh material transaction, undo and scene persistence.

Recipes compose bounded layers using a periodic 2D material domain (XY, XZ or YZ
of the existing rest coordinates), with a tile size in material units. The fields
are generic noise, ridges, cells, bricks and waves. Each mask mixes correlated
base color, roughness, metalness, height, opacity, indirect occlusion and emission.
These are authored properties; occlusion does not replace scene visibility or GI.
Legacy materials keep their exact path. New recipes own their channels and cannot
silently combine with the legacy procedural/glass channels.

Runtime instances may opt into a stable seed derived from recipe seed and instance
ID. Derived GPU material slots share geometry and are rebuilt only with source
changes; changing draw order, camera, pose or tick must not reseed the surface.
Fixed materials share their samples; changing their recipe requires an explicit
regeneration. Fixed samples, rather than the retained recipe, are authoritative.

Height provides screen-gradient bump or opt-in parallax occlusion with a bounded
view-ray search and light-ray self shadow. Relief changes material lookup, not
silhouette, world shadow caster geometry or collision. World shadows still use
the existing scene shadow map. `bake_displacement` explicitly creates a static mesh asset and rebinds one chosen
instance, preserving the source asset and rest coordinates in one undoable
transaction. It samples existing vertices, rejects local triangle inversion and
requires a current mesh reference. Refining the mesh is an explicit existing
modeling operation. No automatic tessellation or universal self-intersection
certificate is implied. Existing collision settings for that instance determine
whether the new geometry participates in physics. Local GI, subsurface scattering,
clearcoat and anisotropy remain separate renderer capabilities.

## Static performance review

- No new dependency, vertex attribute, per-instance GPU stride, thread or lock.
- Material metadata adds one 16-byte reference; programs/fixed samples use one
  separately bounded GPU storage buffer shared by visible and shadow passes.
- Recipe/mip data uses immutable `Arc`s; frame comparisons avoid copying sample
  arrays. Uploads/packing occur only after material/source changes. Device storage
  limits are checked before allocating/uploading; warm frames reuse their buffers.
- Runtime work scales with visible fragments, layer count and explicit relief
  steps. Fixed mode performs filtered sample reads and no procedural field work.
  Mips and footprint attenuation remove subpixel detail; neither means free GPU
  work or guaranteed temporal antialiasing.
- Bake is synchronous, bounded and explicitly requested outside frame drawing.
  Fixed resolutions and file byte limits bound memory and parser work. No bake
  or file read/write is permitted inside an immediate-mode UI drawing callback.
- Save/load uses the existing project-root path checks and atomic replacement.
  Invalid source/data must fail before scene mutation. Large fixed materials may
  exceed the independent replay/network envelopes; those envelopes stay intact.

Validation and observed limits are recorded after implementation. Native image
review, automated correctness, human artistic acceptance and publication are
separate checkpoints. Work uses the public history and preserves parallel planet
changes; the current release executable is replaced in its stable location.

## Authoring and reuse

Select a mesh instance in the editor, expand **Aparência → Biblioteca de
materiais**, and choose Pedra, Madeira, Metal pintado or Cerâmica. Runtime recipes
expose base channels, ordered layers/masks, seed, stable per-object variation,
plane, tile scale, height and relief budgets. The marble family selector adds
12 configurable mineral programs; see [procedural marble](marble-materials.md).
The rock selector adds four configurable mountain-rock families; see
[rock materials](rock-materials.md). The library path saves or loads an
independent material. **Gerar asset fixo** precomputes the maps; **Editar receita
em runtime** restores live evaluation from the retained recipe. **Gerar malha com
relevo** creates a new static asset for the selected instance, with its own ID.

All operations use the `material` MCP command. Existing mesh `set_material` also
accepts the same material data. Examples (the named mesh asset must exist):

```json
{"command":{"op":"material","operation":{"kind":"preset","asset":"wall","preset":"stone","seed":93,"unique_instances":true,"execution":"runtime"}}}
{"command":{"op":"material","operation":{"kind":"set_execution","asset":"wall","execution":{"fixed":{"resolution":128}}}}}
{"command":{"op":"material","operation":{"kind":"save","asset":"wall","path":"assets/materials/stone.ignis-material"}}}
{"command":{"op":"material","operation":{"kind":"load","path":"assets/materials/stone.ignis-material","asset":"another-wall"}}}
{"command":{"op":"material","operation":{"kind":"inspect","asset":"another-wall"}}}
```

`generate` takes a complete `recipe` and execution choice. `load` without `asset`
validates and inspects a file without editing the scene. `bake_displacement` takes
an `instance`, an unused output asset `id` and the current mesh `reference` from
`inspect_mesh`; it changes geometry only on explicit request. The retained source
asset makes further regeneration possible. Tile scale follows the existing rest-domain
contract: scaling an instance scales its pattern. Shader bump/parallax depth is
in world metres. Geometry baking accounts for the selected instance scale; later
rescaling that generated mesh naturally rescales its geometric relief.

The optional `normal_filter` selects `screen` (the legacy default) or `pixel`.
The latter computes relief differences separately per pixel, reducing 2x2 blocks
on steep height transitions at the cost of two extra neighboring material reads.
It also uses the shifted lookup coordinate with parallax. It changes neither
silhouette nor collision. Rock presets choose it; other recipes retain their
previous mode. The opt-in mode and legacy rock1 require Scene117; continuous rock2 requires Scene119.

Recipe1 has at most eight layers. A sample contains three four-channel vectors:

| Vector | Components |
| --- | --- |
| `color_roughness` | Linear RGB multiplier (0..2), perceptual roughness (0.2..1) |
| `response` | Metalness, normalized height, opacity, indirect occlusion (0..1) |
| `emission` | Linear emission RGB (0..64), thin diffuse transmission (0..1) |

A layer selects a periodic field, integer frequency XY (1..64), seed offset,
1..4 octaves, warp, two ordered mask thresholds, strength, inversion and a target
sample. The mask interpolates every channel together. Uniform target channels can
be copied from the base when a layer should vary only a particular property.
Bricks use staggered rows with even Y frequency; odd Y frequency uses aligned
rows so the tile remains periodic. Coordinates are planar by explicit choice;
there is no automatic UV unwrap or triplanar reconstruction in Recipe1.

Fixed data includes the full mip chain at resolution 16, 32, 64, 128, 256, 512
or 1024. Each sample uses 48 bytes: the complete 128 map uses 1,048,560 bytes;
the 256 map uses 4,194,288 bytes; the 1024 map uses 67,108,848 bytes, plus program
headers/table references. Library2 saves fixed samples in a bounded binary
envelope with lossless PNG compression of little-endian float bytes; runtime
recipes remain JSON. Library1 JSON remains readable. The file limit is 96 MiB,
independent of GPU binding, replay and network limits. Mipmaps average linear channels; the runtime field's
footprint attenuation is an approximation and is not promised pixel-identical to
sampled maps at every distance. Resolution, filtering and field frequency affect
how closely the two modes match.

Recipe channels own the appearance; legacy patterns, detail, reflectance,
transmission and glass cannot silently override them. Surface composition may
still choose opaque, cutout, blend or additive. Parallax is restricted to opaque
materials in this version; opacity/cutout/translucent materials use bump and share
the existing coverage/shadow/composition implementation. Emission does not add a
light source. Ambient occlusion is an authored attenuation of indirect light,
not scene-space visibility. Specular temporal antialiasing and HDR are separate.

New materials require Scene112; edit transactions upgrade that version. Replay122
and public visual Content10 reject incompatible readers; protocol5, checkpoint3
and portable-save format1 keep their own envelopes and versions. Older scenes
with no new fields remain readable and use the legacy material path.

## Reproducible native gallery

```sh
cargo build --release --bin ignis
python3 scripts/demo_material_recipes.py
```

The script uses a separate data root under `artifacts/material-recipes`, the one
current release executable, and no window. `review.html` contains a two-mode
comparison, oblique view, four details and stable per-instance variation. The
`library` directory holds reusable material files; `examples` holds the saved
editable scene. Running the script replaces these current outputs in place.

## Validation and performance outcome — EAF-211

- 26 non-GPU checks across nine integration suites passed, covering recipes,
  existing materials/details, shared mesh resources, mesh LOD sidecars, plant
  material frames, public visual content and portable restoration. All 11 native
  GPU checks in the material-recipe, transparency and mesh-resource suites also
  passed (37 distinct checks total). This includes every field family, maximum
  32-bit seeds, runtime/fixed image comparisons, stable instance variation,
  warm/reloaded cache reuse, self shadow, parallax and numeric alpha/emission.
- `scripts/demo_material_recipes.py` passed six MCP checks on Apple M4 Pro/Metal.
  Its native two-mode gallery, oblique view, four close-ups and unique instances
  were inspected. Save/load reproduced the gallery PNG exactly. Repeated capture
  kept the program/map upload counter unchanged. Eight gallery materials used
  4,195,776 GPU storage bytes in the surface-data buffer; this is not total GPU
  memory. Current evidence: `artifacts/material-recipes/verification.json` and
  `review.html`.
- Release build succeeded at the stable `target/release/ignis` path. The gallery
  used Scene112/replay122/Content10; concurrent particle work subsequently advances
  the shared working tree to Scene113/replay123/Content11. The material feature's
  minimum scene version remains112. No secondary application bundle, worktree or
  historical binary copy was created, and no window was opened by this work.
- Formatting of this unit passed. The whole-tree Clippy attempt encountered
  concurrent particle UI/default lints and an eye-source test initializer missing
  its new field; those fronts are outside this commit. Their in-progress state
  must not be presented as a passing whole-repository lint/test run.
- Fixed generation is cold command work. Runtime evaluation and parallax remain
  fragment work; manual filtering uses storage reads rather than hardware image
  sampling. The current implementation has no async baking, streaming texture
  residency or GPU timing/FPS claim. Resource-identity changes refresh caches once
  rather than comparing a newly loaded sample array on every subsequent frame.
- Geometry baking allocates a new position/normal result, preserved rest domain
  and copied triangle list only when requested. It preserves the source, accounts
  for selected instance scale and rejects local face inversion. It is not a
  global self-intersection certificate or automatic subdivision policy.

The four starter recipes are editable demonstrations, not a photorealism or
human artistic acceptance claim. Approval of their appearance remains separate
from the implemented material system. No push or publication is part of this unit.
