# Birch reference analysis

This is an original procedural interpretation of the birch in Genshin Impact,
requested as the first species in a sequential collection, with at least three
variants. No game geometry, texture, leaf atlas or screenshot is an output asset.
Reference images remain local under `artifacts/genshin-birch/references`.

## Evidence and initial inventory

- R1: [in-game Collecting Wood screen](https://static.fanbyte.com/uploads/2021/04/Genshin-Impact-Birch-Wood.png),
  813×743, front-oblique view, tree visible approximately x275..519/y18..388.
- R2: [in-game harvesting view](https://static.actugaming.net/media/2021/04/arbres-genshin-impact-5-1536x864.jpg),
  1536×864, close trunk plus neighboring examples. Character/UI and sword flare
  obscure the near trunk. Other trees, grass, rock, sky and UI are context only.
- The requested [Wood index](https://genshin-impact.fandom.com/wiki/Wood)
  could not be read because of the site's access restriction.

The dominant object is a tall, narrow deciduous crown, broadest in the lower
middle and tapering irregularly to a leafy point. It has a short visible pale
trunk beneath a dense crown. Lobes overlap rather than forming a smooth cone;
small sky holes occur between peripheral sprays. R2 reveals pale bark with
horizontal dark scars, smaller flecks, pale gray patches and thin ascending
branches. Leaves are pointed ovate silhouettes. Counts of hidden leaves,
branch hierarchy, bark thickness, rear silhouette and dimensions are unknown.

## Inspected crops and second reading

Coordinates use original image pixels, upper-left origin, x right/y down;
boxes below are x/y/width/height. Local crops were made and opened separately.

| Card | Crop | Findings and consequence |
|---|---|---|
| C1 | R1 (260,16,275,339), `crown.png` | Approximately six overlapping height bands, not six identical rings. Small terminal sprays interrupt the pointed silhouette. Lower side masses are irregular; crown is not spherical. Use staggered 3D lobes with decreasing reach and irregular height/azimuth. Exact branch count remains hidden. |
| C2 | R2 (610,45,153,650), `bark.png` | Broad dark olive/brown transverse scars mix with much thinner short marks. Ends are irregular, some scars almost wrap around the visible cylinder. Pale gray broad patches interrupt the off-white base. Use anisotropic procedural masks at two scales; do not put dark raised branches across the trunk. Root flare is obscured, so avoid invented buttress roots. |
| C3 | R2 (880,0,280,160), `leaf.png` | Greenish shadow leaves have pointed broad oval profiles and varied orientation, with small open gaps and thin pale twigs. Golden color is lighting/foliage-dependent, not a different branch topology. Use broad tapered blades with shallow fold and varied roll; no resolved serration/vein count can be inferred. |

Revisited R1 and R2 after the crops: pale branches continue into golden masses,
with shaded ochre/olive interiors and bright yellow tips. A regular Christmas
tree arrangement would be a misreading of the height bands. The close R2 foliage
does not establish a precise botanical species, season or metric size. No
finer crop supplies missing branch connections or leaf-vein detail.

## Consolidated construction list, before geometry

All items below were **not built / not validated** at this analysis checkpoint.
B is the tree grouping; executable items have independent review criteria.

| ID / parent | Construct, count and relation | Evidence / epistemic status | Check |
|---|---|---|---|
| B1 / B | One slightly irregular upright main stem, tapered circular/oval section, supporting the crown | R1 lower center, R2 trunks; observed axis, circular rear section hypothesized | Short visible bare stem, continuous upright body in front/side/back |
| B2 / B1 | Ascending primary branches at staggered heights/3D azimuths, thinner than stem, connected into it | C1/C3; branch network observed, exact graph hidden | No radial flat fan, no disconnected limbs; inspect bare structure |
| B3 / B2 | Secondary forks and thin terminal twigs feeding each spray; varying curvature and taper | C3; fine pale lines observed, hidden connectivity hypothesized | Real multilevel parent graph; leaves anchored to twig paths |
| B4 / B | Layered elongated crown, widest below middle, uneven sides, narrow leafy apex | C1 observed | Tall crown/width ratio, asymmetrical lobes and pointed top in multiple views |
| B5 / B4 | Overlapping foliage sprays distributed around each lobe, with small irregular voids | C1/C3 observed family, total count unknown | Dense interior without featureless solid proxy or equal horizontal rings |
| B6 / B3 | Pointed ovate blades, broad middle/base, narrowing tip, small bend/fold, varied roll | C3 observed outline; thickness/underside hypothetical | Close render reads individual leaves rather than needles or round discs |
| B7 / B1 | Warm off-white bark, soft gray/cream patches | C2 observed appearance | Trunk stays pale with visible patch scale, not uniform gray |
| B8 / B7 | Wide irregular horizontal dark olive scars, narrower short marks, sparse flecks | C2 observed, around-trunk continuation hypothetical | Transverse marks at multiple widths, no uniform zebra bands |
| B9 / B5 | Gold/yellow leaf masses, ochre shaded interiors and slightly different leaf tones | C1, R2 observed | Gold remains legible without emission or baked screenshot colors |
| B10 / B1 | Modest basal widening at ground contact, no giant walls/aerial roots | R1/R2 ground obscured; conservative hypothesis | Supported contact, no floating stump, no unsupported buttress claim |
| B11 / B | Three seeded versions: reference-like tall form, broader/shorter, narrower/slightly leaning | User requirement; unobserved variants proposed | Different geometry/branch counts or headings, not recolored copies |

## Construction hypothesis and tools

Use +Y up, origin at ground contact; dimensions in meters are authored game
scale, not measurements of the reference. Crown volume is an elongated,
height-dependent lobed envelope. Curves use the existing botanical Builder and
native spline compiler, with native grafts/forks and closed shallow blades.
Sources remain editable `.ignis-botany`; seed/profile/parameters precede meshes.
Native procedural materials synthesize the bark and foliage, without external
textures or species-specific renderer changes. Inspect the live MCP schema
before invoking publication/material commands.

First review gray multi-view structure and volume; then bark/foliage. Compare
with C1/C2/C3 and whole references again. Source reproduction, solid topology,
save/load and rendering tests establish separate technical gates. Manual
camera/scale choices and different lighting prevent a pixel-fidelity score.
Hidden geometry and exact Genshin shading remain hypotheses; a native render
does not establish an exact replica or runtime performance acceptance.
