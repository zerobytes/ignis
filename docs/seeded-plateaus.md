# Seeded plateau authoring (EAF-221)

Scope correction: user wants procedurally generated plateau forms, not seeded
placement/rotation of the same approved mesh. Preserve approved patchy baseline
and previous composition. Generate forty distinct native shapes in five bands.

Before-code inventory: S1 world seed; S2 stable per-slot derived seed independent
of iteration order; S3 native varying footprints/aspects/heights; S4 shared
regional bedding with local variation; S5 seeded unequal strata, exposure/scars;
S6 seeded finite cuts; S7 five elevation bands with grounded bases and individual
cap geometry; S8 native hash uniqueness and same-seed regeneration/changed-seed
proof; S9 scene persistence/replay and offscreen visual review.

Static performance pre-review: bounded40slots/3candidate attempts, each<=29x28
section controls and18detail cuts plus2trims, native CSG work/input guards.
Cold authoring only; Python supplies source/cutter controls, Rust owns topology.
Uniqueness requires distinct baked meshes; no claim of forty different forms
sharing a single geometry buffer. Native island extraction compacts used cap
vertices instead of copying whole rock payloads into turf assets. No new frame
loops, renderer passes, dependencies or builds. Same-seed determinism scoped to
this generator version/build/environment, not an unproven cross-platform guarantee.

## Contract and implementation

scripts/seeded_plateaus.py uses BLAKE2b world/slot/candidate streams and generator
version1. Each source owns a seeded contour/aspect/height,9..13 unequal strata,
correlated local exposure masks/scars and a regional bedding normal with small
local variations. Inverse instance yaw maps the regional normal into asset space,
so placement rotation does not arbitrarily rotate the regional bedding itself.
No source is selected from a finite catalog of baked meshes. Same slot's geometry
stream does not depend on construction order or adding a later slot.

Native Landform2 compiles each source. Mandatory native convex trims form horizontal
top/base; finite seeded joint/spall/chip proposals use existing native CSG. Each
refused detail keeps prior geometry; up to3 deterministic candidates for critical
trim refusal, then an explicit failure, never a fixed-mesh fallback or weakened
guard. Does not promise every possible seed can bake under current numerical
admission checks. Basal native extrusion supports elevated layers. Native island
extraction retains only used rock/cap vertices; cap rendered exactly once with
matching transforms. Per-pair edge incidence2 is verified.40groups/81instances
and81collision instances including floor, five support depths unchanged.

This is a seeded *authoring client* of existing MCP primitives, not a new native
runtime terrain-streaming/planet generator. Python supplies source sections and
convex clouds; Rust performs all triangulation/topology/extrusion/extraction.
Batch count1..40 is a demonstration guard, not a new engine world-size limit.

## Verification and observed result

Current binary and independent proof session69f40f366971ce24. World seeds81477
and12345 each generated40 distinct native meshes. For each world, one representative
slot was regenerated in an independent native session: exact geometry SHA256 and
accepted operation list matched; changing only world seed at the same slot gave
a different native geometry hash. All40 source-control recipes repeat exactly.
This does not assert full-world native re-bake of every object twice or universal
cross-platform determinism.

Current delivered seed12345:40unique raw geometry hashes AND40unique normalized
cap-boundary sample hashes (translation/axis scale removed, six-decimal diagnostic;
not a general rotation-invariant shape-equivalence solver).40accepted candidates,
0critical retries,22detail proposals conservatively refused without geometry change.
Seed81477 earlier required4second candidates; all40completed without fallback.

Seed12345 rock totals21461vertices/39870triangles; compact caps2976vertices,
not40copies of full rock vertex payloads. Scene2186151bytes/replay2962283bytes;
these are serialized file sizes, not RAM/GPU footprint or performance measurements.
Scene save/load PNG byte equality and replay/mesh agreement pass. Python compile/
diff pass. Optional placement seed parameter preserves the previous amphitheatre's
exact default placement records. Baseline patchy scene SHA256 unchanged.

Overview/valley captures for both seeds inspected; reverse current capture also
reviewed. Distinct wide/narrow, elongated/compact and concave contours replace the
repeated silhouette. Large clean faces, simplified lower support walls and some
local artificial cut marks remain. Individual baseline approval does not approve
all generated shapes or this new scene. Separate objects may overlap; no global
Boolean composition, physical tectonics/erosion, hydrology, streaming or FPS claim.
Directional light without panorama cast shadows follows previous composition.

## Static performance post-review

- Per-frame generation/allocations introduced? No: all source and CSG work is cold.
- Does uniqueness require separate geometry? Yes, forty real bakes; no claim that
  distinct forms share one mesh. Existing renderer still caches by content.
- Unnecessary payload copies? Native ExtractIslands compacts both caps and rocks;
  MCP inspections/atomicity snapshots are bounded cold diagnostics, not frame work.
- Work/retries bounded? At most40slots/3critical candidates/20cuts per accepted
  candidate, native hull/input/pair guards retained; explicit failure on exhaustion.
- New renderer buffers/passes/formats? None; existing mesh/collider paths. No build
  copied, no Rust rebuild attempted over parallel botany source modifications.
- Does verification establish FPS or arbitrary-seed acceptance? No.

Run from project root:
python3 scripts/render_seeded_plateau_world.py --seed 12345
Changing --seed changes both geometry and placement. Default81477 is also tested.
examples/seeded-plateau-world.recipe.json stores compact seed input/expected hashes
and views; full native source/cut traces are artifacts/seeded-plateau-world/bake-records.json.
Current editable scene, PNGs, replay and verification are in that same artifact
folder; generation replaces them, not a scene/build history. Prior approved object
and cloned-composition artifacts untouched. No window opened, no push. EAF-221
remains InProgress for artistic acceptance and broader integration.
