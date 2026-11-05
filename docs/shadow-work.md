# Native zero-contribution shadow queries — EAF-208

Preflight2026-11-05. First execution from the
[GPU Gems applicability review](lighting-gpu-gems3-review.md): reduce EXISTING
lighting work with new GI absent. The reference is the unmodified visibility
function/shading response, not a low-quality replacement or a GI estimate.

Candidate: after existing normal flip/bump, require shadow visibility only when
`N·L>0` or a positive thin-sheet transmission lobe has `(-N)·L>0`. Dielectric/GGX
already returns zero at nonpositive N·L; glass keeps its existing response.
Preserve all directional shadow casters, including those outside the camera.
Receiver-plane derivatives execute before the nonuniform rejection; guard only
the remaining projection/sampling. WGSL derivative checks must remain enabled.

A compile-time override defaults to the legacy path for unrelated shader
consumers. A profiled renderer constructor selects reference/candidate for
standard unique/shared/transparent pipelines; normal constructors initially
retain legacy until quality and timing gates pass. No uniform/storage/resource
layout, scene/save format, runtime settings/UI or shader timestamp overhead in
ordinary construction is introduced. Keep the original function as an oracle.

Compare both paths at1280×832, same geometry/light/shadow map2048, GI absent.
17material/control modes ×3camera poses: matte, receiver-plane bias, dielectric,
metal, thin transmission, masked leaves, glass, bump, emission+IBL, zero-direct,
shadow-off, light-volume boundary, plus shared minimal/bump/transmission/mask/
glass panels. Exact PNG equality for51static pairs and4camera/sun perturbations
per case (204moving pairs). No tolerances hide changed shadow/contact pixels.
Warm8pairs per case and alternate order for64timings/path/case. CPU preparation/
submission, GPU shadow/main/composition and total are separate; readback/fences/
PNG and pipeline construction are outside frame timing. This is not game FPS.

Acceptance BEFORE examining results: every image pair exact; no extra passes or
render targets; clear benefit (at least2%geomean total-GPU reduction) across the
representative matte/bias/dielectric/metal/bump/emission+IBL E/F poses. Front matte,
front bias, all shadow-off and glass/transmission/mask controls must stay within
5%total GPU regression in measured medians. Report all cases and paired timing
uncertainty, not only dark back-facing views. If gates fail, retain the original
normal-constructor path and record the candidate as opt-in/rejected; do not
claim the original engine became cheaper. Shared panels also test the existing
uniform-material specialization and detailed vertex paths.

One current diagnostic executable `target/release/examples/shadow_work`, stable
outputs `artifacts/shadow-work`. Existing user apps/windows and parallel botany/
terrain/game changes remain untouched. This unit's extra GPU timing API is
explicitly diagnostic; no new lighting feature is added.

## Result — candidate not promoted

The full valid run passed51static pairs and204short-sequence pairs, including
51zero-step sequence baselines and153actual camera/sun perturbations. Every PNG
is exact; native instancing/transparency/procedural resource counters also match.
Glass genuinely exercises transparent composition, shared modes exercise shared
instancing, and nonshared material cases use native mesh materials with the
existing unique-residency oracle. Initial fixture attempts with nonexistent
entity material fields or invalid glass ownership were rejected/corrected;
they supply no accepted timing record.

Apple M4 Pro / Metal1280×832,64timings/path/case, rotated order and paired-trial
bootstrap. Representative E/F geometric mean candidate/reference total-GPU
ratio is0.98946 (~1.05%lower), failing the preregistered2%benefit gate. The
shared-mask E control ratio is1.0691, exceeding the5%median-regression gate;
its interval[0.855,1.203]includes equality, so this is not evidence of a universal
6.9%loss either. Many case intervals include1.0. No sufficiently clear aggregate
benefit is demonstrated on this shared workstation/backend.

Decision: keep ordinary constructors on the original path. The compile-time
candidate remains available ONLY through the explicit profiled diagnostic API;
no default optimization or original-renderer speedup is claimed. A geometric
eligibility count does not tell us executed GPU work, bottleneck distribution,
branch/divergence cost or resulting wall-clock savings. Further tuning or another
backend requires a new declared experiment, not repeated runs until it passes.

Both old and selected functions preserve derivative flow. The old visibility
function is kept verbatim as the diagnostic oracle, and unrelated shader consumers
retain the override's legacy default. No additional render targets/passes/layout
or per-frame state are introduced. The diagnostic adds a pipeline-creation option,
not a persisted user setting or a change to scene appearance.

Raw evidence: `artifacts/shadow-work/{results.json,summary.json,review.html}`,
all static images, source/executable hashes and per-pass CPU/GPU timings. CPU
measurements include `Engine::update_render_frame` preparation into a reusable
frame plus draw submission; synchronization/readback/PNG are excluded. This is
neither scanout/gameplay FPS nor an isolated GPU benchmark. No new GI is enabled.

Reproduce `uv run scripts/run_shadow_work.py`; reanalyze unchanged recorded data
with `--report-only` (requires matching source/executable hashes). Continue with
response-coefficient storage for supported color changes as a separate hypothesis.

Final validation: release build and complete51-case GPU execution passed;
3existing shader-validation tests passed (including surface-field and object
impostor consumers of scene.wgsl). Scoped release example Clippy with-Dwarnings
and the pre-existing unrelated collapsible_if lint isolated, Rustfmt2024/PythonF
and diff checks passed. No all-workspace acceptance is inferred.
