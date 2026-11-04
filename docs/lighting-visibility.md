# Camera-driven lighting work — EAF-208

## Contract

Shade only receivers visible to the camera. Keep visibility of receivers separate
from visibility of influencers: an object behind the camera may cast a shadow,
occlude the ambient hemisphere, or contribute through a reflection/bounce.
Do not apply the camera frustum to all shadow casters or indirect-ray geometry.

Audit of the current source found two existing mechanisms:

- `render_visibility.rs` tracks `CAMERA` and `SHADOW` independently and tests
  each against its own conservative volume. This is already active by default.
- `lighting_gallery/estimators.wgsl` creates one receiver at the nearest positive
  primary intersection per camera pixel. Background gets no AO rays. There is
  no lighting evaluation on hidden faces or on the rest of the world; bounding
  tests against invisible geometry are visibility work, not shading that geometry.

This increment tests conservative influence pruning for the finite-radius AO
laboratory, without changing production lighting. For the biased ray origin
`o=p+0.002*n`, keep an AABB only when its distance to `o` is at most4m and its
maximum projection onto `n` is nonnegative. Any occluder reached by a cosine
hemisphere ray shorter than4m necessarily satisfies both conditions. Pad both
comparisons by0.0001m in this bounded scene. Preserve ascending candidate order.
Do not reuse the 4m cutoff for unbounded GI or arbitrary scene scales.

The diagnostic uses a per-receiver32-bit influence mask in unused normal-vector
padding, so the160-byte receiver stride is unchanged. It supports at most32
boxes and rejects larger diagnostics; this is not a new engine asset limit or
an acceleration structure suitable for production worlds. No frustum test is
used for AO influencers. Default gallery execution leaves the optimization off.

## Verification before a performance claim

Run paired full-scan/pruned variants of all ten existing methods with identical
random streams, native raster, camera and sample budgets. Rotate/invert method
order, alternate pair order;128 timing samples per method/variant. Build the mask
outside warm samples, disclose its setup cost, and preserve scene/cache rebuild
requirements. Read counters only after fenced GPU completion, as in the gallery.

Compare every captured PNG and raw AO map byte-for-byte, not merely by MSE.
Read back the receiver map outside sampled time: check receiver frustum coverage,
object hit counts, influence occupancy and conservative CPU bounds independently.
Report candidate-test ceilings separately from actual executed ray/box tests;
early exits make ceilings unsuitable as measured work counts.

A second scene places a blocker entirely behind the camera, near the visible
floor. It must receive zero primary hits while remaining an AO influencer.
Full-scan and safe pruning must agree. A deliberately wrong negative control
removes that blocker only from AO queries and must alter the output. Its image
is a failure example, not another valid candidate or an engine feature.

```sh
cargo build --locked --release --example lighting_gallery
./target/release/examples/lighting_gallery --visibility-audit
./target/release/examples/lighting_gallery --visibility-audit --offscreen-guard
uv run scripts/lighting_visibility_report.py
```

Stable outputs: `artifacts/lighting-visibility/{gallery,guard}` and summary/report
at the parent. Prior lighting-gallery evidence is preserved. One current example
executable at the normal Cargo path; no GUI, production renderer, scene version,
material, app build or asset change is required.

## Results — 2026-11-04

gallery: 153067 visible receivers, 113173 background pixels; mean candidate boxes 11→4.131. MC16 compute 0.558→0.329ms, GPU render+compute 1.480→1.248ms. Paired median compute saving 0.227ms; 95% paired block-bootstrap interval [0.225,0.232]ms (2000 resamples, eight consecutive frames/block, independent rounds). CPU totals and p95 remain descriptive in summary.json; no CPU speedup claim on a shared machine.

guard: 221954 visible receivers, 44286 background pixels; mean candidate boxes 3→1.488. MC16 compute 0.346→0.288ms, GPU render+compute 1.554→1.477ms. Paired median compute saving 0.064ms; 95% paired block-bootstrap interval [0.062,0.067]ms (2000 resamples, eight consecutive frames/block, independent rounds). CPU totals and p95 remain descriptive in summary.json; no CPU speedup claim on a shared machine.

Behind-camera control: object2 has 0 primary hits and remains a possible AO influencer for 108330 visible receivers. Removing it changes 96437 pixels. Full/pruned PNGs and raw maps for all ten methods are byte-exact in both scenes. Independent CPU bounds conservatively cover necessary influences and verify every receiver projects within the camera frustum. All target-query streams/budgets are preserved; only obstacle intersection candidates change.

Mask construction is included in per-method setup_gpu_ms and is not amortized
into the warm-frame totals. The comparison shares this prepared mask between
variants; it is not an isolated incremental setup comparison. Rebuild after
camera/geometry changes. Receiver storage stays160bytes/pixel; no extra GPU
mask buffer or per-frame CPU culling was introduced. Receiver readback is an
additional42MB diagnostic transfer/staging allocation outside timed samples.
It is retained only as current audit evidence, not part of the game frame.

Validation: release example build and both actual GPU runs passed; all captured
variants equal, monotonic fenced timestamps, negative control detects a lost
offscreen influence, CPU receiver/frustum/influence checks, Rustfmt, diff check
and Python F lint passed. Release example Clippy -Dwarnings with the previously
identified unrelated collapsible_if lint isolated passed; no lint allowance was
added to source. Existing shadow-version test and strict-Clippy limitations
remain as documented in lighting-candidate-gallery.md. This increment changes
only the explicit laboratory; it does not enable production GI or claim a
world-scale BVH/streaming/visibility solution.
