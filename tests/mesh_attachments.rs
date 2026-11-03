use glam::{Mat4, Quat, Vec3};
use ignis::{
    engine::{Command, Engine, SceneEdit},
    meshes::*,
    model::RoomRecipe,
};
use std::sync::Arc;

fn sheet() -> Asset {
    Asset {
        id: "sheet".into(),
        geometry: Arc::new(Geometry {
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2]],
            material_coordinates: None,
        }),
        shading: Shading::Smooth,
        material: Default::default(),
        rig: None,
        clips: vec![],
    }
}
fn instance(name: &str) -> Instance {
    Instance {
        id: name.into(),
        asset: "sheet".into(),
        position: [0.; 3],
        rotation: [0., 0., 0., 1.],
        scale: 1.,
        color: [0.5; 3],
        playback: None,
        attachment: None,
    }
}
fn attach(parent: &str, bone: Option<&str>) -> Option<Attachment> {
    Some(Attachment {
        parent: parent.into(),
        bone: bone.map(String::from),
    })
}
fn near(a: [f32; 3], b: [f32; 3]) {
    assert!(
        Vec3::from(a).distance(Vec3::from(b)) < 0.00001,
        "{a:?} {b:?}"
    );
}
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
fn install(e: &mut Engine, assets: Vec<Asset>, instances: Vec<Instance>) {
    let mut edits: Vec<_> = assets
        .into_iter()
        .map(|asset| SceneEdit::Mesh {
            edit: Edit::PutAsset { asset },
        })
        .collect();
    edits.extend(instances.into_iter().map(|instance| SceneEdit::Mesh {
        edit: Edit::PutInstance { instance },
    }));
    e.execute(Command::EditScene { edits }.into()).unwrap();
}
fn animated() -> Asset {
    let mut a = sheet();
    a.rig = Some(Arc::new(Rig {
        bones: vec![
            Bone {
                name: "root".into(),
                parent: None,
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            },
            Bone {
                name: "socket".into(),
                parent: Some(0),
                translation: [0., 2., 0.],
                rotation: [0., 0., 0., 1.],
            },
        ],
        weights: vec![
            Weight {
                extra: None,
                joints: [0; 4],
                weights: [1., 0., 0., 0.]
            };
            3
        ],
    }));
    a.clips.push(Arc::new(Clip {
        id: "bend".into(),
        duration_ticks: 60,
        tracks: vec![Track {
            bone: 1,
            keys: vec![
                Keyframe {
                    tick: 0,
                    translation: [0., 2., 0.],
                    rotation: [0., 0., 0., 1.],
                },
                Keyframe {
                    tick: 60,
                    translation: [0., 3., 0.],
                    rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
                },
            ],
        }],
    }));
    a
}

#[test]
fn arbitrary_storage_order_parent_rotation_scale_and_world_query_match_analytic_positions() {
    let mut parent = instance("parent");
    parent.position = [10., 0., 0.];
    parent.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array();
    parent.scale = 2.;
    let mut child = instance("child");
    child.position = [1., 0., 0.];
    child.scale = 0.5;
    child.attachment = attach("parent", None);
    let d = Document {
        objects: Default::default(),
        assets: vec![sheet()],
        instances: vec![child, parent],
    };
    d.validate().unwrap();
    let p = d.pose("child", 0).unwrap();
    near(p.positions[0], [10., 2., 0.]);
    near(p.positions[1], [10., 3., 0.]);
    let world = Mat4::from_cols_array_2d(&p.world_transform);
    near(world.transform_point3(Vec3::ZERO).to_array(), [10., 2., 0.]);
    assert_eq!(d.poses(0)[0], p);
}

#[test]
fn named_bone_and_nested_child_follow_parent_clip_with_independent_child_clock() {
    let mut parent = instance("parent");
    parent.position = [4., 0., 0.];
    parent.playback = Some(Playback {
        clip: "bend".into(),
        offset_ticks: 0,
        looping: false,
    });
    let mut child = instance("child");
    child.position = [1., 0., 0.];
    child.attachment = attach("parent", Some("socket"));
    child.playback = Some(Playback {
        clip: "bend".into(),
        offset_ticks: 0,
        looping: true,
    });
    let mut grand = instance("grand");
    grand.position = [0., 1., 0.];
    grand.attachment = attach("child", None);
    let d = Document {
        objects: Default::default(),
        assets: vec![animated()],
        instances: vec![grand, child, parent],
    };
    d.validate().unwrap();
    near(d.pose("child", 0).unwrap().positions[0], [5., 2., 0.]);
    let p = d.pose("child", 60).unwrap();
    near(p.positions[0], [4., 4., 0.]);
    assert_eq!(p.clip_tick, Some(0));
    near(d.pose("grand", 60).unwrap().positions[0], [3., 4., 0.]);
    assert_eq!(d.pose("parent", 60).unwrap().clip_tick, Some(60));
    // Bone matrices keep the original asset-space meaning.
    near(
        Mat4::from_cols_array_2d(&d.pose("parent", 60).unwrap().joints[1])
            .transform_point3(Vec3::ZERO)
            .to_array(),
        [0., 3., 0.],
    );
}

#[test]
fn errors_preserve_scene_history_runtime_and_can_be_resolved_in_one_batch() {
    let mut e = engine();
    let p = instance("parent");
    let mut c = instance("child");
    c.attachment = attach("parent", Some("socket"));
    install(&mut e, vec![animated()], vec![c.clone(), p.clone()]);
    e.execute(Command::Step { ticks: 7 }.into()).unwrap();
    let before = e.snapshot();
    let scene = e.scene.clone();
    let mut cycle = p.clone();
    cycle.attachment = attach("child", None);
    let mut missing = c.clone();
    missing.attachment = attach("missing", None);
    let mut badbone = c.clone();
    badbone.attachment = attach("parent", Some("renamed"));
    let mut scale = c.clone();
    scale.scale = 100.;
    let mut p2 = p.clone();
    p2.scale = 2.;
    for edits in [
        vec![Edit::PutInstance { instance: cycle }],
        vec![Edit::PutInstance { instance: missing }],
        vec![Edit::PutInstance { instance: badbone }],
        vec![Edit::RemoveInstance {
            id: "parent".into(),
        }],
        vec![Edit::SetRig {
            asset: "sheet".into(),
            rig: None,
        }],
        vec![
            Edit::PutInstance { instance: scale },
            Edit::PutInstance { instance: p2 },
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
        assert_eq!(e.snapshot(), before);
        assert_eq!(e.scene, scene);
    }
    // Final validation permits removing the parent while explicitly detaching child.
    c.attachment = None;
    e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Mesh {
                    edit: Edit::RemoveInstance {
                        id: "parent".into(),
                    },
                },
                SceneEdit::Mesh {
                    edit: Edit::PutInstance { instance: c },
                },
            ],
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, scene);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene.meshes.instances.len(), 1);
}

#[test]
fn long_chains_are_bounded_and_effective_scale_rejected_without_partial_state() {
    let mut d = Document {
        objects: Default::default(),
        assets: vec![sheet()],
        instances: vec![],
    };
    for index in 0..4096 {
        let mut i = instance(&format!("i{index}"));
        i.position = [0., 1., 0.];
        if index > 0 {
            i.attachment = attach(&format!("i{}", index - 1), None);
        }
        d.instances.push(i);
    }
    d.validate().unwrap();
    near(d.pose("i4095", 0).unwrap().positions[0], [0., 4096., 0.]);
    d.instances[31].attachment = attach("i31", None);
    assert!(d.validate().is_err());
    d.instances[31].attachment = attach("i30", None);
    d.instances[0].scale = 0.001;
    d.instances[1].scale = 0.5;
    assert!(d.validate().is_err());
}

#[test]
fn frame_and_pose_agree_under_ancestor_animation_removal_undo_and_invalidation() {
    let mut e = engine();
    let mut p = instance("parent");
    p.playback = Some(Playback {
        clip: "bend".into(),
        offset_ticks: 0,
        looping: false,
    });
    let mut c = instance("child");
    c.position = [1., 0., 0.];
    c.attachment = attach("parent", Some("socket"));
    install(&mut e, vec![animated()], vec![c, p]);
    let mut frame = Frame::default();
    frame.update(&e.scene.meshes, 0);
    let before = frame.vertices.clone();
    frame.update(&e.scene.meshes, 60);
    assert_ne!(frame.vertices, before);
    for (a, b) in frame.vertices[..3]
        .iter()
        .zip(e.scene.meshes.pose("child", 60).unwrap().positions)
    {
        assert_eq!(a.position, b);
    }
    let ptr = frame.vertices.as_ptr();
    frame.update(&e.scene.meshes, 60);
    assert_eq!(ptr, frame.vertices.as_ptr());
    let mut c = e.scene.meshes.instances[0].clone();
    c.attachment = None;
    e.execute(
        Command::EditMesh {
            edit: Edit::PutInstance { instance: c },
        }
        .into(),
    )
    .unwrap();
    frame.update(&e.scene.meshes, 60);
    let detached = frame.vertices.clone();
    e.execute(Command::Undo.into()).unwrap();
    frame.update(&e.scene.meshes, 60);
    assert_ne!(frame.vertices, detached);
    for (a, b) in frame.vertices[..3]
        .iter()
        .zip(e.scene.meshes.pose("child", 60).unwrap().positions)
    {
        assert_eq!(a.position, b);
    }
}

#[test]
fn formats_preserve_legacy_instances_and_gate_attachment_before_migration() {
    let mut e = engine();
    let p = instance("parent");
    let mut c = instance("child");
    c.attachment = attach("parent", None);
    install(&mut e, vec![sheet()], vec![c, p]);
    let mut old = e.scene.clone();
    old.version = 57;
    assert!(old.validate().unwrap_err().message.contains("version58"));
    assert!(Engine::new(old).is_err());
    let j = serde_json::to_value(instance("legacy")).unwrap();
    assert!(j.get("attachment").is_none());
    assert_eq!(
        serde_json::from_value::<Instance>(j).unwrap(),
        instance("legacy")
    );
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(replay.scene.version, ignis::model::SCENE_VERSION);
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
