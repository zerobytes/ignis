use glam::{DQuat, DVec3};
use ignis::{
    engine::{Command, Engine},
    mesh_preservation::DisplacementPolicy,
    mesh_topology::Topology,
    meshes,
    model::RoomRecipe,
    plants::{Definition, Edit, Trajectory},
};
use std::sync::Arc;

fn drawing() -> Definition {
    serde_json::from_value(serde_json::json!({
        "id":"tube","profile":{"id":"profile","width":0.04,"height":0.3},"depth":0.04,
        "trajectory":{"kind":"cubic_bezier","id":"axis","control_points_xz":[[0,0],[0,0],[-0.1,0],[-0.1,0]],"surface_tolerance":0.0002},
        "sections":{"id":"cuts","cross_section":"ellipse","stations":[
            {"id":"start","at":0,"width":1,"depth":1},{"id":"end","at":1,"width":1,"depth":1}]},
        "segments":[12,32,12],"sampling_tolerance":0.001
    })).unwrap()
}
fn controls(d: &mut Definition) -> &mut [[f32; 2]; 4] {
    let Trajectory::CubicBezier {
        control_points_xz, ..
    } = d.trajectory.as_mut().unwrap();
    control_points_xz
}
fn tolerance(d: &mut Definition, t: f32) {
    let Trajectory::CubicBezier {
        surface_tolerance, ..
    } = d.trajectory.as_mut().unwrap();
    *surface_tolerance = t;
}
fn put(d: Definition) -> Command {
    Command::EditPlant {
        edit: Edit::Put {
            definition: d,
            detail_policy: DisplacementPolicy::Physical,
            animation_translation_policy: DisplacementPolicy::Physical,
        },
    }
}
fn engine(d: Definition) -> Engine {
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    e.execute(put(d).into()).unwrap();
    e
}

#[test]
fn sampling_query_exposes_base_domain_without_mutating_plant() {
    let d = drawing();
    let mut e = engine(d.clone());
    let original = e.execute(Command::Inspect.into()).unwrap();
    let sampling = e
        .execute(Command::InspectPlantSampling { id: d.id.clone() }.into())
        .unwrap();
    assert_eq!(sampling["revision"], original["revision"]);
    let positions: Vec<[f32; 3]> =
        serde_json::from_value(sampling["sampling"]["geometry"]["positions"].clone()).unwrap();
    let material: Vec<[f32; 3]> =
        serde_json::from_value(sampling["sampling"]["material"].clone()).unwrap();
    assert_eq!(positions.len(), material.len());
    for (&p, m) in positions.iter().zip(material) {
        let expected = DVec3::from_array(d.position_at(m.map(f64::from)));
        assert!(DVec3::from_array(p.map(f64::from)).distance(expected) < 1e-7);
    }
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), original);
    assert!(
        e.execute(
            Command::InspectPlantSampling {
                id: "missing".into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), original);
}

#[test]
fn constant_circle_is_normal_to_independently_differentiated_s_curve_and_roundtrips() {
    let d = drawing();
    d.validate().unwrap();
    let height = f64::from(d.profile.height);
    let shift = f64::from(controls(&mut d.clone())[3][0]);
    for t in [0., 0.125, 0.5, 0.875, 1.] {
        let center = DVec3::new(
            shift * (3. * t * t - 2. * t * t * t),
            height * (t - 0.5),
            0.,
        );
        let tangent = DVec3::new(shift * 6. * t * (1. - t), height, 0.).normalize();
        for j in 0..56 {
            let angle = j as f64 * std::f64::consts::TAU / 56.;
            let (s, c) = angle.sin_cos();
            let scale = 0.5 / s.abs().max(c.abs());
            let material = [c * scale, t - 0.5, s * scale];
            let p = DVec3::from_array(d.position_at(material));
            let radial = p - center;
            assert!(radial.dot(tangent).abs() < 1e-14);
            assert!((radial.length() - f64::from(d.profile.width) * 0.5).abs() < 1e-14);
            let inverse = d.material_at(p.to_array()).unwrap();
            assert!(
                material
                    .into_iter()
                    .zip(inverse)
                    .all(|(a, b)| (a - b).abs() < 2e-12)
            );
        }
    }
}

#[test]
fn three_dimensional_frames_keep_circle_and_inverse_for_all_section_shapes() {
    let mut d = drawing();
    for (p, z) in controls(&mut d).iter_mut().zip([0., 0.005, 0.01, 0.02]) {
        p[1] = z;
    }
    let mut middle = d.sections.as_ref().unwrap().stations[0].clone();
    middle.id = "middle".into();
    middle.at = 0.35;
    middle.width = 0.85;
    middle.depth = 0.9;
    middle.offset = [0.001, 0.001];
    d.sections.as_mut().unwrap().stations.insert(1, middle);
    for shape in [
        ignis::plants::CrossSection::Ellipse,
        ignis::plants::CrossSection::Rectangle,
        ignis::plants::CrossSection::Diamond,
    ] {
        d.sections.as_mut().unwrap().cross_section = shape;
        d.validate().unwrap();
        for y in [-0.5, -0.23, 0., 0.33, 0.5] {
            for x in [-0.5, -0.1, 0.2, 0.5] {
                for z in [-0.5, 0.1, 0.5] {
                    let m = [x, y, z];
                    let p = d.position_at(m);
                    let back = d.material_at(p).unwrap();
                    assert!(m.into_iter().zip(back).all(|(a, b)| (a - b).abs() < 3e-12));
                }
            }
        }
        // Moving/scaling a section stays within its normal plane and keeps
        // inversion deterministic, including the bounded cubic section channels.
        let s = d.sections.as_mut().unwrap();
        s.interpolation = ignis::plants::SectionInterpolation::Pchip;
        s.stations[0].offset = [0.001, -0.001];
        s.stations[1].width = 0.8;
        s.stations[1].depth = 0.9;
    }
}

#[test]
fn sampling_reports_error_and_rejects_coarse_mesh_without_increasing_budgets() {
    let mut d = drawing();
    d.segments = [14, 63, 14];
    let sample = d.sample().unwrap();
    let report = sample.trajectory_report.as_ref().unwrap();
    assert!(report.sampled_surface_error > 0. && report.sampled_surface_error < 0.0002);
    assert!(report.surface_probes > 20000 && report.inverse_derivative_margin > 0.);
    assert_eq!(sample.geometry.positions.len(), 3922);
    assert_eq!(sample.geometry.triangles.len(), 7840);
    let t = Topology::inspect(&sample.geometry).unwrap();
    assert_eq!(t.boundary_edges, 0);
    assert_eq!(t.non_manifold_edges, 0);
    assert_eq!(
        sample.geometry.positions.len() as i64 - t.edges.len() as i64
            + sample.geometry.triangles.len() as i64,
        2
    );
    d.segments = [2, 2, 2];
    assert!(d.sample().unwrap_err().message.contains("surface error"));
    tolerance(&mut d, 0.02);
    assert!(
        d.sample()
            .unwrap()
            .trajectory_report
            .unwrap()
            .sampled_surface_error
            > 0.0002
    );
}

#[test]
fn conservative_inverse_rejects_fold_risk_and_points_outside_the_volume() {
    let d = drawing();
    let mut bad = d.clone();
    bad.profile.height = 0.1;
    assert!(
        bad.validate()
            .unwrap_err()
            .message
            .contains("unique inverse")
    );
    for value in [0., f32::NAN, 2.] {
        let mut bad = d.clone();
        tolerance(&mut bad, value);
        assert!(bad.validate().is_err());
    }
    let mut bad = d.clone();
    controls(&mut bad)[1][0] = f32::INFINITY;
    assert!(bad.validate().is_err());
    for p in [
        [0., -0.2, 0.],
        [-0.1, 0.2, 0.],
        [0., 0., 0.1],
        [f64::NAN, 0., 0.],
    ] {
        assert!(d.material_at(p).is_err());
    }
    let mut straight = d.clone();
    *controls(&mut straight) = [[0., 0.]; 4];
    let mut legacy = straight.clone();
    legacy.trajectory = None;
    assert_eq!(
        straight.sample().unwrap().geometry,
        legacy.sample().unwrap().geometry
    );
    assert!(legacy.sample().unwrap().trajectory_report.is_none());
    assert!(
        serde_json::to_value(legacy)
            .unwrap()
            .get("trajectory")
            .is_none()
    );
}

#[test]
fn trajectory_edits_keep_canonical_detail_channels_rig_weights_and_animation() {
    let mut straight = drawing();
    straight.segments = [4, 16, 4];
    tolerance(&mut straight, 0.002);
    *controls(&mut straight) = [[0., 0.]; 4];
    let mut e = engine(straight.clone());
    let mut source = e.scene.meshes.asset("tube").unwrap().clone();
    let base = source.geometry.positions.clone();
    let index = base
        .iter()
        .position(|p| p[2] > 0.019 && p[1].abs() < 0.01)
        .unwrap();
    let parent_rotation = DQuat::from_rotation_y(0.35);
    let root = DVec3::new(0., -0.12, 0.);
    let child = DVec3::new(0.004, 0.015, 0.003);
    let json = serde_json::json!({"bones":[
        {"name":"root","parent":null,"translation":root.to_array(),"rotation":parent_rotation.to_array()},
        {"name":"tip","parent":0,"translation":(parent_rotation.inverse()*(child-root)).to_array(),"rotation":[0,0,0,1]}],
        "weights":base.iter().map(|_|serde_json::json!({"joints":[0,1,0,0],"weights":[0.25,0.75,0,0]})).collect::<Vec<_>>()});
    source.rig = Some(Arc::new(serde_json::from_value(json).unwrap()));
    source.clips=vec![Arc::new(serde_json::from_value(serde_json::json!({"id":"bend","duration_ticks":60,"tracks":[
        {"bone":1,"keys":[{"tick":0,"translation":source.rig.as_ref().unwrap().bones[1].translation,"rotation":[0,0,0,1]},
        {"tick":60,"translation":source.rig.as_ref().unwrap().bones[1].translation,"rotation":[0.1,0,0,0.9949874]}]}]})).unwrap())];
    let stroke=serde_json::from_value(serde_json::json!({"kind":"grab","centers":[base[index]],"radius":0.003,"strength":0.0007,"direction":[0,0,1]})).unwrap();
    for edit in [
        meshes::Edit::Sculpt {
            asset: "tube".into(),
            stroke,
        },
        meshes::Edit::SetMaterialCoordinates {
            asset: "tube".into(),
            coordinates: Some(base.clone()),
        },
        meshes::Edit::SetRig {
            asset: "tube".into(),
            rig: source.rig,
        },
        meshes::Edit::PutClip {
            asset: "tube".into(),
            clip: source.clips.remove(0),
        },
    ] {
        e.execute(Command::EditMesh { edit }.into()).unwrap();
    }
    let canonical = e.scene.plants.entry("tube").unwrap().clone();
    let mut bent = straight.clone();
    *controls(&mut bent) = [[0., 0.], [0., 0.], [-0.1, 0.], [-0.1, 0.]];
    e.execute(put(bent.clone()).into()).unwrap();
    let entry = e.scene.plants.entry("tube").unwrap();
    assert_eq!(entry.source, canonical.source);
    assert_eq!(entry.reference, canonical.reference);
    let result = e.scene.meshes.asset("tube").unwrap();
    let base = bent.sample().unwrap();
    assert!(
        (f64::from(result.geometry.positions[index][2] - base.geometry.positions[index][2])
            - 0.0007)
            .abs()
            < 1e-7
    );
    assert_eq!(
        result.geometry.material_coordinates,
        canonical.source.geometry.material_coordinates
    );
    assert_eq!(
        result.rig.as_ref().unwrap().weights,
        canonical.source.rig.as_ref().unwrap().weights
    );
    let rig = result.rig.as_ref().unwrap();
    let moved_root = DVec3::from_array(rig.bones[0].translation.map(f64::from));
    let moved_child =
        moved_root + parent_rotation * DVec3::from_array(rig.bones[1].translation.map(f64::from));
    let expected = bent.position_at(straight.material_at(child.to_array()).unwrap());
    assert!(moved_child.distance(DVec3::from_array(expected)) < 1e-7);
    assert_eq!(
        rig.bones[0].rotation,
        canonical.source.rig.as_ref().unwrap().bones[0].rotation
    );
    assert_eq!(
        result.clips[0].tracks[0].keys[0].translation,
        rig.bones[1].translation
    );
    let expected_bind = rig.bones[1].translation;
    let mut dense = bent.clone();
    dense.segments = [6, 24, 6];
    e.execute(put(dense).into()).unwrap();
    assert_eq!(
        e.scene.plants.entry("tube").unwrap().source,
        canonical.source
    );
    let refined = e.scene.meshes.asset("tube").unwrap();
    assert!(refined.geometry.positions.len() > canonical.source.geometry.positions.len());
    assert_eq!(
        refined.rig.as_ref().unwrap().bones[1].translation,
        expected_bind
    );
    assert_eq!(
        refined
            .geometry
            .material_coordinates
            .as_ref()
            .unwrap()
            .len(),
        refined.geometry.positions.len()
    );
    assert!(
        refined
            .rig
            .as_ref()
            .unwrap()
            .weights
            .iter()
            .all(|w| (w.weights.iter().sum::<f32>() - 1.).abs() < 1e-6)
    );
    e.execute(put(bent).into()).unwrap();
    e.execute(put(straight).into()).unwrap();
    assert_eq!(
        e.scene.plants.entry("tube").unwrap().source,
        canonical.source
    );
}

#[test]
fn trajectory_identity_version_and_invalid_edits_are_atomic() {
    let d = drawing();
    let mut e = engine(d.clone());
    let before = e.scene.clone();
    let mut changed = d.clone();
    let Trajectory::CubicBezier { id, .. } = changed.trajectory.as_mut().unwrap();
    *id = "different".into();
    assert!(e.execute(put(changed).into()).is_err());
    assert_eq!(&e.scene, &before);
    let mut bad = d.clone();
    bad.profile.height = 0.1;
    assert!(e.execute(put(bad).into()).is_err());
    assert_eq!(&e.scene, &before);
    let mut scene = before.clone();
    scene.version = 63;
    assert!(scene.validate().is_err());
    scene.version = 64;
    scene.validate().unwrap();
    assert!(
        serde_json::from_value::<Definition>(
            serde_json::json!({"id":"x","trajectory":{"kind":"free_spline"}})
        )
        .is_err()
    );
}

#[test]
fn explicit_axis_adoption_preserves_legacy_sculpt_channels_rig_and_canonical_reference() {
    let mut legacy = drawing();
    legacy.trajectory = None;
    legacy.segments = [4, 16, 4];
    let mut e = engine(legacy.clone());
    let base = e
        .scene
        .meshes
        .asset("tube")
        .unwrap()
        .geometry
        .positions
        .clone();
    let index = base
        .iter()
        .position(|p| p[1].abs() < 1e-6 && p[2] > 0.019)
        .unwrap();
    let rig = meshes::Rig {
        bones: vec![
            meshes::Bone {
                name: "root".into(),
                parent: None,
                translation: [0., -0.12, 0.],
                rotation: [0., 0., 0., 1.],
            },
            meshes::Bone {
                name: "tip".into(),
                parent: Some(0),
                translation: [0., 0.135, 0.],
                rotation: [0., 0., 0., 1.],
            },
        ],
        weights: vec![
            meshes::Weight {
                extra: None,
                joints: [0, 1, 0, 0],
                weights: [0.25, 0.75, 0., 0.]
            };
            base.len()
        ],
    };
    let stroke=serde_json::from_value(serde_json::json!({"kind":"grab","centers":[base[index]],"radius":0.003,"strength":0.0007,"direction":[0,0,1]})).unwrap();
    let clip: meshes::Clip = serde_json::from_value(
        serde_json::json!({"id":"bend","duration_ticks":60,"tracks":[{"bone":1,"keys":[
        {"tick":0,"translation":[0,0.135,0],"rotation":[0,0,0,1]},
        {"tick":60,"translation":[0,0.135,0],"rotation":[0.02,0,0,0.99979998]}]}]}),
    )
    .unwrap();
    for edit in [
        meshes::Edit::Sculpt {
            asset: "tube".into(),
            stroke,
        },
        meshes::Edit::SetMaterialCoordinates {
            asset: "tube".into(),
            coordinates: Some(base),
        },
        meshes::Edit::SetRig {
            asset: "tube".into(),
            rig: Some(Arc::new(rig)),
        },
        meshes::Edit::PutClip {
            asset: "tube".into(),
            clip: Arc::new(clip),
        },
    ] {
        e.execute(Command::EditMesh { edit }.into()).unwrap();
    }
    let canonical = e.scene.plants.entry("tube").unwrap().clone();
    let before = e.scene.clone();
    let mut target = drawing();
    target.segments = legacy.segments;
    tolerance(&mut target, 0.001);
    assert!(e.execute(put(target.clone()).into()).is_err());
    assert_eq!(e.scene, before);
    let edit = Edit::AdoptTrajectory {
        definition: target.clone(),
        detail_policy: DisplacementPolicy::Physical,
        animation_translation_policy: DisplacementPolicy::Physical,
    };
    let preview = e
        .execute(Command::PreviewPlant { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(e.scene, before);
    e.execute(Command::EditPlant { edit }.into()).unwrap();
    let adopted = e.scene.clone();
    assert_eq!(
        serde_json::to_value(&adopted.meshes.assets).unwrap(),
        preview["assets"]
    );
    let entry = e.scene.plants.entry("tube").unwrap();
    assert!(Arc::ptr_eq(&entry.source, &canonical.source));
    assert_eq!(entry.reference, canonical.reference);
    let actual = e.scene.meshes.asset("tube").unwrap();
    let sampled = target.sample().unwrap();
    assert!(
        (actual.geometry.positions[index][2] - sampled.geometry.positions[index][2] - 0.0007).abs()
            < 1e-7
    );
    assert_eq!(
        actual.geometry.material_coordinates,
        canonical.source.geometry.material_coordinates
    );
    assert_eq!(
        actual.rig.as_ref().unwrap().weights,
        canonical.source.rig.as_ref().unwrap().weights
    );
    assert_eq!(
        actual.clips[0].tracks[0].keys[0].translation,
        actual.rig.as_ref().unwrap().bones[1].translation
    );
    assert_eq!(
        actual.clips[0].tracks[0].keys[1].rotation,
        canonical.source.clips[0].tracks[0].keys[1].rotation
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, adopted);
    let mut old = adopted.clone();
    old.version = 64;
    assert!(old.validate().unwrap_err().message.contains("version65"));
    Engine::new(serde_json::from_slice(&serde_json::to_vec(&adopted).unwrap()).unwrap()).unwrap();
    assert!(e.execute(put(legacy).into()).is_err());
    assert_eq!(e.scene, adopted);
    let mut dense = target.clone();
    dense.segments = [6, 24, 6];
    e.execute(put(dense).into()).unwrap();
    e.execute(put(target.clone()).into()).unwrap();
    assert_eq!(e.scene, adopted);
    let mut straight = target.clone();
    *controls(&mut straight) = [[0.; 2]; 4];
    e.execute(put(straight).into()).unwrap();
    e.execute(put(target).into()).unwrap();
    assert_eq!(e.scene, adopted);
}
