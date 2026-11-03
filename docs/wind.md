# Spatial wind and free aerodynamic meshes

Implementation contract for EAF-213, authorized on 2026-11-03. Wind is a
velocity field, in meters/second, evaluated at a position and simulation time.
Free meshes carry their own momentum and orientation; no prescribed flight
paths, random per-frame pushes or sinusoidal replacement of rigid motion.

## Model and alternatives (before implementation)

A global Navier–Stokes discretization would require a spatial resolution,
pressure boundary conditions and persistent fluid state for the whole active
domain. That is a different feature from a bounded procedural wind field.
Here we choose an analytic, divergence-free external field and one-way coupling
to dilute rigid surfaces. Their displacement of air is neglected. Currents do
not spontaneously collide, shed wakes or generate weather. A future local
fluid solver can implement the same velocity sampling boundary.

The field combines a mean flow, smooth temporal gusts, transverse Fourier modes
with seeded phases, shear currents and compact vortices/updrafts. The Fourier
polarization is perpendicular to its wavevector. Local events are constructed
as curls of compact vector potentials: localization is applied to the potential,
not multiplied onto the resulting velocity. Therefore the analytic field has
zero divergence, including its return flows and event boundaries. The spectrum
is a finite authored approximation, not a resolved turbulent energy cascade.
Compact events use `f(s)=exp(-s/(1-s))` for ellipsoidal squared radius `s<1`,
and zero outside. This potential is C-infinity at the support boundary. Swirl
normalization uses its analytic speed maximum at `s=1/sqrt(3)` in the central
plane. Updraft is the curl of `.5 U×r f`, including the necessary return flow.
Finite lifetimes use `exp(4-1/(q(1-q)))`, `0<q<1`; persistent onset uses the
normalized exponential smooth step. Their endpoint derivatives also vanish.
Time-only phases, event centers and envelopes are prepared once on the stack
per substep/query batch, shared by all spatial samples. Reproducibility is
validated within the same build/platform; transcendental evaluation is not a
bitwise cross-architecture contract.

The free-body model uses explicitly authored triangular aerodynamic midsurfaces,
separate from the visible mesh. This avoids double-counting the two sides of a
closed leaf. Uniform areal mass integrates triangle area, center of mass and the
full inertia tensor analytically. The convex hull of the extruded aerodynamic
panels is the collision envelope; concave objects need separate convex pieces
in a future extension. Visible geometry can be more detailed than this physical
profile. These approximations must remain visible in inspection/docs.

At three barycentric quadrature points per panel, relative airflow includes
linear and angular body velocity. A passive anisotropic pressure/friction law
acts along the surface normal and its two material tangent axes:

`F_i = (rho A C_i / 2) |u_i| u_i d_i`.

For a point at offset `r`, apply the same impulse to linear momentum and angular
momentum (`r × impulse`). Each scalar drag update is solved implicitly in closed
form using its full translational/rotational effective inverse mass. This avoids
force caps and explicit drag instability on light leaves. Directional operators
are split in a palindromic order; backward Euler remains first order, and step
refinement must demonstrate convergence. Gravity, rigid rotation, CCD and real
contacts use the existing Rapier backend. Aerodynamic evaluation is f64; backend
state/contact arithmetic remains f32. This is a quasi-steady thin-surface model,
not resolved boundary layers, calibrated flutter, cloth or a general CFD solver.
Inertia must be positive definite with condition number at most 1,000,000 for
this backend; numerically unrepresentable extruded collision hulls are rejected.

## Performance preflight

- Compile modes, triangle quadrature, mass/inertia and convex shapes at scene
  admission. Share mesh geometry; never rebuild it during a physics tick.
- Bound bodies, panels, modes, events and the product of samples/substeps/modes.
  No mesh-world raycast or unbounded obstacle search per aerodynamic sample.
- Reuse compiled arrays and body handles. Sampling allocates nothing and takes
  no locks, I/O, network or GPU readback. Report actual admitted work counts.
- Existing scenes without this component retain the original one-step physics
  path. Active aerodynamics substeps the shared contact world deliberately;
  this cost is explicit and must be measured, not advertised as free.
- Aerodynamic geometry is resident even for a unique static mesh. Rendering
  overrides only the bound rigid poses, keeping authored instances, geometry
  generations and unrelated static object bounds intact. No whole-scene
  projection/copy or per-frame retessellation is introduced by moving leaves.
  Contact cost still scales with active bodies; no forest-scale physics claim.
- Persist authoring in a versioned scene component, reset it through ordinary
  edits/undo, replay from initial primaries, and preserve live body primaries in
  native/portable saves. Refuse planetary/network adapters until integrated.
- All controls leave through the shared typed dispatcher/MCP, including the
  editor. Automated numerical/integration evidence and human acceptance differ.

References: [Curl-Noise for Procedural Fluid Flow](https://www.cs.ubc.ca/~rbridson/docs/bridson-siggraph2007-curlnoise.pdf),
[OmniAD](https://visualcomputing.ist.ac.at/publications/2015/OmniAD/).
The implementation does not reproduce OmniAD's fitted coefficient database.

## Equations and numerical contract

For each triangular panel with vertices `a,b,c`, area `A` and constant areal
density `mu`, the first moment is `mu A (a+b+c)/3`. Its raw second moment is
`mu A ((a+b+c)(a+b+c)^T + aa^T + bb^T + cc^T)/12`. Sum all panels and optional
point ballast, accumulating about a nearby reference to avoid cancellation;
subtract `m cc^T` about the combined center. Inertia is
`trace(S) I - S`, with the exact through-thickness contribution of each panel's
uniform extrusion. Body scale scales lengths and inertia, while sheet and
ballast masses remain the explicitly authored masses.

The pressure model's normal axis comes from the panel winding; the first edge
defines its longitudinal material tangent. The remaining tangent is their
cross product. Normal pressure and the two directional friction coefficients
are nonnegative and act on their respective relative-velocity components.
Thus each operator is passive. This is an anisotropic surface model, not a
fitted airfoil polar; do not interpret its coefficients as universally
calibrated leaf aerodynamics. Curved/folded profiles do not currently shadow
one another aerodynamically.

With `u = d·(v_air - v_body - omega×r)`, effective inverse mass is
`k = 1/m + (r×d)^T I_world^-1 (r×d)`. For `c = rho A C/2`, timestep `h`, solve
`u_next + h k c |u_next| u_next = u` through the rationalized positive root:

`u_next = 2u / (1 + sqrt(1 + 4 h k c |u|))`.

The impulse is `h c |u_next| u_next d`; its torque impulse is exactly the cross
product with `r`. Every scalar operator reduces relative slip and, in still
air, cannot increase kinetic energy. Splitting, rotating geometry, field
variation and contacts introduce integration error; timestep refinement is
still required. The method has no force cap or assigned terminal velocity.
Normals and inertia rotate with the actual body; Rapier gyroscopic forces are
enabled. Three barycentric points integrate degree-two functions exactly, but
nonlinear airflow loads are still quadrature approximations.

`Scene116`, `replay126`, `Wind1`; native checkpoint3 and portable PSV1 retain
their outer envelopes with the new scene contract. Local mesh/physics
coordinates remain those of the ordinary scene; this is not a planet-centered
wind implementation. The diagnostic `sample` command admits finite positions
within +/-1,000,000 meters and a finite simulation time. A conservative global
speed envelope must fit 300 m/s; invalid fields are rejected, never clipped.
This numerical domain does not establish a compressible/transonic model.

Admission budgets: 256 bodies, 256 panels/body, 4096 total panels, 64 turbulence
modes, 16 gusts, 16 shear currents, 32 compact events, 1..16 substeps. The product
`3 * panels * substeps * (1 + modes + gusts + currents + events)` must not exceed
4,000,000. It counts field work, not the cost of the contact solver. All active
bodies are simulated, regardless of camera visibility.

## Shared controls and demonstration

The native **Vento** panel and MCP use `Command::Wind` with `Operation`:
`inspect`, `sample`, `replace`, `open`, `save`, `load`. Full replacement and
ordinary `SceneEdit::SetWind` are validated with the whole scene before commit;
undo/redo use the existing scene transaction. Source files use `.ignis-wind`.
Body `instance` names must resolve to independent, static visual instances.
Remove static collision, attachments, animation, object grouping, plant
connections or botany ownership before binding the instance as a free body.

Botanical XPBD sessions can opt into `settings.use_scene_wind=true` at start.
The field is sampled at each material point and at the scene/session clock,
with a 4M point-field-work admission budget. The experimental botanical session
captures the field at start; restart it after editing the scene wind. Existing constant-vector sessions
retain their legacy behavior. The botanical session retains its existing
effective area/mass drag proxy; it does not acquire the free-body surface model.
This does not enable whole-forest gameplay
deformation or automatically detach leaves from a botanical source.

```sh
python3 scripts/demo_wind.py
target/release/ignis --root . --scene examples/wind-leaves.scene.json --continuous
# Press P to run; R resets. Or open the same scene with --editor.
python3 scripts/verify_wind.py
cargo test --locked --release --test wind
```

The demonstration has 64 independent leaf meshes sharing one visible asset,
each with two aerodynamic triangles, a small stem ballast and its own rigid
state. It combines spatial shear currents, advected turbulence, an eight-second
vertical gust and horizontal components that turn over sixteen seconds, plus
a weaker ascending vortex with a smooth onset. Three thin posts are two-meter
world references. The camera includes the lifting region, and the floor is
wide enough for the measured dispersion. Leaves rise, translate, change course
and settle during weaker phases; subsequent gusts can lift them again. They
can disperse beyond the fixed camera over a long session; R starts the example
again. This is an authored example, not an identified species or an emergent
meteorological simulation.

### Correction after native visual review

The initial recipe in `b1798f9` lost its airborne population quickly: 45 of 64
bodies were near the floor at three seconds, and 62 at ten seconds, with mean
body speed 0.0444 m/s at ten seconds. The remaining visible rocking did not
adequately demonstrate the requested transport. A short nonzero-displacement
check had missed this presentation failure.

Only the demo field, framing and floor/reference geometry changed. Surface
mass, inertia, aerodynamic coefficients, contact solver and body integration
remain the same. No trajectories, teleports or angular animation were added.
The focused verifier now checks late transport and opposite horizontal motion:

| Window (s) | Bodies travelling >1 m | Airborne at end | Mean displacement (m) | Mean X displacement (m) |
|---:|---:|---:|---:|---:|
| 8–12 | 61 | 59 | 7.774 | -3.013 |
| 16–20 | 59 | 55 | 7.141 | 2.207 |
| 24–28 | 60 | 57 | 7.294 | -2.891 |
| 56–60 | 56 | 55 | 6.651 | -2.548 |

Nine MCP groups passed on build `d8dec4c557b67a8c`, including the existing
save/replay/contact checks; portable continuation's sampled maximum error was
0.001696 m. Weighted field work is now 35,328 per tick for 64 bodies. The new
native window was also observed after P: PLAY with advancing ticks, airborne
leaves above the reference posts, and spatial displacement. Artistic acceptance
still belongs to the user. Raw diagnosis and parameter-study results are under
`artifacts/wind/{motion-diagnosis,transport-study,transport-refinement}.json`;
the current verifier report is `artifacts/wind/verification.json`.

## Initial validation evidence (2026-11-03, recipe b1798f9)

macOS arm64/Metal, Apple M4 Pro; tested release build `3b1639bd281e9ed2`.
Evidence is local under `artifacts/wind/`; source and the reproducible scripts
are committed. Eight wind MCP groups exercise real translation/rotation,
shared botanical opt-in, convex floor contacts, atomic authoring, source
save/load, undo, editor capture, native/portable primaries and replay. Replay
captures and native continuation are exact in the tested build. Portable
restoration preserves the same primary bits; rebuilding contact history led
to a maximum 0.021201 m position difference after 60 further ticks among the
64 leaves tested. Portable continuation is not a bitwise guarantee.

The maximum sampled central-difference divergence was `1.518355e-7 /s`
with `h=1e-4 m`; the underlying continuous construction is solenoidal.
Botanical external impulse was zero with opt-out and 0.106662 kg m/s with
opt-in in the gravity-free fixture. Existing laboratory and retained-render
MCP checks also pass headlessly. A separate real-executable KCC probe moved
2.000045 m horizontally both without aerodynamics and with 1, 4 or 16
substeps; the full displacement vectors matched exactly in that sample. Numerical tests cover exact lamina moments,
positive/passive drag and impulse torque, scalar timestep convergence, motion,
contacts, authoring/replay/save integrity and unchanged resident geometry.

The timing table below used the original recipe in `b1798f9`; the revised
demonstration above has a different field and does not inherit these timings.
`examples/wind_cost.rs` measures the complete authoritative CPU tick, including
contacts, with two panels/body, four substeps, 16 modes and the demo field.
Each case warms 120 ticks, then measures twelve windows of 60 ticks. The p95
column is the empirical nearest-rank result of those twelve window means.
Other development processes were active; this is a local workload measurement.

| Bodies | Median CPU ms/tick | p95 CPU ms/tick | Weighted field work/tick |
|---:|---:|---:|---:|
| 1 | 0.013 | 0.015 | 504 |
| 64 | 0.791 | 0.874 | 32256 |
| 256 | 3.476 | 3.570 | 129024 |

Prepared field sampling alone took 156.668 ns/query in a 100,000-query batch.
These timings exclude rendering and do not establish a universal body limit,
frame rate or total GPU/memory cost. Larger authored panel counts or contact
loads can cost more within the same admission envelope.

Commands and raw evidence: `artifacts/wind/{verification,cost,rust-summary}.json`,
`build-final.log`, `clippy-release.log`, `fmt.log`, and the regression subfolder.
The verification scripts were executed directly: the general verification
runner's pre-existing discovery/registration mismatch is a separate gap; no
complete MCP-suite claim is made. During that initial automated gate, no native window was opened or replaced;
the existing editor was preserved. The later requested native demonstration
and visual correction are recorded above. The one current application executable is
`target/release/ignis`; no archived/copied app build was created. Human artistic
and native-input acceptance remain separate.
