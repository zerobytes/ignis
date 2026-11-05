# Sequoia grove scene

Author with `python3 scripts/author_sequoia_forest.py`. Open the current scene with
`python3 scripts/author_sequoia_forest.py --prepared --open`.

The editable distribution/light/terrain recipe is
`examples/sequoia-forest.recipe.json`. The native scene is written to the stable
path `artifacts/sequoia-forest/forest.scene.json`; the same directory contains a
ground-level capture, overview, population data and three native living source
files. Regeneration replaces these outputs. No engine build is required.

The 512×512m terrain supports 68 sequoias in an irregular grove with a winding
open corridor, small clearings and 600 scattered grass clumps. The three native
sequoia grammar8 variants use seeds173/83/331 and authorial ages180/700/1600years,
with independent branching and per-instance scale/yaw/tint variation. Ages are
profile inputs, not field-calibrated botanical measurements.

Native botany generates the trees before placement. Wood/foliage assets and
materials are shared between instances. This scene uses static snapshots; the
three editable living sources remain separate `.ignis-botany` files beside it.
There is no per-tree live growth, wind update or trunk collision in this scene.
The ground uses native Terrain4 with noise and two hills, a procedural soil/moss
material and mesh collision; a player is placed at the opening for later walking.
This is a finite scene, with no new streaming, impostor or LOD implementation.

The initial authoring pass used dense foliage and was reduced for forest use:
final shared source geometry is1,329,102triangles; fully expanded instance count
is31,072,700triangles before camera/visibility decisions. Sharing reduces storage
and uploads, not this expanded draw work. These counts are not an FPS result.
Trunks retain the volumetric sequoia relief; foliar sprays are aggregate proxies.
The bark/foliage materials reuse the existing native generated sequoia assets.

Created with the existing native application/MCP and inspected in ground-level
and elevated captures. No automated tests, test cases or verification harnesses
were added or executed. The renderer, other open scenes and parallel source edits
were preserved. Visual quality and sustained gameplay performance remain human
acceptance decisions.
