# Emerald meadow: native grass quality correction

EAF-218 contains a finite 1000×1000m rolling terrain with native collision and a
continuous emerald-grass field. The user rejected the initial rendering: the
nearby blades lacked shape and density/brightness changed in visible squares.
The earlier occupancy and performance checks did not constitute visual acceptance.

## Reproduce and open

```sh
python3 scripts/build_emerald_meadow.py
python3 scripts/verify_emerald_meadow.py
python3 scripts/open_emerald_meadow.py
```

The native executable must include IGFLD2 support. On a cold checkout, compile
the native lawn library first with `python3 scripts/demo_lawns.py`. During concurrent renderer
work, this unit was validated in the isolated active worktree
`/Volumes/Zero/dev/games/custom-engine-emerald-quality`, based on 150d971, with one
current executable at `target/release/ignis`. Existing dependency libraries were
reused; no second SDK/dependency cache was made. Source/compiler/binary provenance
is in that worktree's `target/release/native-build.json`.

The local `bundle.json` binds the terrain Scene, field and validated executable
by SHA256. The launcher automatically chooses that executable. `--binary PATH`
explicitly selects another compatible build; rebuilding/capturing updates the
bundle. Reopen through the launcher to use the new field and executable: an
already-running window keeps its previous resident data. Other user windows and
parallel work are preserved.

This remains a diagnostic session sidecar selected with `IGNIS_HORIZON_FIELD`.
SaveScene alone does not persist or safely rebind the field. The bundle is the
reviewable local composition; generic persistent MCP/UI field authoring and
production streaming are outside this change. The launcher starts first-person
mode; click captures the mouse, WASD moves, Shift runs, Escape releases the mouse.

## What changed

The old field chose one LOD for each 2×2m block and used flat two-triangle leaves
near the character. Patch-wide tint and shared root height reinforced the grid.
Merely increasing occupancy did not repair these problems.

The opt-in IGFLD2 profile carries each vertex's native blade root, identity and
centre-line station. The emerald recipe now:

- Keeps native curvature and the central fold near the camera: 192 blades with
  twelve triangles each in the closest representation, then 192 folded blades
  with four triangles each. The canonical botanical library remains unchanged.
- Places and rotates blades individually using stable world/cell/leaf identities;
  colour varies per blade instead of tinting an entire clump or block.
- Interpolates regular terrain samples at each blade root. A 2mm embedding offset
  replaces one flat root plane shared by a complete tuft. The field is upright
  grass, not a mechanical simulation or species-level growth calibration.
- Chooses detail per blade by distance and physical pixel footprint. Stable
  stochastic transitions span 30% on either side of each threshold. CPU blocks
  submit overlapping relevant representations; their edges no longer define the
  visible transition. Blocks remain a spatial batching mechanism.
- Uses two-sided thin-leaf shading and a base-to-tip colour gradient. The five
  metric thresholds are 0.004/0.015/0.050/0.16/0.50 metres per pixel; the final
  three levels remain approximate distant coverage, not a fidelity certificate.

Far leaf-width compensation is capped and based on the centre-line rather than
stretching the entire blade. Bounds account for this width/crease deformation
and neighbouring terrain samples. The regular grid clamps at the outer half-cell;
perfect root contact at every perimeter point is not asserted.

IGFLD1 reading and writing remain supported. Its original shader is preserved in
`surface_field_legacy.wgsl`, avoiding numerical changes caused by sharing the
larger continuous shader. One native legacy fixture matches its original PNG
byte for byte; this is a scoped regression check, not a universal engine promise.
No global Scene/replay/network format change belongs to this unit.

## Geometry sharing and validation

IGFLD2 templates are internally indexed using the exact position, normal, blade
identity and centre-line bits as the key. Identities are never merged just because
positions coincide. Triangle order, inputs and the rendered image remain the
same; only repeated vertex work is shared. `IGNIS_FIELD_INDEXED=0` is a diagnostic
ablation, not an authoring control. walk/walk_down/close native PNGs are byte-exact
between indexed and unindexed representations. The final bounds/legacy-shader
refinement also retains the same quality-view PNG.

The field remains uniformly occupied across 1km²: 64 million sites at 12.5cm pitch,
with 160,000 in each of 400 audit cells of 50×50m. The six template triangle counts
are 2304/768/96/24/6/2. The height payload is 256,000,000 bytes and the complete
sidecar is 256,614,520 bytes. This is source occupancy, not the number of plants
or triangles submitted in each frame. No paths, water exclusions, camera-specific
planting zones or approved grass-source edits were introduced.

Validation includes five focused source-module tests (format bounds/roundtrip,
legacy bytes, blade metadata/identity, exact indexed reconstruction, frustum and
both WGSL programs), focused Clippy with `-D warnings`, Python compilation,
4,096 regular height samples against persisted native terrain triangles, a 7.945m
grounded KCC walk and byte-exact save/load in the same bound session. These tests
were linked against identified existing libraries for model/projection types and
validate the current field source; they are not a claim that the shared checkout's
full concurrent Cargo suite passed. Native Metal captures and presentation tests
exercise the actual isolated executable.

`artifacts/emerald-meadow/review.html` compares the rejected and corrected images
at the same camera and includes raw 1800×1100 walk, downward eye-height, close,
aerial and actual first-person-player captures. `player.png` comes from the
session's PlayerView after the actor settles on the terrain, with its body hidden
by the existing first-person path. These quality captures are unfiltered native
rasters. Preview files, if generated separately, are not runtime antialiasing.

## Observed performance and remaining limits

The actual 2200×1520 native window on the M4 Pro was sampled ten times per camera
after a two-second warm-up. These are overlapping present-call cadence reports,
including UI/scheduling, not GPU timestamps or scanout. No-vsync was requested;
other desktop processes were preserved. Measurements are observations on this
Mac and are not a universal FPS guarantee.

| Camera | Median observed FPS |
| --- | ---: |
| walk | 15.63 |
| hills | 19.49 |
| panorama | 47.70 |
| close | 11.38 |

Build: `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|11a516ba06ed0dfc`.

The rectangular LOD steps are no longer visible in the reviewed poses and the
near blades retain their fold. Fine distant grass still aliases because the
native raster is single-sample. Distant appearance, smooth motion and performance
are not fully solved. This unit does not claim AAA parity, 60FPS, a production LOD
certificate or human artistic acceptance. EAF-218 remains open for visual/runtime
review; broader engine work stays separate.
