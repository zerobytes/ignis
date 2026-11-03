use ignis::{
    engine::{Command, Engine},
    generator_control::Action,
    generators::Generator,
    model::{SCENE_VERSION, Scene},
    population::Outcome,
    spawn_region::Region,
    waves::Phase,
};
use serde_json::{Value, json};
fn region() -> Region {
    Region {
        min: [-5., 0.8, -1.5],
        max: [5., 0.8, 2.5],
    }
}
fn scene() -> Scene {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-waves.scene.json")).unwrap();
    s.version = SCENE_VERSION;
    let g = &mut s.npc_generators[0];
    g.waves.clear();
    g.points.clear();
    g.region = Some(region());
    g.target_alive = 3;
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
fn control(e: &mut Engine, a: Action) {
    run(
        e,
        Command::ControlNpcGenerator {
            generator: "encounter".into(),
            action: a,
        },
    );
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
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn remove_all(e: &mut Engine) {
    for actor in names(e) {
        run(e, Command::DespawnActor { actor });
    }
    e.advance(1).unwrap();
}
#[test]
fn golden_sample_uses_three_draws_including_fixed_axis() {
    let mut rng = 42;
    let p = region().sample(&mut rng);
    assert_eq!(rng, 2479403867);
    assert!((p[0] - (-2.4765487)).abs() < 0.000002);
    assert_eq!(p[1], 0.8);
    assert!((p[2] - 0.8091247).abs() < 0.000002);
    let mut zero = 0;
    assert_ne!(region().sample(&mut zero), p);
    assert_ne!(zero, 0);
}
#[test]
fn candidate_sequence_same_seed_replay_and_bounds_are_exact() {
    let s = scene();
    let mut a = Engine::new(s.clone()).unwrap();
    let mut b = Engine::new(s).unwrap();
    a.advance(1).unwrap();
    b.advance(1).unwrap();
    assert_eq!(a.simulation.generators, b.simulation.generators);
    assert_eq!(state(&a).births, 3);
    let r = region();
    for receipt in &a.simulation.generators.receipts {
        let p = receipt.position.unwrap();
        assert!(receipt.point.is_none());
        assert!((0..3).all(|i| p[i] >= r.min[i] && p[i] <= r.max[i]));
    }
    assert!(a.replay().population_inputs.is_empty());
    let initial = a.simulation.generators.clone();
    remove_all(&mut a);
    assert_eq!(state(&a).births, 6);
    assert_eq!(
        Engine::replay_at(&a.replay(), Some(1))
            .unwrap()
            .simulation
            .generators,
        initial
    );
    replay(&a);
    let mut other = scene();
    other.npc_generators[0].seed = 43;
    let mut c = Engine::new(other).unwrap();
    c.advance(1).unwrap();
    assert_ne!(
        c.simulation.generators.receipts[0].position,
        b.simulation.generators.receipts[0].position
    );
}
#[test]
fn unsupported_attempts_are_bounded_and_consume_no_identity_or_birth_budget() {
    let mut s = scene();
    s.npc_generators[0].region = Some(Region {
        min: [-2., 9., 0.],
        max: [2., 9., 2.],
    });
    let mut e = Engine::new(s).unwrap();
    let ids = e.simulation.actors.instance_snapshot();
    e.advance(40).unwrap();
    assert_eq!(state(&e).attempts, 320);
    assert_eq!(state(&e).births, 0);
    assert_eq!(e.simulation.actors.instance_snapshot(), ids);
    assert_eq!(e.simulation.generators.receipts.len(), 256);
    assert_eq!(e.simulation.generators.dropped_receipts, 64);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::UnsupportedPosition)
    );
    replay(&e);
}
#[test]
fn blocked_region_retains_position_evidence_until_external_despawn_frees_it() {
    let mut s = scene();
    s.npc_generators[0].region = Some(Region {
        min: [-0.02, 0.8, 0.],
        max: [0.02, 0.8, 0.],
    });
    s.npc_generators[0].target_alive = 1;
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
    let id = run(&mut e, Command::InspectPopulation)["receipts"][0]["authored_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let identity = e.simulation.actors.instance_snapshot();
    e.advance(3).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(e.simulation.actors.instance_snapshot(), identity);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::Occupied && r.position.is_some())
    );
    run(&mut e, Command::DespawnActor { actor: id });
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    replay(&e);
}
#[test]
fn legacy_point_draw_and_index_sequence_remain_unchanged() {
    let mut s = scene();
    let g = &mut s.npc_generators[0];
    g.region = None;
    g.points = vec![[0., 0.8, 0.], [3., 0.8, 0.], [-3., 0.8, 0.]];
    g.target_alive = 2;
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(state(&e).rng, 1083814273);
    assert_eq!(state(&e).attempts, 2);
    let receipts = &e.simulation.generators.receipts;
    assert_eq!(receipts[0].point, Some(0));
    assert_eq!(receipts[1].point, Some(1));
    assert_eq!(receipts[0].position, Some([0., 0.8, 0.]));
    e.advance(20).unwrap();
    assert_eq!(state(&e).rng, 1083814273);
    replay(&e);
}
#[test]
fn pause_and_cadence_do_not_consume_region_draws_or_reset_sequence() {
    let mut s = scene();
    s.npc_generators[0].initial_delay_ticks = 3;
    s.npc_generators[0].interval_ticks = 10;
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    assert_eq!(state(&e).rng, 42);
    control(&mut e, Action::Pause);
    e.advance(20).unwrap();
    assert_eq!(state(&e).rng, 42);
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    let rng = state(&e).rng;
    assert_eq!(state(&e).births, 3);
    for actor in names(&e) {
        run(&mut e, Command::DespawnActor { actor });
    }
    e.advance(8).unwrap();
    assert_eq!(state(&e).rng, rng);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 3);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 6);
    replay(&e);
}
#[test]
fn waves_use_current_template_geometry_and_stop_exactly_at_quota() {
    let mut s = scene();
    s.npc_templates[1].transform.size[1] = 2.;
    let g = &mut s.npc_generators[0];
    g.waves = vec![
        serde_json::from_value(json!({"count":1})).unwrap(),
        serde_json::from_value(json!({"count":1,"template":"elite"})).unwrap(),
    ];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    remove_all(&mut e);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(state(&e).wave.unwrap().current, 1);
    assert_eq!(state(&e).wave.unwrap().phase, Phase::Spawning);
    assert_eq!(
        e.simulation.generators.receipts.back().unwrap().outcome,
        Outcome::Occupied
    ); // taller body penetrates floor before support admission
    replay(&e);
    let mut s = scene();
    s.npc_generators[0].waves = vec![serde_json::from_value(json!({"count":2})).unwrap()];
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    remove_all(&mut e);
    assert_eq!(state(&e).wave.unwrap().phase, Phase::Completed);
    e.advance(20).unwrap();
    assert_eq!(state(&e).births, 2);
    replay(&e);
}
#[test]
fn airborne_volume_requires_hover_capability_and_samples_xyz() {
    let mut s = scene();
    s.npc_generators[0].region = Some(Region {
        min: [-4., 3., 0.],
        max: [4., 5., 3.],
    });
    s.npc_generators[0].target_alive = 2;
    s.npc_templates[0].npc.as_mut().unwrap().mobility =
        Some(serde_json::from_value(json!({"walk":false,"fly_speed":3.0})).unwrap());
    s.npc_templates[0].npc.as_mut().unwrap().local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":8})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    let p: Vec<_> = e
        .simulation
        .generators
        .receipts
        .iter()
        .filter_map(|r| r.position)
        .collect();
    assert!(p.iter().all(|p| p[1] >= 3. && p[1] <= 5.));
    assert_ne!(p[0][1], p[1][1]);
    replay(&e);
}
#[test]
fn source_validation_is_atomic_and_old_scenes_do_not_enable_regions() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let before = e.snapshot();
    let base = e.scene.npc_generators.clone();
    for invalid in [
        json!({"min":[0,0,0],"max":[0,0,0]}),
        json!({"min":[2,0,0],"max":[1,1,1]}),
        json!({"min":[0,0,0],"max":[10001,1,1]}),
    ] {
        let mut bad = base.clone();
        bad[0].region = Some(serde_json::from_value(invalid).unwrap());
        assert!(
            e.execute(Command::SetNpcGenerators { generators: bad }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut bad = base.clone();
    bad[0].points = vec![[0., 0.8, 0.]];
    assert!(
        e.execute(Command::SetNpcGenerators { generators: bad }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut bad = base.clone();
    bad[0].region = None;
    assert!(
        e.execute(Command::SetNpcGenerators { generators: bad }.into())
            .is_err()
    );
    let mut bad = base.clone();
    bad[0].region.as_mut().unwrap().min[0] = f32::NAN;
    assert!(
        e.execute(Command::SetNpcGenerators { generators: bad }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut old = scene();
    old.version = 63;
    assert!(Engine::new(old).is_err());
    assert!(region().validate(SCENE_VERSION).is_ok());
    assert!(
        Region {
            min: [-10000., 0.8, 10000.],
            max: [10000., 0.8, 10000.]
        }
        .validate(SCENE_VERSION)
        .is_ok()
    );
}
#[test]
fn undo_reset_and_tampered_replay_cover_candidate_positions() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(1).unwrap();
    let first = e.simulation.generators.clone();
    run(&mut e, Command::SetNpcGenerators { generators: vec![] });
    run(&mut e, Command::Undo);
    assert_eq!(state(&e).births, 0);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators, first);
    run(&mut e, Command::Reset);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.generators, first);
    replay(&e);
    let mut r = e.replay();
    r.expected_generators.receipts[0].position.as_mut().unwrap()[0] += 1.;
    assert_eq!(Engine::verify_replay(&r).unwrap()["matched"], false);
}
#[test]
fn full_global_pool_and_cancel_remain_bounded_without_stealing_individuals() {
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
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 0);
    assert_eq!(state(&e).attempts, 8);
    assert!(
        e.simulation
            .generators
            .receipts
            .iter()
            .all(|r| r.outcome == Outcome::Capacity)
    );
    let before = e.simulation.actors.instance_snapshot();
    control(&mut e, Action::Cancel);
    e.advance(4).unwrap();
    assert_eq!(e.simulation.actors.instance_snapshot(), before);
    assert_eq!(state(&e).attempts, 8);
    replay(&e);
}
#[test]
fn regional_candidates_respect_spatial_activation() {
    let mut s = scene();
    let z=serde_json::from_value(json!({"id":"zone","name":"Zone","body":null,"color":[0,1,0],"visible":false,"transform":{"position":[3,1,-4],"size":[2,3,2]},"trigger":{"enabled":true,"targets":["player"]}})).unwrap();
    s.entities.push(z);
    s.npc_generators[0].activation =
        Some(serde_json::from_value(json!({"volume":"zone","mode":"while_occupied"})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(20).unwrap();
    assert_eq!(state(&e).rng, 42);
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
    e.advance(50).unwrap();
    assert_eq!(state(&e).births, 3);
    replay(&e);
}
