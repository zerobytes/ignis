# Botanical width and age-driven stature

The neutral specimen from the first wood-authoring increment was rejected as
far from the approved ancestral reference. The reference is
`games/land-of-survivors/references/ancient-white-elven-tree.png`: a low, massive
bole, early broad structural arms, interwoven volumes and sinuous exposed roots.
The small forked fixture is not an accepted substitute. No final tree is installed.

## Width is a design parameter

The source-axis10m radius ceiling, woody-contour50m radius ceiling and5× radius
multipliers have been removed. Relief width/height and local centre offsets also
have no fixed artistic magnitude ceiling. Native fields accept finite positive
radii/multipliers; negative relief remains possible. Polar plots normalize in
f64 before converting screen coordinates, so a large draft value does not turn
into an overflowing f32 diagram.

The organic collar/fusion100m ceiling is removed too. Numeric precision,
nonzero size, valid frames, nonintersection checks in the existing local kernel,
sample/vertex/triangle budgets and the mesh-coordinate domain±10000m remain
technical admission conditions. An unrepresentable result fails explicitly and
preserves the valid scene. This is not a promise to represent infinite geometry.
The minimum authored wood-contour radius0.02m remains a feature-domain rule;
the living seedling-size parameter is a separate physical dimension.

A regression scales the complete shaped trunk/arms by100: nominal trunk radius
250m and contour samples over250m, with closed oriented geometry and unchanged
sample counts. This proves acceptance beyond the former ceilings, not visual
fidelity, a botanical species or performance at world scale.

## A persistent stature profile in years

Living grammar3 adds optional `definition.stature` and preserves grammars1/2
without changing their generation. The new version is required when the field is
present; older readers reject it. The profile is part of the canonical individual:

```json
{"adult_diameter_m":5.0,"seedling_diameter_m":0.01,"maturity_years":100.0}
```

With elapsed game years `a = (time - born) / 365`, let
`u = clamp(a / maturity_years, 0, 1)` and `g = u*u*(3-2*u)`. Then:

`D(a) = seedling_diameter_m*(1-g) + adult_diameter_m*g`.

There is no fixed adult-diameter upper bound. All dimensions must be finite and
positive, and the seedling diameter cannot exceed the adult value. The current
profile begins with a seedling; a modeled seed/germination organ and complete
biological development are not implemented by this equation.

The native generator still creates branches/leaves by canonical age and stable
site identity. It evaluates its graph and then dimensions the woody structure
to `D(a)`. It does not merely scale a stored final triangle mesh. Leaves retain
their own age-dependent length/width/thickness, while their insertions follow the
supporting axes. Production, pruning and harvest identities remain unchanged.

**Scope:** this stature trait retains the structural proportions of the existing
grammar2. It can produce a much taller tree when the adult diameter is increased.
Independent mature height, canopy/bole proportions and the massive low ancestor
architecture require species-grammar controls and remain open. This trait alone
must not be presented as a reconstruction of the approved reference. The
separate wood-surface authoring controls still apply to nonliving authored sources;
the shared living shape-history contract remains future integration work.

## Native controls and commands

Vegetation → **Nova planta viva** → **Árvore · porte configurável** creates a
grammar3 individual. Existing grammar2/3 individuals expose **Porte e crescimento
do tronco**, with numeric adult/seedling diameter and maturity age fields.
**Aplicar perfil de porte** uses the same operation available to procedural clients:

```json
{"op":"botany","operation":{"kind":"living","prefix":"living","action":{"kind":"set_stature","stature":{"adult_diameter_m":5,"seedling_diameter_m":0.01,"maturity_years":100}}}}
```

Changing the profile is an explicit authoring operation at the current saved age;
source, geometry and canonical profile publish atomically. It does not advance
or rewind the clock. Undo/redo restore the previous profile and bake. Existing
reversible age preview remains controlled by the separate editor-preview path.
Save/load retain the new profile and regeneration is deterministic. Reproduction
and the automatically ticking gameplay clock have their existing boundaries.

## Validation

Focused tests cover two stature/growth/persistence/atomicity cases, the former
width ceilings on a large closed shaped source, existing living source/preview
cases and organic collar/fusion integrity. Native evidence and measured values
are written to `artifacts/botany-stature/verification.json`. Tests are technical
checks; the ancestral form remains artistically rejected/pending.
