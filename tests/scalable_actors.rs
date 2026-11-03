//! Critical guard: actor indices above 255 and membership above 64 must not
//! corrupt control ownership, lifecycle, spawn reuse or deterministic replay.
use ignis::{
    actor_set::ActorSet,
    engine::{Command, Engine},
    model::{BodyKind, RoomRecipe, Scene},
    physics::Input,
};
use serde_json::json;

fn run(e: &mut Engine, command: Command) -> serde_json::Value {
    e.execute(command.into()).unwrap()
}
fn id(slot: usize) -> String {
    format!("p{slot:04}")
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 63.,
        depth: 63.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut original = s
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Player))
        .unwrap()
        .clone();
    s.entities.retain(|e| e.body != Some(BodyKind::Player));
    original.footsteps = None;
    original.blocks_vision = Some(false);
    original.vitality =
        Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
    original.abilities = Some(
        serde_json::from_value(json!({"resources":[],"grants":["kill","revive","wave"]})).unwrap(),
    );
    for actor in 0..1000 {
        let mut p = original.clone();
        p.id = id(actor);
        p.name = p.id.clone();
        p.transform.position = [
            -23.25 + (actor % 32) as f32 * 1.5,
            0.8,
            -23.25 + (actor / 32) as f32 * 1.5,
        ];
        s.entities.push(p);
    }
    s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[
        {"id":"kill","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},
        {"id":"revive","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":50}]},
        {"id":"wave","selection":"self_actor","range":0,"area":{"radius":100,"max_targets":1000,"include_source":true,"occlusion":false},"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":1}]}
    ]})).unwrap();
    s.npc_templates.push(serde_json::from_value(json!({"id":"reserve","name":"reserve","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[0,0.8,29],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":3,"stop_distance":0.5,"enabled":true,"target":"p0999","awareness":{"mode":"omniscient"}}})).unwrap());
    s
}

#[test]
fn bitset_word_boundaries_survive_removal_and_serialization() {
    let mut set = ActorSet::EMPTY;
    let slots = [0, 15, 16, 63, 64, 255, 256, 999, 1023];
    for &a in &slots {
        set.insert(a);
    }
    let restored: ActorSet = serde_json::from_str(&serde_json::to_string(&set).unwrap()).unwrap();
    assert_eq!(set, restored);
    set.remove(64);
    set.remove(999);
    for &a in &slots {
        assert_eq!(set.contains(a), a != 64 && a != 999);
    }
    assert!(!set.contains(1024));
    for a in slots {
        set.remove(a);
    }
    assert!(set.is_empty());
}

#[test]
fn thousand_controlled_players_keep_high_slot_lifecycle_identity_and_replay() {
    let start = std::time::Instant::now();
    let mut e = Engine::new(scene()).unwrap();
    assert_eq!(e.simulation.actors.actors.len(), 1008);
    assert_eq!(e.simulation.abilities.state.balances.len(), 1008);
    assert_eq!(e.simulation.vitality.state.actors.len(), 1008);
    let mut tokens = Vec::with_capacity(1000);
    for a in 0..1000 {
        let binding = run(&mut e, Command::BindControl { actor: id(a) });
        let token = binding["control"].as_str().unwrap().to_owned();
        run(
            &mut e,
            Command::SetControlInput {
                control: token.clone(),
                input: Input {
                    x: 1.,
                    z: 0.,
                    ..Default::default()
                },
            },
        );
        tokens.push(token);
    }
    assert_eq!(
        run(&mut e, Command::InspectControls)["players"]
            .as_array()
            .unwrap()
            .len(),
        1000
    );
    let bind_elapsed = start.elapsed();
    e.advance(1).unwrap();
    run(
        &mut e,
        Command::ActivateAbility {
            actor: id(999),
            slot: 3,
            target: None,
        },
    );
    e.advance(1).unwrap();
    assert!(
        e.simulation.vitality.state.actors[..1000]
            .iter()
            .all(|h| h.unwrap().units == 99)
    );
    for (source, victim) in [(0, 64), (1, 256), (2, 999)] {
        run(
            &mut e,
            Command::ActivateAbility {
                actor: id(source),
                slot: 1,
                target: Some(id(victim)),
            },
        );
    }
    e.advance(1).unwrap();
    for victim in [64, 256, 999] {
        assert!(e.simulation.vitality.state.died_this_tick.contains(victim));
        assert_eq!(e.simulation.vitality.state.actors[victim].unwrap().units, 0);
    }
    run(
        &mut e,
        Command::ActivateAbility {
            actor: id(3),
            slot: 2,
            target: Some(id(999)),
        },
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.vitality.state.lives[999], 1);
    assert!(e.simulation.vitality.state.revived_this_tick.contains(999));
    run(
        &mut e,
        Command::SetControlInput {
            control: tokens[999].clone(),
            input: Input {
                x: 0.,
                z: 1.,
                ..Default::default()
            },
        },
    );
    run(
        &mut e,
        Command::SpawnActor {
            template: "reserve".into(),
            position: [0., 0.8, 29.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    let born = e.simulation.actors.reference(1000);
    let label = e
        .simulation
        .actors
        .label(e.simulation.definitions(), born)
        .unwrap()
        .to_owned();
    assert_eq!(
        e.simulation.npcs[&label].memory.unwrap().actor_index,
        Some(999)
    );
    run(&mut e, Command::DespawnActor { actor: label });
    e.advance(1).unwrap();
    run(
        &mut e,
        Command::SpawnActor {
            template: "reserve".into(),
            position: [0., 0.8, 29.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    let replacement = e.simulation.actors.reference(1000);
    assert_ne!(replacement, born);
    assert!(!e.simulation.actors.contains(born));
    let replay = e.replay();
    let decoded = serde_json::from_str(&serde_json::to_string(&replay).unwrap()).unwrap();
    assert_eq!(Engine::verify_replay(&decoded).unwrap()["matched"], true);
    eprintln!(
        "1000 control setup {:?}; critical workload incl replay {:?}",
        bind_elapsed,
        start.elapsed()
    );
}

/// Critical: a noisy owner must not consume another actor's slot, and frame indexing
/// must preserve every source/target identity and replay/prefix admission budget.
#[test]
fn thousand_same_tick_actions_and_control_updates_preserve_fair_admission_and_replay() {
    use ignis::engine::ExecutionMode;
    let mut s = scene();
    for p in s
        .entities
        .iter_mut()
        .filter(|e| e.body == Some(BodyKind::Player))
    {
        let hp = p.vitality.as_mut().unwrap();
        hp.initial = 100_000;
        hp.maximum = 100_000;
    }
    let mut e = Engine::new_with_mode(s, ExecutionMode::Continuous).unwrap();
    let mut controls = Vec::new();
    for a in 0..1000 {
        controls.push(
            run(&mut e, Command::BindControl { actor: id(a) })["control"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    for i in 0..5000 {
        run(
            &mut e,
            Command::SetControlInput {
                control: controls[0].clone(),
                input: Input {
                    x: if i % 2 == 0 { 0. } else { 0.25 },
                    ..Default::default()
                },
            },
        );
    }
    for control in &controls[1..] {
        run(
            &mut e,
            Command::SetControlInput {
                control: control.clone(),
                input: Input::default(),
            },
        );
    }
    run(
        &mut e,
        Command::ReleaseControl {
            control: controls[0].clone(),
        },
    );
    let replacement = run(&mut e, Command::BindControl { actor: id(0) })["control"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(replacement, controls[0]);
    let before = e.inspect_execution();
    assert!(
        e.execute(
            Command::SetControlInput {
                control: controls[0].clone(),
                input: Input::default()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.inspect_execution(), before);
    run(
        &mut e,
        Command::SetControlInput {
            control: replacement,
            input: Input {
                x: 0.25,
                ..Default::default()
            },
        },
    );
    let owned = run(&mut e, Command::InspectControls);
    assert_eq!(owned["players"][0]["input"]["x"], 0.25);
    assert_eq!(owned["recorded_events"], 0);

    let instances = run(&mut e, Command::InspectInstances);
    let tokens: Vec<_> = (0..1000)
        .map(|a| {
            instances["instances"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["authored_id"] == id(a))
                .unwrap()["reference"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    for _ in 0..16 {
        run(
            &mut e,
            Command::ActivateAbility {
                actor: tokens[0].clone(),
                slot: 1,
                target: Some(tokens[1].clone()),
            },
        );
    }
    let before = e.inspect_execution();
    assert!(
        e.execute(
            Command::ActivateAbility {
                actor: tokens[0].clone(),
                slot: 1,
                target: Some(id(1))
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.inspect_execution(), before);
    for a in 1..1000 {
        let source = if a % 2 == 0 { tokens[a].clone() } else { id(a) };
        run(
            &mut e,
            Command::ActivateAbility {
                actor: source,
                slot: 1,
                target: Some(tokens[(a + 1) % 1000].clone()),
            },
        );
    }
    assert_eq!(e.inspect_execution()["pending"]["abilities"], 1015);
    e.advance(1).unwrap();
    assert!(
        e.simulation.vitality.state.actors[..1000]
            .iter()
            .all(|h| h.unwrap().units == 99_900)
    );
    assert_eq!(e.inspect_execution()["pending"]["abilities"], 0);
    assert_eq!(e.inspect_execution()["retained"]["abilities"], 0);

    e.reset();
    e.set_execution_mode(ExecutionMode::Recorded).unwrap();
    for a in 0..1000 {
        run(
            &mut e,
            Command::ActivateAbility {
                actor: id(a),
                slot: 1,
                target: Some(id((a + 1) % 1000)),
            },
        );
    }
    e.advance(1).unwrap();
    let replay = e.recorded_replay().unwrap();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let mut prefix = Engine::replay_at(&replay, Some(0)).unwrap();
    assert_eq!(prefix.inspect_execution()["pending"]["abilities"], 1000);
    for _ in 1..16 {
        run(
            &mut prefix,
            Command::ActivateAbility {
                actor: id(0),
                slot: 1,
                target: Some(id(1)),
            },
        );
    }
    let before = prefix.inspect_execution();
    assert!(
        prefix
            .execute(
                Command::ActivateAbility {
                    actor: id(0),
                    slot: 1,
                    target: Some(id(1))
                }
                .into()
            )
            .is_err()
    );
    assert_eq!(prefix.inspect_execution(), before);
    assert_eq!(
        Engine::verify_replay(&prefix.recorded_replay().unwrap()).unwrap()["matched"],
        true
    );
}

// Critical: source quotas and leases above255 must preserve payment, retained
// identity, dense compaction and replay when1000authors launch simultaneously.
#[test]
fn thousand_projectile_sources_keep_fair_quotas_origins_and_exact_impacts() {
    use ignis::abilities::{Aim, Outcome};
    let mut s = scene();
    let mut npc = s.npc_templates.remove(0);
    npc.id = "zz_source".into();
    npc.name = npc.id.clone();
    npc.transform.position = [28., 0.8, 28.];
    let config = npc.npc.as_mut().unwrap();
    config.enabled = true;
    config.target = None;
    config.objectives = Some(
        serde_json::from_value(json!([{"priority":1,"when":"always","action":{"kind":"wait"}}]))
            .unwrap(),
    );
    s.entities.push(npc);
    s.ability_catalog = serde_json::from_value(json!({
        "resources":[{"id":"energy","scale":1}],"abilities":[{
            "id":"bolt","selection":"spatial","range":0,"cast_ticks":0,"cooldown_ticks":0,
            "costs":[{"resource":"energy","units":5}],
            "delivery":{"kind":"projectile","speed":0.1,"half_extent":0.01,"max_distance":100},
            "effects":[{"kind":"damage","recipient":"target","amount":1}]
        }]
    }))
    .unwrap();
    for p in s
        .entities
        .iter_mut()
        .filter(|p| matches!(p.body, Some(BodyKind::Player | BodyKind::Npc)))
    {
        p.abilities = Some(
            serde_json::from_value(json!({
                "resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["bolt"]
            }))
            .unwrap(),
        );
    }
    let launch = |e: &mut Engine, actor: String, vector| {
        run(
            e,
            Command::AimAbility {
                actor,
                slot: 1,
                aim: Aim::Direction { vector },
            },
        );
    };
    let mut e = Engine::new(s.clone()).unwrap();
    for a in 0..1000 {
        launch(&mut e, id(a), [0., 1., 0.]);
    }
    launch(&mut e, "zz_source".into(), [0., 1., 0.]);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.projectiles.active.len(), 1001);
    let old_npc = e.simulation.actors.reference(1000);
    let high_lease = e
        .simulation
        .abilities
        .state
        .projectiles
        .active
        .source_positions(1000)[0];
    let high_lease = e.simulation.abilities.state.projectiles.active[high_lease]
        .cast
        .origin
        .unwrap();
    assert_eq!(high_lease, 1000);
    let kept_ref = e.simulation.actors.reference(999);
    run(
        &mut e,
        Command::DespawnActor {
            actor: "zz_source".into(),
        },
    );
    for _ in 1..64 {
        launch(&mut e, id(0), [0., 1., 0.]);
        e.advance(1).unwrap();
    }
    let state = &e.simulation.abilities.state;
    assert_eq!(state.projectiles.active.len(), 1064);
    assert_eq!(state.projectiles.active.source_positions(0).len(), 64);
    assert_eq!(state.projectiles.active.source_positions(999).len(), 1);
    let detached = state.origins.get(Some(high_lease), old_npc).unwrap();
    assert!(detached.detached);
    assert!(detached.attributes.bases.capacity() >= 65);
    assert!(detached.attributes.contributions.capacity() >= 64);
    let before = state.balances[0];
    let ready = state.ready_at[0];
    launch(&mut e, id(0), [0., 1., 0.]);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::ProjectileBudget
    );
    assert_eq!(e.simulation.abilities.state.balances[0], before);
    assert_eq!(e.simulation.abilities.state.ready_at[0], ready);
    launch(&mut e, id(1), [0., 1., 0.]);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.projectiles.active.len(), 1065);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .projectiles
            .active
            .source_positions(1)
            .len(),
        2
    );
    let encoded = serde_json::to_value(&e.simulation.abilities.state).unwrap();
    let mut restored: ignis::abilities::State = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(restored, e.simulation.abilities.state);
    assert!(!restored.projectiles.active.available(0, 1001));
    assert!(restored.projectiles.active.available(2, 1001));
    assert_eq!(restored.origins.reserve(999, kept_ref), Some(999));
    restored.origins.collect([(999, kept_ref)]);
    let replacement = ignis::identity::Reference {
        instance: kept_ref.instance + 10000,
        ..kept_ref
    };
    assert_eq!(restored.origins.reserve(999, replacement), Some(0));
    assert!(
        restored
            .origins
            .get(Some(0), e.simulation.actors.reference(0))
            .is_none()
    );
    assert!(restored.origins.get(Some(999), kept_ref).is_some());
    let mut invalid = encoded.clone();
    let shot = invalid["projectiles"]["active"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["actor"] == 0)
        .unwrap()
        .clone();
    invalid["projectiles"]["active"] = json!(vec![shot; 65]);
    assert!(serde_json::from_value::<ignis::abilities::State>(invalid).is_err());
    let mut invalid = encoded;
    invalid["origins"]["slots"][1]["reference"] =
        invalid["origins"]["slots"][0]["reference"].clone();
    assert!(serde_json::from_value::<ignis::abilities::State>(invalid).is_err());
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut prior_version = e.replay();
    prior_version.version -= 1;
    assert!(Engine::verify_replay(&prior_version).is_err());

    // Real impacts remove1000entries and rebuild derived source indices together.
    s.ability_catalog.abilities[0].delivery = ignis::projectiles::Delivery::Projectile {
        speed: 100.,
        half_extent: 0.01,
        max_distance: 100.,
    };
    let mut fast = Engine::new(s).unwrap();
    for a in 0..1000 {
        launch(
            &mut fast,
            id(a),
            [if a % 2 == 0 { 1. } else { -1. }, 0., 0.],
        );
    }
    fast.advance(1).unwrap();
    assert_eq!(
        fast.simulation.abilities.state.projectiles.active.len(),
        1000
    );
    fast.advance(1).unwrap();
    assert!(
        fast.simulation
            .abilities
            .state
            .projectiles
            .active
            .is_empty()
    );
    assert!(
        fast.simulation
            .abilities
            .state
            .projectiles
            .active
            .source_positions(999)
            .is_empty()
    );
    assert!(
        fast.simulation
            .abilities
            .state
            .origins
            .get(Some(999), fast.simulation.actors.reference(999))
            .is_none()
    );
    assert!(
        fast.simulation.vitality.state.actors[..1000]
            .iter()
            .all(|h| h.unwrap().units == 99)
    );
    assert_eq!(
        Engine::verify_replay(&fast.replay()).unwrap()["matched"],
        true
    );
}

// Critical: agenda positions above63 must not alias dispel selections or let a
// late resource failure consume costs, HP, schedules or another source's quota.
#[test]
fn thousand_scheduled_sources_keep_local_dispels_quotas_and_lifecycle_order() {
    use ignis::abilities::Outcome;
    let mut s = scene();
    s.npc_templates.clear();
    s.ability_catalog = serde_json::from_value(json!({"resources":[{"id":"energy","scale":1},{"id":"focus","scale":1}],"abilities":[
        {"id":"dot","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":1}],"effects":[{"kind":"damage","recipient":"target","amount":1}],"periodic":{"interval_ticks":3600,"pulses":1,"reapply":"independent"},"periodic_category":"poison"},
        {"id":"field","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage","recipient":"target","amount":1}],"area":{"radius":0.5,"max_targets":1,"include_source":true,"occlusion":false,"persistence":{"duration_ticks":7200,"interval_ticks":3600,"apply_on_spawn":false,"reapply":"independent","end_on_source_death":true}}},
        {"id":"clean","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":7}],"effects":[{"kind":"dispel","recipient":"target","category":"poison","max_instances":64,"require_match":true}]},
        {"id":"late","selection":"actor","range":100,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":7}],"effects":[{"kind":"dispel","recipient":"target","category":"poison","max_instances":64,"require_match":true},{"kind":"damage","recipient":"target","amount":1},{"kind":"resource","recipient":"target","resource":"focus","delta":1}]},
        {"id":"refresh","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":1}],"effects":[{"kind":"damage","recipient":"target","amount":1}],"periodic":{"interval_ticks":3600,"pulses":1,"reapply":"refresh"},"periodic_category":"poison"},
        {"id":"die","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":1000000}]}
    ]})).unwrap();
    for p in s
        .entities
        .iter_mut()
        .filter(|p| p.body == Some(BodyKind::Player))
    {
        p.abilities = Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["dot","field","clean","late","refresh","die"]})).unwrap());
        p.vitality = Some(
            serde_json::from_value(json!({"initial":1000000,"maximum":1000000,"scale":1})).unwrap(),
        );
    }
    let mut e = Engine::new(s).unwrap();
    let cast = |e: &mut Engine, actor, slot| {
        run(
            e,
            Command::ActivateAbility {
                actor: id(actor),
                slot,
                target: (slot == 4).then(|| id(actor)),
            },
        );
    };
    for a in 0..1000 {
        cast(&mut e, a, 1);
    }
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 1000);
    e.debug_view.npc_routes = true;
    assert_eq!(
        e.render_frame()
            .boxes
            .iter()
            .filter(|b| b.color == [0.8, 0.4, 1.])
            .count(),
        1000
    );

    for a in 0..1000 {
        cast(&mut e, a, 2);
    }
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.abilities.state.persistent_areas.active.len(),
        1000
    );
    cast(&mut e, 999, 5);
    e.advance(1).unwrap();
    for _ in 0..62 {
        cast(&mut e, 999, 1);
        e.advance(1).unwrap();
    }
    cast(&mut e, 999, 5);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 1063);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .periodic
            .active
            .source_positions(999)
            .len(),
        64
    );
    assert_eq!(
        e.simulation
            .abilities
            .state
            .periodic
            .active
            .target_positions(999)
            .len(),
        64
    );
    let balance = e.simulation.abilities.state.balances[999];
    cast(&mut e, 999, 1);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.balances[999], balance);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::PeriodicBudget
    );
    for _ in 0..15 {
        cast(&mut e, 0, 2);
        e.advance(1).unwrap();
    }
    assert_eq!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .source_positions(0)
            .len(),
        16
    );
    let balance = e.simulation.abilities.state.balances[0];
    cast(&mut e, 0, 2);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.balances[0], balance);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::AreaBudget
    );
    cast(&mut e, 0, 1);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation
            .abilities
            .state
            .periodic
            .active
            .source_positions(0)
            .len(),
        2
    );
    let encoded = serde_json::to_value(&e.simulation.abilities.state).unwrap();
    let restored: ignis::abilities::State = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(restored, e.simulation.abilities.state);
    assert_eq!(restored.periodic.active.target_positions(999).len(), 64);
    assert_eq!(
        restored.persistent_areas.active.source_positions(0).len(),
        16
    );
    let mut invalid = encoded;
    invalid["periodic"]["active"][0]["cast"]["target"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<ignis::abilities::State>(invalid).is_err());
    let selected: Vec<_> = e
        .simulation
        .abilities
        .state
        .periodic
        .active
        .iter()
        .filter(|i| i.cast.target == Some(999))
        .map(|i| i.cast.id)
        .collect();
    assert_eq!(selected.len(), 64);
    assert!(selected.windows(2).any(|v| v[0] > v[1])); // Refresh changed an ID without changing dense position.
    let balance = e.simulation.abilities.state.balances[999];
    let hp = e.simulation.vitality.state.actors[999];
    cast(&mut e, 999, 4);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::MissingResource
    );
    assert_eq!(e.simulation.abilities.state.balances[999], balance);
    assert_eq!(e.simulation.vitality.state.actors[999], hp);
    assert_eq!(
        selected,
        e.simulation
            .abilities
            .state
            .periodic
            .active
            .iter()
            .filter(|i| i.cast.target == Some(999))
            .map(|i| i.cast.id)
            .collect::<Vec<_>>()
    );
    cast(&mut e, 999, 3);
    e.advance(1).unwrap();
    assert!(
        e.simulation
            .abilities
            .state
            .periodic
            .active
            .target_positions(999)
            .is_empty()
    );
    assert_eq!(e.simulation.abilities.state.periodic.active.len(), 1000);
    assert_eq!(
        e.simulation.abilities.state.balances[999][0],
        balance[0] - 7
    );
    let removed: Vec<_> = e
        .simulation
        .abilities
        .state
        .periodic
        .receipts
        .iter()
        .filter(|r| r.outcome == ignis::periodic::Outcome::Dispelled)
        .map(|r| r.execution_id)
        .collect();
    assert_eq!(selected, removed);
    // Death occurs before the next source's area commit, exactly as before batching.
    cast(&mut e, 0, 6);
    cast(&mut e, 1, 2);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.vitality.state.actors[0].unwrap().units, 0);
    assert!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .source_positions(0)
            .is_empty()
    );
    assert_eq!(
        e.simulation
            .abilities
            .state
            .persistent_areas
            .active
            .source_positions(1)
            .len(),
        2
    );
    let receipts = &e.simulation.abilities.state.persistent_areas.receipts;
    assert_eq!(
        receipts.back().unwrap().outcome,
        ignis::persistent_area::Outcome::Created
    );
    assert_eq!(
        receipts
            .iter()
            .rev()
            .skip(1)
            .take(16)
            .filter(|r| r.actor == 0 && r.outcome == ignis::persistent_area::Outcome::SourceDead)
            .count(),
        16
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}

/// Critical: one source's sound flood must not consume peers' combat budget or
/// charge partially; pending noise/IDs must survive State restoration and replay.
#[test]
fn thousand_noisy_sources_keep_per_source_atomicity_and_next_tick_delivery() {
    use ignis::abilities::{Effect, Outcome};
    let mut s = scene();
    s.npc_templates.clear();
    s.ability_catalog.resources =
        serde_json::from_value(json!([{"id":"energy","scale":1}])).unwrap();
    let d = &mut s.ability_catalog.abilities[0];
    d.costs = serde_json::from_value(json!([{"resource":"energy","units":5}])).unwrap();
    d.effects =
        serde_json::from_value(json!([{"kind":"damage","recipient":"target","amount":1}])).unwrap();
    d.effects
        .extend(std::iter::repeat_n(Effect::Noise { radius: 0.1 }, 7));
    for p in s
        .entities
        .iter_mut()
        .filter(|e| e.body == Some(BodyKind::Player))
    {
        p.blocks_vision = None;
        p.vitality = Some(
            serde_json::from_value(json!({"initial":1000000,"maximum":1000000,"scale":1})).unwrap(),
        );
        p.abilities = Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["kill"]})).unwrap());
    }
    let mut e = Engine::new(s).unwrap();
    let cast = |e: &mut Engine, a| {
        run(
            e,
            Command::ActivateAbility {
                actor: id(a),
                slot: 1,
                target: Some(id(a ^ 1)),
            },
        )
    };
    for a in 0..1000 {
        cast(&mut e, a);
    }
    cast(&mut e, 999);
    cast(&mut e, 999);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::Busy
    );
    assert_eq!(e.simulation.abilities.state.noises.len(), 7000);
    assert_eq!(e.simulation.abilities.state.emitted, 7000);
    assert_eq!(e.simulation.abilities.state.recent_noises.len(), 128);
    let inspection = run(&mut e, Command::InspectNoises);
    assert_eq!(inspection["ability_noises"].as_array().unwrap().len(), 7000);
    assert_eq!(
        inspection["recent_ability_noises"]
            .as_array()
            .unwrap()
            .len(),
        128
    );
    assert_eq!(inspection["ability_limits"]["per_source_per_tick"], 16);
    assert_eq!(inspection["ability_limits"]["per_tick"], 16000);

    for hp in &e.simulation.vitality.state.actors {
        assert_eq!(hp.unwrap().units, 999999);
    }
    assert!(
        e.simulation
            .abilities
            .state
            .balances
            .iter()
            .all(|balances| balances[0] == 95)
    );
    for (n, noise) in e.simulation.abilities.state.noises.iter().enumerate() {
        assert_eq!(
            noise.id,
            ignis::noise::event_id(ignis::noise::Domain::Ability, n as u64)
        );
        assert_eq!(noise.tick, 1);
        let entity = e.simulation.actors.entity((n / 7) as u16) as u16;
        assert_eq!(noise.emitter_index, Some(entity));
        assert_eq!(
            noise.emitter_instance,
            e.simulation.actors.entity_reference(entity as usize)
        );
    }
    let state =
        serde_json::from_value(serde_json::to_value(&e.simulation.abilities.state).unwrap())
            .unwrap();
    e.simulation.abilities.state = state;
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut prefix = Engine::replay_at(&e.replay(), Some(0)).unwrap();
    prefix.advance(1).unwrap();
    assert_eq!(
        prefix.simulation.abilities.state,
        e.simulation.abilities.state
    );
    for a in 0..1000 {
        cast(&mut e, a);
    }
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.noises.len(), 7000);
    assert!(
        e.simulation
            .abilities
            .state
            .noises
            .iter()
            .all(|n| n.tick == 2)
    );
    assert_eq!(e.simulation.abilities.state.emitted, 14000);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    e.reset();
    assert_eq!(e.simulation.abilities.state.emitted, 0);
    assert!(e.simulation.abilities.state.noises.is_empty());
    e.simulation.abilities.state.emitted = ignis::noise::MAX_SERIAL - 3;
    let balances = e.simulation.abilities.state.balances[999];
    let hp = e.simulation.vitality.state.actors[998];
    cast(&mut e, 999);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::NoiseBudget
    );
    assert_eq!(e.simulation.abilities.state.balances[999], balances);
    assert_eq!(e.simulation.vitality.state.actors[998], hp);
    assert_eq!(
        e.simulation.abilities.state.emitted,
        ignis::noise::MAX_SERIAL - 3
    );
    assert!(e.simulation.abilities.state.noises.is_empty());
}
