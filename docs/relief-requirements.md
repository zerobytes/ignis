# Structured relief: technical requirements (EAF-221)

The three user-supplied references define morphology, not a redistribution of
their game assets. This document specifies reusable authoring tools. Geological
process simulation and artistic acceptance are separate gates.

| ID | Requirement | Observable acceptance |
|---|---|---|
| R1 | Hierarchy: region, dominant mass, shoulders/ridges, blocks and fissures | Three recipes contain dominant and subordinate volumes, varied proportions and directions, and named editable source pieces. |
| R2 | Terraces with independent outlines and irregular flat tops | Sections may change individual contour radii; rays recover horizontal ledges at authored heights. |
| R3 | Vertical, inclined, recessed and projecting walls | Real triangles form vertical walls and overhangs; one vertical ray can hit both roof and underside. |
| R4 | Slender towers with variable silhouettes | Each station can change centre, size and individual contour radii; towers differ in height, taper, shoulders and crown. |
| R5 | Angular mountain masses and subordinate ridges | Shared tools compose nonuniform masses, peaks and inclined slabs, rather than repeated cones or one constant-height rail. |
| R6 | Negative space is real | Existing concave profiles/cavities retain openings; source queries and published mesh collision use those triangles. No boolean union is claimed. |
| R7 | Reproducible procedural authorship | Seeded recipes emit ordinary editable native documents. Python authors parameters only; Rust creates all meshes. Equal recipe/seed produces equal source/geometry. |
| R8 | Plane and sphere share tools | Same sections/contours are compiled through either identity placement or a radial exponential chart. Planet centre uses precise cell differences; no global f32 position. |
| R9 | Shape-derived data | Geometry queries return actual positions and normals, suitable for slope/altitude placement. Automated watershed/ecology/snow simulation is a later consumer, not a hidden image trick. |
| R10 | Safe authoring and persistence | Versioned bounded source, atomic edits/publication, source save/load, undo/replay, legacy byte roundtrips and explicit rejection of invalid shapes. |
| R11 | Practical execution | Compile on explicit authoring/generation only; published geometry is immutable/shared. No source graph, JSON, random geometry or triangulation per frame. |
| R12 | Continuous world integration | Volumes must eventually join world residency, collision, navigation and visibility. Local source placement is not that integration: current transported planetary runtime rejects authored meshes. Keep this gate open. |

## Representation and first implementation

Surface remains the common height-field kernel for broad relief. Landform2
extends the existing closed loft representation for vertical walls, ledges,
undercuts and towers. It does not allocate a voxel grid for the entire world.

Each original profile vertex defines a ray from its local origin. A section may
supply one positive `radial_scales` value per profile vertex, in addition to its
existing uniform scale and centre offset. The base contour must be strictly
star-shaped about the origin when these controls are used. Angular ordering is
fixed. Positive radial interpolation therefore stays inside ordered angular
wedges; section planes remain monotonic. Arbitrary contour correspondence/twist
is deliberately not accepted as if it had the same safety properties.

Horizontal steps require a shared centre and strictly nested contours in one
direction at every vertex. Caps are triangulated from their actual endpoint
contours, not reused from a shape that may have different concavities. Cavity
rings use the same section factors. Degenerate rounded triangles, invalid
orientation or a collapsed shell reject compilation rather than repairing it
by crossing the opening. Parts may overlap in a composition; they are not a CSG
union and are not a certificate of arbitrary imported-mesh self-intersection.

Plane placement preserves existing arithmetic and bytes. Sphere placement uses
local `(x,y,z)`, tangent distance `rho=hypot(x,z)`, angle `rho/R`, and radial height
`R+y`. Positions relative to the authored origin are evaluated with `sinc` and
`-2R*sin(angle/2)^2` to avoid subtracting planet-sized nearly equal numbers.
The chart must stay within a hemisphere and above the planet centre. Triangles
are still straight; report domain and vertex-rounding approximation separately.
This is placement of authored volumes, not a change to the live planet runtime.

## Static performance review before code

- New arrays are bounded at decode: at most96 radial factors per section,
 128sections and32pieces per compilation artifact. These are regional work/data
 budgets, not limits on total world area or total scene population.
- Preflight work includes extra endpoint triangulations and section factors;
 reject before large allocations. Reuse prepared endpoints, frames and buffers.
- Domain projection and orientation checks happen at compile time. Optional
 source-space scratch exists only for spherical authoring, never during draw.
- Keep source1 serialization and its caps/arithmetic unchanged; encode its own
 envelope version. Source2 cannot be smuggled inside a source1 envelope.
- Reuse native publication, shared assets, material and collision paths; no
 renderer, material, global Scene/replay version or new dependencies are needed.
- Validate geometry with critical Rust/MCP cases and inspect real captures. A
 small technical fixture does not establish world integration, FPS, or artistic
 fidelity to these references.

## Native authoring API

Landform documents now accept `version:2`, optional `domain`, and the following
section controls (the four values below correspond to a four-vertex profile):

```json
{"at":20,"offset":[2,-1],"scale":8,"radial_scales":[1,0.8,1.2,0.95]}
```

The radial multiplier is dimensionless, finite and in `0.05..20`. An omitted or
empty array preserves the original contour. A supplied array must have exactly
one entry per profile vertex. The effective radius is the profile coordinate
times `scale` times that vertex's radial multiplier. It can change independently
at every station; the last solid station may still collapse to an apex when
`scale:0` and the radial array is empty. Existing offsets, arbitrary piece axes,
concave cross-profiles, cavities and angular endpoints remain available.

`domain` is absent/`null`, `{"kind":"plane"}`, or:

```json
{
  "kind":"sphere",
  "center":{"cell":["0","0","0"],"offset":[0,0,0]},
  "radius_meters":6371000,
  "forward":[0,0,1]
}
```

For this example the document origin is `[0,6371000,0]`, encoded with the normal
Position cell/offset contract. The origin must lie on the reference sphere;
`forward` selects tangent orientation. Radius accepts `5..5e11` metres. These
are local regional charts: each must stay within a hemisphere. Existing piece
coordinate, vertex and work budgets still apply per artifact; a source is not
one mesh for an entire planet. Global positions remain precise Position values.

MCP `query_landform` provides `preset`, `inspect`, `compile`, `edit`, `raycast`.
An edit batch accepts `set_domain`, `set_sections`, `set_profile`, `append`,
`replace`, `remove`, `set_inner_scale`, with an expected source revision. New
controls promote source1 to source2. Removing them does not silently downgrade
the format. Legacy presets continue emitting source1 until it is extended.
`save_landform`/`load_landform` preserve the source envelope version;
`publish_landform` compiles and publishes through the existing native transaction,
undo and optional `collision:true` path. MCP precedes any specialized GUI panel.

Ray hits add `altitude_meters` and `slope_degrees`, using world Y on the plane or
radial up on the sphere. Top-facing surfaces approach0 degrees; vertical walls90;
undersides180. Queries remain diagnostics, separate from gameplay broadphase.
Compilation reports `maximum_vertex_rounding_meters` and, for spheres,
`maximum_sampled_domain_error_meters`. The latter compares the mapped source at
three edge midpoints and the centroid of each triangle with the rounded straight
triangle. It is a sampled diagnostic, **not a certified global error bound**.
No automatic refinement or world LOD selection is inferred from that number.

## Delivery matrix

| Requirement | Current evidence | Remaining gate |
|---|---|---|
| R1 / R5 | Editable recipes for terraces/inclined slab, towers/plateau, and dominant ridge/shoulders/talus | Landscape composition, scale transitions and artistic acceptance against the references |
| R2 / R3 / R4 | Independent contour radii, actual horizontal ledges/undercuts and varied station profiles | Arbitrary twist/correspondence and unrestricted sculpted shapes are outside this representation |
| R6 | Concave endpoint caps and hollow opening checked by independent topology, volume and rays; KCC passes under a roof | Boolean union, branching caves, excavation and navigation of all volumes |
| R7 | Same seeded recipe authors ordinary source documents; Rust produces every mesh | Regional placement/masking rules and connected geographic generation |
| R8 | Same source in identity/radial charts; Earth/large radius and huge-cell translation regressions | Integration with transported-world residency, actors and collision (R12) |
| R9 | Actual hit altitude, slope, normal and source piece identity | Automatic ecological/snow/rock masks and watershed/hydrology consumers |
| R10 | Version1 source/geometry unchanged; source2 roundtrip, bounds, atomic rejection, undo and replay | Human editing/art acceptance; no arbitrary mesh intersection certificate |
| R11 | Static performance review below; no frame/tick or renderer changes | Streaming/generation scheduling and performance measurements at world scale |
| R12 | Explicitly open; the live transported planet currently rejects authored meshes | Residency ownership, source coverage, cost/collision/navigation integration before showing these on the walkable Earth |

The green lids are thin diagnostic source volumes, not generated vegetation or
an automatic material classification. Composition pieces overlap; hidden internal
surfaces are retained. The six captures are tool-validation blockouts, not a
replacement for the user's Earth scene, nor a recreation of the supplied artwork.

## Static performance review after implementation

1. Frequency: all new work runs during explicit compile/edit/query operations.
   Render frames and simulation ticks consume the existing immutable mesh.
2. Memory: compiled vertices/triangles and spherical source scratch reserve their
   predicted sizes once. Plane compilation allocates no spherical scratch.
   At the16,384-vertex artifact budget, source scratch is at most384KiB of f64
   XYZ, transient during compilation. No planet-sized volume or dense grid.
3. Work: bounded preflight accounts for up to two extra endpoint triangulations
   and radial section processing. Spherical compilation adds one projection per
   vertex, an orientation check and four diagnostic projections per triangle.
   These are real extra cold costs, not free deformation; budgets are abstract
   operation estimates, not milliseconds or a cancellation deadline.
4. Data: scalar f64 source and Position cell arithmetic become native f32 mesh
   coordinates relative to the authored origin. Renderer layouts are unchanged.
   JSON remains the command/authoring boundary, not the draw or physics loop.
5. Reuse: publication retains existing Arc geometry, GPU caching, material and
   collision paths. No new dependency, shader, readback, task worker or lock.
6. Queries: existing bounded linear ray diagnostic is retained. It is not a
   replacement for the spatial index used during gameplay. Rust callers may
   reuse a `Compiled` value; MCP query batches compile once per request.
7. Limits: closed oriented pieces and positive volume do not prove arbitrary
   self-intersection freedom, unions or geological plausibility. Source save
   validates authorship; full compile/publication additionally checks rounded
   geometry and radial triangle orientation before publishing anything.

## Reproduction and evidence

```sh
cargo test --locked --release --test relief_sections --test landforms --test terrain
cargo clippy --locked --release --lib --bin ignis --test relief_sections --test landforms --test terrain -- -D warnings
cargo build --locked --release --bin ignis
python3 scripts/verify_relief_tools.py
```

The nine Rust tests cover actual ledges/undersides, hollow space, endpoint
concavity, legacy bytes, version mismatch, bounded decoding, invalid charts,
Earth/large-radius placement, equatorial orientation and coordinates beyond
exact f64 integer cells. Existing terrain regressions stay in this focused gate.
MCP verifies six plane/sphere compositions, source save/load, late-error atomicity,
undo/redo, published collision, under-roof KCC travel/replay, and exact capture
save/reload. The old native binary's source/geometry hashes are fixed in the
verifier rather than regenerated from the new implementation.

Outputs replace the current `artifacts/relief-tools/verification.json`, source
files, scenes and six PNGs. First six compositions contain between2,391 and3,900
rock vertices (4,742..7,748 triangles), plus separate diagnostic lids. On Earth
the largest sampled mapping deviation was below0.8mm for these recipes; this is
not a general guarantee. No FPS measurement or artistic acceptance is claimed.
One current executable remains at `target/release/ignis`; no GUI/window is opened
by the verifier. Human scenes/windows and parallel work remain untouched.
