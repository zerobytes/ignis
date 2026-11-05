# Volumetric landscape inspection

The user requested the current10×10km scene with their player, free inspection
flight, Fractured catalog rock on walls and a green top surface. The original
landscape remains untouched; the prepared demo is a separate authored Scene.

## Material contract

`StyleMeshBySlope` is a native MCP-first cold operator for static, unattached
meshes. It classifies triangles by transformed normal.y against `min_up`, then
compacts two disjoint indexed meshes. It copies source positions/triangles;
there is no displacement, floating grass shell or coincident duplicate face.
Material-coordinate streams are preserved where authored, or generated in world
rest coordinates to avoid resetting the pattern at every terrain block.
Two materials/instances inherit the original transform and collision policy.
The original asset is removed only when no other instance needs it. Scene edits
are one undoable transaction; invalid thresholds/output IDs fail before publish.
Both groups must be nonempty (this is a split operation, not an implicit fallback).

The wall recipe is the exact data file
`assets/materials/rock/fractured.ignis-material`, SHA256
`5096c3778a1033fa445f0f1ad574b2d4d4ebb8004ab0d6a58976d1e7f02480dd`,
matching the live catalog manifest inspected on2026-11-05:
https://ignis.web.app/catalog/?lang=pt&asset=ignis.material.rock.fractured

Fractured — Guilherme Oliveira / Zerobytes, CC BY4.0;
https://github.com/zerobytes/ignis-catalog . The source material bytes are unchanged. Application overrides only the periodic
tile size to8×8m for readability on the large cliffs; palette, seed and rock
parameters stay those of the catalog asset.
The new green surface is editable content at
`assets/materials/terrain/plateau-green.ignis-material`; it is a surface material,
not grass blades or a vegetation simulation. This work does not publish a new
catalog asset or embed another material preset in Rust. The catalog recipe's
existing projection is retained; general procedural triplanar blending is not
introduced by this operator.

## Flight contract

`SetInspectionFlight`, `SetInspectionFlightVertical`,
`InspectInspectionFlight` plus existing `SetPlayerViewInput` share the native
keyboard/MCP path. This is session-only offline continuous inspection, requiring
a living viewed Player with a KCC. The same physical individual moves; the camera
is not flying independently over an avatar left behind. The mode uses a temporary
kinematic target with collider disabled during each physics step, restoring the
collider, transform and broad-phase AABB afterwards. KCC gravity is bypassed only
for that actor during inspection; disabling inspection resumes the normal motor.
Other simulation clocks continue. Dead/replaced/reset bindings cannot continue
flight. There is no per-tick string allocation or ownership lookup by UI name.

WASD follows camera yaw and optionally pitch. Space ascends, C descends, Shift
selects fast speed, Ctrl selects precision speed; diagonal/combined input is
normalized. Default fixture speeds are100m/s and600m/s, with Ctrl reducing to2m/s for close inspection. Neutral input stops
immediately. Existing click/mouse and Tab first/third-person controls remain.
Third-person camera collision is bypassed while inspecting so a noclip avatar
does not pin the camera against a wall. Inspection applies the existing finite
ordinary-depth projection to the live viewport and headless captures (near0.5m,
far30km); this is not a change to ordinary gameplay/planet projection defaults. The current
reverse-Z renderer path refuses primitive avatars/guides, so it is deliberately
not used for this inspection session.

Portable saves, checkpoints, networking and planet-runtime changes are refused
while inspection is active. Replay requires recorded mode, while this mode
requires continuous execution. A saved Scene is still authored data; it does not
serialize inspection controls or the transient flight position. Mode launchers
must explicitly enable inspection again. Scene/replay/portable/network formats
and the ordinary two-axis gameplay Input have not changed.

## Performance review

Slope classification/compaction runs once per source asset, with dense remap
arrays, shared immutable source, no per-frame regeneration and no Python mesh
construction. Cold-operation envelopes are1,048,576vertices/2,000,000triangles;
these are not world limits. Material separation duplicates only vertices needed
by the two groups and adds a draw/collider per block, not a duplicate surface.
The render cost of the selected procedural rock remains a separate concern.

Flight does fixed vector arithmetic and one bound actor update per tick. Speed
normalization prevents diagonal amplification; one collider/AABB is updated,
not the terrain. Temporary actor flags are restored before public post-step state.
No new physics solver, body-control format, GPU resource layout or persistence
primary is introduced. Renderer/native-window permission is used only for one
final inspection window after headless checks.

## Reproduce

Prepare/verify without a window:
`python3 scripts/open_volume_inspection.py`

Prepare and open the native demo:
`python3 scripts/open_volume_inspection.py --open`

Reopen the already prepared Scene without regenerating:
`python3 scripts/open_volume_inspection.py --prepared --open`

Stable artifacts and launch metadata:
`artifacts/volumetric-terrain-inspection/`. The app bundle entry is a symlink to
the single current `target/release/ignis`, not another copied executable.
The underlying landscape is still the coarse overview mesh (50m horizontal,
160m vertical in the taller variant), not completed near-player LOD or an erosion/vegetation system.

## Initial validation (before chamfer)

28 release integration tests passed across `inspection_flight`, `mesh_slope`,
`player_view` and `character_controller`. Release binary build, focused strict
Clippy (lib/bin and the two new integration tests), focused formatting, diff
whitespace and Python syntax checks passed. A broader unit-test Clippy run also
reported pre-existing warnings in `render/object_impostor.rs` and
`materials/preview.rs`; those unrelated modules were not changed.

The final headless MCP run preserved all182,714triangles:56,236walls and
126,478tops. Saved/reloaded pixels matched. Hover, vertical movement,600m fast
flight and2m precision flight per second, neutral stop, authored Scene isolation
and persistence refusal passed. Native launch metadata is in `launch.json`;
macOS window inventory confirmed an onscreen1100×792window with an advancing
PLAY tick and active VOO controls. The first short startup sample can precede
window presentation; do not treat that empty sample as measured FPS or acceptance.
Human artistic acceptance and near-player LOD remain separate work.

## Taller chamfered variant

The current fixture uses height scale160 (20× the previous8), with a measured
range−4335.80..2608.62m across the unchanged10×10km footprint. The raw source
retains182714triangles and24 matching block borders. The sampled vertical
window is narrowed to60cells without clipping the surface or enlarging sample
budgets; CPU/GPU source scale conventions remain shared.

The optional native `chamfer` parameter is applied before slope materials.
Current recipe: fraction0.16, variation0.75, seed2718. Refined output:
1244386triangles,1199972stone/44414green. This changes geometry and cost;
the initial disjoint partition's shape-preservation result does not apply.
The still-coarse source limits fine-detail width. See `terrain-chamfer.md`.

Hover, vertical/fast/precision flight, Scene isolation, persistence boundaries
and save/reload pixel equality were repeated successfully. Walking grounding
is not asserted in this steeper variant: the inspection is explicitly noclip.

## Current plateau inspection after human review

The height-only approval and preserved tall mountain recipe are recorded in
`terrain-chamfer.md`. Current inspection restores the original height8 density
source, then uses `terrace_lift` (spacing61.5384615384, factor20, phase0) before
the seeded chamfer. Broad original top relief is no longer multiplied20×;
only the offsets between bands increase. The previous taller-variant statistics
above describe that earlier experiment, not acceptance of its composition.
