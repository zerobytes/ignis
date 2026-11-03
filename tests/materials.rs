use ignis::{
    engine::{Command, Engine},
    materials::{Material, Pattern, PatternKind},
    mesh_topology::Plan,
    meshes::*,
    model::RoomRecipe,
};
use std::sync::Arc;

fn pattern() -> Pattern {
    Pattern {
        frequency: [30., 2., 30.],
        low: [0.3; 3],
        high: [1.3; 3],
        bump: 0.005,
        warp: 0.5,
        octaves: 3,
        seed: 42,
        contrast: 1.5,
        kind: PatternKind::Ridged,
    }
}
fn engine() -> Engine {
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    edit(
        &mut e,
        Edit::Grid {
            id: "sheet".into(),
            segments: 2,
            size: 2.,
        },
    );
    edit(
        &mut e,
        Edit::PutInstance {
            instance: Instance {
                id: "one".into(),
                asset: "sheet".into(),
                position: [0., 2., 0.],
                rotation: [0., 0., 0., 1.],
                scale: 1.,
                color: [0.4; 3],
                attachment: None,
                playback: None,
            },
        },
    );
    e
}
fn edit(e: &mut Engine, edit: Edit) {
    e.execute(Command::EditMesh { edit }.into()).unwrap();
}

#[test]
fn invalid_recipes_and_coordinates_are_atomic() {
    let mut e = engine();
    for kind in 0..8 {
        let mut p = pattern();
        match kind {
            0 => p.frequency[0] = f32::NAN,
            1 => p.frequency[1] = 0.,
            2 => p.low[0] = -1.,
            3 => p.high[0] = f32::INFINITY,
            4 => p.bump = 0.1,
            5 => p.warp = -1.,
            6 => p.octaves = 5,
            _ => p.contrast = 0.,
        }
        let before = e.snapshot();
        assert!(
            e.execute(
                Command::EditMesh {
                    edit: Edit::SetMaterial {
                        asset: "sheet".into(),
                        material: Material {
                            surface: None,
                            detail: None,
                            secondary_pattern: None,
                            reflectance: None,
                            pattern: Some(p),
                            transmission: 0.3
                        }
                    }
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    for coordinates in [vec![[0.; 3]; 8], vec![[f32::NAN, 0., 0.]; 9]] {
        let before = e.snapshot();
        assert!(
            e.execute(
                Command::EditMesh {
                    edit: Edit::SetMaterialCoordinates {
                        asset: "sheet".into(),
                        coordinates: Some(coordinates)
                    }
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    assert!(
        Material {
            surface: None,
            detail: None,
            secondary_pattern: None,
            reflectance: None,
            pattern: None,
            transmission: f32::INFINITY
        }
        .validate()
        .is_err()
    );
    assert!(
        Material {
            surface: None,
            detail: None,
            secondary_pattern: None,
            reflectance: None,
            pattern: None,
            transmission: -0.1
        }
        .validate()
        .is_err()
    );
}

#[test]
fn material_setter_preserves_pose_and_shared_geometry_and_undo() {
    let mut e = engine();
    let geometry = e.scene.meshes.assets[0].geometry.clone();
    let pose = e.scene.meshes.pose("one", 0).unwrap();
    let before = e.scene.clone();
    let material = Material {
        surface: None,
        detail: None,
        secondary_pattern: None,
        reflectance: None,
        pattern: Some(pattern()),
        transmission: 0.4,
    };
    edit(
        &mut e,
        Edit::SetMaterial {
            asset: "sheet".into(),
            material: material.clone(),
        },
    );
    assert!(Arc::ptr_eq(&geometry, &e.scene.meshes.assets[0].geometry));
    assert_eq!(pose, e.scene.meshes.pose("one", 0).unwrap());
    assert_eq!(e.render_frame().meshes.materials[1], material);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
}

#[test]
fn explicit_coordinates_stay_on_the_rest_surface_after_sculpt_and_animation() {
    let mut e = engine();
    let coordinates: Vec<_> = e.scene.meshes.assets[0]
        .geometry
        .positions
        .iter()
        .map(|p| [p[0] * 2. + 7., p[1] + 3., p[2] * 3. - 5.])
        .collect();
    edit(
        &mut e,
        Edit::SetMaterialCoordinates {
            asset: "sheet".into(),
            coordinates: Some(coordinates.clone()),
        },
    );
    let before = e.render_frame().meshes.vertices;
    edit(
        &mut e,
        Edit::Sculpt {
            asset: "sheet".into(),
            stroke: Stroke {
                kind: BrushKind::Grab,
                centers: vec![[0.; 3]],
                radius: 2.,
                strength: 0.2,
                direction: [0., 1., 0.],
            },
        },
    );
    edit(
        &mut e,
        Edit::SetRig {
            asset: "sheet".into(),
            rig: Some(Arc::new(Rig {
                bones: vec![Bone {
                    name: "root".into(),
                    parent: None,
                    translation: [0.; 3],
                    rotation: [0., 0., 0., 1.],
                }],
                weights: vec![
                    Weight {
                        extra: None,
                        joints: [0; 4],
                        weights: [1., 0., 0., 0.]
                    };
                    9
                ],
            })),
        },
    );
    edit(
        &mut e,
        Edit::PutClip {
            asset: "sheet".into(),
            clip: Arc::new(Clip {
                id: "move".into(),
                duration_ticks: 60,
                tracks: vec![Track {
                    bone: 0,
                    keys: vec![
                        Keyframe {
                            tick: 0,
                            translation: [0.; 3],
                            rotation: [0., 0., 0., 1.],
                        },
                        Keyframe {
                            tick: 60,
                            translation: [1., 2., 3.],
                            rotation: [0., 0., 0., 1.],
                        },
                    ],
                }],
            }),
        },
    );
    let mut i = e.scene.meshes.instances[0].clone();
    i.playback = Some(Playback {
        clip: "move".into(),
        offset_ticks: 0,
        looping: false,
    });
    edit(&mut e, Edit::PutInstance { instance: i });
    e.execute(Command::Step { ticks: 60 }.into()).unwrap();
    let after = e.render_frame().meshes.vertices;
    assert_ne!(
        before.iter().map(|v| v.position).collect::<Vec<_>>(),
        after.iter().map(|v| v.position).collect::<Vec<_>>()
    );
    assert_eq!(
        before.iter().map(|v| v.rest_position).collect::<Vec<_>>(),
        after.iter().map(|v| v.rest_position).collect::<Vec<_>>()
    );
    assert_eq!(
        e.scene.meshes.assets[0]
            .geometry
            .material_coordinates
            .as_ref()
            .unwrap(),
        &coordinates
    );
}

#[test]
fn subdivision_interpolates_the_channel_and_preserves_recipe() {
    let mut e = engine();
    let coordinates = e.scene.meshes.assets[0]
        .geometry
        .positions
        .iter()
        .map(|p| [p[0] * 2. + 7., p[1] + 3., p[2] * 3. - 5.])
        .collect();
    edit(
        &mut e,
        Edit::SetMaterialCoordinates {
            asset: "sheet".into(),
            coordinates: Some(coordinates),
        },
    );
    edit(
        &mut e,
        Edit::SetMaterial {
            asset: "sheet".into(),
            material: Material {
                surface: None,
                detail: None,
                secondary_pattern: None,
                reflectance: None,
                pattern: Some(pattern()),
                transmission: 0.3,
            },
        },
    );
    let a = &e.scene.meshes.assets[0];
    let divided = Plan::new(&a.geometry, None).unwrap().asset(a).unwrap();
    assert_eq!(a.material, divided.material);
    for (p, c) in divided
        .geometry
        .positions
        .iter()
        .zip(divided.geometry.material_coordinates.as_ref().unwrap())
    {
        assert_eq!(*c, [p[0] * 2. + 7., p[1] + 3., p[2] * 3. - 5.]);
    }
}

#[test]
fn ephemeral_material_slots_are_reindexed_when_an_unused_asset_is_removed() {
    let mut e = engine();
    let mut unused = e.scene.meshes.assets[0].clone();
    unused.id = "unused".into();
    unused.material.transmission = 0.8;
    edit(&mut e, Edit::PutAsset { asset: unused });
    let mut scene = e.scene.clone();
    scene.meshes.assets.swap(0, 1);
    e.replace(scene).unwrap();
    let mut frame = e.render_frame();
    assert!(frame.meshes.vertices.iter().all(|v| v.material == 2));
    edit(
        &mut e,
        Edit::RemoveAsset {
            id: "unused".into(),
        },
    );
    e.update_render_frame(&mut frame);
    assert!(frame.meshes.vertices.iter().all(|v| v.material == 1));
    assert_eq!(frame.meshes.materials.len(), 2);
    assert_eq!(frame.meshes.materials[1], Material::default());
}

#[test]
fn legacy_neutral_material_and_new_channels_cannot_be_smuggled_in_old_versions() {
    let mut e = engine();
    let mut json = serde_json::to_value(&e.scene.meshes.assets[0]).unwrap();
    json.as_object_mut().unwrap().remove("material");
    assert_eq!(
        serde_json::from_value::<Asset>(json).unwrap().material,
        Material::default()
    );
    e.scene.version = 53;
    e.scene.meshes.assets[0].material.transmission = 0.4;
    assert!(
        e.scene
            .validate()
            .unwrap_err()
            .message
            .contains("version54")
    );
    e.scene.meshes.assets[0].material = Material::default();
    Arc::make_mut(&mut e.scene.meshes.assets[0].geometry).material_coordinates =
        Some(vec![[0.; 3]; 9]);
    assert!(
        e.scene
            .validate()
            .unwrap_err()
            .message
            .contains("version54")
    );
    assert_eq!(std::mem::size_of::<ignis::render::Vertex>(), 64);
}
