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

## Structural revision after user rejection

The first revision was rejected: long straight canes still dominated the plant.
The supplied photographs show short basal canes and longer petiole-to-rachis
arcs. The revised recipe shortens basal canes, lengthens continuous frond axes,
opens older fronds further and leaves the proximal 28% free of leaflets. Leaflet
width is increased. Native full/structure captures passed with the same organ
and mesh counts; total bounds now reach 3.14 m high rather than 5.38 m.
Human artistic acceptance is still pending.

## Correction: retain cane height and bend the proximal frond

User clarified that cane height was acceptable; shortening was the wrong fix.
Restore the original seeded height range and radius. Add mild continuous cane
curvature and move frond curvature toward its base (1.45 to 0.55 multiplier,
previously 0.22 to 1.78). Near-upright emerging fronds now bend immediately into
an outward arch, rather than keeping a straight petiole and curling only at the
leafy tip. Heights remain variable. Sheath attachment remains the existing
separate closed organ connection; this recipe does not introduce a new nested
leaf-sheath surface model.

Native full, bare-structure and isolated-cane captures passed in stable paths.
The isolated image is palmeirinha-isolated.png. Counts remain unchanged; current
height 4.94 m. The two earlier revisions were rejected; no artistic approval is
inferred from captures or compilation.
