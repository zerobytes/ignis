use ignis::{
    engine::{Command, Engine},
    mesh_preservation::DisplacementPolicy,
    meshes,
    model::RoomRecipe,
    plants::{CrossSection, Definition, Edit, Rectangle, Sections, Station},
};
use std::sync::Arc;

fn drawing() -> Definition {
    Definition {
        trajectory: None,
        id: "blade".into(),
        profile: Rectangle {
            id: "front".into(),
            width: 1.,
            height: 4.,
        },
        depth: 0.5,
        sections: Some(Sections {
            interpolation: Default::default(),
            id: "cuts".into(),
            cross_section: CrossSection::Diamond,
            stations: vec![
                Station {
                    offset: [0.; 2],
                    id: "heel".into(),
                    at: 0.,
                    width: 1.,
                    depth: 1.,
                },
                Station {
                    offset: [0.; 2],
                    id: "shoulder".into(),
                    at: 0.5,
                    width: 0.75,
                    depth: 0.75,
                },
                Station {
                    offset: [0.; 2],
                    id: "tip".into(),
                    at: 1.,
                    width: 0.125,
                    depth: 0.125,
                },
            ],
        }),
        segments: [4, 4, 4],
        subdivision_levels: 0,
        sampling_tolerance: 1. / 1024.,
    }
}
fn put(d: Definition, policy: DisplacementPolicy) -> Command {
    Command::EditPlant {
        edit: Edit::Put {
            definition: d,
            detail_policy: policy,
            animation_translation_policy: policy,
        },
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
    e.execute(put(drawing(), DisplacementPolicy::Physical).into())
        .unwrap();
    e
}
fn mesh(e: &mut Engine, edit: meshes::Edit) {
    e.execute(Command::EditMesh { edit }.into()).unwrap();
}
fn decorate(e: &mut Engine) {
    mesh(
        e,
        meshes::Edit::Sculpt {
            asset: "blade".into(),
            stroke: meshes::Stroke {
                kind: meshes::BrushKind::Grab,
                centers: vec![[0., 0., 0.1875]],
                radius: 0.4,
                strength: 0.01,
                direction: [0., 0., 1.],
            },
        },
    );
    let n = e
        .scene
        .meshes
        .asset("blade")
        .unwrap()
        .geometry
        .positions
        .len();
    mesh(
        e,
        meshes::Edit::SetRig {
            asset: "blade".into(),
            rig: Some(Arc::new(meshes::Rig {
                bones: vec![
                    meshes::Bone {
                        name: "root".into(),
                        parent: None,
                        translation: [0., -2., 0.],
                        rotation: [0., 0., 0., 1.],
                    },
                    meshes::Bone {
                        name: "shoulder".into(),
                        parent: Some(0),
                        translation: [0.125, 2., 0.],
                        rotation: [0., 0., 0., 1.],
                    },
                ],
                weights: vec![
                    meshes::Weight {
                        extra: None,
                        joints: [0, 1, 0, 0],
                        weights: [0.5, 0.5, 0., 0.]
                    };
                    n
                ],
            })),
        },
    );
    mesh(
        e,
        meshes::Edit::PutClip {
            asset: "blade".into(),
            clip: Arc::new(meshes::Clip {
                id: "flex".into(),
                duration_ticks: 60,
                tracks: vec![meshes::Track {
                    bone: 1,
                    keys: vec![
                        meshes::Keyframe {
                            tick: 0,
                            translation: [0.125, 2., 0.],
                            rotation: [0., 0., 0., 1.],
                        },
                        meshes::Keyframe {
                            tick: 60,
                            translation: [0.15625, 2., 0.],
                            rotation: [0., 0., 0.1, 0.9949874],
                        },
                    ],
                }],
            }),
        },
    );
}
#[test]
fn section_volume_roundtrips_material_and_has_positive_winding() {
    for cross in [CrossSection::Rectangle, CrossSection::Diamond] {
        let mut d = drawing();
        d.sections.as_mut().unwrap().cross_section = cross;
        let s = d.sample().unwrap();
        s.geometry.validate().unwrap();
        for (&p, &m) in s.geometry.positions.iter().zip(&s.material) {
            let inverse = d.material_at(p.map(f64::from)).unwrap();
            for k in 0..3 {
                assert!((inverse[k] - f64::from(m[k])).abs() < 1e-6);
            }
        }
        for tri in &s.geometry.triangles {
            let [a, b, c] = tri.map(|i| glam::Vec3::from(s.geometry.positions[i as usize]));
            // Tapered, centered cross sections are star shaped around the axis.
            let mid = (a + b + c) / 3.;
            let radial = glam::Vec3::new(mid.x, 0., mid.z);
            let n = (b - a).cross(c - a);
            assert!(n.dot(radial) > 0. || n.y * mid.y > 0.);
        }
    }
}
#[test]
fn section_edits_map_off_axis_bind_origins_keep_detail_and_rest_relative_motion() {
    for policy in [
        DisplacementPolicy::Physical,
        DisplacementPolicy::Proportional,
    ] {
        let mut e = engine();
        decorate(&mut e);
        let canonical = e.scene.plants.entries[0].source.clone();
        let old = drawing();
        let base = old.sample().unwrap();
        let mut d = old.clone();
        d.profile.height = 6.;
        d.profile.width = 2.;
        d.depth = 1.;
        let station = &mut d.sections.as_mut().unwrap().stations[1];
        station.at = 0.25;
        station.width = 0.5;
        station.depth = 0.5;
        e.execute(put(d.clone(), policy).into()).unwrap();
        assert!(Arc::ptr_eq(&canonical, &e.scene.plants.entries[0].source));
        let asset = e.scene.meshes.asset("blade").unwrap();
        let rig = asset.rig.as_ref().unwrap();
        assert_eq!(rig.bones[0].translation, [0., -3., 0.]);
        assert!((rig.bones[1].translation[0] - 1. / 6.).abs() < 1e-6);
        assert_eq!(rig.bones[1].translation[1], 1.5);
        let key = &asset.clips[0].tracks[0].keys[1];
        let expected_delta = if policy == DisplacementPolicy::Physical {
            0.03125
        } else {
            0.0625
        };
        assert!((key.translation[0] - rig.bones[1].translation[0] - expected_delta).abs() < 1e-6);
        assert_eq!(key.rotation, canonical.clips[0].tracks[0].keys[1].rotation);
        let target = d.sample().unwrap();
        for i in 0..asset.geometry.positions.len() {
            for k in 0..3 {
                let scale = if policy == DisplacementPolicy::Physical {
                    1.
                } else {
                    d.dimensions()[k] / old.dimensions()[k]
                };
                let expected =
                    (canonical.geometry.positions[i][k] - base.geometry.positions[i][k]) * scale;
                assert!(
                    (asset.geometry.positions[i][k] - target.geometry.positions[i][k] - expected)
                        .abs()
                        < 1e-6
                );
            }
        }
        for _ in 0..4 {
            e.execute(put(old.clone(), policy).into()).unwrap();
            e.execute(put(d.clone(), policy).into()).unwrap();
        }
        e.execute(put(old, policy).into()).unwrap();
        // Canonicalization normalizes rotations/weights once, then remains stable.
        let original = e.scene.meshes.asset("blade").unwrap().clone();
        e.execute(put(d, policy).into()).unwrap();
        e.execute(put(drawing(), policy).into()).unwrap();
        assert_eq!(e.scene.meshes.asset("blade").unwrap(), &original);
    }
}
#[test]
fn identity_collapse_budgets_and_outside_bones_fail_atomically() {
    let mut e = engine();
    decorate(&mut e);
    let before = e.execute(Command::Inspect.into()).unwrap();
    let mut cases = vec![];
    let mut d = drawing();
    d.sections.as_mut().unwrap().stations.swap(0, 1);
    cases.push(d);
    let mut d = drawing();
    d.sections.as_mut().unwrap().stations.remove(1);
    cases.push(d);
    let mut d = drawing();
    d.sections.as_mut().unwrap().stations[1].id = "replacement".into();
    cases.push(d);
    let mut d = drawing();
    d.sections.as_mut().unwrap().stations[1].width = 0.;
    cases.push(d);
    let mut d = drawing();
    d.sections.as_mut().unwrap().stations[1].at = 0.;
    cases.push(d);
    let mut d = drawing();
    d.sections.as_mut().unwrap().cross_section = CrossSection::Rectangle;
    cases.push(d);
    let mut d = drawing();
    d.segments = [63; 3];
    cases.push(d);
    for d in cases {
        assert!(
            e.execute(put(d, DisplacementPolicy::Physical).into())
                .is_err()
        );
        assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    }
    let mut rig = (**e.scene.meshes.asset("blade").unwrap().rig.as_ref().unwrap()).clone();
    rig.bones[0].translation[0] = 10.;
    assert!(
        e.execute(
            Command::EditMesh {
                edit: meshes::Edit::SetRig {
                    asset: "blade".into(),
                    rig: Some(Arc::new(rig))
                }
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    let mut forged = e.scene.clone();
    forged.version = 50;
    assert!(forged.validate().unwrap_err().message.contains("version51"));
    let mut forged = e.scene.clone();
    if let Some(rig) = Arc::make_mut(&mut forged.plants.entries[0].source)
        .rig
        .as_mut()
    {
        Arc::make_mut(rig).bones[1].parent = Some(31);
    }
    assert!(forged.validate().is_err());
}
#[test]
fn retessellation_subdivision_rebase_and_persistence_preserve_source() {
    let mut e = engine();
    decorate(&mut e);
    let before = e.scene.clone();
    let source = e.scene.plants.entries[0].source.clone();
    let mut d = drawing();
    d.segments = [8, 8, 8];
    e.execute(put(d, DisplacementPolicy::Physical).into())
        .unwrap();
    assert!(Arc::ptr_eq(&source, &e.scene.plants.entries[0].source));
    let saved = serde_json::to_vec(&e.scene).unwrap();
    let loaded = Engine::new(serde_json::from_slice(&saved).unwrap()).unwrap();
    assert_eq!(loaded.scene, e.scene);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
    e.execute(Command::Redo.into()).unwrap();
    e.execute(put(drawing(), DisplacementPolicy::Physical).into())
        .unwrap();
    assert_eq!(e.scene.meshes, before.meshes);
    let mut refined = drawing();
    refined.subdivision_levels = 1;
    e.execute(put(refined, DisplacementPolicy::Physical).into())
        .unwrap();
    let mut changed = drawing();
    changed.profile.height = 5.;
    changed.subdivision_levels = 1;
    e.execute(put(changed, DisplacementPolicy::Physical).into())
        .unwrap();
    e.scene.validate().unwrap();
}
