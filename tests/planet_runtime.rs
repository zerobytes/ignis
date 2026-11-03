//! Physical acceptance: the real KCC crosses regions/faces and returns without teleports.
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::{MovementMode, RoomRecipe, Scene},
    physics::Input,
    planet::{Document, Pose, runtime::Config},
    world_space::{Coordinates, Position},
};
use std::f64::consts::TAU;

fn setup(diameter: f64, level: u8, speed: f32) -> (Scene, Config) {
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    scene.entities.retain(|e| e.id == "player");
    let player = &mut scene.entities[0];
    player.character_controller = Some(Default::default());
    player.footsteps = None;
    let mut locomotion = ignis::model::Locomotion::default();
    locomotion.run.speed = speed;
    player.locomotion = Some(locomotion);
    player.transform.position = [0., 0.91, 0.];
    scene.light.atmosphere = Some(ignis::atmosphere::Atmosphere {
        density: 0.02,
        sun_strength: 0.,
        ..Default::default()
    });
    let config = Config {
        document: Document {
            version: 1,
            space_version: 1,
            id: "physical_earth".into(),
            revision: 1,
            center: Position::normalize(Coordinates {
                cell: [9_007_199_254_740_993, -9_007_199_254_740_993, 0],
                offset: [0.125, 0.5, 0.25],
            })
            .unwrap(),
            diameter_meters: diameter,
            surface_gravity_mps2: 9.81,
            base_height_meters: 0.,
            operations: vec![],
        },
        actor: "player".into(),
        spawn: Pose {
            direction: [0., 1., 0.],
            forward: [0., 0., -1.],
            altitude_meters: 0.91,
        },
        level,
        subdivisions: 32,
        physical_radius_meters: 128.,
        color: [0.3, 0.5, 0.2],
        material: Default::default(),
        compile_distant_meshes: false,
    };
    (scene, config)
}
fn engine(scene: Scene, config: Config) -> Engine {
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
    engine
}
fn full_turn(e: &mut Engine) {
    let identity = e.simulation.actors.reference(0);
    e.execute(
        Command::SetInput {
            input: Input {
                x: 0.,
                z: -1.,
                mode: MovementMode::Run,
            },
        }
        .into(),
    )
    .unwrap();
    let mut sampled_grounded = 0;
    let radius = e.inspect_planet_runtime()["diameter_meters"]
        .as_f64()
        .unwrap()
        * 0.5;
    let speed = f64::from(e.simulation.movement_profile(MovementMode::Run).speed);
    let mut stalled = 0;
    loop {
        let angle = e.inspect_planet_runtime()["initial_meridian_progress_radians"]
            .as_f64()
            .unwrap();
        if angle >= TAU {
            break;
        }
        let ticks = (((TAU - angle) * radius / (speed / 60.) * 0.9).floor() as u32).clamp(1, 120);
        e.advance(ticks).unwrap();
        let state = e.inspect_planet_runtime();
        if state["controller"]["grounded"] == true {
            sampled_grounded += 1;
        }
        let a = state["initial_meridian_progress_radians"].as_f64().unwrap();
        if a - angle < 1e-8 {
            stalled += 1;
        } else {
            stalled = 0;
        }
        assert!(stalled < 5, "physical motor stalled: {state}");
        if a >= TAU {
            break;
        }
        assert!(
            e.simulation.tick < 3_000_000,
            "no physical progress: {state}"
        );
    }
    // Finish at a single actual collision-controller step, not a geometric reposition.
    while e.inspect_planet_runtime()["initial_meridian_progress_radians"]
        .as_f64()
        .unwrap()
        < TAU
    {
        e.advance(1).unwrap();
    }
    let state = e.inspect_planet_runtime();
    let step = e.simulation.movement_profile(MovementMode::Run).speed as f64 / 60.;
    assert!(
        state["distance_from_spawn_meters"].as_f64().unwrap() < step + 1.,
        "{state}"
    );
    assert_eq!(e.simulation.actors.reference(0), identity);
    assert!(sampled_grounded > 10);
    assert_eq!(state["controller"]["grounded"], true, "{state}");
    assert!(state["visited_faces"].as_array().unwrap().len() >= 4);
    eprintln!("Physical turn: {state}");
}

#[test]
fn player_physically_circumnavigates_and_reset_preserves_the_world_mode() {
    let (scene, config) = setup(2000., 2, 60.);
    let mut e = engine(scene, config);
    full_turn(&mut e);
    e.execute(Command::Reset.into()).unwrap();
    assert!(e.planet_active());
    assert_eq!(e.simulation.tick, 0);
    assert_eq!(e.input, Input::default());
    assert_eq!(e.inspect_planet_runtime()["distance_from_spawn_meters"], 0.);
}

#[test]
fn planet_activation_failures_are_atomic_and_incomplete_saves_are_refused() {
    let (scene, config) = setup(2000., 2, 60.);
    let mut e = Engine::new_with_mode(scene.clone(), ExecutionMode::Continuous).unwrap();
    let before = e.snapshot();
    let mut bad = config.clone();
    bad.level = 24;
    assert!(
        e.execute(
            Command::SetPlanetRuntime {
                config: Some(Box::new(bad))
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    e.execute(
        Command::SetPlanetRuntime {
            config: Some(Box::new(config)),
        }
        .into(),
    )
    .unwrap();
    let before = e.inspect_planet_runtime();
    assert!(ignis::checkpoint::Checkpoint::capture(&e).is_err());
    assert!(ignis::portable_save::PortableSave::capture(&e).is_err());
    assert!(e.replace(scene).is_err());
    assert!(e.execute(Command::Undo.into()).is_err());
    assert_eq!(e.inspect_planet_runtime(), before);
    e.execute(Command::SetPlanetRuntime { config: None }.into())
        .unwrap();
    assert!(!e.planet_active());
}

#[test]
#[ignore = "long physical proof at real Earth diameter; no GPU or accelerated simulation clock"]
fn earth_sized_world_has_a_complete_physical_turn() {
    let (scene, config) = setup(12_742_000., 10, 1000.);
    let mut e = engine(scene, config);
    full_turn(&mut e);
}

#[test]
fn mountain_valley_crosses_convex_triangle_crests_without_internal_edge_walls() {
    let (mut scene, mut config) = setup(12_742_000., 12, 600.);
    scene.entities[0].transform.size = [0.6, 1.8, 0.6];
    config.document = ignis::planet::decode(include_bytes!(
        "../assets/planets/earth-mountains.ignis-planet"
    ))
    .unwrap();
    config.subdivisions = 64;
    config.physical_radius_meters = 256.;
    let mut e = engine(scene, config);
    let identity = e.simulation.actors.reference(0);
    e.execute(
        Command::SetInput {
            input: Input {
                x: 0.,
                z: -1.,
                mode: MovementMode::Run,
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(1500).unwrap();
    e.execute(
        Command::SetInput {
            input: Input::default(),
        }
        .into(),
    )
    .unwrap();
    for _ in 0..10 {
        e.advance(180).unwrap();
        if e.inspect_planet_runtime()["controller"]["grounded"] == true {
            break;
        }
    }
    let state = e.inspect_planet_runtime();
    assert!(
        state["distance_from_spawn_meters"].as_f64().unwrap() > 14_000.,
        "{state}"
    );
    assert_eq!(state["controller"]["grounded"], true, "{state}");
    assert_eq!(e.simulation.actors.reference(0), identity);
    assert!(state["tile_retirements"].as_u64().unwrap() > 0);
}
