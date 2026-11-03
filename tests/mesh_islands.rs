use glam::Quat;
use ignis::{
    engine::{Command, Engine, SceneEdit},
    mesh_topology::Reference,
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
fn asset() -> Asset {
    let positions = vec![
        [0., 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        [3., 0., 0.],
        [4., 0., 0.],
        [3., 1., 0.],
        [0., 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        [9., 9., 9.],
    ];
    Asset {
        id: "source".into(),
        geometry: Arc::new(Geometry {
            material_coordinates: Some(
                (0..positions.len()).map(|i| [i as f32, 0.25, 1.]).collect(),
            ),
            positions,
            triangles: vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]],
        }),
        material: Default::default(),
        shading: Shading::Smooth,
        rig: Some(Arc::new(Rig {
            bones: vec![
                Bone {
                    name: "base".into(),
                    parent: None,
                    translation: [0.; 3],
                    rotation: [0., 0., 0., 1.],
                },
                Bone {
                    name: "joint".into(),
                    parent: Some(0),
                    translation: [0., 1., 0.],
                    rotation: [0., 0., 0., 1.],
                },
            ],
            weights: (0..10)
                .map(|i| Weight {
                    extra: None,
                    joints: [0, 1, 0, 0],
                    weights: [
                        if i % 2 == 0 { 0.25 } else { 0.75 },
                        if i % 2 == 0 { 0.75 } else { 0.25 },
                        0.,
                        0.,
                    ],
                })
                .collect(),
        })),
        clips: vec![Arc::new(Clip {
            id: "bend".into(),
            duration_ticks: 120,
            tracks: vec![Track {
                bone: 1,
                keys: vec![
                    Keyframe {
                        tick: 0,
                        translation: [0., 1., 0.],
                        rotation: [0., 0., 0., 1.],
                    },
                    Keyframe {
                        tick: 60,
                        translation: [0., 1., 0.],
                        rotation: Quat::from_rotation_z(0.5).to_array(),
                    },
                    Keyframe {
                        tick: 120,
                        translation: [0., 1., 0.],
                        rotation: [0., 0., 0., 1.],
                    },
                ],
            }],
        })],
    }
}
fn instance() -> Instance {
    Instance {
        id: "one".into(),
        asset: "source".into(),
        position: [2., 4., 1.],
        rotation: Quat::from_rotation_x(0.3).to_array(),
        scale: 2.,
        color: [0.4; 3],
        playback: Some(Playback {
            clip: "bend".into(),
            offset_ticks: 0,
            looping: true,
        }),
        attachment: None,
    }
}
fn setup() -> Engine {
    let mut e = engine();
    e.execute(
        Command::EditMesh {
            edit: Edit::PutAsset { asset: asset() },
        }
        .into(),
    )
    .unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::PutInstance {
                instance: instance(),
            },
        }
        .into(),
    )
    .unwrap();
    e
}
fn reference(e: &mut Engine) -> Reference {
    serde_json::from_value(
        e.execute(
            Command::InspectMeshIslands {
                asset: "source".into(),
            }
            .into(),
        )
        .unwrap()["reference"]
            .clone(),
    )
    .unwrap()
}
fn extract(reference: Reference, id: &str, islands: Vec<u32>, mode: ExtractionMode) -> Edit {
    Edit::ExtractIslands {
        asset: "source".into(),
        id: id.into(),
        reference,
        islands,
        mode,
    }
}

#[test]
fn islands_use_shared_indices_including_single_vertex_contact_without_spatial_welding() {
    let mut a = asset();
    a.rig = None;
    a.clips.clear();
    a.geometry = Arc::new(Geometry {
        positions: vec![
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [-1., 0., 0.],
            [0., -1., 0.],
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [8., 8., 8.],
        ],
        triangles: vec![[0, 1, 2], [0, 3, 4], [5, 6, 7]],
        material_coordinates: None,
    });
    let d = Document {
        objects: Default::default(),
        assets: vec![a],
        instances: vec![],
    };
    d.validate().unwrap();
    let parts = d.islands("source").unwrap();
    assert_eq!(parts.isolated_vertices, vec![8]);
    assert_eq!(parts.islands.len(), 2);
    assert_eq!(
        (
            parts.islands[0].id,
            parts.islands[0].vertices,
            parts.islands[0].triangles
        ),
        (0, 5, 2)
    );
    assert_eq!(parts.islands[0].bounds, [[-1., -1., 0.], [1., 1., 0.]]);
    assert_eq!((parts.islands[1].id, parts.islands[1].vertices), (2, 3));
    assert_eq!(parts, d.islands("source").unwrap());
}

#[test]
fn move_preserves_exact_channels_weights_bone_indices_clips_and_posed_world_vertices() {
    let mut e = setup();
    let original = e.scene.meshes.assets[0].clone();
    let pose = e.scene.meshes.pose("one", 60).unwrap();
    let r = reference(&mut e);
    let preview = e
        .execute(
            Command::PreviewMeshEdit {
                edit: extract(r.clone(), "separated", vec![1], ExtractionMode::Move),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(e.scene.meshes.assets[0], original);
    e.execute(
        Command::EditMesh {
            edit: extract(r, "separated", vec![1], ExtractionMode::Move),
        }
        .into(),
    )
    .unwrap();
    let source = e.scene.meshes.asset("source").unwrap();
    let taken = e.scene.meshes.asset("separated").unwrap();
    let keep = [0, 1, 2, 6, 7, 8, 9];
    assert_eq!(
        source.geometry.positions,
        keep.map(|i| original.geometry.positions[i])
    );
    assert_eq!(taken.geometry.positions, original.geometry.positions[3..6]);
    assert_eq!(
        taken.geometry.material_coordinates.as_ref().unwrap(),
        &original.geometry.material_coordinates.as_ref().unwrap()[3..6]
    );
    assert_eq!(taken.geometry.triangles, vec![[0, 1, 2]]);
    for a in [source, taken] {
        assert_eq!(a.material, original.material);
        assert_eq!(a.shading, original.shading);
        assert_eq!(
            a.rig.as_ref().unwrap().bones,
            original.rig.as_ref().unwrap().bones
        );
        assert!(Arc::ptr_eq(&a.clips[0], &original.clips[0]));
    }
    assert_eq!(
        taken.rig.as_ref().unwrap().weights,
        original.rig.as_ref().unwrap().weights[3..6]
    );
    assert_eq!(
        preview["meshes"],
        serde_json::to_value(&e.scene.meshes).unwrap()
    );
    assert_eq!(
        e.scene.meshes.pose("one", 60).unwrap().positions,
        keep.map(|i| pose.positions[i])
    );
    let mut i = instance();
    i.id = "taken".into();
    i.asset = "separated".into();
    e.execute(
        Command::EditMesh {
            edit: Edit::PutInstance { instance: i },
        }
        .into(),
    )
    .unwrap();
    assert_eq!(
        e.scene.meshes.pose("taken", 60).unwrap().positions,
        pose.positions[3..6]
    );
}

#[test]
fn copy_keeps_source_exact_and_omits_only_unselected_isolated_vertices() {
    let mut e = setup();
    let scene = e.scene.clone();
    let r = reference(&mut e);
    e.execute(
        Command::EditMesh {
            edit: extract(r, "duplicate", vec![2, 0, 1], ExtractionMode::Copy),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.scene.meshes.assets[0], scene.meshes.assets[0]);
    assert_eq!(e.scene.meshes.instances, scene.meshes.instances);
    let copy = e.scene.meshes.asset("duplicate").unwrap();
    assert_eq!(
        copy.geometry.positions,
        scene.meshes.assets[0].geometry.positions[..9]
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, scene);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene.meshes.assets.len(), 2);
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(replay.scene.version, ignis::model::SCENE_VERSION);
    let restored = Engine::replay_at(
        &serde_json::from_slice(&serde_json::to_vec(&replay).unwrap()).unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(restored.scene, e.scene);
}

#[test]
fn invalid_selection_destination_scope_and_batches_preserve_live_state_and_history() {
    let mut e = setup();
    e.execute(Command::Step { ticks: 7 }.into()).unwrap();
    let r = reference(&mut e);
    let before = e.snapshot();
    let scene = e.scene.clone();
    let mut stale = r.clone();
    stale.revision += 1;
    let mut wrong = r.clone();
    wrong.scope = "elsewhere".into();
    for edit in [
        extract(r.clone(), "new", vec![], ExtractionMode::Move),
        extract(r.clone(), "new", vec![1, 1], ExtractionMode::Move),
        extract(r.clone(), "new", vec![999], ExtractionMode::Copy),
        extract(r.clone(), "source", vec![1], ExtractionMode::Copy),
        extract(r.clone(), "new", vec![0, 1, 2], ExtractionMode::Move),
        extract(stale, "new", vec![1], ExtractionMode::Move),
        extract(wrong, "new", vec![1], ExtractionMode::Copy),
    ] {
        assert!(
            e.execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
                .is_err()
        );
        assert!(e.execute(Command::EditMesh { edit }.into()).is_err());
        assert_eq!(e.scene, scene);
        assert_eq!(e.snapshot(), before);
    }
    for edits in [
        vec![
            Edit::SetShading {
                asset: "source".into(),
                shading: Shading::Flat,
            },
            extract(r.clone(), "new", vec![1], ExtractionMode::Move),
        ],
        vec![
            extract(r.clone(), "new", vec![1], ExtractionMode::Copy),
            Edit::Subdivide {
                asset: "new".into(),
                reference: r.clone(),
                components: None,
            },
        ],
    ] {
        assert!(
            e.execute(
                Command::EditScene {
                    edits: edits
                        .into_iter()
                        .map(|edit| SceneEdit::Mesh { edit })
                        .collect()
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.scene, scene);
        assert_eq!(e.snapshot(), before);
    }
    // Copy does not invalidate the unchanged source inside the same batch.
    e.execute(
        Command::EditScene {
            edits: ["new1", "new2"]
                .into_iter()
                .map(|name| SceneEdit::Mesh {
                    edit: extract(r.clone(), name, vec![1], ExtractionMode::Copy),
                })
                .collect(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.scene.meshes.assets.len(), 3);
}

#[test]
fn linked_plant_copy_is_independent_and_move_requires_explicit_detach() {
    use ignis::{mesh_preservation::DisplacementPolicy, plants::*};
    let mut e = engine();
    let definition = Definition {
        trajectory: None,
        id: "source".into(),
        profile: Rectangle {
            id: "profile".into(),
            width: 2.,
            height: 3.,
        },
        depth: 1.,
        sections: None,
        segments: [1, 1, 1],
        subdivision_levels: 0,
        sampling_tolerance: 0.001,
    };
    e.execute(
        Command::EditPlant {
            edit: ignis::plants::Edit::Put {
                definition,
                detail_policy: DisplacementPolicy::Physical,
                animation_translation_policy: DisplacementPolicy::Physical,
            },
        }
        .into(),
    )
    .unwrap();
    let original = e.scene.plants.clone();
    let r = reference(&mut e);
    assert!(
        e.execute(
            Command::EditMesh {
                edit: extract(r.clone(), "new", vec![0], ExtractionMode::Move)
            }
            .into()
        )
        .unwrap_err()
        .message
        .contains("Detach plant")
    );
    let all = e
        .scene
        .meshes
        .islands("source")
        .unwrap()
        .islands
        .iter()
        .map(|island| island.id)
        .collect();
    e.execute(
        Command::EditMesh {
            edit: extract(r, "new", all, ExtractionMode::Copy),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.scene.plants, original);
    assert_eq!(
        e.scene.meshes.asset("source").unwrap().geometry,
        e.scene.meshes.asset("new").unwrap().geometry
    );
    let r = reference(&mut e);
    e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Plant {
                    edit: ignis::plants::Edit::Detach {
                        id: "source".into(),
                    },
                },
                SceneEdit::Mesh {
                    edit: extract(r, "moved", vec![0], ExtractionMode::Move),
                },
            ],
        }
        .into(),
    )
    .unwrap();
    assert!(e.scene.plants.entries.is_empty());
    assert_eq!(e.scene.meshes.islands("source").unwrap().islands.len(), 5);
    assert_eq!(e.scene.meshes.islands("moved").unwrap().islands.len(), 1);
}

#[test]
fn extraction_exceeds_old_catalog_and_storage_but_preserves_key_atomicity() {
    let mut e = setup();
    for index in 1..20 {
        let mut a = asset();
        a.id = format!("a{index}");
        a.clips.clear();
        e.execute(
            Command::EditMesh {
                edit: Edit::PutAsset { asset: a },
            }
            .into(),
        )
        .unwrap();
    }
    let r = reference(&mut e);
    e.execute(
        Command::EditMesh {
            edit: extract(r, "new", vec![1], ExtractionMode::Move),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.scene.meshes.assets.len(), 21);
    let mut e = setup();
    let mut a = asset();
    let mut clip = (*a.clips[0]).clone();
    clip.duration_ticks = 4096;
    clip.tracks[0].keys = (0..4096)
        .map(|tick| Keyframe {
            tick,
            translation: [0., 1., 0.],
            rotation: [0., 0., 0., 1.],
        })
        .collect();
    a.clips = vec![Arc::new(clip)];
    e.execute(
        Command::EditMesh {
            edit: Edit::PutAsset { asset: a },
        }
        .into(),
    )
    .unwrap();
    let r = reference(&mut e);
    let before = e.snapshot();
    for mode in [ExtractionMode::Copy, ExtractionMode::Move] {
        assert!(
            e.execute(
                Command::EditMesh {
                    edit: extract(r.clone(), "new", vec![1], mode)
                }
                .into()
            )
            .unwrap_err()
            .message
            .contains("clip/key")
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut e = engine();
    for index in 0..8 {
        e.execute(
            Command::EditMesh {
                edit: Edit::Grid {
                    id: if index == 0 {
                        "source".into()
                    } else {
                        format!("grid{index}")
                    },
                    segments: 63,
                    size: 1.,
                },
            }
            .into(),
        )
        .unwrap();
    }
    let r = reference(&mut e);
    e.execute(
        Command::EditMesh {
            edit: extract(r, "new", vec![0], ExtractionMode::Copy),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.scene.meshes.assets.len(), 9);
}

#[test]
fn generic_preview_handles_existing_edits_without_changing_selection_or_revision() {
    let mut e = setup();
    let before = e.snapshot();
    let r = reference(&mut e);
    let p = e
        .execute(
            Command::PreviewMeshEdit {
                edit: Edit::Subdivide {
                    asset: "source".into(),
                    reference: r,
                    components: None,
                },
            }
            .into(),
        )
        .unwrap();
    assert_ne!(p["meshes"], serde_json::to_value(&e.scene.meshes).unwrap());
    assert_eq!(e.snapshot(), before);
    let p = e
        .execute(
            Command::PreviewMeshEdit {
                edit: Edit::SetShading {
                    asset: "source".into(),
                    shading: Shading::Flat,
                },
            }
            .into(),
        )
        .unwrap();
    assert_eq!(p["meshes"]["assets"][0]["shading"], "flat");
    assert_eq!(e.snapshot(), before);
}

fn island_transform(reference: Reference, islands: Vec<u32>, transform: IslandTransform) -> Edit {
    Edit::TransformIslands {
        asset: "source".into(),
        reference,
        islands,
        transform,
    }
}
fn movement() -> IslandTransform {
    IslandTransform {
        pivot: [3., 0., 0.],
        translation: [1., 2., 3.],
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
        scale: 2.,
    }
}

#[test]
fn transform_uses_pivot_preserves_other_islands_and_all_rig_channels() {
    let mut e = setup();
    let before = e.scene.clone();
    let r = reference(&mut e);
    let edit = island_transform(r.clone(), vec![1], movement());
    let preview = e
        .execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(e.scene, before);
    e.execute(Command::EditMesh { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(
        serde_json::to_value(&e.scene.meshes).unwrap(),
        preview["meshes"]
    );
    let old = before.meshes.asset("source").unwrap();
    let new = e.scene.meshes.asset("source").unwrap();
    for (i, p) in new.geometry.positions.iter().enumerate() {
        if (3..6).contains(&i) {
            let expected = [[4., 2., 3.], [4., 4., 3.], [2., 2., 3.]][i - 3];
            for axis in 0..3 {
                assert!((p[axis] - expected[axis]).abs() < 1e-6);
            }
        } else {
            assert_eq!(p, &old.geometry.positions[i]);
        }
    }
    assert_eq!(new.geometry.triangles, old.geometry.triangles);
    assert_eq!(
        new.geometry.material_coordinates,
        old.geometry.material_coordinates
    );
    assert!(Arc::ptr_eq(
        new.rig.as_ref().unwrap(),
        old.rig.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(&new.clips[0], &old.clips[0]));
    assert_eq!(new.material, old.material);
    assert_eq!(new.shading, old.shading);
    assert_eq!(e.scene.meshes.instances, before.meshes.instances);
    // Bone pose is unchanged, while only the edited rest vertices change under skinning.
    let a = before.meshes.pose("one", 60).unwrap();
    let b = e.scene.meshes.pose("one", 60).unwrap();
    assert_eq!(a.joints, b.joints);
    for i in [0, 1, 2, 6, 7, 8, 9] {
        assert_eq!(a.positions[i], b.positions[i]);
    }
    assert_ne!(a.positions[3], b.positions[3]);
    assert!(e.execute(Command::EditMesh { edit }.into()).is_err());
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
}

#[test]
fn transform_freezes_implicit_material_domain_without_welding_coincident_islands() {
    let mut e = setup();
    let mut a = asset();
    Arc::make_mut(&mut a.geometry).material_coordinates = None;
    e.execute(
        Command::EditMesh {
            edit: Edit::PutAsset { asset: a },
        }
        .into(),
    )
    .unwrap();
    let before = e.scene.meshes.asset("source").unwrap().clone();
    let r = reference(&mut e);
    e.execute(
        Command::EditMesh {
            edit: island_transform(
                r,
                vec![2, 0],
                IslandTransform {
                    pivot: [0.; 3],
                    translation: [0., 0., 2.],
                    rotation: [0., 0., 0., 1.],
                    scale: 1.,
                },
            ),
        }
        .into(),
    )
    .unwrap();
    let g = &e.scene.meshes.asset("source").unwrap().geometry;
    assert_eq!(
        g.material_coordinates.as_ref().unwrap(),
        &before.geometry.positions
    );
    for i in [0, 1, 2, 6, 7, 8] {
        assert_eq!(g.positions[i][2], 2.);
    }
    for i in [3, 4, 5, 9] {
        assert_eq!(g.positions[i], before.geometry.positions[i]);
    }
    assert_eq!(e.scene.meshes.islands("source").unwrap().islands.len(), 3);
}

#[test]
fn transform_invalid_parameters_selection_and_final_precision_are_atomic() {
    let mut e = setup();
    let r = reference(&mut e);
    let before = e.scene.clone();
    let mut transforms = vec![];
    for scale in [0., -1., 101., f32::NAN] {
        transforms.push(IslandTransform {
            scale,
            ..movement()
        });
    }
    transforms.push(IslandTransform {
        rotation: [0.; 4],
        ..movement()
    });
    transforms.push(IslandTransform {
        pivot: [10001.; 3],
        ..movement()
    });
    transforms.push(IslandTransform {
        translation: [f32::INFINITY; 3],
        ..movement()
    });
    transforms.push(IslandTransform {
        translation: [10000.; 3],
        ..movement()
    });
    for t in transforms {
        let edit = island_transform(r.clone(), vec![1], t);
        assert!(
            e.execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
                .is_err()
        );
        assert!(e.execute(Command::EditMesh { edit }.into()).is_err());
        assert_eq!(e.scene, before);
    }
    for selected in [vec![], vec![1, 1], vec![99]] {
        assert!(
            e.execute(
                Command::EditMesh {
                    edit: island_transform(r.clone(), selected, movement())
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.scene, before);
    }
    // At 9999 the f32 ULP erases a rotated thin triangle, though f64 is regular.
    e = engine();
    let mut a = asset();
    a.rig = None;
    a.clips.clear();
    a.geometry = Arc::new(Geometry {
        positions: vec![[0., 0., 0.], [0.0001, 0., 0.], [0., 0.0001, 0.]],
        triangles: vec![[0, 1, 2]],
        material_coordinates: None,
    });
    e.execute(
        Command::EditMesh {
            edit: Edit::PutAsset { asset: a },
        }
        .into(),
    )
    .unwrap();
    let r = reference(&mut e);
    let before = e.scene.clone();
    assert!(
        e.execute(
            Command::EditMesh {
                edit: island_transform(
                    r,
                    vec![0],
                    IslandTransform {
                        pivot: [0.; 3],
                        translation: [9999.; 3],
                        rotation: [0., 0., 0., 1.],
                        scale: 1.
                    }
                )
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.scene, before);
}
