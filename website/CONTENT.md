# Engine content provenance

Reviewed 3 November 2026 against the current main checkout. This replaces the historical 24 October/EAF-46 inventory. The site is a public presentation, not a connection to the Rust process. Illustrative MCP examples never execute engine commands.

## Scope and evidence

65 editorial capability entries, nine root systems plus Everything, 235 top-level Command variants, 16 roadmap domains and 16 native captures. Nested contracts such as Botany/EditMesh contain additional suboperations; the top-level count is not a count of every configurable engine detail. Command names were re-extracted from src/engine.rs and checked against committed engine snapshot 5835a64 (235 variants). Five planetary-view commands appearing in parallel uncommitted work were deliberately excluded from this reviewed catalog.

Current dedicated contracts take precedence over historical introductory limits. NPC v1 has technical closure, while human/art acceptance is separate. Apache-2.0 is selected; code licensing does not by itself mean a packaged engine download or a tested web build exists. Original project source and notices remain authoritative.

### Rendering and the Nanites research track

Integrated work includes instancing/content resources, conservative frustum visibility, GPU occlusion, fine/adaptive occluders, consumed-attribute streams, static residency, compound objects and atmosphere/distant representations. Region-compiled IGL geometric LOD is explicitly experimental/off by default. The forest produced no useful new IGL package; triangle reduction and performance from visibility must not be attributed to that prototype. The separate graphics-research worktree was read only for provenance and is not advertised as shipped wholesale. No blanket claim of being superior to Unreal/Nanite/MetaHuman is made: there is no matched cross-engine benchmark in the reviewed evidence.

Measured details cite the scenario and method in the corresponding entry. The 1.7–7.9% fine-occlusion result is CPU draw plus GPU wait in six specified M4 Pro/Metal cases, not a gameplay FPS promise. The 1,000-player, 30,000-tree and Earth-scale figures are different experiments, not one combined scene.

### In-progress and future boundaries

Planetary runtime is opt-in and currently one Player, with save/multiplayer/authoring paths refused; synchronous preparation and appearance are still being refined. Planar terrain Ridge is not silently advertised on Planet1. Botany XPBD is a session laboratory separate from Rapier gameplay/network saves. Character source incorporates attributed CC0 MakeHuman graphic data; it is not entirely original anatomy or a complete MetaHuman equivalent. Material transparency/glass and VFX remain in development/requirements in the reviewed contracts; no emitters, fluid simulation or universal production pipeline are invented.

## Capability source map

| Entry | Capability | Source contracts | Delivery boundary |
| --- | --- | --- | --- |
| 01 | Cenas que guardam sua origem | `docs/projects.md` | Implemented slice; boundaries in entry |
| 02 | Editar sem perder o caminho de volta | `docs/commands.md` | Implemented slice; boundaries in entry |
| 03 | Física em passos explícitos | `docs/continuous-runtime.md` | Implemented slice; boundaries in entry |
| 04 | Um mundo visível e selecionável | `docs/procedural-environment.md` | Implemented slice; boundaries in entry |
| 05 | Volumes que percebem passagem | `docs/commands.md` | Implemented slice; boundaries in entry |
| 06 | Ver não é saber tudo | `docs/commands.md` | Implemented slice; boundaries in entry |
| 07 | Ouvir uma pista. Não uma coordenada viva. | `docs/npc-presentation-complete.md` | Implemented slice; boundaries in entry |
| 08 | Autonomia, relações e alvos | `docs/commands.md` | Implemented slice; boundaries in entry |
| 09 | Memória, busca e comunicação | `docs/commands.md` | Implemented slice; boundaries in entry |
| 10 | Decisões que explicam seus motivos | `docs/npc-v1-closure.md` | Implemented slice; boundaries in entry |
| 11 | Rotinas com lugar no mundo | `docs/npc-routines-interactions.md` | Implemented slice; boundaries in entry |
| 12 | Encontrar caminho. Dividir passagem. | `docs/ground-local-connections.md` | Implemented slice; boundaries in entry |
| 13 | Movimento que ocupa três dimensões | `docs/npc-hybrid-mobility.md` | Implemented slice; boundaries in entry |
| 14 | Habilidades, um executor compartilhado | `docs/commands.md` | Implemented slice; boundaries in entry |
| 15 | Vida, dano, cura e consequências | `docs/resurrection.md` | Implemented slice; boundaries in entry |
| 16 | Projéteis e intenção espacial | `docs/commands.md` | Implemented slice; boundaries in entry |
| 17 | Efeitos que persistem. Regras que dissipam. | `docs/attributes.md` | Implemented slice; boundaries in entry |
| 18 | O contrato é a interface | `docs/commands.md` | Implemented slice; boundaries in entry |
| 19 | Inspecionar além da aparência | `docs/commands.md` | Implemented slice; boundaries in entry |
| 20 | Reproduzir. Verificar. Corrigir. | `docs/commands.md` | Implemented slice; boundaries in entry |
| 21 | Atributos, buffs e dano composto | `docs/attributes.md`, `docs/damage-components.md`, `docs/outgoing-damage.md` | Implemented slice; boundaries in entry |
| 22 | Canalizar, interromper e dissipar | `docs/channeling.md`, `docs/interrupts.md`, `docs/dispel.md` | Implemented slice; boundaries in entry |
| 23 | Áreas, auras e apoio inteligente | `docs/instant-area.md`, `docs/persistent-area.md`, `docs/auras.md`, `docs/ally-triage.md`, `docs/dispel-triage.md` | Implemented slice; boundaries in entry |
| 24 | Editor, áreas e projetos | `docs/editor-ui.md`, `docs/projects.md`, `docs/editor-workflows.md` | Implemented slice; boundaries in entry |
| 25 | Modelagem por componentes | `docs/editor-modeling.md`, `docs/mesh-topology.md` | Implemented slice; boundaries in entry |
| 26 | Escultura com detalhe local | `docs/sculpt-brushes.md`, `docs/editor-sculpt.md` | Implemented slice; boundaries in entry |
| 27 | Curvas, seções e sweep | `docs/mesh-sweep.md`, `docs/plant-trajectories.md`, `docs/curved-plants.md` | Implemented slice; boundaries in entry |
| 28 | Junções orgânicas e regeneração | `docs/organic-continuity.md`, `docs/botany-collar.md`, `docs/mesh-preservation.md` | Implemented slice; boundaries in entry |
| 29 | Materiais procedurais e detalhes | `docs/procedural-materials.md`, `docs/surface-details.md`, `docs/layered-patterns.md`, `docs/triplanar-details.md` | Implemented slice; boundaries in entry |
| 30 | Dielétricos, metais e ambiente | `docs/dielectric-response.md`, `docs/metallic-response.md`, `docs/procedural-environment.md` | Implemented slice; boundaries in entry |
| 31 | Sombras, câmera e contornos | `docs/directional-shadows.md`, `docs/atmosphere-silhouettes.md` | Implemented slice; boundaries in entry |
| 32 | Geometria compartilhada por conteúdo | `docs/mesh-resources.md`, `docs/small-geometry-cost.md` | Implemented slice; boundaries in entry |
| 33 | Visibilidade conservadora | `docs/render-visibility.md` | Implemented slice; boundaries in entry |
| 34 | Oclusão GPU em duas escalas | `docs/render-occlusion.md`, `docs/fine-occlusion.md`, `docs/adaptive-occluders.md` | Implemented slice; boundaries in entry |
| 35 | Residência e atributos consumidos | `docs/minimal-vertex-stream.md`, `docs/static-residency.md` | Implemented slice; boundaries in entry |
| 36 | Representações distantes e atmosfera | `docs/atmosphere.md`, `docs/atmosphere-silhouettes.md`, `docs/render-optimization-comparison.md` | Implemented slice; boundaries in entry |
| 37 | Geometria virtualizada e LOD (pesquisa) | `docs/automatic-lod.md`, `docs/regional-lod.md` | Experimental, off by default |
| 38 | Objetos compostos e hierarquias | `docs/mesh-objects.md`, `docs/mesh-attachments.md`, `docs/mesh-island-transform.md` | Implemented slice; boundaries in entry |
| 39 | Plantas como fonte construtiva nativa | `docs/botany-integration.md`, `docs/botany-reference-refinement.md` | Implemented slice; boundaries in entry |
| 40 | Idade, vigor e desenvolvimento | `docs/botany-age.md` | Implemented slice; boundaries in entry |
| 41 | Copas volumétricas e ramificação | `docs/botany-architecture.md`, `docs/botany-branching.md`, `docs/botany-rs.md` | Implemented slice; boundaries in entry |
| 42 | Raízes, contrafortes e colo | `docs/botany-roots.md`, `docs/botany-buttress.md`, `docs/botany-collar.md` | Implemented slice; boundaries in entry |
| 43 | Dinâmica botânica em laboratório | `docs/botany-integration.md`, `docs/botany-collar.md` | Implemented slice; boundaries in entry |
| 44 | Cobertura de chão e mato alto | `docs/groundcover-assets.md`, `docs/tallgrass-assets.md` | Implemented slice; boundaries in entry |
| 45 | Corpo humano paramétrico | `docs/character-base.md`, `docs/character-nose.md` | Implemented slice; boundaries in entry |
| 46 | Rosto, orelhas e manipuladores 3D | `docs/character-controls.md`, `docs/character-cupid.md`, `docs/character-nose.md`, `docs/character-smoothing.md` | Implemented slice; boundaries in entry |
| 47 | Rig, clipes, retarget e IK | `docs/npc-presentation-complete.md` | Implemented slice; boundaries in entry |
| 48 | Inteligência e personalidade explicáveis | `docs/npc-mind.md`, `docs/npc-v1-closure.md` | Implemented slice; boundaries in entry |
| 49 | Tática, ordens e cooperação | `docs/npc-tactics-cooperation.md` | Implemented slice; boundaries in entry |
| 50 | Necessidades, objetos e diálogo | `docs/npc-routines-interactions.md` | Implemented slice; boundaries in entry |
| 51 | Descoberta de rede e circulação | `docs/npc-navigation-generation.md`, `docs/npc-circulation.md`, `docs/npc-air-circulation.md` | Implemented slice; boundaries in entry |
| 52 | População, templates e identidade | `docs/spawn-templates.md`, `docs/instance-identity.md`, `docs/npc-population-scale.md` | Implemented slice; boundaries in entry |
| 53 | Geradores, regiões e ondas | `docs/population-generators.md`, `docs/population-waves.md`, `docs/spawn-regions.md`, `docs/weighted-population.md` | Implemented slice; boundaries in entry |
| 54 | Runtime para 1.000 jogadores | `docs/scalable-actor-runtime.md`, `docs/frame-admission.md`, `docs/joint-runtime-load.md` | Implemented slice; boundaries in entry |
| 55 | QUIC, sessões e autoridade | `docs/quic-transport.md`, `docs/network-session-runtime.md`, `docs/joint-network-load.md` | Implemented slice; boundaries in entry |
| 56 | Cliente remoto e apresentação | `docs/remote-controls.md`, `docs/remote-presentation.md` | Implemented slice; boundaries in entry |
| 57 | Checkpoints e continuidade de gameplay | `docs/npc-checkpoints.md` | Implemented slice; boundaries in entry |
| 58 | Save portátil canônico | `docs/portable-saves.md` | Implemented slice; boundaries in entry |
| 59 | Terrenos editáveis e cristas | `docs/terrain.md`, `docs/representative-world.md`, `docs/terrain-collision.md` | Implemented slice; boundaries in entry |
| 60 | Landforms: paredes, arcos e volumes | `docs/landforms.md` | Implemented slice; boundaries in entry |
| 61 | Coordenadas globais e residência | `docs/world-space.md`, `docs/world-residency.md`, `docs/incremental-residency.md` | Implemented slice; boundaries in entry |
| 62 | Planeta caminhável em escala terrestre | `docs/planet-runtime.md`, `docs/planet-appearance-audit.md` | Implemented slice; boundaries in entry |
| 63 | Passeio em primeira e terceira pessoa | `docs/player-walk.md`, `docs/player-perspectives.md` | Implemented slice; boundaries in entry |
| 64 | Comparação visual e autoria por evidência | `docs/image-similarity.md`, `docs/image-to-3d-integration.md` | Implemented slice; boundaries in entry |
| 65 | Áudio espacial e eventos de apresentação | `docs/npc-presentation.md`, `docs/npc-presentation-complete.md` | Implemented slice; boundaries in entry |

## Roadmap

M01–M16 preserve the project roadmap domains, updated with current foundations and remaining scope. IDs are identifiers, not dates, percent-complete measures or approved release order. Quests, complete encounter frameworks, production distribution, web port and full continuous multi-actor world integration remain future work. Current subsystem records distinguish technical implementation, artist/human acceptance, isolated experiment and production delivery.

## Media provenance

See media-provenance.json for each original path, original SHA-256, public output, dimensions and bytes. Sixteen WebP copies total 722,120 bytes at creation, resized to a maximum 1600×1200 with aspect ratio preserved. Originals were not modified. Captions distinguish planar landscape, opt-in planet, historical editor/age studies, technical NPC/network demonstrations and character-source attribution. No third-party reference photographs or generated concepts are presented as engine captures.

The approved opening sculpture and landscape remain the separate Three.js brand composition and retain their label. Still captures and an interactive, accessible gallery were selected here; no new recording or live engine window was needed. No engine build, GPU benchmark or regression suite was run for this editorial/website increment.


## Product narrative and selected imagery — 3 November revision

The creator selected two native landscape images, now retained under website/media and served as optimized horizon.webp / sunset.webp. These lead the product story, followed by procedural plant generation, atmospheric landscapes, NPC behavior, human/AI shared control and offline/optional multiplayer. The curated gallery has five images (valley, sunset, native fig tree, cottage, groundcover); diagnostic images remain within the relevant atlas entries. Total available media is now19, with provenance in media-provenance.json.

Ignis Horizon is the **creator-approved presentation name** for the integrated rendering stack, not a new engine API, renamed IGL format, or claim that the research branch is shipped. The renderer combines content sharing, residency, visibility/occlusion and atmosphere-aware representations. Source: docs/mesh-resources.md, minimal-vertex-stream.md, adaptive-occluders.md, fine-occlusion.md and atmosphere.md.

Product proof: docs/landscape-forest.md records30,000trees/60,006mesh instances; docs/atmosphere-landscape.md records18km to the far chain center (different scene); docs/adaptive-occluders.md records92.530→48.724ms (1.90x draw+GPU wait) at1728x1117 on M4Pro/Metal, not gameplay FPS. The source explicitly does not certify100km landscapes. The graphics-research physical-adaptive probe has a100000 far plane in a cropped experiment with normalized assets, not a100km production world. No blanket Nanite/Unreal comparison is published.

Plant generation is presented as an extensible rule/seed/organ construction system, with grass/groundcover and native palm/plane/eucalyptus/fig exemplars. src/botany/development.rs accepts authored age0.25..2000years; this supports the creative ancient-tree direction, not biological calibration or demonstrated fidelity for every species. Dedicated source contracts remain in the atlas. No engine code/build/window modified for this revision.


## Horizon / Nanite mechanism comparison

Creator accepted Horizon and requested architecture-level differentiation. New module horizon-architecture.js explains source→compile→view evaluation→representation, then compares five axes without claiming a substitute for all of Nanite or Unreal. Sources for Ignis: docs/atmosphere.md (before-upload material replacement, color/contour bounds, full-source fallback, edit invalidation), docs/atmosphere-silhouettes.md (contrast/transmittance), docs/minimal-vertex-stream.md (24B unquantized f32 eligible stream,48/64B fallback), docs/regional-lod.md (experimental/off), docs/mesh-resources.md and render_residency.rs (content-sharing/residency).

Epic primary documentation consulted03Nov2026:
- https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine — hierarchical clusters, view-dependent selection, fine-grained on-demand streaming, section materials with dynamic shader effects.
- https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-technical-details — configurable position quantization, residency and fallback.

Do not claim Nanite lacks atmosphere, editing, preserved source or occlusion; those are not exclusive Ignis capabilities, and Unreal's full renderer is not synonymous with Nanite. Horizon's described distinction is its explicit atmosphere-aware material/representation selection and associated authoring/inspection contract. Comparison is architectural, not an inter-engine benchmark or proof of superiority. Authoring controls belong to Ignis integration; no engine code changed.


## Current headline

User selected Ignite your ideas. English headline and title use that phrase; Portuguese adaptation is Dê vida às suas ideias. Hero body retains the spark/world progression. Both public hosts must serve the same static release.
