# Native living-plant editor integration

This increment connects the [living-plant contract](living-plants.md) and its
small Rust grammar to native scene ownership, commands, the Vegetation panel,
source files and persistence. It is an explicit-time editor prototype. It does
not yet advance automatically with gameplay time or produce physical dropped
fruit/branches. Existing authoring physics rejects living reproductive geometry.

## Reproduce

```sh
python3 scripts/demo_living_plant.py
python3 scripts/open_living_plant.py
# Fresh specimen for testing the age slider:
python3 scripts/open_living_plant.py --from-birth
```

The script owns a headless native MCP process. It does not implement a Python
growth grammar: `create_living` and `living` commands generate all organs and
derived geometry inside Rust. It replaces stable review paths under
`artifacts/living-plants`, including `gallery.png`, `review.html`, individual
stages, `living.scene.json` and `editor.png`. Existing human scenes/windows are
preserved. The source `assets/botany/living-specimen.ignis-botany` starts at birth
and can be loaded through the normal Vegetation source controls.

In the Vegetation panel, open **Nova planta viva · experimental**, configure the
prefix/seed, node interval/count/spacing and reproductive schedule, then create.
The selected living specimen exposes its day, explicit advance, active
reproductive units, individual harvest buttons and persistent site pruning.
These controls dispatch the same operations as MCP; they do not maintain a
second private plant state. The prototype uses authored integer game days, not a
calibrated botanical growth rate. The age slider displays elapsed time since
birth in years of exactly 365 game days, quantized to one day. Its adjustable
horizon defaults to two years. Releasing the slider (or editing its number)
commits a forward advance; dragging does not regenerate every frame. Undo
(⌘Z) restores earlier authoring snapshots, including interventions. It is not
biological reverse time. The current demonstration grows a node every 10 days
and repeats reproduction every 100 days; these accelerated values are test
parameters, not a realistic species calendar.

## Commands

`examples/living-plant.recipe.json` contains the complete initial definition.
Grammar version1 is explicit and validated. Commands go through `botany`:

```json
{"op":"botany","operation":{"kind":"living","prefix":"living","action":{"kind":"advance","time":90}}}
```

```json
{"op":"botany","operation":{"kind":"living","prefix":"living","action":{"kind":"harvest","fruit":{"site":"node0","cohort":0,"slot":0}}}}
```

```json
{"op":"botany","operation":{"kind":"living","prefix":"living","action":{"kind":"prune","site":"node1"}}}
```

`create_living` takes `definition` and absolute `born`; `living/inspect` returns
canonical state and currently attached reproductive units. Creation, advance,
harvest and prune are atomic publications and require the local editor paused,
as other botanical publication does. They do not manipulate a running gameplay
world. Loading a source/scene or using undo can restore an earlier authoring
snapshot; explicit biological advance itself cannot rewind time.

## State and geometry

`Recipe.living` contains the grammar definition, chronological state, stable
site identities and birth times, removed sites and individual harvest records.
The source is regenerated from that canonical state and compared at admission;
a mismatching graph, impossible site birth, unsupported dead flag, invalid
harvest or unsupported grammar version is rejected. `Recipe.pruned` is not a
second pruning authority for living sources. The living prune action removes
the branch/leaf source subgraph and records the site intervention instead.

The grammar1 source is pinned to source8 independently of future global botany
versions. This avoids regenerating a saved source with a new source-version
number simply because another botanical authoring feature advanced the reader.
The current reader also supports recipe/source9 from the parallel palm-frond
work; that feature and its UI/geometry are preserved.

Wood/leaves retain their existing semantic compiler. Three additional derived
parts hold flowers, growing green fruit and ripe red fruit. Closed ellipsoids
form a simple demonstration flower/fruit shape; they are not a species model
or the final Cuihua assets. The fruit follows the evaluated supporting branch
frame. Empty categories use the same tiny-placeholder convention as existing
empty native foliage. Disappearance at expiry is visual only, not physical
detachment. Zero-length phases can be crossed without producing a visible
sample; the scheduler currently exposes state rather than a world event queue.

The derived parts share ownership with the recipe. Manual geometry edits to
reproductive parts cannot be transported and are explicitly refused on
preserved regeneration. Ordinary native material preservation remains active.
Authoring wood/leaf offsets still use existing semantic addresses; loss of an
adjusted address can refuse pruning/regeneration instead of discarding it.

Admission limits: 32 growth nodes, 128 attached reproductive candidates and 4,096
harvest records per individual; times/profile durations at most 1,000,000,000
game days. Quota failures are explicit and preserve the last published state.
Harvest-history compaction and long-running world scheduling remain future
work. Stem diameter and organ shapes still have authored constants.

## Versions and compatibility

- Living state requires recipe8/source8 or compatible later versions.
- Scene121 is required when living state is present; a down-versioned scene
  carrying it is rejected. Source/recipe1..7 remain readable and keep their
  legacy behavior when the optional field is absent.
- Replay131 records the existing scene-edit transactions with the new canonical
  state and geometry. Earlier replay versions remain explicitly incompatible.
- Public content16/protocol5 and portable PSV1/checkpoint3 envelopes are
  unchanged: no new public wire field or physical-engine handle was introduced.
  Scene embedded in these persistence paths still carries its own version.

The increment changes the scene/source contracts rather than hiding plant
history in material names, generated filenames or an untracked sidecar.

## Validation gates

`tests/living_botany.rs` exercises actual Service commands: birth, increasing
organ count, flowers, green/ripe fruit, harvest, prune, repeated-action/rewind
rejection without mutation, Scene/source/portable roundtrips, replay matching,
undo/redo, second-cohort production, version/derivation guards and refusal to
overwrite edited reproductive geometry. A geometry audit checks opposite edge
incidence and positive signed volume of active reproductive parts.

`scripts/demo_living_plant.py` performs the same operations through the actual
native executable and records build identity, timings and raw captures. It
checks exact PNG equality after Scene, portable, source and gallery roundtrips.
The storyboard shows the same definition/seed evaluated at days40/80/90, not
three historical moments coexisting in one simulated global calendar.

Software success and repeatable pixels are separate from visual acceptance,
performance for a forest and a complete living gameplay ecosystem. Environmental
responses, cambium/root growth, leaf cohort turnover, general species grammars,
automatic biological ticks, physical cuts/drops, fire and death remain open.

Validation on macOS/Metal build `45de907307a10d89`: 43 focused Rust tests passed
(5 core, 20 existing botany, 2 living integration, 4 directional-light CPU,
1 content, 1 portable-save, 5 replay-inspection and 5 scene-edit tests). Three
unrelated directional-light GPU tests remained ignored; the actual living
specimen was rendered through native MCP. Clippy lib/bin plus the living test
passed with warnings denied. A broader Clippy run also found existing warnings
in the object-impostor and material-preview test modules; those unrelated files
were preserved. The initial lighting test fixture was updated to expect current
Scene admission version rather than its historical feature-introduction version.

All five native persistence checks in `verification.json` passed: Scene,
portable and source PNG equality, gallery PNG equality and replay matching.
Observed single-command wall times ranged from about 0.3 ms at birth to 5.7 ms
at fruiting for this small specimen; these are not frame times or a population
benchmark. A separate native window was opened at day190 with the Vegetation
controls; `window.json` identifies its process. Human interaction/acceptance is
separate from the captured native editor rendering.

One current executable is retained at `target/release/ignis`; the task's unused
debug executable was removed after validation. Compiler/test incremental data
and other running applications were preserved; see `build-retention.json`.
