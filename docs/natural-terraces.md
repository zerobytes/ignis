# Natural connected terraces — EAF-221 experiment

Build an independent native authoring field; preserve the previous approved
volumetric scene. Each level adds3–15m relative to the previous level. No layer-
count or artistic total-altitude limit: H(k)=mean*k+jitter(k)-jitter(0), with
bounded jitter guaranteeing the adjacent increments. Existing finite-coordinate,
height and mesh/work budgets remain technical limits.

Connections are irregular broad regions of the surface, not overlaid ramp
primitives or engineered paths. Warped radial contours produce plateau stains;
seeded winding angular regions widen the height transition continuously. Region
width, meander, orientation and boundary noise vary; both ends use zero-derivative
smooth interpolation. Cliffs remain outside these regions. Collision uses the
published mesh. Native queries expose inspection routes through broad connected
cores for geometric/physical verification, not geometry authored in Python.

This first field is a single-valued height surface, not a replacement for the
approved volumetric overhang tools. Planet-runtime integration, navigation graph
connections, caves, erosion simulation and final artistic acceptance are separate.

Static performance review before implementation: constant work per field sample,
O(1) level height even for thousands of levels, no prefix scan or per-level node
allocation. At most16connection regions and bounded source samples/work. Cold
bake only; no procedural regeneration per frame. Grid publication uses existing
indexed terrain/physics transaction; chamfer cost remains explicit. Source4 reads
legacy1..3 without rewriting them; MCP query/publish/save/load remain the entry.

## Verification and use

`python3 scripts/open_natural_terraces.py` prepares and verifies headlessly;
`--prepared --open` opens the current scene, initially walking. WASD/mouse,
Shift to run, V to toggle session inspection flight, Space/C vertical in flight,
Ctrl slow flight, Tab first/third person. The native and visual editor bindings
use the pre-existing MCP flight command; keyboard-owned inputs release before
toggling, including rejected mode changes. NPC diagnostic V is retained outside
a player view. The launcher reuses the sole current release executable.

600×600m, nine2m grids, six consecutive increments:
7.15953,8.67294,7.68103,8.61759,9.92917,10.59778m. Native/MCP queries also sampled
a radius10000m formation at2245.3749m; no high-altitude gameplay proof is inferred.

The full styled mesh traversal used gravity/collision, no inspection flight or
teleports:52.6763m ascent,3824fixed ticks, grounded finish. The45° ground motor
is explicit; the default30° motor blocked on a steeper part, so this is not an
all-NPC accessibility guarantee. Curve query's maximum longitudinal slope0.6286
is distinct from the full surface normal and the physical motor limit.

18 release integration tests passed (4new kernel/physical,9surface program,
3terrain,2collision), old source1..3 byte roundtrips preserved; source4 required
for the new operator. Save/load rendered pixels matched. Native captures were
reviewed. Source operation creates91809raw grid vertices/180000triangles;
chamfered/material-separated demo1238508triangles (47692stone/1190816green).
Procedural generation and the complete MCP/physics/capture check took about
13–21s in these runs; shared compiler queue time is not generation time or FPS.

Strict Clippy initially stopped on an unrelated concurrent formatting warning in
`service/botany.rs`; rerun with that single suspicious-assignment-formatting lint
allowed passed. Focused format/Python/whitespace checks passed. Other botany,
lighting and game-UI changes were preserved. This remains an experimental height
field: not volumetric overhang reproduction, erosion, a new nav graph or planetary
runtime integration. The approved volumetric scene and recipe are preserved.
