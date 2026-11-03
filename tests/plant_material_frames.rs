use ignis::{
    engine::{Command, Engine},
    mesh_material_finish::{Affine, Options},
    meshes,
    model::Scene,
    plants,
};
use std::{collections::BTreeMap, sync::Arc};
const ID: &str = "branch-editable-1";
fn scene() -> Scene {
    serde_json::from_str(include_str!("../examples/native-tree-study.scene.json")).unwrap()
}
fn affine(offset: [f32; 3]) -> Affine {
    Affine {
        columns: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        offset,
    }
}
fn framed(s: &Scene) -> plants::Assembly {
    let mut a = s.plants.entry(ID).unwrap().assembly.clone().unwrap();
    let g = &mut a.context_grafts[0];
    g.source_material_frame = Some(Arc::new(affine([-0.4706739, 2.8416772, 0.32152966])));
    g.material_finish = Some(Arc::new(Options {
        reference: affine(g.options.material_offset),
        iterations: 64,
        tolerance: 0.002,
        max_change: 5.,
        max_anisotropy: 10000.,
    }));
    a
}
fn edit(a: plants::Assembly) -> Command {
    Command::EditPlant {
        edit: plants::Edit::SetAssembly {
            id: ID.into(),
            assembly: Some(a),
        },
    }
}
#[test]
fn aligned_graft_finishes_final_collar_with_exact_geometry_and_canonical_sources() {
    let s = scene();
    let old = s.plants.entry(ID).unwrap();
    let original = old.clone();
    let base = old.evaluate().unwrap();
    let mut entry = old.clone();
    entry.assembly = Some(framed(&s));
    let output = entry.evaluate().unwrap();
    assert_eq!(*old, original);
    assert_eq!(entry.source, original.source);
    assert_eq!(entry.reference, original.reference);
    assert!(Arc::ptr_eq(
        &entry.assembly.as_ref().unwrap().context,
        &original.assembly.as_ref().unwrap().context
    ));
    assert!(Arc::ptr_eq(
        &entry.assembly.as_ref().unwrap().context_grafts[0].source,
        &original.assembly.as_ref().unwrap().context_grafts[0].source
    ));
    assert_eq!(
        output.asset.geometry.positions,
        base.asset.geometry.positions
    );
    assert_eq!(
        output.asset.geometry.triangles,
        base.asset.geometry.triangles
    );
    assert_eq!(output.asset.rig, base.asset.rig);
    assert_eq!(output.asset.clips, base.asset.clips);
    let report = output.report.assembly.as_ref().unwrap().context_grafts[0]
        .material_finish
        .as_ref()
        .unwrap();
    assert_eq!(report.after.guard.near_singular_faces, 0);
    assert!(
        report
            .after
            .iso
            .as_ref()
            .unwrap()
            .maximum_anisotropy
            .as_ref()
            .unwrap()
            .value
            < 40.
    );
    assert!(
        report
            .after
            .zipper
            .as_ref()
            .unwrap()
            .maximum_anisotropy
            .as_ref()
            .unwrap()
            .value
            < 8.
    );
    // The seven-point child domain is authored before smoothing; every retained
    // child point stays at its exact geometric position and gets precisely that frame.
    let g = &entry.assembly.as_ref().unwrap().context_grafts[0];
    let child = ignis::mesh_bridge::remove_faces(&g.source, &g.source_faces).unwrap();
    let pairs: BTreeMap<_, _> = output
        .asset
        .geometry
        .positions
        .iter()
        .zip(output.asset.geometry.material_coordinates.as_ref().unwrap())
        .map(|(p, m)| (p.map(f32::to_bits), *m))
        .collect();
    for (p, m) in child
        .geometry
        .positions
        .iter()
        .zip(child.geometry.material_coordinates.as_ref().unwrap())
    {
        let pos = std::array::from_fn(|i| p[i] + g.source_translation[i]);
        let expected = std::array::from_fn(|i| {
            (f64::from(m[i]) + f64::from(g.source_material_frame.as_ref().unwrap().offset[i]))
                as f32
        });
        assert_eq!(pairs[&pos.map(f32::to_bits)], expected);
    }
    entry.assembly = original.assembly.clone();
    assert_eq!(entry.evaluate().unwrap().asset, base.asset);
}
#[test]
fn source_frame_is_general_affine_and_independent_of_geometry_and_skin() {
    let s = scene();
    let old = s.plants.entry(ID).unwrap();
    let base = old.evaluate().unwrap();
    let mut e = old.clone();
    let a = e.assembly.as_mut().unwrap();
    a.plant_material_frame = Some(Arc::new(Affine {
        columns: [[2., 0., 0.5], [0.25, 3., 0.], [0., 0., 1.]],
        offset: [2., -1., 3.],
    }));
    let output = e.evaluate().unwrap();
    assert_eq!(
        output.asset.geometry.positions,
        base.asset.geometry.positions
    );
    assert_eq!(
        output.asset.geometry.triangles,
        base.asset.geometry.triangles
    );
    assert_eq!(output.asset.rig, base.asset.rig);
    assert_eq!(output.asset.clips, base.asset.clips);
    assert_ne!(
        output.asset.geometry.material_coordinates,
        base.asset.geometry.material_coordinates
    );
    let report = output.report.assembly.as_ref().unwrap();
    let n = report.retained_context_vertices;
    let count = report.retained_plant_vertices;
    let frame = e
        .assembly
        .as_ref()
        .unwrap()
        .plant_material_frame
        .as_ref()
        .unwrap();
    let before = base.asset.geometry.material_coordinates.as_ref().unwrap();
    let after = output.asset.geometry.material_coordinates.as_ref().unwrap();
    assert_eq!(&before[..n], &after[..n]);
    for i in n..n + count {
        let expected = std::array::from_fn(|k| {
            (f64::from(frame.offset[k])
                + (0..3)
                    .map(|j| f64::from(frame.columns[j][k]) * f64::from(before[i][j]))
                    .sum::<f64>()) as f32
        });
        assert_eq!(after[i], expected);
    }

    assert_eq!(e.source, old.source);
    assert_eq!(e.reference, old.reference);
    // An explicit rank-zero source frame is valid authoring on its own. The
    // regular finisher, when requested, may refuse singular boundary support.
    e.assembly.as_mut().unwrap().plant_material_frame = Some(Arc::new(Affine {
        columns: [[0.; 3]; 3],
        offset: [2.; 3],
    }));
    e.evaluate().unwrap();
}
#[test]
fn native_recipe_preview_undo_version_and_regeneration_are_deterministic() {
    let mut e = Engine::new(scene()).unwrap();
    let initial = e.scene.clone();
    let a = framed(&initial);
    let state = e.snapshot();
    let result = e
        .execute(
            Command::PreviewPlant {
                edit: plants::Edit::SetAssembly {
                    id: ID.into(),
                    assembly: Some(a.clone()),
                },
            }
            .into(),
        )
        .unwrap();
    assert_eq!(state, e.snapshot());
    e.execute(edit(a.clone()).into()).unwrap();
    let after = e.scene.clone();
    let expected: meshes::Asset = serde_json::from_value(
        result["assets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == ID)
            .unwrap()
            .clone(),
    )
    .unwrap();
    assert_eq!(*e.scene.meshes.asset(ID).unwrap(), expected);
    let entry = e.scene.plants.entry(ID).unwrap().clone();
    let mut changed = entry.definition.clone();
    let plants::Trajectory::CubicBezier {
        control_points_xz, ..
    } = changed.trajectory.as_mut().unwrap();
    control_points_xz[3][0] += 0.001;
    for d in [changed, entry.definition.clone()] {
        e.execute(
            Command::EditPlant {
                edit: plants::Edit::Put {
                    definition: d,
                    detail_policy: ignis::mesh_preservation::DisplacementPolicy::Physical,
                    animation_translation_policy:
                        ignis::mesh_preservation::DisplacementPolicy::Physical,
                },
            }
            .into(),
        )
        .unwrap();
    }
    assert_eq!(e.scene, after);
    let mut old = after.clone();
    old.version = 69;
    assert!(old.validate().unwrap_err().message.contains("version70"));
    let roundtrip: Scene = serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
    roundtrip.validate().unwrap();
    assert_eq!(roundtrip, after);
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(Engine::replay_at(&replay, None).unwrap().scene, after);
    e.execute(edit(initial.plants.entry(ID).unwrap().assembly.clone().unwrap()).into())
        .unwrap();
    assert_eq!(e.scene, initial);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, after);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, initial);
}
#[test]
fn malformed_frames_failed_solver_and_singular_source_are_atomic() {
    let mut e = Engine::new(scene()).unwrap();
    for case in 0..5 {
        let mut a = framed(&e.scene);
        let g = &mut a.context_grafts[0];
        match case {
            0 => Arc::make_mut(g.source_material_frame.as_mut().unwrap()).columns[0][0] = f32::NAN,
            1 => Arc::make_mut(g.source_material_frame.as_mut().unwrap()).offset[1] = 10001.,
            2 => Arc::make_mut(g.source_material_frame.as_mut().unwrap()).columns[1][1] = 10000.,
            3 => Arc::make_mut(g.material_finish.as_mut().unwrap()).max_change = 0.000001,
            _ => Arc::make_mut(g.source_material_frame.as_mut().unwrap()).columns = [[0.; 3]; 3],
        }
        let before = e.snapshot();
        assert!(
            e.execute(
                Command::PreviewPlant {
                    edit: plants::Edit::SetAssembly {
                        id: ID.into(),
                        assembly: Some(a.clone())
                    }
                }
                .into()
            )
            .is_err(),
            "case{case}"
        );
        assert_eq!(before, e.snapshot());
        assert!(e.execute(edit(a).into()).is_err(), "case{case}");
        assert_eq!(before, e.snapshot());
    }
}
