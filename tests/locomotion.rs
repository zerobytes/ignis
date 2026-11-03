use ignis::{
    engine::{Command, Engine},
    model::{FootstepConfig, Locomotion, MovementMode, RoomRecipe, Scene},
    physics::Input,
};

fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    player.locomotion = Some(Locomotion::default());
    player.footsteps = Some(FootstepConfig {
        surface: "floor".into(),
        stride: 0.2,
        radius: 8.,
        min_speed: 0.1,
    });
    s
}
fn input(e: &mut Engine, mode: MovementMode) {
    e.execute(
        Command::SetInput {
            input: Input { x: 1., z: 0., mode },
        }
        .into(),
    )
    .unwrap();
}

#[test]
fn modes_change_physical_speed_without_diagonal_bonus_and_replay_exactly() {
    let mut distances = vec![];
    for mode in [MovementMode::Sneak, MovementMode::Walk, MovementMode::Run] {
        let mut e = Engine::new(scene()).unwrap();
        e.advance(30).unwrap();
        let start = e.simulation.positions()["player"];
        input(&mut e, mode);
        let profile = e.simulation.movement_profile(mode);
        e.advance(30).unwrap();
        let end = e.simulation.positions()["player"];
        let distance = end[0] - start[0];
        assert!(
            (distance - profile.speed * 0.5).abs() < 0.03,
            "{mode:?}: {distance}"
        );
        distances.push(distance);
        assert_eq!(e.snapshot()["player_motion"]["speed"], profile.speed);
        assert_eq!(
            e.simulation.footsteps.recent.back().unwrap().radius,
            8. * profile.noise_multiplier
        );
        e.execute(
            Command::SetInput {
                input: Input {
                    x: -1.,
                    z: 1.,
                    mode,
                },
            }
            .into(),
        )
        .unwrap();
        let start = e.simulation.positions()["player"];
        e.advance(20).unwrap();
        let end = e.simulation.positions()["player"];
        let diagonal = ((end[0] - start[0]).powi(2) + (end[2] - start[2]).powi(2)).sqrt();
        assert!((diagonal - profile.speed / 3.).abs() < 0.03);
        let replay = serde_json::from_str(&serde_json::to_string(&e.replay()).unwrap()).unwrap();
        assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    }
    assert!(distances[0] < distances[1] && distances[1] < distances[2]);
}

#[test]
fn quiet_steps_fail_to_reach_hidden_listener_while_walk_and_run_reach_it() {
    for mode in [MovementMode::Sneak, MovementMode::Walk, MovementMode::Run] {
        let mut s: Scene =
            serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
        s.version = 8;
        let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
        player.footsteps = scene()
            .entities
            .iter()
            .find(|e| e.id == "player")
            .unwrap()
            .footsteps
            .clone();
        player.locomotion = Some(Locomotion::default());
        let mut e = Engine::new(s).unwrap();
        e.advance(30).unwrap();
        input(&mut e, mode);
        e.advance(20).unwrap();
        assert!(e.simulation.footsteps.total_emitted > 0);
        let npc = &e.simulation.npcs["hunter"];
        assert!(!npc.observed_this_tick);
        assert_eq!(
            npc.memory.is_some(),
            mode != MovementMode::Sneak,
            "{mode:?}"
        );
    }
}

#[test]
fn zero_noise_consumes_stride_without_emission_and_positive_radius_is_bounded() {
    let mut s = scene();
    let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    player.locomotion.as_mut().unwrap().sneak.noise_multiplier = 0.;
    player.locomotion.as_mut().unwrap().walk.noise_multiplier = 0.001;
    player.locomotion.as_mut().unwrap().run.noise_multiplier = 4.;
    player.footsteps.as_mut().unwrap().radius = 100.;
    let mut e = Engine::new(s).unwrap();
    e.advance(30).unwrap();
    input(&mut e, MovementMode::Sneak);
    e.advance(90).unwrap();
    assert_eq!(e.simulation.footsteps.total_emitted, 0);
    assert_eq!(e.simulation.footsteps.states["player"].emitted, 0);
    assert!(e.simulation.footsteps.states["player"].distance_since_step < 0.2);
    input(&mut e, MovementMode::Walk);
    e.advance(5).unwrap();
    assert_eq!(e.simulation.footsteps.recent.back().unwrap().radius, 0.1);
    input(&mut e, MovementMode::Run);
    e.advance(5).unwrap();
    assert_eq!(e.simulation.footsteps.recent.back().unwrap().radius, 100.);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}

#[test]
fn profiles_validate_atomically_and_legacy_inputs_default_to_walk() {
    let old: Input = serde_json::from_str(r#"{"x":1,"z":0}"#).unwrap();
    assert_eq!(old.mode, MovementMode::Walk);
    assert!(serde_json::from_str::<Input>(r#"{"x":1,"z":0,"mode":"fly"}"#).is_err());
    let mut e = Engine::new(scene()).unwrap();
    let player = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    for (speed, multiplier) in [(0., 1.), (1001., 1.), (1., -1.), (1., 5.), (f32::NAN, 1.)] {
        let mut bad = player.clone();
        bad.locomotion.as_mut().unwrap().run.speed = speed;
        bad.locomotion.as_mut().unwrap().run.noise_multiplier = multiplier;
        let before = e.snapshot();
        assert!(
            e.execute(Command::Upsert { entity: bad.into() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut wrong_role = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "floor")
        .unwrap()
        .clone();
    wrong_role.locomotion = player.locomotion;
    assert!(
        e.execute(
            Command::Upsert {
                entity: wrong_role.into()
            }
            .into()
        )
        .is_err()
    );
    let mut old = e.scene.clone();
    old.version = 7;
    assert!(Engine::new(old).is_err());
    input(&mut e, MovementMode::Run);
    let mut edited = player;
    edited.locomotion.as_mut().unwrap().run.speed = 8.;
    e.execute(
        Command::Upsert {
            entity: edited.into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.input, Input::default());
    assert_eq!(e.simulation.movement_profile(MovementMode::Run).speed, 8.);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.simulation.movement_profile(MovementMode::Run).speed, 6.);
    input(&mut e, MovementMode::Sneak);
    e.reset();
    assert_eq!(e.input, Input::default());
}
