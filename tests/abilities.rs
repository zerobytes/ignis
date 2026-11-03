use ignis::{
    abilities::*,
    engine::{Command, Engine},
    model::{RoomRecipe, Scene},
    objectives::{Action, Condition, Objective},
};
use serde_json::json;
fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        width: 24.,
        depth: 20.,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":100},{"id":"focus","scale":1}],"abilities":[
 {"id":"convert","selection":"self_actor","range":0,"relation":null,"cast_ticks":3,"cooldown_ticks":4,"costs":[{"resource":"energy","units":3},{"resource":"energy","units":2}],"effects":[{"kind":"resource","recipient":"source","resource":"focus","delta":10}]},
 {"id":"support","selection":"actor","range":6,"relation":"allied","cast_ticks":3,"cooldown_ticks":4,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"resource","recipient":"target","resource":"focus","delta":10}]},
 {"id":"signal","selection":"self_actor","range":0,"relation":null,"cast_ticks":0,"cooldown_ticks":1,"costs":[],"effects":[{"kind":"noise","radius":12}]}
 ]})).unwrap();
    let loadout = Loadout {
        resources: vec![
            Pool {
                resource: "energy".into(),
                initial: 20,
                capacity: 20,
            },
            Pool {
                resource: "focus".into(),
                initial: 0,
                capacity: 12,
            },
        ],
        grants: vec!["convert".into(), "support".into(), "signal".into()],
    };
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [0., 0.8, -3.];
    p.faction = Some("friends".into());
    p.abilities = Some(loadout.clone());
    let mut n=serde_json::from_value::<ignis::model::Entity>(json!({"id":"npc","name":"npc","body":"npc","faction":"friends","color":[0.8,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","target":"player","speed":2,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":12,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":30,"search_turn_degrees_per_second":0,"hearing":{"range":12}},"objectives":[{"priority":1,"when":"always","action":{"kind":"wait"}}]}})).unwrap();
    n.abilities = Some(loadout);
    s.entities.push(n);
    s
}
fn activate(e: &mut Engine, actor: &str, slot: u8, target: Option<&str>) {
    e.execute(
        Command::ActivateAbility {
            actor: actor.into(),
            slot,
            target: target.map(str::to_owned),
        }
        .into(),
    )
    .unwrap();
}
fn view(e: &mut Engine) -> serde_json::Value {
    e.execute(Command::InspectAbilities.into()).unwrap()
}
fn balance(e: &mut Engine, actor: &str, resource: usize) -> i64 {
    view(e)["actors"][actor]["resources"][resource]["units"]
        .as_i64()
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
fn automatic(s: &mut Scene, slot: u8) {
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .objectives = Some(vec![
        Objective {
            priority: 2,
            when: Condition::Always,
            action: Action::UseAbility {
                policy: None,
                slot,
                retry_ticks: 8,
            },
        },
        Objective {
            priority: 1,
            when: Condition::Always,
            action: Action::Wait,
        },
    ]);
}
#[test]
fn cast_commit_cooldown_and_clamped_actual_receipts() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, None);
    assert_eq!(balance(&mut e, "player", 0), 20);
    e.advance(3).unwrap();
    assert_eq!(balance(&mut e, "player", 0), 20);
    e.advance(1).unwrap();
    assert_eq!(balance(&mut e, "player", 0), 15);
    assert_eq!(balance(&mut e, "player", 1), 10);
    assert_eq!(outcome(&e), Outcome::Completed);
    activate(&mut e, "player", 1, None);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Cooldown);
    e.advance(2).unwrap();
    activate(&mut e, "player", 1, None);
    e.advance(4).unwrap();
    assert_eq!(balance(&mut e, "player", 1), 12);
    let r = e.simulation.abilities.state.receipts.back().unwrap();
    assert_eq!(r.effects[0].unwrap().applied, 2);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn aggregated_cost_cannot_be_funded_by_own_effect() {
    let mut s = scene();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.abilities.as_mut().unwrap().resources[0].initial = 4;
    s.ability_catalog.abilities[0].effects.insert(
        0,
        Effect::Resource {
            recipient: Recipient::Source,
            resource: "energy".into(),
            delta: 20,
        },
    );
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, None);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::InsufficientResource);
    assert_eq!(balance(&mut e, "player", 0), 4);
    assert_eq!(balance(&mut e, "player", 1), 0);
}
#[test]
fn cancellation_at_deadline_beats_commit_and_duplicate_requests() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, None);
    e.advance(3).unwrap();
    let id = view(&mut e)["actors"]["player"]["cast"]["execution_id"]
        .as_u64()
        .unwrap();
    e.execute(
        Command::CancelAbility {
            actor: "player".into(),
            execution_id: id,
        }
        .into(),
    )
    .unwrap();
    activate(&mut e, "player", 1, None);
    e.advance(1).unwrap();
    assert_eq!(balance(&mut e, "player", 0), 20);
    assert_eq!(outcome(&e), Outcome::Busy);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == Outcome::Cancelled)
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn missing_pool_and_occlusion_do_not_debit() {
    for missing in [true, false] {
        let mut s = scene();
        if missing {
            s.entities.last_mut().unwrap().abilities = None;
        } else {
            s.entities.push(serde_json::from_value(json!({"id":"screen","name":"screen","body":"fixed","color":[0.2,0.3,0.4],"transform":{"position":[0,1,-1.5],"size":[3,2,0.4]}})).unwrap());
        }
        let mut e = Engine::new(s).unwrap();
        activate(&mut e, "player", 2, Some("npc"));
        e.advance(1).unwrap();
        assert_eq!(
            outcome(&e),
            if missing {
                Outcome::MissingResource
            } else {
                Outcome::Occluded
            }
        );
        assert_eq!(balance(&mut e, "player", 0), 20);
    }
}
#[test]
fn target_revalidated_after_cast_moves_out_of_range() {
    let mut s = scene();
    s.ability_catalog.abilities[1].cast_ticks = 120;
    s.ability_catalog.abilities[1].range = 3.5;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 2, Some("npc"));
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Started);
    e.execute(
        Command::SetInput {
            input: ignis::physics::Input {
                x: 1.,
                z: 0.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(120).unwrap();
    assert_eq!(outcome(&e), Outcome::OutOfRange);
    assert_eq!(balance(&mut e, "player", 0), 20);
}
#[test]
fn pending_inputs_prefix_and_malformed_suffix() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, None);
    let pending = view(&mut e);
    let r = e.replay();
    let mut reconstructed = Engine::replay_at(&r, None).unwrap();
    assert_eq!(view(&mut reconstructed), pending);
    reconstructed.advance(4).unwrap();
    e.advance(4).unwrap();
    assert_eq!(view(&mut reconstructed), view(&mut e));
    let mut r = e.replay();
    r.ability_inputs[0].actor = 99;
    assert!(Engine::replay_at(&r, Some(0)).is_err());
}
#[test]
fn npc_own_threshold_retries_and_resumes_wait() {
    let mut s = scene();
    automatic(&mut s, 1);
    s.ability_catalog.abilities[0].requires_below = Some(Amount {
        resource: "focus".into(),
        units: 10,
    });
    let mut e = Engine::new(s).unwrap();
    e.advance(5).unwrap();
    assert_eq!(balance(&mut e, "npc", 1), 10);
    assert_eq!(
        e.simulation.npcs["npc"].objectives.action,
        Some(ignis::objectives::Kind::Wait)
    );
    e.advance(30).unwrap();
    assert_eq!(e.simulation.abilities.state.receipts.len(), 2);
    let mut s = scene();
    automatic(&mut s, 1);
    s.entities
        .last_mut()
        .unwrap()
        .abilities
        .as_mut()
        .unwrap()
        .resources[0]
        .initial = 0;
    let mut e = Engine::new(s).unwrap();
    e.advance(25).unwrap();
    assert_eq!(e.simulation.abilities.state.receipts.len(), 4);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::InsufficientResource)
    );
}
#[test]
fn npc_support_requires_confirmed_sight_and_not_radio() {
    let mut s = scene();
    automatic(&mut s, 2);
    let mut e = Engine::new(s.clone()).unwrap();
    e.advance(4).unwrap();
    assert_eq!(balance(&mut e, "player", 1), 10);
    assert_eq!(balance(&mut e, "npc", 0), 15);
    s.entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .recognition = Some(ignis::recognition::Config {
        acquire_ticks: 10,
        decay_ticks: 1,
    });
    let mut e = Engine::new(s).unwrap();
    e.advance(8).unwrap();
    assert_eq!(balance(&mut e, "player", 1), 0);
    assert!(
        e.simulation
            .abilities
            .state
            .casts
            .iter()
            .all(Option::is_none)
    );
    e.advance(12).unwrap();
    assert_eq!(balance(&mut e, "player", 1), 10);
}
#[test]
fn generated_signal_is_next_tick_anonymous_and_replayed() {
    let mut s = scene();
    let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    n.target = None;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 3, None);
    e.advance(1).unwrap();
    assert!(!e.simulation.npcs["npc"].heard_this_tick);
    assert_eq!(e.simulation.abilities.state.noises[0].tick, 1);
    e.advance(1).unwrap();
    assert!(e.simulation.npcs["npc"].heard_this_tick);
    let m = e.simulation.npcs["npc"].memory.unwrap();
    assert_eq!(m.source, ignis::perception::Source::Sound);
    assert!(m.actor_index.is_none());
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn history_and_invalid_catalog_preserve_running_state() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, None);
    e.advance(1).unwrap();
    let before = e.snapshot();
    let mut invalid = e.scene.ability_catalog.clone();
    invalid.resources[0].scale = 0;
    assert!(
        e.execute(Command::SetAbilityCatalog { catalog: invalid }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut changed = e.scene.ability_catalog.clone();
    changed.abilities[0].cast_ticks = 0;
    e.execute(Command::SetAbilityCatalog { catalog: changed }.into())
        .unwrap();
    assert_eq!(e.simulation.tick, 0);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene.ability_catalog.abilities[0].cast_ticks, 3);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene.ability_catalog.abilities[0].cast_ticks, 0);
}
#[test]
fn bounded_inputs_and_receipts() {
    let mut e = Engine::new(scene()).unwrap();
    for _ in 0..16 {
        activate(&mut e, "player", 3, None);
    }
    let snapshot = e.snapshot();
    assert!(
        e.execute(
            Command::ActivateAbility {
                actor: "player".into(),
                slot: 3,
                target: None
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), snapshot);
    e.advance(1).unwrap();
    for _ in 0..260 {
        activate(&mut e, "player", 3, None);
        e.advance(1).unwrap();
    }
    assert_eq!(e.simulation.abilities.state.receipts.len(), 256);
    assert!(e.simulation.abilities.state.dropped_receipts > 0);
    assert_eq!(e.simulation.abilities.state.recent_noises.len(), 128);
    assert_eq!(e.simulation.abilities.state.noises.len(), 1);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn two_resource_costs_are_atomic_and_capacity_limits_validate() {
    let mut s = scene();
    s.ability_catalog.abilities[0].costs.push(Amount {
        resource: "focus".into(),
        units: 1,
    });
    let mut e = Engine::new(s.clone()).unwrap();
    activate(&mut e, "player", 1, None);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::InsufficientResource);
    assert_eq!(balance(&mut e, "player", 0), 20);
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .abilities
        .as_mut()
        .unwrap()
        .resources[1]
        .initial = 2;
    let mut e = Engine::new(s.clone()).unwrap();
    activate(&mut e, "player", 1, None);
    e.advance(4).unwrap();
    assert_eq!(balance(&mut e, "player", 0), 15);
    assert_eq!(balance(&mut e, "player", 1), 11);
    s.ability_catalog.abilities[0].costs[0].units = LIMIT;
    s.ability_catalog.abilities[0].costs[1].units = 1;
    assert!(s.validate().is_err());
    s = scene();
    s.entities
        .last_mut()
        .unwrap()
        .abilities
        .as_mut()
        .unwrap()
        .resources[0]
        .capacity = LIMIT + 1;
    assert!(s.validate().is_err());
    s = scene();
    s.version = 25;
    assert!(s.validate().is_err());
}
#[test]
fn other_actor_drains_resource_during_cast_and_commit_rechecks() {
    let mut s = scene();
    s.ability_catalog.abilities[1].cast_ticks = 0;
    s.ability_catalog.abilities[1].effects = vec![Effect::Resource {
        recipient: Recipient::Target,
        resource: "energy".into(),
        delta: -20,
    }];
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, None);
    e.advance(1).unwrap();
    activate(&mut e, "npc", 2, Some("player"));
    e.advance(3).unwrap();
    assert_eq!(balance(&mut e, "player", 0), 0);
    assert_eq!(balance(&mut e, "player", 1), 0);
    assert_eq!(outcome(&e), Outcome::InsufficientResource);
}
#[test]
fn higher_priority_interrupts_cast_and_retry_allows_resume() {
    let mut s = scene();
    automatic(&mut s, 1);
    s.ability_catalog.abilities[0].cast_ticks = 30;
    let npc = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
    npc.target = None;
    npc.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":10,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":2,"search_ticks":1,"search_turn_degrees_per_second":0,"hearing":{"range":12}})).unwrap());
    npc.objectives.as_mut().unwrap().insert(
        0,
        Objective {
            priority: 10,
            when: Condition::SoundMemory,
            action: Action::Wait,
        },
    );
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert!(
        e.simulation
            .abilities
            .state
            .casts
            .iter()
            .any(Option::is_some)
    );
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position {
                position: [0., 0.8, 1.],
            },
            radius: 12.,
        }
        .into(),
    )
    .unwrap();
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Cancelled);
    assert_eq!(balance(&mut e, "npc", 0), 20);
    e.advance(8).unwrap();
    assert!(
        e.simulation
            .abilities
            .state
            .casts
            .iter()
            .any(Option::is_some)
    );
    e.advance(30).unwrap();
    assert_eq!(balance(&mut e, "npc", 1), 10);
}
#[test]
fn radio_memory_cannot_target_hidden_actor_or_leak_position() {
    let mut receipts = Vec::new();
    for x in [-6., 6.] {
        let mut s = scene();
        automatic(&mut s, 2);
        let n = s.entities.last_mut().unwrap().npc.as_mut().unwrap();
        n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":0.1,"fov_degrees":90,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":30,"search_turn_degrees_per_second":0})).unwrap());
        s.entities
            .iter_mut()
            .find(|e| e.id == "player")
            .unwrap()
            .transform
            .position[0] = x;
        let mut e = Engine::new(s).unwrap();
        let player = e
            .simulation
            .actors
            .actors
            .iter()
            .position(|&i| e.scene.entities[i as usize].id == "player")
            .unwrap() as u16;
        let state = e.simulation.npcs.get_mut("npc").unwrap();
        state.memory = Some(ignis::perception::Observation {
            instance: None,
            life: Some(0),
            position: [0., 0.8, -3.],
            tick: 0,
            source: ignis::perception::Source::Report,
            actor_index: Some(player),
            noise_id: None,
        });
        activate(&mut e, "npc", 2, Some("player"));
        e.advance(1).unwrap();
        assert_eq!(outcome(&e), Outcome::TargetUnavailable);
        assert_eq!(balance(&mut e, "npc", 0), 20);
        receipts.push(e.simulation.abilities.state.receipts.clone());
    }
    assert_eq!(receipts[0], receipts[1]);
}
#[test]
fn simultaneous_requests_keep_stable_actor_order_and_independent_source_noise_budgets() {
    let mut s = scene();
    s.ability_catalog.abilities[2].costs = vec![Amount {
        resource: "energy".into(),
        units: 5,
    }];
    s.ability_catalog.abilities[2].effects = vec![Effect::Noise { radius: 12. }; 8];
    let mut third = s.entities.last().unwrap().clone();
    third.id = "third".into();
    third.transform.position[0] = 4.;
    s.entities.push(third);
    let mut reversed = s.clone();
    reversed.entities.reverse();
    let mut results = Vec::new();
    for s in [s, reversed] {
        let mut e = Engine::new(s).unwrap();
        for id in ["third", "player", "npc"] {
            activate(&mut e, id, 3, None);
        }
        e.advance(1).unwrap();
        for actor in ["npc", "player", "third"] {
            assert_eq!(balance(&mut e, actor, 0), 15);
        }
        assert_eq!(outcome(&e), Outcome::Completed);
        assert_eq!(e.simulation.abilities.state.noises.len(), 24);
        assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
        results.push(view(&mut e));
    }
    assert_eq!(results[0], results[1]);
}
#[test]
fn ability_objective_does_not_freeze_committed_flight() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-spatial-patrol.scene.json")).unwrap();
    s.version = 26;
    s.ability_catalog = scene().ability_catalog;
    s.ability_catalog.abilities[0].cast_ticks = 30;
    let flyer = s.entities.iter_mut().find(|e| e.id == "flyer").unwrap();
    flyer.abilities = scene().entities.last().unwrap().abilities.clone();
    flyer.npc.as_mut().unwrap().objectives = Some(vec![
        Objective {
            priority: 2,
            when: Condition::SoundMemory,
            action: Action::UseAbility {
                policy: None,
                slot: 1,
                retry_ticks: 60,
            },
        },
        Objective {
            priority: 1,
            when: Condition::Always,
            action: Action::Patrol,
        },
    ]);
    let mut e = Engine::new(s).unwrap();
    e.advance(40).unwrap();
    assert_eq!(
        e.simulation.mobility.states["flyer"].phase,
        ignis::mobility::Phase::Traversing
    );
    let before = e.simulation.position("flyer").unwrap();
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position { position: before },
            radius: 12.,
        }
        .into(),
    )
    .unwrap();
    e.advance(5).unwrap();
    let slot = e
        .simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.scene.entities[i as usize].id == "flyer")
        .unwrap();
    assert!(e.simulation.abilities.state.casts[slot].is_some());
    let after = e.simulation.position("flyer").unwrap();
    assert!(glam::Vec3::from(before).distance(glam::Vec3::from(after)) > 0.05);
}
#[test]
fn npc_loses_sight_during_cast_and_commit_does_not_follow_memory() {
    let mut s = scene();
    automatic(&mut s, 2);
    s.ability_catalog.abilities[1].cast_ticks = 60;
    if let Some(ignis::model::Awareness::Perceptive { vision_range, .. }) = &mut s
        .entities
        .last_mut()
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        *vision_range = 3.5;
    }
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Started);
    e.execute(
        Command::SetInput {
            input: ignis::physics::Input {
                x: 1.,
                z: 0.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(60).unwrap();
    assert_eq!(outcome(&e), Outcome::TargetUnavailable);
    assert_eq!(balance(&mut e, "npc", 0), 20);
    assert_eq!(balance(&mut e, "player", 1), 0);
    assert_eq!(view(&mut e)["actors"]["npc"]["grants"][1]["retry_at"], 68);
}
#[test]
fn future_cast_does_not_reserve_current_tick_noise_budget() {
    let mut s = scene();
    s.ability_catalog.abilities[2].effects = vec![Effect::Noise { radius: 12. }; 8];
    let mut delayed = s.ability_catalog.abilities[2].clone();
    delayed.id = "delayed".into();
    delayed.cast_ticks = 1;
    s.ability_catalog.abilities.push(delayed);
    let mut third = s.entities.last().unwrap().clone();
    third.id = "third".into();
    third.transform.position[0] = 4.;
    third
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("delayed".into());
    s.entities.push(third);
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "npc", 3, None);
    activate(&mut e, "player", 3, None);
    activate(&mut e, "third", 4, None);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Started);
    assert_eq!(e.simulation.abilities.state.noises.len(), 16);
    e.advance(1).unwrap();
    assert_eq!(outcome(&e), Outcome::Completed);
    assert_eq!(e.simulation.abilities.state.noises.len(), 8);
}
