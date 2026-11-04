# Living broadleaf exemplar

Grammar2 in `src/botany/growth/tree.rs` is a native seeded broadleaf tree,
separate from the deliberately small grammar1 diagnostic. It consumes the same
persistent individual time, site births, pruning and harvest state. Grammar1
sources and evaluation remain unchanged. The definition's explicit version
selects the grammar; source/recipe8 and the existing scene admission are reused. Publication promotes
the recipe to the current reader version (10); the generated source stays8.
Older readers reject definition2 rather than silently evaluating it as grammar1.

The exemplar `examples/living-tree.recipe.json` uses one year per main bud,
ten primary branches and a six-metre mature height. Three woody branching
orders, curved axes, progressive branch extension, secondary/twig birth delays,
seeded angular asymmetry and persistent leaves form a broad crown. Trunk radius
continues developing after height approaches its authored limit. The schedule
uses 365-day years, annual flowering and a five-year reproductive threshold.
These are authored demonstration values, not an empirically calibrated species.
Flowers/fruits still attach to the primary production sites, not every twig.

The tree reaches its authored structural limits; it does not add orders forever,
simulate senescence or regenerate seasonal leaf cohorts. It is not the final
Cuihua asset. Existing physical gameplay/environment limitations still apply.

```sh
cargo build --locked --release --bin ignis
python3 scripts/demo_living_tree.py
python3 scripts/open_living_tree.py
```

Review images at 2/5/10/12/20 years and a native scene are replaced under
`artifacts/living-tree`. The launcher opens an independent **editor** window at
twelve years, with the Vegetation panel selected and a 30-year slider horizon.
The source asset is saved at birth, retaining future growth potential. Native
Rust generates all geometry; Python only drives commands and captures. Existing
human windows/scenes are preserved.
