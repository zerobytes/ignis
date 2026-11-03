# Proportional facial controls

Human review found pinching around facial controls: several new nasal fields
covered only 7–11 cage vertices, and hard masks left immediate neighbors at zero
influence. The new immutable `human-soft.ignis-character` extends all 93 skin
controls along the connected polygon cage. The two rigid eye controls retain
their existing transport. All 95 landmarks and 392 prior coefficients remain.

Fields use surface graph distance, a small central plateau, quintic falloff and
three relaxation steps on the displacement weights. They do not smooth the
character's underlying geometry. Authored radii are calibrated against cage
spacing: the immediate ring follows at least 85% of an isolated small edit,
the subdivision stencil is calibrated for a one-millimetre isolated edit, and
the largest weight jump
across an edge improves for every field. Surface connectivity avoids shortcuts
across an open mouth or eye. Adjacent features can now follow partially; the old
hard-pinned independence assertions do not describe this mode.

Widening fields alone passed isolated tests but failed the native visual review
of the user's combined extreme configuration: overlapping edits accumulated too
much displacement. Source 10 therefore also introduces generic `field_blends`,
with explicitly identified XYZ targets, their channels and a 0..1 strength.
For each vertex, the compiler sums these displacements and divides by the larger
of one and the accumulated active influence. Each field's continuous activation
is the largest absolute channel value divided by its channel limit. Zero-valued
controls exert no constraint. A small isolated drag keeps its calibrated
response; overlapping large drags are bounded rather than adding unchecked.
This is rest-shape authoring, not a collision or animation corrective solver.

`Influência suave dos pontos` controls the new `face_softness` coefficient,
default 1. It interpolates the original fields and the new proportional blend.
At zero the parent source 9 bake is exactly recovered, including the user's
existing values. The setting is shared by the editor and
`SetCharacterParameters`; handles still use `SetCharacterHandle` and the same
read-only preview/commit boundary. Old sources retain their prior evaluation.

Rebuild with `python3 scripts/character/soft_source.py`. The source uses existing
geometry, topology, joints, weights, eye metadata and materials; it adds one
coefficient (393 total) and 279 targets (832 total). It preserves the existing
512-channel, 1024-target, 96 MiB and combined sparse-work budgets. Only the
character source format advances to 10; Recipe 2, Scene, replay, content,
protocol and portable-save envelopes are unchanged. Sources 1–9 remain readable.

`python3 scripts/character/soft_demo.py --fields-only` checks all 93 fields,
all XYZ axes, connected support, edge transitions and calibrated response.
The native variant uses a separate MCP process and the latest saved user scene
in `artifacts/character-soft/user-before.scene.json`. It checks preservation,
undo/redo, the exact strength-zero fallback, spherical eyes, isolated edits,
preview/error atomicity and Scene/configuration/portable reopening. Matched
front and oblique captures must be reviewed in addition to numeric checks;
the rejected first attempt demonstrated why isolated smoothness is insufficient.

Large opposing edits can still fold or intersect a surface. This increment does
not promise injective deformation, automatic repair of every authored shape,
animated eyelid contact or human artistic acceptance. EAF-187 remains open.

The generated source SHA256 is
`04cd4651a43b900a565d1db2606dcfc1f10f8eff60fcad6cbbb4831f9bfca43e`,
76,677,871 bytes; parent source9 SHA256 is
`696e05632888f602f3f48c4994381ded145a3da3d70aa96598f342c196109034`.

Validation: all 15 critical character integration tests passed, including the
new overlap/activation and malformed-metadata regression. The field-only proof
passed all 93 controls. Native MCP build `677e88e92de00145` passed four groups:
392-value/material preservation and exact parent fallback; Scene/configuration/
portable pixel equality; spherical eyes; eight 3 mm edits with smaller adjacent
displacement jumps, plus preview/error atomicity. Front/oblique before/after
images were reviewed after adding overlap normalization.

The editor was reopened on the latest saved human scene in PID 12158,
build `9e0b4b2f9a3a5420`, with all 392 prior values retained and the new slider
at 1.00. CUA verified the slider and framed face; native binding/bake equals the
MCP proof exactly. Receipt, source report, native snapshot and proof are in
`artifacts/character-soft/`. The previous PID 99095 had already exited before
the handoff; no live edits were reset. A blank process auto-opened by CUA was
closed, then the saved scene was loaded explicitly.

Own formatting/Python/diff checks passed. Strict Clippy reached an unrelated
`src/editor_particle_fire_ui.rs:83` unnecessary-unwrap lint; no whole-checkout
Clippy success is claimed. Earlier gate attempts also hit transient parallel
renderer compilation errors; those reserved files were left to their owner.

The final release build also passed after the shared renderer stabilized. The
open editor intentionally keeps its already verified process/build; it is not
restarted by subsequent shared executable replacements.

The freshly compiled library test binary also passed the exact native facial
gesture regression (`editor_ui::character_gizmo::tests::native_face_preview_cancel_hidpi_and_release_commit_are_atomic`):
1 test, including preview, cancellation, HiDPI and release/undo. An initial
Cargo name filter matched zero tests; the exact test was then run explicitly.
