use glam::Quat;
use ignis::{
    engine::{Command, Engine, Request, SceneEdit},
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
fn triangle() -> Asset {
    Asset {
        material: Default::default(),
        shading: Default::default(),
        id: "sheet".into(),
        geometry: Arc::new(Geometry {
            material_coordinates: None,
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2]],
        }),
        rig: None,
        clips: vec![],
    }
}
fn instance(id: &str) -> Instance {
    Instance {
        id: id.into(),
        asset: "sheet".into(),
        position: [0.; 3],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: [0.8, 0.3, 0.1],
        attachment: None,
        playback: None,
    }
}
fn setup() -> Engine {
    let mut e = engine();
    edit(&mut e, Edit::PutAsset { asset: triangle() });
    edit(
        &mut e,
        Edit::PutInstance {
            instance: instance("one"),
        },
    );
    e
}
fn rig() -> Arc<Rig> {
    Arc::new(Rig {
        bones: vec![
            Bone {
                name: "root".into(),
                parent: None,
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            },
            Bone {
                name: "tip".into(),
                parent: Some(0),
                translation: [0., 1., 0.],
                rotation: [0., 0., 0., 1.],
            },
        ],
        weights: vec![
            Weight {
                extra: None,
                joints: [0; 4],
                weights: [1., 0., 0., 0.],
            },
            Weight {
                extra: None,
                joints: [0, 1, 0, 0],
                weights: [0.5, 0.5, 0., 0.],
            },
            Weight {
                extra: None,
                joints: [1, 0, 0, 0],
                weights: [1., 0., 0., 0.],
            },
        ],
    })
}
fn clip() -> Arc<Clip> {
    Arc::new(Clip {
        id: "bend".into(),
        duration_ticks: 60,
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
                    translation: [0., 2., 0.],
                    rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
                },
            ],
        }],
    })
}
fn animated() -> Engine {
    let mut e = setup();
    edit(
        &mut e,
        Edit::SetRig {
            asset: "sheet".into(),
            rig: Some(rig()),
        },
    );
    edit(
        &mut e,
        Edit::PutClip {
            asset: "sheet".into(),
            clip: clip(),
        },
    );
    let mut i = instance("one");
    i.playback = Some(Playback {
        clip: "bend".into(),
        offset_ticks: 0,
        looping: false,
    });
    edit(&mut e, Edit::PutInstance { instance: i });
    e
}
fn close(a: [f32; 3], b: [f32; 3]) {
    assert!(
        a.iter().zip(b).all(|(a, b)| (a - b).abs() < 0.0001),
        "{a:?} != {b:?}"
    );
}
fn rejects(e: &mut Engine, edit: Edit) {
    let before = e.snapshot();
    assert!(e.execute(Command::EditMesh { edit }.into()).is_err());
    assert_eq!(e.snapshot(), before);
}
#[test]
fn bind_pose_preserves_vertices_and_world_transform() {
    let mut e = setup();
    edit(
        &mut e,
        Edit::SetRig {
            asset: "sheet".into(),
            rig: Some(rig()),
        },
    );
    let p = e.scene.meshes.pose("one", 0).unwrap();
    assert_eq!(p.positions, triangle().geometry.positions);
    let mut i = instance("two");
    i.position = [4., 2., 3.];
    i.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array();
    i.scale = 2.;
    edit(&mut e, Edit::PutInstance { instance: i });
    close(
        e.scene.meshes.pose("two", 0).unwrap().positions[1],
        [4., 4., 3.],
    );
}
#[test]
fn skeletal_interpolation_weights_and_endpoint() {
    let e = animated();
    let p = e.scene.meshes.pose("one", 30).unwrap();
    close(p.positions[0], [0., 0., 0.]);
    close(p.positions[2], [0., 1.5, 0.]);
    let s = std::f32::consts::FRAC_1_SQRT_2;
    close(p.positions[1], [(1. + 2. * s) * 0.5, 0.75, 0.]);
    let p = e.scene.meshes.pose("one", 60).unwrap();
    close(p.positions[2], [0., 2., 0.]);
    assert_eq!(p, e.scene.meshes.pose("one", 1000).unwrap());
}
#[test]
fn parent_animation_propagates_and_untracked_bones_keep_bind() {
    let mut e = setup();
    edit(
        &mut e,
        Edit::SetRig {
            asset: "sheet".into(),
            rig: Some(rig()),
        },
    );
    let c = Arc::new(Clip {
        id: "shift".into(),
        duration_ticks: 5,
        tracks: vec![Track {
            bone: 0,
            keys: vec![Keyframe {
                tick: 0,
                translation: [3., 0., 0.],
                rotation: [0., 0., 0., 1.],
            }],
        }],
    });
    edit(
        &mut e,
        Edit::PutClip {
            asset: "sheet".into(),
            clip: c,
        },
    );
    let mut i = instance("one");
    i.playback = Some(Playback {
        clip: "shift".into(),
        offset_ticks: 0,
        looping: false,
    });
    edit(&mut e, Edit::PutInstance { instance: i });
    close(
        e.scene.meshes.pose("one", 3).unwrap().positions[2],
        [3., 1., 0.],
    );
}
#[test]
fn loop_offset_and_independent_instances() {
    let mut e = animated();
    let mut i = instance("two");
    i.playback = Some(Playback {
        clip: "bend".into(),
        offset_ticks: 30,
        looping: true,
    });
    edit(&mut e, Edit::PutInstance { instance: i });
    assert_eq!(e.scene.meshes.pose("two", 30).unwrap().clip_tick, Some(0));
    close(
        e.scene.meshes.pose("two", 0).unwrap().positions[2],
        [0., 1.5, 0.],
    );
    close(
        e.scene.meshes.pose("one", 0).unwrap().positions[2],
        [0., 1., 0.],
    );
}
#[test]
fn stroke_deterministic_and_copy_on_write_history() {
    let mut a = engine();
    edit(
        &mut a,
        Edit::Grid {
            id: "sheet".into(),
            segments: 4,
            size: 2.,
        },
    );
    let original = a.scene.meshes.assets[0].geometry.clone();
    let scene = a.scene.clone();
    assert!(Arc::ptr_eq(&original, &scene.meshes.assets[0].geometry));
    let s = Stroke {
        kind: BrushKind::Draw,
        centers: vec![[0.; 3]],
        radius: 0.75,
        strength: 0.5,
        direction: [0.; 3],
    };
    let mut b = Engine::new(a.scene.clone()).unwrap();
    edit(
        &mut a,
        Edit::Sculpt {
            asset: "sheet".into(),
            stroke: s.clone(),
        },
    );
    edit(
        &mut b,
        Edit::Sculpt {
            asset: "sheet".into(),
            stroke: s,
        },
    );
    assert_eq!(a.scene.meshes, b.scene.meshes);
    let result = a.scene.meshes.assets[0].geometry.clone();
    assert_eq!(result.positions[12], [0., 0.5, 0.]);
    assert_eq!(result.positions[0], original.positions[0]);
    assert_eq!(result.triangles, original.triangles);
    assert_eq!(original.positions[12], [0.; 3]);
    a.execute(Command::Undo.into()).unwrap();
    assert!(Arc::ptr_eq(&a.scene.meshes.assets[0].geometry, &original));
    a.execute(Command::Redo.into()).unwrap();
    assert!(Arc::ptr_eq(&a.scene.meshes.assets[0].geometry, &result));
}
#[test]
fn grab_and_smooth_are_local_and_reversible() {
    let mut e = engine();
    edit(
        &mut e,
        Edit::Grid {
            id: "sheet".into(),
            segments: 4,
            size: 2.,
        },
    );
    edit(
        &mut e,
        Edit::Sculpt {
            asset: "sheet".into(),
            stroke: Stroke {
                kind: BrushKind::Grab,
                centers: vec![[0.; 3]],
                radius: 0.4,
                strength: 0.7,
                direction: [0., 5., 0.],
            },
        },
    );
    assert_eq!(
        e.scene.meshes.assets[0].geometry.positions[12],
        [0., 0.7, 0.]
    );
    edit(
        &mut e,
        Edit::Sculpt {
            asset: "sheet".into(),
            stroke: Stroke {
                kind: BrushKind::Smooth,
                centers: vec![[0., 0.7, 0.]],
                radius: 0.4,
                strength: 1.,
                direction: [0.; 3],
            },
        },
    );
    close(e.scene.meshes.assets[0].geometry.positions[12], [0.; 3]);
}
#[test]
fn invalid_geometry_and_strokes_preserve_live_execution() {
    let mut e = setup();
    e.advance(8).unwrap();
    let mut a = triangle();
    a.geometry = Arc::new(Geometry {
        material_coordinates: None,
        positions: vec![[0.; 3]; 3],
        triangles: vec![[0, 1, 2]],
    });
    rejects(&mut e, Edit::PutAsset { asset: a });
    let mut a = triangle();
    Arc::make_mut(&mut a.geometry).triangles[0] = [0, 1, 3];
    rejects(&mut e, Edit::PutAsset { asset: a });
    for (radius, strength, direction) in [
        (0., 0.5, [0., 1., 0.]),
        (1., 2., [0., 1., 0.]),
        (1., 0.5, [0.; 3]),
    ] {
        rejects(
            &mut e,
            Edit::Sculpt {
                asset: "sheet".into(),
                stroke: Stroke {
                    kind: BrushKind::Grab,
                    centers: vec![[0.; 3]],
                    radius,
                    strength,
                    direction,
                },
            },
        );
    }
}
#[test]
fn invalid_rig_and_clip_reject_atomically() {
    let mut e = animated();
    for variant in 0..5 {
        let mut r = (*rig()).clone();
        match variant {
            0 => r.bones[0].parent = Some(1),
            1 => r.weights.pop().map(|_| ()).unwrap(),
            2 => r.weights[0].weights = [0.5, 0., 0., 0.],
            3 => r.bones[1].name = "root".into(),
            _ => r.weights[0].joints[0] = 32,
        };
        rejects(
            &mut e,
            Edit::SetRig {
                asset: "sheet".into(),
                rig: Some(Arc::new(r)),
            },
        );
    }
    for variant in 0..4 {
        let mut c = (*clip()).clone();
        match variant {
            0 => c.tracks[0].keys[1].tick = 0,
            1 => c.tracks[0].bone = 2,
            2 => c.tracks[0].keys[0].rotation = [0.; 4],
            _ => c.duration_ticks = 0,
        };
        rejects(
            &mut e,
            Edit::PutClip {
                asset: "sheet".into(),
                clip: Arc::new(c),
            },
        );
    }
    rejects(
        &mut e,
        Edit::SetRig {
            asset: "sheet".into(),
            rig: None,
        },
    );
    rejects(
        &mut e,
        Edit::RemoveClip {
            asset: "sheet".into(),
            clip: "bend".into(),
        },
    );
    rejects(&mut e, Edit::RemoveAsset { id: "sheet".into() });
}
#[test]
fn instance_validation_and_concurrency_guard() {
    let mut e = setup();
    for v in 0..4 {
        let mut i = instance("bad");
        match v {
            0 => i.asset = "missing".into(),
            1 => i.rotation = [0.; 4],
            2 => i.scale = -1.,
            _ => i.color = [2.; 3],
        };
        rejects(&mut e, Edit::PutInstance { instance: i });
    }
    let before = e.snapshot();
    assert!(
        e.execute(Request {
            expected_revision: Some(e.revision - 1),
            command: Command::EditMesh {
                edit: Edit::RemoveInstance { id: "one".into() }
            }
        })
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
}
#[test]
fn batch_removes_dependencies_atomically_and_undo_restores() {
    let mut e = animated();
    let before = e.scene.clone();
    e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Mesh {
                    edit: Edit::RemoveAsset { id: "sheet".into() },
                },
                SceneEdit::Mesh {
                    edit: Edit::RemoveInstance { id: "one".into() },
                },
            ],
        }
        .into(),
    )
    .unwrap();
    assert!(e.scene.meshes.is_empty());
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
}
#[test]
fn replay_compares_poses_and_queries_are_read_only() {
    let mut e = animated();
    e.advance(30).unwrap();
    let before = e.snapshot();
    let value = e
        .execute(
            Command::InspectMeshPose {
                instance: "one".into(),
                tick: Some(60),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(value["pose"]["clip_tick"], 60);
    assert_eq!(e.snapshot(), before);
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert!(
        Engine::verify_replay(&replay).unwrap()["matched"]
            .as_bool()
            .unwrap()
    );
    let prefix = Engine::replay_at(&replay, Some(15)).unwrap();
    assert_eq!(prefix.scene.meshes.poses(15), e.scene.meshes.poses(15));
    let mut bad = replay;
    bad.expected_meshes[0].positions[0][0] += 1.;
    assert_eq!(
        Engine::verify_replay(&bad).unwrap()["meshes_matched"],
        false
    );
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(e.scene.meshes.pose("one", 0).unwrap().clip_tick, Some(0));
}
#[test]
fn legacy_scene_migrates_but_cannot_smuggle_meshes() {
    let mut e = setup();
    e.scene.version = 47;
    assert!(e.scene.validate().is_err());
    e.scene.meshes = Document::default();
    assert!(Engine::new(e.scene).unwrap().scene.version == ignis::model::SCENE_VERSION);
}
#[test]
fn operation_budgets_and_nonfinite_data_are_rejected() {
    let mut e = setup();
    let mut a = triangle();
    Arc::make_mut(&mut a.geometry).positions[0][0] = f32::NAN;
    rejects(&mut e, Edit::PutAsset { asset: a });
    rejects(
        &mut e,
        Edit::Grid {
            id: "huge".into(),
            segments: 64,
            size: 1.,
        },
    );
    let mut r = (*rig()).clone();
    r.weights[0].weights[0] = f32::INFINITY;
    rejects(
        &mut e,
        Edit::SetRig {
            asset: "sheet".into(),
            rig: Some(Arc::new(r)),
        },
    );
    let mut d = e.scene.meshes.clone();
    d.instances = (0..33).map(|i| instance(&format!("i{i}"))).collect();
    d.validate().unwrap();
}
#[test]
fn prepared_frame_invalidates_for_tick_edit_reset_and_removal() {
    let mut e = animated();
    let mut f = e.render_frame();
    let base = f.meshes.vertices.clone();
    let pointer = f.meshes.vertices.as_ptr();
    e.update_render_frame(&mut f);
    assert_eq!(pointer, f.meshes.vertices.as_ptr());
    e.advance(30).unwrap();
    e.update_render_frame(&mut f);
    assert_ne!(base, f.meshes.vertices);
    e.execute(Command::Reset.into()).unwrap();
    e.update_render_frame(&mut f);
    assert_eq!(base, f.meshes.vertices);
    edit(&mut e, Edit::RemoveInstance { id: "one".into() });
    e.update_render_frame(&mut f);
    assert!(f.meshes.vertices.is_empty());
    e.execute(Command::Undo.into()).unwrap();
    e.update_render_frame(&mut f);
    assert_eq!(base, f.meshes.vertices);
}

#[test]
fn fine_grid_and_aggregate_storage_grow_with_document() {
    let geometry = Arc::new(Geometry::grid(63, 0.01).unwrap());
    assert_eq!(geometry.positions.len(), 4096);
    let mut document = Document::default();
    for i in 0..9 {
        document.assets.push(Asset {
            material: Default::default(),
            shading: Default::default(),
            id: format!("a{i}"),
            geometry: geometry.clone(),
            rig: None,
            clips: vec![],
        });
    }
    document.validate().unwrap();
    document.assets.truncate(1);
    for i in 0..17 {
        let mut inst = instance(&format!("i{i}"));
        inst.asset = "a0".into();
        document.instances.push(inst);
    }
    document.validate().unwrap();
    document.instances.truncate(16);
    document.validate().unwrap();
}

#[test]
fn four_influences_and_antipodal_quaternion_keys() {
    let mut e = setup();
    let mut r = Rig {
        bones: vec![],
        weights: vec![
            Weight {
                extra: None,
                joints: [0, 1, 2, 3],
                weights: [0.25; 4]
            };
            3
        ],
    };
    for i in 0..4 {
        r.bones.push(Bone {
            name: format!("b{i}"),
            parent: None,
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
        });
    }
    edit(
        &mut e,
        Edit::SetRig {
            asset: "sheet".into(),
            rig: Some(Arc::new(r)),
        },
    );
    let tracks = (0..4)
        .map(|i| Track {
            bone: i,
            keys: vec![
                Keyframe {
                    tick: 0,
                    translation: [i as f32, 0., 0.],
                    rotation: [0., 0., 0., 1.],
                },
                Keyframe {
                    tick: 10,
                    translation: [i as f32, 0., 0.],
                    rotation: [0., 0., 0., -1.],
                },
            ],
        })
        .collect();
    edit(
        &mut e,
        Edit::PutClip {
            asset: "sheet".into(),
            clip: Arc::new(Clip {
                id: "four".into(),
                duration_ticks: 10,
                tracks,
            }),
        },
    );
    let mut i = instance("one");
    i.playback = Some(Playback {
        clip: "four".into(),
        offset_ticks: 0,
        looping: false,
    });
    edit(&mut e, Edit::PutInstance { instance: i });
    close(
        e.scene.meshes.pose("one", 5).unwrap().positions[0],
        [1.5, 0., 0.],
    );
}

#[test]
fn aggregate_key_budget_and_serialized_roundtrip() {
    let mut e = animated();
    let scene = serde_json::to_vec_pretty(&e.scene).unwrap();
    let restored = Engine::new(serde_json::from_slice(&scene).unwrap()).unwrap();
    assert_eq!(restored.scene.meshes.poses(37), e.scene.meshes.poses(37));
    let keys = (0..4096)
        .map(|tick| Keyframe {
            tick,
            translation: [0., 1., 0.],
            rotation: [0., 0., 0., 1.],
        })
        .collect();
    let c = Arc::new(Clip {
        id: "large".into(),
        duration_ticks: 4096,
        tracks: vec![Track { bone: 1, keys }],
    });
    rejects(
        &mut e,
        Edit::PutClip {
            asset: "sheet".into(),
            clip: c,
        },
    );
}

#[test]
fn maximum_vertex_scene_roundtrips_within_document_read_boundary() {
    let mut scene = engine().scene;
    let geometry = Arc::new(Geometry::grid(63, 4.).unwrap());
    let rig = Arc::new(Rig {
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
            geometry.positions.len()
        ],
    });
    scene.meshes.assets = (0..4)
        .map(|i| Asset {
            material: Default::default(),
            shading: Default::default(),
            id: format!("a{i}"),
            geometry: geometry.clone(),
            rig: Some(rig.clone()),
            clips: vec![],
        })
        .collect();
    scene.validate().unwrap();
    assert!(serde_json::to_vec_pretty(&scene).unwrap().len() > 8 * 1024 * 1024);
    let root = tempfile::tempdir().unwrap();
    let mut service = ignis::service::Service::new(root.path(), scene.clone()).unwrap();
    service
        .execute(
            Command::SaveScene {
                path: "large.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert!(
        std::fs::metadata(root.path().join("large.scene.json"))
            .unwrap()
            .len()
            <= 8 * 1024 * 1024
    );
    service
        .execute(
            Command::LoadScene {
                path: "large.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert!(
        service.engine.scene.meshes == scene.meshes,
        "Mesh payload changed across compact JSON roundtrip"
    );
}

#[test]
fn large_pose_replay_roundtrips_in_compact_form() {
    let mut e = setup();
    let mut a = triangle();
    let mut g = Geometry::grid(63, 4.).unwrap();
    g.triangles.truncate(1);
    a.geometry = Arc::new(g);
    edit(&mut e, Edit::PutAsset { asset: a });
    for i in 1..32 {
        edit(
            &mut e,
            Edit::PutInstance {
                instance: instance(&format!("i{i}")),
            },
        );
    }
    let replay = e.replay();
    assert!(serde_json::to_vec_pretty(&replay).unwrap().len() > 8 * 1024 * 1024);
    let root = tempfile::tempdir().unwrap();
    let mut service = ignis::service::Service::new(root.path(), e.scene).unwrap();
    service
        .execute(
            Command::SaveReplay {
                path: "large.replay.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert!(
        std::fs::metadata(root.path().join("large.replay.json"))
            .unwrap()
            .len()
            <= 8 * 1024 * 1024
    );
    assert_eq!(
        service
            .execute(
                Command::RunReplay {
                    path: "large.replay.json".into()
                }
                .into()
            )
            .unwrap()
            .data["matched"],
        true
    );
}

#[test]
fn legacy_draw_formats_remain_gated_and_new_scenes_allow_more() {
    let mut e = engine();
    edit(
        &mut e,
        Edit::Grid {
            id: "sheet".into(),
            segments: 32,
            size: 1.,
        },
    );
    let mut sheet = e.scene.meshes.assets[0].clone();
    sheet.geometry = Arc::new(Geometry::grid(46, 1.).unwrap());
    Arc::make_mut(&mut sheet.geometry).triangles.truncate(4096);
    edit(&mut e, Edit::PutAsset { asset: sheet });
    let edits = (0..32)
        .map(|n| SceneEdit::Mesh {
            edit: Edit::PutInstance {
                instance: instance(&format!("i-{n}")),
            },
        })
        .collect();
    e.execute(Command::EditScene { edits }.into()).unwrap();
    assert_eq!(
        e.scene.meshes.assets[0].geometry.triangles.len() * e.scene.meshes.instances.len(),
        131072
    );
    let mut frame = Frame::default();
    frame.update(&e.scene.meshes, 0);
    assert_eq!(frame.triangle_count(), 131072);
    assert_eq!(frame.resident_vertex_count(), 4096 * 3);
    let mut legacy = e.scene.clone();
    legacy.version = 64;
    assert!(legacy.validate().unwrap_err().message.contains("version65"));
    legacy.meshes.instances.truncate(16);
    legacy.version = 54;
    assert!(legacy.validate().unwrap_err().message.contains("version55"));
    legacy.meshes.instances.truncate(8);
    legacy.validate().unwrap();
    let mut over = Geometry::grid(46, 1.).unwrap();
    over.triangles.truncate(4097);
    edit(
        &mut e,
        Edit::PutAsset {
            asset: Asset {
                id: "over".into(),
                geometry: Arc::new(over),
                shading: Shading::Flat,
                material: Default::default(),
                rig: None,
                clips: vec![],
            },
        },
    );
    e.advance(7).unwrap();
    let mut replacement = instance("i-31");
    replacement.asset = "over".into();
    let before = e.scene.clone();
    e.execute(
        Command::EditMesh {
            edit: Edit::PutInstance {
                instance: replacement,
            },
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.render_frame().meshes.triangle_count(), 131073);
    let mut legacy = e.scene.clone();
    legacy.version = 102;
    assert!(
        legacy
            .validate()
            .unwrap_err()
            .message
            .contains("version103")
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene.meshes, before.meshes);
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(replay.scene.version, ignis::model::SCENE_VERSION);
}
