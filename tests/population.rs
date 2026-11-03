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
    s.ability_catalog=serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"attributes":{"buffs":[{"id":"power","duration_ticks":12,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_dealt.multiplier","operation":{"kind":"multiply","basis_points":20000}}]}]},"abilities":[
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
        .position(|&i| e.scene.entities[i as usize].id == id)
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

#[test]
fn queued_until_tick_duplicates_rejected_selection_cleared_authoring_preserved() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(2).unwrap();
    run(
        &mut e,
        Command::SelectEntity {
            id: Some("target".into()),
        },
    );
    let authored = e.scene.clone();
    let original = token(&mut e, "target");
    let positions = e.simulation.positions();
    assert_eq!(remove(&mut e, &original)["phase"], "pending");
    remove(&mut e, "target");
    assert_eq!(e.simulation.positions(), positions);
    assert_eq!(
        run(&mut e, Command::InspectPopulation)["pending"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.population.receipts[0].outcome,
        Outcome::Despawned
    );
    assert_eq!(
        e.simulation.population.receipts[1].outcome,
        Outcome::InstanceUnavailable
    );
    assert_eq!(e.scene, authored);
    assert!(e.selected_id().is_none());
    assert!(e.simulation.position("target").is_none());
    assert!(!e.simulation.npcs.contains_key("target"));
    assert!(!e.simulation.navigation.contains_key("target"));
    assert!(
        e.simulation
            .contacts()
            .iter()
            .all(|(a, b)| a != "target" && b != "target")
    );
    assert!(
        e.execute(
            Command::DespawnActor {
                actor: original.clone()
            }
            .into()
        )
        .is_err()
    );
    e.reset();
    assert!(e.simulation.position("target").is_some());
    assert_eq!(e.simulation.population.processed, 0);
    assert!(e.selected_id().is_none());
    assert!(
        e.execute(Command::DespawnActor { actor: original }.into())
            .is_err()
    );
}
#[test]
fn invalid_admission_does_not_change_world_or_queue() {
    let mut e = Engine::new(scene()).unwrap();
    let mut other = Engine::new(scene()).unwrap();
    for id in [
        "player".into(),
        "floor".into(),
        "missing".into(),
        token(&mut other, "target"),
    ] {
        let before = e.snapshot();
        assert!(
            e.execute(Command::DespawnActor { actor: id }.into())
                .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    for _ in 0..64 {
        remove(&mut e, "target");
    }
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::DespawnActor {
                actor: "target".into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    e.advance(1).unwrap();
    assert_eq!(e.simulation.population.receipts.len(), 64);
    assert_eq!(
        e.simulation
            .population
            .receipts
            .iter()
            .filter(|r| r.outcome == Outcome::Despawned)
            .count(),
        1
    );
}
#[test]
fn removal_precedes_queued_abilities_and_survivor_cast_continues() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 4, None);
    activate(&mut e, "target", 4, None);
    e.advance(1).unwrap();
    let a = slot(&e, "player");
    let cast = e.simulation.abilities.state.casts[a];
    remove(&mut e, "target");
    activate(&mut e, "target", 1, Some("player"));
    e.advance(1).unwrap();
    assert_eq!(e.simulation.abilities.state.casts[a], cast);
    assert_eq!(hp(&e, "player"), 900);
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == ignis::abilities::Outcome::SourceUnavailable)
    );
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .any(|r| r.outcome == ignis::abilities::Outcome::InstanceChanged)
    );
    e.advance(10).unwrap();
    assert_eq!(hp(&e, "player"), 910);
    assert_eq!(e.simulation.abilities.state.balances[a][0], 993);
    assert!(e.simulation.abilities.state.ready_at[a][3] > e.simulation.tick);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn target_removal_rejects_pending_cast_without_charge_or_new_death() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "player", 1, Some("target"));
    remove(&mut e, "target");
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.abilities.state.balances[slot(&e, "player")][0],
        1000
    );
    assert!(e.simulation.abilities.state.projectiles.active.is_empty());
    assert!(e.simulation.vitality.state.died_this_tick.is_empty());
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn paid_effects_survive_with_original_source_and_buff_deadlines() {
    let mut e = Engine::new(scene()).unwrap();
    activate(&mut e, "target", 3, None);
    e.advance(1).unwrap();
    activate(&mut e, "target", 2, Some("player"));
    e.advance(1).unwrap();
    activate(&mut e, "target", 1, Some("player"));
    e.advance(1).unwrap();
    let old = e.simulation.actors.reference(slot(&e, "target") as u16);
    remove(&mut e, "target");
    e.advance(1).unwrap();
    assert_eq!(
        run(&mut e, Command::InspectOrigins)["reserved"][0]["detached"],
        true
    );
    e.advance(45).unwrap();
    assert_eq!(hp(&e, "player"), 840); // 20+20+10 periodic, then 10 projectile after buff expiry.
    assert!(
        e.simulation
            .abilities
            .state
            .receipts
            .iter()
            .filter(|r| r.actor_instance == old)
            .count()
            > 3
    );
    assert!(
        run(&mut e, Command::InspectOrigins)["reserved"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn trigger_exit_is_removal_and_keeps_original_identity() {
    let mut s = scene();
    s.entities.push(serde_json::from_value(json!({"id":"zone","name":"zone","color":[0.1,0.5,0.5],"transform":{"position":[2,1,0],"size":[4,4,4]},"trigger":{"enabled":true,"targets":["target"]}})).unwrap());
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(e.simulation.triggers.occupied("zone", "target"), Some(true));
    let r = e.simulation.actors.reference(slot(&e, "target") as u16);
    remove(&mut e, "target");
    e.advance(1).unwrap();
    let event = e.simulation.triggers.state.events.back().unwrap();
    assert_eq!(event.kind, ignis::trigger::EventKind::Exit);
    assert_eq!(event.reason, ignis::trigger::EventReason::Removed);
    assert_eq!(event.target_instance, r);
    assert_eq!(
        e.simulation.triggers.occupied("zone", "target"),
        Some(false)
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn replay_pending_full_prefix_and_tampering() {
    let mut e = Engine::new(scene()).unwrap();
    e.advance(2).unwrap();
    remove(&mut e, "target");
    remove(&mut e, "target");
    let pending = e.replay();
    assert_eq!(Engine::verify_replay(&pending).unwrap()["matched"], true);
    let mut prefix = Engine::replay_at(&pending, Some(2)).unwrap();
    assert_eq!(
        run(&mut prefix, Command::InspectPopulation),
        run(&mut e, Command::InspectPopulation)
    );
    e.advance(3).unwrap();
    let replay = e.replay();
    assert_eq!(
        Engine::verify_replay(&replay).unwrap()["population_matched"],
        true
    );
    let mut prefix = Engine::replay_at(&replay, Some(2)).unwrap();
    prefix.advance(3).unwrap();
    assert_eq!(prefix.simulation.population, e.simulation.population);
    assert_eq!(prefix.simulation.positions(), e.simulation.positions());
    let mut bad = replay.clone();
    if let ignis::population::Operation::Despawn { ref mut actor, .. } =
        bad.population_inputs[0].operation
    {
        *actor = u16::MAX;
    }
    assert!(Engine::verify_replay(&bad).is_err());
    let mut bad = replay.clone();
    if let ignis::population::Operation::Despawn {
        ref mut instance, ..
    } = bad.population_inputs[0].operation
    {
        instance.generation += 1;
    }
    assert!(Engine::verify_replay(&bad).is_err());
    let mut bad = replay.clone();
    bad.population_inputs[1].tick = 3;
    assert!(Engine::verify_replay(&bad).is_err());
    let mut bad = replay.clone();
    bad.expected_population.receipts[0].outcome = Outcome::InstanceUnavailable;
    assert_eq!(Engine::verify_replay(&bad).unwrap()["matched"], false);
}
#[test]
fn every_example_can_retire_its_npcs_without_ghost_state_or_panics() {
    for file in std::fs::read_dir("examples").unwrap() {
        let path = file.unwrap().path();
        if !path.to_string_lossy().ends_with(".scene.json") {
            continue;
        }
        let s: Scene = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let ids: Vec<_> = s
            .entities
            .iter()
            .filter(|e| e.body == Some(ignis::model::BodyKind::Npc))
            .map(|e| e.id.clone())
            .collect();
        if ids.is_empty() {
            continue;
        }
        let mut e = Engine::new(s).unwrap();
        e.advance(3).unwrap();
        // Large examples still obey the real per-tick admission contract.
        for batch in ids.chunks(ignis::population::PER_TICK) {
            for id in batch {
                remove(&mut e, id);
            }
            e.advance(1).unwrap();
        }
        e.advance(4).unwrap();
        e.debug_view.npc_vision = true;
        e.debug_view.npc_hearing = true;
        e.debug_view.npc_routes = true;
        e.render_frame();
        e.snapshot();
        assert!(e.simulation.npcs.is_empty(), "{}", path.display());
        assert_eq!(
            Engine::verify_replay(&e.replay()).unwrap()["matched"],
            true,
            "{}",
            path.display()
        );
    }
}

#[test]
fn removing_observed_target_preserves_frozen_memory_without_omniscience() {
    let mut s = scene();
    let mut watcher = s
        .entities
        .iter()
        .find(|e| e.id == "target")
        .unwrap()
        .clone();
    watcher.id = "watcher".into();
    watcher.name = "watcher".into();
    watcher.transform.position = [2., 0.8, 4.];
    let n = watcher.npc.as_mut().unwrap();
    n.enabled = true;
    n.target = Some("target".into());
    n.awareness = Some(ignis::model::Awareness::Perceptive {
        vision_range: 12.,
        fov_degrees: 120.,
        initial_facing_degrees: 0.,
        memory_ticks: Some(300),
        search_ticks: Some(180),
        search_turn_degrees_per_second: 90.,
        max_investigation_distance: None,
        area_search: None,
        hearing: None,
    });
    s.entities.push(watcher);
    let mut e = Engine::new(s).unwrap();
    e.advance(3).unwrap();
    let memory = e.simulation.npcs["watcher"]
        .memory
        .expect("visually observed target");
    remove(&mut e, "target");
    e.advance(1).unwrap();
    assert_eq!(e.simulation.npcs["watcher"].memory, Some(memory));
    assert!(!e.simulation.npcs["watcher"].observed_this_tick);
    assert_ne!(
        e.simulation.npcs["watcher"].phase,
        ignis::perception::Phase::Dead
    );
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
#[test]
fn corpse_retirement_is_not_another_death() {
    let mut s = scene();
    s.entities
        .iter_mut()
        .find(|e| e.id == "target")
        .unwrap()
        .vitality
        .as_mut()
        .unwrap()
        .initial = 10;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", 1, Some("target"));
    e.advance(40).unwrap();
    let deaths = e.simulation.vitality.state.deaths.clone();
    assert_eq!(deaths.len(), 1);
    remove(&mut e, "target");
    e.advance(1).unwrap();
    assert!(
        run(&mut e, Command::InspectVitality)["actors"]
            .get("target")
            .is_none()
    );
    assert_eq!(e.simulation.vitality.state.deaths, deaths);
    assert!(e.simulation.vitality.state.died_this_tick.is_empty());
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}

#[test]
fn retiring_passage_owner_releases_waiting_survivor() {
    let s: Scene =
        serde_json::from_str(include_str!("../examples/npc-passages.scene.json")).unwrap();
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    let owner = e
        .simulation
        .passages
        .slots
        .iter()
        .find_map(|s| s.owner)
        .expect("active passage owner");
    let id = e.scene.entities[owner as usize].id.clone();
    remove(&mut e, &id);
    e.advance(1).unwrap();
    assert!(!e.simulation.passages.agents.contains_key(&id));
    assert!(
        e.simulation
            .passages
            .slots
            .iter()
            .all(|s| s.owner != Some(owner))
    );
    e.advance(300).unwrap();
    assert!(e.simulation.passages.agents.values().any(|a| a.grants > 0));
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
