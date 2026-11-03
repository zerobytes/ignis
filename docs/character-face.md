# Local facial authoring and ocular contact

Source8 adds fifteen XYZ landmarks to the mandibular/ocular base. Existing
sources1..7 are immutable; all343 previously authored coefficients are retained.
The new source is `artifacts/character-base/human-face.ignis-character`, built
offline by `python3 scripts/character/face_source.py` after the
[mandibular pipeline](character-jaw.md). No new vendor data is downloaded.

| Region | Landmarks | Editing |
|---|---|---|
| Submental tissue | Central and left/right lateral papada | Raise, lower, deepen or reduce the underside of the chin |
| Lateral orbital/malar tissue | Two paired levels | Flatten or build the region beside the eye independently of the broader cheekbone field |
| Nasal width | Left/right upper, middle and tip | Use X with symmetry to narrow or widen each height; Y/Z remain available |
| Nostril roof | Left/right | Lower Y to reduce the opening height without translating the whole nasal ala |

These source-owned finite fields use the original cage and normalize their
response at the drawn Catmull-Clark anchor. Lip/ear anchor stencils, nasal
midline anchors and ocular contact regions are excluded as appropriate. Fields
retain bounded stylized ranges; arbitrary combinations are not certified free
of folds or self-intersection. **Papada**, **Olhos e maçãs** and **Nariz** are
native point groups; generic XYZ arrows, optional symmetry, preview, undo and
the ordinary character MCP commands use the same configuration.

## Eyes

**Olhos · globo e encaixe** exposes linked or independent eye controls:

- `eye_L/R_globe_depth`: −3..+8mm in this dataset, zero by default. Positive values recess the
  globe; a finite authored eyelid band follows the depth change.
- `eye_L/R_socket_fit`:0..1, default1. Zero recovers the previous spherical
  sclera. With depth and all new XYZ coefficients zero, disabling both fits
  reproduces the complete source7 bake exactly.

The default corrective preserves the authored skin geometry. It centres the
optical geometry in the aperture's local XY bounds, preserves its prior depth
and spherical iris limbus, and extends the sclera behind the inner eyelid rim.
The scleral contact skirt and hidden rear cap close the lateral void without
projecting the face onto a larger eyeball. Iris pigment, pupil settings and
corneal materials remain unchanged. Each eye pivot follows its new optical
centre in every part's fitted rig. Source-local fitting transports through
head/neck posture instead of assuming world Z.

The source explicitly owns the closed inner-rim edge loop, rigid depth weights
inside the socket and finite fading weights on the external lid band.
Validation checks indices against the actual refined part,
adjacency/closure, distinct ordered weights, disjoint eye regions, leaf socket
ownership and source-owned control bounds within the supported envelope. A malformed part is validated before its
refinement plan is used by contact validation. Apertures must surround their
optical centre and remain within the bounded40mm fitting domain. This is a
rest-shape corrective; animated eye rotations/blinks, tear tissue, spatial
refraction and expression correctives remain separate work.

## Capacity and validation

Source8 explicitly admits128handles/1024targets. Sources5..7 retain80/512;
sources2..4 retain64handles, and source1 has no handles. The existing512parameter
ceiling for sources6+ and96MiB source envelope remain unchanged. Recipe2,
Scene/replay/content/protocol and portable-save envelopes do not change. The
current dataset has392parameters,553targets and95handles, including49new
coefficients. Geometry/triangle/joint counts are unchanged.

`python3 scripts/character/face_demo.py` uses a separate native MCP process and
the saved human snapshot in `artifacts/character-face/user-before.scene.json`.
It checks zero/legacy equivalence, coefficient preservation, transactional
upgrade/undo/preview, fifteen local fields, independent nasal width levels,
ninety XYZ endpoints, both ocular depth endpoints, projected scleral coverage
at the rim, independent eyes and configuration/Scene/portable reopening.
The gallery and receipt are in `artifacts/character-face/`. Pixel equality and
projected coverage are technical checks; visual inspection remains necessary
to reject penetration or unwanted tissue deformation, and human artistic
acceptance remains open under EAF-187.

The final source contains73,595,672bytes, SHA256
`6af29164bd30c2c23c90eab72270e5c52e55ce6ec847fc3d26150f5b132dc768`.
Four MCP groups passed on build`ab91d50c491f1859`; local trials retained the
existing lip/ear/nasal midline pins, lateral orbital edits retained the old lid
and corner pins, and nostril-roof edits retained the nasal ala/base pins. All
three nasal levels were independently editable. The body positions remained
exact at default contact, and Scene/portable reopening produced exact pixels.
These proofs concern the saved authored snapshot, rather than every possible
combination of facial coefficients.

The human window was replaced only after saving its latest scene and layout.
Its successorPID87477 uses the same working file
`artifacts/character-eyes/editor.scene.json`, source8,392values and95handles;
all343values from the immediately preceding save were checked unchanged.
CUA confirmed the new native groups and corrected default eye appearance at
the preserved oblique view. `open_face.py` reuses that scene and checks for an
existing editor before opening another; the earlier ocular launcher delegates
to it once the scene uses the current source. Receipts are in
`artifacts/character-face/{launch,native-verification}.json`.

All13 critical `character_base` Rust tests passed, including source8 bounds,
malformed metadata, contact transport and legacy equivalence. Release build
and owned formatting/Python/diff checks passed. The strict
lib/bin/character-test Clippy attempt stopped in the parallel optional field
renderer at `src/surface_field.rs:605` (`collapsible_if`); earlier attempts also
caught that prototype's transient compilation errors. Those reserved files
were not changed by this unit. This is not a clean whole-checkout Clippy claim.
