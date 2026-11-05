use ignis::{
    terrain::{
        self, Base, Document, Operation,
        plateaus::{Connection, Settings},
    },
    world_space::{Coordinates, Position},
};
fn p(x: f64, z: f64) -> Position {
    Position::normalize(Coordinates {
        cell: [0; 3],
        offset: [x, 0., z],
    })
    .unwrap()
}
fn settings() -> Settings {
    Settings {
        seed: 2718,
        radius_meters: 240.,
        tread_meters: 40.,
        minimum_step_meters: 3.,
        maximum_step_meters: 15.,
        aspect: 1.1,
        contour_irregularity: 0.22,
        warp_meters: 14.,
        warp_period_meters: 180.,
        cliff_fraction: 0.04,
        top_detail_meters: 0.08,
        connections: vec![
            Connection {
                azimuth_degrees: -100.,
                width_meters: 70.,
                meander_radians: 0.45,
                meander_scale_meters: 80.,
            },
            Connection {
                azimuth_degrees: 30.,
                width_meters: 48.,
                meander_radians: 0.7,
                meander_scale_meters: 95.,
            },
        ],
    }
}
fn doc(s: Settings) -> Document {
    Document {
        version: 4,
        space_version: 1,
        id: "natural_terraces".into(),
        revision: 1,
        datum: p(0., 0.),
        base: Base::Constant { height_meters: 0. },
        operations: vec![Operation::NaturalTerraces {
            center: p(0., 0.),
            settings: Box::new(s),
        }],
        water_level_meters: None,
    }
}
#[test]
fn cumulative_levels_have_bounded_deltas_not_a_total_height_cap() {
    let mut s = settings();
    s.validate().unwrap();
    for k in 0..10000 {
        let d = s.level_height(k + 1) - s.level_height(k);
        assert!((3. - 1e-9..=15. + 1e-9).contains(&d));
    }
    assert!(s.level_height(1000) > 8000.);
    s.radius_meters = 10000.;
    doc(s).validate().unwrap();
}
#[test]
fn natural_connections_are_curved_walkable_and_share_block_seams() {
    let s = settings();
    let d = doc(s.clone());
    let sampler = terrain::Sampler::new(&d).unwrap();
    let mut last: Option<([f64; 2], f64)> = None;
    let mut max_slope = 0f64;
    for i in 0..1001 {
        let r = 250. - i as f64 * 0.23;
        let q = s.route_point(r, 0).unwrap();
        let h = sampler.sample(p(q[0], q[1])).unwrap().unwrap();
        if let Some((prev, old)) = last {
            let distance = (q[0] - prev[0]).hypot(q[1] - prev[1]);
            max_slope = max_slope.max(((h - old) / distance).abs());
        }
        last = Some((q, h));
    }
    assert!(max_slope < 0.8, "slope{max_slope}");
    let a = s.route_point(240., 0).unwrap();
    let b = s.route_point(140., 0).unwrap();
    let c = s.route_point(40., 0).unwrap();
    assert!(((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() > 100.);
    let left = terrain::bake(
        &d,
        terrain::Patch {
            origin: p(-20., -20.),
            cells: [20, 20],
            spacing_meters: 2.,
        },
    )
    .unwrap();
    let right = terrain::bake(
        &d,
        terrain::Patch {
            origin: p(20., -20.),
            cells: [20, 20],
            spacing_meters: 2.,
        },
    )
    .unwrap();
    for z in 0..=20 {
        assert_eq!(left.heights[z * 21 + 20], right.heights[z * 21]);
    }
    assert_eq!(terrain::decode(&terrain::encode(&d).unwrap()).unwrap(), d);
    let mut other = d.clone();
    if let Operation::NaturalTerraces { settings, .. } = &mut other.operations[0] {
        settings.seed = 42;
    }
    let altered = terrain::bake(
        &other,
        terrain::Patch {
            origin: p(-20., -20.),
            cells: [20, 20],
            spacing_meters: 2.,
        },
    )
    .unwrap();
    assert_ne!(left.heights, altered.heights);
}
#[test]
fn legacy_versions_stay_unchanged_and_new_operation_requires_four() {
    for version in 1..=3 {
        let mut d = doc(settings());
        d.version = version;
        assert!(d.validate().is_err());
        d.operations.clear();
        let bytes = terrain::encode(&d).unwrap();
        assert_eq!(
            terrain::encode(&terrain::decode(&bytes).unwrap()).unwrap(),
            bytes
        );
    }
    let mut bad = settings();
    bad.connections[0].width_meters = 0.;
    assert!(doc(bad).validate().is_err());
}

#[test]
fn native_player_climbs_the_curved_connection_without_inspection_flight() {
    use ignis::{
        engine::{Command, Engine, ExecutionMode},
        model::RoomRecipe,
        physics::Input,
        player_view,
    };
    let s = settings();
    let d = doc(s.clone());
    let sampler = terrain::Sampler::new(&d).unwrap();
    let start = s.route_point(250., 0).unwrap();
    let mut scene = RoomRecipe::default().generate().unwrap();
    scene.entities.retain(|e| e.id == "player");
    let actor = &mut scene.entities[0];
    actor.transform.position = [
        start[0] as f32,
        sampler.sample(p(start[0], start[1])).unwrap().unwrap() as f32 + 0.95,
        start[1] as f32,
    ];
    actor.character_controller = Some(ignis::character_controller::Config {
        max_slope_degrees: 45.,
        slide_slope_degrees: 50.,
        snap_distance: 0.2,
        ..Default::default()
    });
    actor.footsteps = None;
    let mut e = Engine::new(scene).unwrap();
    for z in 0..3 {
        for x in 0..3 {
            let id = format!("natural_{x}_{z}");
            e.execute(
                Command::PublishTerrain {
                    publication: Box::new(terrain::Publication {
                        collision: Some(true),
                        document: d.clone(),
                        patch: terrain::Patch {
                            origin: p(-300. + 200. * x as f64, -300. + 200. * z as f64),
                            cells: [100, 100],
                            spacing_meters: 2.,
                        },
                        stride: 1,
                        asset: id.clone(),
                        instance: id,
                        frame_origin: p(0., 0.),
                        color: [1.; 3],
                        material: Default::default(),
                    }),
                }
                .into(),
            )
            .unwrap();
        }
    }
    e.execute(
        Command::SetExecutionMode {
            mode: ExecutionMode::Continuous,
        }
        .into(),
    )
    .unwrap();
    e.execute(
        Command::SetPlayerView {
            view: Some(player_view::Config {
                actor: "player".into(),
                yaw_degrees: 0.,
                pitch_degrees: 0.,
                eye_height: 1.45,
                fov_degrees: 70.,
                mode: player_view::Mode::ThirdPerson,
                third_person: Default::default(),
            }),
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Step { ticks: 90 }.into()).unwrap();
    let position = |engine: &mut Engine| {
        let v = engine.execute(Command::InspectPlayerView.into()).unwrap();
        std::array::from_fn::<_, 3, _>(|i| v["resolved"]["pivot"][i].as_f64().unwrap())
    };
    let origin = position(&mut e);
    for index in 1..=192 {
        let q = s.route_point(250. - 226. * index as f64 / 192., 0).unwrap();
        for attempt in 0..80 {
            let a = position(&mut e);
            let delta = [q[0] - a[0], q[1] - a[2]];
            let distance = delta[0].hypot(delta[1]);
            if distance < 0.55 {
                break;
            }
            assert!(attempt < 79, "blocked at{index}: {a:?} toward{q:?}");
            e.execute(
                Command::SetInput {
                    input: Input {
                        x: (delta[0] / distance) as f32,
                        z: (delta[1] / distance) as f32,
                        ..Default::default()
                    },
                }
                .into(),
            )
            .unwrap();
            e.execute(Command::Step { ticks: 4 }.into()).unwrap();
        }
    }
    e.execute(
        Command::SetInput {
            input: Default::default(),
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Step { ticks: 30 }.into()).unwrap();
    let end = position(&mut e);
    assert!(end[1] - origin[1] > 30., "{origin:?} ->{end:?}");
    assert_eq!(
        e.execute(Command::InspectCharacterControllers.into())
            .unwrap()["controllers"]["player"]["grounded"],
        true
    );
}
