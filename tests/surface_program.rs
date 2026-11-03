//! Source identity, domain geometry, conservative certificates and old-reader boundaries.
use glam::DVec3;
use ignis::{
    planet,
    terrain::{
        self,
        surface::{Domain, Knot, Node, Program},
    },
    world_space::{Coordinates, Position},
};
fn zero() -> Position {
    Position::normalize(Coordinates {
        cell: [0; 3],
        offset: [0.; 3],
    })
    .unwrap()
}
fn p(x: f64, z: f64) -> Position {
    zero().translated([x, 0., z]).unwrap()
}
fn sphere(center: Position, r: f64, x: f64, z: f64) -> Position {
    let d = x.hypot(z) / r;
    let unit = if d == 0. {
        DVec3::Y
    } else {
        DVec3::Y * d.cos() + DVec3::new(x, 0., z).normalize() * d.sin()
    };
    center.translated((unit * r).to_array()).unwrap()
}
fn source(nodes: Vec<Node>, output: u16) -> Program {
    Program::new(nodes, output).unwrap()
}
fn curve(points: Vec<Position>) -> Node {
    Node::Curve {
        points,
        width_meters: 100.,
        height_meters: 80.,
        left_exponent: 1.,
        right_exponent: 3.,
        roundness: 0.,
    }
}
fn program(center: Position, points: Vec<Position>) -> Program {
    source(
        vec![
            Node::Constant { value: 10. },
            curve(points),
            Node::Add { a: 0, b: 1 },
            Node::Raise {
                center,
                radius_meters: 50.,
                amount_meters: 1.,
            },
            Node::Constant { value: 25. },
            Node::Mix {
                a: 2,
                b: 4,
                mask: 3,
            },
        ],
        5,
    )
}
#[test]
fn metric_tools_share_the_program_across_plane_and_large_sphere() {
    let c = zero();
    let r = 6_371_000.;
    let flat = program(p(0., 0.), vec![p(-100., 0.), p(100., 0.)]);
    let globe = program(
        sphere(c, r, 0., 0.),
        vec![sphere(c, r, -100., 0.), sphere(c, r, 100., 0.)],
    );
    let a = flat.compile(Domain::Plane { datum: c }).unwrap();
    let b = globe
        .compile(Domain::Sphere {
            center: c,
            radius_meters: r,
        })
        .unwrap();
    for z in [-120., -75., -25., 0., 25., 75., 120.] {
        let expected = a.sample(p(0., z)).unwrap();
        let actual = b.sample(sphere(c, r, 0., z)).unwrap();
        assert!(
            (expected - actual).abs() < 1e-5,
            "z={z} flat={expected} globe={actual}"
        );
        assert!((globe.range_meters[0]..=globe.range_meters[1]).contains(&actual));
    }
    assert!(a.sample(p(0., -75.)).unwrap() > a.sample(p(0., 75.)).unwrap());
    assert!((a.sample(p(0., 0.)).unwrap() - 25.).abs() < 1e-12);
}
#[test]
fn hermite_overshoot_is_enclosed_and_invalid_graphs_fail_before_sampling() {
    let nodes = vec![
        Node::Constant { value: 0.5 },
        Node::Remap {
            input: 0,
            knots: vec![
                Knot {
                    x: 0.,
                    value: 0.,
                    derivative: 8.,
                },
                Knot {
                    x: 1.,
                    value: 0.,
                    derivative: -8.,
                },
            ],
        },
    ];
    let pr = source(nodes, 1);
    let value = pr
        .compile(Domain::Plane { datum: zero() })
        .unwrap()
        .sample(zero())
        .unwrap();
    assert_eq!(value, 2.);
    assert!(pr.range_meters[1] > 2.);
    let mut bad = pr.clone();
    bad.range_meters = [0., 1.];
    assert!(bad.validate().is_err());
    let mut bad = pr.clone();
    bad.nodes[0] = Node::Add { a: 1, b: 1 };
    assert!(bad.validate().is_err());
    let mut bad = pr.clone();
    bad.output = 128;
    assert!(bad.validate().is_err());
    let mut bad = pr.clone();
    bad.version = 2;
    assert!(bad.validate().is_err());
    let mut bad = pr.clone();
    bad.nodes[0] = Node::Constant { value: f64::NAN };
    assert!(bad.validate().is_err());
    let mut bad = pr.clone();
    if let Node::Remap { knots, .. } = &mut bad.nodes[1] {
        knots[1].x = 0.;
    }
    assert!(bad.validate().is_err());
    assert!(Program::new(vec![Node::Constant { value: 1. }; 129], 0).is_err());
    assert!(serde_json::from_value::<Program>(serde_json::json!({"version":1,"nodes":vec![serde_json::json!({"kind":"constant","value":0});129],"output":0,"range_meters":[-1,1]})).is_err());
}

#[test]
fn smooth_composition_accounts_for_height_created_at_junctions() {
    let p = source(
        vec![
            Node::Constant { value: 10. },
            Node::Constant { value: 10. },
            Node::SmoothMaximum {
                a: 0,
                b: 1,
                transition_meters: 8.,
            },
            Node::SmoothMinimum {
                a: 0,
                b: 1,
                transition_meters: 8.,
            },
            Node::Subtract { a: 2, b: 3 },
        ],
        4,
    );
    let h = p
        .compile(Domain::Plane { datum: zero() })
        .unwrap()
        .sample(zero())
        .unwrap();
    assert_eq!(h, 4.);
    assert!(p.range_meters[0] <= h && p.range_meters[1] >= h);
    let draft = terrain::surface::Draft {
        nodes: p.nodes.clone(),
        output: p.output,
    };
    let prepared = terrain::query(terrain::Query::PrepareSurface { draft }).unwrap();
    assert_eq!(prepared["program"], serde_json::to_value(p).unwrap());
}
#[test]
fn field_is_order_independent_precise_and_continuous_across_globe_faces_and_poles() {
    let pr = source(
        vec![Node::Noise {
            seed: 42,
            period_meters: 4096,
            octaves: 6,
            persistence: 0.5,
            amplitude_meters: 30.,
            style: terrain::NoiseKind::Ridged,
        }],
        0,
    );
    let huge = Position::normalize(Coordinates {
        cell: [9_007_199_254_740_993, 0, -9_007_199_254_740_993],
        offset: [1023.75, 0., 0.125],
    })
    .unwrap();
    let a = pr.compile(Domain::Plane { datum: zero() }).unwrap();
    let x = a.sample(huge).unwrap();
    let y = a.sample(huge.translated([0.01, 0., 0.]).unwrap()).unwrap();
    assert!((x - y).abs() < 0.01);
    assert_ne!(x, y);
    assert_eq!(x, a.sample(huge).unwrap());
    let r = 6_371_000.;
    let center = Position::normalize(Coordinates {
        cell: [i64::MAX - 100_000, 70, 0],
        offset: [0.125, 0., 0.],
    })
    .unwrap();
    let b = pr
        .compile(Domain::Sphere {
            center,
            radius_meters: r,
        })
        .unwrap();
    for d in [
        DVec3::Y,
        DVec3::NEG_Y,
        DVec3::new(1., 1., 1.).normalize(),
        DVec3::new(-1., 0., 1.).normalize(),
    ] {
        let point = center.translated((d * r).to_array()).unwrap();
        let h = b.sample(point).unwrap();
        let nearby = center
            .translated(((d + DVec3::X * 1e-10).normalize() * r).to_array())
            .unwrap();
        assert!((b.sample(nearby).unwrap() - h).abs() < 0.001);
        assert_eq!(b.sample(point).unwrap(), h);
    }
}

#[test]
fn submeter_tools_survive_cosine_rounding_on_enormous_reference_spheres() {
    let r = 5e11;
    let up = DVec3::ONE.normalize();
    let right = (DVec3::X - up * up.x).normalize();
    let position = |meters: f64| {
        zero()
            .translated(((up * (meters / r).cos() + right * (meters / r).sin()) * r).to_array())
            .unwrap()
    };
    let pr = source(
        vec![Node::Raise {
            center: position(0.),
            radius_meters: 0.125,
            amount_meters: 1.,
        }],
        0,
    );
    let prepared = pr
        .compile(Domain::Sphere {
            center: zero(),
            radius_meters: r,
        })
        .unwrap();
    let h = prepared.sample(position(0.05)).unwrap();
    assert!(
        h > 0.3 && h < 0.99,
        "small support must not disappear after cosine rounds to one: {h}"
    );
    assert_eq!(prepared.sample(position(1.)).unwrap(), 0.);
}
#[test]
fn new_source_codecs_and_actual_native_samplers_use_the_same_program() {
    let pr = source(
        vec![
            Node::Constant { value: 5. },
            Node::Raise {
                center: zero(),
                radius_meters: 20.,
                amount_meters: 7.,
            },
            Node::Add { a: 0, b: 1 },
        ],
        2,
    );
    let mut flat:terrain::Document=serde_json::from_value(serde_json::json!({"version":3,"space_version":1,"id":"shared","revision":1,"datum":{"cell":["0","0","0"],"offset":[0,0,0]},"base":{"kind":"constant","height_meters":0},"operations":[{"kind":"surface","program":pr}]})).unwrap();
    assert_eq!(
        terrain::Sampler::new(&flat)
            .unwrap()
            .sample(zero())
            .unwrap(),
        Some(12.)
    );
    assert_eq!(
        terrain::decode(&terrain::encode(&flat).unwrap()).unwrap(),
        flat
    );
    flat.version = 2;
    assert!(flat.validate().is_err());
    flat.version = 3;
    let r = 6_371_000.;
    let mut pr = pr;
    if let Node::Raise { center, .. } = &mut pr.nodes[1] {
        *center = sphere(zero(), r, 0., 0.);
    }
    let mut globe:planet::Document=serde_json::from_value(serde_json::json!({"version":2,"space_version":1,"id":"shared","revision":1,"center":{"cell":["0","0","0"],"offset":[0,0,0]},"diameter_meters":2.*r,"surface_gravity_mps2":9.81,"base_height_meters":0,"operations":[{"kind":"surface","program":pr}]})).unwrap();
    assert_eq!(
        planet::Sampler::new(&globe)
            .unwrap()
            .sample([0., 1., 0.])
            .unwrap()
            .height_meters,
        12.
    );
    assert_eq!(
        planet::decode(&planet::encode(&globe).unwrap()).unwrap(),
        globe
    );
    globe.version = 1;
    assert!(globe.validate().is_err());
    globe.version = 2;
    // An invalid program must not partially edit the caller's source.
    let original = flat.clone();
    let mut broken = pr;
    broken.range_meters = [0., 1.];
    assert!(
        terrain::edit(
            flat.clone(),
            1,
            vec![terrain::Edit::Append {
                operation: terrain::Operation::Surface {
                    program: Box::new(broken)
                }
            }]
        )
        .is_err()
    );
    assert_eq!(flat, original);
    let bad = source(vec![curve(vec![sphere(zero(), r, 0., 0.); 2])], 0);
    assert!(
        bad.compile(Domain::Sphere {
            center: zero(),
            radius_meters: r
        })
        .is_err()
    );
}

#[test]
fn plateau_channel_ridge_and_scalar_composition_are_common_tools() {
    fn tools(anchor: impl Fn(f64, f64) -> Position) -> Program {
        source(
            vec![
                Node::Constant { value: 20. },
                Node::Ridge {
                    center: anchor(0., 0.),
                    azimuth_degrees: 0.,
                    width_meters: 300.,
                    depth_meters: 180.,
                    height_meters: 40.,
                    phase: 1.,
                    crest_wander: 0.1,
                    slope_exponent: 2.,
                    folds_meters: 1.,
                    end_fade: 0.3,
                },
                Node::Add { a: 0, b: 1 },
                Node::Level {
                    input: 2,
                    center: anchor(-80., -80.),
                    radius_meters: 30.,
                    height_meters: 10.,
                    slope: [0.1, 0.],
                    blend: 1.,
                },
                Node::Channel {
                    input: 3,
                    points: vec![anchor(-100., 0.), anchor(0., 0.), anchor(100., 0.)],
                    width_meters: 10.,
                    bed_start_meters: 5.,
                    bed_end_meters: -5.,
                    blend: 1.,
                },
                Node::Constant { value: 2. },
                Node::Multiply { a: 4, b: 5 },
                Node::Subtract { a: 6, b: 5 },
                Node::Minimum { a: 7, b: 0 },
                Node::Maximum { a: 8, b: 4 },
                Node::Clamp {
                    input: 9,
                    minimum: -10.,
                    maximum: 100.,
                },
            ],
            10,
        )
    }
    let flat = tools(p);
    let globe = tools(|x, z| sphere(zero(), 6_371_000., x, z));
    let a = flat.compile(Domain::Plane { datum: zero() }).unwrap();
    let b = globe
        .compile(Domain::Sphere {
            center: zero(),
            radius_meters: 6_371_000.,
        })
        .unwrap();
    for (x, z) in [
        (0., 0.),
        (-100., 0.),
        (100., 0.),
        (-80., -80.),
        (0., 40.),
        (20., -40.),
        (300., 300.),
    ] {
        let h = a.sample(p(x, z)).unwrap();
        let h2 = b.sample(sphere(zero(), 6_371_000., x, z)).unwrap();
        assert!((h - h2).abs() < 1e-4, "{x},{z}: {h} vs {h2}");
        assert!(h >= flat.range_meters[0] && h <= flat.range_meters[1]);
    }
    // A baked hand-edited map can consume the same program, then freeze it normally.
    let doc = terrain::Document {
        version: 3,
        space_version: 1,
        id: "fixed".into(),
        revision: 1,
        datum: zero(),
        base: terrain::Base::Samples {
            patch: terrain::Patch {
                origin: p(-128., -128.),
                cells: [2, 2],
                spacing_meters: 128.,
            },
            heights: vec![0.; 9],
        },
        operations: vec![terrain::Operation::Surface {
            program: Box::new(flat),
        }],
        water_level_meters: None,
    };
    let frozen = terrain::bake(
        &doc,
        terrain::Patch {
            origin: p(-128., -128.),
            cells: [2, 2],
            spacing_meters: 128.,
        },
    )
    .unwrap();
    assert_eq!(frozen.heights[4], a.sample(zero()).unwrap() as f32);
}

#[test]
fn shared_source_drives_resident_collision_and_orbital_view_together() {
    use ignis::{
        engine::{Command, Engine, ExecutionMode},
        model::RoomRecipe,
    };
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    scene.entities.retain(|e| e.id == "player");
    scene.entities[0].character_controller = Some(Default::default());
    scene.entities[0].footsteps = None;
    scene.entities[0].transform.position = [0., 0.91, 0.];
    scene.light.atmosphere = Some(ignis::atmosphere::Atmosphere {
        density: 0.02,
        sun_strength: 0.,
        ..Default::default()
    });
    let program = source(vec![Node::Constant { value: 5. }], 0);
    let config = planet::runtime::Config {
        document: planet::Document {
            version: 2,
            space_version: 1,
            id: "resident_shared".into(),
            revision: 1,
            center: zero(),
            diameter_meters: 12_742_000.,
            surface_gravity_mps2: 9.81,
            base_height_meters: 0.,
            operations: vec![
                serde_json::from_value(serde_json::json!({"kind":"surface","program":program}))
                    .unwrap(),
            ],
        },
        actor: "player".into(),
        spawn: planet::Pose {
            direction: [0., 1., 0.],
            forward: [0., 0., -1.],
            altitude_meters: 6.,
        },
        level: 12,
        subdivisions: 32,
        physical_radius_meters: 128.,
        color: [0.3, 0.5, 0.2],
        material: Default::default(),
        compile_distant_meshes: false,
    };
    let mut engine = Engine::new_with_mode(scene, ExecutionMode::Continuous).unwrap();
    engine
        .execute(
            Command::SetPlanetRuntime {
                config: Some(Box::new(config)),
            }
            .into(),
        )
        .unwrap();
    engine.advance(120).unwrap();
    let state = engine.inspect_planet_runtime();
    assert_eq!(state["controller"]["grounded"], true, "{state}");
    assert!(
        (state["pose"]["altitude_meters"].as_f64().unwrap() - 5.91).abs() < 0.1,
        "{state}"
    );
    engine
        .execute(
            Command::SetPlanetView {
                view: Some(planet::orbit::Settings {
                    distance_meters: 4.,
                    ..Default::default()
                }),
            }
            .into(),
        )
        .unwrap();
    let view = engine.inspect_planet_view();
    assert_eq!(view["active"], true);
    assert_eq!(view["all_six_faces_covered"], true, "{view}");
    assert_eq!(engine.inspect_planet_runtime()["pose"], state["pose"]);
}

#[test]
fn incoming_height_allows_real_leveling_of_fixed_maps_and_existing_planets() {
    let tools = |center| {
        source(
            vec![
                Node::Input {
                    minimum_meters: 20.,
                    maximum_meters: 20.,
                },
                Node::Level {
                    input: 0,
                    center,
                    radius_meters: 100.,
                    height_meters: 7.,
                    slope: [0., 0.],
                    blend: 1.,
                },
            ],
            1,
        )
    };
    let mut fixed = terrain::Document {
        version: 3,
        space_version: 1,
        id: "fixed_input".into(),
        revision: 1,
        datum: zero(),
        base: terrain::Base::Samples {
            patch: terrain::Patch {
                origin: p(-200., -200.),
                cells: [2, 2],
                spacing_meters: 200.,
            },
            heights: vec![20.; 9],
        },
        operations: vec![terrain::Operation::Surface {
            program: Box::new(tools(zero())),
        }],
        water_level_meters: None,
    };
    let flat = terrain::Sampler::new(&fixed).unwrap();
    assert_eq!(flat.sample(zero()).unwrap(), Some(7.));
    assert_eq!(flat.sample(p(150., 0.)).unwrap(), Some(20.));
    let r = 6_371_000.;
    let mut globe:planet::Document=serde_json::from_value(serde_json::json!({"version":2,"space_version":1,"id":"existing_input","revision":1,"center":zero(),"diameter_meters":r*2.,"surface_gravity_mps2":9.81,"base_height_meters":20.,"operations":[{"kind":"surface","program":tools(sphere(zero(),r,0.,0.))}]})).unwrap();
    let curved = planet::Sampler::new(&globe).unwrap();
    assert_eq!(curved.sample([0., 1., 0.]).unwrap().height_meters, 7.);
    let far = DVec3::Y * (150f64 / r).cos() + DVec3::X * (150f64 / r).sin();
    assert_eq!(curved.sample(far.to_array()).unwrap().height_meters, 20.);
    if let terrain::Operation::Surface { program } = &mut fixed.operations[0] {
        program.nodes[0] = Node::Input {
            minimum_meters: 19.,
            maximum_meters: 19.,
        };
    }
    assert!(fixed.validate().is_err());
    globe.base_height_meters = 21.;
    assert!(globe.validate().is_err());
}
