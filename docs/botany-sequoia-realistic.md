# Giant sequoia appearance refinement

This replaces the *appearance target* of the simple demonstration, without changing grammar5 or its saved sources. Grammar6 is an explicit native species evaluator in `src/botany/growth/sequoia_realistic.rs`; the same age, seed, pruning, source/mesh compiler and editor preview contracts remain in use. The native captures under `artifacts/sequoia-realistic/` are the review result. They require human visual acceptance; implementation/tests do not establish photorealism.

## Reference and acceptance target

The reference is mature **Sequoiadendron giganteum**, rather than a young triangular conifer. California State Parks describes the old crown as rounded, with large masses of greenery, contrasting with younger pointed crowns ([growth and development](https://www.parks.ca.gov/?page_id=1151)). The National Park Service documents reddish fibrous bark and high branches ([giant sequoias](https://www.nps.gov/seki/learn/nature/bigtrees.htm)). These guide the crown, clear bole and surface; no photograph supplies geometry or a painted lighting effect. The image used to inspect fine branchlets is NPS's [foliage collection photograph](https://www.nps.gov/seki/learn/nature/images/AA_foliage-collection.jpg), credited there to Wendy Baxter/UC Berkeley. This is not an exact reconstruction of General Sherman or a calibrated growth model.

The simple example's regular three-arm levels and broad, flat sprays were rejected. The new evaluator distributes sites at seeded irregular heights/azimuths, bends branches laterally and down then up, varies higher-order directions in three dimensions, and blends the juvenile cone into a compact mature crown envelope. The lower half of potential main sites is omitted after 300 years. This finite architecture is authored, not the outcome of a light/carbon/deadwood simulation.

## Native geometry

Height and nominal diameter retain the independent smooth growth curves. The bole's star-shaped contours are generated from age-dependent radius, seed and longitudinal position, using the existing native woody-surface evaluator: 32 angular samples, 128 longitudinal samples, nine contour stations, basal flare and broad irregular fluting. This is a derived grammar6 field, not an author edit smuggled into a living recipe. The existing restriction on manual `wood_surface` edits to living recipes remains intact.

The nominal diameter in the growth profile is **not** the width across the flared basal contour. Stage reports therefore label it `nominal_diameter_m`. Height reports describe the trunk axis; greenery may extend above it. All final mesh budgets remain unchanged.

Fine foliage is still an aggregate representation. Each closed tuft uses the existing multi-tip contour with thirteen tapered tips, arranged around actual parent twigs. These are geometric branchlet proxies, not thirteen-lobed biological leaves. The point of using this contour is to retain a finely broken silhouette without the broad opaque sheets exposed by a cutout-material attempt. Juvenile/full-scaffold trees use two tufts per twig, mature high-crown trees five, keeping the complete exemplar inside existing budgets. Larger tip/leaf populations are not promised to fit all parameter combinations. Fine branchlets and main limbs use the existing separate, closed sheath connection; this is not universal vascular fusion.

## Texture generator and reusable assets

Both surfaces are authored using the public **native `material.generate` command**, then exported with `material.save`:

- `assets/materials/sequoia/bark.recipe.json` and `bark.ignis-material`: layered anisotropic noise/ridges, restrained fissures, correlated rust-brown albedo, height and roughness; native pixel normals and 9mm parallax depth. The broad trunk flutes belong to geometry.
- `assets/materials/sequoia/foliage.recipe.json` and `foliage.ignis-material`: green variation, roughness and thin diffuse transmission. The final material is opaque; fine silhouette comes from geometry.

The `.ignis-material` files retain editable runtime recipes rather than baked lighting or external photographs. Geometry supplies organ-local material coordinates: bole/branch grain follows the axis and foliage has its own longitudinal/transverse domain. The angular bark map has a periodic-coordinate meridian, so the present per-vertex domain can still stretch detail on the single seam strip; a general corner-UV/triplanar system is outside this change. Current material resolution/filtering and the engine's lighting still constrain the close-up result.

Procedural materials are independent appearance primaries. Existing native botanical preservation keeps them during age changes and previews; the review checks that their inspected definitions remain equal through the growth sequence. Loading only a botanical source does not load independent appearance files. The exemplar button and launcher load the source **and** the two companion materials.

## Reproduce and validate

```sh
python3 scripts/preview_sequoia_materials.py
python3 scripts/demo_sequoia_realistic.py
python3 scripts/open_sequoia_realistic.py --years 1000
```

The source definition is `examples/sequoia-realistic.recipe.json`; the birth-state native asset is `assets/botany/sequoia-realistic.ignis-botany`. The launcher explicitly opens the editor and the age slider. The review creates a native scene, generates/applies/exports materials, captures multiple ages and the whole tree/base/editor, requires exact 5→1000→5 preview images and save/load PNG equality, verifies canonical state preservation, and checks persistent pruning of an extant mature site. Advance to 3000 years is also exercised. The mathematical/compilation tests cover birth, 1, 20, 299, 1000 and 3000 years including the derived native bole. The material card preview is separate from the final tree.

`verification.json` records generation command wall time, not FPS. The scene is a single detailed specimen; this is not a forest performance or gameplay-growth benchmark. Existing cone/reproduction, automatic gameplay, environmental history, physical calibration and full-tissue dynamics limits remain unchanged. The older example and existing user windows are preserved.

## Measured native review

Release `de6b0252e7e4d8ca` generated 6,486 organs at 1000 years: 2,171,349 vertices / 4,316,739 triangles, including reproductive placeholders. Whole-mesh bounds reach 81.021m vertically; the trunk-axis height is 79.986m and the diameter parameter is 7.329m before the basal contour. Native advance command wall time was about 3.48s in this review. The saved scene is deliberately a detailed single-specimen review, not a small forest-ready mesh.

The final species test passed at birth/1/20/299/1000/3000 years, including the bound derived bole and persistent pruning. The six existing growth tests also passed during development; the final appearance changes are grammar6-only. Final release Clippy library/binary, module formatting and Python syntax checks passed. The old grammar5 scene rendered PNG-byte-exact. The final gallery proves reversible preview, canonical preservation, native save/reopen image equality and independent material retention. Native editor PID59660 was opened at 1000 years, with the 0–3000-year slider; older windows were retained. No push or renderer/lighting implementation changes were made. Artistic acceptance is pending.
