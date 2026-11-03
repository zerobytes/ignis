use ignis::{
    engine::{Command, Engine},
    footsteps::AUTOMATIC_ID_BASE,
    model::{Awareness, FootstepConfig, RoomRecipe, Scene},
    noise::NoiseOrigin,
    perception::{Phase, Source},
    physics::Input,
};

fn config() -> FootstepConfig {
    FootstepConfig {
        surface: "floor".into(),
        stride: 0.5,
        radius: 8.,
        min_speed: 0.1,
    }
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .footsteps = Some(config());
    s
}
fn input(e: &mut Engine, x: f32, z: f32) {
    e.execute(
        Command::SetInput {
            input: Input {
                x,
                z,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
}

#[test]
fn footsteps_need_real_supported_motion_and_do_not_fill_the_manual_log() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(60).unwrap();
    assert_eq!(e.simulation.footsteps.total_emitted, 0);
    input(&mut e, 0., -1.);
    e.advance(180).unwrap(); // Hits the north wall and keeps requesting movement.
    let total = e.simulation.footsteps.total_emitted;
    assert!(total > 0);
    e.advance(120).unwrap();
    assert_eq!(e.simulation.footsteps.total_emitted, total);
    assert!(e.replay().noises.is_empty());
    let replay = e.replay();
    let checked = Engine::verify_replay(&replay).unwrap();
    assert_eq!(checked["matched"], true);
    assert_eq!(checked["footsteps_matched"], true);
    let mut tampered = replay;
    tampered.expected_footsteps.total_emitted += 1;
    assert_eq!(Engine::verify_replay(&tampered).unwrap()["matched"], false);
}

#[test]
fn footsteps_stay_silent_in_air_and_legacy_scenes() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [0., 5., 0.];
    let mut e = Engine::new(s).unwrap();
    input(&mut e, 1., 0.);
    e.advance(25).unwrap();
    assert_eq!(e.simulation.footsteps.total_emitted, 0);
    assert!(!e.simulation.footsteps.states["player"].grounded);
    e.advance(55).unwrap();
    assert!(e.simulation.footsteps.states["player"].grounded);
    assert!(e.simulation.footsteps.total_emitted > 0);
    let legacy = serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
    let mut e = Engine::new(legacy).unwrap();
    input(&mut e, 1., 0.);
    e.advance(90).unwrap();
    assert_eq!(e.simulation.footsteps.total_emitted, 0);
}

#[test]
fn footstep_history_is_bounded_and_replay_regenerates_evicted_events() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .footsteps
        .as_mut()
        .unwrap()
        .stride = 0.2;
    let mut e = Engine::new(s).unwrap();
    e.advance(30).unwrap();
    for _ in 0..16 {
        input(&mut e, 1., 0.);
        e.advance(60).unwrap();
        input(&mut e, -1., 0.);
        e.advance(60).unwrap();
    }
    let f = &e.simulation.footsteps;
    assert!(f.total_emitted > 256);
    assert_eq!(f.recent.len(), 256);
    assert_eq!(f.dropped_recent, f.total_emitted - 256);
    assert!(f.pending.len() <= 1);
    assert_eq!(
        f.recent.back().unwrap().id,
        AUTOMATIC_ID_BASE + f.total_emitted - 1
    );
    let replay = serde_json::from_str(&serde_json::to_string(&e.replay()).unwrap()).unwrap();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    assert!(e.replay().noises.is_empty());
    e.reset();
    assert!(e.simulation.footsteps.recent.is_empty());
    assert_eq!(
        e.simulation.footsteps.states["player"].distance_since_step,
        0.
    );
}

#[test]
fn invalid_footstep_config_is_atomic_and_undo_restarts_emitters() {
    let mut e = Engine::new(scene()).unwrap();
    let player = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "player")
        .unwrap()
        .clone();
    for (stride, radius, speed, surface) in [
        (0.1, 8., 0.1, "floor"),
        (0.5, 0., 0.1, "floor"),
        (0.5, 8., 0., "floor"),
        (0.5, 8., 0.1, "missing"),
        (0.5, 8., 0.1, "player"),
    ] {
        let mut bad = player.clone();
        bad.footsteps = Some(FootstepConfig {
            stride,
            radius,
            min_speed: speed,
            surface: surface.into(),
        });
        let before = e.snapshot();
        assert!(
            e.execute(Command::Upsert { entity: bad.into() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut old = e.scene.clone();
    old.version = 6;
    assert!(Engine::new(old).is_err());
    let mut fixed = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "floor")
        .unwrap()
        .clone();
    fixed.footsteps = Some(config());
    assert!(
        e.execute(
            Command::Upsert {
                entity: fixed.into()
            }
            .into()
        )
        .is_err()
    );
    input(&mut e, 1., 0.);
    e.advance(60).unwrap();
    assert!(e.simulation.footsteps.total_emitted > 0);
    let mut silent = player;
    silent.footsteps = None;
    e.execute(
        Command::Upsert {
            entity: silent.into(),
        }
        .into(),
    )
    .unwrap();
    assert!(e.simulation.footsteps.states.is_empty());
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.simulation.footsteps.total_emitted, 0);
    assert_eq!(e.simulation.footsteps.states.len(), 1);
}

#[test]
fn hidden_footsteps_are_consumed_next_tick_without_following_the_emitter() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
    s.version = 7;
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .footsteps = Some(config());
    let mut e = Engine::new(s).unwrap();
    e.advance(30).unwrap();
    input(&mut e, 1., 0.);
    while e.simulation.footsteps.total_emitted == 0 {
        e.advance(1).unwrap();
    }
    let noise = *e.simulation.footsteps.pending.last().unwrap();
    assert_eq!(e.simulation.npcs["hunter"].phase, Phase::Unaware);
    e.advance(1).unwrap();
    let heard = e.simulation.npcs["hunter"].memory.unwrap();
    assert!(e.simulation.npcs["hunter"].heard_this_tick);
    assert_eq!(heard.source, Source::Sound);
    assert_eq!(heard.noise_id, Some(noise.id));
    assert_eq!(heard.position, noise.position);
    input(&mut e, 0., 0.);
    e.advance(5).unwrap();
    assert_eq!(e.simulation.npcs["hunter"].memory.as_ref(), Some(&heard));
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}

#[test]
fn npc_ignores_own_steps_but_other_npcs_can_hear_them() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
    s.version = 7;
    let hunter = s.entities.iter_mut().find(|e| e.id == "hunter").unwrap();
    hunter.footsteps = Some(config());
    if let Some(Awareness::Perceptive { vision_range, .. }) =
        &mut hunter.npc.as_mut().unwrap().awareness
    {
        *vision_range = 0.1;
    }
    let mut other = hunter.clone();
    other.id = "listener".into();
    other.name = "Listener".into();
    other.transform.position = [-3., 0.85, 2.];
    other.footsteps = None;
    s.entities.push(other);
    let mut e = Engine::new(s).unwrap();
    e.advance(30).unwrap();
    // Small decoy only reaches the moving NPC; the other later hears its footsteps.
    e.execute(
        Command::EmitNoise {
            origin: NoiseOrigin::Position {
                position: [2., 0.8, 2.],
            },
            radius: 2.5,
        }
        .into(),
    )
    .unwrap();
    let mut heard_other = false;
    for _ in 0..90 {
        e.advance(1).unwrap();
        let own = &e.simulation.npcs["hunter"];
        assert_eq!(own.memory.as_ref().unwrap().noise_id, Some(0));
        heard_other |= e.simulation.npcs["listener"]
            .memory
            .as_ref()
            .is_some_and(|m| m.noise_id.is_some_and(|id| id >= AUTOMATIC_ID_BASE));
    }
    assert!(e.simulation.footsteps.total_emitted > 0);
    assert!(heard_other);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
