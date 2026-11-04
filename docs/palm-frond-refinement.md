# Continuous palm leaves with progressive curvature

The palmeirinha preset is a cluster of six basal supports bearing five complete
pinnate leaves each. A leaf is one continuous authorial axis from its bare basal
petiole through the leafy rachis to its tip. There is no long central trunk with
branches attached along its height. Leaflets attach to the distal portion of
that same axis. This replaces the three rejected revisions of 2026-11-04.

The user's supplied photograph is the primary visual reference: long emerging
leaf bases overlap low in the clump, open progressively outward, and continue
into the leafy region. Leaf lengths and orientations vary. The earlier original
height range is retained as a seeded stature input, now controlling complete
leaf length rather than an independently elongated pole. Basal supports still
exist; enclosing, overlapping open leaf-sheath surfaces are not modeled by the
existing closed axis/sheath connection. This remains an authorial approximation,
not botanical calibration or a universal model of palm development.

## Curve

For normalized distance `u` along a leaf of length `L` and authored total turn
`T`, curvature magnitude follows:

```
k(u) = T/L * (0.08 + 2.76*u²)
accumulated turn(u) = T * (0.08*u + 0.92*u³)
```

The existing mathematical frame integrator samples a 17-knot approximation of
that increasing field. It exports ordinary editable source curves, and the
native Rust baker evaluates them. Native axis resolution is 32. Nothing changes
in the runtime or source formats. Central leaves remain more upright; external
leaves receive a larger total turn. The petiole and rachis share one taper;
folioles start around 47% of that same curve, with their distribution and sizes
measured within the leafy region. The former inverse progression (strong base,
weak tip) was incorrect and has been removed.

The canonical source has 1,476 organs, 178,488 compiled vertices and 351,072
triangles. At zero growth variation, native bounds reach 4.67 m high; the current 0.8
variation preset reaches 3.09 m. Existing saved sources retain
their authored curves. This preset changes membership; applying it over existing
manual work must use the existing explicit preservation/discard policy.

## Verification

Run:

```
python3 scripts/botany_exemplars.py --only palmeirinha
python3 scripts/demo_botany_native.py --only palmeirinha
python3 scripts/review_palm_fronds.py
```

The review checks canonical regeneration equality and strictly increasing
segment turn on all 30 complete leaf curves. For the first curve, turn between
successive source segments rises from 0.50 to 13.61 degrees. These measurements
verify source curvature; they do not establish artistic acceptance. Native
load/bake, full and bare captures, and a saved/reopened scene with identical PNG
bytes passed. The comparison uses the same camera and light for the previous
rejected source (`5c774be`) and the current source.

Stable evidence under `artifacts/botany-native/`:

- `palmeirinha-before.png`, `palmeirinha-after.png`: whole clumps, same camera.
- `palmeirinha-whole-leaf.png`, `palmeirinha-whole-leaf-axis.png`: one complete
  leaf in profile, including its bare petiole, with and without leaflets. This
  diagnostic removes development scaling to expose the authored source curve;
  it is not a separately generated artist asset.
- `palmeirinha.png`, `palmeirinha-structure.png`: current full source.
- `palm-frond-verification.json`: source measurements, build and native results.

The existing native exemplar integrity/reproducibility test passed for all four
original exemplars (`cargo test --release --test botany
exemplar_sources_compile_continuously_and_are_reproducible`), including closed
mesh checks and repeated compilation. Artistic acceptance is pending. The old revisions are not
accepted merely because they compiled or produced captures.

Earlier botanical references:
- https://www.nparks.gov.sg/florafaunaweb/flora/2/6/2622
- https://plants.ces.ncsu.edu/plants/chrysalidocarpus-lutescens/

## Authored growth stages and leaf-height variation

The continuous leaf shape received user approval; the next control distinguishes
shorter young leaves from developed leaves. `settings.frond_growth_variation`
ranges from 0 to 1. Every complete leaf can carry `life.growth_stage` from 0.1 to
1, where 1 is its developed reference size. Its applied scale is:

```
scale = 1 - frond_growth_variation * (1 - growth_stage)
```

The same scale propagates through the complete petiole/rachis and its descendants,
including leaflet length, width and thickness. Isotropic scaling preserves the
progressive curve's angles. Parent attachment frames are reevaluated before
placing descendants. The native baker is also the rest source for the existing
local dynamics session. Individual organ edits remain separate multipliers.

The Vegetation panel exposes **Variação de crescimento das frondes** and, for a
selected complete leaf, **Estágio desta fronde: jovem → desenvolvida**. Both use
the existing preview/apply/cancel workflow with manual-edit preservation. MCP
uses the same recipe fields through `botany` preview/publish/load/save.

The palmeirinha preset uses stages 0.30 through 1 across its five leaves per
basal support, with a default variation of 0.8. That makes the youngest stage
44% of its developed length and the oldest stage 100%, in addition to the
existing seeded variation in reference lengths. Stages are authored growth
fractions, not calibrated ages in years. They do not automatically advance a
new biological simulation; chronological plant age and the existing local
expansion clock retain their separate roles.

`palm(seed, frond_growth_variation)` and `--frond-growth-variation 0..1` expose
the generator parameter. Source and recipe 9 protect the new semantics. Older
sources default each stage to 1, and absent variation defaults to 0. Zero
variation is omitted on serialization and preserves the old bake exactly.
Nondefault stages require source9; an active variation requires recipe9, and
source9 requires recipe9. Invalid/nonfinite values are rejected before publish.
No global Scene or replay version change is introduced by this increment.

`tests/frond_growth.rs` covers whole-leaf/descendant scaling, unchanged angles,
legacy equality, version/range rejection, preview atomicity and serialization.
`scripts/review_palm_growth.py` compares 0, 0.45, 0.8 and 1 at the same camera/light,
checks scene save/load pixels, and exercises local dynamics identity/attachments.
Evidence uses stable `palmeirinha-growth-*.png` and `palm-growth-verification.json`
paths under `artifacts/botany-native`. This does not establish artistic acceptance
of the new height distribution or calibrated plant development.

Validation of the growth increment passed: two focused Rust tests, the existing
four-exemplar integrity/reproducibility test, and Clippy release for lib/bin plus
the new test target (`-D warnings`). Native comparison, scene roundtrip and local
dynamics used build `e9140564387683bd`; maximum reported attachment error was
3.47e-18 m. The retained current executable is `target/release/ignis`; captures
replace stable artifact paths, with no extra app bundle or binary copy created.
GUI interaction and artistic acceptance of the new distribution remain separate.
