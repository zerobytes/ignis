use ignis::{
    engine::{Command, Engine},
    generator_activation::{Config, Mode},
    generator_control::{Action, Status},
    generators::Generator,
    model::{Entity, SCENE_VERSION, Scene},
    physics::Input,
    waves::Phase,
};
use serde_json::{Value, json};
fn scene(mode: Mode) -> Scene {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-waves.scene.json")).unwrap();
    s.version = SCENE_VERSION;
    let zone:Entity=serde_json::from_value(json!({"id":"zone","name":"Activation","body":null,"visible":false,"transform":{"position":[0,1,-4],"size":[2,3,2]},"color":[0,1,0],"trigger":{"enabled":true,"targets":["player"]}})).unwrap();
    s.entities.push(zone);
    let g = &mut s.npc_generators[0];
    g.waves.clear();
    g.interval_ticks = 4;
    g.target_alive = 1;
    g.retire_dead_after_ticks = Some(2);
    g.activation = Some(Config {
        volume: "zone".into(),
        mode,
    });
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn state(e: &Engine) -> Generator {
    e.simulation.generators.generators[0].clone()
}
fn open(e: &Engine) -> bool {
    state(e).activation.unwrap().open
}
fn info(e: &mut Engine) -> Value {
    run(e, Command::InspectGenerators)
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
fn walk(e: &mut Engine, x: f32, ticks: u32) {
    run(
        e,
        Command::SetInput {
            input: Input {
                x,
                z: 0.,
                ..Default::default()
            },
        },
    );
    e.advance(ticks).unwrap();
    run(
        e,
        Command::SetInput {
            input: Input::default(),
        },
    );
    e.advance(1).unwrap();
}
fn name(e: &Engine) -> String {
    let m = state(e).members.into_iter().flatten().next().unwrap();
    e.simulation
        .actors
        .label(e.simulation.definitions(), m.instance)
        .unwrap()
        .to_owned()
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
fn initially_inside_waits_for_physics_then_preserves_initial_delay() {
    let mut s = scene(Mode::WhileOccupied);
    s.npc_generators[0].initial_delay_ticks = 5;
    let mut e = Engine::new(s).unwrap();
    assert!(!open(&e));
    e.advance(1).unwrap();
    assert!(!open(&e));
    assert_eq!(state(&e).births, 0);
    e.advance(5).unwrap();
    assert!(open(&e));
    assert_eq!(state(&e).next_attempt_tick, 6);
    assert_eq!(state(&e).births, 0);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(state(&e).activation.unwrap().last_change_tick, Some(1));
    replay(&e);
}
#[test]
fn continuous_condition_freezes_rng_cadence_and_replenishment_outside() {
    let mut e = Engine::new(scene(Mode::WhileOccupied)).unwrap();
    e.advance(2).unwrap();
    walk(&mut e, 1., 50);
    assert!(!open(&e));
    let old = name(&e);
    cast(&mut e, 1, &old);
    e.advance(10).unwrap();
    assert!(state(&e).members.iter().all(Option::is_none));
    let rng = state(&e).rng;
    let clock = state(&e).paused_since;
    let remaining = info(&mut e)["controls"][0]["remaining_cadence_ticks"].clone();
    e.advance(30).unwrap();
    assert_eq!(state(&e).births, 1);
    assert_eq!(state(&e).rng, rng);
    assert_eq!(state(&e).paused_since, clock);
    assert_eq!(
        info(&mut e)["controls"][0]["remaining_cadence_ticks"],
        remaining
    );
    walk(&mut e, -1., 50);
    assert!(open(&e));
    assert_eq!(state(&e).births, 2);
    replay(&e);
}
#[test]
fn once_latches_without_restarting_seed_or_adopting_manual_authority() {
    let mut e = Engine::new(scene(Mode::Once)).unwrap();
    e.advance(2).unwrap();
    walk(&mut e, 1., 50);
    assert!(open(&e));
    assert!(!state(&e).activation.unwrap().occupied);
    assert_eq!(state(&e).activation.unwrap().changes, 1);
    let old = name(&e);
    cast(&mut e, 1, &old);
    e.advance(5).unwrap();
    assert_eq!(state(&e).births, 2);
    control(&mut e, Action::Pause);
    e.advance(1).unwrap();
    let old = name(&e);
    cast(&mut e, 1, &old);
    e.advance(10).unwrap();
    assert_eq!(state(&e).births, 2);
    control(&mut e, Action::Activate);
    e.advance(5).unwrap();
    assert_eq!(state(&e).births, 3);
    replay(&e);
}
#[test]
fn manual_pause_and_region_share_one_suspension_and_wave_delay() {
    let mut s = scene(Mode::WhileOccupied);
    let g = &mut s.npc_generators[0];
    g.waves = vec![
        serde_json::from_value(json!({"count":1})).unwrap(),
        serde_json::from_value(json!({"count":1,"delay_ticks":180})).unwrap(),
    ];
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    let old = name(&e);
    run(&mut e, Command::DespawnActor { actor: old });
    e.advance(1).unwrap();
    assert_eq!(state(&e).wave.unwrap().current, 1);
    walk(&mut e, 1., 50);
    assert!(!open(&e));
    let since = state(&e).paused_since.unwrap();
    let due = state(&e).wave.unwrap().due_tick;
    control(&mut e, Action::Pause);
    e.advance(20).unwrap();
    walk(&mut e, -1., 50);
    assert!(open(&e));
    assert_eq!(state(&e).status, Status::Paused);
    assert_eq!(state(&e).paused_since, Some(since));
    let t = e.simulation.tick;
    let remaining = (due - since) as u32;
    control(&mut e, Action::Activate);
    e.advance(remaining).unwrap();
    assert_eq!(state(&e).wave.unwrap().due_tick, due + t - since);
    assert_eq!(state(&e).births, 1);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 2);
    replay(&e);
}
#[test]
fn manual_pause_before_first_occupancy_and_cancel_remain_authoritative() {
    let mut s = scene(Mode::Once);
    s.npc_generators[0].initially_active = false;
    let mut e = Engine::new(s).unwrap();
    e.advance(20).unwrap();
    assert!(open(&e));
    assert_eq!(state(&e).births, 0);
    control(&mut e, Action::Activate);
    control(&mut e, Action::Pause);
    e.advance(1).unwrap();
    assert_eq!(state(&e).births, 0);
    control(&mut e, Action::Cancel);
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    walk(&mut e, 1., 50);
    walk(&mut e, -1., 50);
    assert_eq!(state(&e).status, Status::Cancelled);
    assert_eq!(state(&e).births, 0);
    replay(&e);
}
#[test]
fn activation_cannot_override_closed_condition_and_no_npc_knowledge_is_injected() {
    let mut s = scene(Mode::WhileOccupied);
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position[0] = 4.;
    let mut e = Engine::new(s).unwrap();
    control(&mut e, Action::Activate);
    e.advance(20).unwrap();
    assert!(!state(&e).production_enabled());
    assert_eq!(state(&e).births, 0);
    walk(&mut e, -1., 60);
    assert!(open(&e));
    assert_eq!(state(&e).births, 1);
    let m = state(&e).members[0].unwrap();
    let npc = &e.simulation.definitions().entities[e.simulation.actors.entity(m.actor)];
    assert!(npc.npc.as_ref().unwrap().target.is_none());
    assert!(!npc.npc.as_ref().unwrap().enabled);
    replay(&e);
}
#[test]
fn removed_and_reused_target_does_not_hold_or_open_the_condition() {
    let mut s = scene(Mode::WhileOccupied);
    let mut witness = s.npc_templates[0].clone();
    witness.id = "witness".into();
    witness.transform.position = [0., 0.8, -4.];
    s.entities.push(witness);
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position[0] = 4.;
    s.entities
        .iter_mut()
        .find(|e| e.id == "zone")
        .unwrap()
        .trigger
        .as_mut()
        .unwrap()
        .targets = vec!["witness".into()];
    s.npc_generators[0].initially_active = false;
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    assert!(open(&e));
    run(
        &mut e,
        Command::DespawnActor {
            actor: "witness".into(),
        },
    );
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [0., 0.8, -4.],
            target: None,
        },
    );
    control(&mut e, Action::Activate);
    e.advance(1).unwrap();
    assert!(!open(&e));
    assert_eq!(state(&e).births, 0);
    e.advance(5).unwrap();
    assert!(!open(&e));
    replay(&e);
}
#[test]
fn physical_dead_occupant_counts_until_removed_and_disabled_volume_stays_closed() {
    let mut s = scene(Mode::WhileOccupied);
    let mut witness = s.npc_templates[0].clone();
    witness.id = "witness".into();
    witness.transform.position = [0., 0.8, -4.];
    s.entities.push(witness);
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position[0] = 4.;
    s.entities
        .iter_mut()
        .find(|e| e.id == "zone")
        .unwrap()
        .trigger
        .as_mut()
        .unwrap()
        .targets = vec!["witness".into()];
    let mut e = Engine::new(s.clone()).unwrap();
    e.advance(2).unwrap();
    cast(&mut e, 1, "witness");
    e.advance(2).unwrap();
    assert!(open(&e));
    cast(&mut e, 2, "witness");
    assert!(open(&e));
    replay(&e);
    s.entities
        .iter_mut()
        .find(|e| e.id == "zone")
        .unwrap()
        .trigger
        .as_mut()
        .unwrap()
        .enabled = false;
    let mut e = Engine::new(s).unwrap();
    e.advance(5).unwrap();
    assert!(!open(&e));
    assert_eq!(state(&e).births, 0);
    replay(&e);
}
#[test]
fn trigger_order_xyz_and_explicit_target_selection_are_respected() {
    for (position, expected) in [
        ([0., 1., -4.], true),
        ([0., 8., -4.], false),
        ([0., 1., 4.], false),
        ([8., 1., -4.], false),
    ] {
        let mut s = scene(Mode::WhileOccupied);
        let z = s.entities.iter_mut().find(|e| e.id == "zone").unwrap();
        z.transform.position = position;
        let mut unrelated = z.clone();
        unrelated.id = "aaa_unrelated".into();
        unrelated.transform.position = [0., 1., -4.];
        s.entities.insert(0, unrelated);
        let mut e = Engine::new(s).unwrap();
        e.advance(2).unwrap();
        assert_eq!(open(&e), expected);
        assert_eq!(state(&e).activation.unwrap().volume, 1);
        replay(&e);
    }
}
#[test]
fn dependent_volume_edit_is_atomic_and_reset_undo_rearm_once() {
    let mut e = Engine::new(scene(Mode::Once)).unwrap();
    e.advance(2).unwrap();
    let before = e.snapshot();
    assert!(
        e.execute(Command::Remove { id: "zone".into() }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut bad = e.scene.npc_generators.clone();
    bad[0].activation.as_mut().unwrap().volume = "floor".into();
    assert!(
        e.execute(Command::SetNpcGenerators { generators: bad }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let authored = e.scene.npc_generators.clone();
    run(&mut e, Command::SetNpcGenerators { generators: vec![] });
    run(&mut e, Command::Undo);
    assert!(!open(&e));
    assert_eq!(e.scene.npc_generators, authored);
    e.advance(2).unwrap();
    run(&mut e, Command::Reset);
    assert!(!open(&e));
    assert_eq!(state(&e).births, 0);
    replay(&e);
    let mut old = scene(Mode::Once);
    old.version = 62;
    assert!(Engine::new(old).is_err());
}
#[test]
fn prefix_replays_gate_and_partial_clock_without_future_occupancy() {
    let mut e = Engine::new(scene(Mode::WhileOccupied)).unwrap();
    e.advance(1).unwrap();
    let initial = state(&e);
    e.advance(1).unwrap();
    walk(&mut e, 1., 50);
    let outside = state(&e);
    let tick = e.simulation.tick;
    walk(&mut e, -1., 50);
    replay(&e);
    let r = e.replay();
    assert_eq!(state(&Engine::replay_at(&r, Some(1)).unwrap()), initial);
    assert_eq!(state(&Engine::replay_at(&r, Some(tick)).unwrap()), outside);
    let mut bad = r.clone();
    bad.expected_generators.generators[0]
        .activation
        .as_mut()
        .unwrap()
        .open = false;
    assert_eq!(Engine::verify_replay(&bad).unwrap()["matched"], false);
}
#[test]
fn condition_can_defer_final_completion_without_reopening_completed_waves() {
    let mut s = scene(Mode::WhileOccupied);
    s.npc_generators[0].waves = vec![serde_json::from_value(json!({"count":1})).unwrap()];
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    walk(&mut e, 1., 50);
    let old = name(&e);
    cast(&mut e, 1, &old);
    e.advance(4).unwrap();
    assert_eq!(state(&e).wave.unwrap().phase, Phase::WaitingForClear);
    walk(&mut e, -1., 50);
    assert_eq!(state(&e).wave.unwrap().phase, Phase::Completed);
    let completed = state(&e).wave.unwrap();
    walk(&mut e, 1., 50);
    walk(&mut e, -1., 50);
    assert_eq!(state(&e).wave.unwrap(), completed);
    assert_eq!(state(&e).births, 1);
    replay(&e);
}
