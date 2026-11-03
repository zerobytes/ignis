# Emerald and Japanese procedural lawns

EAF-216 adds three independent lawn asset families under `assets/botany/lawns/`.
The target is the user's paired photograph; the pre-construction analysis and
feature list are in [lawns-analysis.md](references/botany/lawns-analysis.md).
The supplied species labels are retained as reference labels, not confirmed
cultivar identifications. Metric dimensions are authored working dimensions.

## Assets and controls

Each family has three deterministic variants. Each variant is split into mature
and fresh green components for the existing native material pipeline, yielding
18 editable Source/Recipe7 `.ignis-botany` files. Every blade has a curve, normal,
pointed strip outline, folding and closed thickness. There are no image planes,
photographic textures or hidden solid cushions.

| Family | Module / habit | Blades per variant | Width setting | Height setting | Native triangles per variant |
| --- | --- | ---: | ---: | ---: | ---: |
| esmeralda |24cm square carpet|768|3.2mm|4.5cm|49,152|
| japonesa |24cm placement pitch; radial cushion|2,048|0.9mm|9cm centre /3.5cm rim|131,072|
| japonesa_tapete |24cm square fine carpet|2,048|0.9mm|4cm|131,072|

Settings are in `scripts/lawn_assets.py:FAMILIES`. Actual sampled widths vary
between 78% and 114% of the width setting. Heights also vary. Emerald blade origins
fill all stratified rows of a square; azimuth and curvature vary independently.
Japanese origins fill an ellipse, with radial height envelope
`(1 - q*q)^0.72`, an upright centre and increasing outward reach at the rim.
The additional `japonesa_tapete` preserves the Japanese blade width, fold,
thickness and colours, while distributing roots over the complete square and
using short paths without a radial height envelope. Its authored height is 4cm,
with independent variation and modest lean. This is a flat art-directed option,
not a separate species or a biological simulation of mowing. The original
12 source files remain byte-identical.

Radius, height, rim height, width, blade count and fresh-growth fraction can all
be edited. Source curves remain editable independently after generation.

Regenerate the canonical library and native review:

```sh
python3 scripts/lawn_assets.py
python3 scripts/demo_lawns.py
```

Use `--seed U32 --variants 1..8 --output PATH` to create another library without
overwriting this one. A given seed and parameters reproduce the same source in
the same Python environment. New seeds change all component sources. The demo
compiles the canonical `assets/botany/lawns/library.json`; if changing its output
location, adapt `DEST` before compiling that alternate library.

For native editing, load/publish a component `.ignis-botany` through Botany MCP
or the Vegetation panel. Load both green/fresh components at the same transform.
The isolated demo produces `artifacts/lawns/authoring.scene.json`, retaining all
editable sources, plus `templates.json` and `library.scene.json` for shared
static meshes. Static copies retain their family/variant mapping in
`placements.json`, while the canonical sources retain authoring curves.

## Review and validation

`artifacts/lawns/review.html` contains the shared-light comparison, three variants
per family, close views, top/profile views and individual family patches. Each
family patch has 49 modules on a 7×7 layout, spanning about 1.68m per side before
blade overhang. The cushion layout adds seeded offset, rotation, scale and
height variation; both carpet layouts meet edge to edge. Ground roots are embedded
below the support plane, including the smallest scaled Japanese modules.

Raw native images are 1800×1100 (family/variant images 1600×1100). Files suffixed
`-preview.png` explicitly use spatial supersampling: native 2048×1252 rasters
(`-ssaa.png`) downsampled to 1024×626 with macOS `sips`. This is still-image
presentation, not runtime antialiasing. Native capture is limited to 2048 pixels
per dimension. The raw images remain alongside the previews.

The actual compiled blade meshes pass directed-edge closure and orientation
checks. Sources regenerate exactly, every component changes under a different
seed, IDs are unique per variant, and the Japanese centre/rim height ordering is
checked from the authored paths. Saving and reloading the profile scene produces
a byte-identical native PNG. Machine-readable evidence is in `report.json` and
`source-validation.json`; `command-schema.json` records the MCP schema used.

The three-way comparison contains 19 assets including the ground, 295 instances
and 147 plant objects. Shared unique geometry totals 496,132 vertices /933,890
triangles; instancing those variants totals 15,253,506 triangles before view-dependent
selection. These are detailed authoring/close-view assets. Instancing shares
geometry storage but does not remove their raster or shadow work. No frame-rate,
1km coverage budget, automatic LOD, wind simulation or renderer improvement is
claimed here. Large-area deployment needs a separate density/representation
budget and runtime measurements.

The procedural shapes now separate the wider even carpet, the fine rounded
cushions and the fine flat carpet. The comparison order is esmeralda, japonesa,
japonesa_tapete from left to right. The Japanese patch is still more regularly spaced than the photographed
garden; the native raster also aliases subpixel leaves. Colour and lighting are
controlled review choices, not calibrated matches to either photo. Neither an
exact photo reconstruction nor human artistic acceptance follows from topology
and persistence checks. Previous botanical libraries, the world scene and the
renderer are outside this change.

## Botanical clarification and flat option

The user asked whether the pictured cushions were caused only by the ground.
[Clemson Extension](https://hgic.clemson.edu/factsheet/zoysiagrass/) explicitly
describes Zoysia tenuifolia as forming small clumps with a puffy appearance;
[UC IPM](https://ipm.ucanr.edu/TOOLS/TURF/TURFSPECIES/zoysia.html) describes it
as fine-leafed dwarf ground cover. These are compatible: it is a low grass,
and the cushion habit can come from its growth. The reference alone does not
reveal the soil profile beneath it. The extra flat option follows the user's
requested appearance while preserving the cushion preset.

Source checks compare the flat carpet centre and outer annulus: mean tip heights
differ by less than 2% for the three shipped variants, versus the strong radial
height difference in the cushion. The same-seed and alternate-seed checks cover
all 18 sources; all 147 patch origins remain embedded below the support surface.
Human artistic acceptance and large-area runtime performance remain separate.

Native validation build: `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|d8dec4c557b67a8c`. No engine build was created by this asset task.
