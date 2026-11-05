# Seeded terrain chamfer and vertical scale

EAF-221. The ten-kilometre fixture now uses vertical feature scale160 instead
of8, retaining the same horizontal footprint and density sample count. This is
20× the preceding vertical relief, not20× the horizontal detail. Density domain,
terrace spacing, floor and sampling spacing all scale together; no instance-only
stretch that disagrees with the physical mesh.

`StyleMeshBySlope.chamfer` is an optional native Rust cold-authoring operator:
`{fraction:0.16,variation:0.75,seed:2718}`. It insets each triangle's corners
towards its centroid by an independently seeded fraction, joins neighbouring
faces with new strips and closes each vertex fan with indexed cap triangles.
Coplanar source faces remain coplanar; dihedral edges acquire new cutting planes.
This is a bevel/chamfer, not a noise texture or physically calibrated erosion.
Widths depend on local source triangle dimensions; very fine cuts require a
finer source mesh. No claim of smooth LOD or geological realism is implied.

Open boundary vertices remain bit-exact, preserving existing block seams.
Nonmanifold edges, disconnected vertex fans and inconsistent winding refuse the
operation. A static source is required by the parent slope tool; scene edits and
collision replacement are atomic and undoable. No source file-format change.
The operator supports concave neighbourhoods but is not a general intersection
repair/CSG solver: narrow folds can require a smaller fraction.

## Static performance checklist

- Hot-path work? None: runs once when authoring, not on frames/ticks.
- Scaling? Ordered edge adjacency O(T log T), corner/fan work O(T+V).
- Allocation? Bounded corner arrays, adjacency and output; no per-frame copies.
- Geometry growth? Up to roughly13T triangles before collapsed-face removal;
  measured output is reported by MCP. Cost is real and distinct from material.
- Budget?100000source faces/vertices per cold operation, parameter checks before
  output work; partition larger authoring inputs. Not a world-area restriction.
- Physics? Published collision consumes the refined indexed geometry, with no
  new solver or physics/render mismatch. No streaming/LOD inferred.
- Determinism? Integer seeded hashing; same source/config gives exact arrays in
  the same build. Open block borders are not randomly displaced.

## Validation result

Six release integration tests passed (chamfer2, slope1, flight3), including
closed/oppositely paired edges, positive reduced cube volume, exact repeatability,
changed-seed geometry, fixed open boundaries and invalid configuration. Focused
strict release Clippy, release binary, formatting and Python checks passed.
The full terrain recipe regenerated in~1.05s including read-only seam checks
on this run; this is generation time, not FPS.24source block borders matched,
no vertical clipping; range−4335.80..2608.62m. Refined terrain has1244386faces;
MCP save/reload pixels,600m/s fast and2m/s precision flight passed. Captures were
inspected; no artistic or universal self-intersection acceptance is inferred.

## Human review: height approved, composition correction

Guilherme approved the height impression of the20× vertical-scale variant on
2026-11-05. This approves height, not the whole composition or the loss of broad
plateau tops. Its editable density recipe is preserved independently as
`examples/volume-terrain-mountain-scale-approved.recipe.json`.

The inspection must start from the preceding plateau density recipe (height8),
not the stretched mountain variant. `StyleMeshBySlope.terrace_lift` separates
stratum elevation from within-stratum detail. In world Y, band=floor((Y-phase)/S),
Y'=Y+(factor-1)*S*band. With S=61.5384615384m and factor20, heights inside the
same original band retain their differences; neighbouring bands acquire an
additional1169.23m separation. X/Z and triangle connectivity remain unchanged
before chamfer. This is an authored discrete band lift of an existing mesh,
not a continuous density-field deformation or a geological simulation; triangles
across bands form the taller walls. The same mapping applies to shared border
vertices. Unit scale/world-aligned instances only; other poses refuse atomically.

Performance review: one cold O(V) position pass and one geometry copy before
chamfer; no added vertices from lifting and no frame/tick work. Material rest
coordinates are rebuilt from the new positions. Validation rejects nonfinite
parameters/output; no global world-area cap is introduced. The existing coarse
source and chamfer growth remain separate limitations.

Correction validation: original document and all16grids compare exactly with
commit57110dd;182714raw triangles remain. Seven release integration tests pass,
including a60m band example:1m local relief stays1m while60m separation becomes
1200m. Final MCP output1244386faces:360728stone/883658green; pixel reload,
hover/vertical/600m fast/2m precision and persistence refusal pass. Capture
review confirms broad green tops again. Strict Clippy was blocked by concurrent
botany edits (`botany.rs`/`editor_botany_ui.rs` collapsible-if), not changed here.
The release binary built by the successful test run was used for these MCP
checks; a redundant subsequent rebuild after parallel source changes was canceled.

## Plateau scale correction: seeded3–15m strata

Latest human feedback: corners improved and approved; plateau walls remained too
high. The large-height preset remains useful for large mountains, not this
plateau demo. `TerraceLift.random_heights` now accepts min/max/seed. This fixture
uses3/15/2718. Each global stratum interval receives a deterministic bounded
height; a prefix profile anchored at datum0 maps the original coordinate into
that interval. Different bands vary, while connected pieces of the same stratum
share a height (not independently randomized instances). Within-band coordinates
are proportionally compressed, not given kilometre offsets. This is a target
height per stratum interval, not a universal bound on an entire mountain.

`StyleMeshBySlope.wall_rock` is seeded native geometric relief, applied before
the approved chamfer. It adds asymmetric stratum/joint teeth along wall normals,
with controls for amplitude, stratum height, joint width and seed. Top-contact
vertices and open boundary fans stay fixed. No detached rock objects or shader-
only substitution; collision and visual geometry still publish together. This
first relief follows the available source vertices; no claim of geological
fracture simulation or fine near-player tessellation.

Static performance review: random band heights precomputed once (at most4096
bands each side of datum per operation), then O(V) mapping. Wall relief is
O(T+V) accumulation plus a position pass, no adjacency hash or added triangles.
Normals and oriented boundary sums are dense scratch arrays, never per-frame.
Chamfer keeps its existing output cost. Source material recipes and input source
geometry stay editable; the optional operators preserve prior defaults.
Relief displacement is reduced globally by halves (up to8reductions) if any
source face would reverse its normal; unresolved inversion refuses publication.
This prevents local normal inversion, not all possible global self-intersections.
Each vertex's displacement budget is also limited by20% of its shortest incident
edge, so tiny extraction triangles do not receive metre-scale deformation.

Current validation:9release tests passed (slope/profile/relief4, chamfer2,
flight3), focused strict release Clippy and formatting/Python/diff checks passed.
The full MCP recipe compared a stone output against amplitude0 and confirmed
actual geometry differs. Output remains1244386triangles (149382walls,
1095004tops), so relief added no further triangles. Scene save/load pixels,
hover,600m fast and2m precision per second and persistence guards passed.
Captures inspected; small surface resolution limits remain explicit. Inspection
starts nearer the wall, with a horizontal offset and eye near wall mid-height,
rather than carrying the high-altitude spawn offset from the giant-wall demo.

## Regression correction: preserve the approved surface before reducing height

Human review rejected the compact profile: it compressed the raw density surface
before chamfer and classification. Besides changing the bevel construction, this
changed material membership from360728stone/883658green to149382/1095004.
The3–15m parameter bounded abstract raw-height bands, not measured plateau walls;
that was insufficient validation and must not be reported as visual acceptance.

`StyleMeshBySlope.finish_heights` now remaps Y AFTER the approved high plateau
mesh, chamfer and material partition have been constructed. Current recipe first
reproduces the prior factor20 surface with no new wall relief, then maps its
actual stratum anchor spacing1230.769230768m (phase30.7692307692m) to seeded
3–15m intervals. X/Z, triangle indices, chamfer connectivity and stone/green
membership are preserved. Y is rebased to world datum before compression to
avoid wasting f32 precision on the old−384m origin; instance Y becomes0.
Material coordinates are regenerated in physical world metres to avoid shrinking
old kilometre-scale texture coordinates along with the geometry.

Static performance review: cold per-output O(V+T) map/validation and coordinate
regeneration, no new triangles, no per-frame work. Existing seeded profile
budgets apply. Unit regression compares exact material group counts, X/Z and
indices; anchor heights are measured against the player's1.6m body. Full-scene
MCP must also preserve the approved360728/883658counts. Fine rock relief remains
available but is disabled for this correction so it cannot change the reference.

Native full-scene check recovered exactly360728stone and883658green faces.
Read-only measurements of matched published vertices near three adjacent
reference plateau anchors gave10.832817m,6.581184m and7.862504m, respectively
6.77051/4.11324/4.91407times the1.6m player. This samples actual published
surface levels; it is not a universal guarantee about every small natural ledge.
Exact X/Z and triangle indices were compared against the approved control mesh,
and save/reload pixels and the existing flight/persistence checks passed.
Final validation:10release tests passed (profile/slope5, chamfer2, flight3),
focused strict Clippy passed, release binary built by the test run, focused
format/Python/whitespace checks passed. No extra faces were introduced by the
height correction. Previous relief remains opt-in, not part of the restored demo.
