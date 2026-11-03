use ignis::{meshes, model};
#[path = "../src/mesh_preservation.rs"]
mod mesh_preservation;

use glam::{DQuat, DVec3, Quat, Vec3};
use mesh_preservation::{Correspondence, DisplacementPolicy as Policy, Evaluation, evaluate};
use std::sync::Arc;

const DIMS: [f32; 3] = [2., 4., 2.];
fn plane(n: usize, dimensions: [f32; 3]) -> meshes::Geometry {
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    for y in 0..=n {
        for x in 0..=n {
            positions.push([
                (x as f32 / n as f32 - 0.5) * dimensions[0],
                (y as f32 / n as f32 - 0.5) * dimensions[1],
                0.5 * dimensions[2],
            ]);
        }
    }
    for y in 0..n {
        for x in 0..n {
            let a = (y * (n + 1) + x) as u32;
            triangles.extend([
                [a, a + 1, a + n as u32 + 1],
                [a + 1, a + n as u32 + 2, a + n as u32 + 1],
            ]);
        }
    }
    meshes::Geometry {
        material_coordinates: None,
        positions,
        triangles,
    }
}
fn maps(source_segments: usize, target_segments: usize) -> Vec<Correspondence> {
    let mut result = Vec::new();
    for y in 0..=target_segments {
        for x in 0..=target_segments {
            let sx = x as f32 / target_segments as f32 * source_segments as f32;
            let sy = y as f32 / target_segments as f32 * source_segments as f32;
            let cx = (sx.floor() as usize).min(source_segments - 1);
            let cy = (sy.floor() as usize).min(source_segments - 1);
            let u = sx - cx as f32;
            let v = sy - cy as f32;
            let triangle = ((cy * source_segments + cx) * 2) as u32;
            result.push(if u + v <= 1. {
                Correspondence {
                    triangle,
                    barycentric: [1. - u - v, u, v],
                }
            } else {
                Correspondence {
                    triangle: triangle + 1,
                    barycentric: [1. - v, u + v - 1., 1. - u],
                }
            });
        }
    }
    result
}
fn asset(base: &meshes::Geometry) -> meshes::Asset {
    let mut geometry = base.clone();
    geometry.positions[4][0] += 0.03;
    geometry.positions[4][2] += 0.1;
    meshes::Asset {
        material: Default::default(),
        shading: Default::default(),
        id: "limb".into(),
        geometry: Arc::new(geometry),
        rig: None,
        clips: vec![],
    }
}

#[test]
fn material_coordinates_follow_explicit_correspondence_and_return_without_drift() {
    let base = plane(2, DIMS);
    let mut source = asset(&base);
    let coordinates: Vec<_> = base
        .positions
        .iter()
        .map(|p| [p[0] * 2. + 7., p[1] * 3. - 2., p[2] + 5.])
        .collect();
    Arc::make_mut(&mut source.geometry).material_coordinates = Some(coordinates.clone());
    source.material.transmission = 0.35;
    let dims = [3., 5., 2.];
    let target = plane(4, dims);
    let correspondences = maps(2, 4);
    let out = evaluate(&input(&source, &base, &target, &correspondences, dims)).unwrap();
    assert_eq!(out.asset.material, source.material);
    for (p, c) in target
        .positions
        .iter()
        .zip(out.asset.geometry.material_coordinates.as_ref().unwrap())
    {
        let canonical = [
            p[0] / dims[0] * DIMS[0],
            p[1] / dims[1] * DIMS[1],
            p[2] / dims[2] * DIMS[2],
        ];
        let expected = [
            canonical[0] * 2. + 7.,
            canonical[1] * 3. - 2.,
            canonical[2] + 5.,
        ];
        for i in 0..3 {
            assert!((expected[i] - c[i]).abs() < 0.00001);
        }
    }
    let back = evaluate(&input(&source, &base, &base, &maps(2, 2), DIMS)).unwrap();
    assert_eq!(back.asset.geometry.material_coordinates, Some(coordinates));
}
fn input<'a>(
    source: &'a meshes::Asset,
    reference: &'a meshes::Geometry,
    target: &'a meshes::Geometry,
    maps: &'a [Correspondence],
    target_dims: [f32; 3],
) -> Evaluation<'a> {
    Evaluation {
        source,
        reference_base: reference,
        target_base: target,
        reference_dimensions: DIMS,
        target_dimensions: target_dims,
        correspondences: maps,
        detail_policy: Policy::Physical,
        animation_translation_policy: Policy::Physical,
    }
}
fn close(a: [f32; 3], b: [f32; 3]) {
    assert!(
        (Vec3::from(a) - Vec3::from(b)).length() < 2e-5,
        "{a:?} != {b:?}"
    );
}
fn rotation_close(a: [f32; 4], b: [f32; 4]) {
    assert!(
        Quat::from_array(a)
            .normalize()
            .dot(Quat::from_array(b).normalize())
            .abs()
            > 0.99999
    );
}
fn rigged(base: &meshes::Geometry) -> meshes::Asset {
    let mut source = asset(base);
    let rotations = [
        Quat::from_rotation_z(0.3),
        Quat::from_rotation_x(0.2),
        Quat::from_rotation_y(-0.1),
    ];
    let bones: Vec<_> = (0..3)
        .map(|i| meshes::Bone {
            name: format!("bone{i}"),
            parent: if i == 0 { None } else { Some(i as u8 - 1) },
            translation: if i == 0 { [0., -2., 0.] } else { [0., 2., 0.] },
            rotation: rotations[i].to_array(),
        })
        .collect();
    let weights = base
        .positions
        .iter()
        .map(|p| {
            let t = (p[1] / DIMS[1] + 0.5) * 2.;
            if t <= 1. {
                meshes::Weight {
                    extra: None,
                    joints: [0, 1, 0, 0],
                    weights: [1. - t, t, 0., 0.],
                }
            } else {
                meshes::Weight {
                    extra: None,
                    joints: [1, 2, 0, 0],
                    weights: [2. - t, t - 1., 0., 0.],
                }
            }
        })
        .collect();
    let tracks = [0, 1, 2]
        .map(|i| {
            let bone = &bones[i];
            let rest = Vec3::from(bone.translation);
            let delta = rotations[i] * Vec3::new(0.2, 0.1, -0.05);
            meshes::Track {
                bone: i as u8,
                keys: vec![
                    meshes::Keyframe {
                        tick: 0,
                        translation: rest.to_array(),
                        rotation: bone.rotation,
                    },
                    meshes::Keyframe {
                        tick: 10,
                        translation: (rest + delta).to_array(),
                        rotation: (rotations[i] * Quat::from_rotation_z(0.4)).to_array(),
                    },
                ],
            }
        })
        .to_vec();
    source.rig = Some(Arc::new(meshes::Rig { bones, weights }));
    source.clips.push(Arc::new(meshes::Clip {
        id: "bend".into(),
        duration_ticks: 10,
        tracks,
    }));
    source
}
fn globals(rig: &meshes::Rig) -> Vec<(DVec3, DQuat)> {
    let mut result: Vec<(DVec3, DQuat)> = Vec::new();
    for bone in &rig.bones {
        let t = DVec3::from_array(bone.translation.map(f64::from));
        let q = DQuat::from_array(bone.rotation.map(f64::from)).normalize();
        result.push(match bone.parent {
            Some(p) => {
                let (pt, pq) = result[p as usize];
                (pt + pq * t, (pq * q).normalize())
            }
            None => (t, q),
        });
    }
    result
}
fn pose(asset: meshes::Asset, tick: u64) -> meshes::Pose {
    let document = meshes::Document {
        objects: Default::default(),
        assets: vec![asset],
        instances: vec![meshes::Instance {
            id: "instance".into(),
            asset: "limb".into(),
            position: [0.; 3],
            rotation: Quat::IDENTITY.to_array(),
            scale: 1.,
            color: [1.; 3],
            attachment: None,
            playback: Some(meshes::Playback {
                clip: "bend".into(),
                offset_ticks: 0,
                looping: false,
            }),
        }],
    };
    document.validate().unwrap();
    document.pose("instance", tick).unwrap()
}

#[test]
fn physical_detail_survives_nonuniform_dimensions_and_retessellation() {
    let base = plane(2, DIMS);
    let source = asset(&base);
    let target_dims = [4., 12., 1.];
    let target = plane(4, target_dims);
    let maps = maps(2, 4);
    let original = source.clone();
    let result = evaluate(&input(&source, &base, &target, &maps, target_dims)).unwrap();
    close(result.asset.geometry.positions[12], [0.03, 0., 0.6]);
    close(result.asset.geometry.positions[11], [-1. + 0.015, 0., 0.55]);
    assert_eq!(result.report.source_vertices, 9);
    assert_eq!(result.report.target_vertices, 25);
    assert!(result.report.max_mapping_error < 1e-12);
    assert!(
        (result.report.max_detail_displacement - (0.03f64.powi(2) + 0.1f64.powi(2)).sqrt()).abs()
            < 1e-6
    );
    assert_eq!(source, original);
}

#[test]
fn proportional_detail_scales_authored_vector_by_object_axes() {
    let base = plane(2, DIMS);
    let source = asset(&base);
    let dims = [4., 12., 1.];
    let target = plane(4, dims);
    let maps = maps(2, 4);
    let mut request = input(&source, &base, &target, &maps, dims);
    request.detail_policy = Policy::Proportional;
    close(
        evaluate(&request).unwrap().asset.geometry.positions[12],
        [0.06, 0., 0.55],
    );
}

#[test]
fn canonical_return_has_no_accumulated_transfer_drift() {
    let base = plane(2, DIMS);
    let source = rigged(&base);
    let maps = maps(2, 2);
    let initial = evaluate(&input(&source, &base, &base, &maps, DIMS)).unwrap();
    for dims in [[4., 12., 1.], [1., 2., 8.], [2., 8., 2.]] {
        let target = plane(4, dims);
        evaluate(&input(&source, &base, &target, &self::maps(2, 4), dims)).unwrap();
        let restored = evaluate(&input(&source, &base, &base, &maps, DIMS)).unwrap();
        assert_eq!(restored.asset, initial.asset);
    }
    for (&actual, &expected) in initial
        .asset
        .geometry
        .positions
        .iter()
        .zip(&source.geometry.positions)
    {
        close(actual, expected);
    }
}

#[test]
fn rotated_hierarchy_maps_global_origins_without_shearing_rotations() {
    let base = plane(2, DIMS);
    let source = rigged(&base);
    let dims = [4., 12., 1.];
    let target = plane(4, dims);
    let result = evaluate(&input(&source, &base, &target, &maps(2, 4), dims)).unwrap();
    let old = globals(source.rig.as_ref().unwrap());
    let new = globals(result.asset.rig.as_ref().unwrap());
    for ((old_t, old_r), (new_t, new_r)) in old.iter().zip(&new) {
        assert!((*new_t - *old_t * DVec3::new(2., 3., 0.5)).length() < 2e-6);
        assert!(new_r.dot(*old_r).abs() > 0.999999);
    }
    for (old, new) in source
        .rig
        .as_ref()
        .unwrap()
        .bones
        .iter()
        .zip(&result.asset.rig.as_ref().unwrap().bones)
    {
        assert_eq!(old.name, new.name);
        assert_eq!(old.parent, new.parent);
    }
}

#[test]
fn weights_are_barycentric_samples_of_the_canonical_field() {
    let base = plane(2, DIMS);
    let source = rigged(&base);
    let target = plane(4, DIMS);
    let output = evaluate(&input(&source, &base, &target, &maps(2, 4), DIMS)).unwrap();
    let weights = &output.asset.rig.unwrap().weights;
    assert_eq!(weights[5].joints, [0, 1, 0, 0]);
    assert_eq!(weights[5].weights, [0.5, 0.5, 0., 0.]);
    assert_eq!(weights[12].weights, [1., 0., 0., 0.]);
    assert_eq!(weights[12].joints[0], 1);
}

#[test]
fn absolute_clips_preserve_rest_relative_translation_and_rotation() {
    let base = plane(2, DIMS);
    let source = rigged(&base);
    let dims = [4., 12., 1.];
    let target = plane(4, dims);
    let output = evaluate(&input(&source, &base, &target, &maps(2, 4), dims)).unwrap();
    assert_eq!(output.report.adapted_keyframes, 6);
    for (old_track, new_track) in source.clips[0]
        .tracks
        .iter()
        .zip(&output.asset.clips[0].tracks)
    {
        let i = old_track.bone as usize;
        let old_bone = &source.rig.as_ref().unwrap().bones[i];
        let new_bone = &output.asset.rig.as_ref().unwrap().bones[i];
        for (old, new) in old_track.keys.iter().zip(&new_track.keys) {
            let old_delta = Quat::from_array(old_bone.rotation).conjugate()
                * (Vec3::from(old.translation) - Vec3::from(old_bone.translation));
            let new_delta = Quat::from_array(new_bone.rotation).conjugate()
                * (Vec3::from(new.translation) - Vec3::from(new_bone.translation));
            close(new_delta.to_array(), old_delta.to_array());
            rotation_close(
                (Quat::from_array(old_bone.rotation).conjugate() * Quat::from_array(old.rotation))
                    .to_array(),
                (Quat::from_array(new_bone.rotation).conjugate() * Quat::from_array(new.rotation))
                    .to_array(),
            );
            assert_eq!(new.tick, old.tick);
        }
    }
    // Rest key at tick0 must reproduce the entire new detailed geometry via LBS.
    let rest_pose = pose(output.asset.clone(), 0);
    for (&actual, &expected) in rest_pose
        .positions
        .iter()
        .zip(&output.asset.geometry.positions)
    {
        close(actual, expected);
    }
    let bent = pose(output.asset, 10);
    assert!(
        bent.positions
            .iter()
            .zip(&rest_pose.positions)
            .any(|(a, b)| (Vec3::from(*a) - Vec3::from(*b)).length() > 0.1)
    );
}

#[test]
fn proportional_animation_translation_is_scaled_in_object_coordinates() {
    let base = plane(2, DIMS);
    let source = rigged(&base);
    let dims = [4., 12., 1.];
    let target = plane(4, dims);
    let maps = maps(2, 4);
    let mut request = input(&source, &base, &target, &maps, dims);
    request.animation_translation_policy = Policy::Proportional;
    let output = evaluate(&request).unwrap();
    let global_rotations = globals(source.rig.as_ref().unwrap());
    for track in &source.clips[0].tracks {
        let i = track.bone as usize;
        let old_bone = &source.rig.as_ref().unwrap().bones[i];
        let new_bone = &output.asset.rig.as_ref().unwrap().bones[i];
        let old = &track.keys[1];
        let new = &output.asset.clips[0].tracks[i].keys[1];
        let q = DQuat::from_array(old_bone.rotation.map(f64::from));
        let d = |p: [f32; 3]| DVec3::from_array(p.map(f64::from));
        let expected = global_rotations[i].1
            * (q.conjugate() * (d(old.translation) - d(old_bone.translation)))
            * DVec3::new(2., 3., 0.5);
        let actual = global_rotations[i].1
            * (q.conjugate() * (d(new.translation) - d(new_bone.translation)));
        assert!((actual - expected).length() < 2e-6);
    }
}

#[test]
fn missing_or_wrong_surface_samples_fail_without_changing_source() {
    let base = plane(2, DIMS);
    let source = rigged(&base);
    let original = source.clone();
    let mut maps = maps(2, 2);
    maps[0].triangle = 999;
    assert!(
        evaluate(&input(&source, &base, &base, &maps, DIMS))
            .unwrap_err()
            .message
            .contains("missing triangle")
    );
    maps[0] = maps[8];
    assert!(
        evaluate(&input(&source, &base, &base, &maps, DIMS))
            .unwrap_err()
            .message
            .contains("material position")
    );
    assert_eq!(source, original);
}

#[test]
fn matching_vertex_count_is_not_topology_or_position_compatibility() {
    let base = plane(2, DIMS);
    let mut source = asset(&base);
    Arc::make_mut(&mut source.geometry).triangles.swap(0, 1);
    assert!(evaluate(&input(&source, &base, &base, &maps(2, 2), DIMS)).is_err());
    let source = asset(&base);
    let mut target = base.clone();
    target.positions[4][0] = 0.1;
    assert!(evaluate(&input(&source, &base, &target, &maps(2, 2), DIMS)).is_err());
}

#[test]
fn invalid_barycentric_and_dimension_inputs_fail() {
    let base = plane(2, DIMS);
    let source = asset(&base);
    let mut maps = maps(2, 2);
    for bary in [
        [f32::NAN, 0., 1.],
        [-0.1, 0., 1.1],
        [0., 0., 0.],
        [0.5, 0.5, 0.5],
    ] {
        maps[0].barycentric = bary;
        assert!(evaluate(&input(&source, &base, &base, &maps, DIMS)).is_err());
    }
    let maps = self::maps(2, 2);
    for dims in [
        [0., 1., 1.],
        [-1., 1., 1.],
        [f32::INFINITY, 1., 1.],
        [f32::NAN, 1., 1.],
    ] {
        assert!(evaluate(&input(&source, &base, &base, &maps, dims)).is_err());
        let mut request = input(&source, &base, &base, &maps, DIMS);
        request.reference_dimensions = dims;
        assert!(evaluate(&request).is_err());
    }
    assert!(evaluate(&input(&source, &base, &base, &maps[..8], DIMS)).is_err());
}

#[test]
fn input_output_validation_and_material_bounds_are_enforced() {
    let base = plane(2, DIMS);
    let mut source = asset(&base);
    Arc::make_mut(&mut source.geometry).positions[0][0] = f32::NAN;
    assert!(evaluate(&input(&source, &base, &base, &maps(2, 2), DIMS)).is_err());
    let source = asset(&base);
    let mut request = input(&source, &base, &base, &[], DIMS);
    let oversized = meshes::Geometry {
        material_coordinates: None,
        positions: vec![[0.; 3]; meshes::MAX_VERTICES + 1],
        triangles: vec![[0, 1, 2]],
    };
    request.target_base = &oversized;
    assert!(evaluate(&request).is_err());
    let mut moved = base.clone();
    for p in &mut moved.positions {
        p[0] += 10.;
    }
    let source = meshes::Asset {
        geometry: Arc::new(moved.clone()),
        ..source
    };
    assert!(evaluate(&input(&source, &moved, &moved, &maps(2, 2), DIMS)).is_err());
}

#[test]
fn extended_weight_influences_are_transferred_without_discarding() {
    let base = plane(2, DIMS);
    let mut source = rigged(&base);
    let rig = Arc::make_mut(source.rig.as_mut().unwrap());
    for i in 3..6 {
        rig.bones.push(meshes::Bone {
            name: format!("bone{i}"),
            parent: None,
            translation: [0.; 3],
            rotation: Quat::IDENTITY.to_array(),
        });
    }
    rig.weights[0] = meshes::Weight {
        extra: None,
        joints: [0, 1, 0, 0],
        weights: [0.5, 0.5, 0., 0.],
    };
    rig.weights[1] = meshes::Weight {
        extra: None,
        joints: [2, 3, 0, 0],
        weights: [0.5, 0.5, 0., 0.],
    };
    rig.weights[3] = meshes::Weight {
        extra: None,
        joints: [4, 5, 0, 0],
        weights: [0.5, 0.5, 0., 0.],
    };
    let original = source.clone();
    let target = plane(6, DIMS);
    let result = evaluate(&input(&source, &base, &target, &maps(2, 6), DIMS)).unwrap();
    assert!(
        result
            .asset
            .rig
            .as_ref()
            .unwrap()
            .weights
            .iter()
            .any(|w| w.extra.is_some())
    );
    assert_eq!(source, original);
}

#[test]
fn invalid_adapted_clip_is_atomic_even_after_geometry_transfer() {
    let base = plane(2, DIMS);
    let mut source = rigged(&base);
    let rig = Arc::make_mut(source.rig.as_mut().unwrap());
    rig.bones[0].translation = [0., 9000., 0.];
    let clip = Arc::make_mut(&mut source.clips[0]);
    clip.tracks[0].keys[0].translation = [0., 9000., 0.];
    clip.tracks[0].keys[1].translation = [0., 9500., 0.];
    let original = source.clone();
    let dims = [2., 8., 2.];
    let target = plane(2, dims);
    assert!(evaluate(&input(&source, &base, &target, &maps(2, 2), dims)).is_err());
    assert_eq!(source, original);
}

#[test]
fn local_fold_is_rejected_and_last_valid_asset_can_be_retained() {
    let base = plane(2, DIMS);
    let source = asset(&base);
    let maps = maps(2, 2);
    let valid = evaluate(&input(&source, &base, &base, &maps, DIMS)).unwrap();
    let last = valid.asset.clone();
    let mut folded = source.clone();
    Arc::make_mut(&mut folded.geometry).positions[4][0] = 10.;
    assert!(
        evaluate(&input(&folded, &base, &base, &maps, DIMS))
            .unwrap_err()
            .message
            .contains("folds")
    );
    assert_eq!(last, valid.asset);
}

#[test]
fn policies_and_report_have_strict_serializable_contracts() {
    assert_eq!(
        serde_json::to_string(&Policy::Physical).unwrap(),
        "\"physical\""
    );
    assert_eq!(
        serde_json::to_string(&Policy::Proportional).unwrap(),
        "\"proportional\""
    );
    assert!(
        serde_json::from_str::<Correspondence>(
            r#"{"triangle":0,"barycentric":[1,0,0],"guess":true}"#
        )
        .is_err()
    );
    let base = plane(2, DIMS);
    let source = asset(&base);
    let result = evaluate(&input(&source, &base, &base, &maps(2, 2), DIMS)).unwrap();
    let report: mesh_preservation::Report =
        serde_json::from_str(&serde_json::to_string(&result.report).unwrap()).unwrap();
    assert_eq!(report, result.report);
}

#[test]
fn explicit_material_domain_validates_provenance_and_maps_rigid_bind_origins() {
    use mesh_preservation::{MaterialDomain, evaluate_material};
    let base = plane(2, DIMS);
    let source = rigged(&base);
    let mappings = maps(2, 2);
    let material: Vec<_> = base
        .positions
        .iter()
        .map(|p| std::array::from_fn(|k| p[k] / DIMS[k]))
        .collect();
    let origins = [[0., -1.5, 0.], [0.1, 0.25, 0.], [0.2, 1.5, 0.]];
    let domain = MaterialDomain {
        reference: &material,
        target: &material,
        bone_origins: &origins,
    };
    let result =
        evaluate_material(&input(&source, &base, &base, &mappings, DIMS), &domain).unwrap();
    let rig = result.asset.rig.as_ref().unwrap();
    let mut globals = Vec::new();
    let mut rotations = Vec::new();
    for (i, b) in rig.bones.iter().enumerate() {
        let local = Vec3::from(b.translation);
        let rotation = Quat::from_array(b.rotation);
        let (p, r) = if let Some(parent) = b.parent {
            (
                globals[parent as usize] + rotations[parent as usize] * local,
                rotations[parent as usize] * rotation,
            )
        } else {
            (local, rotation)
        };
        globals.push(p);
        rotations.push(r);
        close(p.to_array(), origins[i]);
    }
    let mut wrong = material.clone();
    wrong[4][0] += 0.1;
    assert!(
        evaluate_material(
            &input(&source, &base, &base, &mappings, DIMS),
            &MaterialDomain {
                reference: &material,
                target: &wrong,
                bone_origins: &origins
            }
        )
        .is_err()
    );
    assert!(
        evaluate_material(
            &input(&source, &base, &base, &mappings, DIMS),
            &MaterialDomain {
                reference: &material[..2],
                target: &material,
                bone_origins: &origins
            }
        )
        .is_err()
    );
    assert!(
        evaluate_material(
            &input(&source, &base, &base, &mappings, DIMS),
            &MaterialDomain {
                reference: &material,
                target: &material,
                bone_origins: &origins[..2]
            }
        )
        .is_err()
    );
    let invalid = [[f32::NAN; 3]; 3];
    assert!(
        evaluate_material(
            &input(&source, &base, &base, &mappings, DIMS),
            &MaterialDomain {
                reference: &material,
                target: &material,
                bone_origins: &invalid
            }
        )
        .is_err()
    );
}
