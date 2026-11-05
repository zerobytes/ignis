# Five-level plateau composition (EAF-221)

User approved patchy-plateau as first good visual direction, then requested dozens
in 4 or 5 layers. Preserve that source unchanged as visual baseline.
Construction inventory: C1 forty plateaus/eight per level; C2 five elevation
bands; C3 winding central ravine and irregular spacing; C4 small rotation/size
variation preserving dominant geological direction; C5 high plateaus supported
to ground, not floating; C6 paired turf/rock editable objects; C7 overview/near/
reverse captures; C8 count/geometry/save-load/replay evidence. This is a composed
landscape using approved geometry, not forty unique procedural formations.

Static performance pre-review: reuse five rock assets (one per support depth),
one turf asset and floor. Native Extrude extends the basal cap on high variants; upper approved walls
are unchanged. The duplicated rocky roof is removed in favor of the shared turf
cap at exactly the same coordinates, avoiding two near-coplanar top surfaces. Instancing shares
geometry, bounded 81 instances/40 object groups and collision opt-in. No new
per-frame generators, renderer passes, dependencies or builds. Physics can share
shapes at equal scale; varying scales may require separate collider shapes.
No FPS/performance guarantee inferred from triangle counts.

## Result, evidence and boundaries

40 plateau objects, eight per band, five nominal roof heights above ground:
34/52/72/94/118m before uniform scale variation 0.88..1.12. Bends/stagger/jitter
break a strict row grid; small Y rotations preserve roughly related orientation.
Basal extensions18/38/60/84m reach the inherited floorY-1.5 exactly after each
instance transform. Existing approved upper-wall triangles retain their exact
positions; native extrusion adds new lower walls instead of stretching them.

The initial lower-rim translation was refused for triangle reversal. Used native
basal-cap extrusion instead; no guard change. The old rock/turf coplanar overlay
showed distant depth interference even with a larger offset. Final variants
remove the rocky cap and use the turf cap at identical coordinates, not an
increased floating offset. Read-only coordinate-edge counts confirm incidence2
for each rock+turf pair. Separate assets are not asserted individually closed
once their duplicate roof has been removed; physical/render caps share matching
transforms and both have mesh collision enabled. No KCC playtest claimed.

Five rock assets: level0 1228v/2346tri, each elevated variant1334v/2558tri.
One shared turf110tri and floor128tri;81instances/40object groups/81collision
instances total. Render source counts105152tri across instances are bookkeeping,
not GPU draw timing or FPS. Objects remain separate; intersections are not a
worldwide Boolean union. Source scene SHA256 preserved, no forty unique geology
meshes claimed. Lower stems are simpler than approved upper walls.

Existing release de6b0252e7e4d8ca used, no engine rebuild/new runtime features.
Actual MCP constructed/inspected/saved/loaded/captured the scene. Roundtrip PNG
byte equality and replay/mesh agreement pass; deterministic placement repeat,
upper-wall coordinate preservation, support height, counts and Python compile/
diff checks pass. Overview/valley/reverse opened. Repeated silhouette and simple
lower stems remain visible; scene artistic acceptance is pending independently
of the user's approved individual plateau.

Directional shading and mild atmosphere used. Cast shadows omitted: current
native shadow extent caps at256m, insufficient for full composition coverage;
not replaced with a misleading partly shadowed panorama. No renderer change or
new shadow capability delivered here. No vegetation/settlements/water/streaming,
planet runtime integration or FPS claim. No foreground window opened.

Run python3 scripts/patchy_plateau.py first if the approved base artifact is absent,
then python3 scripts/render_plateau_amphitheatre.py. Composition controls/source
hash/cameras: examples/plateau-amphitheatre.recipe.json. Current scene/captures/
verification: artifacts/plateau-amphitheatre/. Parallel LoS work/source scene
preserved. Local commit only; EAF-221 remains In Progress for broader requirements.

## Scope correction after user review

The user explicitly clarified that seed must generate each plateau's geometry,
not just place this repeated shape. This document describes the historical
instancing experiment only. Current seeded authoring and distinct native bakes:
[seeded-plateaus.md](seeded-plateaus.md). Do not call this forty-instance recipe
a completed procedural plateau generator.
