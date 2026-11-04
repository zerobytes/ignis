# What works today

Ignis is a pre-1.0, source-available research engine under the
[commercial source license](licensing.md). This page describes the public source
baseline, not a support or performance guarantee. Technical documents record
individual milestones; their earlier version numbers and next steps are historical.

The **[feature guide](features.md)** is the detailed capability map. This page
summarizes integration and validation scope; it does not replace the feature list.

| Area | Implemented scope | Boundary |
| --- | --- | --- |
| Shared commands and MCP | Typed dispatcher, structured inspection, stdio MCP, rendered capture, scene edits and undo | The engine does not include a hosted language model or autonomous cloud service. |
| Native editor and modeling | Projects, workspaces, file browsing, component editing, extrusion/inset/bevel, sweeps, organic junctions, sculpt brushes and local refinement | Interface coverage and artistic usability continue to evolve; this is not a complete DCC/CAD toolchain. |
| Ignis Horizon rendering | Native lighting/shadows, shared geometry, visibility/occlusion, GPU residency and atmosphere-aware distant representations | The integrated stack is distinct from experimental virtualized geometry/LOD, which remains off by default. No universal FPS claim. |
| Materials | Reusable procedural recipes, 12 configurable marble families, four continuous mountain-rock families, opt-in per-pixel relief normals and lossless fixed baked maps up to 1024, seeded per-instance runtime variation, layered surface channels, bump/parallax with self shadow, explicit static displacement bake, dielectric/metal response, environment lighting, transparent surfaces and thin glass | Recipe1 uses explicit planar rest coordinates; parallax is opaque and does not change silhouette/collision. Transparent composition and thin-glass optics are approximations; complete refraction, GI and volumetric optics are separate work. See [material recipes](material-recipes.md), [marble](marble-materials.md) and [rock](rock-materials.md). |
| Particles/VFX | Native versioned sources/placements, deterministic analytic emission/motion, instanced sprites using shared materials, parametric fire families, MCP/editor preview and source files | Ordinary-scene cosmetic backend. Smoke uses sprites; collisions, GPU simulation, ribbons, fluids and planetary support remain separate. Human artistic/input acceptance pending. See [particles](particles.md). |
| Spatial wind and free meshes | Opt-in analytic spatial field, seeded turbulence, gusts, currents, compact vortices/updrafts and rigid thin-surface aerodynamics with torque/convex contacts | External field, no fluid feedback or obstacle-flow solver. Ordinary offline scenes; planetary/network adapters refuse it. [Wind contract](wind.md). |
| Terrain and landforms | Native sampled/procedural terrain, ridges, authored profiles/cavities and opt-in terrain collision | Supported source families do not imply general CSG, erosion or unrestricted world streaming. |
| Botany | Native trees/palms, branching, age/vigor, roots/buttresses/collars, groundcover, tall grass and shrub libraries | Profiles are authored, not biologically universal. XPBD growth/pruning/deformation is a separate session laboratory, not networked gameplay plant physics. |
| Character authoring | Parametric body/posture, local facial controls and symmetric 3D handles, source-preserving regeneration and rig data | Supplied base derives from MakeHuman CC0 data. Complete facial animation, tissue dynamics and artistic acceptance remain separate. |
| Physics and player control | Offline fixed-step/continuous simulation, collisions, character controllers, triggers and first-/third-person cameras | Scenario quotas and body/collision contracts remain explicit; fixed simulation ticks are not frame-rate guarantees. |
| NPC behavior | Vision/hearing, memory/search/communication, intelligence/personality, hybrid movement, navigation/circulation, tactics, routines and structured dialogue | Evidence, body capabilities and bounded planning constrain decisions; this is not unrestricted human cognition or a global navmesh. |
| Combat and populations | Shared player/NPC ability executor, resources/attributes, buffs, casts/channeling, projectiles, areas/auras, resurrection, templates, seeded spawning and waves | Storage capacity is distinct from sustainable worst-case gameplay workload. Identity, life and slot reuse have separate contracts. |
| Animation and audio | Rigs/clips, explicit retarget mappings, visual IK, optional root motion, spatial PCM mixing/capture and optional device output | No universal automatic retarget, complete foot locking or final artistic mixing. Device output is disabled by default. |
| Persistence | Scenes, build-scoped replay, portable saves and explicit validation | Native physics checkpoints and portable saves have different portability contracts. |
| Networking | Optional QUIC/TLS host/client, replication and loopback load experiments | WAN quality, production MMO scale and universal frame-rate targets are not established. |
| Planetary walking and viewing | Opt-in spherical source, resident tiles, transported player physics, surface-to-globe camera and budgeted conforming spatial tile LOD | One-player experimental integration. Terrain appearance is still under revision; cold preparation is synchronous. No general asynchronous multi-actor streaming or accepted planetary art. |

## Platform and visual acceptance

Native work is centered on **macOS Apple Silicon/Metal**. Linux CI checks and
Windows source compatibility do not establish interactive desktop acceptance.
GPU comparisons do not establish artistic quality, input handling or a universal
FPS result. No released installer or store distribution is implied by this source
publication.

The README's Horizon landscape images and the experimental planetary views
represent different presentation scopes. A selected landscape capture does not
establish acceptance of the planet's terrain appearance or every workload.
The website's abstract Three.js artwork is a brand illustration, separate from
the Rust engine. Engine screenshots are identified in their captions.

## Useful next contributions

- Reproducible first-run reports on additional platforms and GPUs.
- Concrete MCP client interoperability reports and smaller executable examples.
- Clearer onboarding and translations of the existing technical contracts.
- Focused fixes with a minimal reproduction and compatibility notes.

Research directions include richer continuous-world residency/LOD, planetary
terrain appearance, [advanced particles/VFX and fluids](particle-vfx-requirements.md),
and more mature authoring workflows. These are directions,
not announced release dates. Discuss a large contribution before implementing it.

See [the quickstart](quickstart.md), [architecture](architecture.md),
[NPC capability matrix](npc-v1-closure.md) and [contributing](../CONTRIBUTING.md).
