# Living sequoia: geometric woody volumes

The optional grammar 7 `wood_volume` profile adds seeded bulges, curved longitudinal cords and negative valleys to the native trunk surface. This is vertex geometry, including its silhouette and shadow. The original grammar 6 source graph, crown and default shape remain available unchanged. The living definition, rather than a detached sculpt, owns the new parameters.

`examples/sequoia-volume.recipe.json` supplies a 1000-year comparison through `python3 scripts/demo_sequoia_volume.py`. Open `artifacts/sequoia-volume/review.html` for matching gray before/after cameras. These gray views use a uniform legacy material with no height map or parallax. The same tree also has a textured scene using the existing sequoia material generator assets.

Run `python3 scripts/open_sequoia_volume.py` to open the gray specimen in the native editor; add `--textured` for foliage and bark. The launcher disables projected shadows for shape inspection; add `--shadows` to retain them. The comparison captures retain shadows: narrow valleys currently expose shadow-map aliasing, also documented by the separate `shadow-off.png` control. This is a remaining renderer appearance limitation. In Vegetação, expand “Volumes reais do tronco”, change the controls and click “Aplicar volumes ao mesh”. The age slider still supports reversible previews. Leaves are hidden in the gray study to expose the trunk.

## Mathematics and persistence

The profile independently controls local bulge amplitude/count, cord height/count/width, valley depth/width, angular meander and axial extent. Amplitudes and widths are ratios of the source stem radius R, evaluated before manual instance scaling. Added displacement uses `max(R−0.05,0)²/R`, suppressing large bark volumes on juvenile stems. This is an authored growth rule, not a calibrated bark-development model.

Bulges have compact support in angle and axial position, with independently seeded centres and lengths. Cord/valley paths interpolate seeded angular controls along the axis, with compact transverse support and fading ends. They use the shared woody-surface evaluator and closed native tube/cap compiler. The new surface uses 32 section stations, 32 contour samples, 96 angular vertices and 128 axial intervals. Fine bark fibres remain material detail; this mesh targets the larger masses.

MCP `botany/living/set_wood_volume` and the editor invoke the same transaction. Applying a profile explicitly upgrades grammar 6 to 7; grammar 7 requires its profile, while grammars 1–6 omit it. Invalid profiles fail before publication. Save/load retains the living definition, generated materials and growth state. No global Scene/replay layout changes were introduced. The generic surface's angular budget now permits 96 sides; existing authored resolutions remain unchanged. A numerical tolerance of 16 machine epsilons handles interpolation at the existing 2 cm radial floor without altering the evaluator.

## Validation and boundaries

The focused Rust checks evaluate juvenile/adult radii, deterministic seeds, exact zero-amplitude baseline, invalid values, the full adult source graph, unchanged crown vertices, and closed/oriented trunk edge incidence with positive signed volume. The native comparison also checks undo/redo, rejected-profile atomicity, age preview back and forth, and gray/textured save-load images.

This initial living binding targets the sequoia stem. The profile mathematics is reusable, but other species and branch/root bindings need explicit integration. Star-shaped sections cannot represent tunnels, overhangs or arbitrary unions. This does not add root anastomosis, a bark fracture model, or prove collision/physics performance. Geometry tests and captures do not replace the user's artistic acceptance.
