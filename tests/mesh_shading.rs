use glam::{Quat, Vec3};
use ignis::{
    engine::{Command, Engine},
    mesh_topology::Plan,
    meshes::*,
    model::RoomRecipe,
};
use std::sync::Arc;

fn asset() -> Asset {
    Asset {
        material: Default::default(),
        id: "fold".into(),
        geometry: Arc::new(Geometry {
            material_coordinates: None,
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 3.]],
            triangles: vec![[0, 1, 2], [0, 3, 1]],
        }),
        shading: Shading::Flat,
        rig: None,
        clips: vec![],
    }
}
fn engine(asset: Asset) -> Engine {
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    edit(&mut e, Edit::PutAsset { asset });
    edit(
        &mut e,
        Edit::PutInstance {
            instance: Instance {
                id: "one".into(),
                asset: "fold".into(),
                position: [0.; 3],
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
fn smooth(e: &mut Engine) {
    edit(
        e,
        Edit::SetShading {
            asset: "fold".into(),
            shading: Shading::Smooth,
        },
    );
}
fn near(a: [f32; 3], b: Vec3) {
    assert!((Vec3::from(a) - b).length() < 0.00001, "{a:?} != {b:?}");
}

#[test]
fn shared_normals_are_area_weighted_and_same_tick_cache_observes_mode() {
    let mut e = engine(asset());
    let mut frame = e.render_frame();
    let before = frame.meshes.vertices.clone();
    near(before[0].normal, Vec3::Z);
    near(before[3].normal, Vec3::Y);
    smooth(&mut e);
    e.update_render_frame(&mut frame);
    let after = frame.meshes.vertices.clone();
    let weighted = Vec3::new(0., 3., 1.).normalize();
    for i in [0, 1, 3, 5] {
        near(after[i].normal, weighted);
    }
    near(after[2].normal, Vec3::Z);
    near(after[4].normal, Vec3::Y);
    assert_eq!(
        before.iter().map(|v| v.position).collect::<Vec<_>>(),
        after.iter().map(|v| v.position).collect::<Vec<_>>()
    );
    e.execute(Command::Undo.into()).unwrap();
    e.update_render_frame(&mut frame);
    assert_eq!(before, frame.meshes.vertices);
    e.execute(Command::Redo.into()).unwrap();
    e.update_render_frame(&mut frame);
    assert_eq!(after, frame.meshes.vertices);
}

#[test]
fn duplicated_indices_keep_a_seam_and_smooth_never_welds_positions() {
    let mut a = asset();
    let g = Arc::make_mut(&mut a.geometry);
    g.positions.extend([[0., 0., 0.], [1., 0., 0.]]);
    g.triangles[1] = [4, 3, 5];
    let mut e = engine(a);
    let before = e.render_frame().meshes.vertices;
    smooth(&mut e);
    assert_eq!(before, e.render_frame().meshes.vertices);
}

#[test]
fn normals_follow_instance_transform_without_changing_geometry_or_counts() {
    let mut e = engine(asset());
    smooth(&mut e);
    let mut instance = e.scene.meshes.instances[0].clone();
    instance.rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2).to_array();
    instance.position = [3., -4., 8.];
    instance.scale = 2.;
    edit(&mut e, Edit::PutInstance { instance });
    let frame = e.render_frame();
    near(
        frame.meshes.vertices[0].normal,
        Vec3::new(0., -1., 3.).normalize(),
    );
    assert_eq!(frame.meshes.vertices.len(), 6);
    assert_eq!(e.scene.meshes.assets[0].geometry, asset().geometry);
}

fn animated() -> Asset {
    let mut a = asset();
    a.shading = Shading::Smooth;
    a.rig = Some(Arc::new(Rig {
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
                translation: [0.; 3],
                rotation: [0., 0., 0., 1.],
            },
        ],
        weights: (0..4)
            .map(|i| Weight {
                extra: None,
                joints: [u8::from(i == 3), 0, 0, 0],
                weights: [1., 0., 0., 0.],
            })
            .collect(),
    }));
    a.clips = vec![Arc::new(Clip {
        id: "bend".into(),
        duration_ticks: 60,
        tracks: vec![Track {
            bone: 1,
            keys: vec![
                Keyframe {
                    tick: 0,
                    translation: [0.; 3],
                    rotation: [0., 0., 0., 1.],
                },
                Keyframe {
                    tick: 60,
                    translation: [0.; 3],
                    rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2).to_array(),
                },
            ],
        }],
    })];
    a
}

#[test]
fn smooth_normals_are_recomputed_from_deformed_surface_not_bind_normals() {
    let mut e = engine(animated());
    let mut i = e.scene.meshes.instances[0].clone();
    i.playback = Some(Playback {
        clip: "bend".into(),
        offset_ticks: 0,
        looping: false,
    });
    edit(&mut e, Edit::PutInstance { instance: i });
    let mut frame = e.render_frame();
    near(
        frame.meshes.vertices[0].normal,
        Vec3::new(0., 3., 1.).normalize(),
    );
    e.execute(Command::Step { ticks: 60 }.into()).unwrap();
    e.update_render_frame(&mut frame);
    near(frame.meshes.vertices[0].normal, Vec3::Z);
    near(frame.meshes.vertices[4].normal, Vec3::Z);
}

#[test]
fn cancelled_and_collapsed_normals_remain_finite_with_face_fallback() {
    let mut a = asset();
    Arc::make_mut(&mut a.geometry).triangles = vec![[0, 1, 2], [0, 2, 1]];
    a.shading = Shading::Smooth;
    let frame = engine(a).render_frame();
    near(frame.meshes.vertices[0].normal, Vec3::Z);
    near(frame.meshes.vertices[3].normal, -Vec3::Z);
    let mut a = animated();
    Arc::make_mut(&mut a.clips[0]).tracks[0].keys[1] = Keyframe {
        tick: 60,
        translation: [0., 0., -3.],
        rotation: [0., 0., 0., 1.],
    };
    let mut e = engine(a);
    let mut i = e.scene.meshes.instances[0].clone();
    i.playback = Some(Playback {
        clip: "bend".into(),
        offset_ticks: 60,
        looping: false,
    });
    edit(&mut e, Edit::PutInstance { instance: i });
    for v in e.render_frame().meshes.vertices {
        assert!(v.normal.iter().all(|v| v.is_finite()));
        assert!((Vec3::from(v.normal).length() - 1.).abs() < 0.00001);
    }
}

#[test]
fn mode_survives_subdivision_and_old_versions_cannot_smuggle_it() {
    let mut a = asset();
    a.shading = Shading::Smooth;
    let divided = Plan::new(&a.geometry, None).unwrap().asset(&a).unwrap();
    assert_eq!(divided.shading, Shading::Smooth);
    assert_eq!(divided.clips, a.clips);
    let mut e = engine(a);
    e.scene.version = 51;
    assert!(
        e.scene
            .validate()
            .unwrap_err()
            .message
            .contains("version52")
    );
    let mut legacy = serde_json::to_value(asset()).unwrap();
    legacy.as_object_mut().unwrap().remove("shading");
    assert_eq!(
        serde_json::from_value::<Asset>(legacy).unwrap().shading,
        Shading::Flat
    );
}

#[test]
fn missing_asset_error_is_atomic_and_setting_mode_preserves_payload_sharing() {
    let mut e = engine(animated());
    let g = e.scene.meshes.assets[0].geometry.clone();
    let r = e.scene.meshes.assets[0].rig.clone().unwrap();
    let c = e.scene.meshes.assets[0].clips.clone();
    edit(
        &mut e,
        Edit::SetShading {
            asset: "fold".into(),
            shading: Shading::Flat,
        },
    );
    assert!(Arc::ptr_eq(&g, &e.scene.meshes.assets[0].geometry));
    assert!(Arc::ptr_eq(
        &r,
        e.scene.meshes.assets[0].rig.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(&c[0], &e.scene.meshes.assets[0].clips[0]));
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::EditMesh {
                edit: Edit::SetShading {
                    asset: "missing".into(),
                    shading: Shading::Smooth
                }
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
}

#[test]
fn diagnostics_match_render_corners_and_filtered_neighborhood_exactly() {
    let mut e = engine(asset());
    let report = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 0, None, 32).unwrap();
    assert_eq!(report.maximum_degrees, 0.);
    assert_eq!(report.world_area, 2.);
    smooth(&mut e);
    let all = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 0, None, 32).unwrap();
    let filtered = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 0, Some(&[0]), 1).unwrap();
    assert_eq!(filtered.samples[0], all.samples[0]);
    assert!((filtered.maximum_degrees - 3f64.atan().to_degrees()).abs() < 1e-5);
    let frame = e.render_frame();
    for s in &all.samples {
        for (c, v) in frame.meshes.vertices[(s.face as usize * 3)..][..3]
            .iter()
            .enumerate()
        {
            assert_eq!(s.corner_normals[c], v.normal);
            assert_eq!(s.positions[c], v.position);
        }
    }
    let zero = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 0, None, 0).unwrap();
    assert!(zero.samples.is_empty());
    assert_eq!(zero.maximum_degrees, all.maximum_degrees);
}

#[test]
fn diagnostics_preserve_index_seams_and_report_cancelled_sums() {
    let mut a = asset();
    a.shading = Shading::Smooth;
    let g = Arc::make_mut(&mut a.geometry);
    g.positions.extend([[0., 0., 0.], [1., 0., 0.]]);
    g.triangles[1] = [4, 3, 5];
    let e = engine(a);
    let r = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 0, None, 8).unwrap();
    assert_eq!(r.maximum_degrees, 0.);
    let mut a = asset();
    a.shading = Shading::Smooth;
    Arc::make_mut(&mut a.geometry).triangles = vec![[0, 1, 2], [0, 2, 1]];
    let e = engine(a);
    let r = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 0, None, 8).unwrap();
    assert_eq!(r.smooth_fallback_corners, 6);
    assert_eq!(r.geometric_fallback_faces, 0);
    assert_eq!(r.samples[0].geometric_normal, [0., 0., 1.]);
    assert_eq!(r.samples[1].geometric_normal, [0., 0., -1.]);
}

#[test]
fn diagnostics_evaluate_explicit_rig_tick_transform_and_collapsed_pose() {
    let mut a = animated();
    Arc::make_mut(&mut a.clips[0]).tracks[0].keys[1] = Keyframe {
        tick: 60,
        translation: [0., 0., -3.],
        rotation: [0., 0., 0., 1.],
    };
    let mut e = engine(a);
    let mut i = e.scene.meshes.instances[0].clone();
    i.playback = Some(Playback {
        clip: "bend".into(),
        offset_ticks: 0,
        looping: false,
    });
    i.rotation = Quat::from_rotation_y(0.5).to_array();
    i.scale = 2.;
    i.position = [3., -1., 4.];
    edit(&mut e, Edit::PutInstance { instance: i });
    let r = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 60, None, 8).unwrap();
    assert_eq!(r.geometric_fallback_faces, 1);
    assert!((r.world_area - 2.).abs() < 1e-6);
    assert_eq!(e.simulation.tick, 0);
    e.execute(Command::Step { ticks: 60 }.into()).unwrap();
    let frame = e.render_frame();
    for s in r.samples {
        for c in 0..3 {
            assert_eq!(
                s.corner_normals[c],
                frame.meshes.vertices[s.face as usize * 3 + c].normal
            );
        }
    }
}

#[test]
fn normal_query_order_limits_schema_and_failures_are_pure() {
    let mut e = engine(asset());
    smooth(&mut e);
    let before = e.execute(Command::Inspect.into()).unwrap();
    for command in [
        serde_json::json!({"op":"inspect_mesh_normals","instance":"one","faces":[]}),
        serde_json::json!({"op":"inspect_mesh_normals","instance":"one","faces":[0,0]}),
        serde_json::json!({"op":"inspect_mesh_normals","instance":"one","faces":[2]}),
        serde_json::json!({"op":"inspect_mesh_normals","instance":"absent"}),
        serde_json::json!({"op":"inspect_mesh_normals","instance":"one","tick":36001}),
        serde_json::json!({"op":"inspect_mesh_normals","instance":"one","sample_limit":33}),
    ] {
        let command: Command = serde_json::from_value(command).unwrap();
        assert!(e.execute(command.into()).is_err());
        assert_eq!(before, e.execute(Command::Inspect.into()).unwrap());
    }
    let a = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 0, Some(&[1, 0]), 8).unwrap();
    let b = ignis::mesh_normals::inspect(&e.scene.meshes, "one", 0, None, 8).unwrap();
    assert_eq!(a, b);
    let command: Command =
        serde_json::from_value(serde_json::json!({"op":"inspect_mesh_normals","instance":"one"}))
            .unwrap();
    let r = e.execute(command.into()).unwrap();
    assert_eq!(r["report"]["sample_limit"], 8);
    assert_eq!(before, e.execute(Command::Inspect.into()).unwrap());
}
