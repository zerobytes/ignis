use ignis::{
    engine::{Command, Engine},
    model::{BodyKind, Entity, RoomRecipe, SCENE_VERSION, Scene},
    physics::Input,
    scenario::Scenario,
    trigger::{Config, EVENT_CAPACITY, Runtime},
};
use serde_json::json;

fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    player.transform.position = [-3., 0.85, 2.];
    let mut zone = player.clone();
    zone.id = "exit_zone".into();
    zone.name = "Exit zone".into();
    zone.body = None;
    zone.transform.position = [0., 1., 2.];
    zone.transform.size = [2., 2., 2.];
    zone.visible = Some(false);
    zone.trigger = Some(Config {
        enabled: true,
        targets: vec!["player".into()],
    });
    s.entities.push(zone);
    s
}
fn zone(s: &mut Scene) -> &mut Entity {
    s.entities.iter_mut().find(|e| e.id == "exit_zone").unwrap()
}
#[test]
fn crossing_emits_once_per_transition_without_changing_physics() {
    let s = scene();
    let mut baseline = s.clone();
    baseline.entities.retain(|e| e.trigger.is_none());
    let mut a = Engine::new(s).unwrap();
    let mut b = Engine::new(baseline).unwrap();
    assert_eq!(a.simulation.triggers.state.total_events, 0);
    a.advance(30).unwrap();
    b.advance(30).unwrap();
    a.input = Input {
        x: 1.,
        ..Default::default()
    };
    b.input = a.input;
    a.advance(120).unwrap();
    b.advance(120).unwrap();
    // World placements now include the nonphysical volume; compare physical bodies separately.
    let mut positions = a.simulation.positions();
    assert_eq!(positions.remove("exit_zone"), Some([0., 1., 2.]));
    assert_eq!(positions, b.simulation.positions());
    assert_eq!(a.simulation.contacts(), b.simulation.contacts());
    let v = a.simulation.triggers.volume_state("exit_zone").unwrap();
    assert_eq!((v.entered, v.exited, v.occupancy), (1, 1, 0));
    let report = a.simulation.triggers.inspect();
    assert_eq!(report["events"][0]["kind"], "enter");
    assert_eq!(report["events"][1]["kind"], "exit");
    assert_eq!(report["events"][0]["target"], "player");
    let replay = a.replay();
    assert_eq!(
        Engine::verify_replay(&replay).unwrap()["triggers_matched"],
        true
    );
    let enter = report["events"][0]["tick"].as_u64().unwrap();
    let prefix = Engine::replay_at(&replay, Some(enter)).unwrap();
    assert_eq!(
        prefix.simulation.triggers.occupied("exit_zone", "player"),
        Some(true)
    );
    let mut tampered = replay;
    tampered.expected_triggers.volumes[0].entered += 1;
    assert_eq!(Engine::verify_replay(&tampered).unwrap()["matched"], false);
    a.reset();
    assert_eq!(a.simulation.triggers.state.total_events, 0);
    assert_eq!(
        a.simulation.triggers.occupied("exit_zone", "player"),
        Some(false)
    );
}
#[test]
fn overlap_is_strict_in_three_dimensions_and_does_not_repeat_while_inside() {
    let mut s = scene();
    // Exactly representable boundaries isolate overlap semantics from f32 rounding.
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .size = [1., 1., 1.];
    let mut r = Runtime::new(&s, &ignis::targeting::Catalog::new(&s));
    let half = s
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .size
        .map(|v| v / 2.);
    r.update(1, &ignis::targeting::Catalog::new(&s), |_| {
        [1. + half[0], 1., 2.]
    });
    assert_eq!(r.occupied("exit_zone", "player"), Some(false));
    r.update(2, &ignis::targeting::Catalog::new(&s), |_| {
        [0., 2. + half[1], 2.]
    });
    assert_eq!(r.state.total_events, 0);
    r.update(3, &ignis::targeting::Catalog::new(&s), |_| [0., 1., 2.]);
    for tick in 4..100 {
        r.update(tick, &ignis::targeting::Catalog::new(&s), |_| [0., 1., 2.]);
    }
    assert_eq!(r.state.total_events, 1);
    assert_eq!(
        r.volume_state("exit_zone").unwrap().last_change_tick,
        Some(3)
    );
    r.update(100, &ignis::targeting::Catalog::new(&s), |_| {
        [0., 1., 3. + half[2]]
    });
    assert_eq!(r.state.total_events, 2);
    assert_eq!(r.occupied("exit_zone", "floor"), None);
}
#[test]
fn resolved_targets_and_event_ring_are_bounded_and_order_independent() {
    let mut s = scene();
    let actor = s
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    for i in 1..32 {
        let mut e = actor.clone();
        e.id = format!("actor_{i:02}");
        e.body = Some(BodyKind::Dynamic);
        s.entities.push(e);
    }
    let targets: Vec<_> = s
        .entities
        .iter()
        .filter(|e| e.body == Some(BodyKind::Dynamic) || e.body == Some(BodyKind::Player))
        .map(|e| e.id.clone())
        .collect();
    zone(&mut s).trigger.as_mut().unwrap().targets = targets;
    let original = zone(&mut s).clone();
    for i in 1..32 {
        let mut e = original.clone();
        e.id = format!("zone_{i:02}");
        s.entities.push(e);
    }
    s.validate().unwrap();
    let mut reversed = s.clone();
    reversed.entities.reverse();
    for e in &mut reversed.entities {
        if let Some(t) = &mut e.trigger {
            t.targets.reverse();
        }
    }
    let mut a = Runtime::new(&s, &ignis::targeting::Catalog::new(&s));
    let mut b = Runtime::new(&reversed, &ignis::targeting::Catalog::new(&reversed));
    let capacity = a.state.events.capacity();
    for tick in 1..=20 {
        let mut queries = 0;
        let p = if tick % 2 == 1 {
            [0., 1., 2.]
        } else {
            [10., 1., 2.]
        };
        a.update(tick, &ignis::targeting::Catalog::new(&s), |_| {
            queries += 1;
            p
        });
        b.update(tick, &ignis::targeting::Catalog::new(&reversed), |_| p);
        assert_eq!(queries, 32); // Each shared actor sampled once, not once per volume.
        assert_eq!(a.state.events.capacity(), capacity);
    }
    assert_eq!(a.state, b.state);
    assert_eq!(a.state.total_events, 20 * 32 * 32);
    assert_eq!(a.state.events.len(), EVENT_CAPACITY);
    assert_eq!(
        a.state.dropped_events,
        a.state.total_events - EVENT_CAPACITY as u32
    );
    assert_eq!(a.inspect(), b.inspect());
}
#[test]
fn disabled_volume_and_initial_occupancy_follow_explicit_sampling_lifecycle() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [0., 0.85, 2.];
    let mut active = Engine::new(s.clone()).unwrap();
    assert_eq!(active.simulation.triggers.state.total_events, 0);
    active.advance(1).unwrap();
    assert_eq!(active.simulation.triggers.state.total_events, 1);
    zone(&mut s).trigger.as_mut().unwrap().enabled = false;
    let mut disabled = Engine::new(s).unwrap();
    disabled.advance(10).unwrap();
    assert_eq!(disabled.simulation.triggers.state.total_events, 0);
    active
        .execute(
            Command::Upsert {
                entity: zone(&mut disabled.scene).clone().into(),
            }
            .into(),
        )
        .unwrap();
    active.advance(10).unwrap();
    assert_eq!(active.simulation.triggers.state.total_events, 0);
    active.execute(Command::Undo.into()).unwrap();
    active.advance(1).unwrap();
    assert_eq!(active.simulation.triggers.state.total_events, 1);
}
#[test]
fn references_versions_limits_and_edits_are_validated_atomically() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(20).unwrap();
    let before = e.snapshot();
    for targets in [
        vec![],
        vec!["missing".into()],
        vec!["exit_zone".into()],
        vec!["player".into(); 2],
        vec!["player".into(); 33],
    ] {
        let mut bad = zone(&mut e.scene.clone()).clone();
        bad.trigger.as_mut().unwrap().targets = targets;
        assert!(
            e.execute(Command::Upsert { entity: bad.into() }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let mut bad = zone(&mut e.scene.clone()).clone();
    bad.body = Some(BodyKind::Fixed);
    assert!(
        e.execute(Command::Upsert { entity: bad.into() }.into())
            .is_err()
    );
    assert!(
        e.execute(
            Command::Remove {
                id: "player".into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut old = scene();
    old.version = 9;
    assert!(Engine::new(old.clone()).is_err());
    old.entities.retain(|e| e.trigger.is_none());
    assert_eq!(
        Engine::new(old.clone()).unwrap().scene.version,
        SCENE_VERSION
    );
    old.entities[0].visible = Some(true);
    assert!(Engine::new(old).is_err());
    let mut too_many = scene();
    let original = zone(&mut too_many).clone();
    for i in 0..32 {
        let mut z = original.clone();
        z.id = format!("region_{i}");
        too_many.entities.push(z);
    }
    assert!(Engine::new(too_many).is_err());
    let serialized = serde_json::to_string(&e.scene).unwrap();
    assert_eq!(serde_json::from_str::<Scene>(&serialized).unwrap(), e.scene);
}
#[test]
fn visibility_only_changes_frame_and_never_disables_physics_or_trigger() {
    let mut e = Engine::new(scene()).unwrap();
    assert_eq!(e.render_frame().boxes.len(), e.scene.entities.len() - 1);
    let player = e
        .scene
        .entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap();
    player.visible = Some(false);
    let mut hidden = Engine::new(e.scene.clone()).unwrap();
    e.scene
        .entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .visible = None;
    e.input = Input {
        x: 1.,
        ..Default::default()
    };
    hidden.input = e.input;
    e.advance(60).unwrap();
    hidden.advance(60).unwrap();
    assert_eq!(e.simulation.positions(), hidden.simulation.positions());
    assert_eq!(
        e.simulation.triggers.state,
        hidden.simulation.triggers.state
    );
    assert_eq!(
        hidden.render_frame().boxes.len(),
        e.render_frame().boxes.len() - 1
    );
}
#[test]
fn scenario_v2_verifies_entry_and_exit_with_typed_conditions() {
    let s = scene();
    let script = json!({"version":2,"name":"Cross region","budget_ticks":200,"steps":[
        {"op":"set_input","input":{"x":1,"z":0}},
        {"op":"until","condition":{"kind":"trigger_occupied","id":"exit_zone","target":"player","expected":true},"max_ticks":100},
        {"op":"assert","condition":{"kind":"trigger_entries","id":"exit_zone","at_least":1}},
        {"op":"until","condition":{"kind":"trigger_occupied","id":"exit_zone","target":"player","expected":false},"max_ticks":100}
    ]});
    let mut scenario: Scenario = serde_json::from_value(script.clone()).unwrap();
    let result = scenario.run(&s).unwrap();
    assert!(result.report.passed);
    assert_eq!(result.report.version, 2);
    assert_eq!(
        result
            .engine
            .simulation
            .triggers
            .volume_state("exit_zone")
            .unwrap()
            .exited,
        1
    );
    scenario.version = 1;
    assert!(scenario.validate(&s).is_err());
    let mut bad = script;
    bad["steps"][1]["condition"]["target"] = json!("floor");
    assert!(
        serde_json::from_value::<Scenario>(bad)
            .unwrap()
            .validate(&s)
            .is_err()
    );
}
