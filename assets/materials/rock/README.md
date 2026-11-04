# Editable continuous stone materials

Four native rock2 source presets: Alpine, Fractured, Ochre and Sedimentary.
These coat continuous stone with mineral variation, fine grain, sparse hairline
fissures, shallow pits and mild foliation. Large broken planes, deep crevices and
silhouettes belong to the source mesh. The default total relief budgets are
8/10/8/6mm respectively, with smaller actual sampled variation.

Load a `.ignis-material` through the material library or `material/load`.
Retain seeded runtime evaluation or explicitly bake fixed maps up to 1024.
Rock presets use opt-in per-pixel relief normals. Old rock1 files retain their
original evaluation; rock2 requires Scene119/replay129/Content15.

See [controls, compatibility and the native review workflow](../../../docs/rock-materials.md).
