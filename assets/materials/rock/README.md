# Editable mountain-rock materials

Four native source presets: Alpine, Fractured, Ochre and Sedimentary. These are
configurable surface programs, not sampled photographs or geological simulation.

Load a `.ignis-material` onto a mesh through the material library or `material/load`.
The source supports block/joint density and direction, face planes, strata,
weathering, mineral inclusions and grain. Retain seeded runtime evaluation or
explicitly bake fixed maps up to 1024. Rock presets use the opt-in per-pixel relief
normal mode; existing materials keep their previous mode.

See [the contract, controls and native review workflow](../../../docs/rock-materials.md).
