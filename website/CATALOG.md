# Asset catalog and native WebGPU previews

The site exposes `catalog/` on both existing hosting destinations. Content lives
independently in https://github.com/zerobytes/ignis-catalog. Its pinned committed
index is mirrored into `dist/catalog/data`, with 20 editable material sources,
11 tree records and full-source download links, CC BY 4.0 license, optimized
native-capture thumbnails and GPU preview packets.
There is no additional service, account, paid storage or render server.

## Renderer contract

`catalog/renderer.js` hosts **the exact native Ignis WGSL module** in browser
WebGPU. The material path uses `vs_main` / `fs_main`, native 64-byte vertices, 464-byte globals,
1392-byte material records and the original procedural word buffer. Native Rust
`materials::preview::encode` calls the same validator and `recipe::pack` used by
the desktop renderer. It does not bake material maps or reimplement the mineral
functions in JavaScript. Shared shader inputs and packet/source/thumbnail SHA-256
are recorded in the site snapshot; the browser verifies shader/packet hashes.

The host presents opaque materials on a sphere, cube or plane, and actual static
meshes framed by their bounds (mesh ABI below), with directional light, native
diffuse/specular response, procedural height/bump and recipe parallax/self shadow. Scene shadow maps, IBL, atmospheric/planet views,
transparent composition, gameplay, physics, animation and the editor are not
enabled. Supported metadata does not mean every engine subsystem is web-ready.
The native shader's precedence parentheses were made explicit for strict browser
WGSL parsing; multiplication/XOR evaluation order is preserved.

WebGPU and a secure context are required for interactive preview. Unsupported
devices, missing adapters, compilation/load failures, hash mismatch and device
loss retain the native image and editable download. Preview can be retried.
No static image or Three.js composition is labeled as live Ignis output.

## Generation

In an integrated native engine checkout with the material library available:

```sh
cargo build --release --example export_web_materials
python3 scripts/build_web_catalog.py --catalog /path/to/ignis-catalog
```

The builder validates a clean catalog checkout and verifies every source hash.
All twenty gallery material definitions must match the catalog semantically;
thumbnails come from their existing native captures, with provenance hashes.
Only licenses, listed material sources, captures and verified native GPU packets
enter Hosting. Large editable tree packages remain in versioned Releases. Use one current exporter executable and the normal shared Cargo cache;
no engine application copy or historical build directory is generated.
Stable exclusive staging is removed on success/failure; generated data replaces
the previous current snapshot only after export/checks pass.

The deployment repository may precede the integrated native material work. In
that case the committed web snapshot is independently runnable, while regeneration
requires the integrated native checkout above. `renderer_source.local_snapshot`
is a local parent revision, not a claim that every working-tree input is already
on public GitHub; exact source hashes identify the exporter/shader inputs.
Publishing the website does not publish unrelated native commits.

## Cost and interaction review

| Area | Contract |
| --- | --- |
| Frequency | GPU initializes on the first preview action. One frame after asset, camera, shape, light, resize or renewed visibility; no continuous animation loop. |
| Allocation | One device/context with cached material and static-mesh pipelines. Replace material/program/VB/IB resources on changes; destroy old buffers and resized depth targets. Page exit/device loss disposes the host. |
| Data | Exact native ABI; bounded IGWP1 packet, little-endian words, SHA-256 and original source version. No scene serialization or recipe parsing per frame. |
| Repetition | One shared shader and context across all cards. Cached compiled pipeline, retained geometry until a shape/material-domain change. Static lazy thumbnails have no per-card renderers. |
| Limits | 500 records at admission, 1 MiB shader/material-packet budget, 560k preview pixels, DPR capped at 1.5; current sphere 2665 vertices / 5120 triangles. These are preview budgets, not engine limits. |
| GPU / IO | Two storage buffers for one material; event-time fetch/upload only, no GPU readback or network in draw. The material portion is below1 MB. The assembled catalog data is about201 MB, including200.4 MB of mesh previews loaded only on demand. |
| Concurrency | Abort obsolete asset fetches; generation token prevents an old selection taking over. One queued RAF, no worker/engine locks. Native export runs outside the interactive host. |
| Invalidation | Locale/search/filter changes update DOM without reinitializing GPU. Camera/light reuse data. Hidden/offscreen view does not schedule draws. |
| Validation | Native pack equivalence/rejection tests; native GPU recipe regression; all28 browser packets drawn, eight native scene imports and Khronos GLB validations, controls, filters/EN/PT, narrow layout, fallback and paired served-hash verification. |
| Retained costs | First shader compilation, visible fragment work from native recipes, full source/index metadata, CPU browser geometry and decoded thumbnails. No FPS, total GPU-memory or physical-phone performance claim. |

## Delivery and continuation

Use the existing paired `website/deploy.py` workflow after preparing, committing
and publishing only this website increment on the canonical public history.
Both Firebase Hosting and GitHub Pages must serve the same version and file hashes.
The paired deploy reuses a successful or active Pages workflow for the exact
public commit. A newer queued duplicate cannot hide an already successful run;
served-file verification still confirms the actual bytes on both destinations.
The catalog's assets retain CC BY 4.0 attribution. The shader and host remain under
the separate Ignis engine license, included in `data/engine-license.txt`.

Next content kinds need their own native export/loader contracts and budgets;
the material host is not a universal scene/character/plant player. New catalog
versions require an explicit snapshot refresh, validation and paired publication.

## Meshes and trees

The catalog also admits static meshes. The first eight are the five canonical
botanical trees and three golden birch variants. Large payloads live in pinned
`ignis-catalog` GitHub Releases: one ZIP and one compressed preview per asset.
Git tracks metadata, SHA-256 digests and native-capture JPEGs only. No account,
upload service, database or paid storage plan was added.

Each ZIP includes the original `.ignis-botany`, a full editable Ignis scene with
its source binding and native materials, full-geometry GLB, checksums, attribution
and CC BY 4.0. Birch includes its original `.ignis-material` files. GLB uses
representative portable PBR colors; Ignis procedural patterns remain in the
native scene/materials. Reference photographs and unfinished game assets are
excluded. Source provenance can refer to an unpublished local engine snapshot;
the complete licensed asset is supplied in the release itself.

`examples/export_web_meshes.rs` compiles the original source for the download and
native thumbnail, then recompiles a **preview-only** recipe at axis8 / sides6 /
blade4. This keeps organs, positions and appearance parameters while reducing
native tessellation. Palmate outlines retain the compiler's native 48 contour
samples. Full and preview vertex/triangle counts and bounds are separate metadata.
This is a static presentation, not a biological simulation or a full web editor.

`src/meshes/preview.rs` packs the actual native static position-coordinate stream
and original GPU material programs. IGMP1 has a 16-byte header (magic, version,
padded JSON bytes, binary bytes), JSON metadata and little-endian binary data:
1392-byte material records, native program vec4 words, position/normal vertices
at24 bytes, u32 indices and80-byte native instances. The browser invokes the
existing `vs_shared_position_coordinates` and `fs_main` shader entry points,
frames the world bounds and draws trunk/foliage parts with their original tints
and materials. It does not substitute spheres or reimplement the shader.

Compressed `.igmpz` previews are fetched only after activation, verified by size
and SHA-256, decompressed with a bounded `DecompressionStream`, then verified
again before GPU upload. Limits:64 MiB compressed per asset,192 MiB decoded,
128 parts and384 MiB total assembled runtime snapshot. These are portal limits,
not engine limits. Only one selected mesh is resident on the GPU; requests are
cancelled on selection changes and frames are submitted only after interaction,
resize or visibility changes. Material previews retain their1 MiB packet limit.

`website/runtime_assets.py` assembles previews from immutable release URLs into
ignored `website/dist/catalog/data/meshes/`. Both the Pages workflow and paired
deployment use the same hash-pinned manifest. The public Git check verifies all
tracked site files plus every declared runtime file; the release fingerprint and
post-deployment verification include **all** assembled bytes on both hosts.
`python3 scripts/build_web_catalog.py --catalog ../ignis-catalog --mesh-cache
artifacts/catalog-meshes` can reuse locally verified release payloads. A free
hosting allowance is finite: downloads and GPU activation remain explicit;
no unlimited bandwidth or performance guarantee is implied.

Cuihua1.0.0 adds three original green fruit-tree variants. Their complete downloads
include authored growth grammar9 sources and periodic cohorts; the browser preview
and GLB remain static. Fruit uses an apple-like approximation, not an exact
Sunsettia sculpture.
