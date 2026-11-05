# Full plateau with layered, weathered rock volumes (EAF-221)

User requests a whole plateau following simple-sample.blend's broad overlapping
volumes, extended beyond trapezoids: smaller broken pieces, thin overlapping
layers and irregular chamfered/worn corners. These are explicit construction
requirements, not a claim of physical erosion or already achieved reference art.

Before building, the executable list is: W1 angular lobed plateau footprint/top;
W2 recessed solid core; W3 broad tilted convex rock masses in multiple families;
W4 thin partially exposed overlapping plates; W5 irregular corner truncations;
W6 localized chipped volumes/deep fractures; W7 small-scale surface grain;
W8 native multiview review, connected/closed surface and save/load evidence.
Main planes must remain readable; detail cannot approve missing mass/proportion.

## Native tool contract and static performance review

Add MCP-first mesh_modeling::UnionConvex{points} beside SubtractConvex. Same
preview/commit, fresh topology reference, paused authorship, undo/redo and native
scene/collider pipeline. Static closed unrigged meshes, implicit/identity material
coordinates only. Convex addition must increase volume and leave one connected
closed oriented surface. Disconnected, tangential/nonmanifold, no-op, precision
collapse and resource-budget failures are atomic refusals. No material/rig loss.

Reuse pinned Boolmesh f64 adapter, native hull and bounded surface validation.
Union normalization includes source AND input hull points; subtraction retains
its original normalization. Source vertices that survive normalization exactly
map back to original f32 positions. Check packed geometry as well as kernel data.
Selected result faces match the outward hull planes for union and inward hull
planes for subtraction. Output ordering remains canonical.

Input8192v/8192tri,4..32points (bounded decode/schema), Boolean pair product524288,
existing document/output quotas. This is cold authoring, no per-frame regeneration,
new dependency, GPU format or rendering pass. Third-party temporary allocation
remains bounded by admitted geometry/work, not a hard allocator/time sandbox.
Formats of existing scenes/sources are not changed by adding a command variant.

## Construction approach

Native Landform compiles a flat-topped lobed polygon/core. Client recipe supplies
convex point clouds only; native union performs all hull/Boolean/triangulation.
Large cells use skewed hexahedral corner controls; each corner is truncated by
three unequal edge distances (up to24points/hull), preserving broad flat faces.
Secondary thinner cells overlap existing exposed faces with correlated directions.
Small convex subtractions remove actual chips. Granular material is appearance,
not a substitute for missing large geometry. No Blender remesh or external bake
is used to claim native engine geometry. This is geometric weathering, not a
hydraulic/thermal erosion simulator.

## Validation and observed result

Final release build f5c6cdfde471e729, macOS Apple Silicon, current executable
`target/release/ignis`.18focused Rust tests pass (5mesh_chipping including additive
analytic volume/repeated concave additions/refusals,10mesh_modeling,3shaping).
Strict targeted Clippy lib/bin/three-test-targets with-Dwarnings passes; owned
rustfmt/Python compile/diff pass. No new dependency or format-version change.

Real MCP `verify_mesh_union.py`: union preview is pure and matches commit,
undo/redo restore exact scenes, stale/disconnected/play refusals preserve state,
replay matches. A real KCC rests grounded atY3.2 on the added top surface,
proving the published union reaches the collider path. The plateau driver also
checks each rejected proposal preserves geometry, verifies0open/nonmanifold
edges and1mesh island, scene-save/load PNG byte equality and replay meshes.

Whole plateau:20large-volume proposals,18thin-sheet proposals,16edge-chip and
26small-chip proposals.33unions and42subtractions accepted;5additions rejected
by contact/self-intersection ambiguity guards, recorded in rejections.json,
without fallback geometry or weakening checks. Result2334v/4664tri. The rock is
one connected closed mesh; turf is a separate visual skin (+0.15m) on55flat cap
faces. Rendering still contains material separation and a contextual floor.

Viewed neutral/overview/detail/rear/top native captures. Broad planes and
corner truncations are present, secondary sheets/edge breaks are actual geometry.
The output remains stylized/chunky: sparse fine layers, clean large planes and
some cut marks look triangular. This is not a photorealistic geology claim or
human art acceptance. "Weathering" here means authored chamfers and chipped
volumes, not simulated hydraulic/thermal erosion. Shadow-off diagnostic did not
remove all lines around shallow sheets; do not label every line a shadow bug.

Static post-review: all Boolean work is cold authoring. A deterministic client
recipe chooses geometry controls; native hull/union/subtraction/packing perform
the topology. Runtime renders the baked mesh, with existing collider rebuilding.
There is no newly implemented native geological generator, non-destructive
Landform format stack, world streaming, destruction simulation or FPS proof.

Evidence: artifacts/weathered-plateau/verification.json,union-verification.json,
rejections.json, plateau.scene.json and native PNGs. Editable base+accepted operation
list: examples/weathered-plateau.recipe.json. User Blender file and unrelated LoS
work preserved. No window opened, no copied engine binary/build-history directory,
local atomic commit only, no push. EAF-221 stays InProgress for reference fidelity
and broader terrain integration. User requested a new experiment; art acceptance
must come from reviewing this result rather than software-test success.
