# Plateau wall shaping — EAF-221 correction

## Reference analysis before construction

The user rejected the first terrace captures on 04 November: wall relief must be
part of the actual topographic mesh. This is a tool correction, not a material
change or a claim about how Genshin internally constructs its landscape.

Sources: user crop `codex-clipboard-e3fbcdc9-38fe-4792-8802-2d1c2c9771a6.png`,
882×472, SHA256 `a070fb9202d0444c788e3c4e556101fc7e2b7543afd9e2520d680eb6abb93f12`;
user Ignis capture `codex-clipboard-b8ed2f13-79f8-4732-bff2-2d9dcc3f8745.png`,
1280×800. Both are landscape orientation. Pixels use top-left origin, X right,
Y down; crop boxes below are `(x,y,width,height)`, without resizing/rotation.
The source reference is supplied by the user, not an asset to redistribute.

Global reading: a broad plateau carries an irregular sloping rock wall. A large
oblique projecting slab dominates the left/centre; the lower foreground terrace
is flatter, green above and rocky below. Smaller rock volumes interrupt the
main wall at several depths. The camera sees the front and top obliquely.
No real dimensions, rear geometry, geological origin or original asset boundaries
can be recovered from this one image.

### Local inspection and revisitation

Real crops were generated and opened in `artifacts/plateau-relief/`:

| Review | Source crop | Local observation and consequence |
|---|---|---|
| V1 slab | `reference-slab.png` (85,60,575,332) | One long upper edge slopes down to the right. Its light thin top turns into a large darker oblique face; a second diagonal wedge interrupts it below. Need a wide planar projection with actual thickness and a recessed region underneath, not a ring-wide ripple. |
| V2 left wall | `reference-left-wall.png` (80,155,295,248) | Several uneven upright/oblique bulges terminate at different heights. Their shoulders and recessed joins remain visible beneath the slab. Need independent local support and displacement direction/extent. The back of each is occluded. |
| V3 right wall | `reference-right-wall.png` (570,175,298,265) | Broad wall lobes alternate with deep narrow recesses, with nonuniform spacing and angled faces. The top edge remains tied to the plateau. Need localized positive/negative relief while retaining top control. |
| V4 toe | `reference-toe.png` (112,270,538,178) | Asymmetric small angular rock forms emerge at the foot of the wall, crossing the wall/floor transition. Foreground grass covers a thinner horizontal ledge. Need multi-scale volume and independent ledge thickness. These are not all demonstrably separate boulders. |

Revisiting the complete crop confirms that V1 spans both V2 and V4; treating it
as a small detached rock would lose the dominant structure. V3 is a continuation
of the same broad wall, not a repeated copy of the left slab. Fine dark grooves
on the slab are visible but some may be appearance/shading rather than deep
geometry. Stop at the available pixels; no hidden cave or detached underside is
asserted from shadow alone.

### Consolidated construction list (before geometry)

All executable items below start **not built**, with model verification **not
executed**. The list specifies observable capability, not a metric reconstruction.

| ID / parent | Construct / visible occurrence | Evidence, relations and epistemic state | Verification criterion |
|---|---|---|---|
| P / — | Plateau wall assembly (group) | Whole crop; observed assembly, internal segmentation unknown | Review overall relation of top, wall and foot |
| P1 / P | Broad wall body with real depth | Whole crop, V2/V3; observed front, rear is a coherent hypothesis | Closed volume, oblique/front/side/rear views; one connected surface in the shaping example |
| P2 / P | Wide flat plateau top with controlled rim | Upper right V3; observed, some rim occluded by trees | Local wall shaping leaves designated top vertices at their authored height |
| P3 / P1 | Dominant oblique slab, thin light upper lip and broad projecting face | V1; observed one dominant projection, exact depth unknown | Geometry creates the long inclined silhouette and its own angled faces in neutral shading |
| P4 / P3 | Diagonal lower wedge/shoulder and recessed join | V1/V4; observed face boundary, separate-asset interpretation unknown | Independent oblique displacement and a visible recess without placing a disconnected overlay rock |
| P5 / P1 | Left vertical/oblique buttress family with different terminations | V2; at least three readable protrusions, smaller count unresolved | Separate localized protrusions with different heights/widths and joins |
| P6 / P1 | Right wall lobes and narrow irregular recesses | V3; at least three readable lobes, not an inferred full count | Positive/negative local shape controls alter actual ray intersections and wall silhouette |
| P7 / P1 | Angular foot transitions and low rock protrusions | V4; family, occlusion prevents complete count | Multiple spatial scales and directions, retaining contact with the body |
| P8 / P | Thin foreground terrace with irregular exposed rocky edge | Bottom V4; observed | Horizontal top and real wall/underside, distinct from the large upper wall |
| P9 / P1 | Fine surface grooves/colour variation | V1/V2/V3; observed appearance, geometric depth unresolved | Deferred appearance gate; do not use it to approve missing P3..P7 volumes |
| C1 / — | Trees/grass/background cliffs/atmospheric perspective | Whole crop; observed contextual families, tiny foliage not countable | Outside this geometry correction; retain in inventory, do not implement vegetation to hide wall failures |

Working axes for the diagnostic body: X along the long plateau wall, Y up, Z
depth. Front wall normal is -Z; positive projection there decreases Z. Back and
bottom depth are explicit hypotheses. One unit is one chosen working metre, not
a dimension measured from the reference. Geometry is evaluated from multiple
views; fitting a camera-facing sheet would not satisfy P1.

## Technical correction and static review before code

Landform2 changes radius along fixed contour rays in ordered planar sections.
It can represent coarse local bulges, but the current authoring surface does not
provide a reusable local 3D wall-shaping operation. The rejected recipe also has
too little local resolution and excessive coherent horizontal bands. More noise
or another material would not supply the required controls.

Add optional source3 per-piece shaping: conforming linear subdivision using the
existing native topology planner, followed by bounded oriented volumetric stamps
with independent 3D displacement. Box-shaped support, optional flat core and
linear/smooth falloff allow broad faceted plates, recesses and buttresses. Stamps
sample the frozen source mesh; they do not recursively stamp already-displaced
vertices. The resulting vertices remain part of that piece's connected surface.
Apply before radial placement so plane/sphere consume identical shaping data.

Performance questions/gates:

- Is anything generated per frame? No: compile/edit only, immutable mesh after.
- Can refinement explode allocations? Preflight predicts closed-surface counts
  for up to3 subdivision levels and charges vertex×stamp work before allocating.
  Retain per-artifact vertex/triangle budgets; never allocate by world area.
- Can arrays grow unbounded? At most32 stamps per piece; bounded decoder and
  finite extents/displacement. Prepare stamp frames once, not once per vertex.
- Can old files/pixels change? Source1/2 omit the new option and retain the old
  compile path and envelope; no renderer/material edits are needed.
- Can a command partially publish? Shape candidate privately, check geometry,
  winding/orientation/closed volume, then use existing atomic publication.
  These checks do not certify arbitrary global self-intersection freedom.
- Can a pretty material hide failure? First use neutral material and compare
  against the unshaped source, inspecting multiple views and actual queries.

## Native source3 contract

`Piece.shaping` is optional; source1/2 omit it and retain their existing compile
path. `query_landform.edit` adds `set_shaping {id,shaping}` and promotes to source3
only when needed. Domain/radial-only edits still promote to2. The same ordinary
document can be manually edited or generated by a game/tool; stamps are not a
private evaluator tied to one NPC, biome, agent, asset or AI vendor.

Shaping has `subdivisions:0..3`, up to32 `stamps`, and up to24 `creases`.
Uniform refinement reuses `mesh_topology::Plan`; it preserves the source surface
and creates shared midpoint vertices, without smoothing its silhouette.
Creases are optional `{point,normal}` planes in the frozen document XYZ. They
split both sides of a triangle, retaining its area and sharing edge intersections
between neighbours. They insert actual mesh edges without opening a crack,
removing a half-space or pretending to be a boolean union. Nearly coplanar points
reuse their IDs using a scale-relative local f32 tolerance.

Each stamp contains:

| Field | Meaning |
|---|---|
| `center` | Local XYZ centre, metres |
| `axis`, `profile_x` | Normalized orthogonal stamp frame; second axis is their cross product |
| `half_extents` | Metres along profile_x, axis×profile_x, axis; each0.0001..10000 |
| `displacement` | Independent document XYZ displacement vector, each component bounded to±10000m |
| `flat_core` | Dimensionless0..0.95 fraction with full displacement, default0 |
| `falloff` | `linear` (default, piecewise planar) or `smooth` |

With normalized local coordinates `u`, `q=max(abs(u.x),abs(u.y),abs(u.z))` and
`w=clamp((1-q)/(1-flat_core),0,1)`, the stamp contributes `w*displacement`.
Smooth falloff replaces `w` with `w*w*(3-2*w)`. Sum contributions evaluated at
each **frozen source vertex**, not at vertices displaced by earlier stamps.
That supports oblique buttresses, pockets, flat projected areas and deformation
with tangential/vertical components, rather than only radial scaling of a ring.

Refinement/cuts/shaping precede radial placement. They operate on the native
local f32 mesh, with f64 calculations and shared indices; no planet-sized f32
coordinates are introduced. All stamping/cutting is cold authoring work.
Final geometry, orientation and closed-volume checks run before publication.
`maximum_vertex_rounding_meters` includes the largest individual conversion in
base generation, refinement, crease insertion, shaping and radial mapping. It
does not sum/amplify conversions into a global source-to-result error certificate.
Shaping rejects inverted/collapsed faces; these predicates are not a proof of
arbitrary global self-intersection freedom. There is no automatic clamping or
repair that quietly changes the authored amount.

### Why crease alignment was added

The first 3D-stamp experiment displaced the surface correctly, but flat-shaded
captures exposed narrow triangular transition bands. Turning off shadows did
not remove them; smooth shading softened them but did not define the large
rock planes. Neither treatment was accepted as a geometric fix.

Two corrections followed the visual review:

1. The dominant plate and lower wedge are structural cross-profiles lofted
   **along the wall**, with independent station factors. They are not repeated
   rings around a vertical column. The upper inner shelf remains atY40 in the
   diagnostic working units; the front lip varies independently in depth/height.
2. Authored crease planes put mesh edges on the important changes of stamp
   gradient. For a linear stamp, boundaries such as `u.x=±u.y` are planes. A
   software regression checks a faceted protrusion against its five analytic
   face normals, rather than approving a denser but still misaligned grid.

The explicit cuts are reusable tools. They do not automatically find every
feature boundary, perform global remeshing, or claim that any arbitrary stamp
set becomes an exact polyhedron. Smooth falloff still requires adequate sampling.

## Static performance review after code

- No simulation/render loop work was added. Frozen frames/inverse extents are
  prepared once per stamp during compile. Existing Arc mesh and collision
  publication paths remain unchanged; no shader/renderer/dependency changes.
- Predict uniform closed-surface output counts before generation. Each
  compilation artifact retains16,384vertices/32,768triangles; this is not a
  limit on planet area or scene population.
- Cutting is output-sensitive: refuse before each vertex/triangle insertion at
  those same caps, and check the combined pieces again before appending.
  Plane iteration charges vertices+6×faces against a separate4M-work envelope.
  Preflight source/stamp work retains its4M estimate; neither envelope is a
  real-time deadline. No exponential worst-case output is silently admitted.
- A cut uses a shared edge cache per plane; neighbouring faces reuse the same
  new vertex. Clipped triangle polygons use a four-entry stack array, avoiding
  heap allocation per half-face. Source/output triangles coexist temporarily;
  vectors/maps remain bounded by the artifact quotas.
- Intermediate source clones, base geometry and local candidate/refinement
  buffers are intentional cold costs. This path does not promise in-place
  streaming or zero allocation; plane mode does not pay them without shaping.
- JSON is only the structured command/authorship boundary. Source persistence
  uses the existing typed versioned envelope; runtime collision/draw consume
  native geometry. Source1/2 serialization remains unchanged when shaping is absent.

## Implementation and visual review

The pre-construction inventory above is retained as the original checkpoint.
The final authored source is `examples/plateau-wall.recipe.json`, one connected
rock body. Canonical geometry is entirely generated by Rust. The recipe uses
ten structural contour vertices, seven longitudinal stations, nine local stamps
and ten crease planes, without uniform subdivision. The working upper inner
shelf stays atY40. Dominant front lip and lower wedge vary independently of it.

| Item | Constructed / comparison diagnosis |
|---|---|
| P1 | One closed oriented body, one native mesh island. Front/oblique/side/rear/top captures opened. Rear is an explicit simple depth hypothesis, not a camera-facing sheet. |
| P2 | Authored flat inner top retained, with a separately varying front lip. Top view confirms depth and a nonuniform outer outline. This is not yet a matched plateau ecosystem. |
| P3 / P4 | Broad projecting face, thin upper lip, independent lower profile and recessed underside are actual triangles. The neutral candidate still has different proportions and a more continuous face than the reference; morphology matching remains open. |
| P5 / P6 | Oriented positive/negative local controls are implemented and affect source/ray/collision geometry. The current diagnostic has fewer clearly separated, more angular buttresses than V2/V3. Do not claim their distribution/terminations match. |
| P7 | Low wall/foot transitions and local displacements are present, but the foot remains too continuous compared with the reference's irregular rock family. Further source composition/crease placement is needed. |
| P8 | Not built in this one-body diagnostic; existing terrace tools support it. A flat diagnostic ground is not this foreground terrace. |
| P9 / C1 | Deferred appearance/context; no texture, vegetation or atmosphere is used to hide absent volume. |

The initial dense-grid attempt produced triangular transition bands; a second
capture with shadows disabled confirmed they were not solved by removing the
shadow map. Smooth shading softened them but did not define the requested rock
planes. The current aligned-cut example removes that dense regular pattern.
An18-plane trial was refused after deformation; keeping ten useful structural
planes avoids its excessively narrow cut cells. The field amplitude was then
authored at75% for the revised deeper main profile. These are explicit recipe
decisions, not an engine fallback or automatic clamp. Arbitrary plane sets can
still fail the rounded-geometry/orientation guards.

Final view condition: `verify_plateau_shaping.py` records the five cameras,
neutral flat material, no shadow/atmosphere and a front light that reveals the
underside. The separate882×472 full RGB comparison is intentionally unaligned,
with different context/material/light from the reference. RGB MAE≈0.319 and
SSIM≈0.393 in this diagnostic do **not** measure geometric fidelity or acceptance.
Fixed source/candidate crops use the same boxes from the reference manifest;
their placement differences remain visible rather than being recentered away.

### Software evidence

Twelve focused Rust tests passed:3 shaping (including five analytically checked
facet normals),4 variable-section/domain,2 legacy landforms and3 terrain.
Release Clippy covers lib/bin and those four test targets with `-D warnings`.
MCP verifies source3 edit/persistence, publication/undo/error atomicity, one mesh
island, exact scene-save/reload pixels, five native captures, Earth placement,
actual surface rays and KCC collision/replay. The six source1/2 composition
regressions also pass, preserving the old source/geometry hashes and source1 bytes.

Current example:470vertices/936triangles, one connected piece. Twenty-one rays
measure up to4.17m of additional local stamp displacement over the structural
profile. The major projecting slab is already defined by that profile. KCC stops
atZ≈-30.915 with centreY≈0.775, in contact with the shaped foot. Earth mapping's
largest sampled triangle diagnostic is≈0.0000603m here, not a general certificate.
Build/evidence live in `artifacts/plateau-relief/verification.json` and the five
named PNGs; one current executable remains at `target/release/ignis`.

No window was opened, FPS measured, renderer modified, or transported-world
integration completed. Human artistic acceptance remains open; these results
establish the new authoring capability and its current visual shortcomings.

### Material preview requested by the user

Run `python3 scripts/render_plateau_materials.py` for the native1600×1000 overview
and closer view in `artifacts/plateau-relief/materials/`. This presentation uses
an edited copy of the existing Alpine Rock2 material at24m scale, an explicitly
authored oblique planar coordinate map, a grass surface material, directional
sun/shadow, environment fill and light atmosphere. It does not change the rock's
positions, triangles or collision. The script checks their exact shape hash;
material coordinates are intentionally edited and are not included in that hash.

The green roof is a visual copy of the existing flatY40 faces, selected and cut
with native MCP topology operations, offset45mm to avoid depth ties. It is a
surface material, not a new grass-blade generator. The original rock is retained
whole; no rocks are added to conceal its current shape. A flat surrounding ground
provides context and receives shadow. The script uses its own headless session.

Performance review: no engine code, frame generation or new pass is introduced.
The existing runtime rock/pattern evaluators add their visible-fragment cost;
lighting uses the existing shadow pass. No fixed1024 material maps are baked.
Native captures were opened and scene-save/reload produced exact PNG bytes.
Reproducible script and render metadata are retained; no new test suite/build
was needed for this presentation-only change. Morphology acceptance stays open.

## All-around strata and turntable — requested continuation

Before construction, the user identified the diagnostic as a rectangular body
with one worked face and requested rotation, more divisions and more layers of
actual protruding stone. This extends P1/P5/P6/P7 rather than approving the old
body. Its rear/endcaps were deliberately simple; camera rotation alone cannot
correct that.

| ID | Required construction / check |
|---|---|
| A1 | Retain the prior one-face example as a separate comparison; rotate it enough to expose its original rear and ends. |
| A2 | A single closed new body with an asymmetric perimeter; all four sides must contain authored relief. |
| A3 | Six banks of varying thickness with localized tilted rock lenses, recessed joins and a broken rim. Compare front/side/rear silhouettes and source ray intersections. |
| A4 | More meaningful surface divisions, not a subdivision-only copy of the coarse wall. Check side-face areas and actual source-position changes. |
| A5 | A reproducible360° native turntable plus fixed front/side/rear views with identical material/light settings. Hidden faces are authored hypotheses, not facts recovered from the Genshin crop. |
| A6 | Preserve the existing MCP/native-source route, editable seed/parameters, plane/sphere compilation and flat controllable top. |

Static performance review before this increment: author bounded perimeter and
section controls once; Rust compiles the mesh. No geometry sampling/triangulation
in Python, per-frame generation, new engine dependency or material bake. Existing
per-artifact budgets bound the source; reuse one source mesh for every camera.
Turntable captures are explicit offline work. Frames/GIF replace one current
output in a stable directory; preserve parallel lighting work and human windows.

### Delivered source and views

`examples/plateau-strata.recipe.json` is the compact authored recipe;
`scripts/plateau_strata.py` emits ordinary native contour/section data from its
seed. The compiled geometry still comes entirely from Rust through MCP. The
output source is saved as `artifacts/plateau-strata/plateau.ignis-landform` and
the complete scene as `plateau.scene.json` in that directory. Neither requires
an agent to execute after publication. This example uses existing Landform2
controls; no new engine operator/format or runtime-world integration is claimed.

The new body has64 contour controls,41 section levels, six irregular banks and
seeded local rock lenses/faults distributed over the entire perimeter. It is one
closed oriented mesh island:2,624vertices/5,244triangles, compared with470/936 in
the previous front-wall example. All four sides have measured source relief:
six height probes vary by≈12.64m(front),7.46m(rear),3.53m(left),3.56m(right).
Plane and Earth-domain compilation passed. These are geometric measurements,
not texture-bump or an artistic match to the Genshin image.

Side-face area diagnostics (faces with abs(normal.y)<0.85) decreased from a
mean23.39m²/P90=52.93m² to3.59m²/P90=6.05m². This verifies smaller source faces;
it does not establish rendering performance or visual fidelity. Source authoring
is bounded to96 contour controls,128 levels,64 lenses/bank, eight banks and a
2M cold-work estimate; native artifact quotas remain authoritative. The script
does not regenerate geometry for each camera. Increased geometry still has a
real memory/draw cost; no FPS measurement was made.

The first all-around candidate had overly continuous rounded rings. The recipe
was revised with staggered lens heights/tilts, interrupted deeper joins, stronger
local projections and faults. Flat and smooth native shading were inspected;
the current presentation uses smooth normals on the same actual source geometry.
There are still broad layered motifs; this is an inspectable procedural example,
not acceptance of the complete reference morphology.

Native views `overview/detail/front/left/right/rear.png` are1600×1000. A fixed
camera orbit supplies24 actual960×600 PNG captures around the new body, with
the same world light. Frames6/12/18 and the fixed views were opened to check ends
and rear; the old body also has a12-frame orbit, exposing its original flat rear.
`scripts/encode_turntable.swift` only assembles the native captures into a GIF
and checks decoded frame count. It synthesizes no new camera frames or geometry.
Current GIFs are `artifacts/plateau-strata/turntable.gif` (new) and
`artifacts/plateau-relief/materials/turntable.gif` (previous). Swift/ImageIO is a
local macOS export helper, not an engine dependency.

```sh
python3 scripts/render_plateau_materials.py --strata --orbit-frames 24 --shading smooth
swift scripts/encode_turntable.swift artifacts/plateau-strata/frames artifacts/plateau-strata/turntable.gif
python3 scripts/render_plateau_materials.py --orbit-frames 12
swift scripts/encode_turntable.swift artifacts/plateau-relief/materials/frames artifacts/plateau-relief/materials/turntable.gif
```

Native schema, single-island/manifold checks, deterministic source generation,
typed-source save/load and exact scene-save/reload PNG checks passed on build
83930355ffbce727. Applying materials preserved the already-generated new shape.
Evidence: `render.json` and `shape-comparison.json` beside the images; the saved
source was independently compared with the current generator through MCP.
Python compilation/diff checks and GIF decoding passed. No Rust/renderer changes,
extra engine build, window or broad test-suite rerun was needed for this authored
example. Human shape acceptance and live planetary residency remain open.

## Follow-up: native finite chips

The user's subsequent wall references require broad angular rock plates and
localized missing chips. A reusable static-mesh `subtract_convex` operation is
now available MCP-first; see [mesh-chipping.md](mesh-chipping.md) for the numerical
kernel, guards, evidence and the still-open fracture-generator contract. It
removes actual volume and closes the exposed faces. This does not change the
rounded strata recipe or add Boolean edits to Landform1..3. Its rectangular
test slab is a tool specimen, not an approved natural-rock reconstruction.
