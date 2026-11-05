# Historical graphics research

This directory preserves the 244 research files from `codex/graphics-research`
at `8a5e66454a151ba54b26891b98a963e005fb018b`. Their contents are unchanged;
`source-manifest.json` records the original paths and Git blob identifiers.

These experiments were excluded from the October 29 engine consolidation. They
are collected here following the November 5 request to bring branch work into
the public `main`, without importing private Git ancestry or replacing the
current renderer. The editor, file browser, native meshes, native plants,
image reconstruction and local transaction work were already incorporated.
The emerald meadow branch's final change is already present as `d3d1841`.

The original `docs`, `examples` and `scripts` hierarchy is retained inside this
directory. This is historical source, not a supported Cargo example package or
an enabled runtime feature. The experiments and their existing checks target
their original engine revision and may require API adaptation to run against
the current engine. Generated artifacts and external input assets are not
included. Historical measurements describe those experiments only.

No research checks have been added to the normal verification runner, and no
new automated tests were authored for this consolidation. See the current
[development policy](../../docs/development.md) and [feature guide](../../docs/features.md)
for the maintained engine and its supported entry points.
