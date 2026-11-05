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
