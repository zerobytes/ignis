# Lawn reference analysis before construction

Scope: create procedural emerald and Japanese lawn assets from the user's paired
reference, extending the approved botanical libraries. The photo compares two
different gardens/cameras. It is not a calibrated single-scene reconstruction.

Source: `codex-clipboard-d9704719-2508-4b13-bcc6-7a3377770bd1.png`, 1618×910,
upright. Exact local path, SHA256 and extraction boxes are recorded in
`artifacts/lawns/reference/identity.json`. Crops use original pixels without
resizing; boxes below are left/top/right/bottom, origin at the upper left.

## Global organization and hierarchy

The left panel is labelled grama-esmeralda / Zoysia japonica: a dense, relatively
even green carpet surrounds diagonally arranged exposed paving stones. The
right panel is labelled grama-japonesa / Zoysia tenuifolia: several overlapping
rounded cushions have dark troughs and a fine bristly surface. Both views are
oblique from above. Leaves overlap and hide their bases; neither leaf counts nor
underground connections are recoverable. Metric scale and camera calibration
are unknown. The written labels are supplied reference information, not a new
species identification from pixels.

Hierarchy: paired illustration → separate lawn surfaces → repeated leaf groups
or cushions → pointed individual blades → base/tip shading and occlusion.
Leaves cover the ground and overlap their neighbours. Japanese cushions rise
above adjoining troughs. Left paving is context showing the lawn edge; creating
paving assets, text labels, white frames or the illustration background is outside
the user's grass request. No flowers or seed heads are visibly required.

## Inspected local evidence

All five actual crops were created and opened before generating curves.

| Evidence | Box | Local reading and resulting decision |
| --- | --- | --- |
| E0 context |93,222,728,805|Continuous lawn, mostly upright pointed leaves; brighter broad surface, darker bent leaves at paving edges. Build a low carpet rather than tall wild-grass sprays.|
| E1 detail |265,400,540,645|Individual blades are wider and more separately legible than in J1. Leaves lean locally and overlap, with variation in tip height. Avoid identical vertical sticks or a perfectly flat cut plane.|
| J0 context |905,223,1548,805|Rounded cushions of varied apparent size and orientation fill the panel. Dark troughs occur between them. Build height variation in leaves across each cushion, with organic variation between cushions.|
| J1 detail |1100,375,1435,666|Fine blades form a rounded skyline; upper blades are more upright, perimeter blades open outward. The centre/rim difference must affect actual paths and silhouette.|
| J2 subdetail |1210,445,1365,575|Confirms the bristly rim and dark layered base. Fine nerves, hairs, exact blade thickness and connections are unresolved at this resolution. Stop refinement at that evidence limit.|

The local review confirms the global distinction and adds a construction rule:
Japanese leaf height and lean must vary with radial position, not only with the
overall scale of a generic tuft. A second review of the complete supplied image
confirms that regular spherical solid objects would misrepresent both lawns.

## Consolidated construction list

This list is delivered before geometry generation. At this checkpoint every
construction item is **not built**, and model comparison is **not performed**.

| ID / parent | Executable feature | Evidence / epistemic state | Rule and relations | Check |
| --- | --- | --- | --- | --- |
| E / lawns | Emerald lawn family |User requirement; supplied left label|Group for E1–E4; not an independent mesh|All four features covered|
| E1 / E |Wider, pointed blades with modest folding|E0/E1 observed; metric width authored|Curved blades, approximately2–4mm working width; taper to pointed tips|Close blade silhouette and width relative to Japanese|
| E2 / E |Low, relatively regular carpet|E0 observed; metric height authored|Many blade origins distributed over a reusable square module; short upright/arched paths with small height variation|No wild-grass fountain, bare grid or isolated large mounds|
| E3 / E |Local lean and overlapping tips|E1 observed|Azimuth, tilt and curvature vary; neighbouring modules join the carpet|Oblique and top views, tile seams|
| E4 / E |Green/yellow-green variation with darker bases|E0/E1 observed appearance; exact material unknown|Correlated mature/fresh green components and actual self-shadow|No single flat green backing plane used as foliage|
| J / lawns |Japanese lawn family|User requirement; supplied right label|Group for J1–J4|All four features covered|
| J1 / J |Very fine, pointed needle-like blades|J1/J2 observed; metric width authored|Narrow folded blades, working width below1mm; no flat broad straps|Close view compared to E1|
| J2 / J |Rounded cushion silhouette|J0/J1 observed; metric height authored|Radial height envelope, upright centre and outward leaning lower rim; no opaque mound hidden inside|Profile, front/back and top views|
| J3 / J |Irregular adjoining cushions and troughs|J0 observed; counts unknown through overlap|Seeded size/height/offset/rotation variation; gaps small enough to preserve dense coverage|Carpet view without an exact egg-carton grid|
| J4 / J |Deep green fine surface with lighter growth|J0/J1 observed appearance; exact BRDF unknown|Two green populations, surface orientation and self-shadow; retain pointed detail|Oblique and close views|
| C1 / lawns |Ground attachment and dense overlap|E0/J0 observed; bases occluded|Authored roots below the support surface; visible geometry carries the density|Root transforms and views with ground|
| C2 / lawns |Reusable editable variants|User procedural-system context|Three seeded variants per family; canonical blade curves/width/thickness and patch layout|Regeneration and different-seed checks|

Paving/labels/background are recorded context, excluded from the grass asset
scope. Rhizomes, stolons, hairs and internal nerves are not reconstructed from
these pixels. Mechanical parameters remain authoring defaults; no species-level
physical calibration is inferred.

## Proposed editable grammar and available tool

Use native Botany source7 curves, strip outlines and the existing blade compiler.
Author metre units explicitly; these are chosen garden working dimensions,
not measurements extracted from the photograph. Expose blade width/height,
count, spread/fold, cushion radius/height envelope and seeded variation.
Closed blade skins are the existing compiler representation; a cushion consists
of many leaves rather than one hidden closed dome. Publish/load through the
current executable's MCP schema, save reusable static instances separately from
editable source curves, and retain source-to-instance IDs.

Supplementary morphology references: [UC IPM](https://ipm.ucanr.edu/TOOLS/TURF/TURFSPECIES/zoysia.html)
describes short pointed blades and the fine dwarf habit; [NC State Extension](https://content.ces.ncsu.edu/zoysiagrass)
documents the2–4mm blade-width range in its species-data presentation. Names
vary by region/cultivar; the requested Portuguese labels and visual distinction
remain the target. These texts do not calibrate the photo's dimensions or colour.

## Planned comparison and acceptance boundary

Capture native individual modules, dense lawns, close details, a profile and a
top view. Compare E and J under the same controlled camera/light, then revisit
the reference crops. Numerical checks cover source dimensions, radial profiles,
cost, topology and persistence. The two source photos have different cameras,
ground layouts and lighting; no pixel-equality or RGB similarity score can
certify their shared botanical habit. A normalized side-by-side reference/render
sheet is a visual diagnostic only. Native operation, geometric validity, visual
correspondence and human artistic acceptance remain separate.


## Constructed result and reference revisit

All E1–E4, J1–J4 and C1–C2 construction rules are implemented in
`scripts/lawn_assets.py`; each family has three canonical variants. Component
IDs use `ln_{family}_{variant}_{green|fresh}`; static object IDs use
`{family}_{x}_{z}`. `artifacts/lawns/placements.json` maps every patch object to
its variant, pose and scale. The native schema was read before publication.

| Feature IDs | Native evidence | Review finding / remaining difference |
| --- | --- | --- |
| E1, E3 |`esmeralda_close.png`, `esmeralda_close-preview.png`|Wider pointed folded leaves and individually varied lean are visible. The initial sparse version was replaced with768 blades per24cm module.|
| E2 |`comparison.png`, `top.png`, `profile.png`|Low carpet with edge-to-edge square coverage. Complete stratified rows avoid a systematic empty strip; coarse variation between the three modules remains visible.|
| E4 |`esmeralda.png`, close view|Mature/fresh populations separate colour subtly. Controlled lighting is brighter than the first trial; no measured photographic colour match is claimed.|
| J1 |`japonesa_close.png`, `japonesa_close-preview.png`|Very fine pointed leaves remain distinct from E1; subpixel raster aliasing remains in raw images.|
| J2 |`variants.png`, `comparison.png`, close/profile|Real paths form raised centres and opening rims. Mean centre/rim tip heights are approximately85–89mm /48–51mm before instance placement; there is no hidden supporting dome.|
| J3 |`top.png`, `japonesa.png`|Overlapping cushions and troughs are visible after density/radius increases. Seeded offsets up to25mm reduce alignment, but the7×7 review layout remains more regular than the reference garden.|
| J4 |`japonesa_close.png`, family view|Fine deep-green surface and lighter tips/populations are present; contrast and trough darkness differ from the reference.|
| C1 |`source-validation.json`, close/profile|Every authored patch root is below the support surface after scale and height offsets; tested numerically for all98 modules.|
| C2 |`variants.png`, canonical sources, `source-validation.json`|Three variants each, exact same-seed regeneration and all sources changed with a different seed. Source IDs are unique within each variant.|

The final image and the Japanese context crop were revisited after these changes.
Correspondence is at the habit/leaf-shape level; the reference still has more
organic merging between cushions, stronger local lighting and finer photographic
texture. Native validity and the chosen working dimensions do not certify a
photo replica or species-level calibration. Human artistic acceptance is pending.

The first light setup produced shadow acne even on the flat support. Increasing
the existing shadow bias from0.00008 to0.001 removed the visible floor pattern;
no renderer code changed. Raw captures remain available. Preview files explicitly
use native2048×1252 rendering downsampled to1024×626, not a runtime AA feature.
The full native library passes directed-edge closure/orientation, and the profile
PNG is byte-identical after native save/load. See [lawn-assets.md](../../lawn-assets.md)
for reproduction, exact geometry costs and the large-area performance boundary.


## Follow-up: a fine Japanese carpet without cushions

The user questioned whether the mounds represented the ground and requested a
third option with the same fine leaves but no mounds. This is an explicit design
variant, not a new observation from the photograph. Clemson Extension's
[species description](https://hgic.clemson.edu/factsheet/zoysiagrass/) confirms
that Z.tenuifolia itself can form small puffy clumps. The photo does not establish
the terrain hidden below its leaves; it does not require a shrub or raised soil.

New family `japonesa_tapete`, three variants, shares the Japanese width0.9mm,
fold0.24, thickness0.07mm and material populations. It uses a stratified square,
authored height4cm and short independently oriented paths. No radial envelope or
centre fan is used. Original12 source files remain byte-identical.

| ID / requirement | Construction and evidence | Result |
| --- | --- | --- |
| F1 / fine Japanese leaves |Same cross-section/material controls; `japonesa_tapete_close.png`|Implemented and visually reviewed: finer than emerald, pointed and dense.|
| F2 / flat continuous habit |Uniform square distribution without radial height; `comparison.png`, family view|Implemented: a continuous low carpet, with local blade variation but no rounded cushion skyline.|
| F3 / procedural reusable option |Three additional variants/six native files; canonical regeneration checks|Same seed exact, different seed changes all sources; all previous12 sources unchanged.|
| F4 / native usability |Full comparison/close views, closure and save/load checks|Native meshes closed/oriented; profile PNG byte-exact after reload. No performance or artistic acceptance inferred.|

Centre/outer-annulus mean tip-height ratios in the flat preset are within2% of1,
confirming the absence of a designed radial dome in the shipped variants. The
three-way comparison is ordered emerald / Japanese cushions / Japanese carpet.
The carpet preview retains the explicit still-image supersampling noted above;
raw captures are retained. Ground attachment is checked for all147 placements.
