# Independent continuous growth and metric leaf nodes

Grammar4 implements the next step from [the growth research](botany-growth-research.md).
It is deterministic fixed-site authoring, not a calibrated physiology/carbon model.

## Definition and compatibility

`Definition.continuous` is required exactly for grammar4; `stature` remains exclusive
to grammar3. Grammar4 also requires `foliage`, with leaf turnover at least as long as
leaf expansion. Definition1/2/3 behavior is unchanged; absent new fields serialize
without additional keys. Old readers reject grammar4 explicitly. The generated
source remains8 and published recipe10; Scene121/replay131/protocol5 are unchanged.

`Living/SetContinuous` upgrades a broadleaf individual explicitly, preserving its
time, production sites, harvests, pruned sites and settings. It uses the existing
atomic publication/manual-edit preservation guards. It does not silently upgrade
old saved sources. Invalid profiles or unresolvable junction geometry leave the
saved tree intact. The old uniform Stature operation refuses grammar4.

Both creation and selected-tree controls in Vegetation expose the same profile as
MCP, under **Crescimento contínuo e nós foliares**. The example definition is
`examples/living-tree.recipe.json`; its native birth source is
`assets/botany/living-tree.ignis-botany`.

## Size curves

Age `a` is in years of365 game days. Reference ages indicate95% of the increment
between the initial value and the configured limit; they are not stopping ages.
The rational family deliberately retains a resolvable tail over long authoring ages:

```
tauH = height_reference_years / 19
H(a) = H0 + (Hlimit-H0) * a/(a+tauH)
D95 = D0 + 0.95*(Dlimit-D0)
f95 = (D95^2-D0^2)/(Dlimit^2-D0^2)
tauA = diameter_reference_years*(1/f95-1)
D(a) = sqrt(D0^2 + (Dlimit^2-D0^2)*a/(a+tauA))
```

The diameter equation grows cross-sectional area. An initial diameter equal to
its limit is an explicitly constant profile. Finite precision eventually limits
resolvable increments; no claim of visible growth at every representable age.
These effective limits are authoring parameters, not universal botanical maxima.

H measures the main axis endpoint and D its basal diameter, not DBH. Crown tips
can extend above H. The trunk's vertical path follows H; other woody axes follow
their supporting anchor and a separate slowly increasing crown factor. Changing
only D changes radii, not axis paths. Branch relative radii expand with their
shoots and D, without inheriting the old maturity clamp. Small woody branches use
2.2/3/4mm radius floors (twig/secondary/primary) to keep native junction collars
resolvable; these are geometry proxies, not biological minimum diameters.

Height, diameter and internode parameters remain subject to actual spline/junction
validity and output budgets. Incompatible configurations fail rather than clamp to
a secretly substituted tree. Reference profiles do not encode resource history.

## Shoots and foliage

Shoot extension has its own duration (120days in the example), independent of
radial growth. New secondary and terminal shoots use this clock rather than
multi-year extension delays. Primary production-site birthdays remain unchanged.

Leaf nodes use a cumulative128-sample table of the **same Hermite spline kernel**
as native bake. Distances are in generated rest-axis metres, not normalized
control-point positions. Subsequent manual deformations can override that rest
spacing; this is not a leaf contact/cloth solver.

Two opposite leaves emerge per node. Additional nodes require both space along
the shoot and the configured emergence cadence. Leaf counts cap node capacity;
the whole adult count is not instantiated on a newborn shoot. Leaf emergence is
the later of the cadence and the first integral game day with sufficient axis
length, found by bounded binary search. A local per-axis cache avoids repeating
length evaluations. The starting individual is an established seedling: nodes
that fit its initial shoot already bear leaves.

Each node retains its metric distance and identity. Additional sites do not
redistribute old ones. A bounded apical window follows newly available leader
nodes; older lower leader leaves retire from that window. Independent leaf
expansion still uses `foliage.expansion_days`.

Leaf cohorts renew on `leaf_turnover_days`, with at most two cohorts during the
new leaf's expansion window. IDs include supporting axis, node, cohort and side.
New and old leaves have different orientations. Renewal remains bounded even at
ancient ages; pruning a primary excludes its entire descendant/cohort subtree.
Fruit harvest identities remain separate from leaf-cohort IDs.

The woody scaffold is still finite and authorable; this is not endless creation
of branch orders or biological deadwood. Existing axes continue changing size
and leaves renew after scaffold formation. Sculpt tied to a retired leaf address
can refuse preserved regeneration rather than discard work. Automatic gameplay,
resource assimilation, seasonal climate and full historical replay remain separate.

## Reproduce

```sh
cargo test --locked --test living_continuous
cargo build --locked --release --bin ignis
python3 scripts/demo_living_tree.py
uv run scripts/verify_living_preview.py
python3 scripts/open_living_tree.py
```

The gallery reports heights/diameters at0/1/2/5/10/12/20/30/100/250years, along
with young closeups. Native save/reopen and reversible preview compare exact PNGs,
including the editor viewport. Scalar tests cover reference accuracy, continued
growth through1000years, independent height/diameter paths, node spacing,
renewal, pruning, serialization, explicit upgrade, invalid-profile atomicity and
undo/redo. Visual acceptance and population/frame performance remain distinct.
