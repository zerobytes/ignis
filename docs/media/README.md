# Public image provenance

The current READMEs use creator-selected native captures already prepared for
the website: Horizon as the lead image, with sunset and cottage as supporting
views. The optimized WebP files are copied byte-for-byte from the website, with
no new crop, repainting or generative alteration. Original PNG captures remain
unchanged. These are engine output, not the website's illustrative Three.js art.

| Current image | Website copy | Original capture | Size |
| --- | --- | --- | --- |
| `horizon.webp` | `website/dist/assets/engine/horizon.webp` | `website/media/horizon-forest.png` | 1600×890, 176,888 bytes |
| `sunset.webp` | `website/dist/assets/engine/sunset.webp` | `website/media/horizon-sunset.png` | 1600×1034, 23,266 bytes |
| `cottage.webp` | `website/dist/assets/engine/cottage.webp` | `artifacts/cottage-normals/artifacts/joined-hero.png` | 1536×1024, 159,224 bytes |

The landscape captures are planar rendering studies, not the experimental
planetary runtime. The architecture view is an authored native study; these
images do not promise universal performance or exact reproduction from a
particular public build. Source hashes and captions are recorded in
`website/media-provenance.json` and `website/dist/engine-media.js` in the
authoring checkout; only the selected optimized copies are needed here.

## Earlier captures

These earlier native outputs remain available for technical documents and the
website's existing provenance records, but no longer lead the READMEs.

| Public image | Original generated output | Context |
| --- | --- | --- |
| `terrain.png` | `artifacts/expanded-world/presentation-panorama.png` | Planar expanded-world terrain renderer; see `docs/expanded-world.md`. Not the planetary runtime. |
| `botany.png` | `artifacts/botany-collar/adult-tree.png` | Full native fig-tree bake with roots/collar; see `docs/botany-collar.md`. |
| `editor.png` | `artifacts/editor-design-wide.png` | Native editor sculpting workspace; historical UI capture, not a claim that every current menu is identical. |

The root commercial source license and attribution notice apply to these
original project captures as offered in this distribution. Earlier grants are
not revoked. The README wordmark comes from the project's existing brand
assets in `website/dist/assets/`; those assets are distinct from engine output.
No visual, performance or platform acceptance is inferred beyond the caption.

`website/dist/assets/social-preview.png` is an original code-drawn promotional
diagram (1280×640), not an engine screenshot. It illustrates the shared API.
