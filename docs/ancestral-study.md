# Ancestral-tree authoring study

The current revision addresses the user's feedback about pasted branches,
straight-looking transitions, missing large cords/sockets and pointed leaves.
It remains a review study for EAF-227/LOS-21, not an accepted ancestral-tree
asset or a completed botanical-plausibility system.

## Current construction

`examples/ancestral-study.recipe.json` is the editable seeded authoring input;
`graph()` in `scripts/regenerate_ancestral_study.py` emits the native source.
Rust owns meshing, joints, shaped wood, root morphology, sockets, materials and
rendering. The concept image is never applied to the mesh or bark.

The major scaffold now consists of connected forks. The four auxiliary arms
that overlapped the trunk were removed. Eight attraction clouds,48 sites each,
feed recursive branch partitions;768 terminal shoots carry30,720 individually
closed rounded white leaves. The source's gentler primary axes avoid tight
zigzags inside very thick wood. The nominal trunk radius3.7m is an artistic
choice, not an upper bound imposed on the engine.

Recipe13 native junction flow is enabled on the central three junctions, with
local refinement3. It uses actual swept transition cuffs, implicit volume
fitting and interpolated bark coordinates. Distal branches retain the existing
connected shaped-junction constructor. Large cords and valleys have seeded
irregular spacing/width/height; the material follows their angular drift.
Three historical inward sockets use the existing editable wood-history tools.
The rounded-root system remains the existing native basal construction.
See [junction flow](botany-junction-flow.md) for the mathematical scope, UI/API,
compatibility, local fold guard and resource costs.

Leaf tip exponent0.5 and eight longitudinal samples replace the earlier
pointed contour. Recipe13 redistributes those samples near the endpoints;
no extra leaf vertices are created by the redistribution itself. Increasing the
study's explicit sample budget from4 to8 does increase this full-resolution
specimen's cost.

The source age180years is an authorial mature snapshot. It is not empirical
species calibration, historical simulation or a new persistent living grammar.

## Review files and native editing

Stable local outputs in `artifacts/ancestral-regeneration` include:

- `review.html`: concept reference, six paired neutral/full native views and
  closeups of bark, a socket and the rounded leaves.
- `hero.png`: native presentation,2048x1365.
- `ancestral-study.ignis-botany`: complete editable native source.
- `structure.ignis-botany`, `full.ignis-botany`: neutral inspection sources.
- `bark.recipe.json`, `bark.ignis-material`: procedural recipe/native material.
- `current.scene.json`: complete scene with embedded geometry/material.
- `organic-study.json`: current build, native compiler counts, settings and
  direct observation notes; this is not an automated test report.

Open the saved scene in the native editor with:

```sh
python3 scripts/regenerate_ancestral_study.py --open
```

Vegetation exposes the source, organs, shape fields, junction flow and history.
Load/save use the existing native tools. The Python recipe is a reusable
numerical authoring input; the delivered native source is editable without it.
The older generator entry retains its pre-existing verification code. That code
was neither expanded nor executed as a regression battery during this increment,
in accordance with the user's zero-new-tests instruction. Files named
`*-verification.json` from the previous study are historical, not evidence for
these revised images.

The directly inspected full specimen has32,273organs,2,628,558vertices and
5,134,232triangles; its native source is21,741,480bytes. It is a full-resolution
review asset; population and LOD budgets are not tuned, and no FPS is claimed.

## Observations and remaining work

The revised scaffold no longer has the four pasted auxiliary branch ends, and
rounded leaf tips and a recessed branch socket are visible in the native closeups.
There are still visible collar bands and broad/artificial regions. The central
mass and roots remain much simpler than the reference's deeply braided wood;
material-coordinate interpolation is not a global vascular-fibre solution.
Bump material does not replace geometric silhouette cords, tunnels or deep
structural recesses. The existing socket model makes blind radial cavities,
not arbitrary through-holes or internal rot networks.

Native face/sweep admission protects local geometry; it does not prove global
nonintersection or guarantee that every artistic configuration can exist in
nature. Additional central-volume/collar design and artistic review remain
necessary. Compilation and direct native inspection are distinct from acceptance.
No new automated tests, cases, verification harnesses or CI changes were made.

The game scene, other tree experiments and independent catalog are preserved;
this study is not installed into gameplay or published. One existing release
output is updated in place; no additional executable/build history is created.
