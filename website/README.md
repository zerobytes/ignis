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

The expanded `#reality` chapter contains the shared MCP contract, five illustrative command phases, a searchable/filterable atlas of 20 implemented domains, all 56 current dispatcher operations, the interactive 16-domain roadmap and an actual laboratory capture. EN/PT content and provenance are in `dist/engine-data.js`, `dist/engine-chapters.js` and `CONTENT.md`. Roadmap selection supports arrows/Home/End; native details expose system boundaries and command names. Search, filter, open rows and selected roadmap/command phase survive language changes. The presentation does not execute engine commands. SVG motion pauses offscreen, when hidden, with the global pause control and under reduced-motion preferences; no additional animation loop or dependency is introduced.

The capability atlas now uses a joined rectangular tab strip for Combat, NPCs, Character, World, Rendering and Tools, plus Everything. Membership may overlap for shared systems; the 20 source records remain unchanged. CSS perspective gives fine-pointer hover depth, disabled by global pause/reduced motion. ARIA tabs use roving focus and Left/Right/Home/End; the horizontal strip scrolls internally on mobile. Search is scoped to the selected system.

The capability rows use layered 3D surfaces, bounded decorative wireframe SVGs, cursor-position lighting and independent glyph/title depth. Native details unfold a second content layer. Finite entrance animations are gated by intersection; cached pointer bounds avoid repeated layout reads. Motion respects global pause, hidden documents and reduced preferences; no additional render loop or runtime dependency is introduced.

## Public Firebase Hosting

Published with explicit authorization on 3 November 2026: **https://ignis.web.app**. Firebase project `ignisengine` (Ignis Engine), Hosting site `ignis`, deployment target `website`, account `zerobyte30k@gmail.com`. Project ID `ignis-engine` was occupied; the requested short Hosting name `ignis` was available and reserved successfully. Only the 21 static files under `website/dist` are deployed. No engine binaries, source, local artifacts, credentials, backend services or billing setup were added.

From `website/`, publish authorized future updates with:

```sh
npx -y firebase-tools@latest deploy --only hosting:website --project ignisengine --account zerobyte30k@gmail.com --non-interactive
```

`firebase.json` keeps existing `.html` links and missing-asset 404s intact. Non-fingerprinted content revalidates through Cache-Control. The first release is `1793716375631000`, version `57fb38fd345f6841`. The native deploy connector reported success without creating a release; explicit CLI deployment uploaded 21 files and produced the verified live release. Browser EN/PT, WebGL and pulse interactions passed, with no console errors. Key served assets matched local SHA-256 using system curl TLS validation; Python's independent certificate store could not validate this environment's certificate chain and was not weakened.

The engine catalog remains the explicitly dated 24 October content snapshot; publishing this presentation does not update its capability inventory, distribute the engine, or implement a WASM runtime. Custom domain and downloads remain separate work.
