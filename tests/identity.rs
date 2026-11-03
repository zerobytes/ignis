use ignis::{
    engine::{Command, Engine, Replay},
    model::Scene,
};
use serde_json::{Value, json};

fn scene() -> Scene {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/npc-resurrection.scene.json")).unwrap();
    for e in &mut s.entities {
        if let Some(n) = &mut e.npc {
            n.enabled = false;
        }
    }
    s
}
fn inspect(e: &mut Engine) -> Value {
    e.execute(Command::InspectInstances.into()).unwrap()
}
fn token(v: &Value, id: &str) -> String {
    v["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["authored_id"] == id)
        .unwrap()["reference"]
        .as_str()
        .unwrap()
        .into()
}
fn activate(e: &mut Engine, a: &str, t: &str) -> ignis::model::Result<Value> {
    e.execute(
        Command::ActivateAbility {
            actor: a.into(),
            slot: 1,
            target: Some(t.into()),
        }
        .into(),
    )
}
#[test]
fn scoped_tokens_resolve_only_in_their_execution_and_invalid_commands_are_atomic() {
    let mut e = Engine::new(scene()).unwrap();
    let mut other = Engine::new(scene()).unwrap();
    let ids = inspect(&mut e);
    let foreign = token(&inspect(&mut other), "player");
    let p = token(&ids, "player");
    let v = token(&ids, "patient");
    let fixed = ids["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["actor"] == false)
        .unwrap()["reference"]
        .as_str()
        .unwrap();
    for bad in [
        foreign.as_str(),
        fixed,
        "ignis:invalid",
        "ignis:",
        "missing",
    ] {
        let before = e.snapshot();
        assert!(activate(&mut e, bad, &v).is_err());
        assert!(activate(&mut e, &p, bad).is_err());
        assert_eq!(e.snapshot(), before);
        assert_eq!(inspect(&mut e), ids);
    }
    activate(&mut e, &p, &v).unwrap();
    let pending = e.replay();
    assert_eq!(pending.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(pending.scene.version, ignis::model::SCENE_VERSION);
    assert!(
        Engine::verify_replay(&pending).unwrap()["matched"]
            .as_bool()
            .unwrap()
    );
    e.advance(1).unwrap();
    assert_eq!(e.snapshot()["vitality"]["actors"]["patient"]["units"], 0);
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    e.reset();
    assert!(activate(&mut e, &p, &v).is_err());
    activate(&mut e, "player", "patient").unwrap();
}
#[test]
fn edits_invalidate_tokens_only_after_success_and_undo_does_not_revive_old_tokens() {
    let mut e = Engine::new(scene()).unwrap();
    let first = inspect(&mut e);
    let p = token(&first, "player");
    let mut bad = scene();
    bad.entities[0].transform.size[0] = -1.;
    assert!(e.replace(bad).is_err());
    assert_eq!(inspect(&mut e), first);
    e.replace(scene()).unwrap();
    assert!(activate(&mut e, &p, "patient").is_err());
    let second = token(&inspect(&mut e), "player");
    e.execute(Command::Undo.into()).unwrap();
    assert!(activate(&mut e, &p, "patient").is_err());
    assert!(activate(&mut e, &second, "patient").is_err());
    let third = token(&inspect(&mut e), "player");
    e.execute(Command::Redo.into()).unwrap();
    assert!(activate(&mut e, &third, "patient").is_err());
}
#[test]
fn identities_are_deterministic_but_transport_scope_is_not_replayed() {
    let mut e = Engine::new(scene()).unwrap();
    let mut reordered = scene();
    reordered.entities.reverse();
    let mut b = Engine::new(reordered).unwrap();
    let a_ids = inspect(&mut e);
    let b_ids = inspect(&mut b);
    assert_ne!(a_ids["execution_scope"], b_ids["execution_scope"]);
    for a in a_ids["instances"].as_array().unwrap() {
        let b = b_ids["instances"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["authored_id"] == a["authored_id"])
            .unwrap();
        assert_eq!(a["instance_id"], b["instance_id"]);
        assert_eq!(a["generation"], b["generation"]);
    }
    activate(&mut e, "player", "patient").unwrap();
    let pending = e.snapshot();
    e.advance(3).unwrap();
    let replay = e.replay();
    let mut prefix = Engine::replay_at(&replay, Some(0)).unwrap();
    assert_eq!(prefix.snapshot()["abilities"], pending["abilities"]);
    assert_ne!(
        inspect(&mut prefix)["execution_scope"],
        a_ids["execution_scope"]
    );
    let serialized = serde_json::to_string(&replay).unwrap();
    assert!(!serialized.contains(a_ids["execution_scope"].as_str().unwrap()));
    let mut bad = replay.clone();
    bad.ability_inputs[0].life.source_instance.generation += 1;
    assert!(Engine::replay_at(&bad, Some(0)).is_err());
    let mut bad = replay.clone();
    bad.ability_inputs[0]
        .life
        .target_instance
        .as_mut()
        .unwrap()
        .instance += 1;
    assert!(Engine::verify_replay(&bad).is_err());
    let mut tampered = serde_json::to_value(replay).unwrap();
    tampered["expected_instances"]["next_instance"] = json!(999);
    let tampered: Replay = serde_json::from_value(tampered).unwrap();
    let result = Engine::verify_replay(&tampered).unwrap();
    assert_eq!(result["instances_matched"], false);
    assert_eq!(result["matched"], false);
}

#[test]
fn queued_references_are_rechecked_at_consumption_and_cancel_cannot_cross_instances() {
    use ignis::abilities::{Operation, Outcome};
    for source in [true, false] {
        let mut e = Engine::new(scene()).unwrap();
        activate(&mut e, "player", "patient").unwrap();
        let mut entry = e.replay().ability_inputs[0];
        if source {
            entry.life.source_instance.generation += 1;
        } else {
            entry.life.target_instance.as_mut().unwrap().generation += 1;
        }
        let balance = e.simulation.abilities.state.balances.clone();
        e.simulation
            .step_with_commands(Default::default(), &[], &[entry]);
        assert_eq!(e.simulation.abilities.state.balances, balance);
        assert_eq!(e.snapshot()["vitality"]["actors"]["patient"]["units"], 100);
        assert_eq!(
            e.simulation
                .abilities
                .state
                .receipts
                .back()
                .unwrap()
                .outcome,
            Outcome::InstanceChanged
        );
    }
    let mut s = scene();
    s.ability_catalog
        .abilities
        .iter_mut()
        .find(|d| d.id == "strike")
        .unwrap()
        .cast_ticks = 10;
    let mut e = Engine::new(s).unwrap();
    activate(&mut e, "player", "patient").unwrap();
    let mut entry = e.replay().ability_inputs[0];
    e.advance(1).unwrap();
    let a = entry.actor as usize;
    let cast = e.simulation.abilities.state.casts[a].unwrap();
    entry.tick = e.simulation.tick;
    entry.sequence += 1;
    entry.operation = Operation::Cancel {
        execution_id: cast.id,
    };
    entry.life.source_instance.instance += 1;
    entry.life.target_instance = None;
    entry.life.target = None;
    e.simulation
        .step_with_commands(Default::default(), &[], &[entry]);
    assert_eq!(e.simulation.abilities.state.casts[a].unwrap().id, cast.id);
    assert_eq!(
        e.simulation
            .abilities
            .state
            .receipts
            .back()
            .unwrap()
            .outcome,
        Outcome::InstanceChanged
    );
}
