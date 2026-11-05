# Regional coefficient repair and frame admission — EAF-208

Preflight 2026-11-05. Extend the scalar one-bounce box laboratory, not native
GI-off rendering. Keep 32² samples per face, 4096 cosine-stratified paths, seed
317, stable <=12 object slots, and finite-reference limitations. No GUI or
production defaults change. Output is `artifacts/lighting-response-budget`.

Store first-hit/first-shadow-blocker/coverage masks and endpoint bounds per atlas
row independently of source color. Geometry edits conservatively invalidate old
hits/blockers/coverage, the edited object's own charts, outward-hemisphere new
primary hits and possible new sun occluders. Union all edited objects. Sun
DIRECTION invalidates every row; colors/RGB sun scale only recombine weights.
Use private and committed banks; never derive a replacement job from partially
repaired dependencies. New revisions supersede old jobs, queue-ordered work may
finish, but stale revisions must never become visible. Publish only after the
entire current revision is repaired and recolored.

Schedule at most one asynchronous GPU batch per headless 60 Hz tick. Poll/map
completion without Wait in the tick path. Use prior batch timestamp feedback,
cold-build throughput, an online one-row launch probe and safety headroom to admit row batches for
1.0 ms and 2.0 ms repair targets. These are measured soft GPU targets, not a
hardware preemption guarantee. Charge staging-bank copies, invalidation,
compaction, repair, recombination and RGB publication. Report every overshoot,
CPU tick/submit cost, outstanding batches, idle/poll ticks, accumulated GPU work,
wall/tick latency and equivalent latency at 60 Hz. Allocation, offline oracles,
PNG/readback diagnostics and native rendering are separate costs. Actual GPU
callbacks, not synchronous waits disguised as frames, must drive progress.

Quality gate: repaired weights/dependencies/validity bit-exact to a fresh full
coefficient build, combined RGB <=2e-4 versus the old full4096-ray bake, PNG
maxchannel <=1; no unmarked changed coefficients. Check nonnegative finite
weights and row-energy bound <=1.0001. Recolor after geometry repair to reveal
otherwise invisible coefficient errors. Use deterministic moves/returns,
roof edits, offscreen blockers, sun direction, zero light/black source, camera
views and a superseded in-flight edit. Record exact stable slots and revisions.

Strict visual policy holds the last complete scene+lighting image until the
current revision is ready. This guarantees a coherent finite-reference image
but DELAYS scene changes: report this latency and the held image's error against
the requested new scene. It is an editor/static-scene experiment, not immediate
dynamic gameplay lighting. Also evaluate the old regional direct64 fallback as
a diagnostic: measure its error and frame cost; do not call it quality-certified
unless it independently passes the full-reference image gate. No per-frame
oracle is allowed inside the scheduler or admission policy.

Full/unbudgeted regional control timings, publication and native+GI frame cost
must be distinguished from the incremental repair target. An inexpensive repair
slice alone does not prove a cheap complete frame or a hard real-time system.

## Pilot-driven implementation refinements

The initial 0.5 ms experiment was rejected: whole4096-ray serial rows and a
controller based only on GPU time/row produced tiny batches and excessive
latency. These aborted pilots are not the retained final benchmark. Final
admission separates a measured one-row fixed cost from throughput, has a
bounded2048row maximum, and reports every actual overrun (including a target
below the measured floor). The targets are SOFT; no GPU preemption is assumed.

The repair kernel cooperates across128lanes per row, preserving all4096paths
and their RNG sequence. A scene-independent64checkpoint RNG table costs256B
per atlas row (12/13.5MiB for open/thin), generated once and reused through all
geometry/color/sun edits. This is an explicit extra memory/cold-build tradeoff.
Weights use a tree reduction, so full coefficient builds use the same kernel;
old plain-bake RGB/PNG comparisons retain the original2e-4/1channel gate.
Independent CPU basis integration uses f64 accumulation to validate the new
reduction, plus exact checkpoint/hash checks; no new sampling distribution.

Metal measurement needs BOTH post-completion query resolution and storage
hazards linking the boundary markers to real work/copies. Independent dummy
markers can overlap work and severely understate cross-pass spans. The private bank
now reuses a query set/readback buffer only after completion and unmapping,
resolves after the completion callback, and reads working/published storage at
its boundaries. Compute-only bind groups use private placeholder images, so
repair does not bind the real native/display images unnecessarily. Every event cross-checks all four
full-control spans against a separately fenced single-pass full kernel (no
span may fall below half that calibration). Failed timing pilots were rejected.
Telemetry completion may consume an extra tick; latency includes that delay.

The default runner builds with Cargo. `--cached-backend JSON` explicitly compiles
only the example against supplied prebuilt dependency files, pins their hashes
at link time and pins the resulting executable. This avoids interfering with
parallel Cargo builds; it is not a claim of whole-dirty-tree test acceptance.
The linked backend is identified in manifest/cached-backend.json; no duplicate
binary snapshots are retained. The current engine library was successfully
built in release separately, but subsequent parallel Cargo work removed its
intermediate rlib while rebuilding. The final dataset explicitly links the
pinned prebuilt `libignis-d429e93dab8f18af.rlib` backend, not a claim that the
latest dirty tree is fully accepted. Both initial native fixture PNGs match
the newly built current-core PNG hashes exactly; the final example also passes
a typecheck against current `407a45696254c2ac` API metadata. Backend hashes and
executable identity remain explicit in the manifest.

## Final policy and measurement scope

During every pending tick, rerender the last COMMITTED scene and its baked GI
using a separate stable view bank. Assert its final PNG/linear output equals
the held revision exactly, including supersession during publication. Rendering
and repair share the GPU; timestamp spans can include contention. Native+GI
standalone controls and background/total CPU tick costs are recorded separately.
These headless60Hz ticks are not gameplay FPS or a presentation-latency guarantee.

The scheduler admits at most one batch per tick and uses no Wait. It gives up
at150ticks once pending telemetry has drained (observed150–151ticks, about2.5s). It drains the pending
telemetry without publishing, rejects that revision and keeps the prior image.
The TEST DRIVER then runs a separately timed unbudgeted regional completion to
validate/settle that fixture and advance the test sequence. This offline repair
is neither a runtime fallback nor evidence of successful budgeted publication.
The gallery and JSON label rejected jobs and charge their offline GPU work.

GPU batch spans charge parameter staging-to-uniform copies, geometry staging
copies, bank copies, invalidation/compaction, repair, recombination and RGB
publication. The160B per-batch queue upload into reusable parameter staging is
outside the timestamp span (CPU submission included). Callback telemetry,
allocation, diagnostic readbacks/PNG/oracle work and native raster are separate;
background rendering continues during scheduling. Pooled queries, readback and
parameter staging avoid allocation in the tick path. Safety headroom and the
initial one-row probe do not turn soft admission into a hard GPU deadline.

## Result — policy rejected for dynamic geometry

Final automated run:2scenes ×2budgets ×17events =68cases,480×320 viewport,
32²atlas samples/face,4096paths/texel, Apple M4 Pro. Each event has4rotated
unbudgeted controls/method. Budget traces are single ordered runs on a shared
workstation, not randomized repeated evidence that one target is faster.

| Scene | Budget target | Batches over target / all | Largest slice | Jobs rejected at age limit |
|---|---:|---:|---:|---:|
| Open |1ms|452/759|4.167ms|10/17|
| Open |2ms|70/736|9.393ms|8/17|
| Thin wall |1ms|440/761|4.618ms|10/17|
| Thin wall |2ms|76/760|4.718ms|10/17|

All40geometry/sun-direction events FAILED the joint criterion of completion
before the age limit AND every batch within target. Only2geometry events
completed before the age limit (both open/2ms), and both still had overruns.
The28zero-repair/color/camera/light-scale cases completed. Across the full
suite,8supersession tests exercised geometry/publication.38jobs were rejected rather
than displaying incomplete lighting. Accepted palette-like events took about
34–71ms (2–4ticks); rejected jobs retained the old scene for about2.50–2.52s.
This is not acceptable evidence of immediate dynamic-gameplay lighting.

Regional mathematics remains useful without frame slicing: median geometry
controls cost about26.0–26.4ms open and34.5–34.6ms thin versus full coefficient
builds32.2–32.9/39.8ms (plain RGB rebakes34.3–34.7/40.7–40.9ms).
Budgeted geometry attempts spent median86–106ms BEFORE any offline settling:
fine fragmentation increased total GPU work instead of reducing it here.
Do not claim a whole-frame/native-GI-off saving from these update numbers.

Repair CPU tick p95 medians across cases:0.215–0.275ms, with observed maxima
0.917–2.708ms depending scene/budget. Including background-render CPU work,
per-case p95 medians were about0.382–0.520ms. Standalone native+baked-query
GPU control medians were about0.120–0.125ms at this small viewport; these are
separate observations, not continuous whole-frame GPU distributions.

Quality after settling: all68weight/dependency fields match fresh full
coefficient builds byte-exactly; combined/view RGB stays within the2e-4 gate
versus the unchanged plain bake, PNG maxchannel1. This includes offline settling
for the38rejected jobs; it must not be presented as68successful asynchronous
repairs. Every committed background image stayed byte-exact during pending
work, stale revisions were discarded, and offscreen blockers changed transport
without appearing in the primary image. Independent f64 CPU basis checks max
1.61e-7; all-row CPU recombination versus plain bake max1.63e-5. Five RNG rows
per scene ×64checkpoints also match the independent integer hash sequence.
The finite bake remains an approximation, not physical/artistic ground truth.

Direct64 pending preview FAILED the fixed visual gate: worst PNG channel
error73open/68thin against the full coefficient image. This difference can
include sampling/spatial-estimator differences, not only noise; it is not a
proof against a converged physical reference. It is not a certified substitute
for this consistency target.

Per-bank weights1.50/2.53125MiB, dependency records2.25/2.53125MiB and RNG
checkpoints12/13.5MiB. The full diagnostic's5Lab banks allocate114.35/122.13MiB
in buffers (excludes native renderer, textures, query internals and CPU data).
These are actual laboratory buffer sizes, not a proposed production footprint.

Review `artifacts/lighting-response-budget/review.html`, raw `results.json`,
per-event timings, binary fields, source/backend/executable hashes and independent
`summary.json`. Reproduce with `uv run scripts/run_lighting_response_budget.py`
or the explicitly pinned `--cached-backend` option. Release library build,
release-linked example, scoped Clippy/rustfmt/Python F/diff and all retained
CPU/GPU/PNG audits pass. No GUI, production default, scene format or runtime
lighting toggle was changed. EAF-208 remains In Progress.

Next contract: prioritize the exact atlas support needed by visible receivers,
while retaining offscreen emitters/occluders as contributors. Integrate admission
and publication into one native frame graph and decouple delayed telemetry from
work progression. Treat hard-budget compliance and edit-to-visible latency as
separate gates. The current all-rows-before-publication policy is REJECTED for
realtime geometry updates; it is retained only as a reproducible laboratory.
