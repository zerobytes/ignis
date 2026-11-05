# Dual Contouring rock experiment

Scope: a local 24 × 12 metre cliff specimen, independent of the approved terrain.
It evaluates planar rock geometry at inspection distance. It does not change terrace heights or planet streaming. The requested independent
seeded choice of access location per adjacent level remains a separate next step.

## Native authoring contract

MCP `engine_command` → `publish_fractured_rock`, with `publication` containing
`settings`, asset/instance IDs, position, color, material and optional collision.
The publication shares the normal scene edit transaction and undo path. It
requires paused execution. The resulting static mesh is saved in the Scene;
the editable source parameters live in `examples/dual-rock.recipe.json`.
No new Scene or volume source version; existing generators remain unchanged.
There is no dedicated visual control yet (MCP first).

`python3 scripts/author_dual_rock.py` authors the specimen, saves its Scene and
captures three views through MCP. Python supplies parameters/material/cameras;
the density field, Hermite data, QEF and mesh are entirely Rust. This is an
authoring entry point, with no automated acceptance checks or test scenarios.

## Field and extraction

Positive means solid. Union of convex oblique strata overlaps a common solid
core; clipping gives a finite specimen. Shared bedding and cross-joint shear
set the dominant direction; seed changes layer thickness, widths, exposure,
face angles and chipped margins. Each convex uses normalized plane distances:
`f_i(p) = min_j(d_j - n_j · p)`. Union uses maximum; clipping uses minimum.
This is a procedural geometric model, not tectonic or physical erosion simulation.

The prototype follows the uniform-grid Dual Contouring construction in Ju,
Losasso, Schaefer and Warren, *Dual Contouring of Hermite Data* (SIGGRAPH 2002):
https://people.eecs.berkeley.edu/~jrs/meshpapers/JuLosassoSchaeferWarren.pdf

Each sign-changing lattice edge supplies a surface point (28 bisections) and
analytic outward plane normal. One vertex per active cell minimizes
`sum((n_i · (x - p_i))²)`, with tiny mass-point regularization. A 27-state active
set solves box constraints rather than clipping an unconstrained answer.
Coordinates are relative to the cell for numerical stability. Four incident
cells form each dual quad; a shorter-diagonal triangulation supplies the renderer.

This is not the paper's octree/simplification extension. One vertex per cell
cannot represent arbitrarily many disconnected surface branches. Features below
the grid interval may disappear; ambiguous cells, self intersections and manifold
certification remain unresolved. Degenerate triangles are omitted and reported;
this must not be presented as universal watertight geometry or production terrain.
A padded domain rejects surfaces touching the outer lattice boundary.

## Static performance review

- Generation/solving occurs only during authoring; no per-frame procedural work.
- Lattice samples and Hermite edges are cached; adjacent cells reuse edge data.
- Cell scratch reuses a 12-entry buffer; no per-sample heap allocation.
- Cold work is O(samples × fracture planes + crossing edges × 28 × planes +
  active cells × 27 small solves). This is deliberately a CPU quality experiment.
- Temporary memory is O(lattice volume): f64 samples, three u32 edge
  indices and one u32 cell index per lattice point (24 bytes total per point),
  plus sparse 48-byte Hermite records only at crossing edges. This is not sparse/streaming.
- Existing 2.2M-sample/600k-vertex/1.2M-triangle budgets apply; source fracture
  count has a separate 512-cell guard. Finite metric inputs are checked before
  allocating the lattice; over-budget publication fails before scene mutation.
- Plane evaluation short-circuits once a convex candidate cannot beat the union.
- Local normals are analytic, no finite-difference noise sampling per edge.
- Runtime reuses the published mesh and ordinary material/collision machinery.

Validation for this work is compilation/static analysis and direct MCP render
inspection, following the current no-new-automated-tests instruction. Artistic
acceptance, physical traversal, FPS, global geology and production integration
are separate decisions.

## Local visual inspection

The authored specimen uses seed2718, width24m, height12m, depth3m, relief2m,
nominal layer2.2m/block3.8m and spacing0.14m. Native MCP returned105fracture
cells,1352988lattice samples,58640Hermite crossings,58598vertices and117280
triangles; no triangles omitted as degenerate. Reported generation time was
0.567s in this local run, not a benchmark or frame-rate measurement.

Direct inspection of `artifacts/dual-rock/{overview,close,side,neutral}.png`
found planar projections, recesses and bevels, with the later specimen breaking
the first attempt's regular brick rows. Small triangular facets remain around
some multi-plane junctions. They are visible without the material and remain an
open limitation, not artistic acceptance. The original terrain scene is untouched.
Material is the existing catalog fractured rock, mapped in metres. Directional
shadowing is enabled for this close-range study; it is not a terrain optimization.

The Scene is `artifacts/dual-rock/world.scene.json`; generation metadata is
`artifacts/dual-rock/generation.json`. These current local outputs are overwritten
by authoring again; no binary snapshots or alternative build folders were added.
No new automated tests or verification scripts were created or executed.
