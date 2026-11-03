use ignis::{
    engine::{Command, Engine},
    materials::{Detail, Material, Plane, Segment},
    meshes::*,
    model::RoomRecipe,
};
use std::sync::Arc;
fn engine() -> Engine {
    Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap()
}
fn edit(e: &mut Engine, edit: Edit) {
    e.execute(Command::EditMesh { edit }.into()).unwrap();
}
fn detail() -> Detail {
    Detail {
        profile: Default::default(),
        repeat: None,
        triplanar: None,
        plane: Plane::Xz,
        tint: [0.4, 0.6, 0.8],
        opacity: 0.7,
        bump: -0.002,
        segments: vec![Segment {
            start: [-1., 0.],
            end: [1., 1.],
            widths: [0.02, 0.004],
        }],
    }
}
fn setup() -> Engine {
    let mut e = engine();
    edit(
        &mut e,
        Edit::PutAsset {
            asset: Asset {
                id: "surface".into(),
                geometry: Arc::new(Geometry {
                    positions: vec![
                        [0., 0., 0.],
                        [1., 0., 0.],
                        [0., 0., 1.],
                        [3., 0., 0.],
                        [5., 0., 0.],
                        [3., 0., 2.],
                        [9., 9., 9.],
                    ],
                    triangles: vec![[0, 2, 1], [3, 5, 4]],
                    material_coordinates: Some(vec![
                        [10., 1., 10.],
                        [12., 1., 10.],
                        [10., 1., 14.],
                        [20., 2., 20.],
                        [24., 2., 20.],
                        [20., 2., 28.],
                        [99., 99., 99.],
                    ]),
                }),
                shading: Shading::Smooth,
                material: Default::default(),
                rig: None,
                clips: vec![],
            },
        },
    );
    e
}
#[test]
fn bounded_detail_rejects_bad_fields_without_runtime_mutation() {
    let mut e = setup();
    e.execute(Command::Step { ticks: 7 }.into()).unwrap();
    for case in 0..10 {
        let mut d = detail();
        match case {
            0 => d.opacity = f32::NAN,
            1 => d.opacity = 1.1,
            2 => d.bump = -0.051,
            3 => d.tint[0] = -0.1,
            4 => d.segments.clear(),
            5 => d.segments = vec![d.segments[0]; 33],
            6 => d.segments[0].end = d.segments[0].start,
            7 => d.segments[0].widths[0] = 0.,
            8 => d.segments[0].start[0] = f32::INFINITY,
            _ => d.segments[0].widths[1] = 11.,
        }
        let before = e.snapshot();
        let scene = e.scene.clone();
        assert!(
            e.execute(
                Command::EditMesh {
                    edit: Edit::SetMaterial {
                        asset: "surface".into(),
                        material: Material {
                            detail: Some(Arc::new(d)),
                            ..Default::default()
                        }
                    }
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
        assert_eq!(e.scene, scene);
    }
}
#[test]
fn per_island_fit_uses_authored_coordinates_and_keeps_geometry_and_history() {
    let mut e = setup();
    let original = e.scene.clone();
    let fit = Edit::FitMaterialCoordinates {
        asset: "surface".into(),
        per_island: true,
        bounds: [[-1., 0., 0.], [1., 0., 1.]],
    };
    let preview = e
        .execute(Command::PreviewMeshEdit { edit: fit.clone() }.into())
        .unwrap();
    assert_eq!(e.scene, original);
    edit(&mut e, fit);
    let a = &e.scene.meshes.assets[0];
    assert_eq!(
        a.geometry.positions,
        original.meshes.assets[0].geometry.positions
    );
    assert_eq!(
        a.geometry.triangles,
        original.meshes.assets[0].geometry.triangles
    );
    let c = a.geometry.material_coordinates.as_ref().unwrap().clone();
    assert_eq!(&c[..3], &c[3..6]);
    assert_eq!(c[6], [0., 0., 0.5]);
    assert_eq!(&c[..3], &[[-1., 0., 0.], [1., 0., 0.], [-1., 0., 1.]]);
    assert_eq!(
        preview["meshes"],
        serde_json::to_value(&e.scene.meshes).unwrap()
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(
        &e.scene.meshes.assets[0].geometry.material_coordinates,
        &Some(c.clone())
    );
}
#[test]
fn asset_fit_and_invalid_bounds_are_atomic_and_constant_axes_use_midpoint() {
    let mut e = setup();
    for bounds in [
        [[1.; 3], [0.; 3]],
        [[0.; 3], [f32::NAN, 1., 1.]],
        [[0.; 3], [10001., 1., 1.]],
    ] {
        let before = e.snapshot();
        assert!(
            e.execute(
                Command::EditMesh {
                    edit: Edit::FitMaterialCoordinates {
                        asset: "surface".into(),
                        per_island: false,
                        bounds
                    }
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    edit(
        &mut e,
        Edit::SetMaterialCoordinates {
            asset: "surface".into(),
            coordinates: None,
        },
    );
    edit(
        &mut e,
        Edit::FitMaterialCoordinates {
            asset: "surface".into(),
            per_island: false,
            bounds: [[0.; 3], [1.; 3]],
        },
    );
    let c = e.scene.meshes.assets[0]
        .geometry
        .material_coordinates
        .as_ref()
        .unwrap();
    assert_eq!(c[0], [0.; 3]);
    assert_eq!(c[6], [1.; 3]);
}
#[test]
fn shared_detail_is_preserved_by_subdivision_extraction_and_replay() {
    let mut e = setup();
    let d = Arc::new(detail());
    edit(
        &mut e,
        Edit::SetMaterial {
            asset: "surface".into(),
            material: Material {
                detail: Some(d.clone()),
                ..Default::default()
            },
        },
    );
    let t = e
        .execute(
            Command::InspectMeshIslands {
                asset: "surface".into(),
            }
            .into(),
        )
        .unwrap();
    edit(
        &mut e,
        Edit::ExtractIslands {
            asset: "surface".into(),
            id: "part".into(),
            reference: serde_json::from_value(t["reference"].clone()).unwrap(),
            islands: vec![1],
            mode: ExtractionMode::Copy,
        },
    );
    assert!(Arc::ptr_eq(
        e.scene.meshes.assets[1].material.detail.as_ref().unwrap(),
        &d
    ));
    let t = e
        .execute(
            Command::InspectMeshTopology {
                asset: "surface".into(),
            }
            .into(),
        )
        .unwrap();
    edit(
        &mut e,
        Edit::Subdivide {
            asset: "surface".into(),
            reference: serde_json::from_value(t["reference"].clone()).unwrap(),
            components: None,
        },
    );
    assert!(Arc::ptr_eq(
        e.scene.meshes.assets[0].material.detail.as_ref().unwrap(),
        &d
    ));
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(
        Engine::replay_at(
            &serde_json::from_slice(&serde_json::to_vec(&replay).unwrap()).unwrap(),
            None
        )
        .unwrap()
        .scene,
        e.scene
    );
}
#[test]
fn detail_metadata_preserves_linked_source_and_old_format_rejects_it() {
    use ignis::{mesh_preservation::DisplacementPolicy, plants};
    let mut e = engine();
    let d = plants::Definition {
        trajectory: None,
        id: "surface".into(),
        profile: plants::Rectangle {
            id: "profile".into(),
            width: 2.,
            height: 3.,
        },
        depth: 0.3,
        sections: None,
        segments: [1, 1, 1],
        subdivision_levels: 0,
        sampling_tolerance: 0.001,
    };
    e.execute(
        Command::EditPlant {
            edit: plants::Edit::Put {
                definition: d,
                detail_policy: DisplacementPolicy::Physical,
                animation_translation_policy: DisplacementPolicy::Physical,
            },
        }
        .into(),
    )
    .unwrap();
    let before = e.scene.plants.entries[0].clone();
    let g = e.scene.meshes.assets[0].geometry.clone();
    edit(
        &mut e,
        Edit::SetMaterial {
            asset: "surface".into(),
            material: Material {
                detail: Some(Arc::new(detail())),
                ..Default::default()
            },
        },
    );
    let after = &e.scene.plants.entries[0];
    assert_eq!(before.source.geometry, after.source.geometry);
    assert_eq!(before.reference, after.reference);
    assert_eq!(g, e.scene.meshes.assets[0].geometry);
    let mut old = e.scene.clone();
    old.version = 58;
    assert!(old.validate().is_err());
    old.meshes.assets[0].material.detail = None;
    Arc::make_mut(&mut old.plants.entries[0].source)
        .material
        .detail = None;
    assert!(Engine::new(old).is_ok());
    edit(
        &mut e,
        Edit::FitMaterialCoordinates {
            asset: "surface".into(),
            per_island: false,
            bounds: [[0.; 3], [1.; 3]],
        },
    );
    assert_eq!(
        e.scene.plants.entries[0].source.geometry,
        e.scene.meshes.assets[0].geometry
    );
}

#[test]
fn template_copy_uses_explicit_topology_correspondence_and_rejects_mismatches() {
    let mut e = setup();
    let mut template = e.scene.meshes.assets[0].clone();
    template.id = "template".into();
    template.geometry = Arc::new(Geometry {
        positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 0., 1.]],
        triangles: vec![[0, 2, 1]],
        material_coordinates: Some(vec![[-1., 0., 0.], [1., 0., 0.], [-1., 0., 1.]]),
    });
    edit(
        &mut e,
        Edit::PutAsset {
            asset: template.clone(),
        },
    );
    let original = e.scene.meshes.assets[0].clone();
    let mut target = original.clone();
    Arc::make_mut(&mut target.geometry).positions.pop();
    Arc::make_mut(&mut target.geometry)
        .material_coordinates
        .as_mut()
        .unwrap()
        .pop();
    edit(
        &mut e,
        Edit::PutAsset {
            asset: target.clone(),
        },
    );
    let copy = Edit::CopyMaterialCoordinates {
        asset: "surface".into(),
        source: "template".into(),
        per_island: true,
    };
    let predicted = e
        .execute(Command::PreviewMeshEdit { edit: copy.clone() }.into())
        .unwrap();
    edit(&mut e, copy.clone());
    assert_eq!(
        e.scene.meshes.assets[0]
            .geometry
            .material_coordinates
            .as_ref()
            .unwrap(),
        &template
            .geometry
            .material_coordinates
            .as_ref()
            .unwrap()
            .repeat(2)
    );
    assert_eq!(
        e.scene.meshes.assets[0].geometry.positions,
        target.geometry.positions
    );
    assert_eq!(
        predicted["meshes"],
        serde_json::to_value(&e.scene.meshes).unwrap()
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene.meshes.assets[0], target);
    for wrong in 0..3 {
        let mut bad = template.clone();
        match wrong {
            0 => Arc::make_mut(&mut bad.geometry).triangles[0].swap(1, 2),
            1 => Arc::make_mut(&mut bad.geometry).material_coordinates = None,
            _ => {
                edit(
                    &mut e,
                    Edit::PutAsset {
                        asset: original.clone(),
                    },
                );
            }
        }
        edit(&mut e, Edit::PutAsset { asset: bad });
        let before = e.snapshot();
        let scene = e.scene.clone();
        assert!(
            e.execute(Command::EditMesh { edit: copy.clone() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
        assert_eq!(e.scene, scene);
    }
}

#[test]
fn whole_template_copy_preserves_deformed_positions_with_isolated_vertex_slots() {
    let mut e = setup();
    let source = e.scene.meshes.assets[0].clone();
    let mut target = source.clone();
    target.id = "deformed".into();
    let g = Arc::make_mut(&mut target.geometry);
    g.material_coordinates = None;
    for p in &mut g.positions {
        p[0] = -p[0];
        p[1] += 2.;
    }
    let positions = target.geometry.positions.clone();
    edit(&mut e, Edit::PutAsset { asset: target });
    edit(
        &mut e,
        Edit::CopyMaterialCoordinates {
            asset: "deformed".into(),
            source: "surface".into(),
            per_island: false,
        },
    );
    let a = &e.scene.meshes.assets[1];
    assert_eq!(a.geometry.positions, positions);
    assert_eq!(
        a.geometry.material_coordinates,
        source.geometry.material_coordinates
    );
}
