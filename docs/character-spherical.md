# Spherical eyeballs and eyelid contact

Human review rejected source 8's concave scleral contact skirt. Projection
coverage and save/reopen checks had passed, but those checks did not enforce
the shape of an eyeball. Source 9 makes spherical geometry an explicit invariant:
all scleral vertices share one centre/radius and every triangle winds outward.
The sclera is never stretched to follow the eyelid opening.

The source-owned inner-rim loop and classified socket walls guide the fitting.
In the rigid socket frame, the compiler fits a bounded radius and centre, wraps
the eyelid rim around the front hemisphere, keeps external skin outside the
sphere, and keeps socket walls behind it. Corrections fade in a finite eyelid
region. Depth still moves the globe and neighbouring tissue; the rigid eye
pivot follows its fitted centre in every part's rig. Strength 0 returns the
legacy spherical eye, while intermediate strengths also remain spherical.
The spherical iris limbus, recessed iris drawing and corneal dome keep their
existing controls/materials. This is rest authoring, not an animated eyelid or
blink solver.

`python3 scripts/character/spherical_source.py` builds the new immutable
`human-spherical.ignis-character` from source 8. All 392 coefficients, 553 targets,
95 handles, original positions, topology, weights, joints and part identities
are unchanged. Each eye adds 1,481 classified socket vertices within its existing
2,433-vertex lid region and 56-vertex closed contact loop. Source 9 retains all
source 8 capacity/byte limits; Recipe 2 and global persistence/network formats
do not change. Sources 1..8 retain their prior evaluation, including the rejected
source 8 shape for exact undo/history. New default authoring uses source 9.

Source SHA256 is
`696e05632888f602f3f48c4994381ded145a3da3d70aa96598f342c196109034`,
73,612,947bytes. Its parent source 8 hash is
`6af29164bd30c2c23c90eab72270e5c52e55ce6ec847fc3d26150f5b132dc768`.

`spherical_demo.py` validates the actual saved human configuration in a separate
native MCP process. On build `9a78e1f299aba38d`, the former source 8 mesh departed
from its best-fit radius by 10.64 mm; source 9's maximum deviation was 0.000091 mm
on either eye, with outward winding on every triangle. Fitted radius was 12.077 mm,
and the drawn lid-rim separation stayed below 0.047 mm. The proof retains all 392
coefficients and the exact iris material, checks source 8 undo/redo, unchanged
skin outside the lid regions, spherical depth/strength/head/neck variants,
atomic error rejection and configuration/Scene/portable reopening with exact
native pixels. Numeric contact does not replace artistic review.

Native renders include the whole face, oblique eye close-up, face profile and
isolated globe from three directions in `artifacts/character-sphere/`. The
isolated profile is required visual evidence: a frontal crop can hide incorrect
curvature. The synthetic source 9 regression also checks constant radius,
positive winding, rim contact, external clearance, legacy equivalence and
rigid posture transport. No extra vendor data, vertices, triangles or bones
were introduced.

All 14 critical character Rust tests and release build passed. Owned Rust
formatting, Python compilation and diff checks passed. The strict
lib/bin/character-test Clippy attempt stopped on a parallel renderer prototype's
`src/mesh_proxy.rs:294` (`type_complexity`); that reserved file was not modified
by this correction. This does not certify a clean whole-checkout Clippy run.

The saved native editor was replaced with source 9 in PID 99095, build
`d8dec4c557b67a8c`. The launcher verified all 392 authored values against the
latest pre-upgrade scene, saved the replacement, and captured the native editor
in `artifacts/character-face/editor.png`. Receipt and equality evidence are in
`artifacts/character-sphere/native-verification.json`. Later unsaved interactive
changes were left intact; the acceptance snapshot is the saved launch state.
Human artistic acceptance remains pending under EAF-187.
