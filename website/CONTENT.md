# Engine content provenance

Snapshot: 24 October 2026, through EAF-46 (periodic effects and dispelling). The website is a local presentation, not a connection to the engine process. Contract examples are illustrative and execute no commands.

## Implemented inventory

`dist/engine-data.js` holds 20 bilingual editorial domains. These summarize implemented slices, not completed milestones.

- Scenes, procedural room, transactional editing and shared controls: `src/engine.rs`, `docs/commands.md`, `docs/architecture.md` and ADR 0001. The 56-operation inventory is extracted from the current `Command` enum; schema derivation is shared with the dispatcher.
- Rendering, selection, triggers, inspection, scenario checks and replay: `docs/commands.md`, `docs/validation.md`, current scenario/replay contracts. Replay scope remains same-build; developer inspection is separate from NPC knowledge.
- Perception, hearing, physical footsteps, relations, objectives, territory, routes, spatial mobility, patrols, partial routes, support and communication: current dedicated contracts and consolidation in `docs/npc-roadmap.md`. Authored networks/passages are not automatic navmesh or general crowd navigation.
- Abilities, vitality, movement policies, projectiles, free/automatic aiming, periodic effects and dispelling: `docs/abilities.md`, `docs/vitality.md`, `docs/cast-movement.md`, `docs/npc-ability-policy.md`, `docs/projectiles.md`, `docs/free-aim.md`, `docs/npc-aim.md`, `docs/periodic-effects.md`, `docs/dispel.md`.

The old introductory README limits predate several implemented increments. Latest dedicated contracts and source take precedence over those historical summaries. Automated engine validation recorded in `docs/validation.md` is source evidence; website work does not rerun or claim new engine regression results.

## Roadmap

All 16 domains preserve the project roadmap in Kordo, Ignis project `prj_8b6f130352a14ff5af`, consulted on 24 October 2026. M01–M16 are identifiers, not approved execution order, dates or completion percentages. Each entry separates existing foundations from remaining scope. NPC completion remains the current priority; the ten continuation fronts are consolidated under M09, with dependencies in their respective domains.

M01 core/extensions; M02 scenes/assets/editor; M03 rendering/input/audio/UI; M04 physics/controllers; M05 animation/avatars; M06 procedural framework/topology; M07 particles/water/environment; M08 abilities/combat/resources; M09 NPCs/navigation; M10 spawn/populations; M11 quests; M12 sessions/multiplayer/co-op; M13 open world/levels; M14 encounters/dungeons/raids/PvP/PvE; M15 observation/tests/replay; M16 reference games/export/open-source delivery.

MCP-first applies to every controllable capability, current and future. It does not mean an AI call per frame, dependence on a particular model/vendor, or that future systems already exist. macOS is the first validated platform. Public release, other-platform validation and permissive-license selection remain separate future deliveries.

## Visual provenance

The opening WebGL scene is a brand composition implemented with Three.js. The laboratory image is an actual existing Ignis capture, identified as an early perception laboratory. Neither the procedural landscape nor the SVG contract diagram claims to be rendered by the Rust engine.
