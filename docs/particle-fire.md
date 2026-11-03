# Parametric fire — EAF-219

## Contract and preflight

Six cosmetic families share the native particle source: candle, campfire, torch,
blue jet, turbulent inferno and arcane. Intensity, height/width in metres,
turbulence and seed are authored controls, preserved with the source. Intensity
zero extinguishes all layers; style changes alter emission, shape, life and motion
as well as palette. Emission curves, motion and the sprite backend remain common.

A bounded 32×32 emission/opacity material map describes pointed, irregular flame
lobes; complete box-filtered mips use the existing fixed-material contract. Bake
occurs only at recipe authoring/admission. Source2 files carry the compact recipe rather than derived emitters/maps; native
admission rebuilds and validates them. Placed emitter layers share immutable
material resources; there are no images loaded or maps generated during drawing,
no new pass and no change to the 80B particle instance stream. Transparency and
opaque depth stay shared. Seeded births and analytic trajectories retain seeking.

Source2 adds an optional fire recipe and generic constant spawn Offset module.
Readers preserve Source1 unchanged. Recipe-bound emitters must equal their native
derivation; manual emitter editing uses explicit `detach_fire` (or “Editar emissores
manualmente” in the panel), preserving the derived emitters. Regeneration refuses
to overwrite a detached/manual source.
The Scene118 minimum gate distinguishes Source2, with replay128/content14 reserved
over the current117/127/13. Concurrent higher format values must be preserved.

This is an artistic sprite model. Combustion, spreading fire, heat transfer,
gameplay damage, smoke volumes and light emitted onto nearby objects require
separate contracts. The first implementation keeps the existing planetary and
network presentation boundaries of native particles.

## Static performance review

- Work frequency: bake maps/build emitters on explicit authoring; validate before
  mutation; sample only live birth windows during presentation.
- Allocations: immutable source/material Arcs, reused frame/upload scratch. Maps
  are shared among flame layers in memory; compact recipes keep source/network
  payloads small. Explicitly detached/manual sources retain full emitter data.
- Data: fixed32 maps contain1365 samples×48B, with no new shader or particle ABI.
  Palette/motion coefficients and budgets are bounded and deterministic locally.
- Repetition: source edits bake once; an immutable derivation proof validates subsequent
  serialization/edits without rebaking. Native file admission rebuilds that proof;
  camera movement and editor drawing never regenerate maps.
- Limits: six families; intensity0..3, height0.1..8m, width0.03..5m, turbulence0..2;
  existing source/material/particle and IO budgets remain enforced.
- GPU: overdraw and existing OIT targets remain actual costs. Reuse does not imply
  free transparency or a sustainable population/FPS claim.
- Concurrency/IO: no new locks, threads, dependencies, network packets or drawing
  IO. Existing authoring transactions and source files provide persistence.
- Invalidation: parameter edit publishes a fully validated candidate, preserves
  placement IDs, and uses normal reset/undo; fixed-time results reconstruct.
- Validation: zero/intensity/style/seed/recipe corruption, legacy preservation,
  IO/scene/replay/portable/public-content round trips and native image comparisons.
- Human acceptance: opening the gallery does not substitute for artistic approval.

## Usage and scope

`particles.fire {id, settings}` creates a source and its first placement, or updates
a recipe-bound source while preserving every existing placement. Configuration
contains `kind` (`candle`, `campfire`, `torch`, `blue_jet`, `inferno`, `arcane`),
`intensity`, nominal `height`/`width`, `turbulence` and `seed`. The visible envelope
also depends on birth size, life and motion; these are artistic scales.

The editor can apply and run while the demo is playing: its command batch pauses,
authors, clears fixed preview time and resumes. Direct MCP authoring requires a
paused ordinary scene. `detach_fire` preserves all generated emitters and maps
for manual editing and stops recipe regeneration from overwriting them.

Source2 fixes the native derivation algorithm; incompatible changes to that look
require a new source version. Compact fire files contain settings, never the
compiled proof/maps. Decode regenerates them once; subsequent validation compares
against immutable derivation evidence, including protection against accidentally
serializing a manually modified recipe-bound source. Generic sources and Source1
retain explicit emitters and existing material serialization.

The gallery script constructs all nine examples through MCP, saves sources and
scene, checks seeking/persistence and generates local native captures. Six types
occupy the front/middle rows; the back row uses identical campfire settings/seed
with intensities0.3/1.0/2.2. The brazier/candle/labels/rings are ordinary meshes,
independent of the cosmetic effect and its intensity control.

## Validation evidence

Formatting (`cargo fmt --all -- --check`) and release all-target Clippy passed.
All21 directed release tests passed (`particle_fire`, `particles`,
`material_transparency`, including ignored native GPU tests). Seven fire tests
cover six distinct silhouettes, complete mips/transparent borders, zero and
intensity scaling, seed/seek/cache stability, Source1 preservation, Source2
offsets, atomic recipe rejection/undo, explicit detachment, source IO, replay,
portable save/public content and native visibility/warm upload reuse.

The gallery passed six MCP groups: nine effects/six families, compact recipes,
fixed-time/seek PNG equality, portable restore PNG equality, scene restore PNG
equality and native editor controls. Local evidence lives in
`artifacts/fire/verification.json` and `review.html`; regenerate it with
`python3 scripts/demo_particle_fire.py`. Use `--open-only` for the saved live demo.

Apple M4 Pro/Metal at1600×1100: tick240 shows1473 live sprites in one particle
draw (117840 active/163840 allocated instance bytes); eighteen placed flame
material references share590544B of fixed programs/samples. Shared OIT images
use49280000 logical bytes and also compose decorative mesh rings. Nine source
files total2170B, while the gallery's ordinary meshes/labels add their own
geometry and source cost. These counts do not establish FPS or universal capacity.

Automated validation and opening the native scene remain separate from the
user's artistic acceptance of the new fire families.
