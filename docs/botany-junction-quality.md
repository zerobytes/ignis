# Local botanical junction resolution

EAF-227 is the engine task; LOS-21 supplies the ancestral-tree use case. This
increment follows the handoff from the game chat, which now owns gameplay. It
reuses the existing wood-surface and stature work rather than replacing it.
The preserved `games/land-of-survivors/ancient_tree.py`, its source/configuration,
and terrain work are independent inputs and are not modified or installed.

## What this increment adds

Recipe11 adds optional `OrganEdit.junction_refinement` (0–3). It controls the
interior subdivision of junctions incident to a selected woody organ, through
the same `EditOrgan`, `Preview`, `Publish`, undo and persistence operations used
by other native controls. In Vegetação, select an organ and expand “Resolução
local das junções”. Preview, apply and cancel use the native compiler. Numerical
and procedural authors write the identical field; no generator script owns a
separate algorithm.

Overrides on incident organs combine by maximum. Without overrides, side/terminal
junctions keep level1 and basal multicollars keep level2, exactly as before.
The boundary sample counts and body tessellation settings are unchanged. The
existing admissibility search still runs: a different refinement can reject an
earlier candidate and select different collar positions. This increment does
not lock collar placement; that coupling remains a structural limitation. Interior subdivision
increases work locally; it does not multiply all leaf and branch samples. The
existing organic level limit3, maximum160 boundary samples for multicollars and
final botanical mesh budgets remain in force. Preview compiles the candidate and
reports actual counts; no FPS claim follows from those counts.

Invalid levels, old recipe versions carrying the field, and edits on nonwoody
organs are rejected. EditOrgan explicitly upgrades the recipe to11. Existing
sculpt preservation rules still apply: a topology change is not permission to
discard unmatched manual edits. Absent fields omit serialization, preserving
older source/recipe defaults. The global Scene/replay layouts are unchanged.

## Diagnosis and next structural work

The complete neutral ancestral experiment shows broad angular junction patches
and interrupted cord fields even with directional shadows removed. The scaffold
also branches first into two large axes; this is an authorship decision, not a
universal two-branch limit. Lateral grafts already permit additional topology,
while terminal forks currently require two continuations.

More interior triangles provide an authorable resolution choice; they do not
fix collar placement/size, tangent continuity, field transport, or the branching
layout. Those remain separate required increments. In particular, the next
junction design must preserve boundary positions, target the adjoining surface
tangents, and transport local cord/valley fields through the interior instead
of merely interpolating boundary displacements. A many-arm central mass must be
validated against real collar-space/overlap constraints, rather than stretching
the trunk to fit disjoint patches.

The initial reference proportions from the handoff (crown5–6D, approximately7
visible primary exits and10 visible roots) are adjustable projected estimates.
They do not certify the hidden architecture. No final elven tree, foliage or bark
is installed by this increment; all30 authoring requirements remain individually
subject to technical and artistic acceptance.

## Verification

`tests/wood_authoring.rs` checks that explicit level1 reproduces the existing
terminal fixture exactly, higher local refinement preserves tube vertex
addresses/positions in that fixture, the complete result stays closed/oriented with positive
volume, and roundtrip, preview/cancel, undo/redo and rejected-edit atomicity work.

Run `python3 scripts/verify_botany_junction_quality.py` after the existing
ancestral experiment is available. It reads that complete saved scene, compares
local level1/2 at the trunk under matching neutral light, retains applied
materials, checks full-tree edge incidence, previews/transactions and exact PNG
save/load. Output is `artifacts/botany-junction-quality/review.html` and
`verification.json`, separate from the game experiment. This is visual evidence
for review, not artistic approval or a claim of sufficient tangent continuity.
