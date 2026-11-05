# Editable locomotion and contact authoring

Ignis motion libraries (`.ignis-motion`, version 1) contain a reference skeleton,
ordinary native clips, named motions, stride distances, and foot support intervals.
They are separate from the world scene, simulation history and portable saves.
The existing mesh editor/MCP remains the rig and keyframe authoring surface.

Land of Survivors loads
`games/land-of-survivors/assets/animations/locomotion.ignis-motion` for idle, walk
and run. The old draft cycles only seed the file when it does not exist. They
are not a newly approved natural walk. Saving the library reloads the visual
models in a running game session, normally within half a second. Physics,
character identity and the physical save are not replaced. Invalid reloads keep
the last usable models and display an error. Rebuilding a large character can
cause a brief pause; this is authoring reload, not asynchronous streaming.

## Open an authoring scene

```sh
python3 games/land-of-survivors/app.py --build \
  --export-animation-scene games/land-of-survivors/artifacts/app/animation.scene.json
./target/release/ignis --root . \
  --scene games/land-of-survivors/artifacts/app/animation.scene.json --editor --mcp
```

The exported scene contains the canonical game character, with the native body
rig/clips and eyes attached to the head. It uses the ordinary editor animation
playback and mesh operations. It does not open or overwrite a saved character.
For game controls and live reload in a disposable session:

```sh
python3 games/land-of-survivors/app.py --animation-preview
```

This stage does not acquire a character save lease or write a character save.
WASD, Shift and the existing mouse camera controls work as in the game.

## MCP operations

Use `engine_command` with the normal request wrapper. For example:

```json
{"command":{"op":"animation","operation":{"kind":"load","path":"games/land-of-survivors/assets/animations/locomotion.ignis-motion"}}}
```

`inspect` returns `animation_revision` and the document. Supply
`expected_animation_revision` on an animation command to reject stale edits.
Library edits have their own undo/redo and do not reset the scene. `export_clips`
is a scene edit and follows the ordinary scene reset/undo rules.

| Operation kind | Fields / behavior |
| --- | --- |
| `replace` | `document`: validated complete library |
| `put_clip` | `clip`: ordinary native keyframe clip, replace by ID or insert |
| `set_motion` | `motion`: name, clip ID, stride and foot definitions |
| `sample` | `motion`, `phase` (0..1), optional explicit IK `targets`; returns native pose, bone matrices, foot positions/normals/floor gaps and correction-limit count |
| `calibrate_stride` | `motion`: estimate and store stride from forward travel during authored stance intervals |
| `bake_supports` | `motion`, `samples` (4..240): bake straight floor supports into ordinary clip keys |
| `import_asset` | scene mesh `asset`, `motions`: capture its reference bones and edited clips into a library |
| `export_clips` | scene mesh `asset`: retarget library clips into that native rig in one scene transaction |
| `save` / `load` | `path`: atomic save / bounded load, confined to the service root |
| `undo` / `redo` | library history, independent of scene history |

After loading a library, `put_clip` or `set_motion` followed by `save` updates the
game. To see those changes in the authoring scene, also call `export_clips` with
the body asset ID. Conversely, native `edit_mesh` / `put_clip` edits in the scene
can be captured using `import_asset` with the current motion definitions, then
saved. Use normal `put_instance` playback, Play/Pause and Step to select/scrub a
clip, and Capture to inspect real rendered frames. Body/eye attachment stays
native; no alternate animation evaluator is introduced.

## Contact and timing conventions

Stride is reference-rig metres per **full** cycle, not per step. The game advances
moving clip phase from actual horizontal controller displacement divided by the
scaled stride. Idle uses native 60 Hz clip time. Edited durations affect timing
and cross-fades. Visual floor placement uses the rest datum, so vertical motion
is not erased by re-grounding the lowest animated vertex every frame.

A foot specifies three ancestor joint names, a normalized stance interval, a
bend pole, local sole normal, sole offset, and maximum correction distance.
Intervals with start greater than end wrap through phase zero; `[0,1]` means
full-cycle support. Intermediate twist bones are supported; geometry and weights
are not changed. Sample targets are asset-local and cannot move the physics body.
Unreachable limbs do not stretch. Bounded or degenerate corrections are reported
in `pose.ik_limited`; the returned foot positions expose remaining error.

Support baking assumes an in-place cycle advancing along -Z over a flat plane
at `floor_y`, with short contact-boundary blending. Root X/Z travel is rejected;
root Y animation is retained, while non-root retargeting preserves target rest
lengths. Calibration uses the declared support endpoints and is only as useful
as the authored contacts. The starter contact windows are editable draft data.
Bake explicitly: the original draft clips are not silently corrected on load.
A bake can retain support error when a target exceeds limb reach or the correction
bound; inspect the sampled foot positions/floor gaps after baking. It is not a
certificate that a foot remains perfectly planted. Different character proportions
retain native bone lengths; the game scales stride using model height, without
claiming exact contact preservation across different morphologies.

This increment does not provide planted-foot anchors during turns, arbitrary
terrain IK for the player, motion matching, a Blender-equivalent curve editor,
or an approved natural walk. Those require follow-up animation and runtime work.
The generic native NPC root-motion/physics support systems remain separate.

The library inherits native clip/key/rig budgets and adds a 2 MiB document limit,
16 retained undo states and revision guards. Invalid edits are rejected before
publication. No automated tests or new verification harnesses were added under
the temporary project policy.
