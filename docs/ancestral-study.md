# Regenerated ancestral-tree study

The user explicitly requested another complete tree generation after the wood
authoring increments. This study uses the approved white ancestral-tree concept
as a visual target, while preserving the separate game experiment and gameplay
scene. EAF-227 and LOS-21 remain in progress; this is a reviewable generation,
not artistic acceptance of the ancestral tree or completion of its toolset.

## Reproduce and inspect

```sh
python3 scripts/regenerate_ancestral_study.py --structure
python3 scripts/regenerate_ancestral_study.py
python3 scripts/regenerate_ancestral_study.py --open
```

The first two commands run the existing release binary headlessly through MCP.
The last optionally opens the resulting scene in the native **editor**, with
Vegetation and the trunk selected. No existing human window is closed.
The editable configuration is `examples/ancestral-study.recipe.json`.
All generated outputs replace stable paths in `artifacts/ancestral-regeneration`:

- `review.html`: reference and six paired neutral/foliage views.
- `hero.png`: native material/light presentation, 2048 × 1365.
- `ancestral-study.ignis-botany`: complete editable native source.
- `structure.ignis-botany` and `full.ignis-botany`: neutral inspection sources.
- `bark.recipe.json` and `bark.ignis-material`: procedural recipe and native bake.
- `current.scene.json`: standalone scene with geometry and material embedded.
- `structure-verification.json` and `full-verification.json`: actual compiler
  counts, bounds, build identity and validation results.

Run structure first whenever changing the recipe; the paired gallery assumes
both passes use the same configuration. Generated study assets stay in the
review directory. This does not install a tree in the game, publish to the
independent asset catalog, or add another executable/build copy.

## Construction

The scaffold opens near the base and bends into broad asymmetric arms. Four
auxiliary arms occupy front/back space rather than projecting every major arm
into one plane. Twelve seeded attraction clouds drive recursive spatial
partitioning into finer branches. Terminal shoots carry individual native
closed leaf blades, rather than leaf cards or a photographed canopy.

The Python script authors the existing native graph and editable fields; Rust
performs meshing, junction fitting, woody-surface evaluation, root generation,
root morphology, material generation and rendering. The study reuses Recipe12,
Source9, woody contours/cords, local junction refinement and seeded root details.
Root proportions were adjusted after a native collar-space rejection; admission
guards were preserved. The trunk's optional surface override is disabled in
this recipe because its interaction with this root crown produced a conspicuous
basal ring. Arm surfaces still use the native authoring fields.

The nominal source trunk radius is 3.7 m. Actual trunk contours and overall root
spread are different measurements; inspect the source and compiled bounds rather
than interpreting whole-tree bounds as trunk diameter. These are chosen artistic
dimensions, not new limits on the general tools.

The bark uses warped anisotropic noise/ridged fields with native material
coordinates and bump shading. It does not use the concept image as a texture.
The neutral captures precede material application, preserving visibility of
structural defects.

## Validation and limits

Both passes check wood edge incidence of two, opposing edge orientation,
positive signed volume and nonzero triangle areas. This establishes a closed,
oriented mesh of potentially multiple components; it does not prove global
absence of self-intersection. Auxiliary arms use closed `sheath` components
overlapping the trunk, while the main scaffold uses native connected forks.
These attachments are not anastomosis or a boolean union.

Save/load of the complete scene must reproduce the PNG bytes exactly. The
presentation also reloads its native botanical source, recompiles it while
retaining the material and requires the same PNG. Large sources use native
file loading rather than exceeding the smaller MCP command envelope. The
verification files record actual costs; there is no FPS or forest-density claim.

The reviewed generation contains 1,549 woody organs and 30,720 leaves:
1,383,810 vertices and 2,644,720 triangles in total. Its native source is
21,659,051 bytes. This is a full-resolution review specimen; population/LOD
budgets have not been tuned. Neutral and presented wood geometry hashes match
exactly, independently of the added leaves, material and lighting.

The age of 180 years is an authorial mature snapshot with developmental
parameters. It does not establish an empirical species age, simulate historical
growth or turn this custom scaffold into a persistent living grammar.

Visible shortcomings remain: large fork patches are broad/angular, wood cords
do not flow smoothly through every junction, the base lacks the reference's
braided mass and recesses, root silhouette remains simpler, and the canopy has
more separated clumps. Bump material is not geometric bark fibre. Increased
polygon counts and valid persistence do not resolve these artistic differences.

No engine source, game scene, catalog release, global save format or renderer
behavior is changed by this study. Its scripts and recipe are the tracked
reproduction inputs; native outputs are local review artifacts.
