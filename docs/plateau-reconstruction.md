# Reference-specific plateau reconstruction attempt (EAF-221)

Target: user crop e3fbcdc9,882x472, hash/regions/inventory P1..P9/C1
in plateau-rock-relief.md. Those crops were revisited before construction;
P3 upper lip/oblique face dominates P5 supports/P7 foot. Background trees and
ruins remain explicit excluded context for this structural iteration. No scale
can be measured from the crop; authored metres and hidden depths are hypotheses.

Driver scripts/reconstruct_plateau_crop.py uses native Landform profiles,
sections and local shaping through MCP. The executable list delivered before
construction is P1 wall, P2 plateau rim, P3 thick inclined slab, P4 lower wedge,
P5 left supports, P6 right lobes, P7 foot outcrops, P8 foreground terrace,
P9 small grooves. Reuse of existing native tools, no engine implementation change.

Axes: X along wall, Y up, Z depth; wall front -Z. For Y-axis extrusions,
profile second coordinate maps toward -Z, whereas X/Y profiles use +Z.
The initial terrace had a gap because that difference was misapplied; corrected.
Slab longitudinal heights and camera side were iterated to correct the sloping
upper rim. Camera was not held fixed across these exploratory attempts; only
final captures/comparison are a fixed final baseline, not a measured improvement
series. The attempted camera is manual, not solved or aligned to the photograph.

Current recipe contains17 closed native solids, not a Boolean union or proof of
all joints. Side/rear images were opened: actual volumes exist, but rear shape
is a hypothesis. Shaping on the slab is native cold subdivision2 plus10 local
stamps; the resulting little repeated facets are themselves visually divergent.
Physics opt-in uses native publication; no new gameplay test/FPS claim follows.

Structural review: P1/P8 broad wall/terrace exist but too regular. P3 still reads
as a flat raised blade with wrong proportions rather than the reference's
massive tilted rock plate. P4 looks like an attached block, not the diagonal
shoulder. P5/P6 remain repetitive prismatic supports, P7 resembles placed
polyhedra. P9 is not reconstructed. P2 is partly occluded and needs checking.
The result does NOT pass the structural/reference-fidelity gate; do not present
material detail, manifold checks or source formats as artistic acceptance.

Evidence artifacts/plateau-reconstruction: reference/front/side/rear.png,
crop.scene.json, verification.json, comparison.json and difference.png.
User crop resized1.5x to1323x708 for the diagnostic only; no added source detail.
Full-frame native RGB/SSIM comparison is unaligned, without matching lighting,
background or vegetation. It is appearance evidence, not geometric fidelity,
a similarity percentage or proof that the camera is correct.

Static performance before/after: bounded17 small sources, subdivision only on
one slab, immutable published meshes; no generation per tick, new render pass,
dependency, build or window. Native closure/manifold per piece and scene
save/reload byte-equal PNG pass. Python compile/diff checked. No new tests for
unchanged engine code; human art/fidelity acceptance remains pending. Parallel
botany and LoS files remain untouched. Source is editable in
examples/plateau-reconstruction.recipe.json; current scene is saved via MCP.
