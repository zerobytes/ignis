# Continuous stone surfaces

## Corrected surface contract — user review

The first collection was rejected: closed cellular joints and per-cell inclined
planes read as fitted masonry. Mountain/boulder volume, silhouette, broken faces
and large crevices must come from the mesh. The material must coat a continuous
stone surface, with mineral variation, fine grain, sparse interrupted hairline
fissures, shallow pits and subtle foliation. Smooth means continuous stone, not
polished/plastic. Replace the default presets with this surface contract and
show the same material on flat and angular source geometry, including a neutral
geometry comparison and opposing light. Do not derive the mesh from the material.

Preflight: retain rock1 decoding/evaluation exactly, introduce rock2 as an explicit
program semantic version. Reuse the nine-vector block/header and library2; use a
new program-kind tag with no buffer/vertex/instance ABI change. Scene119,
replay129 and Content15 guard the new semantics. Evaluate bounded periodic fields
without cellular faces or height steps: no new allocation, pass, dependency,
worker/lock, per-frame I/O or geometry generation. GPU work remains per visible
fragment/explicit sample; no performance claim. Reuse fixed/runtime, cache and
transaction paths; regenerate only current review outputs. Geometry examples are
small authored meshes independent of the height channel. Verify legacy rock1,
marble, CPU/GPU correspondence, fixed persistence and geometry identity before
and after applying material; native artistic review remains a separate gate.

## Current material families

| Family | Surface characteristics | Source mesh supplies |
| --- | --- | --- |
| Alpine | Pale gray mineral matrix, fine crystalline grain, sparse hairline defects | Alpine planes, ridges and broken cliff faces |
| Fractured | Dark continuous stone, pale inclusions, shallow pits and slight oxidation | Boulder volume and large fracture planes |
| Ochre | Warm mineral matrix, fine abrasion and mild directional foliation | Oblique plates and deep crevices |
| Sedimentary | Smooth mineral bands with irregular phase, subtle layer wear | Stepped ledges and broken strata geometry |

A material is reusable independently of the source shape. Smooth here means
continuous stone, not polished marble: roughness, fine surface normals, grain and
small defects remain. The default height budgets are 8/10/8/6mm respectively;
actual sampled variation is smaller. No cell ID colors, cellular face normals,
per-cell planes or layer-height steps are evaluated by rock2. Sparse interrupted
noise contours supply local fissures; continuous filtered mineral fields supply
albedo, roughness, shallow height and slight indirect cavity attenuation. Large
silhouettes and crevices remain geometry. Color stores no directional lighting.

## Controls and reuse

Select **Aparência → Biblioteca de materiais → Gerar rocha**. The same `material`
MCP command serves the editor. Modify seed, tile scale, mineral palettes, grain,
inclusions, oxidation, fissure scale/width/depth, shallow wear and foliation.
The stored names `blocks` and `faceting` are retained for source compatibility;
in rock2 these control fissure frequency and slight mineral undulation, not
fitted blocks. Version1 files retain their original meanings and equations.

```json
{"command":{"op":"material","operation":{"kind":"rock","asset":"stone","family":"fractured","seed":4271,"unique_instances":true,"execution":"runtime"}}}
{"command":{"op":"material","operation":{"kind":"set_execution","asset":"stone","execution":{"fixed":{"resolution":1024}}}}}
{"command":{"op":"material","operation":{"kind":"save","asset":"stone","path":"materials/stone.ignis-material"}}}
```

Four compact editable sources live under `assets/materials/rock`. Fixed execution
retains the editable recipe and lossless maps, while runtime can derive stable
variation per instance ID. Both use the same fields. Rock and marble are mutually
exclusive base programs; both accept ordered overlays. The generic opt-in pixel
normal mode takes two neighbor samples; older recipes default to screen normals.
Mapping still uses the existing explicit planar rest/material coordinates; this
change does not introduce triplanar procedural mapping or a new UV unwrapper.

## Compatibility and cost

Rock1 evaluation is preserved. Rock2 uses the same nine-vector (144-byte) block
and existing 96-byte header, with a distinct program-kind tag. Library2 and the
48-byte fixed sample layout are unchanged. Rock1/pixel normals require Scene117;
rock2 requires Scene119/replay129/Content15. Scene118 can still read the old rock
program but must reject rock2. Tests retain a serialized rock1 fixture. Marble
paths are unchanged; physics, terrain, planet, portable-save and checkpoint
formats remain independent.

The evaluator uses bounded stack arithmetic and periodic filtered fields.
Runtime work is per visible fragment and explicit relief sample. Pixel normals
add two neighbor reads; optional parallax adds its existing view/light steps.
There is no new pass, buffer binding, worker, dependency, vertex/instance stride,
per-frame I/O or geometry generation. Fixed1024 still costs 67,108,848 sample bytes
plus its header: baking and encoding are explicit synchronous operations, not
free memory/performance improvements. Cavity attenuation is an authored
approximation. No temporal stability or FPS guarantee is inferred.

## Native review

```sh
cargo build --locked --release --bin ignis
python3 scripts/demo_rock_materials.py
```

The current `artifacts/rock/review.html` contains flat surfaces under two lights,
fixed/runtime maps, seed and surface-control variants, and the new mesh
comparison. Neutral flat/angular/rounded geometry is shown first, then the same
vertices and instances with one continuous material, then opposing light.
Geometry hashes and exact source comparisons verify that applying the material
never changes mesh shape. No displacement-derived grid or generated material
height supplies the large rock form. The four-family collection is also rendered
on small independently authored angular hull meshes. These example hulls test
material application, not a general mountain/geology generator.

The script reuses one current output directory. Former dense displacement grids
are superseded by the small authored examples after successful replacement.
Accepted marble gallery outputs are only read for regression comparisons.
No human window or world is opened or altered. Native image inspection and
human artistic acceptance remain distinct from automated numerical checks.

## Review history and validation

The first gallery (`f7fe37a`) passed numerical tests but was rejected by the user:
cellular faces and closed joints looked like masonry. Those automated results
were not artistic acceptance. This correction changes the default surface
contract and the demonstration, preserving old saved sources through version1.

- Nine CPU and eight native GPU material tests passed (17 distinct tests).
  Both rock1 and rock2 are covered by the same CPU/GPU color/cavity oracle,
  ordered-overlay offset, fixed-map comparison and warm-upload checks.
  Rock2 mean RGB8 CPU/GPU error was 0.01390–0.01925; runtime/fixed512 mean
  error was 0.75260–1.82855. The old rock1 means remained 0.02416/1.59546.
  CPU checks also bound the new physical relief variation below 4mm over sampled
  surfaces, preserve periodic/instance behavior, reject invalid input atomically,
  accept the serialized rock1 fixture in Scene117 and reject rock2 in Scene118.
- The current native gallery passed six checks: four families under two lights,
  fixed1024 lossless library round trips, unchanged warm uploads, independent
  surface controls, ID-derived variation, unchanged geometry after material
  assignment and three accepted green marble PNGs exactly preserved.
- Native flat/collection/neutral-versus-material images were inspected. The
  corrected gallery replaces the rejected masonry demonstration. Simple angular
  and rounded example hulls establish source shape; they are not artistic
  replicas of the reference mountains. Geometry hashes and image metadata are
  recorded in `artifacts/rock/verification.json`.
- Render-cache regression passed 5 checks and the general laboratory passed 11,
  both headless. Rust formatting for this unit passed, as did diff whitespace.
  The broad all-target run compiled successfully, then stopped at
  `directional_shadows::receiver_plane_bias_is_explicit_versioned_and_round_trips`:
  its exact-version assertion expected 111 but normalization produced 119.
  Normalization assigns the current scene version; this unit does not change
  that policy. The full suite therefore is not reported passing.
  Strict all-target Clippy stopped at `needless_range_loop` in the concurrent
  `src/render/object_impostor/gpu.rs:418`; global formatting found parallel render
  edits. No whole-repository lint/format pass is claimed.
- The release build initially encountered a move/borrow error in ongoing
  vegetation code. Capturing the scalar `spacing` instead of moving its parent
  `Arc<Pack>` through a radius-iterator closure allowed the shared build to finish.
  That minimal fix remains within the parallel `surface_field` changes and is
  excluded from this material commit.
- One current executable location is retained at `target/release/ignis`.
  Superseded dense grid scenes/images from the rejected demonstration were
  removed after replacement succeeded; current example scene data is about 60KiB
  instead of 75MiB. No app-binary snapshots or human windows were created.

Final static review: rock2 does not call the cellular rock1 evaluator. Fields are
bounded and stack-only; the same nine-vector block/cache/read path is used.
Continuous microrelief and slight cavity attenuation cannot manufacture masonry
shadows or source planes. Fixed file work stays explicit and synchronous; pixel
normals retain their two extra reads. Applying a material preserves all source
positions/topology and instance transforms. The small closed example hulls are
authored once by the review script and never generated by a runtime material.
No new allocation/pass/lock/worker/dependency/ABI or collision behavior was added.
No FPS, GUI-input, cross-platform or human artistic acceptance is inferred.


Native run: build `1863847ea6ee3b26`, SHA256 `1ffe5bf645bd99f842010b767a710a45a206ebabdcdf485535050b5b78b05401`, Scene119. Shared working tree includes concurrent vegetation work.
