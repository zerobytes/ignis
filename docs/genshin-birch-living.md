# Living adaptation of the existing Birch assets

The three Birch sources already exist. Cuihua is still the next species to
create; this increment does not invent a second pre-existing collection.
Original files/materials in `assets/botany/genshin-birch` remain unchanged.

## Author and open

```
python3 scripts/adapt_genshin_birch.py
python3 scripts/adapt_genshin_birch.py --prepared --open
```

The authored timing recipe is `examples/genshin-birch-living.recipe.json`.
Current outputs replace `artifacts/genshin-birch-living`: three birth-state
`.ignis-botany` files, adult `gallery.scene.json`, mixed-age `ages.scene.json`,
and native captures. The first scene retains all three individuals at20years;
the second shows2/8/20years. Age captures use different camera distances to
make seedlings visible. No publication/catalog package is modified.

## Native data

Growth grammar9 adds optional `Definition.authored`: an immutable adult source,
its authoring settings, independent height/crown/radial time curves, organ-order
expansion timing and named primary production sites. The existing canonical
individual/production data stores time, pruning and harvest records. The adult
template uses source-organ IDs, not a baked mesh or Python time evaluator.

`Botany.adopt_living` takes a prefix, timing and birth day. It consumes the native
static source already loaded in the editor, so a large adult source does not
need to be sent through the2MiB command envelope. The Vegetation panel offers
“Adaptar fonte adulta ao desenvolvimento vivo”. Adoption refuses existing
manual/sculpt/pruning/history edits instead of discarding them. Its initial
implementation accepts a single upright root axis, no prior development/root
system, and at most32 primary woody production sites.

Example command:

```json
{"op":"botany","operation":{"kind":"adopt_living","prefix":"birch_0","born":0,"timing":{"reference_years":20,"seedling_height_m":0.55,"height_time_years":35,"crown_time_years":35,"radial_time_years":35,"branch_interval_days":200,"order_delay_days":120,"expansion_days":730}}}
```

The native evaluator grows the original curves, exposes branches by order and
keeps their IDs. Leaves accompany the first stem and branch frontiers before
adult sprays emerge. Millimetric collar requirements of the existing organic
compiler determine the earliest supported fine-branch radius; fine axes wait
for that date rather than being artificially fattened or bypassing integrity
guards. Fork siblings share their emergence date. Very fine twigs may therefore
appear later than their nominal order date.

Height, crown and radius use separately normalized exponential curves: the
configured20year reference reproduces adult dimensions; later years continue
approaching each curve's asymptote. This is an authorial development model, not
field-calibrated birch biology. Potential topology is the finite adult graph;
unbounded new branching beyond that template is not implemented.

Poda uses the original primary IDs (`branch00`, etc.) and remains a canonical
production-site intervention. Optional wood history records loss marks and
supports causal age previews using the existing native operations. The three
adapted sources enable that layer with no invented historical damage. Ordinary
materials remain preserved during publication/age previews.

The reproduction scheduler remains available to authored grammar9, but Birch
profiles disable the generic demonstration flowers/red fruits. Birch-specific
catkins/seed geometry is not implemented. Seasonal foliage, automatic world-clock
updates, dropped rigid bodies and individual forest-instance simulation are
separate features; this adaptation does not imply them.

## Compatibility and inspection

The optional field is absent in old definitions; grammars1..8 retain their paths.
Sequoia-specific volume, material and maturity-loss rules are explicitly bounded
to their existing grammar versions. Existing source/scene envelopes and budgets
remain in effect. The adult source is shared in memory and serialized inside the
living definition for self-contained offline persistence.

Compilation/static analysis and direct use of the native MCP/editor are the
validation workflow. No automated tests, cases or verification scripts were
added or run. One existing in-source test fixture only receives `authored:None`
for structural compatibility with the extended Rust definition; its case and
assertions are unchanged.

The adult inspection retained8,329 organs and7,848 blades in each of the three
variants, matching the original authored population. The shared time constants
start at35years for height/crown/radius so later thickening remains coherent
with branch spacing; they remain independently configurable.
