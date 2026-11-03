use ignis::{
    engine::{Command, Engine},
    generators::Generator,
    model::{SCENE_VERSION, Scene},
    population::Outcome,
    waves::{Completion, Config, EventKind, Phase},
};
use serde_json::{Value, json};
fn wave(count: u32) -> Config {
    Config {
        count,
        template: None,
        templates: Vec::new(),
        delay_ticks: 0,
        completion: Completion::DefeatedOrRemoved,
    }
}
fn scene() -> Scene {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-generators.scene.json")).unwrap();
    s.version = SCENE_VERSION;
    let mut elite = s.npc_templates[0].clone();
    elite.id = "elite".into();
    elite.color = [0.8, 0.2, 0.1];
    elite.vitality.as_mut().unwrap().initial = 40;
    s.npc_templates.push(elite);
    let g = &mut s.npc_generators[0];
    g.interval_ticks = 1;
    g.attempts_per_cycle = 3;
    g.max_births = 20;
    g.retire_dead_after_ticks = Some(0);
    g.waves = vec![
        wave(2),
        Config {
            template: Some("elite".into()),
            delay_ticks: 2,
            ..wave(1)
        },
    ];
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn state(e: &Engine) -> Generator {
    e.simulation.generators.generators[0].clone()
}
fn phase(e: &Engine) -> Phase {
    state(e).wave.unwrap().phase
}
fn names(e: &Engine) -> Vec<String> {
    state(e)
        .members
        .iter()
        .flatten()
        .filter(|m| e.simulation.actors.matches(m.actor, m.instance))
        .map(|m| {
            e.simulation
                .actors
                .label(e.simulation.definitions(), m.instance)
                .unwrap()
                .to_owned()
        })
        .collect()
}
fn remove_all(e: &mut Engine) {
    for id in names(e) {
        run(e, Command::DespawnActor { actor: id });
    }
    e.advance(1).unwrap();
}
fn cast(e: &mut Engine, slot: u8, id: &str) {
    run(
        e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot,
            target: Some(id.into()),
        },
    );
    e.advance(1).unwrap();
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn finite_sequence_changes_template_and_honors_exact_delays() {
    let mut s = scene();
    s.npc_generators[0].initial_delay_ticks = 2;
    s.npc_generators[0].waves[0].delay_ticks = 3;
    let mut e = Engine::new(s).unwrap();
    e.advance(5).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(phase(&e), Phase::Delay);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    assert_eq!(phase(&e), Phase::WaitingForClear);
    let at_first = e.simulation.generators.clone();
    remove_all(&mut e);
    assert_eq!(state(&e).wave.unwrap().current, 1);
    assert_eq!(state(&e).births, 2);
    assert_eq!(phase(&e), Phase::Delay);
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::Delay);
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::WaitingForClear);
    assert_eq!(state(&e).births, 3);
    let m = state(&e).members.iter().flatten().next().copied().unwrap();
    assert_eq!(
        e.simulation.vitality.state.actors[m.actor as usize]
            .unwrap()
            .units,
        40
    );
    remove_all(&mut e);
    assert_eq!(phase(&e), Phase::Completed);
    e.advance(10).unwrap();
    assert_eq!(state(&e).births, 3);
    let events = &e.simulation.generators.wave_events;
    assert_eq!(
        events.iter().map(|e| (e.tick, e.kind)).collect::<Vec<_>>(),
        vec![
            (5, EventKind::Started),
            (6, EventKind::Cleared),
            (8, EventKind::Started),
            (9, EventKind::Completed)
        ]
    );
    assert_eq!(
        Engine::replay_at(&e.replay(), Some(6))
            .unwrap()
            .simulation
            .generators,
        at_first
    );
    replay(&e);
}
#[test]
fn quota_can_exceed_concurrency_and_never_replenishes_a_finished_wave() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.target_alive = 1;
    g.waves = vec![wave(3)];
    g.interval_ticks = 4;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(phase(&e), Phase::Spawning);
    let id = names(&e)[0].clone();
    cast(&mut e, 1, &id);
    e.advance(2).unwrap();
    assert_eq!(state(&e).births, 1);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    let id = names(&e)[0].clone();
    cast(&mut e, 1, &id);
    e.advance(3).unwrap();
    assert_eq!(state(&e).births, 3);
    assert_eq!(phase(&e), Phase::WaitingForClear);
    let id = names(&e)[0].clone();
    cast(&mut e, 1, &id);
    e.advance(8).unwrap();
    assert_eq!(phase(&e), Phase::Completed);
    assert_eq!(state(&e).births, 3);
    replay(&e);
}
#[test]
fn failed_candidates_leave_quota_and_identity_unchanged_until_space_opens() {
    let mut s = scene();
    s.npc_generators[0].points = vec![[0., 0.8, 0.]];
    s.npc_generators[0].waves = vec![wave(1)];
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [0., 0.8, 0.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    let r = e
        .simulation
        .population
        .receipts
        .back()
        .unwrap()
        .instance
        .unwrap();
    let ids = e.simulation.actors.instance_snapshot();
    e.advance(20).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(state(&e).wave.unwrap().births, 0);
    assert_eq!(e.simulation.actors.instance_snapshot(), ids);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::Occupied)
    );
    let id = e
        .simulation
        .actors
        .label(e.simulation.definitions(), r)
        .unwrap()
        .to_owned();
    run(&mut e, Command::DespawnActor { actor: id });
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(phase(&e), Phase::WaitingForClear);
    replay(&e);
}
#[test]
fn removed_completion_preserves_corpse_until_explicit_retirement() {
    for completion in [Completion::Removed, Completion::DefeatedOrRemoved] {
        let mut s = scene();
        let g = &mut s.npc_generators[0];
        g.retire_dead_after_ticks = None;
        g.waves = vec![Config {
            completion,
            ..wave(1)
        }];
        let mut e = Engine::new(s).unwrap();
        e.advance(1).unwrap();
        let id = names(&e)[0].clone();
        cast(&mut e, 1, &id);
        e.advance(2).unwrap();
        assert_eq!(names(&e), vec![id.clone()]);
        assert_eq!(
            phase(&e),
            if completion == Completion::Removed {
                Phase::WaitingForClear
            } else {
                Phase::Completed
            }
        );
        run(&mut e, Command::DespawnActor { actor: id });
        e.advance(1).unwrap();
        assert_eq!(phase(&e), Phase::Completed);
        replay(&e);
    }
}
#[test]
fn old_wave_revival_does_not_rewind_and_counts_toward_living_limit() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.target_alive = 1;
    g.retire_dead_after_ticks = None;
    g.waves[0].count = 1;
    g.waves[1].delay_ticks = 3;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let old = names(&e)[0].clone();
    cast(&mut e, 1, &old);
    e.advance(1).unwrap();
    assert_eq!(state(&e).wave.unwrap().current, 1);
    cast(&mut e, 2, &old);
    e.advance(3).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(phase(&e), Phase::Spawning);
    assert_eq!(
        run(&mut e, Command::InspectGenerators)["waves"][0]["waiting_for"],
        "living_limit"
    );
    run(&mut e, Command::DespawnActor { actor: old });
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    assert_eq!(
        state(&e).members.iter().flatten().next().unwrap().wave,
        Some(1)
    );
    replay(&e);
}
#[test]
fn revival_while_another_member_is_alive_prevents_clear() {
    let mut s = scene();
    s.npc_generators[0].retire_dead_after_ticks = None;
    s.npc_generators[0].waves = vec![wave(2)];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let ids = names(&e);
    cast(&mut e, 1, &ids[0]);
    cast(&mut e, 2, &ids[0]);
    cast(&mut e, 1, &ids[1]);
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::WaitingForClear);
    cast(&mut e, 1, &ids[0]);
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::Completed);
    let completed = state(&e).wave.unwrap().completed_tick;
    cast(&mut e, 2, &ids[1]);
    e.advance(2).unwrap();
    assert_eq!(phase(&e), Phase::Completed);
    assert_eq!(state(&e).wave.unwrap().completed_tick, completed);
    replay(&e);
}
#[test]
fn unrelated_replacement_is_not_a_wave_member() {
    let mut s = scene();
    s.npc_generators[0].waves = vec![wave(1)];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let id = names(&e)[0].clone();
    let old = state(&e).members.iter().flatten().next().unwrap().instance;
    run(&mut e, Command::DespawnActor { actor: id });
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [5., 0.8, 0.],
            target: None,
        },
    );
    replay(&e);
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::Completed);
    assert!(names(&e).is_empty());
    assert_ne!(
        e.simulation
            .population
            .receipts
            .back()
            .unwrap()
            .instance
            .unwrap(),
        old
    );
    replay(&e);
}
#[test]
fn future_wave_target_is_bound_to_original_individual() {
    let mut s = scene();
    let mut target = s.npc_templates[0].clone();
    target.id = "target".into();
    target.transform.position = [5., 0.8, 5.];
    s.entities.push(target);
    s.npc_templates[1].npc.as_mut().unwrap().target = Some("target".into());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    run(
        &mut e,
        Command::DespawnActor {
            actor: "target".into(),
        },
    );
    remove_all(&mut e);
    e.advance(5).unwrap();
    assert_eq!(state(&e).wave.unwrap().current, 1);
    assert_eq!(state(&e).wave.unwrap().births, 0);
    assert_eq!(phase(&e), Phase::Spawning);
    assert_eq!(
        e.simulation.generators.receipts.back().unwrap().outcome,
        Outcome::InstanceUnavailable
    );
    replay(&e);
}
#[test]
fn completed_sequence_still_retires_owned_corpses() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.retire_dead_after_ticks = Some(3);
    g.waves = vec![wave(1)];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let id = names(&e)[0].clone();
    cast(&mut e, 1, &id);
    e.advance(1).unwrap();
    assert_eq!(phase(&e), Phase::Completed);
    assert_eq!(names(&e).len(), 1);
    e.advance(3).unwrap();
    assert!(names(&e).is_empty());
    assert_eq!(state(&e).births, 1);
    assert_eq!(
        e.simulation.generators.receipts.back().unwrap().outcome,
        Outcome::Despawned
    );
    replay(&e);
}
#[test]
fn invalid_edits_preserve_live_progress_and_history() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let before = e.snapshot();
    let original = e.scene.npc_generators.clone();
    for n in 0..6 {
        let mut bad = original.clone();
        match n {
            0 => bad[0].waves[0].count = 0,
            1 => bad[0].waves[0].template = Some("missing".into()),
            2 => bad[0].max_births = 1,
            3 => bad[0].waves = vec![wave(1); 17],
            4 => bad[0].waves[0].delay_ticks = 36001,
            _ => bad[0].waves[0].count = 36001,
        };
        assert!(
            e.execute(Command::SetNpcGenerators { generators: bad }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let templates = vec![e.scene.npc_templates[0].clone()];
    assert!(
        e.execute(Command::SetNpcTemplates { templates }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut old = scene();
    old.version = 60;
    assert!(Engine::new(old).is_err());
    run(&mut e, Command::SetNpcGenerators { generators: vec![] });
    run(&mut e, Command::Undo);
    assert_eq!(phase(&e), Phase::Delay);
    assert_eq!(state(&e).births, 0);
    run(&mut e, Command::Redo);
    assert!(e.simulation.generators.generators.is_empty());
    run(&mut e, Command::Undo);
    e.advance(1).unwrap();
    run(&mut e, Command::Reset);
    assert_eq!(phase(&e), Phase::Delay);
    replay(&e);
}
#[test]
fn maximal_sequences_have_bounded_events_and_no_same_tick_cascade() {
    let mut s = scene();
    let base = s.npc_generators[0].clone();
    s.npc_generators = (0..8)
        .map(|i| {
            let mut g = base.clone();
            g.id = format!("g{i}");
            g.points = vec![[-7. + i as f32 * 2., 0.8, 2.]];
            g.target_alive = 1;
            g.waves = vec![wave(1); 16];
            g
        })
        .collect();
    let mut e = Engine::new(s).unwrap();
    for wave in 0..16 {
        e.advance(1).unwrap();
        for g in &e.simulation.generators.generators {
            assert_eq!(g.wave.unwrap().current, wave);
            assert_eq!(g.wave.unwrap().phase, Phase::WaitingForClear);
        }
        let ids: Vec<_> = e
            .simulation
            .generators
            .generators
            .iter()
            .flat_map(|g| g.members.iter().flatten())
            .map(|m| {
                e.simulation
                    .actors
                    .label(e.simulation.definitions(), m.instance)
                    .unwrap()
                    .to_owned()
            })
            .collect();
        for id in ids {
            run(&mut e, Command::DespawnActor { actor: id });
        }
        e.advance(1).unwrap();
    }
    assert!(
        e.simulation
            .generators
            .generators
            .iter()
            .all(|g| g.wave.unwrap().phase == Phase::Completed)
    );
    assert_eq!(e.simulation.generators.wave_events.len(), 256);
    assert_eq!(e.simulation.generators.dropped_wave_events, 0);
    assert_eq!(
        e.simulation
            .generators
            .wave_events
            .iter()
            .filter(|ev| ev.kind == EventKind::Completed)
            .count(),
        8
    );
    replay(&e);
}
#[test]
fn retained_corpses_can_block_next_wave_without_stealing_slots() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.points = (0..8).map(|i| [-7. + i as f32 * 2., 0.8, 2.]).collect();
    g.target_alive = 8;
    g.attempts_per_cycle = 8;
    g.retire_dead_after_ticks = None;
    g.waves = vec![wave(8), wave(1)];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    for id in names(&e) {
        cast(&mut e, 1, &id);
    }
    e.advance(2).unwrap();
    assert_eq!(state(&e).wave.unwrap().current, 1);
    assert_eq!(state(&e).births, 8);
    assert_eq!(
        run(&mut e, Command::InspectGenerators)["waves"][0]["waiting_for"],
        "member_capacity"
    );
    let old = names(&e)[0].clone();
    run(&mut e, Command::DespawnActor { actor: old });
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 9);
    replay(&e);
}
#[test]
fn replay_checks_wave_state_and_reproduces_same_seed() {
    let mut a = Engine::new(scene()).unwrap();
    let mut b = Engine::new(scene()).unwrap();
    a.advance(1).unwrap();
    b.advance(1).unwrap();
    assert_eq!(a.simulation.generators, b.simulation.generators);
    let mut r = a.replay();
    r.expected_generators.generators[0]
        .wave
        .as_mut()
        .unwrap()
        .births += 1;
    assert_eq!(Engine::verify_replay(&r).unwrap()["matched"], false);
    replay(&a);
    let view = run(&mut a, Command::InspectGenerators);
    assert_eq!(view["waves"][0]["ordinal"], json!(1));
    assert_eq!(view["waves"][0]["total"], json!(2));
}
