# Localized solid subtraction

This continues EAF-221. The objective is a reusable authoring operation, not a
reference-specific mesh generator. Large rock planes, broken edges and exposed
surfaces require actual removed volume. The existing stratified/rounded landform
mode remains available.

## Contract before implementation

`mesh_modeling::Operation::SubtractConvex { points }` subtracts the convex hull of
4..32 asset-local points. The source may already be concave: successive operations
must work on the result of earlier cuts. The cutter is finite, so a wedge/chip can
affect one region without clipping the entire object by an infinite plane.

Use the existing `model_selection` EditMesh/preview and `model_editor_mesh` MCP
entry points, including revision checks, paused authorship, explicit procedural
source detachment, undo/redo, collision rebuild and scene persistence. MCP comes
first; no dedicated mouse tool is part of this increment.

The first scope is one static, closed, outward-oriented, non-self-intersecting
solid. Reject rigged meshes, animation clips and custom material-coordinate maps
that cannot be preserved. Preserve the asset material and local procedural mapping.
Reject empty results, disconnected results, isolated enclosed cavities, no-volume
cuts, invalid topology and exhausted budgets before scene publication. Supporting
detached debris or internal cavities requires a separate explicit policy.

The Boolean kernel is pinned Boolmesh0.2.2 (unmodified MPL-2.0 Cargo dependency),
in its default f64 mode. Rapier/Parry still supplies the convex hull and the BVH
for surface-validation candidates. An initial Parry Boolean trial failed the
closed-surface gate on the analytic subdivided-cube case and was not retained.

Normalize local coordinates. Set the pinned kernel's public `eps`/`tol` to
`f32::EPSILON * (1 + max(abs(center))/span)`, matching the precision of the mesh
document at its local origin. Refuse values above0.001 normalized units; use a
closer local origin instead. The operation report exposes
`numerical_tolerance_meters = eps * span`. This is the simplification tolerance,
not a certified Hausdorff error or a promise about every geometric predicate.
The compile-time adapter requires a64-bit Boolmesh Real.

The kernel can retriangulate/simplify coplanar and subprecision features. There
is no proximity-based weld added by the adapter, smoothing, voxel conversion or
silent fragment dropping. Surviving exact normalized source positions map back
to their original f32 values. Validate closure, orientation, vertex fans, volume
and triangle contacts again after f32 publication packing. The contact guard is
conservative and tolerance-based, not an exact-arithmetic CSG certificate.
Ambiguous/invalid cases return an error without committing partial geometry.

## Performance review before coding

- Is this on a frame/tick path? No: explicit cold authoring/preview only.
- Does resolution need to increase everywhere? No: Boolean intersections introduce
  edges locally; existing large planes remain planar.
- Is input/work/output bounded before publication? Required: source count, cutter
  count, candidate-pair/validation work and final mesh limits are separate guards.
- Are large scene/JSON structures used inside the numeric loop? No: typed local
  arrays, indexed meshes and an existing spatial acceleration structure.
- Is a new package/runtime/process needed? One pinned pure-Rust dependency,
  Boolmesh0.2.2, with glam0.32.1 (already resolved in the lock graph). No Bevy,
  Rayon, f32 feature, external process or new runtime is enabled.
- Can source resources or unrelated assets be changed on failure? No: evaluate a
  candidate and use the existing transaction.

## Acceptance gates

Reference inventory (reviewed before generating the validation specimen): the
720x720 wall reference and its 640x440 crop at x50/y35 show broad angular plates,
interrupted oblique joints, sharp local ledges and deeper exposed planes. The
736x817 wall reference and its 680x400 crop at x10/y0 show smaller overlapping
angular breaks with a preferred diagonal direction, several relief scales and
dark recesses. References are visual evidence, not verified geological samples.
Furniture, interior lighting fixtures and photographic authenticity are outside
the tool contract. Crop files: `artifacts/rock-chipping/reference-{plates,fractures}.png`.

Construction list for a tool specimen: (C1) one closed block with broad planes;
(C2) a localized oblique wedge exposing a recessed plane; (C3) a second intersecting
chip demonstrating a concave input; (C4) smaller nonperiodic edge breaks;
(C5) material-only fine grain after geometry is checked; (C6) front/oblique/side
views and source/operation parameters. This is a morphology demonstration,
not a reconstruction or an acceptance claim for either reference.

Analytic volume and ray/point probes must show localized removal and new closed
faces. Repeat on a concave result. Verify untouched regions, outward orientation,
deterministic output, open/nonmanifold/self-crossing rejection, invalid/degenerate
cutters, no-op/empty/disconnected cuts, stale references, atomicity, preview parity,
undo/redo, persistence/replay and opt-in collision through MCP. Native multiview
captures check whether the tool produces angular plates and recesses. Geometry
tests do not establish reference fidelity, frame rate or human artistic acceptance.

Higher-level rock profiles should compose this operation with existing volume,
crease and shaping tools. Seed, preferred fracture directions, plate scale, chip
depth and spatial masks belong to editable recipes. A low-level cutter alone does
not deliver a universal geological generator, erosion, world streaming or runtime
physical destruction.

## MCP usage

Inspect `inspect_mesh_topology { asset }` for the fresh revision reference. Then
use `preview_mesh_edit` or `edit_mesh` with this edit (example wedge, local metres):

```json
{
  "kind": "model_selection",
  "asset": "rock",
  "reference": {"scope": "from_inspection", "revision": 123},
  "operation": {
    "kind": "subtract_convex",
    "points": [[-2, 3, -5], [2, 3, -5], [0, 7, -5], [0, 4, -1]]
  }
}
```

Coordinates are asset-local, not world positions or triangle IDs. The hull closes
the cutter; the caller does not triangulate it. The result report selects exposed
cut faces and exposes its numerical tolerance. The generic operation works without
a preselected face region. `model_editor_mesh` additionally selects the resulting
faces in the editor, using the same evaluator.

`examples/rock-chipping.recipe.json` contains an editable base landform and explicit
cutter operations. `scripts/verify_mesh_chipping.py` authors it through MCP and
writes the baked scene/current evidence under `artifacts/rock-chipping`. No mesh
triangulation or Boolean is done in Python. This recipe is not a new native
Landform source version: Landform1..3 remain unchanged; the editable cutter list
is replayed explicitly by the recipe driver. A non-destructive fracture stack
inside the Landform format and parameterized geological profiles remain subsequent
contracts, as does integration with the transported planet runtime.

## Performance review after implementation

The present numerical domain also requires an asset span of at least0.0001m,
cutter points within four source spans of its center, and a normalized cutter
volume of at least1e-9. Coordinates are within the existing +/-10000 asset-local
mesh envelope. These are local authoring guards, not world-coordinate/planet-size
limits. Very fine cutters relative to a large artifact may require a smaller
local artifact; this increment does not promise arbitrary-scale exact CSG.

- Input: at most8192 vertices/8192 triangles and4..32 cutter points. The hull
  has at most60 faces. Boolean triangle-pair product is capped at524288.
  The MCP/serde cutter decoder stops at32 entries, and schema min/max are explicit.
- Surface guards use the existing BVH, with at most524288 unique candidate pairs
  per validation; pair-local contact points use stack arrays. Global native mesh
  quotas and current document replacement quotas still govern output.
- Transient indexed f64 kernel data, f32 packed mesh, adjacency and lookup tables
  exist only during authoring. Sorting output is O(VlogV+FlogF); there is no cache,
  worker, new draw pass, frame readback or per-tick execution.
- The external kernel can allocate transient intersection/triangulation state
  before the final output count is known. Bounded input/pair work reduces this
  exposure; this is not a hard allocator-byte/time sandbox. Keep it off frame paths.
- f64-to-f32 packing exposed a collinear triangle in the subdivided analytic cube
  with the kernel's1e-12 default. The publication-precision tolerance fixes that
  fixture; unsupported near-degenerate cases still fail instead of deleting faces
  from the packed mesh. This is numerical policy, not a frame-rate optimization.
- No FPS, total-memory or universal robustness claim follows from these checks.
  Volume comparison uses compensated summation in the normalized unit bounding
  cube and a f64 roundoff floor proportional to triangle count, rather than a
  minimum percentage of the whole solid. A1cm corner chip in a2m cube is a separate
  analytic regression, including the report's local-metre tolerance.

Primary kernel source: [Boolmesh0.2.2](https://crates.io/crates/boolmesh/0.2.2),
[upstream implementation](https://github.com/komietty/boolmesh). MPL notice is in
`THIRD_PARTY_NOTICES.md`; the unmodified upstream license is retained in `licenses/`.

## Specimen review and remaining morphology

The initial native oblique/front/side captures were opened together with both
reference crops. C1 is a closed block; C2/C3 show actual recessed planes and
intersecting grooves; C5 has material grain; C6 exposes depth in multiple views.
Six operations produce102 vertices/200 triangles and reduce volume1920 to
1723.499m³. These numbers demonstrate geometry, not likeness.

Visually the specimen is an engraved rectangular slab. C4 is divergent: the small
triangular recesses look like holes rather than chipped, irregular broken plate
edges. The long groove is too straight/continuous, the perimeter remains too
rectangular, fracture density is sparse and the plates lack a coherent multiscale
directional structure. Do not label this specimen a realistic rock, a reconstruction
of the references or human-approved topology.

The next authoring tool should construct a directional fracture network and
correlated plate/chip volumes, with user-controlled scale, orientation, depth,
irregularity, density, seed and masks. It should call the native cutter/shaping
tools and retain an editable operation stack. These controls are not implemented
by merely storing the six cutters in this example recipe. The accepted rounded
strata modality is independent and unchanged.

## Final validation of this increment

- macOS Apple Silicon, release build8561ef69900e9cb0, one current executable at
  `target/release/ignis`; no window opened or user scene changed.
-26 focused Rust tests pass:4mesh_chipping,10mesh_modeling,3relief_shaping,
  4relief_sections,2landforms,3terrain. Includes analytic repeated volumes/rays,
 1cm chip, translated origin/material preservation, self-crossing and malformed
  source/cutter refusals, bounded decoding, and existing topology/source regressions.
- Four real-MCP groups pass in `artifacts/rock-chipping/verification.json`:
  preview/undo/redo; atomic refusals and stale/late-batch/playing errors; actual
  KCC collider rebuild and replay/persistence; recipe/captures and save/reload PNG.
  KCC centerY1.2 before andY0.4 after the top chip, grounded in both cases.
- Oblique/front/side final images are byte-identical to the images reviewed above.
  Scene120/replay130, Landform1..3 and terrain formats are unchanged by this unit.
- Owned rustfmt, Python compile and diff checks pass. Strict Clippy encountered
  an existing `collapsible_if` in parallel botany code at
  `src/botany/production.rs:97`. Lib/bin/six-test-target Clippy passes with
  `-D warnings -A clippy::collapsible_if`; this is an explicit lint exception,
  not a globally clean strict Clippy result. The botany file was not edited.
- Temporary standalone numerical probes were removed automatically; the obsolete
  task-only pre-Boolmesh test executable/depfile (1438894 bytes) were removed.
  No old engine binary, copied bundle or timestamped build directory was retained.

Implementation of this low-level tool is complete. The directional/multiscale
fracture generator, non-destructive native Landform stack, reference fidelity,
live-world integration and human artistic acceptance remain open in EAF-221.

## Convex additive counterpart

UnionConvex now complements finite subtraction through the same native modeling
entry points. See weathered-plateau.md for combined-bounds normalization, the
connected-solid contract and a whole-plateau demonstration. Native Landform1..3
and the absence of physical erosion/world-streaming integration are unchanged.
