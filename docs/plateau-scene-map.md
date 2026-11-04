# Layer map and native landscape blockout — EAF-221

Reference: user image04cca724,1920x1080. Full image and actual contextual crops
artifacts/plateau-scene/reference-main.png (x540,y220,w1250,h420) and
reference-right.png (x1250,y350,w650,h650) were inspected before construction.
Six height bands are an authoring hypothesis, NOT six measured geological
strata or a count of every visible shelf. Perspective/occlusion do not establish
exact elevations or whether similarly placed shelves share a world height.

## Construction list delivered before generation

| ID | Reference evidence and relation | Native construction / working elevations |
|---|---|---|
| L0 | Water at bottom of frame, bordering plaza | Visual plane atY-3; no fluid simulation |
| L1 | Lower courtyard above water, inside depression | Irregular plaza atY2; inferred dry bank atY-1.5 connects the terrain |
| L2 | Low shelves to right of courtyard and lower left | RightY5/leftY8, independent receding footprints |
| L3 | Right-hand middle shelves above L2 | RightY12, set back from lower shelf |
| L4 | Central grassy terrace below dominant wall, extends left | CenterY20, leftY20.7, rightY21; broad irregular rims |
| L5 | Plateau behind slab, continuing right and into distant left | Left-backY30, upperY39/rightY40; distinct depth placements |
| W1 | Dominant inclined plate and connected rear mass, center image | Existing native volumetric plateau-wall source translated to sit above L4; upper limitY60 |
| W2 | Several angular wall lobes/recesses | Shared contour/section compiler; middle radius changes and extra shoulder controls on each segment |
| B1 | Background mountain silhouettes, separate from walkable shelves | Three native Terrain2 Ridge operations in a96x48 height patch |
| C1 | Trees, bushes, grass tufts, paths, bridge, ruined walls/columns, stair/arch fragments, water-edge masonry | Visible context inventoried; faithful assets/placement are not constructed in this topology blockout. Grass is only a surface material, not vegetation |

Ten named plateau regions + one shore/bank + one main escarpment are published
as12 local closed landforms. Independent footprints/positions are in
examples/plateau-scene.recipe.json, REGIONS, source documents and view parameters.
X in REGION layout is image-layout horizontal; world placement uses-X, because
camera looks along+Z. Y-axis profile second coordinate maps to-Z.
Heights above are actual top coordinates (baseY-4 plus authored region height),
not metres measured from the reference. All depths/back faces remain hypotheses.

Terrain relationships: upper plateau behind escarpment; middle shelf immediately
in front of its foot; left chain steps down/outward; right chain has two lower
shelves; plaza sits in the bottom depression and borders the foreground water.
A first render accidentally read as islands because the depression lacked its
dry floor. Added the concave shore bank; lowered atmospheric density0.0015→0.00035
to inspect geometry instead of washing it out. Cameras changed during exploration;
there is no fixed-camera quantitative improvement series.

## Walls and technical boundaries

Profiles are authored irregular polygons with an intermediate shoulder per edge,
not circular columns. Four elevation sections change wall radius/offset and retain
flat caps. Stone and turf use separate visual surfaces; turf is offset0.6m to
avoid depth conflicts at this scene scale. This is presentation, not turf physics.
The dominant source uses existing connected shaping, not a new fracture generator.
Regions may overlap; individual closure is NOT a Boolean-union or seamless-world
certificate. No new native source format/runtime/world-streaming feature is implied.

## Review of actual captures

Overview/near/side opened after publication. The blockout communicates multiple
receding shelves, a central depression, the dominant wall and background ridges.
It still differs substantially: geometry too sparse, slopes/caps too regular,
main plate not matched, walls lack coherent rock plates, bank/plaza too flat,
background peaks too pointed and no ruins/vegetation. It is NOT an identical or
artistically accepted reproduction. Full-frame RGB scores would also mix omitted
context, unmatched camera/light and material differences; none is claimed here.

Static performance before/after:12 bounded cold landform sources, one4753-vertex
background patch and one289-vertex visual water patch, immutable meshes; no new
passes, workers, dependencies, binaries or frame-time procedural generation.
Every native landform has0 boundary/nonmanifold edges. Opt-in mesh collision;
no new KCC/navigation test. Save/load reproduces overview PNG byte-for-byte.
Native verification.json records actual build/counts. Python compile/diff checked;
unchanged Rust tools do not require a new broad suite or engine build. No FPS claim.
Parallel LoS files preserved. Local commit only, no push. EAF-221 remains InProgress
for artistic/reference fidelity, remaining context and existing world integration.
