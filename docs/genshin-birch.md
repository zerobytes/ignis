# Procedural birch study

The first tree in the requested Genshin reference collection has three original,
seeded variants. The [reference inventory](genshin-birch-analysis.md) preceded
geometry. This is an editable native botany study, not an extracted game asset
or an assertion of exact visual equivalence.

## Reproduce and open

From the repository root, with the current native executable:

```sh
python3 scripts/genshin_birch.py
python3 scripts/demo_genshin_birch.py --structure-only
python3 scripts/demo_genshin_birch.py
python3 scripts/open_genshin_birch.py
```

The review launcher opens its own editor window. Existing projects/windows are
not modified. `--binary /absolute/path/to/ignis` is accepted by both native
review scripts. `--only birch_0` limits review to one specimen. Current outputs
replace stable paths in `artifacts/genshin-birch`; no additional engine build
or Blender/image-generation stage is required.

`examples/genshin-birch.recipe.json` is the authored profile. It controls seeds,
height, reach, lean, crown insertion range, primary count, fork depth, shoots,
leaves, blade dimensions and tessellation. `scripts/genshin_birch.py` extends
the existing Builder/curved-axis mathematical lab; native Rust alone compiles
the curves, junctions, blade geometry and materials. No renderer modifications.

Sources in `assets/botany/genshin-birch`:

- `birch_0.ignis-botany`: tall golden spire, seed 220041.
- `birch_1.ignis-botany`: broader, shorter crown, seed 221050.
- `birch_2.ignis-botany`: slender leaning crown, seed 222059.
- `library.json`: source/material paths and variation metadata.
- `materials.recipe.json`: editable procedural bark/foliage recipes.
- `<prefix>_wood.ignis-material` and `<prefix>_leaves.ignis-material`: native
  fixed 256² material outputs. These contain generated fields, no photographs.

To publish a specimen elsewhere, `Botany.load` its source with `publish:true`,
then `Material.load` each manifest material into `<prefix>_wood` or
`<prefix>_leaves`. The source uses white instance tint so the generated material
owns its appearance. Loading only the botanical source, without its material
files, displays neutral geometry. Native regeneration with preserve policy
retains generated materials on the current compiler. The demo performs that
same public command workflow rather than embedding images into geometry.

## Result and validation

Native macOS/Metal build: `84893626b0fb0d2e`. No new application build was made.
All variants contain 8,329 organs, 7,848 closed blades, 481 woody axes and 396 native
junctions; seed and profile change actual curves, not just tint/rotation.

| Variant | Actual height including leaves | Vertices | Triangles |
|---|---:|---:|---:|
| birch_0 |8.354m|376330|721264|
| birch_1 |7.441m|376242|721088|
| birch_2 |8.826m|376426|721456|

The original 10/18-branch experiments exposed graft-spacing constraints. Narrowly
spaced collars and shoots almost parallel to their parent were rejected. The
final graph spaces 12 primary branches, gives terminal axes sufficient length,
and builds lateral shoots in the transported parent frame. Guards were not
disabled and no engine tolerance was changed.

The first native volume review also showed disconnected-looking foliage
islands and excessive visible stem. Lower crown insertion, shorter bare branch
reach, longer leafy shoots and real interior ascending shoots improved
overlap. These are general graph/distribution changes, not photo-projected
surfaces. Front/side/back/top neutral captures preceded appearance work.

Current evidence in `artifacts/genshin-birch`:

- `source-validation.json`: regenerated recipe equality for all 3 sources,
  one-root/parent-before-child/unique-organ checks and distinct hashes.
- `structure-report.json`, `*-gray-{front,side,back,top}.png`, `*-structure.png`:
  actual volume and woody graph, including rear and overhead hypotheses.
- `verification.json`: native compile/counts; every triangle edge has one
  oppositely directed mate in each woody/foliage output; individual 3 specimens
  and full gallery save/load return byte-identical PNGs.
- `gallery.scene.json`, individual `.scene.json`, `gallery.png`, `bark.png`,
  `foliage.png`, individual front/side/back captures and `review.html`.

The topology check establishes closed oriented edge incidence, not universal
self-intersection freedom or biological fidelity. No physics/contact/wind
calibration or FPS benchmark is implied by this study. The full closed-leaf
assets are high-detail authoring outputs; they are not a certified forest
runtime budget or Genshin's card/LOD implementation.

## Visual comparison and remaining differences

| Analysis item | Native implementation/evidence | Comparison |
|---|---|---|
| B1..B3,B10 | `trunk`, `branch*/f*`, `*/shoot*`, `*/interior`; bare/gray views | Connected ascending hierarchy and supported base constructed; hidden original graph remains unknown. Basal widening is deliberately modest. |
| B4,B5 | Height-dependent reach, staggered golden-angle insertions and overlapping sprays; whole/front/side/top | Elongated crown and irregular lobes achieved. Lower/middle gaps remain more open and apex more rounded than R1; not an exact silhouette match. |
| B6 | Folded pointed strip blades with authored length/width; `foliage.png` | Broad pointed leaves constructed; close-up edges are more geometric and leaves somewhat oversized relative to the reference. Serration and resolved veins were not visible in reference and are not claimed. |
| B7,B8 | Native generated wood material; `bark.png` versus C2 | Pale bark with multiscale transverse dark marks achieved. Marks are smoother/more repetitive than the game's irregular painted scars. |
| B9 | Native gold/ochre procedural material, direct light/IBL/shadow; whole/foliage captures | Gold identity achieved. Game lighting, foliage normals, occlusion and painted tonal masses are not replicated. Initial flat yellow was reduced with stronger procedural tonal variation. |
| B11 | Three independent sources/seeds and gallery at a shared camera/scale | Three geometric variants constructed; exact art match and user acceptance remain pending. |

All 11 executable inventory items were constructed and manually compared; the
differences above remain differences, not passed fidelity gates. Context grass,
rocks, UI and other tree species were excluded from this one-species asset
study. Reference images are local comparison inputs and are not redistributed
in the asset library. There is no numerical similarity score: the available
references, chosen camera, scale and lighting are not aligned well enough for
an RGB error to represent fidelity.
