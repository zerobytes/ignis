# Regional sampled-bake repair — EAF-208

Preflight 2026-11-04. Extend the32per-face/4096samples atlas diagnostic, not the
production renderer. Target: reuse unchanged texels after a local material or
axis-aligned geometry edit and match a full rebuild, including off-camera data.
One fixed sample stream is retained; equality to this finite sampled atlas is
not certification of the continuous radiance integral or general-mesh quality.

## Dependency support and conservative invalidation

Each texel stores48bytes: first-hit object bitmask, first sun blocker in the visibility loop order bitmask,
covering-solid mask (including previously invalid texels), and AABB of secondary
endpoints with positive sun cosine, including shadowed endpoints. Up to32stable
object slots, one declared edit per transaction, same chart/sampling topology.
Material changes invalidate texels whose first-hit mask contains that object.
A receiver's own albedo is applied at query time; its indirect incident field
can still depend on that material elsewhere through the first-hit mask.

Geometry changes also invalidate the moved receiver's charts and recorded
hit/blocker/cover dependencies. To catch new intersections, an unchanged receiver
is conservatively dirty if the new object reaches its outward hemisphere.
To catch new secondary shadows, test the cached endpoint AABB against the new
object's sun-aligned shadow prism: overlap on two transverse axes and the
necessary depth ordering. Expand endpoint bounds for the0.002m normal offset
and numeric margins. No camera/frustum cutoff, distance radius or spatial
neighborhood is used to exclude possible influences. False positives are allowed;
missed changed texels are a failure. This is support for sampled paths, not a
universal error bound or arbitrary multi-bounce dependency graph.

Sun direction/global light changes invalidate everything. Insertions/deletions,
slot reordering, multiple undeclared edits and chart topology changes are outside
this regional protocol and require a new cache. Camera changes do not affect
these dependencies. The existing bake package format is unchanged; regional
metadata snapshots/manifests are diagnostic sidecars, not a production save API.

GPU classification and workgroup-prefix compaction produce an indirect repair
list; CPU does not read counters to schedule repair. Recompute only dirty texels
and their dependency records with identical RNG keyed by texel. Classification,
compaction, dependency maintenance and publication cost are charged, not just
ray count. Compare against both ordinary full bake and full bake with dependencies.
A high dirty fraction or global edit may erase the benefit; report those cases.

## Publication and pending views

Repair uses a private candidate atlas. The visible atlas remains separate.
While dirty, any lookup whose four bilinear taps touch dirty support traces
fresh64; clean support reuses the previous bake. After repair, atlas and cleared
flags are copied in one ordered GPU command batch. The pending frame must equal
fresh64 on fallback pixels and the fully rebuilt atlas on clean pixels. The
published RGB/PNG must equal the full rebuild. No partially repaired chart may
be consumed as valid. This laboratory uses fences/readbacks for diagnostics;
it does not implement an asynchronous engine job scheduler or continuous FPS.

## Fixed sequence and verification

Both original open/thin-wall scenes run a sequential12-event chain: no change,
wall recolor, small-object recolor, two translations, return, roof resize/return,
an elevated blocker/return, sun direction change, no change after the global
update. Previously updated dependencies are reused through the whole sequence.
The elevated blocker is at(5,9,3), size(3,1,3); report actual visible pixel count
rather than assume it is outside the camera. Off-camera shadows remain relevant.

Each event compares every atlas texel and dependency record against a full bake,
checks that no changed texel was omitted, verifies the compact list/indirect args,
unchanged/empty and full dirty lists, pending composition and published RGB/PNG.
An independent CPU classifier verifies dependency decisions; CPU transport checks
bright metadata samples. Repeated timing restores the same old cache outside
measurement, rotates16samples each of partial/full-with-deps/full-plain, and
charges publication to each. Parameters/geometry are uploaded before GPU timing;
that small upload and setup/snapshot/readback/file I/O are disclosed separately
or excluded, never gameplay FPS. Render-after-repair costs are measured separately.

Stable output `artifacts/lighting-regional`; one current example executable.
No production shader, engine format, material defaults, app GUI or dependency
change. Preserve parallel botany/game edits. Human acceptance, dynamic actors,
textures/curvature, UV/triangle meshes and general multi-bounce transport remain
separate requirements.

Implementation notes: global light events bypass dirty-list compaction for the
actual rebuild and use a full dependency bake after stamping pending support.
The empty-list path remains an explicit diagnostic control for no-change events.
After timing trials, restore snapshots of the actual partial output; do not
rebuild the cache from the oracle between sequential edits. The recorded legacy
`changed_rgb` count compares the entire RGBA texel, including validity; the
report labels this as changed atlas state, not only a change in emitted RGB.

## Result — 2026-11-04

All24events passed: zero missed changed atlas texels, repaired atlas and all
48-byte dependency records byte-identical to full dependency rebuild, published
RGB/PNG exact. The ordinary full bake also has exactly equal atlas RGB/validity
on these runs (maxfloat difference0), not merely a perceptual tolerance. Restored
partial-output snapshots feed each next edit. GPU classifier and compacted list/
indirect args match independent CPU checks for every texel.12bright CPU samples
×4096paths verify all three masks, RGB (maxdifference0) and endpoint bounds
(maxdifference7.18e−5m versus3e−4check tolerance).0.001m classification margin
exceeds that observed error in this bounded scene; no arbitrary-scale proof.

Apple M4 Pro / Metal,32per chart edge,4096samples,16rotated update samples per
method/event. Below: full-plain and regional GPU cost INCLUDING publication;
classification/compaction/dependency refresh are included in regional. Geometry/
parameter uploads, diagnostic setup/restore/readback and file I/O are excluded
from GPU intervals. Per-event432/384byte geometry upload CPU submission is logged
separately, and is not a measured GPU upload cost. No gameplay/FPS inference.

| Scene / edit | Dirty texels | Full bake GPU ms | Regional GPU ms | Reduction | Changed object pixels |
|---|---:|---:|---:|---:|---:|
| open/wall-color | 44.2% | 33.884 | 20.337 | 40.0% | 8717 |
| open/small-color | 21.3% | 33.812 | 10.296 | 69.5% | 1720 |
| open/move-a | 64.2% | 34.380 | 26.305 | 23.5% | 1625 |
| open/offscreen-blocker | 74.8% | 32.232 | 27.543 | 14.5% | 0 |
| open/sun-direction | 100.0% | 33.467 | 31.715 | 5.2% | — |
| thin-wall/wall-color | 46.6% | 41.162 | 25.242 | 38.7% | 8597 |
| thin-wall/small-color | 13.5% | 41.037 | 10.062 | 75.5% | 1720 |
| thin-wall/move-a | 75.0% | 40.971 | 34.843 | 15.0% | 1734 |
| thin-wall/offscreen-blocker | 75.7% | 38.764 | 34.211 | 11.7% | 0 |
| thin-wall/sun-direction | 100.0% | 40.637 | 37.474 | 7.8% | — |

Small-object recolor saves69.5–75.5%update GPU, wall recolor38.7–40.0%, geometry
edits8.0–33.6%. No-change events trace no repair rays and cost0.052–0.116ms for
the diagnostic empty-list/mark/publication path. Global sun change dirties100%:
it uses full dependency rebuild directly, with no local-work saving inferred.
Full-with-dependencies happens to run faster than the older plain bake in this
shader/compiler setup; these are different implementations, not evidence that
recording metadata is intrinsically free or faster. Both baselines are retained.

The elevated blocker contributes0camera pixels in BOTH scenes, yet changes
18632/23776atlas texels (RGB/validity). Off-camera geometry cannot be culled from
lighting solely because the camera does not see it. Dependency masks/half-space/
shadow-prism tests preserve its effects. Dirty fractions are conservative and
reach82.1%for a geometry event: correctness is prioritized over excluding every
unchanged sample. This leaves room for tighter directional support representations.

The required dependency payload is2.25/2.531MiB (48B/texel), flags0.188/0.211MiB,
in addition to0.75/0.844MiB atlas and private/visible copies, work lists and dispatch
arguments. The shared diagnostic actually reserves7.031MiB per receiver/metadata
buffer and1.172MiB for its existing two-list work buffer; the required payload
figures are NOT total allocated GPU memory. It also keeps separate oracle/view
labs and snapshot/readback resources for verification. Compression/coarser support
would trade memory against conservatism and is not implemented here.

CPU preparation/submission increases roughly0.002–0.089ms versus full-plain in
these medians; synchronization and diagnostics are excluded. No CPU speedup.
Local repair still takes approximately10–37ms per EVENT in this small lab, and
while pending44–96%of pixels may need fresh64 in representative local edits.
It is not yet a60Hz dynamic-GI solution: budgeted/incremental jobs and a real
async publication scheduler remain next work. Global direction changes still
need full work; unchanged steady frames reuse the baked atlas.

Build and final scoped release Clippy(-Dwarnings with the pre-existing unrelated
collapsible_if lint isolated), Rustfmt2024, Python F checks, CLI rejection,
current source/executable manifests and both GPU runs passed.1152timed update
records +768post-repair render records are retained. No all-workspace test claim.
No production renderer, engine/save format, GUI or dependency change, no push.
One current example executable and bounded current scientific outputs.

Reproduce: `uv run scripts/run_lighting_regional.py`. Evidence in
`artifacts/lighting-regional/{review.html,summary.json}`, per-event old/new chain
metadata, dirty masks/list/args, pending/published/direct maps and images. The
main next boundary is a frame-budgeted repair/publication scheduler, then tighter
support/memory, multi-edit/version handling and general triangle/UV surfaces.
EAF-208 remains In Progress; motion/artistic/production acceptance remains open.
