# Mandibular landmark chain

Thirteen additional XYZ landmarks cover the chin-to-angle contour and ascending
ramus. The native **Mandíbula** group includes these thirteen points and the
existing chin tip and left/right jaw angles (sixteen controls total). Each new
paired point supports optional symmetry. Selecting a point exposes XYZ arrows
and the same bounded offsets through the ordinary character MCP tools.

| Region | New controls |
|---|---|
| Inferior midline | `chin_base` |
| Mental tubercles | `chin_side_L/R` |
| Inferior chin corners | `chin_corner_L/R` |
| Anterior body | `jaw_front_L/R` |
| Middle body | `jaw_body_L/R` |
| Posterior body | `jaw_back_L/R` |
| Ascending ramus | `jaw_ramus_L/R` |

Source `human-jaw.ignis-character` is a new immutable source7 dataset, ID
`human_adult_jaw_v1`; the schema/Recipe2 and global formats do not change.
Reproduce with `python3 scripts/character/jaw_source.py` after the
[ocular source](character-eyes.md) pipeline. No new vendor data or anatomy mesh
is downloaded. It retains the same topology,163joints,7parts,67oldhandles,
469oldtargets and304oldparameters, adding39zero coefficients/39targets.
The three previous mandibular handles are only regrouped; their fields, limits
and stored values remain unchanged. All304 previous values and the complete
bake are identical at zero in the real upgrade proof.

Each local polynomial field uses the original cage and is normalized at its
Catmull-Clark drawn anchor. Fields affect13..39cage vertices with smooth finite
support. Lip/ear anchor stencils and the bounded physical ear region form
878protected input vertices; the new fields do not move the existing31visible
lip/ear pins. Radius/limits are recorded in the generated source report.
Individual limits reach10..18mm and intentionally permit stylized shapes;
combinations are not certified free of self-intersection.

The dataset has343parameters,508targets and80handles. It reaches the existing
handle ceiling; further landmarks need an explicit capacity review. The existing
512parameter/512target limits remain. Runtime vertex/triangle/bone counts do not
increase: the controls change the existing baked geometry. The ocular material
and its cost are unchanged by the new chain.

## Validation and delivery

`python3 scripts/character/jaw_demo.py` runs a separate MCP process on the saved
human scene. Four groups passed on build9f6a5e131794ac36: exact zero bake and304
values preserved with undo/redo; all13anchors moved2mm; optional paired symmetry
and unilateral editing; all78XYZ endpoints compiled; preview/error rejection
remained atomic. The31lip/ear pins plus nose/eyes/forehead sentinels moved0m in
the individual2mm trials. Configuration/Scene/portable reload preserved all343
values and the source hash. Source SHA is
23e844801bb2eacdc72a71ab649313c52a6c113eecd8f5f032f7470912fca970,
73,396,944bytes. Four shape PNGs and a native point-group capture are in
`artifacts/character-jaw/`, with source/verification receipts and the original
saved snapshot. No new per-slider unit tests were added; the existing12critical
character Rust tests passed during the ocular unit.

The running native editorPID61737/build49ebe1482fac4650 was upgraded in place
through its guarded source control. CUA verified the new group and visible
points. The user continued authoring during validation; their additional
changes to ears, eyes, lips and other existing controls were retained. The live
scene was saved with343values and39newzeros, recorded separately as
`editor.scene.json` and `native-verification.json` under character-jaw. The old
working scene/window was not replaced by a stale test snapshot.

Strict release Clippy was attempted for lib/bin/character_base/cost example.
It stopped on two unrelated in-progress terrain lints:
`src/terrain/surface/domain.rs:23` (`needless_range_loop`) and
`src/terrain/surface.rs:565` (`collapsible_if`). Those reserved files were not
modified or included in this change. Owned formatting, Python compilation and
diff checks pass; native preservation/shape trials are distinct from human
artistic acceptance, which remains open under EAF-187.
