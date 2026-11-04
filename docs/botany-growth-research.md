# Tree growth research and current-model audit

Research checkpoint: 2026-11-04. This is a diagnosis and design proposal;
no growth curve, asset, renderer or running scene was changed in this checkpoint.

## Botanical evidence

Woody trees have separate mechanisms for shoot extension and cambial thickening.
Buds contain shoots and leaves; cambium adds wood around existing woody axes.
Growth is episodic and responds to resources. A branch does not need to wait for
secondary thickening to support expanding leaves.
[University of Georgia Extension, *Shade and Street Tree Care*](https://fieldreport.caes.uga.edu/publications/B1031/shade-and-street-tree-care/).

Height is subject to hydraulic and mechanical constraints. Koch et al.'s redwood
study estimated height limitations from water relations. Its numerical estimate
is specific to the studied context, not a universal maximum for every species.
[Koch et al., Nature (2004), DOI10.1038/nature02417, author institution record](https://experts.azregents.edu/en/publications/the-limits-to-tree-height/).

Large old trees can remain productive. Stephenson et al.'s403-species analysis
found that absolute mass growth increased with size in most studied species.
This is not evidence that height or diameter increments accelerate forever in
one individual. Size and age are different variables.
[Stephenson et al., Nature (2014), DOI10.1038/nature12914](https://www.nature.com/articles/nature12914).

A study of old ginkgo trees found retained leaf photosynthetic performance and
reproductive competence across ages up to600years; this supports separating
chronological age from a compulsory senescence cutoff.
[Wang et al., PNAS (2020), DOI10.1073/pnas.1916548117](https://pmc.ncbi.nlm.nih.gov/articles/PMC6995005/).

There is no universal birthday or trunk diameter at which all woody growth
stops. Species/site-specific reference sizes and slowing rates are appropriate
model parameters. A rigid diameter ceiling is an authored approximation, not a
botanical law. Palms need their own anatomical model; these cambial-growth
conclusions should not be copied indiscriminately to them.

## Confirmed source limitations

`src/botany/growth/tree.rs` caps trunk height using `maximum_nodes` and an age
clamp. With the current ten-node,365-day exemplar, trunk height stops increasing
at10years. The trunk-radius smoothstep reaches a hard plateau at18years.
Later branch/leaf expansion can still change the crown after those dates.

`src/botany/growth/stature.rs` clamps age/maturity to0..1 and reaches the adult
diameter exactly at maturity. It also scales all woody paths proportionally,
so this trait alone cannot represent continued radial growth with nearly
constant height. Its diameter refers to the root-axis base, not field DBH.

The early-foliage profile currently places24 leaves on each new primary branch,
including at its initial length of approximately0.25m. Insertions span only80%
of that length: about8.7mm between nodes. Leaves reach0.25m length and0.085m
width in30days while branch extension uses a four-year smoothstep. This is a
concrete source-level crowding mechanism. The user's particular offending frame
has not been reproduced at an exact timestamp; no renderer fault is established.

## Proposed mathematical revision (not implemented)

Use independent curves for height and woody cross-sectional area, with maturity
as a reference timescale rather than a hard stopping time. For fixed-site
procedural authoring, one possible bounded family is:

```
H(a) = H0 + (Hcap - H0) * (1 - exp(-kH*a))^pH
A(a) = A0 + (Acap - A0) * (1 - exp(-kA*a))^pA
D(a) = 2 * sqrt(A(a) / pi)
```

Here age is in years, H and D are metres, A is square metres, k>0 and p>=1.
Hcap/Acap are optional authored effective limits for a species/site profile,
not measured universal constants. Finite-age rates decline smoothly instead of
being set to zero at one maturity birthday. Numerical precision can still make
very small late increments unresolvable. The family is an engineering proposal,
not a fitted biological law or a full carbon-allocation model.

Area and diameter are distinct: A=pi*D^2/4 implies dD/da=2*(dA/da)/(pi*D).
The same area increment makes a thinner radial layer on a larger trunk. Mass,
area, diameter and height increments must therefore be reported separately.

For foliage, retain several initial leaves but create additional leaf-bearing
nodes as a shoot extends. Use actual arc-length internode spacing, a bud/leaf
emergence schedule and rapid individual leaf expansion. Do not place an adult
axis's entire leaf count on its newborn quarter-metre shoot. Woody thickening
and seasonal shoot extension need separate controls. Leaf/branch IDs must stay
stable as new nodes appear; do not redistribute old leaves arbitrarily to fix
crowding or erase the seedling's foliage.

Mature crowns should continue producing/retiring shoots and leaf cohorts even
when external size changes slowly. A topological or geometry budget is separate
from a biological size limit. With variable environmental history, chronological
age alone is insufficient: integrated growth state or event history is needed.
Reversible age preview remains a fixed-definition authoring comparison, not
reconstruction of water, fire, pruning or harvest history.

Before implementation: preserve old definitions/derived sources explicitly;
verify late-age increments, independent H/D behavior, young-shoot spacing,
intervention persistence and back-and-forth editor viewport equality. Keep
biological calibration, visual acceptance and population performance separate.
