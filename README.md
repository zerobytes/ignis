<div align="center">
  <img src="website/dist/assets/lockup.webp" alt="Ignis Engine" width="460">
  <h3>A 3D engine humans and AI agents can build in together.</h3>
  <p>Build → run → inspect → capture → refine. One command API. One world.</p>
  <p>
    <a href="https://github.com/zerobytes/ignis/actions/workflows/ci.yml"><img src="https://github.com/zerobytes/ignis/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-Ignis_Commercial-orange" alt="Ignis commercial source license"></a>
    <img src="https://img.shields.io/badge/status-early_development-8b949e" alt="Early development">
  </p>
  <p><a href="docs/features.md">Feature guide</a> · <a href="https://ignis.web.app/">Website</a> · <a href="docs/quickstart.md">Quickstart</a> · <a href="docs/status.md">What works today</a> · <a href="https://github.com/zerobytes/ignis/discussions">Discussions</a> · <a href="README.pt-BR.md">Português</a></p>
</div>

![Ignis Horizon: a forested valley surrounded by layered mountain ranges](docs/media/horizon.webp)
***Ignis Horizon.** Forested valleys and mountain ranges rendered by the native Ignis engine. A creator-selected capture from the website.*

Ignis is a source-available **Rust 3D engine with Model Context Protocol (MCP)
authoring and a native visual editor**. A human can edit a scene while an agent
uses typed commands to inspect it, change it, step the simulation and request
real rendered captures. Both operate through the same dispatcher.

The core runs **offline**. No cloud account, API key or LLM is required to run
the engine. Connect your own MCP client when you want agent assistance.

## Why Ignis?

- **A shared, inspectable world.** MCP and the native editor share the command
  API, validation and state. Agents receive structured results and real pixels.
- **Procedural content you can edit.** Terrain, botanical structures, meshes
  and character parameters have native authoring data, rather than only a
  flattened image or opaque generated asset.
- **Gameplay beyond rendering.** Physics, character controllers, NPC perception,
  memory, navigation, abilities and offline simulation are integrated.
- **Reproducible experiments.** Explicit state inspection, bounded operations,
  undo, persistence and build-scoped replay make failures easier to investigate.

**Early development:** this is a working research engine, with breaking changes
and substantial unfinished work. It is not a production-ready replacement for
established engines. Current desktop validation is centered on macOS Apple
Silicon; other platforms need independent validation. See [status and limits](docs/status.md).

## What can you build?

Create editable landscapes and plant libraries, shape characters and meshes,
prototype NPC-driven gameplay, or automate authoring and verification through
an agent. The public source includes the systems below.

| Area | Available capabilities |
| --- | --- |
| **[Ignis Horizon rendering](docs/features.md#ignis-horizon-rendering)** | Native lighting and shadows, shared geometry, GPU visibility/occlusion, residency and atmosphere-aware distant representations. |
| **[Materials and surface detail](docs/features.md#materials-and-surface-detail)** | Procedural color, roughness, bump and triplanar detail; dielectric/metal response; cutout, translucency, additive surfaces and thin glass. |
| **[Particles and VFX](docs/features.md#particles-and-vfx)** | Reusable seeded effects, analytic motion and life curves, GPU instanced translucent/additive sprites, shared MCP/editor authoring and preview. |
| **[Spatial wind and free meshes](docs/wind.md)** | Seeded currents, gusts and curl vortices; rigid thin-surface aerodynamics, torque and convex contacts in ordinary offline scenes. |
| **[Worlds, terrain and planets](docs/features.md#worlds-terrain-and-planets)** | Editable terrain and ridges, landforms and cavities, terrain collision, global coordinates, plus experimental planetary walking and surface-to-globe tile LOD. |
| **[Procedural botany](docs/features.md#procedural-botany)** | Trees, palms, grass and shrubs built from organs and curves; branching, age/vigor, roots, buttresses and collars. Separate deformation/growth/pruning laboratory. |
| **[Parametric characters](docs/features.md#parametric-characters)** | Body proportions, posture, local face/ear/lip/nose controls, symmetric 3D handles and source-preserving regeneration. |
| **[Modeling and sculpting](docs/features.md#modeling-and-sculpting)** | Vertex/edge/face editing, extrusion/inset/bevel, sculpt brushes and local refinement, sweeps, organic junctions and composite objects. |
| **[Physics and player control](docs/features.md#physics-and-player-control)** | Fixed-step or continuous simulation, collision and character controllers, slopes/steps, triggers, first- and third-person exploration. |
| **[NPC perception and personality](docs/features.md#npc-perception-and-personality)** | Vision, hearing, footsteps, memory, search, relationships, communication, intelligence, independent traits and inspectable decisions. |
| **[Navigation, tactics and interaction](docs/features.md#navigation-tactics-and-interaction)** | Generated routes, ground/air circulation, walk/jump/glide/fly, cover/flanking, orders/formations, schedules, needs, objects and structured dialogue. |
| **[Abilities and combat](docs/features.md#abilities-and-combat)** | Resources and cooldowns, casts/channeling/interruptions, projectiles, damage/healing, buffs, dispelling, areas, auras, death and resurrection. |
| **[Populations and spawning](docs/features.md#populations-and-spawning)** | Actor templates, persistent individual identity, seeded/weighted regions, population generators and finite waves. |
| **[Animation and audio](docs/features.md#animation-and-audio)** | Rigs/clips, retarget mappings, visual IK and optional root motion; spatial PCM mixing, presentation events and optional device output. |
| **[Multiplayer and actor runtime](docs/features.md#multiplayer-and-actor-runtime)** | Optional authoritative QUIC/TLS sessions, scoped remote controls, replication/reconnect and documented local/loopback 1,000-player load experiments. |
| **[Editor, MCP and persistence](docs/features.md#editor-mcp-and-persistence)** | Projects, workspaces, file browsing, typed commands, undo/redo, real captures, image comparison, replay, checkpoints and portable saves. |

**[Read the full feature guide →](docs/features.md)** for concrete capabilities,
technical contracts and the scope of each experimental or laboratory system.
The [current status](docs/status.md) separates integrated features from research
and future work.

## Try it

Install [Rust](https://www.rust-lang.org/tools/install) (1.88 or newer) and your
platform's native development tools. A compatible GPU is required for windows
and rendered captures; inspection and simulation can run headlessly.

```sh
git clone https://github.com/zerobytes/ignis.git
cd ignis
cargo build --locked --release

# Native editor with an included scene
./target/release/ignis --root "$PWD" --editor --scene examples/stealth-lab.scene.json
```

For a playable scene, use `--window` instead of `--editor`. **P** starts or pauses,
**WASD** moves, and **R** resets. [Setup, platform notes and troubleshooting →](docs/quickstart.md)

No GPU or MCP client is needed for this first inspection:

```sh
./target/release/ignis --root "$PWD" \
  --scene examples/stealth-lab.scene.json \
  --command '{"command":{"op":"inspect"}}'
```

## Connect an agent

Point any compatible **local stdio MCP client** at your build and project folder:

```json
{
  "mcpServers": {
    "ignis": {
      "command": "/absolute/path/to/ignis/target/release/ignis",
      "args": ["--root", "/absolute/path/to/ignis", "--mcp"]
    }
  }
}
```

Add `--editor` to those arguments to share the same process with the visual
editor. Without it, MCP runs headlessly. The client discovers the
`engine_command` tool and its generated schema through `tools/list`.

Try this prompt in your client:

> Inspect the scene first. Generate a small seeded room, run 120 simulation
> steps, capture the result and explain what changed. Ask before discarding
> existing edits.

An `engine_command` argument looks like this:

```json
{"command":{"op":"capture","path":"artifacts/view.png","width":1280,"height":900}}
```

Captures return actual engine PNG pixels and metadata. The engine is the tool;
the language model and its provider are your choice. [MCP walkthrough →](docs/quickstart.md#mcp-walkthrough)

## Explore the engine

Browse the **[full feature guide](docs/features.md)** for the capability map,
the [English architecture overview](docs/overview.md) for how the systems fit
together, and the [demo catalog (PT)](docs/demo-catalog.pt-BR.md) for existing
scenarios. The [command contract](docs/commands.md) and MCP schema describe the
actual control interface. Detailed technical contracts are mostly in Portuguese.

<table>
  <tr>
    <td><img src="docs/media/sunset.webp" alt="Mountain silhouettes and atmospheric layers at sunset, rendered by Ignis" width="600" height="388"></td>
    <td><img src="docs/media/cottage.webp" alt="A native architecture study: a detailed cottage rendered in Ignis" width="600" height="400"></td>
  </tr>
  <tr><td>Mountain silhouettes and atmospheric layers. Native Ignis capture.</td><td>Architecture study. Native Ignis capture.</td></tr>
</table>

Technical documents include historical milestones and their validation
boundaries, mostly in Portuguese. Generated test artifacts and private
reference inputs are not bundled. [Image provenance](docs/media/README.md).

## Help shape Ignis

The most useful first contribution is **trying the quickstart and reporting
exactly where it succeeds or fails**. Build fixes, clearer examples, docs and
MCP client interoperability reports are welcome.

Read [CONTRIBUTING](CONTRIBUTING.md), open a
[reproducible bug report](https://github.com/zerobytes/ignis/issues/new?template=bug_report.yml),
or share an experiment in [Discussions](https://github.com/zerobytes/ignis/discussions).
If you want to follow the work, star the repository or watch releases.

## License and credit

Created by **Guilherme Oliveira ([zerobytes](https://github.com/zerobytes))**.
This distribution uses the [Ignis Engine Commercial Source License 1.0](LICENSE).
It is **source-available, not OSI-approved open source**.

**Free to start. After a Game exceeds USD 1 million in lifetime revenue,
royalties are 1% of eligible profit, after eligible costs and unrecovered losses.**
The first million is exempt; there is no retroactive charge on exempt profit.
No eligible profit means no royalty. Proprietary games are permitted. Retain
[NOTICE](NOTICE) and the applicable license/attribution notices.

Read the [calculation examples and full policy](docs/licensing.md) and
[private reporting guide](docs/royalty-reporting.md). Third-party components
retain [their own licenses](THIRD_PARTY_NOTICES.md). Earlier Apache-2.0 grants
for previously published copies are not revoked by this repository replacement.
