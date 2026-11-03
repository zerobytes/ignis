use glam::{DQuat, DVec3};
use ignis::{
    engine::{Command, Engine},
    mesh_preservation::DisplacementPolicy,
    mesh_topology::Topology,
    meshes,
    model::{RoomRecipe, Scene},
    plants::{CrossSection, Definition, Edit, Rectangle, Sections, Station},
};
use std::sync::Arc;

fn drawing() -> Definition {
    Definition {
        trajectory: None,
        id: "trunk".into(),
        profile: Rectangle {
            id: "trunk_profile".into(),
            width: 0.8,
            height: 4.8,
        },
        depth: 0.65,
        sections: Some(Sections {
            interpolation: Default::default(),
            id: "trunk_sections".into(),
            cross_section: CrossSection::Ellipse,
            stations: vec![
                Station {
                    id: "root".into(),
                    at: 0.,
                    width: 1.,
                    depth: 1.,
                    offset: [0., 0.],
                },
                Station {
                    id: "bend".into(),
                    at: 0.34,
                    width: 0.75,
                    depth: 0.83,
                    offset: [0.16, 0.13],
                },
                Station {
                    id: "crown".into(),
                    at: 1.,
                    width: 0.3,
                    depth: 0.2,
                    offset: [-0.35, 0.4],
                },
            ],
        }),
        segments: [4, 5, 4],
        subdivision_levels: 0,
        sampling_tolerance: 0.001,
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
fn vec(p: [f32; 3]) -> DVec3 {
    DVec3::from_array(p.map(f64::from))
}

#[test]
fn pchip_volume_roundtrips_all_section_shapes_and_retains_exact_station_values() {
    use ignis::plants::SectionInterpolation;
    for cross in [
        CrossSection::Rectangle,
        CrossSection::Diamond,
        CrossSection::Ellipse,
    ] {
        let mut d = drawing();
        let linear = d.sample().unwrap();
        d.sections.as_mut().unwrap().cross_section = cross;
        d.sections.as_mut().unwrap().interpolation = SectionInterpolation::Pchip;
        for x in -2..=2 {
            for y in -30..=30 {
                for z in -2..=2 {
                    let m = [f64::from(x) / 4., f64::from(y) / 60., f64::from(z) / 4.];
                    let inverse = d.material_at(d.position_at(m)).unwrap();
                    assert!(DVec3::from_array(m).distance(DVec3::from_array(inverse)) < 1e-12);
                }
            }
        }
        let sections = d.sections.as_ref().unwrap();
        for (i, s) in sections.stations.iter().enumerate() {
            let m = [
                0.,
                i as f64 / (sections.stations.len() - 1) as f64 - 0.5,
                0.,
            ];
            assert!(
                DVec3::from_array(d.position_at(m)).distance(DVec3::new(
                    f64::from(s.offset[0]),
                    (f64::from(s.at) - 0.5) * f64::from(d.profile.height),
                    f64::from(s.offset[1])
                )) < 1e-12
            );
        }
        let smooth = d.sample().unwrap();
        smooth.geometry.validate().unwrap();
        assert_eq!(
            d.counts().unwrap(),
            (
                smooth.geometry.positions.len(),
                smooth.geometry.triangles.len()
            )
        );
        if cross == CrossSection::Ellipse {
            assert_eq!(linear.material, smooth.material);
            assert_eq!(linear.geometry.triangles, smooth.geometry.triangles);
            assert_ne!(linear.geometry.positions, smooth.geometry.positions);
            let topology = Topology::inspect(&smooth.geometry).unwrap();
            assert_eq!(topology.boundary_edges, 0);
            assert_eq!(topology.non_manifold_edges, 0);
        }
    }
}

#[test]
fn switching_linear_pchip_keeps_detail_weights_material_rotated_rig_and_motion_without_rebasing() {
    use ignis::plants::SectionInterpolation;
    for policy in [
        DisplacementPolicy::Physical,
        DisplacementPolicy::Proportional,
    ] {
        let mut e = engine();
        decorate(&mut e);
        let source = e.scene.plants.entries[0].source.clone();
        let original = e.scene.meshes.asset("trunk").unwrap().clone();
        let old = drawing();
        let base = old.sample().unwrap();
        let mut d = old.clone();
        d.profile.height = 5.8;
        d.profile.width = 1.;
        let s = d.sections.as_mut().unwrap();
        s.interpolation = SectionInterpolation::Pchip;
        s.stations[1].offset = [-0.3, 0.28];
        s.stations[1].at = 0.45;
        s.stations[2].offset = [0.4, -0.2];
        s.stations[2].depth = 0.4;
        e.execute(put(d.clone(), policy).into()).unwrap();
        let result = e.scene.meshes.asset("trunk").unwrap().clone();
        assert!(Arc::ptr_eq(&source, &e.scene.plants.entries[0].source));
        assert_eq!(result.material, original.material);
        assert_eq!(result.shading, original.shading);
        assert_eq!(
            result.geometry.material_coordinates,
            original.geometry.material_coordinates
        );
        assert_eq!(
            result.rig.as_ref().unwrap().weights,
            original.rig.as_ref().unwrap().weights
        );
        let target = d.sample().unwrap();
        for i in 0..target.material.len() {
            let scale = if policy == DisplacementPolicy::Physical {
                DVec3::ONE
            } else {
                vec(d.dimensions()) / vec(old.dimensions())
            };
            let delta =
                (vec(source.geometry.positions[i]) - vec(base.geometry.positions[i])) * scale;
            assert!(
                (vec(result.geometry.positions[i]) - vec(target.geometry.positions[i]) - delta)
                    .length()
                    < 1e-6
            );
        }
        let rig = result.rig.as_ref().unwrap();
        let rotation = DQuat::from_array(rig.bones[0].rotation.map(f64::from)).normalize();
        let global = vec(rig.bones[0].translation) + rotation * vec(rig.bones[1].translation);
        assert!(global.distance(DVec3::from_array(d.position_at([-0.1, 0.25, 0.1]))) < 1e-6);
        assert_eq!(
            result.clips[0].tracks[0].keys[0].translation,
            rig.bones[1].translation
        );
        assert_eq!(
            result.clips[0].tracks[0].keys[1].rotation,
            original.clips[0].tracks[0].keys[1].rotation
        );
        for _ in 0..3 {
            e.execute(put(old.clone(), policy).into()).unwrap();
            assert_eq!(e.scene.meshes.asset("trunk").unwrap(), &original);
            e.execute(put(d.clone(), policy).into()).unwrap();
            assert_eq!(e.scene.meshes.asset("trunk").unwrap(), &result);
        }
        let mut fine = d.clone();
        fine.segments = [8, 10, 8];
        e.execute(put(fine, policy).into()).unwrap();
        assert!(Arc::ptr_eq(&source, &e.scene.plants.entries[0].source));
        e.execute(Command::Undo.into()).unwrap();
        assert_eq!(e.scene.meshes.asset("trunk").unwrap(), &result);
        let scene: Scene = serde_json::from_slice(&serde_json::to_vec(&e.scene).unwrap()).unwrap();
        assert_eq!(Engine::new(scene).unwrap().scene, e.scene);
        let replay = e.replay();
        assert_eq!(Engine::replay_at(&replay, None).unwrap().scene, e.scene);
    }
}

#[test]
fn pchip_is_opt_in_versioned_in_target_and_reference_while_linear_serialization_stays_exact() {
    use ignis::plants::SectionInterpolation;
    let d = drawing();
    let serialized = serde_json::to_value(&d).unwrap();
    assert!(serialized["sections"].get("interpolation").is_none());
    let mut explicit = serialized.clone();
    explicit["sections"]["interpolation"] = "linear".into();
    let decoded: Definition = serde_json::from_value(explicit.clone()).unwrap();
    assert_eq!(decoded, d);
    assert_eq!(
        decoded.sample().unwrap().geometry,
        d.sample().unwrap().geometry
    );
    explicit["sections"]["interpolation"] = "unbounded_spline".into();
    assert!(serde_json::from_value::<Definition>(explicit).is_err());
    let mut smooth = d.clone();
    smooth.sections.as_mut().unwrap().interpolation = SectionInterpolation::Pchip;
    let mut e = engine();
    e.execute(put(smooth.clone(), DisplacementPolicy::Physical).into())
        .unwrap();
    let mut old = e.scene.clone();
    old.version = 56;
    assert!(old.validate().unwrap_err().message.contains("version57"));
    // Start with PCHIP so the canonical reference retains it after a Linear edit.
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    e.execute(put(smooth, DisplacementPolicy::Physical).into())
        .unwrap();
    e.execute(put(d, DisplacementPolicy::Physical).into())
        .unwrap();
    let mut old = e.scene.clone();
    old.version = 56;
    assert!(old.validate().unwrap_err().message.contains("version57"));
}

#[test]
fn curved_solid_inverse_is_unique_inside_and_on_all_boundaries() {
    let d = drawing();
    for x in -4..=4 {
        for y in -10..=10 {
            for z in -4..=4 {
                let m = [f64::from(x) / 8., f64::from(y) / 20., f64::from(z) / 8.];
                let p = d.position_at(m);
                let inverse = d.material_at(p).unwrap();
                assert!(DVec3::from_array(m).distance(DVec3::from_array(inverse)) < 1e-12);
            }
        }
    }
    for m in [[0.6, 0., 0.], [0., 0., 0.6]] {
        assert!(d.material_at(d.position_at(m)).is_err());
    }
    assert!(
        d.material_at([0., -0.6 * f64::from(d.profile.height), 0.])
            .is_err()
    );
}

#[test]
fn ellipse_sampling_is_closed_oriented_shared_and_matches_preflight_at_each_level() {
    for segments in [[1, 1, 1], [3, 4, 7], [4, 5, 4]] {
        for level in 0..=2 {
            let mut d = drawing();
            d.segments = segments;
            d.subdivision_levels = level;
            let s = d.sample().unwrap();
            let t = Topology::inspect(&s.geometry).unwrap();
            assert_eq!(
                d.counts().unwrap(),
                (s.geometry.positions.len(), s.geometry.triangles.len())
            );
            assert_eq!(t.boundary_edges, 0);
            assert_eq!(t.non_manifold_edges, 0);
            assert!(t.isolated_vertices.is_empty());
            assert_eq!(
                s.geometry.positions.len() as i64 - t.edges.len() as i64
                    + s.geometry.triangles.len() as i64,
                2
            );
            let mut adjacency = vec![vec![]; s.material.len()];
            for edge in &t.edges {
                assert_eq!(edge.faces.len(), 2);
                let [a, b] = edge.vertices;
                adjacency[a as usize].push(b as usize);
                adjacency[b as usize].push(a as usize);
                let orientation: i32 = edge
                    .faces
                    .iter()
                    .map(|&face| {
                        let tri = s.geometry.triangles[face as usize];
                        if (0..3).any(|i| tri[i] == a && tri[(i + 1) % 3] == b) {
                            1
                        } else {
                            -1
                        }
                    })
                    .sum();
                assert_eq!(orientation, 0);
            }
            let mut visited = vec![false; s.material.len()];
            let mut pending = vec![0];
            while let Some(i) = pending.pop() {
                if visited[i] {
                    continue;
                }
                visited[i] = true;
                pending.extend(adjacency[i].iter().copied());
            }
            assert!(visited.iter().all(|v| *v));
            for (&m, &face) in s.material.iter().zip(&s.surface) {
                let axis = face as usize / 2;
                let side = if face % 2 == 0 { -0.5 } else { 0.5 };
                assert_eq!(m[axis], side);
                let (tri, weights) = s.correspondence(face as usize, m);
                let mapped: DVec3 = s.geometry.triangles[tri as usize]
                    .into_iter()
                    .zip(weights)
                    .map(|(i, w)| vec(s.material[i as usize]) * f64::from(w))
                    .sum();
                assert!(mapped.distance(vec(m)) < 1e-6);
            }
            // Oriented, closed surface has positive signed volume.
            let volume: f64 = s
                .geometry
                .triangles
                .iter()
                .map(|t| {
                    let [a, b, c] = t.map(|i| vec(s.geometry.positions[i as usize]));
                    a.dot(b.cross(c)) / 6.
                })
                .sum();
            assert!(volume > 0.);
        }
    }
}

fn decorate(e: &mut Engine) {
    let d = drawing();
    let s = d.sample().unwrap();
    let p = d.position_at([0.5, 0., 0.]);
    mesh(
        e,
        meshes::Edit::Sculpt {
            asset: "trunk".into(),
            stroke: meshes::Stroke {
                kind: meshes::BrushKind::Grab,
                centers: vec![p.map(|x| x as f32)],
                radius: 0.5,
                strength: 0.004,
                direction: [1., 0., 0.],
            },
        },
    );
    mesh(
        e,
        meshes::Edit::SetMaterialCoordinates {
            asset: "trunk".into(),
            coordinates: Some(
                s.material
                    .iter()
                    .map(|m| [m[0] * 2., m[1] * 3., m[2] * 4.])
                    .collect(),
            ),
        },
    );
    mesh(
        e,
        meshes::Edit::SetShading {
            asset: "trunk".into(),
            shading: meshes::Shading::Smooth,
        },
    );
    let root = DVec3::from_array(d.position_at([0.1, -0.4, -0.1]));
    let tip = DVec3::from_array(d.position_at([-0.1, 0.25, 0.1]));
    let rotation = DQuat::from_rotation_y(0.3);
    let local = (rotation.conjugate() * (tip - root))
        .to_array()
        .map(|x| x as f32);
    mesh(
        e,
        meshes::Edit::SetRig {
            asset: "trunk".into(),
            rig: Some(Arc::new(meshes::Rig {
                bones: vec![
                    meshes::Bone {
                        name: "root".into(),
                        parent: None,
                        translation: root.to_array().map(|x| x as f32),
                        rotation: rotation.to_array().map(|x| x as f32),
                    },
                    meshes::Bone {
                        name: "branch".into(),
                        parent: Some(0),
                        translation: local,
                        rotation: [0., 0., 0., 1.],
                    },
                ],
                weights: s
                    .material
                    .iter()
                    .map(|m| meshes::Weight {
                        extra: None,
                        joints: [0, 1, 0, 0],
                        weights: [0.5 - m[1], 0.5 + m[1], 0., 0.],
                    })
                    .collect(),
            })),
        },
    );
    let mut moved = local;
    moved[0] += 0.02;
    mesh(
        e,
        meshes::Edit::PutClip {
            asset: "trunk".into(),
            clip: Arc::new(meshes::Clip {
                id: "bend".into(),
                duration_ticks: 60,
                tracks: vec![meshes::Track {
                    bone: 1,
                    keys: vec![
                        meshes::Keyframe {
                            tick: 0,
                            translation: local,
                            rotation: [0., 0., 0., 1.],
                        },
                        meshes::Keyframe {
                            tick: 60,
                            translation: moved,
                            rotation: [0., 0., 0.1, 0.9949874],
                        },
                    ],
                }],
            }),
        },
    );
}

#[test]
fn reauthoring_curvature_preserves_canonical_sculpt_weights_materials_and_rotated_rig_without_drift()
 {
    for policy in [
        DisplacementPolicy::Physical,
        DisplacementPolicy::Proportional,
    ] {
        let mut e = engine();
        decorate(&mut e);
        let old = drawing();
        let source = e.scene.plants.entries[0].source.clone();
        let base = old.sample().unwrap();
        let original = e.scene.meshes.asset("trunk").unwrap().clone();
        let mut d = old.clone();
        d.profile.width = 1.;
        d.profile.height = 5.8;
        let sections = d.sections.as_mut().unwrap();
        sections.stations[1].offset = [-0.3, 0.28];
        sections.stations[1].at = 0.45;
        sections.stations[2].offset = [0.4, -0.2];
        sections.stations[2].depth = 0.4;
        e.execute(put(d.clone(), policy).into()).unwrap();
        assert!(Arc::ptr_eq(&source, &e.scene.plants.entries[0].source));
        let result = e.scene.meshes.asset("trunk").unwrap().clone();
        let target = d.sample().unwrap();
        assert_eq!(result.shading, original.shading);
        assert_eq!(result.material, original.material);
        assert_eq!(
            result.geometry.material_coordinates,
            original.geometry.material_coordinates
        );
        assert_eq!(
            result.rig.as_ref().unwrap().weights,
            original.rig.as_ref().unwrap().weights
        );
        for i in 0..target.geometry.positions.len() {
            let scale = if policy == DisplacementPolicy::Physical {
                DVec3::ONE
            } else {
                vec(d.dimensions()) / vec(old.dimensions())
            };
            let detail =
                (vec(source.geometry.positions[i]) - vec(base.geometry.positions[i])) * scale;
            assert!(
                (vec(result.geometry.positions[i]) - vec(target.geometry.positions[i]) - detail)
                    .length()
                    < 1e-6
            );
        }
        let rig = result.rig.as_ref().unwrap();
        let rotation = DQuat::from_array(rig.bones[0].rotation.map(f64::from)).normalize();
        let global = vec(rig.bones[0].translation) + rotation * vec(rig.bones[1].translation);
        assert!(global.distance(DVec3::from_array(d.position_at([-0.1, 0.25, 0.1]))) < 1e-6);
        assert_eq!(
            result.clips[0].tracks[0].keys[0].translation,
            rig.bones[1].translation
        );
        assert_eq!(
            result.clips[0].tracks[0].keys[1].rotation,
            original.clips[0].tracks[0].keys[1].rotation
        );
        for _ in 0..3 {
            e.execute(put(old.clone(), policy).into()).unwrap();
            assert_eq!(e.scene.meshes.asset("trunk").unwrap(), &original);
            e.execute(put(d.clone(), policy).into()).unwrap();
            assert_eq!(e.scene.meshes.asset("trunk").unwrap(), &result);
        }
        let mut fine = d.clone();
        fine.segments = [8, 10, 8];
        e.execute(put(fine, policy).into()).unwrap();
        assert!(Arc::ptr_eq(&source, &e.scene.plants.entries[0].source));
        let decoded: Scene =
            serde_json::from_slice(&serde_json::to_vec(&e.scene).unwrap()).unwrap();
        assert_eq!(Engine::new(decoded).unwrap().scene, e.scene);
        e.execute(Command::Undo.into()).unwrap();
        assert_eq!(e.scene.meshes.asset("trunk").unwrap(), &result);
    }
}

#[test]
fn invalid_offsets_topology_identity_and_old_versions_fail_without_mutation() {
    let mut e = engine();
    decorate(&mut e);
    let before = e.snapshot();
    let mut cases = vec![];
    for offset in [[f32::NAN, 0.], [101., 0.]] {
        let mut d = drawing();
        d.sections.as_mut().unwrap().stations[1].offset = offset;
        cases.push(d);
    }
    let mut d = drawing();
    d.sections.as_mut().unwrap().cross_section = CrossSection::Diamond;
    cases.push(d);
    let mut d = drawing();
    d.segments = [63; 3];
    cases.push(d);
    for d in cases {
        assert!(
            e.execute(put(d, DisplacementPolicy::Physical).into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut legacy = e.scene.clone();
    legacy.version = 55;
    assert!(legacy.validate().unwrap_err().message.contains("version56"));
    // The canonical reference is gated even when a forged target looks legacy.
    legacy.plants.entries[0]
        .definition
        .sections
        .as_mut()
        .unwrap()
        .cross_section = CrossSection::Rectangle;
    assert!(legacy.validate().is_err());
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(replay.scene.version, ignis::model::SCENE_VERSION);
    assert_eq!(Engine::replay_at(&replay, None).unwrap().scene, e.scene);
}
