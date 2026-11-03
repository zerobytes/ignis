# What works today

Ignis is a pre-1.0, source-available research engine under the
[commercial source license](licensing.md). This page describes the public source
baseline, not a support or performance guarantee. Technical documents record
individual milestones; their earlier version numbers and next steps are historical.

| Area | Implemented scope | Boundary |
| --- | --- | --- |
| Shared commands and MCP | Typed dispatcher, structured inspection, stdio MCP, rendered capture, scene edits and undo | The engine does not include a hosted language model or autonomous cloud service. |
| Native editor | Scene inspection, authoring, mesh tools, sculpting and project workflows | Interface coverage and artistic usability continue to evolve. |
| Procedural content | Native terrain, landforms, atmosphere, botanical structures and character controls | A generator's supported family is not a universal content model. |
| Gameplay | Offline simulation, physics, player controllers, NPC perception/memory/navigation and abilities | Specific scenarios have explicit quotas; combined workloads need their own validation. |
| Persistence | Scenes, build-scoped replay, portable saves and explicit validation | Native physics checkpoints and portable saves have different portability contracts. |
| Networking | Optional QUIC/TLS host/client, replication and loopback load experiments | WAN quality, production MMO scale and universal frame-rate targets are not established. |
| Planetary surface | Experimental spherical source, resident tiles and transported player physics | Visual terrain correction is open; asynchronous streaming, spatial geometric LOD and distant coverage are not completed guarantees. |

## Platform and visual acceptance

Native work is centered on **macOS Apple Silicon/Metal**. Linux CI checks and
Windows source compatibility do not establish interactive desktop acceptance.
GPU comparisons do not establish artistic quality, input handling or a universal
FPS result. No released installer or store distribution is implied by this source
publication.

The README's terrain image is from the planar expanded-world demo. Its visual
result should not be attributed to the experimental planetary implementation.
The website's abstract Three.js artwork is a brand illustration, separate from
the Rust engine. Engine screenshots are identified in their captions.

## Useful next contributions

- Reproducible first-run reports on additional platforms and GPUs.
- Concrete MCP client interoperability reports and smaller executable examples.
- Clearer onboarding and translations of the existing technical contracts.
- Focused fixes with a minimal reproduction and compatibility notes.

Research directions include richer continuous-world residency/LOD, planetary
terrain appearance and more mature authoring workflows. These are directions,
not announced release dates. Discuss a large contribution before implementing it.

See [the quickstart](quickstart.md), [architecture](architecture.md),
[NPC capability matrix](npc-v1-closure.md) and [contributing](../CONTRIBUTING.md).
