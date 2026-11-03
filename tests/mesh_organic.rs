use ignis::mesh_organic::{Collar, Fusion, Options, fused_junction, junction};
use ignis::mesh_topology::Topology;
use std::collections::BTreeSet;

fn collars() -> [Collar; 3] {
    let s = 0.5f32.sqrt();
    let normals = [[0., -1., 0.], [-s, s, 0.], [s, s, 0.]];
    std::array::from_fn(|i| Collar {
        id: format!("collar{i}"),
        center: normals[i].map(|x| x * 1.2),
        normal: normals[i],
        radial: [0., 0., 1.],
        radius: [0.45, 0.35, 0.3][i],
        samples: 8,
    })
}
fn raw() -> Options {
    Options {
        refinement_levels: 0,
        relaxation_steps: 0,
        ..Default::default()
    }
}

#[test]
fn y_is_one_oriented_manifold_surface_with_three_exact_openings() {
    let rings = collars();
    let out = junction(&rings, raw()).unwrap();
    assert_eq!(out.collar_vertices, 24);
    assert_eq!(out.report.euler_characteristic, -1);
    assert_eq!(out.report.boundary_edges, 24);
    assert!(out.report.initial_volume > 0.);
    let t = Topology::inspect(&out.geometry).unwrap();
    assert!(t.isolated_vertices.is_empty());
    assert_eq!(t.non_manifold_edges, 0);
    let mut expected = BTreeSet::new();
    for b in &out.boundaries {
        for i in 0..b.vertices.len() {
            expected.insert([b.vertices[(i + 1) % b.vertices.len()], b.vertices[i]]);
        }
    }
    let mut actual = BTreeSet::new();
    for e in &t.edges {
        let [a, b] = e.vertices;
        let oriented: Vec<_> = e
            .faces
            .iter()
            .map(|&f| {
                let tri = out.geometry.triangles[f as usize];
                if (0..3).any(|i| tri[i] == a && tri[(i + 1) % 3] == b) {
                    [a, b]
                } else {
                    [b, a]
                }
            })
            .collect();
        if oriented.len() == 1 {
            actual.insert(oriented[0]);
        } else {
            assert_ne!(oriented[0], oriented[1]);
        }
    }
    assert_eq!(actual, expected);
    assert_eq!(
        out.geometry.positions.len() as i64 - t.edges.len() as i64
            + out.geometry.triangles.len() as i64,
        -1
    );
}
#[test]
fn refinement_and_relaxation_keep_boundary_indices_positions_and_envelope() {
    let rings = collars();
    let base = junction(&rings, raw()).unwrap();
    for level in 0..=3 {
        let out = junction(
            &rings,
            Options {
                refinement_levels: level,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out.geometry.positions[..24], base.geometry.positions[..]);
        assert_eq!(out.report.boundary_edges, 24);
        assert!(out.report.volume_loss >= -1e-5 && out.report.volume_loss <= 0.15 + 1e-6);
        assert_eq!(
            out.boundaries
                .iter()
                .map(|b| &b.vertices)
                .collect::<Vec<_>>(),
            base.boundaries
                .iter()
                .map(|b| &b.vertices)
                .collect::<Vec<_>>()
        );
        out.geometry.validate().unwrap();
        if level > 0 {
            assert!(out.geometry.positions.len() > 24);
            assert!(out.report.max_interior_movement > 0.);
        }
    }
}
#[test]
fn winding_frames_counts_and_transformations_remain_deterministic() {
    let rings = collars();
    let a = junction(&rings, Options::default()).unwrap();
    assert_eq!(
        a.geometry,
        junction(&rings, Options::default()).unwrap().geometry
    );
    let mut changed = rings.clone();
    for (i, c) in changed.iter_mut().enumerate() {
        c.samples = [5, 9, 12][i];
    }
    let out = junction(&changed, Options::default()).unwrap();
    assert_eq!(out.collar_vertices, 26);
    assert_eq!(out.report.boundary_edges, 26);
    let mut moved = rings.clone();
    let q = glam::Quat::from_rotation_y(0.3) * glam::Quat::from_rotation_z(0.2);
    for c in &mut moved {
        c.center = (q * glam::Vec3::from(c.center) + glam::Vec3::new(10., -4., 3.)).to_array();
        c.normal = (q * glam::Vec3::from(c.normal)).to_array();
        c.radial = (q * glam::Vec3::from(c.radial)).to_array();
    }
    let out = junction(&moved, Options::default()).unwrap();
    assert!(out.report.final_volume > 0.);
    assert!((out.report.final_volume - a.report.final_volume).abs() < 1e-4);
}
#[test]
fn invalid_frames_collapsed_overlapping_tangent_and_acute_collars_reject_without_mutation() {
    let original = collars();
    let mut cases = vec![];
    let mut c = original.clone();
    c[1].id = c[0].id.clone();
    cases.push(c);
    let mut c = original.clone();
    c[0].normal = [0.; 3];
    cases.push(c);
    let mut c = original.clone();
    c[0].radial = c[0].normal;
    cases.push(c);
    let mut c = original.clone();
    c[0].center[0] = f32::NAN;
    cases.push(c);
    let mut c = original.clone();
    c[0].radius = 0.;
    cases.push(c);
    let mut c = original.clone();
    c[0].samples = 2;
    cases.push(c);
    let mut c = original.clone();
    c[1].center = c[2].center;
    c[1].normal = c[2].normal;
    cases.push(c);
    let mut c = original.clone();
    for r in &mut c {
        r.normal = [0., 1., 0.];
        r.center[1] = 0.;
    }
    cases.push(c);
    let mut c = original.clone();
    c[1].normal = c[2].normal;
    cases.push(c);
    let mut c = original.clone();
    c[1].center = [-0.01, 0.02, 0.];
    c[2].center = [0.01, 0.02, 0.];
    cases.push(c);
    let mut c = original.clone();
    for r in &mut c {
        r.center[0] += 1e6;
    }
    cases.push(c);
    for c in cases {
        let before = c.clone();
        assert!(junction(&c, Options::default()).is_err());
        for (a, b) in c.iter().zip(&before) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.radius, b.radius);
        }
        assert_eq!(original, collars());
    }
}
#[test]
fn bounded_options_and_excessive_shrink_reject_atomically() {
    let rings = collars();
    for options in [
        Options {
            refinement_levels: 4,
            ..Default::default()
        },
        Options {
            relaxation_steps: 9,
            ..Default::default()
        },
        Options {
            relaxation_factor: f32::NAN,
            ..Default::default()
        },
        Options {
            max_volume_loss: 1.,
            ..Default::default()
        },
    ] {
        assert!(junction(&rings, options).is_err());
    }
    assert!(
        junction(
            &rings,
            Options {
                refinement_levels: 2,
                relaxation_steps: 8,
                relaxation_factor: 0.5,
                max_volume_loss: 0.0
            }
        )
        .unwrap_err()
        .message
        .contains("volume loss")
    );
    assert_eq!(rings, collars());
}
#[test]
fn maximum_sampling_and_refinement_fit_existing_asset_limits() {
    let mut rings = collars();
    for c in &mut rings {
        c.samples = 32;
    }
    let out = junction(
        &rings,
        Options {
            refinement_levels: 3,
            relaxation_steps: 0,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.geometry.positions.len() <= 4096);
    assert!(out.geometry.triangles.len() <= 8192);
    assert_eq!(out.report.boundary_edges, 96);
}

#[test]
fn fused_form_preserves_both_guard_rings_and_fits_bounded_field() {
    fused_junction(&collars(), Options::default(), Fusion::default()).unwrap();
    let rings = collars();
    let fusion = Fusion::default();
    let options = Options {
        refinement_levels: 2,
        ..Default::default()
    };
    let out = fused_junction(&rings, options, fusion).unwrap();
    let raw = junction(&rings, raw()).unwrap();
    assert_eq!(out.geometry.positions[..24], raw.geometry.positions[..]);
    for b in &out.boundaries {
        for &i in &b.vertices {
            let index = i as usize;
            let c = &rings[index / 8];
            let expected = glam::DVec3::from_array(out.geometry.positions[index].map(f64::from))
                - glam::DVec3::from_array(c.normal.map(f64::from)).normalize()
                    * f64::from(fusion.guard_length);
            assert!(
                expected.distance(glam::DVec3::from_array(
                    out.geometry.positions[index + 24].map(f64::from)
                )) < 1e-6
            );
        }
    }
    let report = out.report.field.as_ref().unwrap();
    assert_eq!(report.fixed_vertices, 48);
    assert!(report.max_residual <= f64::from(fusion.tolerance));
    assert!(report.max_face_rotation_degrees <= f64::from(fusion.max_face_rotation_degrees));
    assert!(
        out.report.volume_loss > 0.05
            && out.report.volume_loss <= f64::from(fusion.max_volume_loss)
    );
    assert_eq!(out.report.boundary_edges, 24);
    assert_eq!(out.report.euler_characteristic, -1);
    assert_eq!(
        out.geometry,
        fused_junction(&rings, options, fusion).unwrap().geometry
    );
}
#[test]
fn fusion_envelopes_and_unresolved_inputs_reject_without_changing_collars() {
    let rings = collars();
    let options = Options {
        refinement_levels: 2,
        ..Default::default()
    };
    for fusion in [
        Fusion {
            center_radius: f32::NAN,
            ..Default::default()
        },
        Fusion {
            center: [0., 20., 0.],
            ..Default::default()
        },
        Fusion {
            guard_length: 1.,
            ..Default::default()
        },
        Fusion {
            max_displacement: 0.001,
            ..Default::default()
        },
        Fusion {
            max_volume_loss: 0.,
            ..Default::default()
        },
        Fusion {
            max_face_rotation_degrees: 0.1,
            ..Default::default()
        },
    ] {
        assert!(fused_junction(&rings, options, fusion).is_err());
        assert_eq!(rings, collars());
    }
    assert!(
        fused_junction(&rings, raw(), Fusion::default())
            .unwrap_err()
            .message
            .contains("refinement")
    );
}

#[test]
fn collar_transition_keeps_guards_exact_and_legacy_bit_identical() {
    let rings = collars();
    let options = Options {
        refinement_levels: 2,
        ..Default::default()
    };
    let legacy = fused_junction(&rings, options, Fusion::default()).unwrap();
    let explicit: Fusion =
        serde_json::from_value(serde_json::json!({"transition_lengths": [0,0,0]})).unwrap();
    assert_eq!(
        serde_json::to_value(&legacy).unwrap(),
        serde_json::to_value(fused_junction(&rings, options, explicit).unwrap()).unwrap()
    );
    let fusion = Fusion {
        transition_lengths: [0.4; 3],
        ..Default::default()
    };
    let result = fused_junction(&rings, options, fusion).unwrap();
    assert_eq!(result.geometry.triangles, legacy.geometry.triangles);
    assert_eq!(
        result.geometry.positions[..48],
        legacy.geometry.positions[..48]
    );
    assert_ne!(
        result.geometry.positions[48..],
        legacy.geometry.positions[48..]
    );
    assert_eq!(
        result.report.field.as_ref().unwrap().transition_lengths,
        Some([0.4; 3])
    );
    assert_eq!(result.report.boundary_edges, 24);
    assert_eq!(result.report.euler_characteristic, -1);
    assert!(result.report.field.unwrap().max_residual <= f64::from(fusion.tolerance));
    for lengths in [
        [-1.; 3],
        [f32::NAN; 3],
        [0., 0.1, 0.1],
        [0.001, 0.1, f32::INFINITY],
        [0.85; 3],
    ] {
        assert!(
            fused_junction(
                &rings,
                options,
                Fusion {
                    transition_lengths: lengths,
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
    assert_eq!(rings, collars());
}

#[test]
fn preview_dispatcher_is_pure_during_runtime_and_does_not_enter_history() {
    use ignis::{
        engine::{Command, Engine},
        meshes,
        model::RoomRecipe,
    };
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    e.execute(
        Command::EditMesh {
            edit: meshes::Edit::Grid {
                id: "independent".into(),
                segments: 1,
                size: 1.,
            },
        }
        .into(),
    )
    .unwrap();
    let authored = e.scene.clone();
    e.execute(Command::Step { ticks: 2 }.into()).unwrap();
    e.execute(Command::Play.into()).unwrap();
    let before = e.execute(Command::Inspect.into()).unwrap();
    let reply = e
        .execute(
            Command::PreviewMeshJunction {
                collars: Box::new(collars()),
                options: Options {
                    refinement_levels: 2,
                    ..Default::default()
                },
                fusion: Some(Fusion::default()),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(reply["report"]["euler_characteristic"], -1);
    assert!(reply["geometry"]["positions"].as_array().unwrap().len() > 48);
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    let mut bad = collars();
    bad[0].radius = 0.;
    assert!(
        e.execute(
            Command::PreviewMeshJunction {
                collars: Box::new(bad),
                options: Options::default(),
                fusion: None
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.execute(Command::Inspect.into()).unwrap(), before);
    e.execute(Command::Undo.into()).unwrap();
    assert!(e.scene.meshes.is_empty());
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, authored);
}
#[test]
fn preview_json_schema_defaults_and_unknown_fields_are_explicit() {
    use ignis::engine::{Command, Request};
    let value = serde_json::json!({"command":{"op":"preview_mesh_junction","collars":collars()}});
    let request: Request = serde_json::from_value(value.clone()).unwrap();
    let Command::PreviewMeshJunction {
        options, fusion, ..
    } = request.command
    else {
        panic!("preview")
    };
    assert_eq!(options.refinement_levels, 1);
    assert!(fusion.is_none());
    let mut bad = value.clone();
    bad["command"]["options"] = serde_json::json!({"unsupported":true});
    assert!(serde_json::from_value::<Request>(bad).is_err());
    let mut bad = value;
    bad["command"]["collars"][0]["automatic_weld"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Request>(bad).is_err());
    assert!(
        serde_json::to_string(&schemars::schema_for!(Command))
            .unwrap()
            .contains("preview_mesh_junction")
    );
}
