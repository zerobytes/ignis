# Native lighting candidate gallery — EAF-208 / Lumens

Experiment completed, 2026-11-04. User requests one scene with current Ignis lighting,
each candidate, and CPU/GPU costs. Existing candidates are CPU integral
estimators, not production GI implementations. This unit ports their core
estimation policies to an explicit, headless GPU visibility experiment.

## Comparison contract

Create a small editable native box scene. Keep camera, geometry, colors,
directional light, shadow settings and display response fixed. The baseline
is the actual current Ignis renderer, unmodified shading. Candidates replace
only its uniform ambient contribution with finite-radius cosine-weighted
ambient visibility (AO). This is not multi-bounce indirect lighting, emissive
transport, ReSTIR or a finished scene-wide lighting system.

For these matte, nonprocedural opaque boxes, the native ambient term is exactly
`albedo*ambient`. The experimental composite is
`native_linear + albedo*ambient*(estimated_visibility-1)`.
An all-visible passthrough must reproduce native RGB8 pixels. GPU ray/box primary
hits use the same inverse camera projection and exact scene bounds. The shader
keeps signed estimates; the existing SDR output range still limits displayed
colors. Raw visibility error and final displayed differences are distinct.

Compare native only, plain Monte Carlo, cache-only, exact-cache correction,
per-query learned weight, per-query validated weight, shared TTL, supplied
revision and paired sentinels. Include a high-sample visibility reference.
All estimators aim at the same visibility integral; visual differences primarily
show sampling error, while native-only omits local ambient visibility entirely.

The screen-space receiver/cache prototype is invalidated by rebuilding on camera
or geometry changes. It is deliberately not claimed to be the eventual sparse
world cache. Static warm-frame results must be accompanied by setup cost and
logical GPU resource bytes; per-pixel cache storage is a retained prototype cost.

## Measurement and static cost review

- Add an opt-in render-pass GPU timer. Default renderer construction retains
  no query resources/features/readbacks. The headless profiler requests only
  supported `TIMESTAMP_QUERY`; Metal Apple GPUs require pass-boundary sampling,
  not unsupported encoder timestamps. [wgpu timestamp example](https://wgpu.rs/doc/wgpu_examples/timestamp_queries/index.html).
- GPU raster timings cover named render passes. GPU compute timestamps cover
  experimental setup and estimation/composition passes. Report sums with that
  exact scope, not present cadence, full device time or gameplay FPS.
- CPU measures frame preparation, encoding/upload/submission separately from
  blocking timestamp collection and image readback/PNG. Keep setup/shader
  construction separate. No texture/buffer allocation inside sampled frames.
- Reuse buffers, targets and query sets; one source cache per candidate run,
  bounded angular resolution and full target-call accounting including pilots
  and sentinel checks. Do not conceal initial cache ray queries in warm samples.
- Use the same native renderer and device, paired forward/reverse candidate
  ordering rotated every frame, warmup and repeated complete reuse cycles. Retain raw samples,
  adapter/features/clock period/resolution/source hashes. Shared-machine CPU/GPU
  contention is not excluded. Missing timestamp support is unavailable, not zero.
- No native GUI interaction or birch asset mutation. One current example binary
  and stable artifact directory; replace outputs and remove task staging after
  success. Preserve parallel changes and the running user's applications.
- Verify ordinary/profiled native pixel identity, passthrough identity, GPU
  cache normalization, finite outputs, exact sample accounting and identical
  static TTL/revision images. Run relevant renderer validation for instrumentation.

## Reproduction

```sh
cargo build --locked --release --example lighting_gallery
./target/release/examples/lighting_gallery
uv run scripts/lighting_gallery_report.py
```

Current outputs replace `artifacts/lighting-gallery/`: editable `scene.json`,
native/candidate/reference PNGs, signed float AO maps, raw timing `results.json`,
`summary.json` with source hashes and independent CPU checks, and `review.html`.
The current executable stays in Cargo's ordinary example output, without a
second copied app bundle. Run the benchmark when other compiler/GPU work is idle
where possible; this machine is shared, not an isolated benchmark station.

The GPU scalar port uses a 16-cell angular cache, 16 target queries per shaded
receiver/frame, 4 training queries, optional 4 validation queries and TTL32.
Independent pilots never inspect evaluation samples. The validation gate uses
centered sample variances and the remaining-budget penalty from the CPU study.
An exactly constant cache bypasses pilots. Sentinels use two fixed directions
and check every fourth frame; their queries subtract from the frame budget.
Revision is externally supplied by the diagnostic uniform, not a new engine
dependency tracker. Static revision and TTL must produce identical raw maps.
The cache-only mode has no per-frame target query but retains setup cost and
bias. This is parameter adaptation of the prior estimators, not byte-equivalent
PRNG streams or a speed comparison with the CPU numerical laboratory.

All ten methods, including the 512-query reference, are interleaved within each
measured frame. Rotate order every frame and reverse it in the second 64-frame
round. This avoids interpreting the initial sequential-run clock/order drift as
a method benefit. Static receiver geometry and angular cache are common data;
TTL/revision/sentinel coefficients coincide under this unchanging target, so
their shared coefficient slot is valid for this static-only comparison. Refresh
queries and sentinel checks are still run and charged separately per method.
This arrangement must not be reused for divergent dynamic policy states.

Metal measurement audit: the first interleaved probe returned repeated compute
intervals after expensive passes, so those timings were rejected. Retrieval now
fences pass completion before submitting query resolution/readback separately;
all that blocking/collection is outside the measured CPU interval. The compute
reader rejects nonmonotonic or repeated absolute timestamp intervals across
serial dispatches. This is diagnostic measurement overhead, not an engine frame
scheduling policy. Validate the heavy/light workload response before trusting
candidate timings.

Validation limitation: the existing `directional_shadows` target passed five of
six tests, including the native GPU receiver-plane/cast-shadow regression. Its
`receiver_plane_bias_is_explicit_versioned_and_round_trips` test fails at the
old assertion that an upgraded scene is exactly version111; the current scene
version is120. That test and the version/migration code were not changed by this
unit. Preserve current format versions; do not downgrade the engine or silently
claim the whole target passed. Native ordinary/profiled/passthrough identity is
checked separately by this executable on its actual scene.

## Measured result

Apple M4 Pro / Metal, 640×416, 11 matte boxes, shadow map2048, 128 samples per
method. Medians; no presentation or blocking/readback/PNG in CPU totals.

| Method | CPU total ms | GPU render + compute ms | GPU added ms | AO MSE vs512 |
|---|---:|---:|---:|---:|
| Atual — sem candidato | 0.119 | 0.932 | 0.000 | 0.059182 |
| Monte Carlo · 16 | 0.136 | 1.437 | 0.503 | 0.005268 |
| Cache isolado | 0.141 | 1.088 | 0.158 | 0.003548 |
| Correção · peso 1 | 0.147 | 1.450 | 0.521 | 0.004097 |
| Peso aprendido | 0.149 | 1.443 | 0.520 | 0.005540 |
| Peso validado | 0.149 | 1.539 | 0.607 | 0.007748 |
| Compartilhado · TTL32 | 0.147 | 1.448 | 0.519 | 0.004238 |
| Revisão explícita | 0.143 | 1.454 | 0.521 | 0.004238 |
| Sondas pareadas | 0.179 | 1.452 | 0.521 | 0.004238 |
| Referência · 512 | 0.134 | 14.938 | 14.000 | 0.000000 |

The final fenced timings have the expected workload response: reference512 costs
27.82× MC16 in compute; zero-query cache costs 0.31× MC16. Every serial
compute interval is positive and begins after its predecessor ends. Initial
unfenced measurements were replaced, not retained as valid evidence.

Native ordinary/profiled and all-visible composite PNGs are byte-exact. An
independent CPU ray/box/cosine implementation matched 88 randomly selected
foreground receivers for both MC16 and the complete 16-cell cache; background
receivers were also checked. TTL/revision raw maps are bit-exact in this static
scene; all saved signed estimates are finite. The approximate reference uses
an independent random stream; it is not exact transport ground truth.

This prototype retains 43.67MiB logical extra GPU resources (including shared
receiver/cache allocation and experimental textures), plus readback/driver
costs. Resource/shader setup was 1.158ms CPU in this warm-driver run; renderer
creation is reported separately. Per-method receiver/cache GPU setup is exposed
in summary.json/review.html, outside warm-frame totals. Camera/geometry changes
require rebuilding the screen cache. No engine scene version, gameplay lighting,
material, GUI, birch asset or native default rendering behavior was changed.

Visual result: cache-only hides noise but exhibits structured angular-cell
artifacts. Correction removes that fixed pattern but has sample noise. At this
small budget, learned/validated pilots do not beat plain MC consistently; the
validated pilot is worse in this frame. Reused weights save training work but
are not a demonstrated GPU speedup over MC here. Do not infer GI, dynamic
invalidation correctness, world-scale cache efficiency, denoising or superiority
to Lumen. Shared-machine CPU load and driver caches remain measurement limits.

Final checks: release example build, GPU execution with fenced/monotonic
counter checks, independent CPU image-oracle checks, Rust formatting, diff
whitespace check and Python F lint passed. Strict release Clippy is blocked by
an unchanged `clippy::collapsible_if` at `src/botany/production.rs:97`; rerunning
`cargo clippy --locked --release --lib --example lighting_gallery -- -D warnings
-A clippy::collapsible_if` passed with that one existing lint isolated. No lint
allowance was added to source. The separate six-test shadow target retains the
one existing Scene111-vs120 assertion failure described above; it was not hidden
or corrected as part of this lighting unit. No default executable, GUI app,
installer or historical binary copy was created. Current example executable:
`target/release/examples/lighting_gallery`; failed diagnostic text was removed.
