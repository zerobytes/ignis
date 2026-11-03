# Ignis feature guide

Explore the capabilities available in the public engine source, organized by what you can create and control. Start with the [quickstart](quickstart.md), browse the [website](https://ignis.web.app/), or return to the [README](../README.md).

**How to read this guide:** unmarked entries are integrated native capabilities with their own contracts, not production-readiness guarantees. **Opt-in**, **Experimental**, **Laboratory** and **Research** identify narrower integration scopes. Expand each section’s scope notes for limits. Technical contracts are mostly in Portuguese; the overview here is in English.

For exact command arguments, discover `engine_command` through MCP `tools/list`. A feature entry is an editorial grouping, not one command or a promise that every combination is supported. [Validation and platform status](status.md).

[Ignis Horizon rendering](#ignis-horizon-rendering) | [Materials and surface detail](#materials-and-surface-detail) | [Worlds, terrain and planets](#worlds-terrain-and-planets) | [Procedural botany](#procedural-botany) | [Parametric characters](#parametric-characters) | [Modeling and sculpting](#modeling-and-sculpting) | [Physics and player control](#physics-and-player-control) | [NPC perception and personality](#npc-perception-and-personality) | [Navigation, tactics and interaction](#navigation-tactics-and-interaction) | [Abilities and combat](#abilities-and-combat) | [Populations and spawning](#populations-and-spawning) | [Animation and audio](#animation-and-audio) | [Multiplayer and actor runtime](#multiplayer-and-actor-runtime) | [Editor, MCP and persistence](#editor-mcp-and-persistence)

## Ignis Horizon rendering

Horizon is the presentation name for the integrated native rendering stack: geometry sharing, visibility, residency, lighting and atmosphere-aware representations. It does not rename the experimental IGL path or imply a complete Nanite equivalent.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Native rendering and capture | Native wgpu renderer with cameras, directional light, shadows, procedural materials, dielectric and metallic response, prefiltered environment and atmosphere. The editor selects objects and components; offscreen captures use the same pipeline. | [Procedural environment](procedural-environment.md) |
| Directional shadows, cameras and outlines | Directional shadows and environment have separate costs and sources. Picking and outlines respect visible coverage; guides and selection are observation tools, not NPC knowledge. | [Directional shadows](directional-shadows.md) · [Atmosphere silhouettes](atmosphere-silhouettes.md) |
| Shared geometry and instancing | Repeated static assets share GPU geometry while transform, tint and material vary per instance. Authoring capacity no longer depends on the old 16-asset or 32-instance caps. | [Mesh resources](mesh-resources.md) · [Small geometry cost](small-geometry-cost.md) |
| Conservative visibility | Camera and light reject candidates outside coverage before preparation and submission. Bounds derive from source and pose, including components and articulated presentation. | [Render visibility](render-visibility.md) |
| GPU occlusion | Current depth, reverse-Z, conservative reduction and stable compaction avoid drawing genuinely occluded instances. Small candidates can use 4×4 blocks; others retain 16×16 or fallback. | [Render occlusion](render-occlusion.md) · [Fine occlusion](fine-occlusion.md) · [Adaptive occluders](adaptive-occluders.md) |
| GPU residency and minimal vertex streams | Eligible static geometry remains resident, including large unique meshes. Shaders without procedural channels use minimal position/normal streams; absent fields are not prepared unnecessarily. | [Minimal vertex stream](minimal-vertex-stream.md) · [Static residency](static-residency.md) |
| Atmosphere-aware distant representations | Compiled packages preserve source and admit cheaper representations when contribution bounds allow. Atmosphere separates sky and mountains; optional edge finishing has its own cost. | [Atmosphere](atmosphere.md) · [Atmosphere silhouettes](atmosphere-silhouettes.md) · [Render optimization comparison](render-optimization-comparison.md) |
| Virtualized geometry and LOD research — **Research / off by default** | The Ignis Nanites research track includes candidate compilation while preserving boundaries and components; a verifier recomputes surface/normal correspondence bounds. Runtime has error criteria and source fallback. | [Automatic lod](automatic-lod.md) · [Regional lod](regional-lod.md) |

<details>
<summary>Scope and validation notes</summary>

- **Native rendering and capture:** Native desktop validation is centered on macOS/Metal. Linux headless CI does not establish desktop/GPU acceptance; a browser/WebAssembly engine runtime is not delivered.
- **Directional shadows, cameras and outlines:** Bounded directional shadows do not equal global illumination or complete physical transparency.
- **Shared geometry and instancing:** Sharing geometry does not eliminate the cost of drawing millions of triangles across copies.
- **Conservative visibility:** Frustum culling differs from occlusion and does not simplify surviving geometry.
- **GPU occlusion:** Specified M4 Pro/Metal scenes and resolutions; draw+wait is not a GPU timestamp or gameplay FPS.
- **GPU residency and minimal vertex streams:** CPU source remains resident. Buffer savings are not total game RAM measurements.
- **Atmosphere-aware distant representations:** The mountain comparison reduced buffers, not curved triangles; timing varied by scene.
- **Virtualized geometry and LOD research:** Experimental and disabled by default. The forest produced no useful package; this is not complete Nanite or demonstrated universal superiority.

</details>

## Materials and surface detail

Use one material system across authored meshes, architecture, vegetation and character presentation.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Procedural materials and surface detail | Rest-space 3D fields describe color, bump, roughness and surface detail. Veins, grooves and triplanar patterns follow form without sliding through world space during animation. | [Procedural materials](procedural-materials.md) · [Surface details](surface-details.md) · [Layered patterns](layered-patterns.md) · [Triplanar details](triplanar-details.md) |
| Dielectrics, metals and environment lighting | GGX surface response, roughness, Fresnel and metallicity combine with direct light and prefiltered procedural environment. One material system serves wood, stone, leaves or metal. | [Dielectric response](dielectric-response.md) · [Metallic response](metallic-response.md) · [Procedural environment](procedural-environment.md) |
| Transparency and thin glass — **Opt-in** | Author opaque, cutout, translucent and additive surfaces, emissive response and thin-glass absorption/Fresnel through the shared material system. Opaque and cutout coverage participates in depth; transparent composition uses weighted blended OIT. | [Material transparency](material-transparency.md) |

<details>
<summary>Scope and validation notes</summary>

- **Procedural materials and surface detail:** Bump does not alter physical geometry; a pattern is not constitutive material simulation.
- **Dielectrics, metals and environment lighting:** This is not ray tracing, dynamic GI, HDR import or complete local reflections.
- **Transparency and thin glass:** Thin glass and overlapping transparency are approximations. Spatial refraction, transmission blur, caustics, colored shadows and volumetric optics are not implemented by this foundation.

</details>

## Worlds, terrain and planets

Build editable planar landscapes and landforms, or explore the opt-in planetary runtime. Terrain generation, visual residency, physical collision and general world streaming have separate contracts.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Editable terrain and ridges | Fixed sampled and procedural sources use one sampler. Raising, lowering, leveling, noise and anisotropic Ridge generate native topography with editable recipes and a defined datum. | [Terrain](terrain.md) · [Representative world](representative-world.md) · [Terrain collision](terrain-collision.md) |
| Landforms, profiles and cavities | Concave profiles and parallel sections create walls, columns, ledges, peaks and real cavities. Caps and annuli preserve openings, with validation before mesh publication. | [Landforms](landforms.md) |
| Global coordinates and region planning | Integer-cell addresses and local offsets separate global position from resident-frame precision. Interest/dependency planning tracks region load/removal intentions without treating a seed as identity. | [World space](world-space.md) · [World residency](world-residency.md) · [Incremental residency](incremental-residency.md) |
| Planetary walking runtime — **Experimental** | The opt-in runtime installs/removes tiles across six faces, transports origin/frame and keeps gravity, camera and light radial. KCC walks the physical source; visual residency is separate. | [Planet runtime](planet-runtime.md) · [Planet appearance audit](planet-appearance-audit.md) |
| Surface-to-globe camera and tile LOD — **Experimental** | Move the view from the walking player to the globe using shared native/MCP controls. Six-face conforming tiles refine within an explicit budget; high/low coordinates maintain visual precision across scales and inspection reports quality-limited coverage. | [Planet zoom](planet-zoom.md) · [Planet runtime](planet-runtime.md) |

<details>
<summary>Scope and validation notes</summary>

- **Editable terrain and ridges:** Heightfields do not represent ceilings/caves; planar Ridge is not yet integrated into the current planet source.
- **Landforms, profiles and cavities:** This is not CSG/boolean union, geological erosion or universal branching caves.
- **Global coordinates and region planning:** The general planner is a read/contract layer; it is not unrestricted production multi-actor streaming.
- **Planetary walking runtime:** One Player, without active saves/multiplayer/authoring edits. Synchronous preparation; mountain appearance is under revision.
- **Surface-to-globe camera and tile LOD:** Experimental, opt-in and tied to the planetary runtime. Synchronous preparation, one-player integration and unfinished terrain appearance remain explicit; this is not general multi-actor streaming or a performance guarantee.

</details>

## Procedural botany

Plants retain native construction data: organs, oriented curves, sections, contours and seeded recipes. Trees, palms, grass, shrubs and root structures are editable sources.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Native procedural plant generation | A plant starts as a graph of organs, oriented curves, sections and contours—not an opaque mesh. The Rust compiler publishes native geometry and retains the editable recipe in the scene. | [Botany integration](botany-integration.md) · [Botany reference refinement](botany-reference-refinement.md) |
| Plant age, vigor and development | Age in years coordinates height, canopy, thickening, organ birth and current foliage. Vigor modifies development; palms do not automatically inherit the woody growth law. | [Botany age](botany-age.md) |
| Canopies and branching architecture | Subcanopy partitions, direction, load-based radii, interior buds and additional orders generate branching architecture. Terminal forks join three collars; lateral grafts and sheaths have distinct roles. | [Botany architecture](botany-architecture.md) · [Botany branching](botany-branching.md) · [Botany rs](botany-rs.md) |
| Roots, buttresses and collars | Radial, taproot and fibrous systems produce editable axes. Buttresses add walls, ridges, meanders and ribs; the collar connects roots and trunk through closed native skin. | [Botany roots](botany-roots.md) · [Botany buttress](botany-buttress.md) · [Botany collar](botany-collar.md) |
| Groundcover and tall grass | Authored libraries combine blades, stems, spikes, panicles, plumes and flowers. Real source/distribution variations replace merely scaling one tuft. | [Groundcover assets](groundcover-assets.md) · [Tallgrass assets](tallgrass-assets.md) |
| Procedural shrub libraries | Generate seeded woody shrubs with editable stems, branches, leaves and flowers. Collections include compact/open silhouettes, colored foliage, desert forms and dense topiary profiles alongside the existing grass libraries. | [Shrub assets](shrub-assets.md) · [Colored shrub assets](colored-shrub-assets.md) · [Reference shrub assets](reference-shrub-assets.md) |
| Botanical dynamics laboratory — **Laboratory** | An XPBD laboratory keyed by identity, mass and momentum supports deformation, growth/pruning and contact proxies. Rest curves are the same source used by native authoring. | [Botany integration](botany-integration.md) · [Botany collar](botany-collar.md) |

<details>
<summary>Scope and validation notes</summary>

- **Native procedural plant generation:** Profiles are authored, not universal botanical calibration or automatic photo reconstruction.
- **Plant age, vigor and development:** Age is profile configuration, not real dating or decades of temporal simulation.
- **Canopies and branching architecture:** Species/photo resemblance requires separate artistic acceptance; generation does not identify species.
- **Roots, buttresses and collars:** Soil interaction, anastomosis and universal tangential continuity are not implemented.
- **Groundcover and tall grass:** Names describe silhouettes, not calibrated species. Instancing does not guarantee zero per-copy cost.
- **Procedural shrub libraries:** Collections are authored plant families, not scientific species identification or an unrestricted ecology simulation. These shrub axes meet by contact/overlap rather than universally welded branching.
- **Botanical dynamics laboratory:** Session laboratory separate from Rapier gameplay, saves and networking; not universal constitutive tissue or soil.

</details>

## Parametric characters

Author body proportions, posture and local face controls, then regenerate the native character with explicit preservation policies. The supplied base retains its MakeHuman CC0 provenance.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Parametric human body | A native source combines proportions, body mass, musculature, posture and local adjustments. Regeneration preserves recipe/bake and rig under explicit preservation policies. | [Character base](character-base.md) · [Character nose](character-nose.md) |
| Face, ears and 3D handles | Local XYZ fields adjust anatomical regions with optional symmetry while preserving defined neighborhoods. Lips, Cupid’s bow, nose and ears have dedicated controls and bounds. | [Character controls](character-controls.md) · [Character cupid](character-cupid.md) · [Character nose](character-nose.md) · [Character smoothing](character-smoothing.md) |

<details>
<summary>Scope and validation notes</summary>

- **Parametric human body:** Graphic base derived from MakeHuman CC0 data; builds do not certify anatomy or human art.
- **Face, ears and 3D handles:** This is not facial capture, universal expressions, complete MetaHuman or entirely original Ignis anatomy.

</details>

## Modeling and sculpting

Create and refine real mesh geometry through the native editor and shared commands, from individual components to connected organic structures.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Mesh component modeling | Vertices, edges, faces and islands are selectable through session/revision references. Extrusion, inset and bevel produce real geometry through a shared editor/MCP kernel. | [Editor modeling](editor-modeling.md) · [Mesh topology](mesh-topology.md) |
| Sculpt brushes and local refinement | Draw, move, smooth, flatten, pinch, crease, scrape, fill and relax are geometric operations. Brushes can refine the affected region, with strength/radius controls and one undo per gesture. | [Sculpt brushes](sculpt-brushes.md) · [Editor sculpt](editor-sculpt.md) |
| Curves, profiles and sweeps | Editable profiles and trajectories compose branches, structures and swept forms. Sampling, radius, taper and oriented frames remain authored data, with resolution and domain inspection. | [Mesh sweep](mesh-sweep.md) · [Plant trajectories](plant-trajectories.md) · [Curved plants](curved-plants.md) |
| Organic junctions and regeneration | Collars, branches and connected surfaces use native junctions with closure validation. Canonical sources and material addresses allow regeneration with detail preservation when valid correspondence exists. | [Organic continuity](organic-continuity.md) · [Botany collar](botany-collar.md) · [Mesh preservation](mesh-preservation.md) |
| Composite objects and hierarchies | Explicit bindings organize pieces without destroying instancing. Groups can be rejected by conservative bounds; partially visible or deformed groups retain per-piece tests. | [Mesh objects](mesh-objects.md) · [Mesh attachments](mesh-attachments.md) · [Mesh island transform](mesh-island-transform.md) |

<details>
<summary>Scope and validation notes</summary>

- **Mesh component modeling:** Curved bevels, concave insets and global self-intersection detection are not current capabilities.
- **Sculpt brushes and local refinement:** Masks, tablet support, universal remeshing and full DCC workflows remain separate.
- **Curves, profiles and sweeps:** Sweep families have validated domains; they are not an unrestricted CAD kernel.
- **Organic junctions and regeneration:** There is no general boolean union or universal C1-continuity guarantee.
- **Composite objects and hierarchies:** Grouping is not collective appearance LOD or a boolean union of pieces.

</details>

## Physics and player control

Run local gameplay with physics, character control, trigger events and session cameras. The engine operates without a hosted service or a mandatory AI client.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Fixed-step and continuous simulation | Rapier physics with gravity, collisions, fixed 60 Hz simulation and per-character input. Recorded execution preserves replay; continuous mode keeps gameplay advancing without an unbounded command history. | [Continuous runtime](continuous-runtime.md) |
| Trigger volumes and events | AABB triggers, explicit targets, post-physics entry and exit, and bounded event history. Visibility can change independently of collision or perception. | [Commands](commands.md) |
| First- and third-person exploration | Session camera follows Player identity/life with yaw/pitch, distance and shoulder controls. Looking and movement use operations shared by native input and MCP without turning camera motion into body teleportation. | [Player walk](player-walk.md) · [Player perspectives](player-perspectives.md) |

<details>
<summary>Scope and validation notes</summary>

- **Fixed-step and continuous simulation:** The fixed step is a simulation contract, not an FPS guarantee for every scene.
- **Trigger volumes and events:** Current volumes are fixed and unrotated; detection is discrete.
- **First- and third-person exploration:** A complete authored human Player avatar is a separate integration; cameras do not imply FPS arm animation.

</details>

## NPC perception and personality

NPC decisions combine available evidence, relationships, memory, intelligence and independent personality traits. Developer inspection stays separate from what an NPC knows.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Visual perception | Volumetric XYZ vision with yaw and pitch, range, field of view and occlusion. Gradual per-individual confirmation separates suspicion from recognition and memory. | [Commands](commands.md) |
| Hearing and physical footsteps | Anonymous acoustic clues preserve emission position and time, acoustic transmission and attention arbitration. Footsteps derive from physical motion and locomotion profile. Presentation has a separate spatial-audio and PCM mixing system. | [Npc presentation complete](npc-presentation-complete.md) |
| Factions, relationships and autonomous targets | NPCs can operate without targeting the player, perceive other NPCs and select observed hostiles. Factions, directional attitudes and hysteresis govern eligibility and switching. | [Commands](commands.md) |
| Memory, search and communication | Last known positions, expiry, search hypotheses and explicit budgets. Allies can share confirmed observations through channels, XYZ radius and preserved TTL. | [Commands](commands.md) |
| Inspectable decision policies | Priorities and conditions compose response, fleeing, guarding, waiting, patrol, tactics, interaction and abilities. Cognition and traits influence choices and effort; inspection exposes scores, reasons and next evaluation without implicit omniscience. | [Npc v1 closure](npc-v1-closure.md) |
| Intelligence and personality | Intelligence controls deliberation effort and cadence without granting wall vision. Anxiety, caution, courage, curiosity, aggression, sociability and other traits vary independently per individual. | [Npc mind](npc-mind.md) · [Npc v1 closure](npc-v1-closure.md) |

<details>
<summary>Scope and validation notes</summary>

- **Visual perception:** Sensor orientation does not animate the body. Omniscience is an explicit policy only.
- **Hearing and physical footsteps:** Hearing does not track a hidden source. Audio-device output is optional and disabled by default.
- **Factions, relationships and autonomous targets:** Eligibility never supplies a hidden location.
- **Memory, search and communication:** Reports do not cascade; a search hypothesis never becomes a real observation.
- **Inspectable decision policies:** NPC v1 has technical closure; art, perceived behavior and load scenarios retain separate acceptance gates.
- **Intelligence and personality:** No mandatory LLM; traits are an authored model, not complete human psychology.

</details>

## Navigation, tactics and interaction

Combine body-aware movement with generated routes, local circulation, tactics, cooperative orders, authored routines and structured dialogue.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Ground navigation and reserved passages | Planar and spatial navigation, partial routes, recovery, local connectors and exclusively reserved passages. Ground control queries real geometry for slopes and steps; circulation handles neighbors while preserving physical authority. | [Ground local connections](ground-local-connections.md) |
| Walk, jump, glide and fly | XYZ network with walk/jump/glide/fly capabilities, local connections, composed support and partial approaches. Hybrid locomotion integrates ground KCC and aerial segments; curved flight controls acceleration, turning, hover and orbit. | [Npc hybrid mobility](npc-hybrid-mobility.md) |
| Navigation generation and actor circulation | Networks can be generated from declared geometry/regions, admitted by body and capability. Ground and air circulation query anonymous neighbors; committed actions preserve their movement authority. | [Npc navigation generation](npc-navigation-generation.md) · [Npc circulation](npc-circulation.md) · [Npc air circulation](npc-air-circulation.md) |
| Tactics, orders and cooperation | Desired range, cover and flanking produce destinations constrained by navigation and known evidence. Group orders, formations and support reservations preserve ownership, identity and life. | [Npc tactics cooperation](npc-tactics-cooperation.md) |
| Patrols, schedules and territory | XYZ patrols, loop/ping-pong, waiting and resumption coexist with cyclic schedules, resource-driven needs and points of interest. Territory constrains clue acquisition and physical return under explicit rules. | [Npc routines interactions](npc-routines-interactions.md) |
| Needs, object interactions and dialogue | Cyclic schedules and resource-based needs drive routines. Interactions apply typed effects to objects or game state; structured dialogues retain revisions and life references. | [Npc routines interactions](npc-routines-interactions.md) |

<details>
<summary>Scope and validation notes</summary>

- **Ground navigation and reserved passages:** Generated networks have declared regions and budgets. This is not a global navmesh or a universal deadlock-free guarantee.
- **Walk, jump, glide and fly:** Actor capability and physical viability are revalidated. No full aerodynamics or unbudgeted free-space navigation.
- **Navigation generation and actor circulation:** Bounded networks are not a world navmesh. Committed jump/glide segments do not receive arbitrary later avoidance.
- **Tactics, orders and cooperation:** A suggested position does not prove a viable route; there is no omniscient global danger sensor.
- **Patrols, schedules and territory:** Schedules are game-authored; circular territory is not geopolitics or a universal social simulation.
- **Needs, object interactions and dialogue:** This is not autonomous natural language or unrestricted memory/Scene mutation.

</details>

## Abilities and combat

Players and NPCs use the same gameplay executor for costs, casts, effects, targets, cooldowns and cancellation.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Shared ability executor | Player and NPC share one executor: scaled integer resources, atomic costs, casts, cooldowns and cancellation. Movement can remain free, interrupt or be locked by a cast. | [Commands](commands.md) |
| Damage, healing, death and resurrection | Damage/healing and death are transactional, cancelling casts, reservations and motors. Explicit resurrection preserves identity, resources and cooldowns, advances life index and requires fresh evidence for observers unaware of the return. | [Resurrection](resurrection.md) |
| Projectiles and free aim | Straight XYZ projectiles with AABB sweeps and first contact. Aim by direction, point or cursor on an explicit plane; automatic aim uses confirmed, frozen visual evidence. | [Commands](commands.md) |
| Periodic effects and dispelling | Damage, healing and resource schedules; refresh or independent executions; category-based dispelling, limits and protection. Channeling, external interruption and attributes/buffs add composition without duplicating the ability executor. | [Attributes](attributes.md) |
| Attributes, buffs and damage components | Scaled integer resources and configurable numeric/boolean attributes support temporary effects and modifiers. Damage is composed of classified components, with separate source and recipient factors. | [Attributes](attributes.md) · [Damage components](damage-components.md) · [Outgoing damage](outgoing-damage.md) |
| Channeling and interruptions | An ability can prepare, channel pulses, be cancelled or receive external interruption. Initial or per-pulse charging, cooldown and interruption protection have distinct semantics. | [Channeling](channeling.md) · [Interrupts](interrupts.md) · [Dispel](dispel.md) |
| Areas, auras and ally support | Instant spheres, persistent volumes and pulsing auras reuse relationships, occlusion and per-recipient transactions. NPCs choose injured or dispellable allies from authorized evidence. | [Instant area](instant-area.md) · [Persistent area](persistent-area.md) · [Auras](auras.md) · [Ally triage](ally-triage.md) · [Dispel triage](dispel-triage.md) |

<details>
<summary>Scope and validation notes</summary>

- **Shared ability executor:** NPC policies revalidate conditions and evidence; an approach does not guarantee a viable route.
- **Damage, healing, death and resurrection:** Resurrecting the same individual differs from template spawning or slot reuse.
- **Projectiles and free aim:** Homing, ballistics and target prediction remain separate. Areas and tactics have dedicated contracts described in this atlas.
- **Periodic effects and dispelling:** Categories, effects, duration and quotas are explicit; effects do not execute arbitrary scripts.
- **Attributes, buffs and damage components:** Storage limits do not imply sustained worst-case 60 Hz.
- **Channeling and interruptions:** Protection does not implicitly block death, self-cancellation or movement policy.
- **Areas, auras and ally support:** Pulsing auras do not imply continuous enter/exit callbacks or universal tactical immunity.

</details>

## Populations and spawning

Use reusable templates, seeded distributions and explicit lifetime/identity rules to populate a scenario.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Actor templates and persistent identity | Spawning and removing actors are runtime operations separate from the document. Templates initialize components; generation references prevent old memories from being inherited by a new slot occupant. | [Spawn templates](spawn-templates.md) · [Instance identity](instance-identity.md) · [Npc population scale](npc-population-scale.md) |
| Population generators, regions and waves | Continuous population and finite waves share templates, quotas and identity. Seeded regions and weights select candidates; generator activation, pause and control are explicit. | [Population generators](population-generators.md) · [Population waves](population-waves.md) · [Spawn regions](spawn-regions.md) · [Weighted population](weighted-population.md) |

<details>
<summary>Scope and validation notes</summary>

- **Actor templates and persistent identity:** 1,000 players does not prove1,000 NPCs with full sensing and planning.
- **Population generators, regions and waves:** Spawning does not guarantee a viable route or behavior at every chosen point.

</details>

## Animation and audio

Connect actors to rigs, clips, retarget mappings, visual IK, optional root motion and spatial presentation audio. Physical motion and presentation keep separate ownership.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Rigs, clips, retargeting and IK | Native hierarchies, weights and clips drive individual presentation. Authored retargeting maps rest poses and scale; visual IK queries physical contact without moving the collider to correct a pose. | [Npc presentation complete](npc-presentation-complete.md) |
| Spatial audio and presentation events | Presentation events feed a PCM mixer and device-free capture. CPAL output is optional; voices, distance and public state belong to presentation without feeding acoustic knowledge into sensors. | [Npc presentation](npc-presentation.md) · [Npc presentation complete](npc-presentation-complete.md) |

<details>
<summary>Scope and validation notes</summary>

- **Rigs, clips, retargeting and IK:** Mapping is explicit; no automatic anatomical retarget or universal foot locking/pelvis solve.
- **Spatial audio and presentation events:** PCM capture does not certify speakers/devices, a full music system or final artistic mixing.

</details>

## Multiplayer and actor runtime

Networking is optional. Authoritative sessions, scoped client operations and replication reuse the offline gameplay contracts.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Multi-actor load scenarios | Identity, queues, per-actor controls, local transactions and delivery pools were scaled beyond the initial laboratory. Continuous mode does not depend on an endless replay history. | [Scalable actor runtime](scalable-actor-runtime.md) · [Frame admission](frame-admission.md) · [Joint runtime load](joint-runtime-load.md) |
| QUIC/TLS sessions and authority | Real QUIC transport with TLS, scoped ownership, deduplication, monotonic ACKs and resumption. Clients send allowed operations; connection does not grant administrative world access. | [Quic transport](quic-transport.md) · [Network session runtime](network-session-runtime.md) · [Joint network load](joint-network-load.md) |
| Remote client controls and presentation | Public content, poses and remote controls reach a native window/capture without replicating private decision state. Picking, aiming, abilities and camera use the same ownership contracts. | [Remote controls](remote-controls.md) · [Remote presentation](remote-presentation.md) |

<details>
<summary>Scope and validation notes</summary>

- **Multi-actor load scenarios:** Specified local/loopback load; maximum storage limits do not guarantee 60Hz for arbitrary combat.
- **QUIC/TLS sessions and authority:** WAN, production authentication, prediction and interpolation remain separate deliveries.
- **Remote client controls and presentation:** No promised web client or validated internet experience; current evidence is native and loopback.

</details>

## Editor, MCP and persistence

Work visually or through typed commands, inspect results, capture real rendered pixels and save through the appropriate scene/runtime contract.

| Capability | What you can do | Technical contracts |
| --- | --- | --- |
| Native editor, projects and file browsing | Native editor with configurable areas, menus, project library and integrated file browsing. Creating, opening, favoriting and discovering projects shares documented operations with agents. | [Editor ui](editor-ui.md) · [Projects](projects.md) · [Editor workflows](editor-workflows.md) |
| Versioned scenes and authored sources | Versioned scenes preserve IDs, references, procedural sources and configuration. The seeded room recipe remains available alongside native terrain, plant and character documents. Projects collect scenes and an executable entry point. | [Projects](projects.md) |
| Transactional edits, undo and redo | Typed batch edits, validation before commit, final-state reference checks, expected revisions and transactional undo/redo. Authored documents stay separate from simulation. | [Commands](commands.md) |
| Typed commands and MCP | Shared typed dispatcher, command-derived schemas and structured results. MCP over stdio and native controls reach the same API. Projects run without the agent that created them. | [Commands](commands.md) |
| Structured inspection and visual debugging | Queryable state, sources, reasons, resources, routes and decisions. Vision, hearing, noise and route guides, vitality bars and captures through the same renderer. | [Commands](commands.md) |
| Replay and scenario verification | Tick-inspectable replay and isolated captures without changing the live session. Scenarios with typed checks, finite budgets and reports distinguishing behavior failures from transport errors. | [Commands](commands.md) |
| Native gameplay checkpoints | Durable checkpoints preserve individuals, memories, routines, clocks and native physics. Loading prepares and validates a candidate before replacing the world and neutralizes external input tokens. | [Npc checkpoints](npc-checkpoints.md) |
| Portable saves | PSV1 stores Ignis primary state without serializing Rapier arenas, handles or manifolds. The adapter rebuilds physics and AABBs without advancing a tick, publishing a validated world. | [Portable saves](portable-saves.md) |
| Image comparison and reconstruction workflows | Native captures can be compared by regions/masks with RGB8, luminance SSIM and explicit coverage. Image-based authoring keeps construction inventories, hypotheses and revisions separate from results. | [Image similarity](image-similarity.md) · [Image to 3d integration](image-to-3d-integration.md) |

<details>
<summary>Scope and validation notes</summary>

- **Native editor, projects and file browsing:** The UI capture is historical; it does not promise identical current menus.
- **Versioned scenes and authored sources:** Each generator has its own contract; universal whole-game generation is not implemented.
- **Transactional edits, undo and redo:** Undo reverses authorship, not physics time. Applied edits restart the paused simulation.
- **Typed commands and MCP:** MCP is the control plane; simulation advances internally without an AI call per frame.
- **Structured inspection and visual debugging:** Developer inspection does not enter NPC knowledge.
- **Replay and scenario verification:** Replay requires the same build; cross-platform determinism is not a current guarantee.
- **Native gameplay checkpoints:** Native physics snapshots have explicit version/platform scope; they are not universally portable saves.
- **Portable saves:** Format portability does not prove bit-exact trajectories across backends or tested support on other systems.
- **Image comparison and reconstruction workflows:** SSIM is not a3D-fidelity percentage; study reconstructions retain artistic limitations.

</details>

## Research and roadmap

The current engine provides foundations for larger systems. These plans are not included as delivered features above.

| Direction | Current boundary |
| --- | --- |
| Particles, VFX and fluid simulation | [Requirements and staged plan](particle-vfx-requirements.md). Shared surface/transparency materials exist; a complete emitter/solver authoring system does not. |
| General continuous worlds | [World and portable-save direction](continuous-world-and-portable-saves.md). Local/global coordinates, planning and an opt-in planetary session exist; unrestricted asynchronous multi-actor streaming remains work ahead. |
| Virtualized geometry | [Automatic LOD](automatic-lod.md) and [regional LOD](regional-lod.md) are experimental, off by default; no universal Nanite parity or speedup is established. |
| Complete character pipeline | Parametric bodies and presentation exist; universal species authoring, complete facial animation, dynamic tissues and a general DCC importer remain separate work. |
| Quests and encounter frameworks | Gameplay primitives, NPC interactions and waves exist. Complete questlines, dungeon/raid frameworks and finished game systems are future integrations. |
| Production distribution and browser runtime | Public source and desktop build instructions exist. Accepted desktop platforms, installers, game export and a WebAssembly runtime require their own delivery and validation. |

**Scope matters:** a successful compile, a loopback load test, a native pixel comparison and artistic acceptance are different evidence. Per-system documents record the measured workloads; they do not establish universal FPS, WAN behavior or every combined workload.

**Keeping this current:** update this guide, the README overview and [status](status.md) when a public feature is added or its integration scope changes. Older milestone version numbers in linked contracts remain historical.
