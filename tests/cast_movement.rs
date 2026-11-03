use ignis::{
    abilities::{CastMovement, Outcome},
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    physics::Input,
};
use serde_json::json;
fn scene(mode: CastMovement) -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[
        {"id":"cast","selection":"self_actor","range":0,"cast_ticks":20,"cooldown_ticks":30,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"noise","radius":5}]},
        {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]}
    ]})).unwrap();
    s.ability_catalog.abilities[0].movement = mode;
    let loadout=serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["cast","kill"]})).unwrap();
    let p = entity(&mut s, "player");
    p.transform.position = [0., 0.8, -4.];
    p.footsteps = None;
    p.abilities = Some(loadout);
    p.vitality =
        Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
    s
}
fn entity<'a>(s: &'a mut Scene, id: &str) -> &'a mut Entity {
    s.entities.iter_mut().find(|e| e.id == id).unwrap()
}
fn activate(e: &mut Engine, id: &str) {
    e.execute(
        Command::ActivateAbility {
            actor: id.into(),
            slot: 1,
            target: None,
        }
        .into(),
    )
    .unwrap();
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
fn index(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == id)
        .unwrap()
}
fn outcome(e: &Engine) -> Outcome {
    e.simulation
        .abilities
        .state
        .receipts
        .back()
        .unwrap()
        .outcome
}
fn energy(e: &Engine, id: &str) -> i64 {
    e.simulation.abilities.state.balances[index(e, id)][0]
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn interrupt() -> CastMovement {
    CastMovement::Interrupt {
        speed_tolerance: 0.2,
    }
}
fn npc(s: &mut Scene, spatial: bool) {
    let mut n:Entity=serde_json::from_value(json!({"id":"npc","name":"npc","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[-5,0.8,1],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":2,"stop_distance":0.4,"enabled":true,"patrol":{"route":{"kind":"points","points":[[5,1],[-5,1]]},"traversal":"loop","wait_ticks":10,"travel_ticks":60,"retry_ticks":30,"arrival_distance":0.4}}})).unwrap();
    n.abilities = entity(s, "player").abilities.clone();
    n.vitality = entity(s, "player").vitality;
    if spatial {
        let config = n.npc.as_mut().unwrap();
        config.mobility = Some(
            serde_json::from_value(
                json!({"walk":true,"jump_speed":null,"fly_speed":null,"glide":null}),
            )
            .unwrap(),
        );
        config.local_connections = Some(
            serde_json::from_value(json!({"walk":true,"fly":false,"max_distance":8})).unwrap(),
        );
        config.patrol.as_mut().unwrap().route =
            serde_json::from_value(json!({"kind":"points_3d","points":[[2,0.8,1],[-5,0.8,1]]}))
                .unwrap();
    }
    s.entities.push(n);
}
#[test]
fn free_preserves_cast_while_moving_and_legacy_format() {
    let mut s = scene(CastMovement::Free {});
    s.version = 28;
    assert!(
        serde_json::to_value(&s).unwrap()["ability_catalog"]["abilities"][0]
            .get("movement")
            .is_none()
    );
    let mut e = Engine::new(s).unwrap();
    input(&mut e, 1., 0.);
    activate(&mut e, "player");
    e.advance(21).unwrap();
    assert_eq!(outcome(&e), Outcome::Completed);
    assert_eq!(energy(&e, "player"), 95);
    assert!(e.simulation.position("player").unwrap()[0] > 1.);
    replay(&e);
}
#[test]
fn interrupt_rejects_same_tick_input_without_cost_or_cooldown() {
    let mut e = Engine::new(scene(interrupt())).unwrap();
    input(&mut e, 1., 0.);
    activate(&mut e, "player");
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Moving);
    assert_eq!(energy(&e, "player"), 100);
    assert_eq!(
        e.simulation.abilities.state.ready_at[index(&e, "player")][0],
        0
    );
    replay(&e);
}
#[test]
fn movement_at_deadline_interrupts_before_commit() {
    let mut e = Engine::new(scene(interrupt())).unwrap();
    e.advance(5).unwrap();
    activate(&mut e, "player");
    e.advance(20).unwrap();
    assert!(e.simulation.abilities.state.casts[index(&e, "player")].is_some());
    input(&mut e, 0., 1.);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::InterruptedByMovement);
    assert_eq!(energy(&e, "player"), 100);
    assert_eq!(e.simulation.abilities.state.emitted, 0);
    replay(&e);
}
#[test]
fn idle_contact_jitter_does_not_interrupt_and_reset_clears_state() {
    let mut e = Engine::new(scene(interrupt())).unwrap();
    activate(&mut e, "player");
    e.advance(21).unwrap();
    assert_eq!(outcome(&e), Outcome::Completed);
    replay(&e);
    e.execute(Command::Reset.into()).unwrap();
    assert!(e.simulation.abilities.state.receipts.is_empty());
    assert!(e.simulation.abilities.movement_locks().is_empty());
}
#[test]
fn lock_stops_voluntary_motion_then_resumes_held_input_at_completion() {
    let mut e = Engine::new(scene(CastMovement::Lock {})).unwrap();
    input(&mut e, 1., 0.);
    e.advance(5).unwrap();
    let before = e.simulation.position("player").unwrap();
    activate(&mut e, "player");
    e.advance(20).unwrap();
    assert!(!e.simulation.abilities.movement_locks().is_empty());
    assert!((e.simulation.position("player").unwrap()[0] - before[0]).abs() < 0.001);
    let restored = Engine::replay_at(&e.replay(), Some(10)).unwrap();
    assert!(!restored.simulation.abilities.movement_locks().is_empty());
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Completed);
    assert!(e.simulation.abilities.movement_locks().is_empty());
    assert!(e.simulation.position("player").unwrap()[0] > before[0] + 0.01);
    replay(&e);
}
#[test]
fn cancel_and_failed_start_never_leave_a_movement_lock() {
    let mut e = Engine::new(scene(CastMovement::Lock {})).unwrap();
    input(&mut e, 1., 0.);
    activate(&mut e, "player");
    e.advance(1).unwrap();
    let cast = e.simulation.abilities.state.casts[index(&e, "player")].unwrap();
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: cast.id,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Cancelled);
    assert!(e.simulation.abilities.movement_locks().is_empty());
    assert!(e.simulation.position("player").unwrap()[0] > 0.01);
    assert_eq!(energy(&e, "player"), 100);
    replay(&e);
    let mut s = scene(CastMovement::Lock {});
    entity(&mut s, "player")
        .abilities
        .as_mut()
        .unwrap()
        .resources[0]
        .initial = 0;
    let mut e = Engine::new(s).unwrap();
    input(&mut e, 1., 0.);
    activate(&mut e, "player");
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::InsufficientResource);
    assert!(e.simulation.abilities.movement_locks().is_empty());
    assert!(e.simulation.position("player").unwrap()[0] > 0.01);
    replay(&e);
}
#[test]
fn falling_interrupts_stationary_cast_but_lock_keeps_gravity() {
    for mode in [interrupt(), CastMovement::Lock {}] {
        let mut s = scene(mode);
        entity(&mut s, "player").transform.position[1] = 5.;
        s.ability_catalog.abilities[0].cast_ticks = 120;
        let mut e = Engine::new(s).unwrap();
        activate(&mut e, "player");
        e.advance(20).unwrap();
        assert!(e.simulation.position("player").unwrap()[1] < 4.8);
        if mode == (CastMovement::Lock {}) {
            assert!(!e.simulation.abilities.movement_locks().is_empty());
        } else {
            assert_eq!(outcome(&e), Outcome::InterruptedByMovement);
            assert_eq!(energy(&e, "player"), 100);
        }
        replay(&e);
    }
}
#[test]
fn npc_locomotion_cannot_continue_a_stationary_cast_started_at_rest() {
    let mut s = scene(interrupt());
    npc(&mut s, false);
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "npc");
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::InterruptedByMovement);
    assert_eq!(energy(&e, "npc"), 100);
    assert!(e.simulation.abilities.state.casts[index(&e, "npc")].is_none());
    replay(&e);
}
#[test]
fn npc_lock_suspends_ground_and_spatial_patrol_without_timeout_and_resumes() {
    for spatial in [false, true] {
        let mut s = scene(CastMovement::Lock {});
        npc(&mut s, spatial);
        s.ability_catalog.abilities[0].cast_ticks = 90;
        let mut e = Engine::new(s).unwrap();
        e.advance(5).unwrap();
        let before = e.simulation.position("npc").unwrap();
        activate(&mut e, "npc");
        e.advance(80).unwrap();
        assert!(e.simulation.npcs["npc"].locomotion_locked);
        assert_eq!(
            e.simulation.patrols["npc"].phase,
            ignis::patrol::Phase::Suspended
        );
        assert_eq!(e.simulation.patrols["npc"].skipped, 0);
        assert!((e.simulation.position("npc").unwrap()[0] - before[0]).abs() < 0.001);
        e.advance(20).unwrap();
        assert!(!e.simulation.npcs["npc"].locomotion_locked);
        assert!(e.simulation.position("npc").unwrap()[0] > before[0] + 0.1);
        assert_eq!(e.simulation.patrols["npc"].skipped, 0);
        replay(&e);
    }
}
#[test]
fn committed_flight_rejects_lock_and_preserves_trajectory() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-spatial-patrol.scene.json")).unwrap();
    s.version = 29;
    let base = scene(CastMovement::Lock {});
    s.ability_catalog = base.ability_catalog.clone();
    entity(&mut s, "flyer").abilities = base
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .abilities
        .clone();
    let mut e = Engine::new(s).unwrap();
    e.advance(40).unwrap();
    assert_eq!(
        e.simulation.mobility.states["flyer"].phase,
        ignis::mobility::Phase::Traversing
    );
    let before = e.simulation.position("flyer").unwrap();
    activate(&mut e, "flyer");
    e.advance(5).unwrap();
    assert_eq!(outcome(&e), Outcome::MovementLockUnavailable);
    assert!(
        glam::Vec3::from(before).distance(e.simulation.position("flyer").unwrap().into()) > 0.05
    );
    assert_eq!(energy(&e, "flyer"), 100);
    replay(&e);
}
#[test]
fn death_releases_npc_movement_lock() {
    let mut s = scene(CastMovement::Lock {});
    npc(&mut s, false);
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "npc");
    e.advance(1).unwrap();
    assert!(e.simulation.npcs["npc"].locomotion_locked);
    e.execute(
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 2,
            target: Some("npc".into()),
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert!(e.simulation.abilities.movement_locks().is_empty());
    assert!(!e.simulation.npcs["npc"].locomotion_locked);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::CancelledByDeath)
    );
    replay(&e);
}
#[test]
fn movement_policy_version_and_tolerance_validation() {
    let mut s = scene(interrupt());
    s.version = 28;
    assert!(s.validate().is_err());
    for speed_tolerance in [0., -1., 2.1, f32::NAN, f32::INFINITY] {
        assert!(
            scene(CastMovement::Interrupt { speed_tolerance })
                .validate()
                .is_err()
        );
    }
    assert!(
        serde_json::from_value::<CastMovement>(json!({"mode":"lock","speed_tolerance":1})).is_err()
    );
}

#[test]
fn physical_push_interrupts_without_player_input() {
    let mut s = scene(interrupt());
    s.ability_catalog.abilities[0].cast_ticks = 180;
    // Off-centre falling body causes a real solver displacement; navigation cannot avoid it.
    s.entities.push(serde_json::from_value(json!({"id":"crate","name":"crate","body":"dynamic","color":[0.4,0.4,0.4],"transform":{"position":[0.4,3,-4],"size":[1,1,1]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player");
    e.advance(120).unwrap();
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::InterruptedByMovement)
    );
    assert_eq!(energy(&e, "player"), 100);
    assert!(!e.simulation.abilities.state.motion[index(&e, "player")].requested);
    replay(&e);
}
#[test]
fn lock_releases_passage_reservation_before_motor_and_resumes_after_cancel() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-passages.scene.json")).unwrap();
    s.version = 29;
    let base = scene(CastMovement::Lock {});
    s.ability_catalog = base.ability_catalog.clone();
    entity(&mut s, "alpha").abilities = base
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .abilities
        .clone();
    s.ability_catalog.abilities[0].cast_ticks = 120;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let alpha = e
        .scene
        .entities
        .iter()
        .position(|a| a.id == "alpha")
        .unwrap() as u16;
    assert_eq!(e.simulation.passages.slots[0].owner, Some(alpha));
    activate(&mut e, "alpha");
    e.advance(1).unwrap();
    assert_ne!(e.simulation.passages.slots[0].owner, Some(alpha));
    assert_eq!(
        e.simulation.passages.agents["alpha"].phase,
        ignis::passages::Phase::Idle
    );
    let id = e.simulation.abilities.state.casts[index(&e, "alpha")]
        .unwrap()
        .id;
    e.execute(
        Command::CancelAbility {
            actor: "alpha".into(),
            execution_id: id,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert!(!e.simulation.npcs["alpha"].locomotion_locked);
    replay(&e);
}

#[test]
fn automatic_npc_cast_uses_shared_movement_modes() {
    for mode in [interrupt(), CastMovement::Lock {}] {
        let mut s = scene(mode);
        npc(&mut s, false);
        entity(&mut s, "npc").npc.as_mut().unwrap().objectives = Some(serde_json::from_value(json!([
            {"priority":2,"when":"always","action":{"kind":"use_ability","slot":1,"retry_ticks":5}},
            {"priority":1,"when":"always","action":{"kind":"patrol"}}
        ])).unwrap());
        let mut e = Engine::new(s).unwrap();
        e.advance(22).unwrap();
        assert_eq!(energy(&e, "npc"), 95);
        assert_eq!(outcome(&e), Outcome::Completed);
        assert!(!e.simulation.npcs["npc"].locomotion_locked);
        replay(&e);
    }
}
