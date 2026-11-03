//! Critical save-game boundary: never lose state, transfer ownership or pay twice.
#[path = "../examples/support/npc_scene.rs"]
mod scale_fixture;
use ignis::{
    checkpoint::Checkpoint,
    engine::{Command, Engine, ExecutionMode},
    model::Scene,
};
use serde_json::{Value, json};
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}
fn same(a: &Engine, b: &Engine) {
    assert_eq!(a.simulation.tick, b.simulation.tick);
    assert_eq!(a.simulation.positions(), b.simulation.positions());
    assert_eq!(a.simulation.npcs, b.simulation.npcs);
    assert_eq!(
        a.simulation.actors.instance_snapshot(),
        b.simulation.actors.instance_snapshot()
    );
    assert_eq!(a.simulation.abilities.state, b.simulation.abilities.state);
    assert_eq!(a.simulation.vitality.state, b.simulation.vitality.state);
    assert_eq!(
        a.simulation.interactions.state,
        b.simulation.interactions.state
    );
    assert_eq!(a.simulation.mobility.states, b.simulation.mobility.states);
    assert_eq!(
        a.simulation.character_controllers,
        b.simulation.character_controllers
    );
    assert_eq!(a.simulation.presentation, b.simulation.presentation);
    assert_eq!(a.simulation.footsteps, b.simulation.footsteps);
    assert_eq!(a.simulation.generators, b.simulation.generators);
}
#[test]
fn native_checkpoint_preserves_live_states_contacts_origins_epochs_and_replay_baseline() {
    let mut scene: Scene =
        serde_json::from_str(include_str!("../examples/npc-v1.scene.json")).unwrap();
    scene.ability_catalog.attributes.buffs=serde_json::from_value(json!([{"id":"ward","duration_ticks":240,"reapply":"refresh","modifiers":[{"attribute":"combat.damage_received.multiplier","priority":1,"operation":{"kind":"multiply","basis_points":7500}}]}])).unwrap();
    for definition in [
        json!({"id":"ward","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":300,"costs":[{"resource":"energy","units":1}],"effects":[{"kind":"apply_buff","recipient":"source","buff":"ward"}]}),
        json!({"id":"dot","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":300,"costs":[{"resource":"energy","units":3}],"periodic":{"interval_ticks":15,"pulses":20,"reapply":"refresh"},"effects":[{"kind":"damage","recipient":"target","amount":1}]}),
    ] {
        scene
            .ability_catalog
            .abilities
            .push(serde_json::from_value(definition).unwrap());
    }
    scene
        .entities
        .iter_mut()
        .find(|e| e.id == "ally")
        .unwrap()
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("ward".into());
    scene
        .entities
        .iter_mut()
        .find(|e| e.id == "guard")
        .unwrap()
        .abilities
        .as_mut()
        .unwrap()
        .grants
        .push("dot".into());
    let mut source = Engine::new(scene).unwrap();
    let old_control = run(
        &mut source,
        Command::BindControl {
            actor: "player".into(),
        },
    )["control"]
        .as_str()
        .unwrap()
        .to_owned();
    run(
        &mut source,
        Command::TalkNpc {
            actor: "player".into(),
            npc: "ally".into(),
        },
    );
    run(
        &mut source,
        Command::Interact {
            actor: "player".into(),
            point: "door".into(),
            action: "open".into(),
        },
    );
    run(
        &mut source,
        Command::ActivateAbility {
            actor: "ally".into(),
            slot: 2,
            target: None,
        },
    );
    run(
        &mut source,
        Command::ActivateAbility {
            actor: "guard".into(),
            slot: 2,
            target: Some("player".into()),
        },
    );
    run(
        &mut source,
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position {
                position: [2., 0.8, 4.],
            },
            radius: 8.,
        },
    );
    assert!(Checkpoint::capture(&source).is_err());
    source.advance(60).unwrap();
    assert!(source.simulation.character_controllers["creature"].airborne);
    let old_session = run(
        &mut source,
        Command::InspectDialogue {
            actor: "player".into(),
        },
    )["handle"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(!source.simulation.abilities.state.periodic.active.is_empty());
    assert!(!source.simulation.abilities.state.buffs.active.is_empty());
    assert!(
        !run(&mut source, Command::InspectNpcCooperation)["support_claims"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let bytes = Checkpoint::capture(&source).unwrap().encode().unwrap();
    assert_eq!(&bytes[..8], b"IGNISCPT");
    let mut loaded = Engine::from_checkpoint(&bytes, Some(ExecutionMode::Recorded)).unwrap();
    same(&source, &loaded);
    assert!(
        loaded
            .execute(
                Command::SetControlInput {
                    control: old_control.clone(),
                    input: Default::default()
                }
                .into()
            )
            .is_err()
    );
    assert!(
        loaded
            .execute(
                Command::ChooseDialogue {
                    actor: "player".into(),
                    session: old_session.clone(),
                    choice: "yes".into()
                }
                .into()
            )
            .is_err()
    );
    let new_session = run(
        &mut loaded,
        Command::InspectDialogue {
            actor: "player".into(),
        },
    )["handle"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(old_session, new_session);
    run(
        &mut source,
        Command::ReleaseControl {
            control: old_control,
        },
    );
    for (e, s) in [(&mut source, old_session), (&mut loaded, new_session)] {
        run(
            e,
            Command::ChooseDialogue {
                actor: "player".into(),
                session: s,
                choice: "yes".into(),
            },
        );
    }
    for e in [&mut source, &mut loaded] {
        run(
            e,
            Command::EmitNoise {
                origin: ignis::noise::NoiseOrigin::Position {
                    position: [2., 0.8, 4.],
                },
                radius: 8.,
            },
        );
    }
    for _ in 0..180 {
        source.advance(1).unwrap();
        loaded.advance(1).unwrap();
        same(&source, &loaded);
    }
    let replay = loaded.replay();
    assert_eq!(replay.inputs.len(), 180);
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let prefix = Engine::replay_at(&replay, Some(0)).unwrap();
    assert_eq!(prefix.simulation.tick, 60);
    assert_eq!(
        prefix.simulation.positions(),
        Engine::from_checkpoint(&bytes, None)
            .unwrap()
            .simulation
            .positions()
    );
    let before = loaded.snapshot();
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() ^= 1;
    assert!(loaded.load_checkpoint(&bad, None).is_err());
    assert_eq!(loaded.snapshot(), before);
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(loaded.load_checkpoint(&extra, None).is_err());
    assert_eq!(loaded.snapshot(), before);
    // A valid checksum does not excuse invalid primary mappings.
    let mut malformed: ciborium::value::Value = ciborium::de::from_reader(&bytes[50..]).unwrap();
    fn member<'a>(v: &'a mut ciborium::value::Value, key: &str) -> &'a mut ciborium::value::Value {
        let ciborium::value::Value::Map(m) = v else {
            panic!("map")
        };
        &mut m
            .iter_mut()
            .find(|(k, _)| k.as_text() == Some(key))
            .unwrap()
            .1
    }
    let ciborium::value::Value::Array(order) =
        member(member(member(&mut malformed, "state"), "actors"), "order")
    else {
        panic!("order")
    };
    order[0] = ciborium::value::Value::Integer(65535u64.into());
    let mut payload = Vec::new();
    ciborium::ser::into_writer(&malformed, &mut payload).unwrap();
    let mut forged = bytes[..50].to_vec();
    forged[10..18].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    forged[18..50].copy_from_slice(ring::digest::digest(&ring::digest::SHA256, &payload).as_ref());
    forged.extend(payload);
    assert!(loaded.load_checkpoint(&forged, None).is_err());
    assert_eq!(loaded.snapshot(), before);
    // Counter-domain boundary: a continuous world's serial must keep its namespace.
    let mut counter: ciborium::value::Value = ciborium::de::from_reader(&bytes[50..]).unwrap();
    let ciborium::value::Value::Array(sequences) = member(&mut counter, "sequences") else {
        panic!("sequences")
    };
    sequences[0] = ciborium::value::Value::Integer((1u64 << 30).into());
    let mut payload = Vec::new();
    ciborium::ser::into_writer(&counter, &mut payload).unwrap();
    let mut counter_bytes = bytes[..50].to_vec();
    counter_bytes[10..18].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    counter_bytes[18..50]
        .copy_from_slice(ring::digest::digest(&ring::digest::SHA256, &payload).as_ref());
    counter_bytes.extend(payload);
    let mut long = Engine::from_checkpoint(&counter_bytes, Some(ExecutionMode::Recorded)).unwrap();
    run(
        &mut long,
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position {
                position: [0., 0.8, 0.],
            },
            radius: 2.,
        },
    );
    long.advance(1).unwrap();
    assert_eq!(
        Engine::verify_replay(&long.replay()).unwrap()["matched"],
        true
    );
    // Retire the author while its paid projectile is still flying, and reuse storage.
    let mut origin = Engine::from_checkpoint(&bytes, None).unwrap();
    for _ in 0..240 {
        if !origin
            .simulation
            .abilities
            .state
            .projectiles
            .active
            .is_empty()
        {
            break;
        }
        origin.advance(1).unwrap();
    }
    assert!(
        !origin
            .simulation
            .abilities
            .state
            .projectiles
            .active
            .is_empty()
    );
    let guard = origin.simulation.npcs["guard"].memory;
    assert!(guard.is_some());
    let pos = origin.simulation.position("guard").unwrap();
    run(
        &mut origin,
        Command::DespawnActor {
            actor: "guard".into(),
        },
    );
    origin.advance(1).unwrap();
    let retired = Checkpoint::capture(&origin).unwrap().encode().unwrap();
    same(&origin, &Engine::from_checkpoint(&retired, None).unwrap());
    run(
        &mut origin,
        Command::SpawnActor {
            template: "creature_template".into(),
            position: pos,
            target: None,
        },
    );
    origin.advance(1).unwrap();
    assert!(!origin.simulation.npcs.contains_key("guard"));
    let bytes2 = Checkpoint::capture(&origin).unwrap().encode().unwrap();
    let mut origin_loaded = Engine::from_checkpoint(&bytes2, None).unwrap();
    same(&origin, &origin_loaded);
    for _ in 0..120 {
        origin.advance(1).unwrap();
        origin_loaded.advance(1).unwrap();
        same(&origin, &origin_loaded);
    }
    assert_eq!(
        Engine::verify_replay(&origin_loaded.replay()).unwrap()["matched"],
        true
    );
    // Continuous saves do not require retaining or replaying old user inputs.
    let mut continuous = Engine::from_checkpoint(&bytes, Some(ExecutionMode::Continuous)).unwrap();
    continuous.advance(5).unwrap();
    let encoded = Checkpoint::capture(&continuous).unwrap().encode().unwrap();
    let c = Engine::from_checkpoint(&encoded, None).unwrap();
    assert_eq!(c.execution_mode(), ExecutionMode::Continuous);
    assert_eq!(c.inspect_execution()["retained"]["ticks"], 0);
    assert!(c.recorded_replay().is_err());
    let mut large =
        Engine::new_with_mode(scale_fixture::scene(), ExecutionMode::Continuous).unwrap();
    large.advance(2).unwrap();
    let big = Checkpoint::capture(&large).unwrap().encode().unwrap();
    let restored = Engine::from_checkpoint(&big, None).unwrap();
    same(&large, &restored);
    assert_eq!(restored.simulation.npcs.len(), 100);
    // Authoring and reset deliberately return to the document, not a nested baseline.
    loaded.reset();
    assert_eq!(loaded.baseline_tick(), 0);
    assert_eq!(loaded.simulation.tick, 0);
    assert!(loaded.replay().baseline.is_none());
    let mut mismatch = replay.clone();
    mismatch.scene.name = "changed".into();
    assert!(Engine::verify_replay(&mismatch).is_err());
    let _ = json!({"native_bytes":bytes.len()});
}
