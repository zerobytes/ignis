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
triangles. Current native bounds reach 4.67 m high. Existing saved sources retain
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
