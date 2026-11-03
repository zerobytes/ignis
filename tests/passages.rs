use ignis::{
    engine::{Command, Engine},
    model::{Entity, RoomRecipe, Scene},
    passages::{Config, Phase, Reason},
};
use serde_json::json;
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 18.,
        depth: 14.,
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
        .position = [-6., 0.85, 4.];
    for (id, x, goal, priority) in [("alpha", -4., 6., 10), ("beta", 4., -6., 0)] {
        let e:Entity=serde_json::from_value(json!({"id":id,"name":id,"body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[x,0.85,0],"size":[0.7,1.6,0.7]},"npc":{"surface":"floor","speed":2,"stop_distance":0.3,"enabled":true,"passage_priority":priority,"navigation_recovery":{"retry_ticks":15,"max_retry_ticks":60,"stalled_ticks":60,"min_progress":0.1},"objectives":[{"priority":1,"when":"always","action":{"kind":"guard","point":[goal,0],"arrival_distance":0.3}}]}})).unwrap();
        s.entities.push(e);
    }
    for (id, z) in [("north", 4.15), ("south", -4.15)] {
        s.entities.push(serde_json::from_value(json!({"id":id,"name":id,"body":"fixed","color":[0.2,0.65,0.45],"transform":{"position":[0,1.5,z],"size":[0.4,3,6.9]}})).unwrap());
    }
    s.passages = vec![Config {
        id: "door".into(),
        surface: "floor".into(),
        a: [-1., 0.],
        b: [1., 0.],
        width: 1.4,
        wait_a: vec![[-3., 2.], [-3., -2.]],
        wait_b: vec![[3., 2.], [3., -2.]],
        lease_ticks: 600,
        retry_ticks: 60,
        aging_ticks: 60,
    }];
    s
}
fn step(e: &mut Engine, n: u32) {
    e.execute(Command::Step { ticks: n }.into()).unwrap();
}
fn owner(e: &Engine) -> Option<&str> {
    e.simulation.passages.slots[0]
        .owner
        .map(|i| e.scene.entities[i as usize].id.as_str())
}
#[test]
fn opposite_agents_cross_exclusively_and_resume_original_goals_with_replay() {
    let mut e = Engine::new(scene()).unwrap();
    step(&mut e, 1);
    assert_eq!(owner(&e), Some("alpha"));
    assert_eq!(e.simulation.passages.agents["beta"].phase, Phase::Waiting);
    let mut saw_beta = false;
    for _ in 0..900 {
        step(&mut e, 1);
        let states = &e.simulation.passages.agents;
        assert!(
            states
                .values()
                .filter(|a| a.phase == Phase::Crossing)
                .count()
                <= 1
        );
        saw_beta |= owner(&e) == Some("beta");
    }
    assert!(saw_beta);
    let p = e.simulation.positions();
    assert!(
        p["alpha"][0] > 5.5 && p["beta"][0] < -5.5,
        "{p:?} {:?}",
        e.simulation.passages
    );
    assert_eq!(e.simulation.passages.agents["alpha"].completions, 1);
    assert_eq!(e.simulation.passages.agents["beta"].completions, 1);
    let replay = e.replay();
    let restored = Engine::replay_at(&replay, None).unwrap();
    assert_eq!(e.simulation.passages, restored.simulation.passages);
    assert_eq!(e.simulation.navigation, restored.simulation.navigation);
}
#[test]
fn priority_tie_is_stable_and_reservation_is_not_preempted() {
    let mut s = scene();
    for e in &mut s.entities {
        if let Some(n) = &mut e.npc {
            n.passage_priority = Some(0);
        }
    }
    let mut reversed = s.clone();
    reversed.entities.reverse();
    let mut a = Engine::new(s).unwrap();
    let mut b = Engine::new(reversed).unwrap();
    step(&mut a, 1);
    step(&mut b, 1);
    assert_eq!(owner(&a), Some("alpha"));
    assert_eq!(owner(&b), Some("alpha"));
    step(&mut a, 61);
    assert_eq!(owner(&a), Some("alpha"));
    assert!(a.simulation.passages.agents["beta"].effective_priority >= 1);
}
#[test]
fn occupied_passage_defers_grant_and_invalid_configuration_preserves_runtime() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [0., 0.85, 0.];
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 50);
    assert_eq!(owner(&e), None);
    assert!(e.simulation.passages.slots[0].occupied);
    assert_eq!(
        e.simulation.passages.agents["alpha"].reason,
        Reason::Occupied
    );
    let before = e.snapshot();
    let mut bad = e.scene.passages.clone();
    bad[0].wait_a.clear();
    assert!(
        e.execute(Command::SetPassages { passages: bad }.into())
            .is_err()
    );
    assert_eq!(before, e.snapshot());
    e.execute(
        serde_json::from_value::<Command>(json!({"op":"set_input","input":{"x":1,"z":1}}))
            .unwrap()
            .into(),
    )
    .unwrap();
    step(&mut e, 60);
    assert!(e.simulation.passages.slots[0].grants > 0);
}
#[test]
fn lease_timeout_releases_owner_and_preserves_waiter_age() {
    let mut s = scene();
    s.passages[0].lease_ticks = 30;
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 31);
    assert_eq!(e.simulation.passages.agents["alpha"].phase, Phase::Cooldown);
    assert_eq!(e.simulation.passages.agents["alpha"].timeouts, 1);
    assert_eq!(owner(&e), Some("beta"));
    assert_eq!(e.simulation.passages.agents["beta"].requested_tick, 0);
}
#[test]
fn legacy_reset_and_versions_are_explicit() {
    let mut s = scene();
    s.version = 17;
    assert!(s.validate().is_err());
    s.passages.clear();
    for e in &mut s.entities {
        if let Some(n) = &mut e.npc {
            n.passage_priority = None;
        }
    }
    assert!(s.validate().is_ok());
    let mut old = Engine::new(s).unwrap();
    step(&mut old, 10);
    assert!(old.simulation.passages.agents.is_empty());
    let mut e = Engine::new(scene()).unwrap();
    step(&mut e, 10);
    e.execute(Command::Reset.into()).unwrap();
    assert_eq!(owner(&e), None);
    assert_eq!(e.simulation.passages.slots[0].grants, 0);
    let mut bad = scene();
    bad.passages[0].b = bad.passages[0].a;
    assert!(bad.validate().is_err());
}

#[test]
fn aged_waiter_beats_new_priority_and_cancellation_preserves_sensory_privacy() {
    let mut s = scene();
    s.passages[0].aging_ticks = 1;
    let mut e = Engine::new(s).unwrap();
    // Pure coordinator snapshots: beta waits at its side, alpha has no crossing intent yet.
    let mut runtime = e.simulation.passages.clone();
    let positions = e.simulation.positions();
    let mut npcs = e.simulation.npcs.clone();
    for n in npcs.values_mut() {
        n.objectives.changed_this_tick = false;
    }
    let navgoal = |x| {
        Some(ignis::navigation::NavGoal {
            position: [x, 0.85, 0.],
            excluded_entity: None,
            target_half_height: 0.8,
            stop_distance: 0.3,
            remaining_travel: None,
            replan_on_target_change: false,
        })
    };
    let mut goals =
        std::collections::BTreeMap::from([("alpha".into(), None), ("beta".into(), navgoal(-6.))]);
    // Player occupies the lane so both requests can be compared before granting.
    let mut blocked = positions.clone();
    blocked.insert("player".into(), [0., 0.85, 0.]);
    runtime.update(
        &e.scene,
        &e.simulation.actors,
        &blocked,
        0,
        &npcs,
        &mut goals,
        &mut e.simulation.navigation,
    );
    goals.insert("beta".into(), navgoal(-6.));
    goals.insert("alpha".into(), navgoal(6.));
    runtime.update(
        &e.scene,
        &e.simulation.actors,
        &positions,
        20,
        &npcs,
        &mut goals,
        &mut e.simulation.navigation,
    );
    assert_eq!(
        runtime.slots[0]
            .owner
            .map(|i| e.scene.entities[i as usize].id.as_str()),
        Some("beta")
    );
    assert_eq!(runtime.agents["beta"].effective_priority, 20);
    assert!(npcs.values().all(|n| n.memory.is_none()));
    goals.insert("beta".into(), None);
    goals.insert("alpha".into(), navgoal(6.));
    runtime.update(
        &e.scene,
        &e.simulation.actors,
        &positions,
        21,
        &npcs,
        &mut goals,
        &mut e.simulation.navigation,
    );
    assert_eq!(runtime.agents["beta"].phase, Phase::Idle);
    assert_eq!(
        runtime.slots[0]
            .owner
            .map(|i| e.scene.entities[i as usize].id.as_str()),
        Some("alpha")
    );
}

#[test]
fn yielding_keeps_distance_budget_but_sound_memory_still_expires() {
    let mut s = scene();
    let beta = s
        .entities
        .iter_mut()
        .find(|e| e.id == "beta")
        .unwrap()
        .npc
        .as_mut()
        .unwrap();
    beta.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":12,"fov_degrees":120,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":1,"max_investigation_distance":0.2,"search_turn_degrees_per_second":90,"hearing":{"range":20}})).unwrap());
    beta.objectives = Some(
        serde_json::from_value(
            json!([{"priority":1,"when":"evidence","action":{"kind":"respond"}}]),
        )
        .unwrap(),
    );
    let mut e = Engine::new(s).unwrap();
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position {
                position: [-6., 0.85, 0.],
            },
            radius: 20.,
        }
        .into(),
    )
    .unwrap();
    step(&mut e, 30);
    assert_eq!(e.simulation.passages.agents["beta"].phase, Phase::Waiting);
    assert!(e.simulation.npcs["beta"].memory.is_some());
    assert_eq!(e.simulation.npcs["beta"].investigation_distance, 0.);
    step(&mut e, 40);
    assert!(e.simulation.npcs["beta"].memory.is_none());
    assert_eq!(e.simulation.passages.agents["beta"].phase, Phase::Idle);
}

#[test]
fn unreserved_sweep_is_stopped_but_retreat_from_an_occupied_lane_is_allowed() {
    let e = Engine::new(scene()).unwrap();
    let runtime = &e.simulation.passages;
    let mut positions = e.simulation.positions();
    positions.insert("alpha".into(), [-1.5, 0.85, 0.]);
    let mut velocities = std::collections::BTreeMap::from([("alpha".into(), [10., 0.])]);
    let mut nav = e.simulation.navigation.clone();
    runtime.gate(&e.scene, &positions, &mut velocities, &mut nav);
    assert_eq!(velocities["alpha"], [0., 0.]);
    assert_eq!(nav["alpha"].reason, "reservation_required");
    positions.insert("alpha".into(), [-0.5, 0.85, 0.]);
    velocities.insert("alpha".into(), [-2., 0.]);
    runtime.gate(&e.scene, &positions, &mut velocities, &mut nav);
    assert_eq!(velocities["alpha"], [-2., 0.]);
}

#[test]
fn z_axis_and_three_agent_queue_use_distinct_slots_and_complete() {
    let mut s = scene();
    let mut gamma = s.entities.iter().find(|e| e.id == "beta").unwrap().clone();
    gamma.id = "gamma".into();
    gamma.name = "Gamma".into();
    gamma.transform.position = [6., 0.85, 0.];
    gamma.npc.as_mut().unwrap().objectives.as_mut().unwrap()[0].action =
        ignis::objectives::Action::Guard {
            point: [-5., 0.5],
            arrival_distance: 0.3,
        };
    s.entities.push(gamma);
    s.passages[0].wait_a.push([-3., 4.]);
    s.passages[0].wait_b.push([3., 4.]);
    // Rotate the whole authoring document 90 degrees by exchanging X/Z.
    for e in &mut s.entities {
        e.transform.position.swap(0, 2);
        e.transform.size.swap(0, 2);
        if let Some(n) = &mut e.npc
            && let Some(actions) = &mut n.objectives
        {
            for action in actions {
                if let ignis::objectives::Action::Guard { point, .. } = &mut action.action {
                    point.swap(0, 1);
                }
            }
        }
    }
    let c = &mut s.passages[0];
    c.a.swap(0, 1);
    c.b.swap(0, 1);
    for p in c.wait_a.iter_mut().chain(c.wait_b.iter_mut()) {
        p.swap(0, 1);
    }
    let mut e = Engine::new(s).unwrap();
    step(&mut e, 1);
    assert_ne!(
        e.simulation.passages.agents["beta"].slot,
        e.simulation.passages.agents["gamma"].slot
    );
    step(&mut e, 1200);
    assert!(
        e.simulation
            .passages
            .agents
            .values()
            .all(|a| a.completions == 1),
        "{:?}",
        e.simulation.passages
    );
}

#[test]
fn authoring_rejects_overlap_bad_slots_unknown_surfaces_and_diagonal_lanes() {
    let mut cases = vec![];
    let mut s = scene();
    s.passages[0].b = [1., 1.];
    cases.push(s);
    let mut s = scene();
    s.passages[0].wait_a[0] = [-3., 0.];
    cases.push(s);
    let mut s = scene();
    s.passages[0].wait_b[1] = s.passages[0].wait_b[0];
    cases.push(s);
    let mut s = scene();
    s.passages[0].surface = "missing".into();
    cases.push(s);
    let mut s = scene();
    let mut duplicate = s.passages[0].clone();
    duplicate.id = "overlap".into();
    s.passages.push(duplicate);
    cases.push(s);
    for s in cases {
        assert!(s.validate().is_err());
    }
}

#[test]
fn death_releases_reservation_in_same_tick_without_removing_body() {
    let mut s = scene();
    s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[{"id":"expire","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"source","amount":1}]}]})).unwrap();
    let n = s.entities.iter_mut().find(|e| e.id == "alpha").unwrap();
    n.vitality = Some(ignis::vitality::Config {
        initial: 1,
        maximum: 1,
        scale: 1,
    });
    n.abilities = Some(ignis::abilities::Loadout {
        resources: vec![],
        grants: vec!["expire".into()],
    });
    let mut e = Engine::new(s.clone()).unwrap();
    step(&mut e, 1);
    assert_eq!(owner(&e), Some("alpha"));
    e.execute(
        Command::ActivateAbility {
            actor: "alpha".into(),
            slot: 1,
            target: None,
        }
        .into(),
    )
    .unwrap();
    step(&mut e, 1);
    assert_ne!(owner(&e), Some("alpha"));
    assert_eq!(e.simulation.passages.agents["alpha"].phase, Phase::Idle);
    assert!(e.simulation.position("alpha").is_some());
    assert_eq!(
        e.simulation.npcs["alpha"].phase,
        ignis::perception::Phase::Dead
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);

    let mut e = Engine::new(s).unwrap();
    for _ in 0..300 {
        step(&mut e, 1);
        if e.simulation.position("alpha").unwrap()[0] > -0.2 {
            break;
        }
    }
    assert!(e.simulation.position("alpha").unwrap()[0].abs() < 0.25);
    assert_eq!(owner(&e), Some("alpha"));
    e.execute(
        Command::ActivateAbility {
            actor: "alpha".into(),
            slot: 1,
            target: None,
        }
        .into(),
    )
    .unwrap();
    step(&mut e, 1);
    assert_eq!(owner(&e), None);
    assert!(e.simulation.passages.slots[0].occupied);
    assert_eq!(e.simulation.passages.agents["beta"].phase, Phase::Waiting);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
