# Irregular roots and persistent woody history

EAF-227 (shape authoring) and EAF-226 (living time) share this increment. The
user rejected bent circular tubes and requested roots with local distortions,
swellings and texture, plus the visible history of lost branches and pruning.
This extends the same native recipe/compiler/editor; it does not install the
ancestral tree in Land of Survivors or finish its 30 authoring requirements.

## Canonical data and commands

Recipe12 adds optional `wood_details`. Absent data keeps the legacy evaluation.
The object owns a seed, static snapshot age in years, authorial healing duration,
optional root morphology, optional natural-loss schedule, and up to64 stable
scar records. Source curves/organs remain distinct from the derived mesh.

`Botany.SetWoodDetails` uses the existing preview/publish/manual-preservation
transaction. `Botany.LoseBranch` records a pruning or breakage event, hides the
branch and its descendants, and preserves the parent mark. A primary living
production site is removed semantically too, preventing its future production.
The normal living Prune action records scars when wood history is enabled.
Edit/load/undo/redo/save and the editor use this same data and compiler.

In Vegetação, open “Raízes irregulares · nós e histórico”. Root controls expose
bulge count, swelling, variable flattening, axis bending, twist and shorter
wood reliefs. Each node exposes its visibility, dimensions, orientation and
healing duration. Manually authored nodes also expose parent, distance and date.
Loss buttons accept the woody branch's native organ ID. Preview/apply/cancel
are explicit. Hiding a node preserves the lost branch and its history; disabling
or dropping a loss record through SetWoodDetails is refused, with undo offered
as the way to reverse that intervention. Publishing/replacing a whole authored
recipe remains an explicit source-authoring operation, not an immutable audit log.

## Roots

Each rounded root receives seeded, localized swellings with independently
varying centres, angular positions, lengths and strengths, plus changing ovality,
twist and nonperiodic-start/end short cords. The rest curve bends before child
attachment, with fixed endpoints and transported child frames. Cross-section twist
does not roll the child attachment frames or turn descending roots upwards. Two-point axes
are resampled so they can also bend. The same physical source-radius scaling and
manual edits still apply. The distortion is not unmasked white noise.

This profile uses the shared woody Surface evaluator with32 section stations and
at least96 longitudinal samples, retaining the local angular sample count so
multicollar budgets are respected. It does not raise the160 boundary-sample
junction budget or force the crown to use the root's longitudinal resolution.
Buttress-wall roots are refused for this rounded-root profile; they retain their
separate existing authoring model. Thin sections retain the existing2cm surface
floor. No mechanical soil/contact/fracture model is introduced.

## Scars, knots and grain

A record stores stable ID, parent, optional lost branch, cause, event year,
source distance along the parent in metres, angle, mark radius, rim height,
initial depression and closure duration. Distance is metric, not a normalized
fraction of an increasingly long trunk: the mark does not climb the tree merely
because the trunk length grows. A historical parent/branch may cease to exist;
the record remains, and evaluation on missing wood is inactive. Captured subtree
IDs retain archived edits when semantic production removes those organs.

The mark is an irregular elliptical radial patch with a raised annulus and a
central depression. `1−exp(−elapsed/closing_years)` drives a transient larger rim,
shrinking depression and persistent residual knot. The shape deflects nearby
geometric cords and the material coordinate field around the mark. Source-frame
cylindrical material coordinates follow each axis and use a periodic sin/cos
embedding around its perimeter, avoiding an artificial longitudinal UV jump.
Existing materials remain independently editable and are retained on rebuilds.
This is an authored shape/evolution rule, not an empirically calibrated botanical
healing law, biological compartmentalization or a constitutive tissue model.

Extra longitudinal rings are placed near active marks, with IDs derived from the
scar and local ring index. Hidden marks keep their rings and addresses, preserving compatible sculpt data.
This bounds the added work to the affected woody organ.
Uncollared/sheath-only scar supports can use96 angular samples; graft/fork/basal
supports keep the existing collar-compatible count. Budget, finite-coordinate,
radial-collapse and sculpt-preservation checks still apply. Subpixel or very small
marks remain subject to the geometric/raster resolution and2cm surface domain.

The representation stays radially star shaped: side recesses are geometric,
but arbitrary tunnels, fracture surfaces and through-holes are not implemented.
Marks hidden inside a replaced junction interior still require the separate
field-transport/tangency work documented in `botany-junction-quality.md`; this
increment does not solve those structural transitions by adding triangles.

## Chronological losses and previews

Living evaluation uses canonical game time (365 days/year), rather than the
static snapshot clock. The optional seeded schedule processes bounded future
loss dates in chronological order, selecting existing primary branch sites and
recording their size and position at that time. Large and small time steps must
produce the same history under unchanged configuration. It is a deterministic
authorial event schedule, not a probability model of wind, disease or branch load.

Enable the schedule while the individual is young, then advance its age. It does
not invent past events when enabled on an already adult saved individual. It also
records known future maturity losses in the existing sequoia grammars, before their
branches disappear. Those inherited maturity rules are authorial too. Static
historical nodes are explicitly authored rather than presented as recovered data.
Forward age previews use the same chronological advance as confirmation; backward
previews reconstruct growth and apply saved marks/losses after their event dates.
Canonical time/history remains unchanged. Arbitrary harvest and sculpt history
is still outside the backward preview replay contract.
Detached rigid bodies are not spawned: loss changes the plant's topology/state;
a falling-branch physics feature is a separate integration.

## Evidence

`tests/wood_details.rs` covers deterministic closed root geometry, source retention,
legacy absence/roundtrip, persistent loss, metric anchoring through growth,
periodic shape/grain evaluation, causal previews, event-step independence,
transactional refusal and undo/redo. Existing botany and wood-authoring tests
exercise the old source/material/sculpt/physics contracts.

`python3 scripts/demo_wood_details.py` writes the current study to
`artifacts/wood-details/review.html` and `verification.json`. It reads the preserved
complete ancestral experiment when available, otherwise the tracked woody fixture.
Root before/after views share source, camera and neutral lighting. A separate
living specimen starts at birth with the loss schedule, demonstrates recorded
pruning and knots, age previews back and forth, exact save/load images, and the
existing native bark material. `assets/botany/wood-history.ignis-botany` stores
that new individual's birth state with its schedule, not a final game asset.
Native technical checks do not establish artistic acceptance or runtime FPS.

Open the saved neutral history specimen with `python3 scripts/open_wood_details.py`.
The editor exposes both the age slider and the wood-history controls.

Current native study (build829bbd6db24f11bb): full preserved root specimen
106934→120070 vertices and213864→240136 triangles; application about0.44s.
The leafless living study uses413150 vertices/818880 triangles. These are
authoring/rebuild measurements, not FPS. Both meshes passed edge incidence,
orientation and positive-volume checks. Preview/publication, cancellation,
undo/redo, save/reopen and backward/forward age comparisons passed exact PNG
checks. Thirty release tests and focused strict Clippy passed.

Visual review still exposes faceting in local marks/root ridges and imperfect
wide collars. The existing bark material demonstrates coordinate deflection;
it is not a new final bark asset or proof of photorealism.
