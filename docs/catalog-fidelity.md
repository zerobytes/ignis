# Native asset fidelity in the catalog

The catalog preview must represent the saved asset. Publication refuses reduced
mesh previews and representative-color substitutions. Material samples use their
original licensed source and native GPU material packing. Tree previews come
from the exact native scenes inside the hash-pinned published source ZIPs.
They do not regenerate a botanical recipe, change tessellation, remove organs,
flatten materials or repaint the coating.

`export_catalog_assets` and `meshes::preview::prepare_exact` transport independently
loadable IGMP2 parts. Every authored triangle is retained. Unused vertices are
removed and indices remapped without changing positions or corner attributes.
Smooth normals, rest coordinates, material projection weights, intrinsic moss
edge weights and the complete packed material program/fixed mip samples survive.
Anatomical wood uses the native renderer's face-coherent metric charts and its
pipeline specialization. Equal chart vertices are deduplicated by every float
bit; different charts remain separate. No surface color is baked into vertices.

The web host uses the native surface/opacity shaders and material ABI, with
explicit pipeline layouts, full material buffers and the original instance
transforms. A browser derivative-uniformity diagnostic directive accommodates
native material branches; it does not replace material evaluation. Prepared
charts and fixed/material constants avoid compiling/evaluating unused anatomical
search paths. Local static previews have no growth or physics loop.

Inspection lighting uses the native prefiltered sky/horizon/ground environment
and DFG integration, a movable directional light, cached 2048px mesh shadows and
4x MSAA. The canvas follows device pixel ratio up to 2 and a four-million-pixel
working budget. This is an asset inspection environment, not the lighting,
scenery or post-processing of a particular game scene. Existing camera close-up,
surface focus, pan, touch and keyboard controls remain available.

## Authoring and publishing

1. Build the current release exporter and engine. Do not regenerate source ZIPs
   merely to update their web preview.
2. Run `python3 scripts/build_exact_catalog_previews.py --catalog PATH`.
   `--only ASSET_ID` supports focused authoring. The builder verifies the source
   archive against its existing manifest, extracts the saved scene, invokes the
   native exporter and compresses each part. Receipts pin the archive and exporter.
3. Review the assets in the browser. Native triangle-count integrity and source
   hashes are required in catalog manifests; source, build and visual acceptance
   remain separate evidence.
4. Publish the new preview parts in a pinned catalog Release, commit/index the
   updated catalog records, then build the portal with the verified cache.
5. Publish the same small portal snapshot to Firebase and Pages. Large immutable
   runtime payloads use the canonical Firebase origin with public read-only CORS;
   Pages validates their pinned declarations instead of duplicating the entire
   library in its static deployment. Both websites load the same verified bytes. The mirror waits for the matching
   canonical version before deploying, so it never references unready new payloads.

The size is shown before activation. Parts are downloaded only after the user
opens an asset; size/hash checks precede GPU upload. If a source or device exceeds
the supported transport limits, export/loading fails visibly. It must not silently
substitute colors or simplify the source. Browser transfer/working-memory limits
are distinct from authoring budgets and game runtime performance.

The portable GLB files in older source packages still have their documented
compatibility limitations. The native scene remains the original asset; replacing
its faithful web preview does not modify those immutable source downloads.

The shared payload origin also keeps the mirror below the [GitHub Pages published site size limit](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits).

Payload generation requires a compatible native authoring build (recipe34 / scene138 for the ancestral sources). Publishing the browser snapshot does not publish unrelated engine development history. Its shader/build and catalog source hashes remain explicit.
