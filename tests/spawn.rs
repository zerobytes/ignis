use ignis::{
    engine::{Command, Engine},
    model::{RoomRecipe, Scene},
    population::Outcome,
};
use serde_json::{Value, json};

fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        width: 24.,
        depth: 20.,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [-4., 0.8, 0.];
    p.footsteps = None;
    s.entities.push(serde_json::from_value(json!({"id":"target","name":"target","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[2,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"target":"player","awareness":{"mode":"omniscient"},"surface":"floor","speed":3,"stop_distance":0.5,"enabled":true}})).unwrap());
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"attributes":{"buffs":[{"id":"power","duration_ticks":120,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_dealt.multiplier","operation":{"kind":"multiply","basis_points":20000}}]}]},"abilities":[
        {"id":"bolt","selection":"actor","range":20,"delivery":{"kind":"projectile","speed":12,"half_extent":0.1,"max_distance":20},"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage","recipient":"target","amount":10}]},
        {"id":"dot","selection":"actor","range":20,"periodic":{"interval_ticks":5,"pulses":3,"reapply":"independent"},"cast_ticks":0,"cooldown_ticks":0,"costs":[{"resource":"energy","units":5}],"effects":[{"kind":"damage","recipient":"target","amount":10}]},
        {"id":"power","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"apply_buff","recipient":"target","buff":"power"}]},
        {"id":"slow","selection":"self_actor","range":0,"cast_ticks":10,"cooldown_ticks":30,"costs":[{"resource":"energy","units":7}],"effects":[{"kind":"heal","recipient":"target","amount":10}]}
    ]})).unwrap();
    for id in ["player", "target"] {
        let e = s.entities.iter_mut().find(|e| e.id == id).unwrap();
        e.abilities=Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":1000,"capacity":1000}],"grants":["bolt","dot","power","slow"]})).unwrap());
        e.vitality =
            Some(serde_json::from_value(json!({"initial":900,"maximum":1000,"scale":1})).unwrap());
    }
    let mut template = s
        .entities
        .iter()
        .find(|e| e.id == "target")
        .unwrap()
        .clone();
    template.id = "guard".into();
    template.npc.as_mut().unwrap().enabled = true;
    template.npc.as_mut().unwrap().target = None;
    let mut healer = template.clone();
    healer.id = "healer".into();
    healer.color = [0.2, 0.8, 0.4];
    healer.vitality.as_mut().unwrap().initial = 500;
    healer.abilities.as_mut().unwrap().resources[0].initial = 300;
    healer.abilities.as_mut().unwrap().grants = vec!["slow".into()];
    s.npc_templates = vec![template, healer];
    s.entities
        .iter_mut()
        .find(|e| e.id == "target")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .enabled = false;
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn remove(e: &mut Engine, id: &str) -> Value {
    run(e, Command::DespawnActor { actor: id.into() })
}
fn activate(e: &mut Engine, id: &str, slot: u8, target: Option<&str>) {
    run(
        e,
        Command::ActivateAbility {
            actor: id.into(),
            slot,
            target: target.map(Into::into),
        },
    );
}
fn slot(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.simulation.definitions().entities[i as usize].id == id)
        .unwrap()
}
fn hp(e: &Engine, id: &str) -> i64 {
    e.simulation.vitality.state.actors[slot(e, id)]
        .unwrap()
        .units
}
fn token(e: &mut Engine, id: &str) -> String {
    run(e, Command::InspectInstances)["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["authored_id"] == id)
        .unwrap()["reference"]
        .as_str()
        .unwrap()
        .into()
}

fn queue(e: &mut Engine, t: &str, p: [f32; 3]) {
    run(
        e,
        Command::SpawnActor {
            template: t.into(),
            position: p,
            target: None,
        },
    );
}
fn latest(e: &Engine) -> String {
    let r = e.simulation.population.receipts.back().unwrap();
    assert_eq!(r.outcome, Outcome::Spawned);
    e.simulation
        .actors
        .label(e.simulation.definitions(), r.instance.unwrap())
        .unwrap()
        .into()
}
fn birth(e: &mut Engine, t: &str, p: [f32; 3]) -> String {
    queue(e, t, p);
    e.advance(1).unwrap();
    latest(e)
}
fn replays(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}

#[test]
fn templates_are_data_not_individuals_and_birth_preserves_authoring_and_survivors() {
    let mut e = Engine::new(scene()).unwrap();
    let scene = e.scene.clone();
    let initial = e.simulation.positions();
    let survivor = token(&mut e, "player");
    assert!(!initial.contains_key("guard"));
    assert_eq!(e.simulation.npcs.len(), 1);
    queue(&mut e, "guard", [0., 0.8, 4.]);
    assert_eq!(e.simulation.positions(), initial);
    replays(&e);
    e.advance(1).unwrap();
    let id = latest(&e);
    let a = slot(&e, &id);
    assert_eq!(e.scene, scene);
    assert_eq!(token(&mut e, "player"), survivor);
    assert_eq!(hp(&e, &id), 900);
    assert_eq!(e.simulation.abilities.state.balances[a][0], 1000);
    assert_eq!(e.simulation.vitality.state.lives[a], 0);
    assert_eq!(e.simulation.npcs.len(), 2);
    assert!(
        e.simulation
            .definitions()
            .entities
            .iter()
            .find(|x| x.id == id)
            .unwrap()
            .npc
            .as_ref()
            .unwrap()
            .enabled
    );
    for c in [
        Command::Inspect,
        Command::InspectTargeting,
        Command::InspectKnowledge,
        Command::InspectAbilities,
        Command::InspectRelations,
        Command::InspectTerritory,
        Command::InspectObjectives,
        Command::InspectHearing,
        Command::InspectMobility,
        Command::InspectPassages,
    ] {
        run(&mut e, c);
    }
    run(
        &mut e,
        Command::SelectEntity {
            id: Some(id.clone()),
        },
    );
    assert_eq!(e.selected_id(), Some(id.as_str()));
    replays(&e);
}

#[test]
fn reuse_has_new_identity_fresh_health_resources_cooldowns_casts_and_buffs() {
    let mut e = Engine::new(scene()).unwrap();
    let first = birth(&mut e, "guard", [0., 0.8, 4.]);
    let a = slot(&e, &first);
    let old = e.simulation.actors.reference(a as u16);
    let old_token = token(&mut e, &first);
    activate(&mut e, &first, 3, None);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.buffs.active.len(), 1);
    activate(&mut e, &first, 4, None);
    e.advance(11).unwrap();
    assert!(e.simulation.abilities.state.ready_at[a][3] > e.simulation.tick);
    assert_eq!(e.simulation.abilities.state.balances[a][0], 993);
    activate(&mut e, &first, 1, Some("player"));
    e.advance(1).unwrap();
    remove(&mut e, &first);
    queue(&mut e, "healer", [0., 0.8, 4.]);
    e.advance(1).unwrap();
    let next = latest(&e);
    assert_eq!(slot(&e, &next), a);
    let current = e.simulation.actors.reference(a as u16);
    assert_ne!(current, old);
    assert_ne!(current.instance, old.instance);
    assert_ne!(current.generation, old.generation);
    assert_eq!(hp(&e, &next), 500);
    assert_eq!(e.simulation.abilities.state.balances[a][0], 300);
    assert_eq!(
        e.simulation.abilities.state.ready_at[a],
        [0; ignis::abilities::GRANTS]
    );
    assert!(e.simulation.abilities.state.casts[a].is_none());
    assert_eq!(e.simulation.vitality.state.lives[a], 0);
    assert!(e.simulation.abilities.state.buffs.active.is_empty());
    assert!(e.simulation.npcs[&next].memory.is_none());
    assert!(
        e.execute(
            Command::ActivateAbility {
                actor: old_token,
                slot: 1,
                target: None
            }
            .into()
        )
        .is_err()
    );
    assert!(
        e.execute(
            Command::ActivateAbility {
                actor: next.clone(),
                slot: 2,
                target: None
            }
            .into()
        )
        .is_err()
    );
    activate(&mut e, &next, 1, None);
    e.advance(12).unwrap();
    assert_eq!(e.simulation.abilities.state.balances[a][0], 293);
    replays(&e);
}

#[test]
fn stale_commands_and_target_bindings_never_retarget_recycled_slots() {
    let mut e = Engine::new(scene()).unwrap();
    let id = birth(&mut e, "guard", [0., 0.8, 4.]);
    let a = slot(&e, &id);
    let old = e.simulation.actors.reference(a as u16);
    activate(&mut e, &id, 4, None);
    activate(&mut e, "player", 1, Some(&id));
    remove(&mut e, &id);
    queue(&mut e, "healer", [0., 0.8, 4.]);
    e.advance(1).unwrap();
    let next = latest(&e);
    assert_eq!(hp(&e, &next), 500);
    assert!(e.simulation.abilities.state.casts[a].is_none());
    assert!(e.simulation.abilities.state.receipts.iter().any(
        |r| r.actor_instance == old && r.outcome == ignis::abilities::Outcome::InstanceChanged
    ));
    let old2 = e.simulation.actors.reference(a as u16);
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [4., 0.8, 4.],
            target: Some(next.clone()),
        },
    );
    remove(&mut e, &next); // birth before removal: binding freezes old individual
    e.advance(1).unwrap();
    let born = e
        .simulation
        .population
        .receipts
        .iter()
        .rev()
        .find(|r| r.outcome == Outcome::Spawned)
        .unwrap()
        .instance
        .unwrap();
    assert_ne!(born, old2);
    e.advance(1).unwrap();
    replays(&e);
}

#[test]
fn failed_positions_and_missing_target_are_atomic_without_identity_consumption() {
    let mut e = Engine::new(scene()).unwrap();
    let identities = e.simulation.actors.instance_snapshot();
    for p in [
        [2., 0.8, 0.],
        [0., -0.5, 4.],
        [100., 0.8, 100.],
        [0., 5., 4.],
    ] {
        queue(&mut e, "guard", p);
    }
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::SpawnActor {
                template: "missing".into(),
                position: [0., 0.8, 4.],
                target: None
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    assert!(
        e.execute(
            Command::SpawnActor {
                template: "guard".into(),
                position: [f32::NAN, 0., 0.],
                target: None
            }
            .into()
        )
        .is_err()
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.actors.instance_snapshot(), identities);
    assert!(
        e.simulation
            .population
            .receipts
            .iter()
            .all(|r| matches!(r.outcome, Outcome::Occupied | Outcome::UnsupportedPosition))
    );
    run(
        &mut e,
        Command::DespawnActor {
            actor: "target".into(),
        },
    );
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [0., 0.8, 4.],
            target: Some("target".into()),
        },
    );
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.population.receipts.back().unwrap().outcome,
        Outcome::InstanceUnavailable
    );
    replays(&e);
}

#[test]
fn full_pool_respects_queue_order_and_same_template_instances_are_independent() {
    let mut e = Engine::new(scene()).unwrap();
    let mut ids = vec![];
    for i in 0..7 {
        ids.push(birth(&mut e, "guard", [-6. + i as f32 * 2., 0.8, 5.]));
    }
    assert_eq!(e.simulation.npcs.len(), 8);
    let old = e.simulation.actors.instance_snapshot();
    queue(&mut e, "healer", [0., 0.8, -5.]);
    remove(&mut e, &ids[0]);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation
            .population
            .receipts
            .iter()
            .rev()
            .nth(1)
            .unwrap()
            .outcome,
        Outcome::Capacity
    );
    assert_ne!(e.simulation.actors.instance_snapshot(), old);
    let new = birth(&mut e, "healer", [0., 0.8, -5.]);
    assert_eq!(hp(&e, &new), 500);
    assert_eq!(hp(&e, &ids[1]), 900);
    activate(&mut e, "player", 1, Some(&ids[2]));
    e.advance(60).unwrap();
    assert!(hp(&e, &ids[2]) < 900);
    assert_eq!(hp(&e, &ids[3]), 900);
    remove(&mut e, &new);
    queue(&mut e, "guard", [0., 0.8, -5.]);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.population.receipts.back().unwrap().outcome,
        Outcome::Spawned
    );
    replays(&e);
}

#[test]
fn old_paid_effects_retain_origin_after_a_different_template_reuses_storage() {
    let mut s = scene();
    let n = s.npc_templates[0].npc.as_mut().unwrap();
    n.target = Some("player".into());
    n.stop_distance = 5.;
    let mut e = Engine::new(s).unwrap();
    let id = birth(&mut e, "guard", [0., 0.8, 2.]);
    let a = slot(&e, &id);
    let old = e.simulation.actors.reference(a as u16);
    activate(&mut e, &id, 3, None);
    e.advance(1).unwrap();
    activate(&mut e, &id, 2, Some("player"));
    e.advance(1).unwrap();
    activate(&mut e, &id, 1, Some("player"));
    e.advance(1).unwrap();
    remove(&mut e, &id);
    queue(&mut e, "healer", [0., 0.8, -4.]);
    e.advance(1).unwrap();
    let new = latest(&e);
    e.advance(80).unwrap();
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.actor_instance == old && r.outcome == ignis::abilities::Outcome::Impacted)
    );
    assert_eq!(hp(&e, &new), 500);
    assert_eq!(hp(&e, "player"), 820);
    assert_eq!(e.simulation.abilities.state.balances[a][0], 300);
    assert_eq!(run(&mut e, Command::InspectOrigins)["reserved"], json!([]));
    replays(&e);
}

#[test]
fn repeated_reuse_keeps_live_maps_bounded_and_old_tokens_invalid() {
    let mut e = Engine::new(scene()).unwrap();
    let mut previous: Option<String> = None;
    for i in 0..300 {
        if let Some(id) = previous.take() {
            remove(&mut e, &id);
        }
        let id = birth(
            &mut e,
            if i % 2 == 0 { "guard" } else { "healer" },
            [0., 0.8, 4.],
        );
        assert_eq!(e.simulation.npcs.len(), 2);
        assert_eq!(e.simulation.navigation.len(), 2);
        assert!(e.simulation.mobility.states.len() <= 2);
        assert!(e.simulation.mobility.compiled.len() <= 2);
        assert!(e.simulation.patrols.len() <= 2);
        previous = Some(id);
    }
    assert_eq!(e.simulation.population.receipts.len(), 256);
    assert!(e.simulation.population.dropped_receipts > 0);
    replays(&e);
}

#[test]
fn template_edits_reset_and_undo_atomically_restore_authoring_not_spawned_state() {
    let mut e = Engine::new(scene()).unwrap();
    let id = birth(&mut e, "guard", [0., 0.8, 4.]);
    let old = token(&mut e, &id);
    let before = e.snapshot();
    let mut bad = e.scene.npc_templates.clone();
    bad[0].npc.as_mut().unwrap().speed = -1.;
    assert!(
        e.execute(Command::SetNpcTemplates { templates: bad }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut templates = e.scene.npc_templates.clone();
    templates[0].vitality.as_mut().unwrap().initial = 321;
    run(&mut e, Command::SetNpcTemplates { templates });
    assert_eq!(e.simulation.tick, 0);
    assert!(e.simulation.position(&id).is_none());
    assert!(
        e.execute(Command::DespawnActor { actor: old }.into())
            .is_err()
    );
    let next = birth(&mut e, "guard", [0., 0.8, 4.]);
    assert_eq!(hp(&e, &next), 321);
    run(&mut e, Command::Undo);
    let next = birth(&mut e, "guard", [0., 0.8, 4.]);
    assert_eq!(hp(&e, &next), 900);
    replays(&e);
}

#[test]
fn resurrection_life_counter_does_not_survive_a_new_birth() {
    let mut s = scene();
    for (id, kind, amount) in [("kill", "damage", 1000), ("revive", "resurrect", 100)] {
        s.ability_catalog.abilities.push(serde_json::from_value(json!({"id":id,"selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":kind,"recipient":"target","amount":amount}]})).unwrap());
        s.entities
            .iter_mut()
            .find(|e| e.id == "player")
            .unwrap()
            .abilities
            .as_mut()
            .unwrap()
            .grants
            .push(id.into());
    }
    let mut e = Engine::new(s).unwrap();
    let id = birth(&mut e, "guard", [0., 0.8, 4.]);
    let a = slot(&e, &id);
    activate(&mut e, "player", 5, Some(&id));
    e.advance(1).unwrap();
    assert_eq!(hp(&e, &id), 0);
    activate(&mut e, "player", 6, Some(&id));
    e.advance(1).unwrap();
    assert_eq!(e.simulation.vitality.state.lives[a], 1);
    remove(&mut e, &id);
    queue(&mut e, "guard", [0., 0.8, 4.]);
    e.advance(1).unwrap();
    let new = latest(&e);
    assert_eq!(slot(&e, &new), a);
    assert_eq!(hp(&e, &new), 900);
    assert_eq!(e.simulation.vitality.state.lives[a], 0);
    replays(&e);
}

#[test]
fn compiled_template_bases_replace_previous_base_without_changing_detached_origin() {
    let mut s = scene();
    s.ability_catalog.attributes.bases=serde_json::from_value(json!([
        {"entity":"guard","attribute":"combat.damage_received.multiplier","value":{"kind":"number","units":5000}},
        {"entity":"healer","attribute":"combat.damage_received.multiplier","value":{"kind":"number","units":20000}}
    ])).unwrap();
    let mut e = Engine::new(s).unwrap();
    let id = birth(&mut e, "guard", [0., 0.8, 4.]);
    let inspect = |e: &mut Engine, id: String| {
        run(
            e,
            Command::InspectAttribute {
                entity: id,
                attribute: "combat.damage_received.multiplier".into(),
                ability: None,
                tags: vec![],
            },
        )
    };
    let base1 = inspect(&mut e, id.clone());
    assert_eq!(base1["effective"]["units"], 5000);
    remove(&mut e, &id);
    queue(&mut e, "healer", [0., 0.8, 4.]);
    e.advance(1).unwrap();
    let new = latest(&e);
    let base2 = inspect(&mut e, new);
    assert_eq!(base2["effective"]["units"], 20000);
    replays(&e);
}

#[test]
fn airborne_birth_requires_capability_and_flight_state_is_new() {
    let mut s = scene();
    let n = s.npc_templates[0].npc.as_mut().unwrap();
    n.mobility = Some(serde_json::from_value(json!({"walk":false,"fly_speed":3})).unwrap());
    n.flight = Some(serde_json::from_value(json!({"mode":"straight"})).unwrap());
    n.local_connections =
        Some(serde_json::from_value(json!({"walk":false,"fly":true,"max_distance":8})).unwrap());
    let mut e = Engine::new(s).unwrap();
    let id = birth(&mut e, "guard", [0., 5., 4.]);
    assert!(e.simulation.mobility.states.contains_key(&id));
    e.advance(10).unwrap();
    let p = e.simulation.position(&id).unwrap();
    assert!((p[1] - 5.).abs() < 0.1);
    remove(&mut e, &id);
    queue(&mut e, "healer", [0., 0.8, 4.]);
    e.advance(1).unwrap();
    let new = latest(&e);
    assert!(!e.simulation.mobility.states.contains_key(&id));
    assert!(!e.simulation.mobility.states.contains_key(&new));
    assert!(!e.simulation.mobility.compiled.contains_key(&id));
    replays(&e);
}

#[test]
fn recycled_low_slot_does_not_execute_before_older_individual() {
    let mut s = scene();
    s.entities.iter_mut().find(|e| e.id == "target").unwrap().id = "aaa".into();
    let mut e = Engine::new(s).unwrap();
    remove(&mut e, "aaa");
    let born = birth(&mut e, "guard", [0., 0.8, 4.]);
    assert!(slot(&e, &born) < slot(&e, "player"));
    activate(&mut e, &born, 3, None);
    activate(&mut e, "player", 3, None);
    e.advance(1).unwrap();
    let last: Vec<_> = e
        .simulation
        .abilities
        .state
        .receipts
        .iter()
        .filter(|r| r.outcome == ignis::abilities::Outcome::Completed)
        .rev()
        .take(2)
        .collect();
    assert!(last[0].actor_instance.instance > last[1].actor_instance.instance);
    replays(&e);
}

#[test]
fn old_periodic_target_is_not_replaced_and_surviving_cast_keeps_deadline() {
    let mut e = Engine::new(scene()).unwrap();
    let id = birth(&mut e, "guard", [0., 0.8, 4.]);
    activate(&mut e, "player", 2, Some(&id));
    e.advance(1).unwrap();
    activate(&mut e, "player", 4, None);
    e.advance(1).unwrap();
    let p = slot(&e, "player");
    let cast = e.simulation.abilities.state.casts[p].unwrap();
    remove(&mut e, &id);
    queue(&mut e, "guard", [0., 0.8, 4.]);
    e.advance(1).unwrap();
    let fresh = latest(&e);
    assert_eq!(e.simulation.abilities.state.casts[p].unwrap(), cast);
    e.advance(20).unwrap();
    assert_eq!(hp(&e, &fresh), 900);
    assert_eq!(e.simulation.abilities.state.balances[p][0], 988);
    replays(&e);
}

#[test]
fn patrol_footsteps_and_navigation_start_fresh_after_reuse() {
    let mut s = scene();
    let t = &mut s.npc_templates[0];
    t.footsteps = Some(
        serde_json::from_value(json!({"surface":"floor","stride":0.4,"radius":5,"min_speed":0.1}))
            .unwrap(),
    );
    t.npc.as_mut().unwrap().patrol=Some(serde_json::from_value(json!({"route":{"kind":"points","points":[[4,4],[0,4]]},"traversal":"loop","wait_ticks":10,"travel_ticks":900,"retry_ticks":60,"arrival_distance":0.25})).unwrap());
    let mut e = Engine::new(s).unwrap();
    let id = birth(&mut e, "guard", [0., 0.8, 4.]);
    e.advance(70).unwrap();
    assert!(e.simulation.footsteps.states[&id].emitted > 0);
    remove(&mut e, &id);
    let new = birth(&mut e, "guard", [0., 0.8, 4.]);
    assert!(!e.simulation.footsteps.states.contains_key(&id));
    assert_eq!(e.simulation.footsteps.states[&new].emitted, 0);
    assert!(!e.simulation.patrols.contains_key(&id));
    assert!(!e.simulation.patrol_routes.contains_key(&id));
    assert_eq!(e.simulation.patrols.len(), 1);
    e.advance(70).unwrap();
    assert!(e.simulation.footsteps.states[&new].emitted > 0);
    replays(&e);
}

#[test]
fn passage_capacity_refusal_is_atomic_then_released_slot_accepts_birth() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-passages.scene.json")).unwrap();
    s.normalize();
    let mut t = s.entities.iter().find(|e| e.id == "alpha").unwrap().clone();
    t.id = "walker".into();
    t.npc.as_mut().unwrap().enabled = false;
    s.npc_templates = vec![t];
    let mut e = Engine::new(s).unwrap();
    let prior = e.simulation.actors.instance_snapshot();
    queue(&mut e, "walker", [-6., 0.8, 3.]);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.population.receipts.back().unwrap().outcome,
        Outcome::PassageCapacity
    );
    assert_eq!(e.simulation.actors.instance_snapshot(), prior);
    remove(&mut e, "alpha");
    let id = birth(&mut e, "walker", [-6., 0.8, 3.]);
    assert!(!e.simulation.passages.agents.contains_key("alpha"));
    assert!(e.simulation.passages.agents.contains_key(&id));
    replays(&e);
}

#[test]
fn queued_cancel_for_retired_source_gets_one_receipt_and_cannot_cancel_newborn() {
    let mut e = Engine::new(scene()).unwrap();
    let id = birth(&mut e, "guard", [0., 0.8, 4.]);
    let a = slot(&e, &id);
    activate(&mut e, &id, 4, None);
    e.advance(1).unwrap();
    let execution = e.simulation.abilities.state.casts[a].unwrap().id;
    let queued = run(
        &mut e,
        Command::CancelAbility {
            actor: id.clone(),
            execution_id: execution,
        },
    );
    remove(&mut e, &id);
    queue(&mut e, "healer", [0., 0.8, 4.]);
    e.advance(1).unwrap();
    let request = queued["request_id"].as_u64().unwrap();
    let matching: Vec<_> = e
        .simulation
        .abilities
        .state
        .receipts
        .iter()
        .filter(|r| r.request == Some(request))
        .collect();
    assert_eq!(matching.len(), 1);
    assert_eq!(
        matching[0].outcome,
        ignis::abilities::Outcome::InstanceChanged
    );
    replays(&e);
}

#[test]
fn born_npc_acquires_its_own_evidence_and_replacement_does_not_inherit_it() {
    let mut s = scene();
    let n = s.npc_templates[0].npc.as_mut().unwrap();
    n.awareness=Some(serde_json::from_value(json!({"mode":"perceptive","vision_range":12,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":300,"search_ticks":60,"search_turn_degrees_per_second":90})).unwrap());
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [-1., 0.8, 0.],
            target: Some("player".into()),
        },
    );
    e.advance(1).unwrap();
    let first = latest(&e);
    assert!(e.simulation.npcs[&first].observed_this_tick);
    assert!(e.simulation.npcs[&first].memory.is_some());
    e.advance(60).unwrap();
    assert!(e.simulation.position(&first).unwrap()[0] < -2.);
    remove(&mut e, &first);
    let next = birth(&mut e, "guard", [0., 0.8, 4.]);
    assert!(e.simulation.npcs[&next].memory.is_none());
    assert!(!e.simulation.npcs[&next].observed_this_tick);
    replays(&e);
}

#[test]
fn malformed_templates_are_rejected_without_reset_or_partial_edit() {
    let mut e = Engine::new(scene()).unwrap();
    birth(&mut e, "guard", [0., 0.8, 4.]);
    let original = e.scene.npc_templates.clone();
    let mut cases = vec![];
    let mut t = original[0].clone();
    t.id = "player".into();
    cases.push(vec![t]);
    let mut t = original[0].clone();
    t.id = "_ignis_bad".into();
    cases.push(vec![t]);
    let mut t = original[0].clone();
    t.body = Some(ignis::model::BodyKind::Fixed);
    cases.push(vec![t]);
    let mut t = original[0].clone();
    t.npc.as_mut().unwrap().target = Some("missing".into());
    cases.push(vec![t]);
    let mut t = original[0].clone();
    t.abilities.as_mut().unwrap().grants = vec!["missing".into()];
    cases.push(vec![t]);
    cases.push(
        (0..17)
            .map(|i| {
                let mut t = original[0].clone();
                t.id = format!("template{i}");
                t
            })
            .collect(),
    );
    for templates in cases {
        let before = e.snapshot();
        assert!(
            e.execute(Command::SetNpcTemplates { templates }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    replays(&e);
}
