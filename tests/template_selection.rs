use ignis::{
    engine::{Command, Engine},
    generator_control::Action,
    generators::Generator,
    model::{SCENE_VERSION, Scene},
    population::Outcome,
    template_selection::{Choice, SEED_SALT},
    waves::Phase,
};
use serde_json::{Value, json};
fn choices() -> Vec<Choice> {
    vec![
        Choice {
            template: "guard".into(),
            weight: 3,
        },
        Choice {
            template: "elite".into(),
            weight: 1,
        },
    ]
}
fn scene() -> Scene {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-spawn-regions.scene.json")).unwrap();
    s.version = SCENE_VERSION;
    let g = &mut s.npc_generators[0];
    g.template.clear();
    g.templates = choices();
    g.waves.clear();
    g.activation = None;
    g.target_alive = 4;
    g.interval_ticks = 1;
    g.attempts_per_cycle = 8;
    g.max_births = 12;
    g.retire_dead_after_ticks = Some(0);
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn state(e: &Engine) -> Generator {
    e.simulation.generators.generators[0].clone()
}
fn names(e: &Engine) -> Vec<String> {
    state(e)
        .members
        .into_iter()
        .flatten()
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
    for actor in names(e) {
        run(e, Command::DespawnActor { actor });
    }
    e.advance(1).unwrap();
}
fn control(e: &mut Engine, a: Action) {
    run(
        e,
        Command::ControlNpcGenerator {
            generator: "encounter".into(),
            action: a,
        },
    );
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn known_seed_selects_mixed_templates_and_independent_member_state() {
    let mut a = Engine::new(scene()).unwrap();
    let mut b = Engine::new(scene()).unwrap();
    a.advance(1).unwrap();
    b.advance(1).unwrap();
    assert_eq!(a.simulation.generators, b.simulation.generators);
    assert_eq!(state(&a).births, 4);
    assert_eq!(state(&a).template_draws, 4);
    assert!(!state(&a).pending_template);
    let templates: Vec<_> = a
        .simulation
        .generators
        .receipts
        .iter()
        .filter(|r| r.outcome == Outcome::Spawned)
        .map(|r| r.template.unwrap())
        .collect();
    assert_eq!(templates, vec![0, 0, 0, 1]);
    for m in state(&a).members.into_iter().flatten() {
        let health = a.simulation.vitality.state.actors[m.actor as usize].unwrap();
        assert_eq!(health.units, if m.template == 0 { 100 } else { 40 });
        assert_eq!(a.simulation.vitality.state.lives[m.actor as usize], 0);
    }
    let first = a.simulation.generators.clone();
    remove_all(&mut a);
    assert_eq!(state(&a).births, 8);
    assert_eq!(state(&a).template_draws, 8);
    assert_eq!(
        Engine::replay_at(&a.replay(), Some(1))
            .unwrap()
            .simulation
            .generators,
        first
    );
    replay(&a);
    let mut s = scene();
    s.npc_generators[0].seed = 43;
    let mut c = Engine::new(s).unwrap();
    c.advance(1).unwrap();
    assert_ne!(state(&c).template_rng, state(&b).template_rng);
}
#[test]
fn one_weighted_candidate_preserves_position_stream_of_single_template() {
    let mut a = scene();
    a.npc_generators[0].template = "guard".into();
    a.npc_generators[0].templates.clear();
    let mut b = a.clone();
    b.npc_generators[0].template.clear();
    b.npc_generators[0].templates = vec![Choice {
        template: "guard".into(),
        weight: 17,
    }];
    let mut a = Engine::new(a).unwrap();
    let mut b = Engine::new(b).unwrap();
    a.advance(1).unwrap();
    b.advance(1).unwrap();
    assert_eq!(state(&a).rng, state(&b).rng);
    assert_eq!(
        a.simulation.generators.receipts,
        b.simulation.generators.receipts
    );
    assert_eq!(state(&a).template_draws, 0);
    assert_eq!(state(&a).template_rng, 42 ^ SEED_SALT);
    assert_eq!(state(&b).template_draws, 4);
    replay(&a);
    replay(&b);
}
#[test]
fn failed_geometry_does_not_reroll_template_and_pause_freezes_both_streams() {
    let mut s = scene();
    s.npc_generators[0].templates = vec![Choice {
        template: "guard".into(),
        weight: 1,
    }];
    s.npc_generators[0].region.as_mut().unwrap().min[1] = 9.;
    s.npc_generators[0].region.as_mut().unwrap().max[1] = 9.;
    let mut e = Engine::new(s).unwrap();
    let ids = e.simulation.actors.instance_snapshot();
    e.advance(3).unwrap();
    assert_eq!(state(&e).attempts, 24);
    assert_eq!(state(&e).births, 0);
    assert_eq!(state(&e).template_draws, 1);
    assert!(state(&e).pending_template);
    assert_eq!(e.simulation.actors.instance_snapshot(), ids);
    let t = state(&e).template_rng;
    let p = state(&e).rng;
    control(&mut e, Action::Pause);
    e.advance(10).unwrap();
    assert_eq!(state(&e).template_rng, t);
    assert_eq!(state(&e).rng, p);
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    assert_eq!(state(&e).template_draws, 1);
    assert_ne!(state(&e).rng, p);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.template == Some(0) && r.outcome == Outcome::UnsupportedPosition)
    );
    replay(&e);
}
#[test]
fn removing_external_blocker_reuses_pending_choice_then_selects_next_birth() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.region = Some(ignis::spawn_region::Region {
        min: [-0.02, 0.8, -1.],
        max: [0.02, 0.8, -1.],
    });
    g.target_alive = 1;
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [0., 0.8, -1.],
            target: None,
        },
    );
    e.advance(3).unwrap();
    assert_eq!(state(&e).births, 0);
    let rng = state(&e).template_rng;
    assert_eq!(state(&e).template_draws, 1);
    let id = run(&mut e, Command::InspectPopulation)["receipts"][0]["authored_id"]
        .as_str()
        .unwrap()
        .to_owned();
    run(&mut e, Command::DespawnActor { actor: id });
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(state(&e).template_rng, rng);
    assert!(!state(&e).pending_template);
    remove_all(&mut e);
    assert_eq!(state(&e).births, 2);
    assert_eq!(state(&e).template_draws, 2);
    replay(&e);
}
#[test]
fn pending_template_never_falls_back_to_another_that_would_fit() {
    let mut s = scene();
    s.npc_generators[0].templates = vec![
        Choice {
            template: "boss".into(),
            weight: 1,
        },
        Choice {
            template: "guard".into(),
            weight: 3,
        },
    ];
    s.npc_generators[0].target_alive = 1;
    s.npc_templates[2].transform.size[1] = 2.;
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    assert_eq!(state(&e).template, 2);
    assert_eq!(state(&e).template_draws, 1);
    assert_eq!(state(&e).births, 0);
    assert!(state(&e).pending_template);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.template == Some(2))
    );
    replay(&e);
}
#[test]
fn waves_inherit_or_override_sources_without_extra_draws() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.waves = vec![
        serde_json::from_value(json!({"count":1})).unwrap(),
        serde_json::from_value(json!({"count":1,"templates":[{"template":"elite","weight":1}]}))
            .unwrap(),
        serde_json::from_value(json!({"count":1,"template":"boss"})).unwrap(),
    ];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(state(&e).members[0].unwrap().template, 0);
    assert_eq!(state(&e).template_draws, 1);
    remove_all(&mut e);
    e.advance(1).unwrap();
    assert_eq!(state(&e).members[0].unwrap().template, 1);
    assert_eq!(state(&e).template_draws, 2);
    remove_all(&mut e);
    e.advance(1).unwrap();
    assert_eq!(state(&e).members[0].unwrap().template, 2);
    assert_eq!(state(&e).template_draws, 2);
    remove_all(&mut e);
    assert_eq!(state(&e).wave.unwrap().phase, Phase::Completed);
    assert_eq!(state(&e).births, 3);
    replay(&e);
}
#[test]
fn future_weighted_candidate_target_is_bound_to_original_individual() {
    let mut s = scene();
    let mut witness = s.npc_templates[0].clone();
    witness.id = "witness".into();
    witness.transform.position = [-6., 0.8, -3.];
    s.entities.push(witness);
    s.npc_templates[1].npc.as_mut().unwrap().target = Some("witness".into());
    s.npc_templates[1].npc.as_mut().unwrap().enabled = true;
    s.npc_generators[0].waves = vec![
        serde_json::from_value(json!({"count":1,"template":"guard"})).unwrap(),
        serde_json::from_value(json!({"count":1,"templates":[{"template":"elite","weight":1}]}))
            .unwrap(),
    ];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    run(
        &mut e,
        Command::DespawnActor {
            actor: "witness".into(),
        },
    );
    remove_all(&mut e);
    e.advance(2).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(state(&e).template, 1);
    assert!(state(&e).pending_template);
    assert_eq!(
        e.simulation.generators.receipts.back().unwrap().outcome,
        Outcome::InstanceUnavailable
    );
    replay(&e);
}
#[test]
fn weights_and_all_dependencies_validate_atomically_and_old_scene_rejects_choices() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let before = e.snapshot();
    let base = e.scene.npc_generators.clone();
    for list in [
        vec![Choice {
            template: "guard".into(),
            weight: 0,
        }],
        vec![Choice {
            template: "guard".into(),
            weight: 1_000_001,
        }],
        vec![Choice {
            template: "missing".into(),
            weight: 1,
        }],
        vec![
            Choice {
                template: "guard".into(),
                weight: 1
            };
            2
        ],
        vec![
            Choice {
                template: "guard".into(),
                weight: 1
            };
            17
        ],
    ] {
        let mut bad = base.clone();
        bad[0].templates = list;
        assert!(
            e.execute(Command::SetNpcGenerators { generators: bad }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut bad = base.clone();
    bad[0].template = "guard".into();
    assert!(
        e.execute(Command::SetNpcGenerators { generators: bad }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut templates = e.scene.npc_templates.clone();
    templates.remove(1);
    assert!(
        e.execute(Command::SetNpcTemplates { templates }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut old = scene();
    old.version = 64;
    assert!(Engine::new(old).is_err());
    let mut g = base[0].clone();
    g.waves = vec![
        serde_json::from_value(
            json!({"count":1,"template":"guard","templates":[{"template":"elite","weight":1}]}),
        )
        .unwrap(),
    ];
    assert!(
        e.execute(
            Command::SetNpcGenerators {
                generators: vec![g]
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
}
#[test]
fn reset_undo_and_replay_cover_pending_and_compiled_sources() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let first = e.simulation.generators.clone();
    run(&mut e, Command::SetNpcGenerators { generators: vec![] });
    run(&mut e, Command::Undo);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators, first);
    run(&mut e, Command::Reset);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators, first);
    replay(&e);
    let mut r = e.replay();
    r.expected_generators.generators[0].template_rng ^= 1;
    assert_eq!(Engine::verify_replay(&r).unwrap()["matched"], false);
}
#[test]
fn cancelled_or_inactive_generation_does_not_start_choices() {
    let mut s = scene();
    s.npc_generators[0].initially_active = false;
    let mut e = Engine::new(s).unwrap();
    e.advance(10).unwrap();
    assert_eq!(state(&e).template_draws, 0);
    control(&mut e, Action::Cancel);
    control(&mut e, Action::Activate);
    e.advance(2).unwrap();
    assert_eq!(state(&e).template_draws, 0);
    assert!(!state(&e).pending_template);
    replay(&e);
}
#[test]
fn full_pool_keeps_one_pending_choice_and_limited_receipts() {
    let mut e = Engine::new(scene()).unwrap();
    for i in 0..8 {
        run(
            &mut e,
            Command::SpawnActor {
                template: "guard".into(),
                position: [-7. + i as f32 * 2., 0.8, 3.],
                target: None,
            },
        );
    }
    e.advance(40).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(state(&e).template_draws, 1);
    assert!(state(&e).pending_template);
    assert_eq!(state(&e).attempts, 320);
    assert_eq!(e.simulation.generators.receipts.len(), 256);
    assert_eq!(e.simulation.generators.dropped_receipts, 64);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.template == Some(0) && r.outcome == Outcome::Capacity)
    );
    replay(&e);
}
#[test]
fn retirement_receipt_keeps_original_template_after_wave_changes() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.retire_dead_after_ticks = Some(4);
    g.waves = vec![
        serde_json::from_value(json!({"count":1,"templates":[{"template":"elite","weight":1}]}))
            .unwrap(),
        serde_json::from_value(json!({"count":1,"template":"guard"})).unwrap(),
    ];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let target = names(&e)[0].clone();
    run(
        &mut e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some(target),
        },
    );
    e.advance(1).unwrap();
    e.advance(6).unwrap();
    let retired = e
        .simulation
        .generators
        .receipts
        .iter()
        .find(|r| r.outcome == Outcome::Despawned)
        .unwrap();
    assert_eq!(retired.template, Some(1));
    assert_eq!(state(&e).template, 0);
    replay(&e);
}
