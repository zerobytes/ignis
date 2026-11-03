# Botanical valley performance measurements

EAF-210: the user rejected the playable scene at approximately 2 FPS. These
measurements characterize that preserved landscape; they do not optimize it.

## Workload and method

The original 1 km scene is unchanged: 411,732 plants/tufts, 694,498 instances,
208 assets and 1,199,294,230 triangles when summing all instances before
visibility rejection. Input SHA256:
`da0805a03945de538bbcdf65bf1769a8dc66fcb21826e646e91454918d9c22c9`
(772,560,586 bytes). The hash was checked before and after the main matrix.

Host: Apple M4 Pro, Metal. The diagnostic links the existing release library,
build `ffdbdb0180bc9443` (macOS arm64, Rust1.97.1), without rebuilding the shared
renderer or replacing the native app. The checkout contained parallel work.
This library build is distinct from the earlier composition captures' build
`9f3acc12d0dedf60`; do not claim a byte-identical executable replay of the user's
session. Library and diagnostic-executable hashes are in `link-provenance.json`.

Three frozen cameras: the first-person player spawn (65° FOV, yaw0/pitch−3,
eye1.65m), the authored valley view and the aerial panorama. All vegetation,
materials, geometry and placements remain present. The first-person body is
hidden through the native player-view mode. A no-shadow control disables only
the shadow setting; it is a diagnostic, not a proposed replacement appearance.

The main matrix has two physical target sizes, 2200×1520 and1100×760, with
shadows on/off: **12 cases, 16 measured frames each (192 total)**. Four alternating
blocks of four observations per variant follow three warm-up draws per block.
The render target and renderer are reused. CPU simulation and frame preparation
have 24 separate observations per camera (72 each).

`Renderer::draw` wall time includes visibility, batching, command recording,
encoding and queue submission, with possible driver backpressure. It is followed
by an explicit completion wait. The timed loop excludes PNG, readback and
presentation. **Residual completion wait is not an isolated GPU timestamp.**
These serial draw+wait measurements must not be presented as native-window FPS.

## Main matrix

Medians in milliseconds:

| Camera | Pixels | Shadows | CPU draw/submit | Residual wait | Draw + wait |
| --- | --- | --- | ---: | ---: | ---: |
| Player spawn |2200×1520|On|241.84|333.10|573.65|
| Player spawn |2200×1520|Off|116.44|213.07|330.02|
| Player spawn |1100×760|On|238.70|313.84|554.35|
| Player spawn |1100×760|Off|118.34|196.91|316.94|
| Valley |2200×1520|On|248.65|303.61|552.22|
| Valley |2200×1520|Off|125.36|185.86|311.41|
| Valley |1100×760|On|255.29|285.99|541.24|
| Valley |1100×760|Off|130.84|169.00|300.09|
| Panorama |2200×1520|On|329.10|599.58|929.45|
| Panorama |2200×1520|Off|222.73|483.63|708.66|
| Panorama |1100×760|On|320.81|589.95|910.44|
| Panorama |1100×760|Off|220.25|466.99|690.76|

At player spawn, the combined shadow-off effect is **243.63 ms /42.5%**.
The valley improves43.6%, the panorama23.8%. These differences include changes
to CPU masks/batches, shadow drawing and shader sampling, not only GPU depth-pass
time. Removing shadows leaves a very expensive render workload.

Reducing the pixel count to one quarter improves total draw+wait only3.4% at
spawn and approximately2% in the other views. This sensitivity test, together
with the hardware trace below, points to geometry/submission as the principal
problem rather than pixel fill alone.

Simulation after settling, with one player and neutral input, takes approximately
**0.043–0.047 ms per fixed tick**. CPU frame
preparation takes **13.78–14.43 ms**; this is separate from the much larger
renderer CPU costs above. A simulated tick is not a presented frame.

The spawn view reports810,374,264 selected detailed triangles and374,573 draw
units with shadows, versus504,908,260 and173,855 without shadows. The draw-unit
counter is the renderer's batch/page accounting, not a hardware API trace count.
LOD and GPU occlusion remain inactive in this workload. Geometry uploads are
cached; generation/upload is not being redone for every measured frame.

## Actual GPU hardware intervals

A separate, warmed **3-second Time Profiler + GPU Instruments** recording
captures five complete shadow/main pass pairs. Recording begins only after the
runner warms its shaders/uploads and receives the Instruments start notification.
The XML parser filters to the runner process, checks command-buffer completion,
and unions overlapping channel/nesting intervals instead of summing them.

| GPU activity | Complete samples | Median active time | Median first-start→last-end span |
| --- | ---: | ---: | ---: |
| Directional shadows |5|110.64 ms|112.44 ms|
| Main scene |5|214.75 ms|218.07 ms|
| Combined, overlaps deduplicated |5|324.72 ms|329.84 ms|

Vertex-channel activity is109.77ms for shadows and213.70ms for the main pass.
This describes the Metal hardware channel, not a certified percentage of shader
instructions. Fragment/channel work overlaps it. Do not add these GPU durations
to CPU draw/wait wall times as independent stages; overlap and scheduling exist.
Hardware timing covers this camera/sample, not every location in the kilometre.

The same trace's statistical CPU stacks inside `Renderer::draw` attribute:

- 33.9% to wgpu command-encoder finishing/render-pass encoding;
- 17.6% to visibility update;
- 15.4% to recording instanced draw commands;
- 33.1% to other renderer preparation/submission work.

These categories are disjoint sampled stacks (1,256ms of sampled CPU weight),
not exact wall-clock substage timers. They support investigating command/batch
volume and visibility preparation before adding more expensive visual effects.

## Environment and limits

Other chats were compiling:37/97 two-second monitoring observations detected
compiler/trace CPU above5%, peaking378.4% across processes. The primary matrix is
not an entirely idle-machine benchmark. A conservative post-hoc subset requires
both surrounding monitoring observations to show no such process above5%:
four spawn/full observations give568.93ms, versus573.65ms across all16; twelve
valley/full observations give550.53ms, versus552.22ms across all16. These subsets
corroborate the scale of the result; coarse monitoring does not prove perfect
isolation. An additional strictly idle confirmation could not start within its
120-second wait because parallel compiler activity continued.

Initial full Metal System Trace recording was too intrusive and failed while
expanding temporary storage; its partial trace and temporary ktrace were removed.
An 8-second attempt timed out during finalization and was discarded. Only the
successful shorter recording contributes GPU results. Its current trace is
about58MB; raw exports and summaries stay under `artifacts/` and are not published.
No unrelated caches, builds, scenes or user windows were removed.

This investigation makes no renderer changes and does not claim a performance
improvement. The native gameplay result remains rejected. The next work is
effective draw batching/culling, a scalable occlusion path, coverage-preserving
vegetation representations and appropriate shadow work. Each improvement needs
the same workload and image/presentation validation.

## Reproduction and evidence

Portable build when the checkout is ready:

```sh
cargo run --release --example botanical_valley_cost -- \
  artifacts/botanical-valley/performance/cost.json
```

In a shared dirty checkout, `scripts/profile_botanical_valley.py --stage build
--library PATH_TO_COMPLETED_RELEASE_RLIB` links the existing library and records
its identity. Subsequent stages are `cost`, `trace`, `export`, and optional `confirm`.
The current runner is one diagnostic executable, not another retained Ignis app.

```sh
python3 scripts/summarize_botanical_valley_cost.py
python3 scripts/profile_botanical_valley.py --stage export
python3 scripts/analyze_botanical_valley_trace.py
```

The latter requires the successful Instruments exports: tables
`metal-command-buffer-completed`, `metal-gpu-intervals` and `time-profile`.
Current evidence is in `artifacts/botanical-valley/performance`:
`cost.json`, `summary.json`, `measurements.md`, `scene-integrity.json`,
`cost-concurrency.json`, `link-provenance.json`, `trace-full.json`, `metal.trace`,
the exported XML tables and `gpu-cpu-trace-summary.json`.
