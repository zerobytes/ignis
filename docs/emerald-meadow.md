# Emerald meadow: 1 km × 1 km

EAF-218 is an independent finite scene using the approved emerald grass sources
from EAF-216 and the existing experimental IGFLD1 field renderer from EAF-215.
The terrain, grass sources and captures are native Ignis data. Renderer and
botanical-library sources are unchanged by this composition.

## Open and reproduce

From the repository root:

```sh
python3 scripts/build_emerald_meadow.py
python3 scripts/verify_emerald_meadow.py
python3 scripts/open_emerald_meadow.py
```

The launcher verifies the scene and field SHA256 values in
`artifacts/emerald-meadow/bundle.json`, binds its own field resource and opens a
new first-person window. Click captures the mouse; WASD moves, Shift runs,
Escape releases the mouse and F3 displays the existing frame-rate meter.
It preserves other windows. The launcher itself was statically checked; the
native window, bundle binding, free-camera presentation and walking/collision
were exercised by the measurement and verification workflows.

**Open through this launcher.** `world.scene.json` contains terrain and the
walking actor; grass lives in `ground.ignis-field`. The existing diagnostic field
is a process/session resource selected through `IGNIS_HORIZON_FIELD`, not a
persisted Scene component. Loading the Scene alone omits grass. A normal
SaveScene does not include or safely rebind this resource to another scene.
The bundle and launcher make this specific pairing explicit, without changing
global persistence formats or claiming production MCP/UI parity for fields.

Build phases can be repeated individually with `--stage terrain`, `field`,
`templates` or `capture`. `templates` preserves every compiled planting height
while replacing only the six approximate geometry representations. Outputs use
stable paths; no engine rebuild or extra app bundle is produced. A local
native release library mismatch prevented the separate draw-timing diagnostic;
its temporary link inputs/executable were removed. Current measurements use
the actual native executable instead of altering a Scene version for a stale
library.

## Terrain and coverage

The terrain footprint is exactly [-500,500]m on X and Z: 1,000,000m². Native
Terrain2 raises, three broad ridges and low-amplitude noise form rolling hills
and shallow valleys, with heights 5.438–28.200m. The grid has 251×251 vertices,
4m spacing and 125,000 triangles. The same complete mesh supplies collision.
There are no roads, water, flower clearings, exclusions or camera-specific
planting zones.

The field has an 8000×8000 planting lattice at 12.5cm pitch, stored as 500×500
blocks of 16×16 sites. All 64,000,000 root sites are occupied. Every 50×50m audit
cell contains 160,000 sites. This is an occupancy check, not proof that every
surface point/pixel is covered by an individual grass triangle. Oblique, close
and aerial views provide separate visual review. The earlier 25cm candidate
left visible gaps; uniform overlap was increased across the entire square.

The source is `esmeralda_0` from the verified native lawn library. Blade width,
height variation and curvature come from its native compiled stations. Mature
and fresh leaf geometry is combined into a single green field with patch tint
variation; separate per-leaf source colours are not carried by IGFLD1. The source
library remains byte-identical. The logical full-source population would contain
49.152 billion blades; this is not the number drawn or resident as geometry.

The authored field representations select 384/192/48/12/3/1 blades and use
1,536/384/96/24/6/2 thin-ribbon triangles per selected module. They are approximate
representations of native stations, not the closed high-detail source mesh or
an image-error certificate. The existing renderer chooses scale/stride by
projected resolution and frustum-tests spatial blocks. No source-name special
case, hard distance cutoff or new engine renderer code is introduced here.

The recipe counteracts the diagnostic shader's stride-based X/Z enlargement.
This retains physical grass dimensions at distant scales and limits the effective
template radius to 0.1812m, avoiding distant templates inflating every near-block
bound. Sparse far representatives still lose fine appearance, and raw captures
show subpixel aliasing and scale transitions. Large-scale quality equivalence
has not passed. The field package is 256,196,696 bytes; its height payload alone
is 256,000,000 bytes, before GPU command buffers and renderer overhead.

Root datums use the persisted terrain's exact triangle diagonal and an embedded
4mm offset. Independent Grid interpolation of 4,096 distributed root datums
found maximum error below 1 micrometre. Each blade remains in the module's local
rest shape: IGFLD1 does not warp each blade root onto the slope or provide grass
physics. Datum correctness is not a proof of universal individual-leaf contact.

## Native validation and presentation

`verification.json` records bounds, complete occupancy, sampled datum error,
finite template geometry/radius, bundle hashes and a 7.945m grounded KCC walk.
Native save/load produces a byte-identical close PNG in the same bound session;
this does not mean the Scene persisted its field binding. `command-schema.json`
records the actual MCP schema. Python compile and focused artifact checks pass.
No Rust engine changes, global-format changes, prior scene edits or approved
botanical-source changes belong to this task.

`review.html` contains native 1800×1100 panorama/hills/walk/close captures.
`*-preview.png` files are those native rasters downsampled to 1024×626 with macOS
`sips`; this is still-image presentation, not a runtime AA feature.

The actual window was measured at 2200×1520 on the Apple M4 Pro. The existing
counter reports present-call cadence over up to 1 second/512 intervals. Ten fresh,
overlapping reports per camera were collected after 2 seconds of warm-up. The
following values are medians of those reports, not isolated GPU timings,
scanout, independent benchmark runs or a universal FPS guarantee. No-vsync was
requested; the reported mode string is `AutoNoVsync (requested)`. Other desktop
processes were preserved and listed in the local report.

| Camera | Observed median FPS |
| --- | ---: |
| walk | 14.03 |
| hills | 22.34 |
| panorama | 31.24 |
| close | 9.84 |

Measurements and raw samples: `artifacts/emerald-meadow/presentation.json`.
Native capture build: `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|677e88e92de00145`.
Native presentation build: `aarch64-apple-darwin|release|rustc 1.97.1 (8bab26f4f 2026-07-14)|677e88e92de00145`.

The scene is still heavy near the ground. Performance is not declared solved,
and human visual/runtime acceptance remains open. Engine-wide optimisation,
field persistence, safe generic MCP/UI field editing, grass dynamics and broad
quality certification remain outside this authored-scene unit.
