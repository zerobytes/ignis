# Botanical valley: a finite 1 km landscape

EAF-210 composes the existing botanical libraries into a native Ignis scene,
separate from the ongoing renderer/material work. This is landscape authoring
and placement, not a new rendering feature or an offline Cycles image.

The region spans `[-500,500]` metres in X and Z: **1,000 × 1,000 m**, one square
kilometre. A native Terrain2 recipe creates three asymmetric ridges, a valley,
small rolling variations and a channel. The 2 m grid has 251,001 vertices and
500,000 triangles. Its measured height range is approximately 3.00–113.72 m.
The same complete terrain mesh supplies walking collision.

## Composition

`scripts/build_botanical_valley.py` reads the existing native library bakes;
it does not regenerate or overwrite canonical plant sources. All 36 groundcover
variants and 24 tallgrass variants are included. The heavier shrub collections
use one intact variant per habit: eight green/flowering, four colored, and eight
desert/topiary habits. The four earlier native tree-gallery exemplars and the
corrected RS fig tree complete 45 habits / 85 selected variants.

The four tree-gallery bakes are the existing gallery snapshots, not newly
regenerated mature presets. The RS fig tree uses the full corrected collar/root
bake from `artifacts/botany-collar/figueira.scene.json`. `composition.json`
records the actual input paths and hashes. Do not interpret older tree snapshots
as an exact representation of every current tree generator setting.

Placement uses a seed, smooth spatial fields and grove centres. Short vegetation
is present across all 400 cells of a 50 m occupancy grid; this measures geographic
distribution, **not continuous blade coverage**. Three explicit foreground zones
have much denser grass. White and purple flower islands occur at trail bends.
Tall growth occupies patches, colored shrubs give accents, and dry shrub habits
appear among the collection's contrasting forms. This is an authored mixed
botanical landscape, not a scientifically calibrated ecosystem.

The trail is an exclusion strip in the vegetation. It is not a separate road
material or a route/navigation graph. Water is a flat reflective surface exposed
by the terrain channel; it is visual, without flow, fluid simulation or swimming.

Final composition:

| Collection | Plants / tufts |
| --- | ---: |
| Groundcover, including flowers | 401,600 |
| Tall growth | 9,024 |
| Green / flowering shrubs | 688 |
| Colored shrubs | 184 |
| Desert / topiary shrubs | 128 |
| Trees and palm clumps | 108 |
| **Total** | **411,732** |

Several plants contain separate material components. The scene has **694,498
mesh instances**, 208 shared assets, 7,800,135 unique vertices and 15,138,790
unique triangles, including terrain/water. Summing triangles across every
instance before visibility rejection gives **1,199,294,230** triangles. Sharing
geometry saves duplication; it does not make that draw workload free. There is
no new distance LOD or streaming in this composition, and no presentation FPS
claim. This is intentionally a heavy authoring scene, not an optimized shipping
level.

Vegetation is static visual geometry. It has no new wind animation, plant
interaction or vegetation collision; the walking check covers terrain only.

## Reproduction and use

Existing prerequisites are the bakes produced by `demo_groundcover.py` for all
five collections, `demo_botany_native.py`, and the corrected fig-tree scene.
The composer refuses stale source/library fingerprints. Generated files remain
under `artifacts/`; libraries under `assets/botany/` remain unchanged.

```sh
python3 scripts/build_botanical_valley.py
```

The recipe is `examples/botanical-valley.recipe.json`. Counts and placement seed
are configurable. The authored domain is fixed at 1 km, with 2 m or 4 m grid
spacing. Ridge/channel parameters are explicit in `terrain_recipe()`.
Stages `terrain`, `compose`, `validate`, and `review` allow focused rebuilding.
The composer retains the scene version emitted by the executable; it does not
copy a newer version number from unbuilt parallel Rust work.

Current outputs in `artifacts/botanical-valley`:

- `world.scene.json`: the native scene with shared static vegetation and player.
- `terrain.ignis-terrain`: editable native terrain source.
- `terrain.scene.json`: the unpopulated physical base.
- `placements.json`, `composition.json`: placement ledger, provenance and costs.
- `views.json`: aerial, valley, flowers, grove and ridge cameras.
- `review.html` and the five PNGs: native images, captured at 2048 × 1280.
- `native-validation.json`, `captures.json`: runtime checks and capture metadata.

From the repository root, open a separate editor instance:

```sh
target/release/ignis --root . \
  --scene artifacts/botanical-valley/world.scene.json --editor
```

Or use the existing native first-person mode on the physical player:

```sh
target/release/ignis --root . \
  --scene artifacts/botanical-valley/world.scene.json \
  --window --continuous --first-person player
```

Generation uses an isolated headless MCP process. It neither opens nor replaces
the user's current world/window. No new executable, shader, format or library
source is introduced by this work.

## Validation and visual limits

The complete candidate is loaded, saved by native Ignis, and loaded again.
Every persisted plant transform is compared with the actual published terrain
triangles. Material components of a plant must share its transform. The physical
player is allowed to settle, walks for 120 ticks, and must remain grounded after
moving more than 3 m. The reviewed camera/light image must be byte-identical
after save/load. Reports record the executable's actual build ID.

Final native validation used macOS arm64 release build `9f3acc12d0dedf60`.
All 411,732 persisted plant transforms passed, with maximum root-transform
surface error 0.00002144 m. All 400 occupancy cells contained short vegetation.
The player walked 7.96 m and remained grounded. The final ridge PNG was byte
identical after save/load. Five 2048 × 1280 views were captured and visually
reviewed. Python compilation and the change's whitespace check passed.

First visual review found sparse foregrounds. The close-zone count was increased
from 36,000 to 180,000, and flower islands were added before final capture.
Foreground zones use radial density falloff to soften their transition into
the surrounding lower-density meadow.
The close flower camera was moved off the trail to show the actual planted area.

Native materials, direct/ambient lighting, shadow coverage and pixel sampling
still impose visible limits: thin distant leaves can look stippled, the ground
has exposed gaps, silhouettes are faceted at close range, and lighting is less
rich than the separate Cycles hero. A finite square edge is visible in the aerial
layout view. The directional shadow map covers a limited local area; review
cameras move its centre rather than claiming simultaneous kilometre-wide fine
shadows. These are recorded limits, not evidence of Cycles parity or artistic
acceptance. The separate renderer effort can use this maintained native scene
as a larger visual and cost reference.

The user's runtime test subsequently rejected the scene at approximately2FPS.
CPU/GPU/shadow measurements and reproducible diagnostics are documented in
[Botanical valley performance](botanical-valley-performance.md). The composition
has not been optimized by that measurement task.
