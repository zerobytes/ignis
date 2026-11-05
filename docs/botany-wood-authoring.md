# Woody volume authoring — first increment

This is the first implemented tool increment from
[the 30 authoring requirements](botany-authoring-requirements.md), tracked as
EAF-227 and motivated by LOS-21. It is not the completed ancestral-tree toolset
or an accepted elven-tree asset. The living-age slider is a separate task.

## Use

```sh
python3 scripts/demo_wood_authoring.py
python3 scripts/open_wood_authoring.py
```

These commands replace stable review files under `artifacts/wood-authoring`.
The second explicitly opens `--editor`, not the plain playable window. It selects
Vegetation, `woodlab`, organ `trunk`. Scroll to **Volume de madeira · seções /
cordões / vales**. Load either `assets/botany/wood-authoring.ignis-botany` (trunk
and arms) or `wood-authoring-with-roots.ignis-botany` (complete neutral specimen)
through the existing source loader. Neither is installed in the game's scene.

The first source has a nominal 2.5 m trunk radius. Its authored sections keep
about 4–5 m diameter through the upper trunk; individual polar directions vary.
The root crown adds its existing vertical neck/lift, so the source trunk path
height and the compiled shoulder height differ. Do not report the basal spread
as trunk diameter. Exact bounds and vertex/triangle counts are recorded by the
demo. The present fixture contains one trunk, two arms and four rounded roots,
with no leaves, bark pattern or texture.

## Persistent source and shared operations

`Recipe10.edits[organ].wood_surface` is optional. Source9 and earlier botanical
source graphs are unchanged. Recipes1–9 cannot contain the new option; omitted
options preserve the previous evaluation. Existing Scene121/replay131 admit the
versioned recipe, with no new public protocol/save-envelope format. Older recipe
readers reject Recipe10 instead of silently dropping shape data.

Manual controls and procedural clients write exactly the same `OrganEdit` and
use existing `Botany.EditOrgan`, `Preview`, `Publish`, `CancelPreview`, Save/Load,
Undo/Redo operations. Applying an organ edit with a surface promotes its recipe
to10. The panel exposes numerical parameters and draggable polar-contour handles.
The plot scale is held fixed while dragging, avoiding feedback from a moving
scale. Organ controls resynchronize after publication/undo/load revisions;
cancelling a surface preview resets the draft from the committed recipe.

Section and relief IDs are stable, unique within the organ, serialized ASCII
identifiers. Sections are explicitly ordered by longitudinal position; the panel
can insert, remove and sort them. Relief layers can be reordered, disabled and
removed. There is no runtime name/species-specific branch in the evaluator.

## Mathematical domain

The shape is a sweep in the existing transported axis frame. A section has a
normalized axis-sample coordinate `u`, angular orientation, two local centre
offsets in metres and 8–32 periodic radial samples in metres. Smoothstep
interpolation connects adjacent angular samples and longitudinal sections.
The contour is star-shaped about its local centre: it permits asymmetry and
concavity, but cannot represent a tunnel or arbitrary cross-section topology.

A relief layer has a piecewise smooth path `(u, unwrapped angle)`, signed height,
width in metres and an end fade in normalized coordinates. Positive height adds
a cord/bulge; negative height carves a geometric valley. Its transverse profile
is compact: `(max(0, 1 - (d / width)^2))^2`, where `d` is the chord-distance
estimate on the unmodified section. Layers add in declared order; overlapping
layers currently commute, with no subtractive CSG or arbitrary blend operators.
At a given `u`, layers outside their path interval contribute zero.

Metres are source-space units. Whole-plant scale, developmental radial scale
and the organ radius multiplier scale both contour and relief together. Keeping
bark microdetail at fixed physical size is a later material-tool requirement,
not implemented by this geometric field. Offsets and contours follow existing
axis bending/twist frames; they are not an independent world-space displacement.

The same evaluator supplies body rings, cap centres, area/bound estimates and
junction collar targets. Existing bounded junction displacement extension
transports shaped collars into the continuous mesh. It guarantees neither a
universal C1 blend nor constant-thickness solids. The neutral views expose
remaining faceting and collar/tangent transitions for review.

## Resolution, integrity and limits

Resolution is local to the shaped organ: the manual panel retains 12–32 angular
samples and8–128 longitudinal samples. The shared surface accepts up to96 sides
for the optional living sequoia stem volume profile, which uses separate sheath
connections. This does not raise the organic graft collar kernel's quota. A section edit does
not force all other organs to use that resolution. Body preflight includes the
local sample count; the existing final3Mvertex/6Mtriangle budgets still apply.
The panel's estimate is the body tube only, explicitly before junctions.

Each organ allows2–32 sections and0–32 relief layers, with2–32 path controls per
layer. Section parameters must span0..1 in strict order, separated by at least
0.005. Contours cannot contain more radial samples than the organ tessellation.
Duplicate IDs, invalid lengths, nonfinite values and incompatible organ
kinds are refused. Source contour radii are at least0.02m and relief width at least0.02m;
there are no fixed artistic upper bounds on radii, offsets, width or signed
relief height. Finite arithmetic and actual mesh-coordinate admission remain
required. See [width and age-driven stature](botany-stature.md). A dense2× resolution diagnostic rejects sampled radial
collapse below0.02m. It is **not** a continuous interval proof or a global
self-intersection detector. Existing junction guards reject local inversions;
sharp bends, offset sections and remote surfaces can still intersect. No new
claim of global collision detection or minimum solid thickness is made.

The existing material-address transport and b447ccd material-preservation path
remain in use. Compatible shape edits retain manual offsets and materials.
Changing local resolution can remove an edited address and must be explicitly
resolved through the existing preservation/discard policy. The new tool does
not silently discard sculpted geometry to make an invalid edit succeed.

## Evidence and remaining scope

Native captures: front, side, back, top, trunk/base/fork details, and editor UI in
`artifacts/wood-authoring/review.html`; source/bake timings, build fingerprint,
bounds and counts in `verification.json`. Camera/light/resolution are saved in
the scene and script; screenshot save/load comparison must remain byte-identical.
The script uses native commands for compilation and roots; Python is an authoring
client, not a second triangulation kernel.

Tests exercise closed oriented edge incidence, positive volume/nondegenerate
faces, source roundtrip, relief activation, version rejection, failed edit
atomicity, preview/cancel, undo/redo and rounded-root crown integration. The
legacy20-test botanical suite includes material preservation, sculpt transport,
full exemplars and root/buttress compatibility. These establish technical
properties for the checked fixtures, not aesthetic acceptance or runtime FPS.

Coverage is intentionally partial: GEO-01–06 and VAL/AUT now have an initial
section/relief/resolution implementation on top of existing curves/junctions.
Rounded roots reuse the existing root system. Still open: truly local multiscale
seeded geometry layers (GEO-07), authored swelling at junctions, improved tangent
transitions, local nodes/scars/open recesses, shared fields across root/trunk/arm,
per-root exposure/soil editing improvements, full spatial guide tools and mask
visualization, all surface-oriented bark/fibre/plate/fissure/painter/weathering
requirements, and artistic review of the mass/base/arms. The complete first-stage
acceptance from the requirements document is **not** declared finished.
