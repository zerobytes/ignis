# Ignis — immersive direction

## Intent

The initial static proposal was rejected as too conventional. The replacement is a continuous 3D composition with a scroll-directed camera and a visitor-controlled terrain. The user authorized the new direction after reviewing the proposed approach.

The current opening is “Ideas aren't flat.” / “Ideias não são planas.” The signature line remains “Ignite the spark. Create worlds.” / “Acenda a centelha. Crie mundos.” A sculptural molten form is the visual metaphor for raw potential; it fragments and gives way to a seeded landscape and a portal. Architectural sans typography contrasts with oversized italic serif forms. Charcoal, mineral green, ivory and ember-orange create the palette. Native scroll remains the navigation model.

This WebGL composition is branded concept art, not output of the Rust engine. The actual perception-lab capture is isolated and labeled. No public release, selected license, finished roadmap, platform portability or rendering capability is implied by the artwork. The symbol and full PNG signature remain on the identity sheet.

## Renderer decision

Three.js 0.180.0, vendored locally under MIT, provides camera, geometry, materials, resource management and WebGL infrastructure. Direct WebGL avoids the library but would require owning all of those primitives; a video would sacrifice genuine interaction. The accepted implementation tradeoff is a roughly 720 KB local renderer dependency for actual 3D interaction. No framework, build pipeline, animation package or external runtime requests are introduced.

Materials use generated geometry and procedural shader terms; terrain uses InstancedMesh. No external models, image environment maps, shadow maps or bloom/postprocessing chain. The environment is prepared once from simple light panels. The renderer does not access the Ignis engine, and the site's actions do not edit scenes or control gameplay.

## Pre-implementation checklist

1. One bounded animation loop. Stop for hidden documents, pause, reduced motion and after the interactive chapters; redraw for explicit events.
2. Allocate geometries, uniforms, instance/particle buffers and scratch objects once. Rebuild terrain matrices only on seed changes.
3. Typed particle/shard buffers and bounded instancing. No scene data or engine identities.
4. Scroll stores progress; one loop interpolates the camera, uniforms and transforms. Avoid multiple tickers/timers.
5. Fixed geometry counts and a bounded pulse pool; expired waves must release their slots.
6. Persistent GPU resources, capped pixel ratio and mobile geometry tier. No readback or per-frame uploads of terrain geometry.
7. No engine locks or backend. Website state is local.
8. Resize camera/projection/resolution; invalidate terrain when seed changes; cancel the loop on visibility/pause/reduced-motion changes; dispose on teardown/context loss.
9. Verify actual rendered transitions and controls, EN/PT, keyboard, mobile layout, reduced motion and unavailable WebGL.
10. Accept the additional renderer/GPU cost for the creative brief; no unmeasured FPS or engine-performance claim.

## Review of implementation

| Question | Result and evidence |
| --- | --- |
| Frequency | Adequate: one RAF in `experience.js`; hidden document cancels RAF, paused/reduced states redraw on events, continuous loop stops at progress 3. |
| Allocation | Adequate: fixed typed buffers, cached DOM references/uniforms, reused Object3D/Vector3/Color. The per-frame shard matrix upload is intentional for fragment motion. |
| Types/access | Adequate: Float32Array particle/shard storage; instanced terrain and fragments; bounded current seed and 32 reusable pulse timestamp/age slots. |
| Repetition | Corrected: seed recalculates terrain only on regeneration/reset. No runtime image fetch, geometry rebuild or environment preparation in the frame loop. |
| Scale | Adequate: desktop 1,936 columns/1,400 points/70 fragments; mobile 1,024/600/40. Quality tier is selected at initial load; resize does not allocate duplicate tiers. |
| GPU/I/O | Adequate: local modules and assets, capped resolution, instanced objects, no shadows/postprocessing/readback. Warm/cool environment generated once. Corrected excessive PMREM blur after browser warning. |
| Synchronization | Not applicable: independent web renderer, no Rust/MCP runtime calls, workers, locks or backend. |
| Invalidation | Validated: seed changes the terrain signature; reset reproduces seed 42 within a quality tier; locale refreshes status/feedback/metadata; resize and media changes update rendering behavior. Teardown disposes geometries/materials/instance buffers/environment/renderer. |
| Contract | Validated: browser EN/PT in 320/390/768/1440 px without horizontal overflow or clipped text; real WebGL, scroll camera/growth, regeneration, pulse, wireframe, deterministic reset, pause clock, reduced-motion event-only rendering and context-loss fallback/restoration. |
| Remaining cost | Kept intentionally: metallic sculpture, procedural emissive shader, seed-based columns and per-frame bounded fragment transforms. No profiling/FPS claim. Raster identity is still a proposal. |

## Evidence and boundaries

Browser runtime diagnostics provide read-only state for this web composition: seed, terrain signature, instance counts, animation clock, progress, wireframe, active pulse ages/count and pending-frame state. They expose no controls or engine data.

Seed 42 desktop signature: 1343530169; next seed 4273: 2482539031; reset restored 1343530169. Mobile signature differs intentionally with the 32×32 geometry tier. Paused clock remained unchanged with no pending frame. Reduced motion had clock 0/no pending frame while seed/pulse remained usable. Simulated context loss disabled the four 3D controls and restored a readable static presentation; reload restored WebGL. No JavaScript/shader errors observed after the final changes.

Mobile camera and Portuguese title sizing were corrected after visual overlap was observed. Actual desktop/narrow browser captures are kept at stable paths under ignored `artifacts/`; this is not physical-device acceptance. The local preview is the review deliverable. No publication or source push. Concurrent Rust/NPC work is preserved and outside this diff. Guilherme approved the immersive visual direction; the subsequent overlapping-pulse refinement is documented below.

## Overlapping pulses — review before implementation

Requested behavior: each click starts a new wave without restarting earlier waves. Keep the web renderer independent of the Rust engine. Use 32 reusable timestamp/age slots, expire waves after they have left the landscape, and sum active wave contributions in the existing shader. When full, preserve active waves and reject the additional emission with localized feedback. Regenerating/resetting the world clears its old waves. Bound combined displacement/light to avoid extreme amplitudes under rapid clicks.

Checklist: (1) update ages in the existing frame; (2) allocate typed buffers once, compact in place; (3) CPU timestamps retain double precision, shader ages use floats; (4) calculate each pulse age once per frame; (5) fixed 32-wave shader bound with active-count early exit; (6) reuse uniforms and geometry, no extra draw calls/readback; (7) no synchronization/backend; (8) expiration, pause/reduced motion, full capacity and reset must preserve valid lifetimes; (9) browser-test overlap and earlier-wave continuity, expiry/reset, capacity, shader compilation and both locales; (10) accept extra bounded shader work only while waves overlap, without a performance claim.

### Overlapping-pulse results

- Corrected: emitting a wave no longer replaces a shared timestamp. Browser sequence preserved the first wave while active counts advanced 1 → 2 → 3; ages after the third click were approximately 0.800 / 0.525 / 0.234 seconds. Independent concurrent contributions were rendered without shader/JavaScript errors at the 32×32 and 44×44 terrain tiers.
- Adequate: in-place timestamp compaction expires waves at 3.2 seconds, after radius 16 has passed the landscape corners. The original test waves later expired to zero. CPU/GLSL buffers are reused; active-count early exit bounds GPU work and adds no draw calls.
- Validated: paused capacity test retained all 32 waves on a 33rd click with localized feedback; reset cleared all waves. Pause freezes ages; reduced motion uses the existing static preview behavior. A new world clears old waves and starts its own initial pulse.
- Kept intentionally: shader cost grows with concurrent waves up to 32. Combined wave strength is capped at 3 to bound displacement and light under rapid clicks. No performance claim. Web renderer only; no Rust/runtime changes or engine regression suite required for this diff.

## Engine chapters — review before implementation

New scope: give the engine itself the visual and narrative weight of the opening. Preserve the approved WebGL chapters and pulse behavior. Add an animated MCP-first schematic, an explorable inventory of implemented systems and commands, a complete 16-domain roadmap, and actual-engine evidence. The current library has progressed through the EAF-46 dispel increment. Older README/ADR summaries contain historical limits that later focused contracts supersede; resolve those against src/engine.rs and the latest system documents.

MCP-first is an architectural rule for every controllable engine capability, current and planned. The AI control plane, native UI and shared typed dispatcher must be described without implying that roadmap systems already exist, that MCP runs once per physics frame, or that developer inspection grants an NPC omniscience. The web schematic demonstrates the contract; it does not connect to a live engine.

Before coding: (1) interaction/locale/filter handlers only; no second render loop; (2) prepare bounded catalog DOM/search text, reuse it for filtering; (3) fixed source-backed records, command names extracted from the shared enum; (4) no per-frame search/layout/JSON reconstruction; (5) 20 current domains, all shared commands and 16 roadmap domains, no growing history; (6) SVG/CSS motion, pause outside viewport/when hidden and honor reduced motion, reuse current local assets; (7) no backend, engine mutation or locks; (8) locale preserves open/selected/filter state, layout informs existing scroll metrics; (9) verify complete coverage, source provenance, accessible keyboard/filter/roadmap behavior, mobile and top-scene regressions; (10) accept longer editorial content and bounded DOM for completeness, without new runtime dependencies or FPS claims. No Rust changes or engine regression execution needed for this website-only scope.

### Engine chapters — review after implementation

1. No second RAF, renderer, engine connection or dependency. Interaction/locale events update bounded DOM only (`engine-chapters.js`).
2. 20 existing details rows filtered in place; normalized search text cached per rendered locale. Search is capped at 120 characters. Language changes rebuild this small fixed catalog while preserving open IDs, query, filter and selections.
3. Inventory matches all 56 `Command` variants; 16 roadmap domains have bilingual current/planned explanations. Content audit and provenance are in `CONTENT.md`.
4. No per-frame search, JSON formatting, DOM measurement or system generation. JSON examples format only on stage/locale changes. Existing scene metrics recompute only for layout events.
5. SVG/CSS graph has bounded geometry; IntersectionObserver, visibility, global pause and reduced-motion stop motion. Existing laboratory/brand assets reused. No additional GPU scene, I/O, locks or backend calls.
6. Full bilingual content is the consciously retained payload (approximately tens of KB); fixed DOM and event-time rebuilds keep implementation direct. No FPS improvement claimed.
7. Browser checks: all 16 roadmap panels, arrow navigation, system disclosure, category/search, locale state preservation, five contract phases, pause/reduced motion, no new console errors; 320/390/768/1440 widths without horizontal overflow. Existing paused pulse composition retained two simultaneous waves. Desktop/mobile screenshots reviewed.
8. JavaScript syntax, content/command inventory and whitespace checks passed. Rust unchanged; engine regression suites were not rerun for this presentation-only increment. Human visual acceptance of the expanded chapters remains pending. Local delivery only.

## Root-system tabs — before implementation

Replace category pills with seven joined rectangular tabs: combat, NPCs, character, world, rendering, tools and all. Keep the existing 20 records; explicit bounded index memberships allow shared capabilities without duplicating content. Filter/search remain event-time scans of existing rows, no new RAF, dependencies, assets, I/O or engine calls. CSS perspective/transform supplies hover depth only on fine pointers, disabled by pause/reduced motion. Use ARIA tabs, one associated panel, roving focus and arrows/Home/End; horizontal overflow stays inside the tab strip on narrow screens. Preserve locale, open-row and search state. Validate mapping coverage, keyboard/locale/search, desktop/mobile and unchanged content boundaries. No Rust changes or engine regressions needed for this UI increment.

### Root-system tabs — after implementation

Seven ARIA tabs share single border rules; CSS perspective faces lift/rotate on hover without pointer listeners or animation loops. Membership is explicit and overlaps intentionally; Combat4, NPCs9, Character4, World3, Rendering2, Tools4, Everything20 preserve all records. Existing rows are hidden in place; selected tab, search and open details survive translation. Arrows/Home/End use one tab stop and update panel association; mobile keyboard navigation reveals the selected tab inside the strip.

Browser evidence: all seven memberships and panel labels checked; NPC selection retained in EN, flight search found one result; Home/ArrowRight and mobile Home selected correctly. Desktop computed matrix3d confirmed hover; global pause returned transform:none; reduced motion returned transition0s. Mobile390 had no document overflow while the tab strip scrolled795px inside353px. Desktop/mobile screenshots reviewed, no console errors. Syntax and whitespace checks passed. Kept event-time 20-row filtering and fixed mappings; no new dependencies, GPU resources, engine changes, I/O or FPS claims. Evidence: artifacts/website-tabs.png. New UI visual acceptance pending.

## Depth list — before implementation

Keep 20 native details and their content/contracts. Add bounded inline SVG geometry and a summary surface with CSS perspective, cursor light and finite entrance/open transitions. Pointer coordinates update CSS variables only on the active summary; cache its rectangle on entry, reset on leave/scroll/resize. No continuous RAF, WebGL scene, dependency or new asset. IntersectionObserver marks visibility; finite row animations never run on hidden rows. Global pause/reduced motion/hidden document disable visual motion and pointer processing; semantic details keyboard/touch remain unchanged. Validate categories, locale/open preservation, disclosure, cursor depth, pause/reduced preferences, narrow layout and screenshots. Rust/runtime unchanged.

### Depth list — after implementation

20 bounded inline SVG glyphs, one native summary/content pair per original record, no duplicated technical content. Summary uses perspective, independent title/glyph Z layers, cursor radial light, stacked edge shadows and finite visible-row arrival. Native opening unfolds a second layer; touch/keyboard remain details behavior. No continuous loop, dependency, asset fetch or engine mutation. Active pointer rectangle is cached once, four CSS scalars updated on pointer movement; scroll/resize/leave/preference/visibility/pause clear it. Locale rebuild disconnects the row observer before replacing nodes, avoiding detached-row retention. Global pause, document hidden and reduced motion disable animations/transforms; pointer processing is gated too.

Verified in browser: 20 glyphs, four visible combat rows, keyboard Enter opens, EN/PT keeps the open row; desktop and390px mobile have no document overflow. Desktop pointer produced a real matrix3d and spot79.9995%; global pause and reduced motion both returned transform:none and animation:none. Desktop/mobile screenshots reviewed; console error list empty. JS syntax and whitespace passed. Kept bounded pointer-event style updates and small SVGs; no measured FPS claim. Evidence artifacts/website-depth-list.png. Rust unchanged; engine regression not rerun. New visual acceptance remains pending; local only.

## Firebase publication — before configuration

Publish the existing approved static experience only, as explicitly authorized on 3 November 2026. Hosting public root must be website/dist (no engine source, artifacts, documents or credentials). Configure a named site target and explicit project/account; deploy Hosting only, no SDK/backend/billing services. Existing EN/PT routes/assets remain intact; use revalidatable caching for non-fingerprinted assets and no SPA catch-all that would mask missing resources. Validate syntax, upload boundary, deployed HTTP/assets and representative browser interactions. No new tests or Rust regressions warranted for hosting configuration. Preserve unrelated work; one scoped local commit. Engine content remains the visibly dated 24 October snapshot; refreshing its inventory is a separate task.

### Firebase publication — after configuration

Scoped hosting root `website/dist`, 21 static files; named website target points to ignis in project ignisengine. CLI15.34.2 account and live release metadata confirmed zerobyte30k@gmail.com, release1793716375631000/version57fb38fd345f6841. Cache revalidation retains safe updates for existing unfingerprinted filenames; routes unchanged/no wildcard rewrite. HTML asset paths and JS syntax passed. Public browser rendered WebGL, switched EN/PT and sent a pulse with no JS errors; screenshot artifacts/website-published.png. Served key assets matched local SHA-256 via system curl with TLS verification. Python certificate-store failure was isolated, no verification bypass applied. No new renderer/loop/SDK/backend/billing, no Rust changes or regressions. Content snapshot consciously retained and disclosed. Public delivery now authorized and complete; Git push not performed.

## Current engine atlas — before implementation (3 November 2026)

Preserve approved opening/interaction identity. Replace obsolete 24 October catalog/roadmap with source-backed current domains, status boundaries, current command inventory and curated real captures. Integrated renderer optimizations and experimental IGL remain distinct; no blanket Unreal superiority or universal FPS claim without a matched benchmark. Apache-2.0 now selected. Materials/transparency and VFX are proposals unless completion evidence exists. Planet runtime is opt-in, single-player and visually under refinement.

Bounded static data/DOM, no new dependency or animation/render loop; reuse existing native details/tabs, add lazy images and source captions, finite gallery transitions and accessible image enlargement. Optimize web copies without changing originals; no live demo/window, Cargo/GPU or process interactions. Native captures come only from known project evidence, no reference photographs or synthetic mockups represented as engine output. No video unless existing footage adds evidence. Pointer observers reset on locale rebuild; filter/search stay event-time. Validate data coverage and explicit operation names, image provenance/weight, EN/PT, keyboard/mobile and deployed assets; no new automated test suite or engine regression for editorial changes. Commit only website files and deploy the existing Firebase target after review.

### Current engine atlas — after implementation

65 bilingual source-backed entries/235 top-level operations/16 updated roadmap domains, grouped into nine roots plus Everything. Shared records keep cross-domain membership; native details preserve keyboard controls and open state across locale changes. Current technical data replaces stale claims about ramp control, death/respawn, audio, assets/editor, NPC closure, multiplayer and licensing. Nested Botany/EditMesh contracts are not counted as single operations per internal action. Nanites research/IGL remains marked experimental/off; measured renderer wins are attributed to their actual visibility/material/buffer path, not universal superiority.

16 original captures inspected as a contact sheet and in the site; WebP copies retain aspect ratio, maximum1600×1200,722120B total. Provenance records source SHA256 and captions; character-source CC0 credit/historical editor/age/planetary appearance identified. Lazy image loading and enlargement use native dialog; Escape/close return normally, gallery links select the corresponding root. No invented video or engine window/build interaction. Core data payload109KB retained as a conscious cost for the detailed bilingual inventory; fixed65row DOM, finite CSS effects and cached search, no new RAF/GPU/dependency. Existing glyph observer disconnect/reuse behavior retained on locale rebuild.

Browser QA: all10root counts,16gallery selections,235inventory items, Nanites/raízes search, context link, details/modal/Escape, open-row EN/PT preservation and arrow navigation.320/390/768/1440 widths have no horizontal document overflow. Desktop/mobile gallery screenshots reviewed, all16capture files/provenance/source contracts checked and JS console errors empty. Syntax/whitespace and command-enum/source/media audits passed. No automated test suite added or engine regression rerun for editorial/website-only work. Human acceptance of new content remains separate from publication.

Inventory checkpoint: the committed engine at 5835a64 has235 Command variants. Five new planetary-view variants in parallel uncommitted work are excluded until reviewed; the published catalog reflects the stable snapshot, not in-flight code.

## Product storytelling revision — 3 November

User rejected the gallery's image choices and requested strong product arguments. Use the two supplied native landscapes as the lead imagery; curate the public gallery to finished visual views, retaining diagnostic captures only inside relevant technical entries. Introduce Ignis Horizon as a proposed umbrella presentation name for the integrated visibility/representation stack (not a rename of IGL or an engine API). Lead with creative outcomes: procedural plant generation, large native scenes, human/AI parity, NPC behavior, offline/optional networking. Match claims to dedicated source: 30k trees/60,006 mesh instances,18km landscape centers,2,000-year authored plant age range,1.90x draw+GPU-wait in specified adaptive-occluder test. Do not convert these into universal species fidelity,100km terrain,FPS, or Unreal-superiority claims. Keep detailed methodology in accessible disclosure, without making caveats the main pitch. Reuse image dialog and root links; no renderer/frame loop/dependency changes. Validate EN/PT/responsive/keyboard/image loading and served assets, then publish Hosting-only.

Revision QA: the two supplied renders are optimized without artistic alteration; source originals are outside Hosting. Curated5-view gallery loaded every selection. Product image dialog/Escape and root exploration links passed; EN/PT translated the narrative and retained root state.320/390/768/1440 had no horizontal overflow; desktop and390px compositions inspected. No console errors. Source/module/media syntax and provenance audit passed. Opening WebGL and its loop unchanged; no engine regression suite warranted for this editorial scope. Awaiting human visual/content acceptance.

## Horizon architecture comparison — review before edit

Deepen the accepted Horizon name with a bilingual architecture explanation, four-step conceptual pipeline, source/edit/fallback contract, and a mechanism-level comparison with Nanite. Consult dedicated Ignis docs and current Epic official overview/technical details. Distinguish whole-instance ATM from Nanite clusters/streaming; include material-contribution selection, consumed attributes and conservative fallbacks. Do not equate Horizon with a shipped Nanite replacement, claim Nanite lacks fog/editing/occlusion, or generalize superiority. Accessible semantic table with internal horizontal scroll on mobile, source links in disclosure, no new loop/dependency/engine work. Validate syntax, EN/PT, keyboard disclosure/table focus, mobile overflow and publication hashes. No new unit tests warranted for source-backed editorial content.

Comparison QA:5rows and4pipeline stages in both locales; source disclosure opened with Enter, official links verified, no JS errors.320/390/768/1440 no document overflow; narrow widths contain the semantic table in its own focusable scroll region. Syntax and whitespace passed. Name Horizon now accepted by the creator. Native renderer untouched; publication and artistic acceptance remain separate.

## Paired destination publication

User requires every future website release on Firebase and GitHub Pages. Add one explicit paired publication entrypoint: deterministic static version, website-only Git API commit based on current public head/no force, Pages success gate, immutable temporary Firebase staging with cleanup, then version and all served-file hashes on both domains. Preserve public engine content and private-checkout remote/history. No scheduled automation or new credentials. The accepted headline is Ignite your ideas / Dê vida às suas ideias; update runtime and static fallback together. Validate responsive headings/localization and execute the actual paired release; no speculative regression suite for copy. Current licensing changes from the parallel project release remain untouched.

Headline QA: EN/PT at320/390/768/1440 fit without document overflow; desktop composition inspected. JS syntax and Python AST parse passed. Actual paired deployment below is the integration acceptance for the publishing script; source-only checks do not establish public delivery.
