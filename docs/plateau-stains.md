# Angular stains and directional triangular rock prisms

EAF-221: user's new construction rule replaces the previous reference-copy
attempt for this experiment. The top plan resembles an irregular spill, with
straight angular edges rather than rounded lobes. Three spill seeds are shown.
Walls carry triangular-section prisms with varied corner angles, width, depth,
length and taper. Shared preferred direction[0.38,1,-0.42] plus bounded local
normal/jitter gives a coherent family, rather than independent random rotations.

`scripts/plateau_stains.py` authors native Landform source. It does NOT triangulate
or render geometry. Source controls: seed, width/depth/height, contour_points,
rock_count, fracture_direction, rock_width/depth. The driver
`scripts/render_plateau_stains.py` queries/publishes all geometry via MCP and
persists editable sources/parameters in examples/plateau-stains.recipe.json.
This is a client authoring recipe over existing native tools, not a new native
Landform format or a built-in geological simulation.

One polygon body plus26 triangular-section rock volumes per plateau, at most31
rocks+body under the32-piece source limit. Contour has28 polygon vertices with
coherent lobes/recesses. Triangular profiles are recentered at their centroid;
two sections vary taper/offset. Original trial looked like small attached pins;
widths enlarged to9..22m, depths8..16m, lengths65..105% of plateau height,
roots lowered into the base. Independent triangle corners/skew create acute and
obtuse forms. This is an intentional prism assembly, not Boolean-welded topology.

Rooting is checked before publication: native body-only raycasts at three samples
along each prism centroid axis. At least one odd positive-intersection count
indicates a sample inside the closed body. Missing roots are moved10% inward
and rechecked, at most5attempts. This proves sampled overlap, not CSG union,
perfect junctions, all collider behavior or geological realism. Final78prisms
pass this check across three plateaus. Every published compound mesh has zero
boundary/nonmanifold edges; components intentionally overlap.

Static performance before/after: cold bounded recipes only;3compound assets,
240vertices/372triangles each, same native renderer/materials/collision options.
Body plus26prisms per asset; no per-tick generation, new passes or dependencies.
Native ray probing is bounded3samples*26prisms*5attempts per plateau. No profiling
or FPS claim. Skin of grass is presentation-only at+0.75m to avoid depth conflicts;
no physical soil thickness inferred. Roof remains flat while its boundary varies.

Overview/top/near captures were opened. Top confirms the spill lobes/recesses;
near confirms actual triangular projecting faces with correlated inclinations.
Some faces still look too geometric and blank wall remains; no Genshin likeness
or human artistic approval is claimed. verification.json records build/counts,
root checks and byte-equal scene-save/load PNG. Python compile/diff pass; unchanged
Rust tools do not require a new broad suite/build. One current engine binary,
headless MCP, no window. Parallel botany/LoS files preserved. EAF-221 remains open.
