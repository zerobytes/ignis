# Game UI: authored documents and native workbench

EAF-228, first vertical slice, 2026-11-05. This is reusable engine tooling;
Land of Survivors remains a consumer with its existing front end unchanged.
The public API is `ignis::game_ui`, `Command::Ui` and the existing
`engine_command` MCP tool. The backend remains egui/egui-wgpu/egui-winit0.33.3.

## Audit and delivery matrix

The audit examined `editor_ui.rs`, `editor.rs`, `editor_window.rs`, `mcp.rs`,
`service.rs`, and the game's `front_ui.rs`/`front_window.rs` at the public
checkout descended from8e9572a. Existing editor workspaces (EAF-104) contain
fixed editor panel kinds, not authored game screens; reusing that document
would conflate two different lifecycles. The new file describes game UI only.
Persistence uses the existing confined-path/atomic-write implementation and
all edits use the existing typed dispatcher. No change of renderer is needed.

| Requirement | Already present before this slice | Integrated in this slice | Still needs integration/implementation |
| --- | --- | --- | --- |
| 1. Components and lifecycle | egui IDs, nesting, visibility, clipping, hit testing; game-specific Rust composition | Versioned tree with unique IDs; reusable `Node` subtree/`instantiate(prefix)`; stable runtime identity across text changes/reparenting; separate authoring/data/events | Named component assets, overrides/slots, explicit instance lifecycle tooling. Consumers must allocate a new ID for a semantically different element; recycling an ID intentionally reuses egui memory. |
| 2. Layout | egui row/column/wrap, scroll, logical points and winit DPI | Column, wrapping row, wrapped labels, padding, preferred maximum width clamped to available width, minimum widget height, vertical scrolling | Min/max constraints on both axes, weighted/aspect sizing, authored alignment/anchors, safe-area adapter and breakpoints. No fixed screen resolution required by the document. |
| 3. Theme | Per-screen egui style; bundled proportional font and game logo texture | Persisted colors for normal/hover/pressed/disabled/focus, accent, font size, spacing and rounding; visible focus outline; progress bar | Font asset registration/fallback policies, image/icon resources, nine-slice panels and radial meters. Default egui fonts are used; font size is not a custom font pipeline. |
| 4. Input | winit-to-egui mouse/keyboard adapter, existing editor focus routing; library modal primitive | Pointer/text/Tab/Enter in authored widgets; `Capture` result; demo consumer modal/focus isolation | Authored modal nodes, consumer drag/drop validation, gamepad navigation, touch/device acceptance. `Capture` must be read after all layers and honored before gameplay; this does not silently migrate existing consumers. |
| 5. Visual motion | egui widget feedback and repaint requests; editor style uses animation_time0 | Immediate visual states preserved | Explicit interruptible opacity/position/scale/color transition tracks and later sequence/timeline authoring. No animation system claimed. |
| 6. Dynamic content | Rust consumers own game data; egui scrolling/row virtualization available in library | Bounded typed text/number bindings; action/change events carrying IDs and keys; missing text keys shown explicitly | Authored repeaters and stable item identity, virtualized long lists, tooltips, inventory/skill-tree components. Current tree is bounded to256 authored nodes; it is not a large-list solution. |
| 7. Native3D | `Graphics::build_custom` and `update_custom_scene` compose real Ignis viewport texture | Authored viewport node exposes logical rect; same renderer paints a neutral stage | Camera/framing adapter as a public consumer contract; world-anchored names/bars with projection, depth/occlusion and lifecycle. One texture bridge does not implement world-space UI. |
| 8. Authoring | Typed MCP schema, editor workspace persistence and atomic file writer | UI1 JSON file; shared inspect/replace/edit/save/load/undo/redo; native hierarchy, selection, property/theme drafts, add/remove/reparent and live preview | Canvas picking/drag/resize, component palette/templates, asset references, transactional preview gestures and richer editor integration. The workbench is an initial document tool, not a complete visual UI editor. |
| 9. Localization/accessibility | Game has hand-coded EN/PT strings; egui optional accessibility support | Key-based translation tables with EN fallback, PT-BR example, wrapping and visible focus | Font coverage/IME/device acceptance, text direction/plurals, semantic roles and enabled native accessibility bridge. Library availability is not Ignis accessibility acceptance. |
| 10. Robustness/performance | Existing native GPU and MCP transports, confined atomic writes | Finite validated dimensions, bounded hierarchy/text/data/history, guarded publication, recoverable errors; composition and UI-only paint CPU measurements | Representative large HUD/list/skill-tree workloads, GPU timings, backend/device matrix and profiling. No FPS claim. |

The [egui integration documentation](https://docs.rs/egui/0.33.3/egui/)
explains logical-point DPI, the input/output integration boundary and optional
accessibility integration. Local dependency features do not enable an
AccessKit platform bridge in this workbench. Touch/controller support has not
been exercised. egui's own persistence of widget memory is deliberately not
serialized as the game's authored document or gameplay save.

## Ownership and runtime contract

- `Document` owns UI version1, document ID, `Theme`, translation tables and a
  `Node` tree. Sibling array order is visual/focus traversal order. Only column
  and row nodes accept children. Unknown fields/versions are rejected.
- `State` publishes validated documents, revision and a32-entry undo history.
  Its current snapshot and history are shared by `Arc`; one candidate tree is
  cloned per edit transaction. It never resets/pauses/ticks the3D world and is
  not added to Scene, replay, portable-save, network or physics formats.
- `Runtime::new(instance_key)` owns transient consumer text/number data and
  locale. It renders only published `State`. Its widget namespace is runtime
  instance + document ID + node ID, independent of hierarchy position. Use a
  unique instance key for simultaneous views. Host egui memory owns focus,
  scroll/caret/interaction state; dropping runtime data does not erase a shared
  egui context. Change instance/document/node identity for a new logical UI.
- `Runtime::show` returns `Frame`: hit/focus responses, consumer events and one
  optional viewport rect. Buttons emit an action key; text edits update
  transient data and emit `Changed`. Consumers decide how to validate/use those
  events. UI files never contain shell, service-command or privileged callbacks.
  Binding type mismatches use an empty input/zero progress; missing display
  bindings/translations show `[key]`. Progress values are clamped visually0..1.
- `Capture::read(ctx, modal_open)` is read after every UI layer. Modal capture
  includes both keyboard and pointer. Consumers must route captured input away
  from gameplay and clear held gameplay state on capture/focus transitions.
  This slice tests the contract in its neutral consumer; it does not modify
  the Land of Survivors input loop. The demo modal is consumer Rust using egui,
  not an authored modal component.
- Hosts select the document background when composing a panel; they may also
  use the same runtime over a transparent HUD. The3D texture and camera remain
  owned by Ignis/the consumer, never by the UI file. Hidden subtrees emit no
  widgets/events; disabled subtrees use egui's disabled interaction policy.

## Authoring and storage contract

Use `.ignis-ui` for JSON documents. There is no new MCP transport or parallel
command dispatcher. `Command::Ui` is exposed in the normal `Request` schema:

```json
{"command":{"op":"ui","expected_ui_revision":0,"operation":{"op":"load","path":"examples/ui-workbench.ignis-ui"}}}
```

```json
{"command":{"op":"ui","expected_ui_revision":1,"operation":{"op":"edit","edits":[{"op":"move","id":"name","parent":"root","index":0}]}}}
```

```json
{"command":{"op":"ui","operation":{"op":"inspect"}}}
```

Operations: `inspect`, `replace {document}`, `edit {edits}`, `undo`, `redo`,
`save {path}`, `load {path}`. Edits: `theme {theme}`, `replace_node {id,node}`,
`insert {parent,index,node}`, `remove {id}`, `move {id,parent,index}`. Move index
is interpreted **after removal**. Replacing a node preserves its ID; use
remove/insert for deliberate new identity. Root removal/movement and cyclic
moves fail. Theme replacement and arbitrary hierarchy/property edits can be
combined into one transaction. The full candidate is validated before publish;
a failed transaction/load/revision guard leaves document/history unchanged.
Undo/redo/load increment UI revision monotonically; no-op replacement does not.

`expected_ui_revision` guards UI state independently. Optional outer
`expected_revision` still guards the engine's scene revision. Use inspect to
obtain the appropriate guard. Saves and inspection do not increment revisions;
UI edits do not participate in scene undo/redo. Mutation receipts are compact;
inspect returns the full document/history counts.

The native inspector keeps a local property/theme draft until Apply. The
captured UI revision rejects stale application after an MCP edit. All published
operations, including save/load/undo/redo, go through `Service::execute`, as
MCP/CLI do. Runtime preview data and language toggling are example-consumer
logic, not authored edits or new engine/gameplay commands.

Limits:256 KiB compact serialized document/file,256 nodes, depth16 edges from
root,64 edits per transaction,32 undo snapshots,1 viewport,8 locales with512
entries each,4096 UTF-8 bytes per text,128 runtime bindings,64-byte ASCII keys.
Font size8..48 points, spacing/padding0..64, rounding0..64, optional width24..4096
and minimum widget height0..1024 are finite. The existing MCP2 MiB envelope and
project-relative path/symlink policy still apply. Save uses a same-directory
temporary file and atomic replacement; failed load never replaces the document.
Native close discards unsaved UI session/drafts; Save is explicit.
UI state belongs to the service's authoring session. It is not implicitly
loaded/saved with a project scene; consumers explicitly select their UI file.

## Run and verify

```sh
cargo build --locked --release --example ui_workbench
./target/release/examples/ui_workbench
./target/release/examples/ui_workbench --mcp
./target/release/examples/ui_workbench --verify
cargo test --locked --release --test game_ui
python3 scripts/verify_game_ui.py --native-mcp
```

`--mcp` opens the same native workbench and accepts the normal newline-delimited
MCP handshake/tools on stdio. A bounded mailbox marshals commands onto the
native event-loop thread. A client can inspect/edit/save/load the **same**
authoring state displayed manually. Closing stdin does not close the window;
`engine_command` with `exit` or native close ends it. No secret socket/backend
or background game connection is created. The existing CLI can also use
`ignis --command '{"command":{"op":"ui",...}}'` or `ignis --mcp`; a one-shot
CLI invocation has a fresh authoring session, so load before editing via MCP.

The demo starts with `examples/ui-workbench.ignis-ui`, a neutral fixed-geometry
stage and independent consumer data. It never opens game scenes or modifies
LOS code/assets. Its default save is
`artifacts/ui-workbench/current.ignis-ui`. Native resizing/DPI comes from winit;
the verification harness uses the **same** egui/Graphics/runtime with pointer,
text and keyboard events at1280×860/1× and820×960/1.5×. Native GPU offscreen
captures are `wide.png`, `narrow.png`, `modal.png`, with `verification.json`.
Harness interaction is not a desktop mouse/keyboard session or human acceptance.
The optional Python verifier also opens a temporary native window, exercises
its actual MCP mailbox (inspect/edit/undo/exit), then closes only that window.
It records native transport acceptance separately from desktop input.

CPU composition timing includes the workbench's hierarchy/inspector and egui
layout. UI-only paint samples explicitly disable the3D viewport draw and cover
tessellation, texture/buffer preparation and queue submit. Neither number is a
GPU timestamp or a present/gameplay FPS measure. The interactive status also
shows the combined normal paint +3D CPU elapsed time.

Keep the one current example binary at
`target/release/examples/ui_workbench`; do not create copies/bundles per run.
Artifacts and binary outputs are ignored. Required compilation state for this
active target stays in the shared target directory; parallel outputs are not
cleaned by this task.

## Prioritized next units

1. Complete basic layout contracts: alignment, symmetric min/max constraints,
   anchors/safe-area input, aspect/weighted sizing; focused multi-size/DPI checks.
   Add canvas selection/drag preview using the same transactional edits.
2. Introduce reusable named component instances/overrides and authored modal/
   input routing. Virtualized repeaters require stable consumer item keys;
   inventory/skills remain consumer rules. Add validated drag/drop and gamepad
   navigation only with a real input/backend acceptance path.
3. Add interruptible visual transition tracks (duration, curve, target, current
   value, cancellation/replacement), separate from gameplay time. Only then
   build a small sequence/timeline authoring surface over those operations.
4. Add font/image/icon resources with asset validation/fallback, nine-slice and
   meters as justified by consumers. Extend localization and enable/test native
   accessibility integration explicitly.
5. Add world anchors/camera framing through separate3D contracts, measure real
   HUD/list/tree workloads, and coordinate a behavior-preserving LOS migration
   with the game chat. No front-end migration is part of this first slice.

Human acceptance of the workbench and broader requirements remain open under
EAF-228. Build, model/MCP tests, GPU harness, actual native-window input and
artistic acceptance must be reported separately.

## Static cost review for this increment

- Authoring copies one bounded candidate tree per transaction; current/history
  documents use `Arc`. No document serialization, file access or validation is
  repeated during normal preview frames. The byte quota is a serialized quota,
  not a certified total RAM budget.
- Rendering walks the tree once and performs keyed translation/data lookups.
  Per-frame text/response/event allocations are bounded by the authored limits;
  the workbench's inspector does small bounded tree searches. Virtualized lists
  require a different item/repeater contract rather than raising these limits.
- Transient edits never rebuild the scene, change physics or upload new world
  geometry. The existing registered viewport texture is reused/replaced on
  size changes; font atlases, clipping and UI buffers remain egui/wgpu concerns.
- Native authoring runs on the event-loop thread. MCP uses one bounded mailbox
  and the existing protocol parser/schema; no extra world mutex or renderer
  instance is created for protocol-only requests. Idle preview rendering follows
  egui repaint requests, not a forced continuous3D clock. The optional MCP
  mailbox polls at33ms while the window is open.
- Cache invalidation follows explicit document revisions/IDs and egui style/
  text changes. Save/load are explicit operations. Native offscreen verification
  uses the same renderer and labels CPU submission separately from3D work;
  no GPU timing or universal frame-rate result is inferred.

## Current validation checkpoint

The three `game_ui` integration tests passed: failed/stale transaction rollback,
file/undo/redo round trips preserving the live scene/replay/clock, bounded data
and documents, component identity, and MCP schema/dispatcher integration.
The repository-wide Clippy command was attempted and currently reports
`items_after_test_module` in `src/render/object_impostor.rs` and
`field_reassign_with_default` in `src/materials/preview.rs`; these are outside
this increment. Whole-repository formatting also reports parallel botanical
files. UI-owned source formatting is checked separately. GPU/native results
are recorded in the generated verification report after the executable check.

The final same-runtime GPU harness passed on Apple M4 Pro/Metal, including
focus after moving the text input into a different container, pointer/text/Tab/
Enter, modal isolation, disabled controls, manual property/undo/save/load and
preferred-width row wrapping at820×960/1.5×. Captures at1280×860/1× and
820×960/1.5× (1230×1440 physical pixels) were visually inspected. The native
window MCP probe passed initialize, inspect, guarded edit, undo and exit; only
its temporary window was closed. No desktop mouse/keyboard interaction or
human/artistic acceptance is inferred.

Twelve warmed CPU samples in the final report measured about0.067ms for
workbench/egui composition and0.128ms for UI-only paint preparation/submission
with3D drawing disabled. These numbers describe this small document and machine
only. The current report is `artifacts/ui-workbench/verification.json`; normal
combined3D/UI painting remains separately displayed by the demo.

A later strict targeted Clippy attempt was also blocked by
`suspicious_assignment_formatting` in `src/service/botany.rs`. A separate
targeted check allows that known external style lint; the strict command and
its external failure remain recorded rather than being presented as a global
pass. No unrelated botanical/render/material code was changed for this unit.
