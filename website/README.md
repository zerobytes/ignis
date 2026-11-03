# Ignis — immersive web experience

Bilingual, scroll-directed 3D presentation for Ignis Engine. The website is independent from the Rust engine. Its interactive composition is explicitly identified as a brand experiment; the perception-laboratory section uses a real engine capture.

## Local preview

From the repository root:

```sh
python3 -m http.server 4173 --bind 127.0.0.1 --directory website/dist
```

Open `http://127.0.0.1:4173/?lang=pt` or `http://127.0.0.1:4173/?lang=en`.

No package installation or build step. Three.js 0.180.0 is vendored in `dist/assets/vendor` with its MIT license. Font, images and modules are all local; no runtime CDN, analytics, forms or account integration.

## Experience

- A rotating sculptural ember with a procedural emissive material and instanced fragments.
- Scroll interpolates camera position, fragmentation, lighting and terrain growth.
- A seeded terrain composition with overlapping pulse waves, a new-seed action, wireframe and exact reset to seed 42 for the current quality tier.
- Each pulse has its own lifetime: later clicks preserve earlier waves. Up to 32 can coexist; expired slots are reused. At capacity, active waves are preserved and a localized message rejects the new emission. New world/reset clear old waves.
- Pointer movement changes the perspective. Native scrolling and anchor navigation remain intact; there is no scroll hijacking.
- Pause stops the animation clock; explicit controls and scroll still redraw on demand. The OS reduced-motion preference disables the continuous loop and entrance animations.
- A hidden document suspends the loop. After the WebGL chapters, continuous animation stops. Context loss displays the static artwork and disables the unavailable 3D controls.
- Mobile initial loads use 1,024 terrain instances and 600 points; desktop uses 1,936 instances and 1,400 points. Pixel ratio is capped. Resizing updates projection and resolution, while keeping the allocated geometry tier bounded for that session.

Language follows explicit query, saved preference, then browser language. EN/PT changes title, metadata, labels, image descriptions, feedback and the identity link. Optional local-storage failure does not prevent the experience.

## Files

- `dist/index.html`, `dist/experience.css`, `dist/experience.js`: main experience.
- `dist/brand.html`, `dist/style.css`, `dist/app.js`: existing identity sheet.
- `dist/assets/symbol.png`: standalone transparent symbol proposal.
- `dist/assets/lockup.png`: full signature proposal on dark background.
- `DESIGN.md`: direction, renderer decision and static performance review.
- `brand/ART-DIRECTION.md`: identity and illustration briefs.

Raster identity assets remain proposals for creative review. Vector masters and color variants are a subsequent finishing step after the direction is accepted.

## Verification boundary

Browser review covers both languages at 320/390/768/1440 px, real WebGL rendering, seed/regenerate/reset, wireframe, pulse, pause/resume, reduced motion, context loss/static fallback, restoration by reload, metadata, anchors and asset delivery. Browser viewport review is not physical iPhone/iPad acceptance. No engine source, MCP dispatcher or simulation contracts were changed; Rust suites and runtime scenarios are outside this website-only change. No source push or publication.

## Engine chapters

The expanded `#reality` chapter contains the shared MCP contract, five illustrative command phases, a searchable/filterable atlas of 65 reviewed capabilities, 235 top-level dispatcher operations from the reviewed engine snapshot, the interactive 16-domain roadmap and an actual laboratory capture. EN/PT content and provenance are in `dist/engine-data.js`, `dist/engine-chapters.js` and `CONTENT.md`. Roadmap selection supports arrows/Home/End; native details expose system boundaries and command names. Search, filter, open rows and selected roadmap/command phase survive language changes. The presentation does not execute engine commands. SVG motion pauses offscreen, when hidden, with the global pause control and under reduced-motion preferences; no additional animation loop or dependency is introduced.

The capability atlas uses a joined rectangular tab strip for Rendering, Botany, World, Characters, Modeling, NPCs, Combat, Multiplayer and Tools, plus Everything. Membership may overlap for shared systems; shared source records are reused across relevant domains. CSS perspective gives fine-pointer hover depth, disabled by global pause/reduced motion. ARIA tabs use roving focus and Left/Right/Home/End; the horizontal strip scrolls internally on mobile. Search is scoped to the selected system.

The capability rows use layered 3D surfaces, bounded decorative wireframe SVGs, cursor-position lighting and independent glyph/title depth. Native details unfold a second content layer. Finite entrance animations are gated by intersection; cached pointer bounds avoid repeated layout reads. Motion respects global pause, hidden documents and reduced preferences; no additional render loop or runtime dependency is introduced.

## Public Firebase Hosting

Published with explicit authorization on 3 November 2026: **https://ignis.web.app**. Firebase project `ignisengine` (Ignis Engine), Hosting site `ignis`, deployment target `website`, account `zerobyte30k@gmail.com`. Project ID `ignis-engine` was occupied; the requested short Hosting name `ignis` was available and reserved successfully. Only the 21 static files under `website/dist` are deployed. No engine binaries, source, local artifacts, credentials, backend services or billing setup were added.

From `website/`, publish authorized future updates with:

```sh
python3 deploy.py
```

`firebase.json` keeps existing `.html` links and missing-asset 404s intact. Non-fingerprinted content revalidates through Cache-Control. The first release is `1793716375631000`, version `57fb38fd345f6841`. The native deploy connector reported success without creating a release; explicit CLI deployment uploaded 21 files and produced the verified live release. Browser EN/PT, WebGL and pulse interactions passed, with no console errors. Key served assets matched local SHA-256 using system curl TLS validation; Python's independent certificate store could not validate this environment's certificate chain and was not weakened.

The first public release used the 24 October snapshot. The 3 November catalog below supersedes it. Website publication does not distribute the engine or implement a WASM runtime. Custom domain and downloads remain separate work.

## Current native engine catalog (3 November 2026)

The current engine chapter replaces the earlier20-system snapshot with65 source-backed entries across Rendering, Botany, World, Characters, Modeling, NPCs, Combat, Multiplayer and Tools. It includes235 top-level commands and an updated16-domain roadmap. Counts describe editorial entries and first-level operations, not complete production systems or every nested suboperation.

The native gallery contains16 project captures, with thumbnail selection and a keyboard-accessible enlargement dialog. Per-capability disclosures add implementation facets, MCP names, explicit boundaries and relevant images. Captions distinguish experimental/historical imagery, including planetary appearance and CC0 MakeHuman-based character data. `media-provenance.json` records originals/hashes; public WebP copies total approximately705KiB. Core catalog sources and rendering/experimental distinctions are in `CONTENT.md`.

No video was fabricated from stills and no live engine session was manipulated. The approved Three.js opening remains identified separately. Apache-2.0, technically closed NPCv1, QUIC/loopback scope, portable saves, botanical sources and planetary limits are reflected in current copy. Publishing the current site supersedes the historical24October catalog note above; engine downloads/web export remain separate deliveries.

The product-story revision introduces the proposed Ignis Horizon name, two creator-selected native landscapes, a dedicated procedural plant-generation story and a five-view curated gallery. Diagnostic captures remain in the technical atlas. Copy and claim provenance: CONTENT.md; originals: media/ (outside Hosting public root).


## Paired publication: Firebase + GitHub Pages

User direction: every website publication must update both https://ignis.web.app/ and https://zerobytes.github.io/ignis/.

From this checkout run `python3 website/deploy.py` (or `python3 deploy.py` inside website). Requires authenticated gh/Firebase CLI, Node and system curl. `--prepare` only regenerates deterministic dist/version.json and performs no external writes; run this before the scoped local commit. The release ID hashes every public file except the version manifest itself.

First run `--prepare`, commit the intended website files and push them to the public `main` through the normal Git workflow. The command captures the static bytes once and requires **website/dist/** in public `main` to match exactly. It then dispatches/awaits the existing Pages workflow and publishes the same bytes to Firebase through an automatically removed staging directory. A source mismatch stops deployment before either destination is published. The command creates no remote-only source commits and does not push unrelated local changes. No credentials are written to source.

Completion requires matching version.json and SHA256 of every served file on both domains; cache propagation retries are bounded. A failed destination is reported as incomplete, and rerunning reconciles the same snapshot. Publication across two services is sequential, not an atomic distributed transaction. Use this paired command for normal releases; the Firebase-only command is reserved for explicit destination-specific recovery.
