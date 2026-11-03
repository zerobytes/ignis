use ignis::{
    engine::{Command, Engine},
    generator_control::{Action, Status},
    generators::Generator,
    model::{SCENE_VERSION, Scene},
    population::{Operation, Outcome},
    waves::Phase,
};
use serde_json::{Value, json};
fn scene(waves: bool) -> Scene {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-waves.scene.json")).unwrap();
    s.version = SCENE_VERSION;
    let g = &mut s.npc_generators[0];
    g.target_alive = 1;
    g.interval_ticks = 4;
    g.max_births = 5;
    g.retire_dead_after_ticks = Some(2);
    if waves {
        g.waves.truncate(2);
        g.waves[0].count = 1;
        g.waves[1].count = 1;
        g.waves[1].delay_ticks = 5;
    } else {
        g.waves.clear();
    }
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn control(e: &mut Engine, action: Action) -> Value {
    run(
        e,
        Command::ControlNpcGenerator {
            generator: "encounter".into(),
            action,
        },
    )
}
fn state(e: &Engine) -> Generator {
    e.simulation.generators.generators[0].clone()
}
fn info(e: &mut Engine) -> Value {
    run(e, Command::InspectGenerators)
}
fn last(e: &Engine) -> Outcome {
    e.simulation.population.receipts.back().unwrap().outcome
}
fn name(e: &Engine) -> String {
    let m = state(e).members.into_iter().flatten().next().unwrap();
    e.simulation
        .actors
        .label(e.simulation.definitions(), m.instance)
        .unwrap()
        .to_owned()
}
fn cast(e: &mut Engine, slot: u8, target: &str) {
    run(
        e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot,
            target: Some(target.into()),
        },
    );
    e.advance(1).unwrap();
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn inactive_start_freezes_initial_and_wave_delay_and_pending_replay() {
    let mut s = scene(true);
    let g = &mut s.npc_generators[0];
    g.initially_active = false;
    g.initial_delay_ticks = 2;
    g.waves[0].delay_ticks = 3;
    let mut e = Engine::new(s).unwrap();
    e.advance(10).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(info(&mut e)["controls"][0]["remaining_delay_ticks"], 5);
    control(&mut e, Action::Activate);
    assert_eq!(state(&e).status, Status::Paused);
    replay(&e);
    let prefix = Engine::replay_at(&e.replay(), Some(10)).unwrap();
    assert_eq!(state(&prefix), state(&e));
    e.advance(5).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(last(&e), Outcome::ControlApplied);
    assert_eq!(state(&e).wave.unwrap().due_tick, 15);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    replay(&e);
}
#[test]
fn pause_at_deadline_preserves_rng_and_remaining_cadence() {
    let mut e = Engine::new(scene(false)).unwrap();
    e.advance(1).unwrap();
    let old = name(&e);
    run(&mut e, Command::DespawnActor { actor: old });
    e.advance(3).unwrap(); // now tick4 deadline
    let rng = state(&e).rng;
    control(&mut e, Action::Pause);
    e.advance(10).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(state(&e).rng, rng);
    assert_eq!(info(&mut e)["controls"][0]["remaining_cadence_ticks"], 0);
    control(&mut e, Action::Pause);
    e.advance(1).unwrap();
    assert_eq!(last(&e), Outcome::ControlUnchanged);
    assert_eq!(state(&e).paused_since, Some(4));
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    replay(&e);
}
#[test]
fn partial_cadence_and_interwave_delay_resume_remaining_ticks() {
    let mut e = Engine::new(scene(true)).unwrap();
    e.advance(1).unwrap();
    let old = name(&e);
    run(&mut e, Command::DespawnActor { actor: old });
    e.advance(1).unwrap();
    assert_eq!(state(&e).wave.unwrap().due_tick, 6);
    e.advance(1).unwrap();
    control(&mut e, Action::Pause);
    e.advance(20).unwrap();
    assert_eq!(info(&mut e)["controls"][0]["remaining_delay_ticks"], 3);
    control(&mut e, Action::Activate);
    e.advance(3).unwrap();
    assert_eq!(state(&e).births, 1);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    replay(&e);
    let mut e = Engine::new(scene(false)).unwrap();
    e.advance(1).unwrap();
    let old = name(&e);
    run(&mut e, Command::DespawnActor { actor: old });
    control(&mut e, Action::Pause);
    e.advance(9).unwrap();
    assert_eq!(info(&mut e)["controls"][0]["remaining_cadence_ticks"], 3);
    control(&mut e, Action::Activate);
    e.advance(3).unwrap();
    assert_eq!(state(&e).births, 1);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    replay(&e);
}
#[test]
fn paused_world_runs_damage_retirement_but_defers_wave_completion() {
    let mut e = Engine::new(scene(true)).unwrap();
    e.advance(1).unwrap();
    let old = name(&e);
    control(&mut e, Action::Pause);
    cast(&mut e, 1, &old); // damage still commits
    assert_eq!(state(&e).status, Status::Paused);
    e.advance(3).unwrap();
    assert!(state(&e).members.iter().all(Option::is_none));
    assert_eq!(state(&e).wave.unwrap().phase, Phase::WaitingForClear);
    assert_eq!(e.simulation.generators.wave_events.len(), 1);
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    assert_eq!(state(&e).wave.unwrap().current, 1);
    assert_eq!(state(&e).wave.unwrap().phase, Phase::Delay);
    replay(&e);
}
#[test]
fn revival_during_pause_is_observed_on_resume_without_reopening_completed_history() {
    let mut s = scene(true);
    s.npc_generators[0].retire_dead_after_ticks = None;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let old = name(&e);
    control(&mut e, Action::Pause);
    cast(&mut e, 1, &old);
    e.advance(4).unwrap();
    cast(&mut e, 2, &old);
    let member = state(&e).members.into_iter().flatten().next().unwrap();
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    assert_eq!(state(&e).wave.unwrap().current, 0);
    assert_eq!(state(&e).births, 1);
    assert_eq!(state(&e).members[0].unwrap().instance, member.instance);
    assert_eq!(info(&mut e)["counts"][0]["alive"], 1);
    replay(&e);
}
#[test]
fn cancel_is_terminal_preserves_survivors_and_continues_corpse_retirement() {
    let mut e = Engine::new(scene(false)).unwrap();
    e.advance(1).unwrap();
    let old = name(&e);
    let m = state(&e).members[0].unwrap();
    let rng = state(&e).rng;
    control(&mut e, Action::Cancel);
    e.advance(1).unwrap();
    assert!(e.simulation.actors.matches(m.actor, m.instance));
    assert_eq!(state(&e).cancelled_tick, Some(1));
    control(&mut e, Action::Activate);
    control(&mut e, Action::Pause);
    control(&mut e, Action::Cancel);
    e.advance(1).unwrap();
    let outcomes: Vec<_> = e
        .simulation
        .population
        .receipts
        .iter()
        .map(|r| r.outcome)
        .collect();
    assert_eq!(
        outcomes,
        vec![
            Outcome::ControlApplied,
            Outcome::ControlTerminal,
            Outcome::ControlTerminal,
            Outcome::ControlUnchanged
        ]
    );
    cast(&mut e, 1, &old);
    e.advance(20).unwrap();
    assert!(state(&e).members.iter().all(Option::is_none));
    assert_eq!(state(&e).births, 1);
    assert_eq!(state(&e).rng, rng);
    assert_eq!(info(&mut e)["controls"][0]["terminal"], "cancelled");
    replay(&e);
}
#[test]
fn completed_wave_and_exhausted_continuous_generator_never_restart() {
    for waves in [false, true] {
        let mut s = scene(waves);
        s.npc_generators[0].max_births = 1;
        if waves {
            s.npc_generators[0].waves.truncate(1);
        }
        let mut e = Engine::new(s).unwrap();
        e.advance(1).unwrap();
        if waves {
            let old = name(&e);
            run(&mut e, Command::DespawnActor { actor: old });
            e.advance(1).unwrap();
        }
        let before = state(&e);
        for action in [Action::Activate, Action::Pause, Action::Cancel] {
            control(&mut e, action);
        }
        e.advance(2).unwrap();
        assert_eq!(state(&e), before);
        assert!(
            e.simulation
                .population
                .receipts
                .iter()
                .rev()
                .take(3)
                .all(|r| r.outcome == Outcome::ControlTerminal)
        );
        replay(&e);
    }
}
#[test]
fn same_tick_order_is_sequential_and_does_not_spawn_between_commands() {
    let mut e = Engine::new(scene(false)).unwrap();
    control(&mut e, Action::Activate);
    control(&mut e, Action::Pause);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(last(&e), Outcome::ControlApplied);
    control(&mut e, Action::Activate);
    control(&mut e, Action::Cancel);
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(last(&e), Outcome::ControlTerminal);
    replay(&e);
    let mut e = Engine::new(scene(false)).unwrap();
    control(&mut e, Action::Pause);
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    let old = name(&e);
    control(&mut e, Action::Pause);
    run(&mut e, Command::DespawnActor { actor: old });
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [0., 0.8, 0.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    assert_eq!(last(&e), Outcome::Spawned);
    assert!(state(&e).members.iter().all(Option::is_none));
    e.advance(10).unwrap();
    assert_eq!(state(&e).births, 1);
    replay(&e);
}
#[test]
fn invalid_commands_and_input_budgets_are_atomic() {
    let mut e = Engine::new(scene(false)).unwrap();
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::ControlNpcGenerator {
                generator: "missing".into(),
                action: Action::Pause
            }
            .into()
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<Command>(
            json!({"op":"control_npc_generator","generator":"encounter","action":"rewind"})
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    for _ in 0..64 {
        control(&mut e, Action::Activate);
    }
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::ControlNpcGenerator {
                generator: "encounter".into(),
                action: Action::Pause
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.population.processed, 64);
    for _ in 1..64 {
        for _ in 0..64 {
            control(&mut e, Action::Activate);
        }
        e.advance(1).unwrap();
    }
    assert_eq!(e.replay().population_inputs.len(), 4096);
    assert_eq!(e.simulation.population.receipts.len(), 256);
    assert_eq!(e.simulation.population.dropped_receipts, 3840);
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::ControlNpcGenerator {
                generator: "encounter".into(),
                action: Action::Cancel
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    replay(&e);
}
#[test]
fn replay_checks_control_state_and_rejects_invalid_indices() {
    let mut e = Engine::new(scene(true)).unwrap();
    control(&mut e, Action::Pause);
    e.advance(5).unwrap();
    let prefix = state(&e);
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    control(&mut e, Action::Cancel);
    let r = e.replay();
    replay(&e);
    assert_eq!(state(&Engine::replay_at(&r, Some(5)).unwrap()), prefix);
    let mut bad = r.clone();
    bad.population_inputs[0].operation = Operation::ControlGenerator {
        generator: 255,
        action: Action::Pause,
    };
    assert!(Engine::verify_replay(&bad).is_err());
    let mut encoded = serde_json::to_value(&r).unwrap();
    encoded["expected_generators"]["generators"][0]["status"] = json!("cancelled");
    let bad = serde_json::from_value(encoded).unwrap();
    assert_eq!(Engine::verify_replay(&bad).unwrap()["matched"], false);
}
#[test]
fn authoring_legacy_reset_and_undo_do_not_persist_runtime_control() {
    let mut s = scene(false);
    s.npc_generators[0].initially_active = false;
    s.version = 61;
    assert!(Engine::new(s.clone()).is_err());
    s.version = SCENE_VERSION;
    let mut e = Engine::new(s).unwrap();
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    control(&mut e, Action::Cancel);
    e.advance(1).unwrap();
    assert!(!e.scene.npc_generators[0].initially_active);
    let saved = e.scene.clone();
    let reloaded = Engine::new(saved).unwrap();
    assert_eq!(state(&reloaded).status, Status::Paused);
    assert_eq!(state(&reloaded).births, 0);
    run(&mut e, Command::Reset);
    assert_eq!(state(&e).status, Status::Paused);
    assert!(e.replay().population_inputs.is_empty());
    let mut generators = e.scene.npc_generators.clone();
    generators[0].initially_active = true;
    run(&mut e, Command::SetNpcGenerators { generators });
    assert_eq!(state(&e).status, Status::Active);
    run(&mut e, Command::Undo);
    assert_eq!(state(&e).status, Status::Paused);
    run(&mut e, Command::Redo);
    assert_eq!(state(&e).status, Status::Active);
    let mut legacy = serde_json::to_value(scene(false)).unwrap();
    legacy["version"] = json!(61);
    legacy["npc_generators"][0]
        .as_object_mut()
        .unwrap()
        .remove("initially_active");
    let mut old = Engine::new(serde_json::from_value(legacy).unwrap()).unwrap();
    old.advance(1).unwrap();
    assert_eq!(state(&old).births, 1);
    replay(&old);
}

#[test]
fn pausing_production_keeps_npc_locomotion_and_player_input_live() {
    let mut s = scene(false);
    let npc = s.npc_templates[0].npc.as_mut().unwrap();
    npc.enabled = true;
    npc.target = Some("player".into());
    npc.awareness = Some(ignis::model::Awareness::Omniscient);
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let id = name(&e);
    let npc_before = e.simulation.position(&id).unwrap();
    let player_before = e.simulation.position("player").unwrap();
    control(&mut e, Action::Pause);
    run(
        &mut e,
        Command::SetInput {
            input: ignis::physics::Input {
                x: 1.,
                z: 0.,
                ..Default::default()
            },
        },
    );
    e.advance(30).unwrap();
    let npc_after = e.simulation.position(&id).unwrap();
    let player_after = e.simulation.position("player").unwrap();
    assert!((npc_after[0] - npc_before[0]).abs() + (npc_after[2] - npc_before[2]).abs() > 0.2);
    assert!((player_after[0] - player_before[0]).abs() > 0.2);
    assert_eq!(state(&e).status, Status::Paused);
    assert_eq!(state(&e).births, 1);
    replay(&e);
}

#[test]
fn control_is_scoped_to_one_generator_and_shared_capacity_stays_intact() {
    let mut s = scene(false);
    let mut other = s.npc_generators[0].clone();
    other.id = "other".into();
    s.npc_generators.push(other);
    let mut e = Engine::new(s).unwrap();
    control(&mut e, Action::Pause);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(e.simulation.generators.generators[1].births, 1);
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    run(
        &mut e,
        Command::ControlNpcGenerator {
            generator: "other".into(),
            action: Action::Cancel,
        },
    );
    e.advance(1).unwrap();
    assert_eq!(state(&e).status, Status::Active);
    assert_eq!(
        e.simulation.generators.generators[1].status,
        Status::Cancelled
    );
    assert_eq!(e.simulation.generators.generators[1].births, 1);
    replay(&e);
}
