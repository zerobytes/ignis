# Seeded iris authoring

Source7 adds two bounded ocular generators to the existing character compiler.
The source is `human-eyes.ignis-character`, reproduced offline by
`scripts/character/eyes_source.py` after the source6 pipeline. No additional
MakeHuman data, bitmap photographs or external assets are required.

The existing 262 body/face parameters, sparse targets, cage, refinements and
163-joint skeleton are retained. Forty-two ocular coefficients bring the total
to304. Source1..6 decode with an empty ocular list and retain their old bake.
Source7 admits at most two eyes and verifies distinct part ownership, shared
rigid socket, bounded center/radius and complete bounded parameter declarations.
The existing seven part IDs remain stable: each old `eye_*_pupil` ID now holds
its corneal surface; the pupil is a black region in the iris map.

## Appearance and controls

The generator combines periodic polar noise at several angular/radial scales,
warped radial fibers, central pigment, irregular collarette/crypt depressions,
outer contraction grooves, pupil ruff and limbal ring. Four strata per texel and
a complete box-filtered mip chain suppress distant shimmer. There is no painted
highlight: common lighting and a separate smooth thin-glass corneal dome provide
reflection. The sclera has an opening at the limbus; the recessed iris remains
visible without coplanar overlapping caps. Every surface follows the existing
eye socket, including whole-head tilt, and pigment uses local rest coordinates.

The Character panel exposes **Olhos · íris** with color pickers, five color
presets, pupil ratio, iris radius, two pigment colors, central blend, ring
width/darkness, fibers, crypts, grooves, relief, depth, roughness, seed and map
resolution. **Enquadrar olhos** changes the editor camera. Linked editing copies
only coefficients actually edited to the other eye; unlinking permits
heterochromia. Both parameter sets persist independently. MCP uses the exact
same `SetCharacterParameters`/Configuration transaction, with undo and guards.

The draw identity (`seed`) is deterministic, not a simulation of a particular
person's anatomy. Pupil diameter is manually authored. The current renderer uses
thin-sheet glass reflection/transmission, **without spatial ray refraction**;
this is a visual authoring increment, not complete photoreal ocular optics.
Automatic light response, tear meniscus, lacrimal/caruncle geometry, eyelash and
lid shadow/correctives remain separate work.

## Cost and persistence

Pigment generation occurs during authoring. A bounded per-eye last-result cache
reuses the same material Arc when body/posture alone changes. Rendering samples
the fixed mip maps; it does not evaluate the polar generator per fragment.
Resolution64/128/256 uses5461/21845/87381 samples,48bytes each:
about0.25/1/4MiB per distinct map, plus96bytes of program metadata. Scene
serialization includes those maps. The cornea requires the existing transparent
composition passes, whose target cost is not restricted to the eye's pixel size.
A small visible eye therefore does not imply zero or universally negligible cost.

Source7 stays within existing source96MiB,512parameters,80handles,100k compiled
vertices and rig16influences limits. Recipe2 is unchanged. Baked procedural
materials require Scene112 (provided by the material system), and the normal mesh
publication transaction promotes that version. Existing replay/network/portable
byte limits remain; the refined full body already exceeds replayJSON8MiB and
this increment does not bypass that limit. Old sources remain immutable.

Reproduce an empty checkout in order with `prepare_source.py`, `extend_source.py`,
`refine_source.py`, `smooth_source.py`, `cupid_source.py` and `nose_source.py`
under `scripts/character/` (see the base documentation for pinned data fetching).
Run `python3 scripts/character/eyes_source.py` and then
`python3 scripts/character/eyes_demo.py` with the current executable. Evidence,
controlled close/portrait/oblique captures and a separate editable scene go to
`artifacts/character-eyes/`; the user's original character scene is preserved.

Anatomical reference for the surface feature inventory:
[Edwards et al., *Analysis of iris surface features in populations of diverse ancestry*](https://pmc.ncbi.nlm.nih.gov/articles/PMC4736923/).
The corneal IOR1.376 is a conventional graphics approximation; it is not an
individual optical measurement or a model of the tear film and depth variation.
No reference photograph is embedded in the generated maps.

Open the verified scene with `python3 scripts/character/open_eyes.py`. The launcher
uses the one current executable, preserves unrelated windows and reuses its own
process when already open. Its layout disables the work-light override so the
editor and native captures use the same scene lighting.

## Validation and handoff

Twelve `character_base` Rust tests passed, including a source7 fixture for
legacy rejection, rigid socket transport, material cache reuse, bounded atomic
edits, undo/redo, Scene/portable writing and a small replay. The whole refined
body retains the pre-existing replay8MiB limitation. Release bin and the cost
example compiled. Owned Rust formatting, Python compilation and diff checks
passed. Strict Clippy is tracked separately from unrelated terrain warnings.

Four real MCP groups and nine native close/portrait/oblique views passed on
build49ebe1482fac4650. All262 previous values and the complete body/rig were
exact after upgrade; configuration/Scene/portable reload produced identical
PNGs. In that sample edits took42–55ms; source upgrade took0.72s. These are
observations on this machine, not latency guarantees. Source SHA is
fb4c3a25da1e735b0bcb835a920c0a5ee1c351793d5338a7b5c55083aaeda92c,
73,343,828bytes. Total61,200vertices/121,904triangles versus55,922/111,664
previously; body53,514/107,024 is unchanged.

A provisional 1000×900 offscreen sample on buildab1bea9817a33269 measured the
following median draw plus blocking GPU completion (16warm samples each):

| Pose | Old caps | Iris + cornea |
|---|---:|---:|
| Close | 8.54ms | 13.36ms |
| Portrait | 6.15ms | 14.43ms |
| Body | 3.22ms | 8.29ms |

`artifacts/character-eyes/cost.json` records the raw samples and renderer memory
counters. Concurrent builds and an interactive editor were active: these are
provisional workload measurements, not isolated throughput, pure GPU timings or
gameplay FPS. The current transparent cornea has appreciable overhead; a small
iris does not make the complete ocular implementation free. Runtime eye LOD and
more compact material maps remain performance work, not a claim of this change.

The native editor was opened separately as PID61737/build49ebe1482fac4650,
paused/FPSoff, with the eye panel and close camera. CUA observed its controls and
the user's live color/pupil edits. Subsequent human edits were preserved and
saved before the next mandibular increment. No unrelated window was selected.
The launcher uses a symlink to the single current executable. Artistic
acceptance, full corneal optics and broader character milestones remain open.
