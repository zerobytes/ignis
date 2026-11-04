# User-authored rock sample: inspection and isolated solid-union proof

User supplied simple-sample.blend as an explanation of interlocking rock volumes,
not a finished plateau or a single mesh. Blender5.0.1,16mesh objects, no modifiers.
Fourteen meshes have8vertices/6faces; two have13vertices/9faces. Count is source
structure, not a complexity/quality score. World-space vertices, triangle loops,
bounds and face normals are in ignored artifacts/simple-sample/geometry.json.
Original file and live scene were read only, never saved or edited. Headless
factory-startup Blender performed audits/renders without opening another window.

## What the sample establishes

Broad tapered slabs/trapezoidal solids overlap at different depths; long exposed
planes form large parts of the wall itself. One large backing volume spans several
smaller elements. There are multiple visual orientation families, rather than one
normal/tangent-oriented line of slender triangular projections. Different lengths,
widths, thicknesses, rotations and overlap depths create recesses, shoulders,
overhangs and occlusion boundaries. Flat planar faces remain readable. The user
explicitly says there is no plateau here; none is inferred from this sample.

The earlier stain/prism generator overemphasized narrow independent projections
and a single preferred axis. This sample supplies actual editable 3D geometry
instead of an image-depth inference. It should be retained as the geometric target
before attempting more procedural variation or fine surface texture.

## Isolated feasibility experiment

Triangulate the source faces in Blender, transform vertices to world space and
correct orientation if needed. Run sequential f64 Boolmesh Add in a temporary
standalone Rust probe using the already resolved release dependency; start with
Cube.015 (the broad backing volume), then add the remaining solids. No source code
or native Ignis command was changed. No voxel remesh/smoothing/reconstruction.

Result:287vertices/570triangles, kernel reports manifold. Independent edge/face
traversal:0boundary edges,0edges with>2uses,1connected face component. This proves
this specimen can be represented by a single connected closed surface through
this kernel. It does not establish a universal Boolean robustness guarantee,
plateau integration, f32 packing acceptance, materials/rig propagation or gameplay
collision. The native engine still needs a bounded union authoring operation.

Original and unified specimens were rendered from front/right/threequarter/back
in Blender Workbench. A first comparison used independent automatic framing and
was invalid as an equality test. Re-rendering fixes both camera calculations to
the original sample's bounds and point cloud. Native Ignis compare_images results
on700x700 silhouette images:

| View | Changed RGB pixels | Maximum channel difference |
|---|---:|---:|
| Front |4|3|
| Right |2|49|
| Three-quarter |0|0|
| Back |3|18|

These are silhouette-raster comparisons, not a whole-material or physical
fidelity certificate. Front/three-quarter united matcap images were also opened;
large planes and their occlusions are retained visually. The source's topology
need not be retained: intersected faces are split and internal faces removed.
Output is in artifacts/simple-sample/union.json; fixed-original/fixed-union images
and fixed-comparison.json preserve evidence. The ephemeral compiled probe was
removed. No engine binary, source format or runtime performance claim changed.

## Native topology direction (not yet implemented)

Keep authoring volumes and their parameters, compile the exposed boundary of their
union with a recessed plateau core, and preserve hard planar normals. Do not merely
join object lists or proximity-weld vertices: that leaves intersecting/internal
faces. Do not replace these planes by a smooth displaced heightfield. Overhangs
require the existing volumetric/mesh terrain path, not one height per XZ.

Parameters should describe multiple plane/slab families: frame/orientation and
angular spread, broad span, thickness/taper, placement extent and overlap depth.
A plateau footprint/top cap is a separate input. The core must not occlude the
intended rock relief; volume placement and recess depth need explicit controls.
Blender Z-up to Ignis Y-up conversion and local-origin precision are explicit
adapter steps. This geometric sample should first be reproduced unchanged through
the native MCP path, then unified with a plateau, before any stochastic generator.

Static performance review: Boolean generation is cold authoring/bake work, not a
per-frame operation. Input/output/work limits and precision validation are required
for integration. Runtime consumes the compiled boundary; this does not imply free
destruction/regeneration. Existing subtract adapter is a useful numerical/atomicity
reference, not an already implemented Union API. EAF-221 remains InProgress.
