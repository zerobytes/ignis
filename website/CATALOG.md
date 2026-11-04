# Asset catalog and native WebGPU previews

The site exposes `catalog/` on both existing hosting destinations. Content lives
independently in https://github.com/zerobytes/ignis-catalog. Its pinned committed
index is mirrored into `dist/catalog/data`, together with the 20 editable sources,
CC BY 4.0 license, optimized native-capture thumbnails and GPU preview packets.
There is no additional service, account, paid storage or render server.

## Renderer contract

`catalog/renderer.js` hosts **the exact native Ignis WGSL module** in browser
WebGPU, with `vs_main` / `fs_main`, native 64-byte vertices, 464-byte globals,
1392-byte material records and the original procedural word buffer. Native Rust
`materials::preview::encode` calls the same validator and `recipe::pack` used by
the desktop renderer. It does not bake material maps or reimplement the mineral
functions in JavaScript. Shared shader inputs and packet/source/thumbnail SHA-256
are recorded in the site snapshot; the browser verifies shader/packet hashes.

This first web host presents opaque materials on a sphere, cube or plane with a
directional light, native diffuse/specular response, procedural height/bump and
recipe parallax/self shadow. Scene shadow maps, IBL, atmospheric/planet views,
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
Only licenses, listed material sources, captures and native GPU packets enter
Hosting. Use one current exporter executable and the normal shared Cargo cache;
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
| Allocation | One device/context/pipeline. Replace material/program/VB/IB resources on changes; destroy old buffers and resized depth targets. Page exit/device loss disposes the host. |
| Data | Exact native ABI; bounded IGWP1 packet, little-endian words, SHA-256 and original source version. No scene serialization or recipe parsing per frame. |
| Repetition | One shared shader and context across all cards. Cached compiled pipeline, retained geometry until a shape/material-domain change. Static lazy thumbnails have no per-card renderers. |
| Limits | 500 records at admission, 1 MiB shader/packet budget, 560k preview pixels, DPR capped at 1.5; current sphere 2665 vertices / 5120 triangles. These are preview budgets, not engine limits. |
| GPU / IO | Two storage buffers for one material; event-time fetch/upload only, no GPU readback or network in draw. Site data snapshot is about 0.92 MB including all thumbnails, sources and packets. |
| Concurrency | Abort obsolete asset fetches; generation token prevents an old selection taking over. One queued RAF, no worker/engine locks. Native export runs outside the interactive host. |
| Invalidation | Locale/search/filter changes update DOM without reinitializing GPU. Camera/light reuse data. Hidden/offscreen view does not schedule draws. |
| Validation | Native pack equivalence/rejection tests; native GPU recipe regression; all twenty browser packets drawn, controls, filters/EN/PT, narrow layout, fallback and paired served-hash verification. |
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
