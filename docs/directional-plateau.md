# Directional plateau experiment (EAF-221)

User observation: previous united geometry still reads like assembled blocks.
New requirement is a common formation history, expressed through directions.
This is an artistic geometric hypothesis, not a tectonic simulation.

Construction list before authoring: D1 complete lobed solid; D2 persistent bedding
normal [0.34,1,0.08] normalized; D3 thirteen unequal strata, with continuous
boundaries and shared angular recesses; D4 local correlated offsets and resistance;
D5 horizontal truncation exposing inclined layers below flat summit; D6 three
finite steep joint cuts in one family; D7 secondary corner chips; D8 neutral and
material overview/detail/rear/top review, save/load and replay.

Existing native Landform2 compiles variable sections in a common tilted frame.
Native SubtractConvex trims summit/base and opens finite joints. Python supplies
section/convex controls, never vertex/triangle topology. No new runtime operator.
Before-code performance checklist: bounded 27 sections/20 controls, 31 CSG
proposals, cold authoring only; baked mesh on existing render/collider paths.
No new per-frame allocations, streaming, dependencies, format or GPU passes.
Conservative CSG guards and atomic rejections retained. Source/accepted operations
stored as an editable recipe; this does not add a runtime geology simulator.

## Native result and review

Current binary reported 91bc9ac8bafe7c09 (parallel botany build, no engine rebuild
or Rust change by this experiment). Native MCP authored 31 accepted cuts:
2 truncations, 3 finite joints, 12 edge chips, 14 smaller chips. Final rock is
679 vertices/1354 triangles, zero boundary/nonmanifold edges and one island.
80 roof triangles receive a separate turf skin. Save/load capture byte equality
and replay+mesh agreement pass. Python compile and diff checks pass.

An initial 40-section/28-profile-control version was refused by conservative
self-intersection/contact validation before truncation. The recipe was simplified
to 27 sections/20 controls; guards were not changed. This is a demonstrated
accepted recipe, not proof that every dense section configuration is CSG-safe.

Neutral/overview/detail/rear captures were inspected. Compared with the earlier
assembled-cell experiment, continuous inclined bands now cross the walls and
are interrupted by steep joints, two of which end below the summit. Local
radial changes avoid identical retreat around every layer. Large planes remain
very clean and ledges/rifts still look schematic and regular; this is a structural
study with visible bedding, not reference fidelity or artistic acceptance.
Material coordinates follow the same bedding normal. No physical tectonics,
erosion, hydrology, world integration, new rendering cost or FPS claim.

Recipe examples/directional-plateau.recipe.json retains native source and accepted
cuts; scripts/directional_plateau.py authors section/convex controls and
scripts/render_directional_plateau.py runs through MCP. Evidence and editable
scene under artifacts/directional-plateau/. Earlier weathered-plateau preserved;
no new engine build copies, foreground window or changes to parallel botany/LoS.
EAF-221 remains In Progress pending art acceptance and broader integration.
