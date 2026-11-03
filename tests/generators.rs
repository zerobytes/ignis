use ignis::{
    engine::{Command, Engine},
    generators::{Config, Generator},
    model::{RoomRecipe, Scene},
    population::Outcome,
};
use serde_json::{Value, json};
fn config() -> Config {
    serde_json::from_value(json!({"id":"camp","template":"guard","points":[[0,0.8,0],[3,0.8,0],[-3,0.8,0]],"seed":42,"target_alive":2,"interval_ticks":5,"attempts_per_cycle":3,"max_births":20})).unwrap()
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 24.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[
      {"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},
      {"id":"revive","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":40}]}]})).unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [0., 0.8, -4.];
    p.footsteps = None;
    p.abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["kill","revive"]})).unwrap());
    s.npc_templates=vec![serde_json::from_value(json!({"id":"guard","name":"Guard","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":2,"stop_distance":1,"enabled":false},"vitality":{"initial":100,"maximum":100,"scale":1}})).unwrap()];
    s.npc_generators = vec![config()];
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn state(e: &Engine) -> Generator {
    e.simulation.generators.generators[0].clone()
}
fn first(e: &Engine) -> String {
    let m = state(e).members.iter().flatten().next().copied().unwrap();
    e.simulation
        .actors
        .label(e.simulation.definitions(), m.instance)
        .unwrap()
        .into()
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
fn seeded_births_are_derived_and_replay_prefixes_match() {
    let s = scene();
    let mut a = Engine::new(s.clone()).unwrap();
    let mut b = Engine::new(s).unwrap();
    assert_eq!(state(&a).births, 0);
    a.advance(1).unwrap();
    b.advance(1).unwrap();
    assert_eq!(a.simulation.generators, b.simulation.generators);
    assert_eq!(state(&a).births, 2);
    assert_eq!(a.simulation.population.processed, 0);
    assert!(a.replay().population_inputs.is_empty());
    assert_eq!(a.scene.npc_templates[0].id, "guard");
    let prefix = a.replay();
    a.advance(20).unwrap();
    assert_eq!(state(&a).attempts, 2);
    assert_eq!(state(&a).rng, state(&b).rng);
    let restored = Engine::replay_at(&a.replay(), Some(1)).unwrap();
    assert_eq!(restored.simulation.generators, prefix.expected_generators);
    replay(&a);
    let mut bad = a.replay();
    bad.expected_generators.generators[0].rng ^= 1;
    assert_eq!(Engine::verify_replay(&bad).unwrap()["matched"], false);
}
#[test]
fn delay_cadence_birth_budget_and_external_removal() {
    let mut s = scene();
    s.npc_generators[0].initial_delay_ticks = 3;
    s.npc_generators[0].max_births = 3;
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    assert_eq!(state(&e).births, 0);
    e.advance(1).unwrap();
    let old = first(&e);
    run(&mut e, Command::DespawnActor { actor: old });
    e.advance(4).unwrap();
    assert_eq!(state(&e).births, 2);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 3);
    for m in state(&e).members.iter().flatten() {
        let id = e
            .simulation
            .actors
            .label(e.simulation.definitions(), m.instance)
            .unwrap()
            .to_owned();
        run(&mut e, Command::DespawnActor { actor: id });
    }
    e.advance(20).unwrap();
    assert_eq!(state(&e).births, 3);
    assert!(state(&e).members.iter().all(Option::is_none));
    replay(&e);
}
#[test]
fn occupied_unsupported_and_capacity_fail_without_identity_or_birth_budget() {
    let mut s = scene();
    s.npc_generators[0].points = vec![[0., 0.8, -4.], [0., 9., 0.]];
    s.npc_generators[0].attempts_per_cycle = 8;
    let mut e = Engine::new(s).unwrap();
    let ids = e.simulation.actors.instance_snapshot();
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(state(&e).attempts, 2);
    assert_eq!(e.simulation.actors.instance_snapshot(), ids);
    let outcomes: Vec<_> = e
        .simulation
        .generators
        .receipts
        .iter()
        .map(|r| r.outcome)
        .collect();
    assert!(outcomes.contains(&Outcome::Occupied));
    assert!(outcomes.contains(&Outcome::UnsupportedPosition));
    e.advance(4).unwrap();
    assert_eq!(state(&e).attempts, 2);
    e.advance(1).unwrap();
    assert_eq!(state(&e).attempts, 4);
    replay(&e);
}
#[test]
fn corpses_preserved_and_revival_never_evicts_replacements() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let id = first(&e);
    let original = state(&e).members[0].unwrap();
    cast(&mut e, 1, &id);
    e.advance(5).unwrap();
    assert_eq!(state(&e).births, 3);
    assert!(
        e.simulation
            .actors
            .matches(original.actor, original.instance)
    );
    cast(&mut e, 2, &id);
    e.advance(6).unwrap();
    let counts = run(&mut e, Command::InspectGenerators);
    assert_eq!(counts["counts"][0]["alive"], 3);
    assert_eq!(counts["counts"][0]["dead"], 0);
    assert_eq!(state(&e).births, 3);
    assert_eq!(state(&e).members[0].unwrap().dead_since, None);
    replay(&e);
}
#[test]
fn configured_retirement_precedes_same_tick_revive_and_reuses_identity_safely() {
    let mut s = scene();
    s.npc_generators[0].target_alive = 1;
    s.npc_generators[0].interval_ticks = 1;
    s.npc_generators[0].points.truncate(1);
    s.npc_generators[0].retire_dead_after_ticks = Some(2);
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let id = first(&e);
    let old = state(&e).members[0].unwrap();
    cast(&mut e, 1, &id);
    e.advance(2).unwrap();
    assert!(e.simulation.actors.matches(old.actor, old.instance));
    cast(&mut e, 2, &id);
    assert!(!e.simulation.actors.matches(old.actor, old.instance));
    assert_eq!(state(&e).births, 2);
    let new = state(&e).members[0].unwrap();
    assert_ne!(new.instance, old.instance);
    assert_eq!(e.simulation.vitality.state.lives[new.actor as usize], 0);
    assert_eq!(
        e.simulation.vitality.state.actors[new.actor as usize]
            .unwrap()
            .units,
        100
    );
    replay(&e);
}
#[test]
fn resurrection_before_retirement_clears_timer() {
    let mut s = scene();
    s.npc_generators[0].target_alive = 1;
    s.npc_generators[0].points.truncate(1);
    s.npc_generators[0].retire_dead_after_ticks = Some(4);
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let id = first(&e);
    cast(&mut e, 1, &id);
    e.advance(1).unwrap();
    cast(&mut e, 2, &id);
    e.advance(6).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(first(&e), id);
    assert_eq!(state(&e).members[0].unwrap().dead_since, None);
    replay(&e);
}
#[test]
fn external_spawn_has_priority_and_is_never_owned_or_retired() {
    let mut s = scene();
    s.npc_generators[0].points = vec![[0., 0.8, 0.]];
    s.npc_generators[0].retire_dead_after_ticks = Some(0);
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
    assert_eq!(state(&e).births, 0);
    let m = e
        .simulation
        .population
        .receipts
        .back()
        .unwrap()
        .instance
        .unwrap();
    let id = e
        .simulation
        .actors
        .label(e.simulation.definitions(), m)
        .unwrap()
        .to_owned();
    cast(&mut e, 1, &id);
    e.advance(10).unwrap();
    assert!(
        e.simulation
            .actors
            .label(e.simulation.definitions(), m)
            .is_some()
    );
    assert_eq!(state(&e).births, 0);
    replay(&e);
}
#[test]
fn reused_external_occupant_is_not_retired_or_counted() {
    let mut s = scene();
    s.npc_generators[0].initial_delay_ticks = 2;
    s.npc_generators[0].target_alive = 1;
    s.npc_generators[0].retire_dead_after_ticks = Some(0);
    s.npc_generators[0].max_births = 1;
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    let old = first(&e);
    run(&mut e, Command::DespawnActor { actor: old });
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [0., 0.8, 4.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    assert!(state(&e).members.iter().all(Option::is_none));
    let m = e
        .simulation
        .population
        .receipts
        .back()
        .unwrap()
        .instance
        .unwrap();
    let id = e
        .simulation
        .actors
        .label(e.simulation.definitions(), m)
        .unwrap()
        .to_owned();
    cast(&mut e, 1, &id);
    e.advance(4).unwrap();
    assert!(
        e.simulation
            .actors
            .label(e.simulation.definitions(), m)
            .is_some()
    );
    replay(&e);
}
#[test]
fn ring_and_attempts_bounded_under_long_occupation() {
    let mut s = scene();
    s.npc_generators[0].points = vec![[0., 0.8, -4.]];
    s.npc_generators[0].interval_ticks = 1;
    let mut e = Engine::new(s).unwrap();
    e.advance(300).unwrap();
    assert_eq!(state(&e).attempts, 300);
    assert_eq!(state(&e).births, 0);
    assert_eq!(e.simulation.generators.receipts.len(), 256);
    assert_eq!(e.simulation.generators.dropped_receipts, 44);
    assert!(e.replay().population_inputs.is_empty());
    replay(&e);
}
#[test]
fn invalid_edit_is_atomic_and_valid_edit_undo_reset_rebuild() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let before = e.snapshot();
    let g = config();
    for mutate in 0..9 {
        let mut bad = g.clone();
        match mutate {
            0 => bad.template = "missing".into(),
            1 => bad.points.clear(),
            2 => bad.interval_ticks = 0,
            3 => bad.target_alive = 9,
            4 => bad.attempts_per_cycle = 9,
            5 => bad.max_births = 0,
            6 => bad.seed = 0,
            7 => bad.points[0][0] = f32::NAN,
            _ => bad.initial_delay_ticks = 36001,
        };
        if mutate == 6 {
            continue;
        }
        assert!(
            e.execute(
                Command::SetNpcGenerators {
                    generators: vec![bad]
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    assert!(
        e.execute(
            Command::SetNpcGenerators {
                generators: vec![g.clone(), g]
            }
            .into()
        )
        .is_err()
    );
    assert!(
        e.execute(Command::SetNpcTemplates { templates: vec![] }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    run(&mut e, Command::SetNpcGenerators { generators: vec![] });
    assert!(e.simulation.generators.generators.is_empty());
    run(&mut e, Command::Undo);
    assert_eq!(state(&e).births, 0);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    run(&mut e, Command::Reset);
    assert_eq!(state(&e).births, 0);
    replay(&e);
}
#[test]
fn competing_generators_respect_global_pool_and_author_order() {
    let mut s = scene();
    let mut g = config();
    g.points = (0..8).map(|i| [-7. + i as f32 * 2., 0.8, 2.]).collect();
    g.target_alive = 8;
    g.attempts_per_cycle = 8;
    s.npc_generators = vec![
        g.clone(),
        Config {
            id: "second".into(),
            ..g
        },
    ];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 8);
    assert_eq!(e.simulation.generators.generators[1].births, 0);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .filter(|r| r.generator == 1)
            .all(|r| r.outcome == Outcome::Capacity)
    );
    replay(&e);
}
#[test]
fn original_template_target_does_not_rebind_after_removal() {
    let mut s = scene();
    let mut target = s.npc_templates[0].clone();
    target.id = "target".into();
    target.transform.position = [4., 0.8, 4.];
    s.entities.push(target);
    s.npc_templates[0].npc.as_mut().unwrap().target = Some("target".into());
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        Command::DespawnActor {
            actor: "target".into(),
        },
    );
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 0);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::InstanceUnavailable)
    );
    replay(&e);
}
#[test]
fn different_seeds_change_candidate_and_zero_seed_is_valid() {
    let mut s = scene();
    s.npc_generators[0].target_alive = 1;
    s.npc_generators[0].seed = 0;
    let mut a = Engine::new(s.clone()).unwrap();
    s.npc_generators[0].seed = 1000;
    let mut b = Engine::new(s).unwrap();
    a.advance(1).unwrap();
    b.advance(1).unwrap();
    assert_ne!(
        a.simulation.generators.receipts[0].point,
        b.simulation.generators.receipts[0].point
    );
    replay(&a);
    replay(&b);
}
