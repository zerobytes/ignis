# Discontinuous exposure plateau study (EAF-221)

User rejects the regularity of directional-plateau: continuous shelves and
planned-looking joints. Keep common bedding, break surface exposure locally.
Construction inventory before geometry: N1 same whole lobed plateau and camera;
N2 tilted common strata frame; N3 broad preserved faces alternating with local
recesses; N4 layer exposure masks varying gradually around the perimeter and
between adjacent strata; N5 broad asymmetric recesses spanning several strata;
N6 shorter tapered joints in a common family, not three full-height slots;
N7 unequal local chips; N8 neutral/material/rear views plus native topology,
save/load and replay checks. No physical erosion or tectonic simulation claim.

Static performance pre-review: existing native variable sections and bounded
convex subtraction only; at most 27x32 source controls and 40 cuts. Client creates
controls, native Rust creates topology. Cold authoring, no new runtime loops,
GPU passes, dependencies, format or retained build. Existing transactional
refusals remain authoritative. Preserve parallel sequoia and LoS changes.

## Observed result and validation

Current existing binary de6b0252e7e4d8ca; no Rust or renderer rebuild by this unit.
40 proposed convex cuts: 36 accepted/4 conservative contact refusals, each refusal
checked against the previous geometry. Final 1228 vertices/2456 triangles,
zero boundary/nonmanifold edges and one mesh island; this closed-surface check
does not assert absence of through-passages. 110 top triangles receive turf.
Native save/load PNG equality and replay/mesh agreement pass. Python compile and
diff checks pass. The shared driver was parameterized instead of duplicating it;
its default recipe equals the committed directional recipe, and five original
PNG hashes remain byte-exact after rerunning on the current binary.

Neutral, overview, detail and rear were opened. Compared with the prior recipe,
shelf exposure varies strongly around the perimeter and between nearby heights;
large clean faces coexist with locally corrugated recesses. No three repeated
full-height slots. Broad cuts were moved from face centers to convex edges after
an intermediate attempt left triangular punctures. Intermediate variants replace
the same output paths, not a history of scenes/builds.

Remaining visual issues: broad planar column-like patches, locally repetitive
striation on the right, and a few sharp artificial-looking cut marks. Rear is
simpler and smoother. This demonstrates discontinuous exposure controls; it is
not human art acceptance, a Genshin reconstruction, branched fault simulation or
physical erosion. Original directional and weathered recipes remain available.
All geometry comes from native Landform/CSG via MCP; no Blender bake or Python
triangulation. Source controls, accepted cuts and cameras are retained in
examples/patchy-plateau.recipe.json. Captures, scene, verification, rejection
records and default-driver regression evidence: artifacts/patchy-plateau/.

Post-review performance checklist: all surface fields/candidate scans are bounded
cold authoring, accepted mesh baked once, no new gameplay/render loops or GPU
passes; native input/work/collision/atomicity guards unchanged. No frame-rate,
world-streaming or universal geology claim. Parallel botany/LoS files untouched;
no foreground window, copied binary, push or publication. EAF-221 In Progress.
