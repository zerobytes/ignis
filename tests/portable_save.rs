//! Critical save-game boundary: never lose state, transfer ownership or pay twice.
#[path = "../examples/support/npc_scene.rs"]
mod scale_fixture;
use ignis::{
    engine::{Command, Engine, ExecutionMode},
    model::Scene,
    portable_save::PortableSave,
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
fn portable_restore_preserves_paid_effects_identity_and_continuation() {
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
    scene.entities.push(serde_json::from_value(json!({"id":"crate","name":"crate","body":"dynamic","color":[0.6,0.4,0.2],"transform":{"position":[8.,3.,6.],"size":[0.6,0.6,0.6]}})).unwrap());
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
    assert!(PortableSave::capture(&source).is_err());
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
    let bytes = PortableSave::capture(&source).unwrap().encode().unwrap();
    assert_eq!(&bytes[..8], b"IGNISPSV");
    let mut loaded = Engine::from_portable_save(&bytes, Some(ExecutionMode::Recorded)).unwrap();
    same(&source, &loaded);
    let mut original: ciborium::value::Value = ciborium::de::from_reader(&bytes[50..]).unwrap();
    let recaptured = PortableSave::capture(&loaded).unwrap().encode().unwrap();
    let mut restored_primaries: ciborium::value::Value =
        ciborium::de::from_reader(&recaptured[50..]).unwrap();
    let first = member(member(&mut original, "state"), "physical")
        .as_array()
        .unwrap();
    let second = member(member(&mut restored_primaries, "state"), "physical")
        .as_array()
        .unwrap();
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second) {
        for (key, value) in a.as_map().unwrap() {
            let other = &b
                .as_map()
                .unwrap()
                .iter()
                .find(|(k, _)| k == key)
                .unwrap()
                .1;
            let id = &a
                .as_map()
                .unwrap()
                .iter()
                .find(|(k, _)| k.as_text() == Some("id"))
                .unwrap()
                .1;
            assert_eq!(value, other, "Physical primary {id:?}/{key:?}");
        }
    }
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
    // Solver cache is deliberately cold. Compare both restored continuations on
    // this backend. Original/cold-solver future drift is measured, not claimed
    // bounded by node alignment: contact warmstart is outside this format.
    let mut local_twin = Engine::replay_at(&loaded.replay(), None).unwrap();
    let mut max_cold_solver_drift = 0f32;
    for _ in 0..180 {
        source.advance(1).unwrap();
        loaded.advance(1).unwrap();
        local_twin.advance(1).unwrap();
        same(&local_twin, &loaded);
        for (id, p) in source.simulation.positions() {
            let q = loaded.simulation.positions()[&id];
            for i in 0..3 {
                assert!(q[i].is_finite(), "{id}: nonfinite resumed pose");
                let drift = (p[i] - q[i]).abs();
                max_cold_solver_drift = max_cold_solver_drift.max(drift);
                assert!(
                    drift < 0.002,
                    "Fixture cold-solver continuation: {id}: {p:?} vs {q:?}"
                );
            }
        }
        assert_eq!(
            source.simulation.abilities.state.balances,
            loaded.simulation.abilities.state.balances
        );
        assert_eq!(
            source.simulation.abilities.state.ready_at,
            loaded.simulation.abilities.state.ready_at
        );
        assert_eq!(
            source.simulation.abilities.state.next_execution,
            loaded.simulation.abilities.state.next_execution
        );
        assert_eq!(
            source.simulation.vitality.state,
            loaded.simulation.vitality.state
        );
    }
    eprintln!(
        "Cold solver/original max coordinate drift after 180 ticks: {max_cold_solver_drift}m; local replay/restore equality is gated separately"
    );
    let replay = loaded.replay();
    assert_eq!(replay.inputs.len(), 180);
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let prefix = Engine::replay_at(&replay, Some(0)).unwrap();
    assert_eq!(prefix.simulation.tick, 60);
    assert_eq!(
        prefix.simulation.positions(),
        Engine::from_portable_save(&bytes, None)
            .unwrap()
            .simulation
            .positions()
    );
    let before = loaded.snapshot();
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() ^= 1;
    assert!(loaded.load_portable_save(&bad, None).is_err());
    assert_eq!(loaded.snapshot(), before);
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(loaded.load_portable_save(&extra, None).is_err());
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
    assert!(loaded.load_portable_save(&forged, None).is_err());
    assert_eq!(loaded.snapshot(), before);
    // A checksummed body from a different person cannot acquire this slot.
    let mut wrong_body: ciborium::value::Value = ciborium::de::from_reader(&bytes[50..]).unwrap();
    let ciborium::value::Value::Array(bodies) =
        member(member(&mut wrong_body, "state"), "physical")
    else {
        panic!("physical")
    };
    *member(member(&mut bodies[0], "instance"), "generation") =
        ciborium::value::Value::Integer(u64::MAX.into());
    let mut payload = Vec::new();
    ciborium::ser::into_writer(&wrong_body, &mut payload).unwrap();
    let mut forged = bytes[..50].to_vec();
    forged[10..18].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    forged[18..50].copy_from_slice(ring::digest::digest(&ring::digest::SHA256, &payload).as_ref());
    forged.extend(payload);
    assert!(loaded.load_portable_save(&forged, None).is_err());
    assert_eq!(loaded.snapshot(), before);
    // No native solver/platform discriminator in the canonical file.
    let payload: ciborium::value::Value = ciborium::de::from_reader(&bytes[50..]).unwrap();
    fn no_backend(v: &ciborium::value::Value) {
        use ciborium::value::Value::*;
        match v {
            Map(entries) => {
                for (k, v) in entries {
                    if let Some(k) = k.as_text() {
                        assert!(
                            ![
                                "islands",
                                "broad",
                                "narrow",
                                "handles",
                                "surface_bodies",
                                "colliders",
                                "bodies",
                                "physics_version",
                                "platform",
                                "build_id"
                            ]
                            .contains(&k),
                            "{k}"
                        );
                    }
                    no_backend(v);
                }
            }
            Array(items) => {
                for v in items {
                    no_backend(v);
                }
            }
            _ => {}
        }
    }
    no_backend(&payload);
    let native = ignis::checkpoint::Checkpoint::capture(&loaded)
        .unwrap()
        .encode()
        .unwrap();
    // Fence schema drift: a new gameplay primary in native checkpoint must
    // explicitly enter the canonical contract, rather than silently disappearing.
    let native_payload: ciborium::value::Value = ciborium::de::from_reader(&native[50..]).unwrap();
    let state_keys = |v: &ciborium::value::Value| -> std::collections::BTreeSet<String> {
        let state = v
            .as_map()
            .unwrap()
            .iter()
            .find(|(k, _)| k.as_text() == Some("state"))
            .unwrap()
            .1
            .as_map()
            .unwrap();
        state
            .iter()
            .filter_map(|(k, _)| k.as_text())
            .filter(|k| {
                ![
                    "islands",
                    "broad",
                    "narrow",
                    "bodies",
                    "colliders",
                    "impulse",
                    "multibody",
                    "handles",
                    "surface_bodies",
                    "names",
                    "players",
                    "physical",
                ]
                .contains(k)
            })
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(state_keys(&native_payload), state_keys(&payload));
    let migrated = PortableSave::migrate(&native).unwrap();
    let mut migrated_engine = Engine::from_portable_save(&migrated, None).unwrap();
    same(&loaded, &migrated_engine);
    migrated_engine.advance(30).unwrap();
    assert_eq!(
        Engine::verify_replay(&migrated_engine.replay()).unwrap()["matched"],
        true
    );
    let mut future = bytes.clone();
    future[8..10].copy_from_slice(&2u16.to_le_bytes());
    assert!(loaded.load_portable_save(&future, None).is_err());
    assert_eq!(loaded.snapshot(), before);
    let mut continuous =
        Engine::from_portable_save(&bytes, Some(ExecutionMode::Continuous)).unwrap();
    continuous.advance(10).unwrap();
    let encoded = PortableSave::capture(&continuous)
        .unwrap()
        .encode()
        .unwrap();
    let restored = Engine::from_portable_save(&encoded, None).unwrap();
    same(&continuous, &restored);
    assert_eq!(restored.inspect_execution()["retained"]["ticks"], 0);
    // A removed delivery origin and a reused slot must remain distinct people.
    run(
        &mut continuous,
        Command::DespawnActor {
            actor: "guard".into(),
        },
    );
    continuous.advance(1).unwrap();
    run(
        &mut continuous,
        Command::SpawnActor {
            template: "creature_template".into(),
            position: [-4., 0.81, -3.],
            target: None,
        },
    );
    continuous.advance(1).unwrap();
    let reused = PortableSave::capture(&continuous)
        .unwrap()
        .encode()
        .unwrap();
    let mut restored = Engine::from_portable_save(&reused, Some(ExecutionMode::Recorded)).unwrap();
    same(&continuous, &restored);
    restored.advance(120).unwrap();
    assert_eq!(
        Engine::verify_replay(&restored.replay()).unwrap()["matched"],
        true
    );
    let mut large =
        Engine::new_with_mode(scale_fixture::scene(), ExecutionMode::Continuous).unwrap();
    large.advance(2).unwrap();
    let bytes = PortableSave::capture(&large).unwrap().encode().unwrap();
    same(&large, &Engine::from_portable_save(&bytes, None).unwrap());
    // New presentation clocks and derived physical IK queries also survive before
    // the first destination tick; empty rebuilt broadphase would lose foot support.
    let scene: Scene = serde_json::from_str(include_str!(
        "../examples/npc-presentation-complete.scene.json"
    ))
    .unwrap();
    let mut root = Engine::new(scene).unwrap();
    run(&mut root,Command::OrderNpc {actor:"a".into(),order:serde_json::from_value(json!({"kind":"move","position":[3,0.81,-2],"arrival_distance":0.2,"for_ticks":600,"formation":"offsets","spacing":1,"facing_degrees":0})).unwrap()});
    root.advance(30).unwrap();
    assert!(!root.simulation.presentation.motion.is_empty());
    let pose = root.simulation.presentation_poses();
    let bytes = PortableSave::capture(&root).unwrap().encode().unwrap();
    let mut restored = Engine::from_portable_save(&bytes, None).unwrap();
    same(&root, &restored);
    assert_eq!(pose, restored.simulation.presentation_poses());
    let mut root_twin = Engine::replay_at(&restored.replay(), None).unwrap();
    for _ in 0..20 {
        root.advance(1).unwrap();
        restored.advance(1).unwrap();
        root_twin.advance(1).unwrap();
        same(&root_twin, &restored);
        assert_eq!(
            root.simulation.presentation.motion,
            restored.simulation.presentation.motion
        );
        assert_eq!(
            root.simulation.presentation_poses(),
            restored.simulation.presentation_poses()
        );
        for id in ["a", "b"] {
            let p = root.simulation.position(id).unwrap();
            let q = restored.simulation.position(id).unwrap();
            for i in 0..3 {
                assert!((p[i] - q[i]).abs() < 0.002, "Root KCC continuation {id}");
            }
        }
    }
    assert_eq!(
        Engine::verify_replay(&restored.replay()).unwrap()["matched"],
        true
    );
}
