# Palm frond refinement

The palmeirinha preset now varies the initial elevation and total turn across
its seven fronds per cane. Inner fronds rise; outer fronds open and recurve.
Curvature increases toward the tip through the existing mathematical frame
integrator. Leaflets are regenerated in the resulting local frames.

References consulted on 2026-11-04:
- https://www.nparks.gov.sg/florafaunaweb/flora/2/6/2622 describes outward and downward arching fronds.
- https://plants.ces.ncsu.edu/plants/chrysalidocarpus-lutescens/ describes arching pinnate leaves and pinnae arranged along the rachis.

This is an authorial approximation, not botanical calibration. Existing saved
sources retain their curves; the preset asset is replaced for future insertion.
No runtime or file-format change is required. Other presets are unchanged.

Validation: Python compilation, native MCP load/bake and full/structure captures
passed. The current renderer produced 4,128 organs, 481,728 vertices and 946,944
triangles. Captures and report use stable paths under artifacts/botany-native/
(palmeirinha.png, palmeirinha-structure.png, palmeirinha-report.json).
Artistic acceptance remains pending. No new binary was built for this recipe edit.
