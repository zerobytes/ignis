use ignis::{
    engine::{Command, Engine},
    model::{RoomRecipe, Scene},
    noise::NoiseOrigin,
    objectives::{Action, Condition, Kind, Objective, Reason, State},
    perception::{self, Observation, Phase, Source},
};
use serde_json::json;
use std::collections::BTreeMap;
fn objective(priority: u8, when: Condition, action: Action) -> Objective {
    Objective {
        priority,
        when,
        action,
    }
}
fn scene(actions: Vec<Objective>) -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 24.,
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
        .position = [0., 0.85, 5.];
    let mut npc:ignis::model::Entity=serde_json::from_value(json!({"id":"guard","name":"Guard","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[0,0.85,0],"size":[0.7,1.6,0.7]},"npc":{"surface":"floor","speed":2,"stop_distance":1.2,"enabled":true,"awareness":{"mode":"perceptive","vision_range":12,"fov_degrees":90,"initial_facing_degrees":0,"memory_ticks":30,"search_ticks":10,"search_turn_degrees_per_second":90,"hearing":{"range":10}},"patrol":{"route":{"kind":"points","points":[[5,0],[-5,0]]},"traversal":"loop","wait_ticks":5,"travel_ticks":600,"retry_ticks":60,"arrival_distance":0.3}}})).unwrap();
    npc.npc.as_mut().unwrap().objectives = Some(actions);
    s.entities.push(npc);
    s
}
fn evidence(source: Source) -> Observation {
    Observation {
        instance: None,
        life: Some(0),
        position: [2., 0.85, 0.],
        tick: 1,
        source,
        noise_id: (source == Source::Sound).then_some(1),
        actor_index: None,
    }
}
fn step(e: &mut Engine, n: u32) {
    e.execute(Command::Step { ticks: n }.into()).unwrap();
}
fn sound(e: &mut Engine, x: f32) {
    e.execute(
        Command::EmitNoise {
            origin: NoiseOrigin::Position {
                position: [x, 0.85, 0.],
            },
            radius: 10.,
        }
        .into(),
    )
    .unwrap();
}
fn sense(
    e: &mut Engine,
    p: &BTreeMap<String, [f32; 3]>,
    tick: u64,
) -> BTreeMap<String, Option<ignis::navigation::NavGoal>> {
    let mut goals = BTreeMap::new();
    perception::update_with_noises(
        &e.scene,
        p,
        tick,
        &e.simulation.navigation,
        &mut e.simulation.npcs,
        &mut goals,
        &[],
    );
    goals
}
#[test]
fn priority_tie_sources_and_intrinsic_evidence_requirements_are_deterministic() {
    let config = [
        objective(4, Condition::Always, Action::Respond),
        objective(4, Condition::SoundMemory, Action::Flee { distance: 4. }),
        objective(1, Condition::Always, Action::Wait),
    ];
    let mut s = State::default();
    assert_eq!(s.select(Some(&config), None, false, 0), Some(Action::Wait));
    assert_eq!(s.eligible_mask, 4);
    assert_eq!(
        s.select(Some(&config), Some(evidence(Source::Sound)), false, 1),
        Some(Action::Respond)
    );
    assert_eq!(s.active, Some(0));
    assert_eq!(s.reason, Reason::HigherPriority);
    assert_eq!(s.eligible_mask, 7);
    s.select(Some(&config), None, false, 2);
    assert!(s.resumed);
    assert_eq!(s.previous, Some(0));
    assert_eq!(s.reason, Reason::ConditionLost);
    s.select(Some(&config), None, false, 3);
    assert_eq!(s.transitions, 3);
    assert!(!s.changed_this_tick);
    let conditions = [
        Condition::Evidence,
        Condition::VisualMemory,
        Condition::SoundMemory,
        Condition::VisibleTarget,
    ];
    for (condition, expected) in conditions.into_iter().zip([true, true, false, false]) {
        let o = [objective(1, condition, Action::Wait)];
        s.select(Some(&o), Some(evidence(Source::Vision)), false, 4);
        assert_eq!(s.active.is_some(), expected);
    }
}
#[test]
fn patrol_interrupts_on_sound_and_resumes_same_point_after_expiry() {
    let mut e = Engine::new(scene(vec![
        objective(10, Condition::Evidence, Action::Respond),
        objective(1, Condition::Always, Action::Patrol),
    ]))
    .unwrap();
    step(&mut e, 5);
    assert_eq!(e.simulation.npcs["guard"].objectives.active, Some(1));
    let point = e.simulation.patrols["guard"].point_index;
    sound(&mut e, -4.);
    step(&mut e, 1);
    assert_eq!(
        e.simulation.npcs["guard"].objectives.action,
        Some(Kind::Respond)
    );
    assert_eq!(
        e.simulation.patrols["guard"].phase,
        ignis::patrol::Phase::Suspended
    );
    step(&mut e, 30);
    let npc = &e.simulation.npcs["guard"];
    assert_eq!(npc.objectives.active, Some(1));
    assert!(npc.objectives.resumed);
    assert!(npc.memory.is_none());
    assert_eq!(e.simulation.patrols["guard"].point_index, point);
    assert!(e.simulation.patrols["guard"].leg_started_tick.unwrap() >= 35);
    assert_eq!(
        e.simulation.navigation["guard"].target_position.unwrap()[0],
        5.
    );
}
#[test]
fn higher_priority_guard_keeps_sensing_without_spending_investigation_distance() {
    let mut e = Engine::new(scene(vec![
        objective(
            20,
            Condition::Always,
            Action::Guard {
                point: [5., 0.],
                arrival_distance: 0.3,
            },
        ),
        objective(10, Condition::Evidence, Action::Respond),
    ]))
    .unwrap();
    sound(&mut e, -4.);
    step(&mut e, 12);
    let npc = &e.simulation.npcs["guard"];
    assert!(npc.memory.is_some());
    assert_eq!(npc.phase, Phase::Suspended);
    assert_eq!(npc.investigation_distance, 0.);
    assert_eq!(npc.objectives.active, Some(0));
    assert!(e.simulation.positions()["guard"][0] > 0.1);
    step(&mut e, 20);
    assert!(e.simulation.npcs["guard"].memory.is_none());
    assert_eq!(e.simulation.npcs["guard"].objectives.active, Some(0));
}
#[test]
fn fleeing_freezes_destination_and_never_moves_toward_an_already_distant_cue() {
    let mut s = State::default();
    let action = Some(Action::Flee { distance: 5. });
    let m = evidence(Source::Sound);
    let first = s.goal(action, Some(m), [0., 0.85, 0.], 0.8).unwrap();
    assert_eq!(first.position, [-3., 0.85, 0.]);
    let later = s.goal(action, Some(m), [-1., 0.85, 2.], 0.8).unwrap();
    assert_eq!(first.position, later.position);
    let mut s = State::default();
    assert_eq!(
        s.goal(action, Some(m), [-8., 0.85, 0.], 0.8)
            .unwrap()
            .position,
        [-8., 0.85, 0.]
    );
    let mut s = State::default();
    assert_eq!(
        s.goal(action, Some(m), m.position, 0.8).unwrap().position,
        [7., 0.85, 0.]
    );
    let mut e = Engine::new(scene(vec![
        objective(10, Condition::Evidence, Action::Flee { distance: 4. }),
        objective(0, Condition::Always, Action::Wait),
    ]))
    .unwrap();
    sound(&mut e, 2.);
    step(&mut e, 20);
    assert!(e.simulation.positions()["guard"][0] < -0.2);
    assert_eq!(e.simulation.npcs["guard"].investigation_distance, 0.);
    assert_eq!(e.simulation.npcs["guard"].memory.unwrap().actor_index, None);
}
#[test]
fn hidden_target_changes_do_not_change_flee_goal_and_disabled_resets_selector() {
    let mut s = scene(vec![
        objective(10, Condition::Evidence, Action::Flee { distance: 4. }),
        objective(0, Condition::Always, Action::Wait),
    ]);
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .target = Some("player".into());
    let mut a = Engine::new(s.clone()).unwrap();
    let mut b = Engine::new(s).unwrap();
    let mut p = a.simulation.positions();
    p.insert("player".into(), [0., 0.85, -2.]);
    sense(&mut a, &p, 0);
    sense(&mut b, &p, 0);
    p.insert("player".into(), [4., 0.85, 5.]);
    let ga = sense(&mut a, &p, 1);
    p.insert("player".into(), [-4., 0.85, 5.]);
    let gb = sense(&mut b, &p, 1);
    assert_eq!(
        ga["guard"].as_ref().unwrap().position,
        gb["guard"].as_ref().unwrap().position
    );
    assert_eq!(a.simulation.npcs["guard"], b.simulation.npcs["guard"]);
    a.scene
        .entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .enabled = false;
    sense(&mut a, &p, 2);
    assert_eq!(
        a.simulation.npcs["guard"].objectives.reason,
        Reason::Disabled
    );
    assert!(a.simulation.npcs["guard"].memory.is_none());
}
#[test]
fn switch_cannot_consume_previous_navigation_success_and_wait_does_not_advance_search() {
    let mut e = Engine::new(scene(vec![
        objective(20, Condition::VisibleTarget, Action::Wait),
        objective(10, Condition::Evidence, Action::Respond),
    ]))
    .unwrap();
    e.scene
        .entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .target = Some("player".into());
    let mut p = e.simulation.positions();
    p.insert("player".into(), [0., 0.85, -2.]);
    sense(&mut e, &p, 0);
    assert_eq!(e.simulation.npcs["guard"].phase, Phase::Suspended);
    let nav = e.simulation.navigation.get_mut("guard").unwrap();
    nav.status = "arrived".into();
    nav.target_position = Some([0., 0.85, -2.]);
    p.insert("player".into(), [0., 0.85, 5.]);
    let goals = sense(&mut e, &p, 1);
    assert!(goals["guard"].is_some());
    assert_eq!(e.simulation.npcs["guard"].phase, Phase::Investigating);
    assert_eq!(e.simulation.npcs["guard"].search_started_tick, None);
}
#[test]
fn validation_is_atomic_and_replay_prefix_history_include_objective_state() {
    let mut e = Engine::new(scene(vec![
        objective(10, Condition::Evidence, Action::Respond),
        objective(
            1,
            Condition::Always,
            Action::Guard {
                point: [3., 0.],
                arrival_distance: 0.3,
            },
        ),
    ]))
    .unwrap();
    step(&mut e, 5);
    sound(&mut e, -3.);
    step(&mut e, 8);
    let replay = e.replay();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let prefix = Engine::replay_at(&replay, Some(5)).unwrap();
    assert_eq!(
        prefix.simulation.npcs["guard"].objectives.action,
        Some(Kind::Guard)
    );
    let before = e.snapshot();
    let mut bad = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "guard")
        .unwrap()
        .clone();
    bad.npc.as_mut().unwrap().objectives = Some(vec![]);
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(bad)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    for actions in [
        vec![objective(0, Condition::Always, Action::Wait); 9],
        vec![objective(
            0,
            Condition::Always,
            Action::Flee { distance: f32::NAN },
        )],
        vec![objective(
            0,
            Condition::Always,
            Action::Guard {
                point: [0., 0.],
                arrival_distance: 0.,
            },
        )],
    ] {
        assert!(Engine::new(scene(actions)).is_err());
    }
    let mut old = scene(vec![objective(0, Condition::Always, Action::Wait)]);
    old.version = 14;
    assert!(Engine::new(old).is_err());
    let mut bad = scene(vec![objective(0, Condition::Always, Action::Patrol)]);
    bad.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .patrol = None;
    assert!(Engine::new(bad).is_err());
    let mut changed = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "guard")
        .unwrap()
        .clone();
    changed.npc.as_mut().unwrap().objectives = None;
    e.execute(
        Command::Upsert {
            entity: Box::new(changed),
        }
        .into(),
    )
    .unwrap();
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.simulation.npcs["guard"].objectives, State::default());
    e.execute(Command::Redo.into()).unwrap();
    assert!(
        e.scene
            .entities
            .iter()
            .find(|e| e.id == "guard")
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .objectives
            .is_none()
    );
}
#[test]
fn search_wall_clock_expires_while_suspended_and_legacy_remains_optional() {
    let mut s = scene(vec![objective(1, Condition::Always, Action::Wait)]);
    let mut e = Engine::new(s.clone()).unwrap();
    let state = e.simulation.npcs.get_mut("guard").unwrap();
    state.memory = Some(evidence(Source::Sound));
    state.search_started_tick = Some(0);
    state.search_point = Some([2., 0.85, 0.]);
    let p = e.simulation.positions();
    sense(&mut e, &p, 3);
    assert_eq!(e.simulation.npcs["guard"].search_point_index, 0);
    assert_eq!(e.simulation.npcs["guard"].search_started_tick, Some(0));
    sense(&mut e, &p, 10);
    assert!(e.simulation.npcs["guard"].memory.is_none());
    s.entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .objectives = None;
    let mut legacy = Engine::new(s).unwrap();
    sound(&mut legacy, -3.);
    step(&mut legacy, 1);
    assert_eq!(legacy.simulation.npcs["guard"].phase, Phase::Investigating);
    assert_eq!(
        legacy.simulation.npcs["guard"].objectives.reason,
        Reason::Legacy
    );
}
