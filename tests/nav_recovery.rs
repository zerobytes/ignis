use glam::Vec2;
use ignis::{
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    nav_recovery::{Config, Failure, State},
    navigation::{self, NavGoal, NavState, NavigationScratch},
};
use serde_json::json;
use std::collections::BTreeMap;
fn config() -> Config {
    Config {
        retry_ticks: 15,
        max_retry_ticks: 60,
        stalled_ticks: 30,
        min_progress: 0.1,
    }
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 16.,
        depth: 12.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [0., 0.85, 0.];
    let guard:Entity=serde_json::from_value(json!({"id":"guard","name":"Guard","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[-4,0.85,0],"size":[0.7,1.6,0.7]},"npc":{"surface":"floor","speed":2,"stop_distance":0.3,"enabled":true,"navigation_recovery":config(),"objectives":[{"priority":1,"when":"always","action":{"kind":"guard","point":[4,0],"arrival_distance":0.3}}]}})).unwrap();
    s.entities.push(guard);
    for (id, z) in [("north", 3.65), ("south", -3.65)] {
        s.entities.push(serde_json::from_value(json!({"id":id,"name":id,"body":"fixed","color":[0.2,0.65,0.45],"transform":{"position":[0,1.5,z],"size":[0.4,3,5.9]}})).unwrap());
    }
    s
}
fn step(e: &mut Engine, n: u32) {
    e.execute(Command::Step { ticks: n }.into()).unwrap();
}
#[test]
fn capped_retry_clock_and_progress_reset_have_exact_boundaries() {
    let c = config();
    let mut s = State::default();
    for (tick, delay) in [(0, 15), (15, 30), (45, 60), (105, 60)] {
        s.failed(c, tick, Failure::NoClearPath);
        assert_eq!(s.retry_at_tick, Some(tick + delay));
        assert!(s.waiting(tick + delay - 1));
        assert!(!s.waiting(tick + delay));
    }
    assert!(!s.stalled(c, 165, Vec2::ZERO));
    s.movement_requested = true;
    assert!(!s.stalled(c, 194, Vec2::ZERO));
    s.movement_requested = true;
    assert!(s.stalled(c, 195, Vec2::ZERO));
    assert!(!s.stalled(c, 195, Vec2::new(0.11, 0.)));
    assert_eq!(s.consecutive_failures, 0);
    assert_eq!(s.failures, 4);
    s.arrived();
    assert_eq!(s.progress_since_tick, None);
}
#[test]
fn blocked_passage_retries_then_resumes_after_player_moves_without_reset() {
    let mut e = Engine::new(scene()).unwrap();
    step(&mut e, 100);
    let nav = &e.simulation.navigation["guard"];
    assert_eq!(nav.status, "retry_wait");
    assert_eq!(nav.recovery.path_attempts, 3);
    assert_eq!(nav.recovery.failures, 3);
    assert!(e.simulation.positions()["guard"][0] < -3.9);
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"set_input","input":{"x":1,"z":1}}))
            .unwrap()
            .into(),
    )
    .unwrap();
    step(&mut e, 60);
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"set_input","input":{"x":0,"z":0}}))
            .unwrap()
            .into(),
    )
    .unwrap();
    step(&mut e, 360);
    assert!(
        e.simulation.positions()["guard"][0] > 3.5,
        "{:?}",
        e.simulation.navigation["guard"]
    );
    assert_eq!(e.simulation.navigation["guard"].status, "arrived");
    assert_eq!(
        e.simulation.navigation["guard"]
            .recovery
            .consecutive_failures,
        0
    );
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    let reconstructed = Engine::replay_at(&replay, None).unwrap();
    assert_eq!(e.simulation.navigation, reconstructed.simulation.navigation);
}
#[test]
fn requested_motion_with_frozen_body_stalls_but_idle_and_disable_clear() {
    let mut s = scene();
    s.entities.retain(|e| e.id != "north" && e.id != "south");
    let mut positions: BTreeMap<_, _> = s
        .entities
        .iter()
        .map(|e| (e.id.clone(), e.transform.position))
        .collect();
    positions.get_mut("player").unwrap()[2] = 4.;
    let goals = BTreeMap::from([(
        "guard".into(),
        Some(NavGoal {
            position: [4., 0.85, 0.],
            excluded_entity: None,
            target_half_height: 0.8,
            stop_distance: 0.3,
            remaining_travel: None,
            replan_on_target_change: false,
        }),
    )]);
    let mut states = BTreeMap::from([("guard".into(), NavState::default())]);
    let mut velocities = BTreeMap::new();
    let mut scratch = NavigationScratch::default();
    for tick in 0..=30 {
        navigation::update(
            &s,
            &positions,
            tick,
            &goals,
            &mut states,
            &mut velocities,
            &mut scratch,
        );
    }
    assert_eq!(states["guard"].status, "retry_wait");
    assert_eq!(
        states["guard"].recovery.last_failure,
        Some(Failure::NoProgress)
    );
    assert_eq!(velocities["guard"], [0., 0.]);
    let attempts = states["guard"].recovery.path_attempts;
    for tick in 31..45 {
        navigation::update(
            &s,
            &positions,
            tick,
            &goals,
            &mut states,
            &mut velocities,
            &mut scratch,
        );
    }
    assert_eq!(states["guard"].recovery.path_attempts, attempts);
    navigation::update(
        &s,
        &positions,
        45,
        &goals,
        &mut states,
        &mut velocities,
        &mut scratch,
    );
    assert_eq!(states["guard"].status, "following");
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .enabled = false;
    navigation::update(
        &s,
        &positions,
        46,
        &goals,
        &mut states,
        &mut velocities,
        &mut scratch,
    );
    assert_eq!(states["guard"].recovery, State::default());
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .enabled = true;
    navigation::update(
        &s,
        &positions,
        47,
        &BTreeMap::new(),
        &mut states,
        &mut velocities,
        &mut scratch,
    );
    assert_eq!(states["guard"].status, "idle");
    assert_eq!(states["guard"].recovery, State::default());
}
#[test]
fn validation_is_atomic_versions_and_legacy_remain_explicit() {
    let s = scene();
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 5);
    let before = e.snapshot();
    let mut npc = s.entities.iter().find(|e| e.id == "guard").unwrap().clone();
    npc.npc
        .as_mut()
        .unwrap()
        .navigation_recovery
        .as_mut()
        .unwrap()
        .retry_ticks = 0;
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(npc)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    let mut old = s.clone();
    old.version = 16;
    assert!(old.validate().is_err());
    old.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .navigation_recovery = None;
    let mut legacy = Engine::new(old).unwrap();
    step(&mut legacy, 100);
    assert_eq!(legacy.simulation.navigation["guard"].status, "unreachable");
    assert_eq!(
        legacy.simulation.navigation["guard"].recovery,
        State::default()
    );
    for c in [
        Config {
            min_progress: f32::NAN,
            ..config()
        },
        Config {
            max_retry_ticks: 1,
            ..config()
        },
        Config {
            stalled_ticks: 11,
            ..config()
        },
        Config {
            min_progress: 1.,
            ..config()
        },
    ] {
        assert!(c.validate(0.1).is_err());
    }
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(e.simulation.navigation["guard"].recovery, State::default());
}

#[test]
fn sound_interrupts_retry_and_memory_expiry_resumes_guard_with_fresh_budget() {
    let mut s = scene();
    let npc = s
        .entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    npc.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":10,"fov_degrees":120,"initial_facing_degrees":0,"memory_ticks":6,"search_ticks":6,"search_turn_degrees_per_second":90,"hearing":{"range":10}})).unwrap());
    npc.objectives.as_mut().unwrap().insert(
        0,
        serde_json::from_value(
            json!({"priority":10,"when":"sound_memory","action":{"kind":"wait"}}),
        )
        .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 5);
    assert_eq!(e.simulation.navigation["guard"].status, "retry_wait");
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position {
                position: [-3., 0.85, 0.],
            },
            radius: 10.,
        }
        .into(),
    )
    .unwrap();
    step(&mut e, 1);
    assert_eq!(e.simulation.navigation["guard"].status, "idle");
    assert_eq!(e.simulation.navigation["guard"].recovery, State::default());
    step(&mut e, 20);
    assert!(e.simulation.npcs["guard"].memory.is_none());
    assert_eq!(e.simulation.navigation["guard"].status, "retry_wait");
    assert_eq!(e.simulation.navigation["guard"].recovery.failures, 1);
}

#[test]
fn territory_return_remains_committed_while_navigation_waits_and_replays() {
    let mut s = scene();
    let npc = s
        .entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    npc.objectives = None;
    npc.territory = Some(ignis::territory::Config {
        home: [4., 0.],
        radius: 3.,
        arrival_distance: 0.3,
        rearm_ticks: 10,
    });
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 100);
    assert_eq!(
        e.simulation.npcs["guard"].territory.phase,
        ignis::territory::Phase::Returning
    );
    assert_eq!(e.simulation.navigation["guard"].recovery.failures, 3);
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"set_input","input":{"x":1,"z":1}}))
            .unwrap()
            .into(),
    )
    .unwrap();
    step(&mut e, 60);
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"set_input","input":{"x":0,"z":0}}))
            .unwrap()
            .into(),
    )
    .unwrap();
    step(&mut e, 360);
    assert_eq!(
        e.simulation.npcs["guard"].territory.phase,
        ignis::territory::Phase::Active
    );
    assert!(e.simulation.positions()["guard"][0] > 3.5);
    let replay = e.replay();
    let restored = Engine::replay_at(&replay, None).unwrap();
    assert_eq!(e.simulation.navigation, restored.simulation.navigation);
}
