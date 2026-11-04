# Third-party materials

The root Ignis Engine Commercial Source License 1.0 covers the original Ignis
work offered in this distribution. It does not replace the licenses or ownership
of third-party materials, or revoke earlier grants for previously published copies.

| Component | Location | License and provenance |
| --- | --- | --- |
| Boolmesh 0.2.2, mesh Boolean kernel (unmodified Cargo dependency) | `Cargo.toml` / `Cargo.lock` | MPL-2.0; copyright Saki Komikado; pinned source package https://crates.io/crates/boolmesh/0.2.2 and upstream https://github.com/komietty/boolmesh. License retained at `licenses/boolmesh-MPL-2.0.txt`. The dependency retains its own license. |
| MakeHuman graphical data: mesh, targets and rig data | `assets/characters/female-base/` | CC0-1.0; upstream revision, per-file hashes and original notices in `provenance.json`, `LICENSE.md` and `LICENSE.ASSETS.md`. No MakeHuman program logic is bundled here. |
| img2threejs camera-fitting and tapered-sweep subset | `scripts/vendor/img2threejs/` | Apache-2.0; upstream authors, revision and changes in `NOTICE.md`; full license retained in `LICENSE`. |
| Three.js, website renderer | `website/dist/assets/vendor/` | MIT; copyright and license in `LICENSE`. The website renderer is separate from the Rust engine. |
| Space Grotesk, website font | `website/dist/assets/space-grotesk.ttf` | SIL Open Font License 1.1; copyright and license in `font-license.txt`. |

Rust dependencies are recorded in `Cargo.toml` and `Cargo.lock`. They retain
their upstream licenses; obtain the applicable notices when redistributing
binaries. The root license is not a blanket license for the dependency graph.

Reference inputs without established redistribution permission are omitted from
the public source export. Some research documents and scripts describe local
inputs or generated `artifacts/` that are not included. Do not infer a license
for external material from those descriptions.

Games, scenes and other original content you create with Ignis belong to you,
subject to any third-party material you choose to include. Distributing Ignis
code or binaries in your product still requires compliance with their licenses.
