//! Critical guards for bounded lifetime memory, command identity, and replay integrity.
use ignis::{
    engine::{Command, Engine, ExecutionMode, MAX_RUNTIME_TICK},
    model::{RoomRecipe, Scene},
    noise::{Domain, MAX_SERIAL, NoiseOrigin, event_id},
    physics::Input,
};
use serde_json::{Value, json};

fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    s.ability_catalog = serde_json::from_value(json!({"resources":[],"abilities":[
        {"id":"signal","selection":"self_actor","range":0,"cast_ticks":0,"cooldown_ticks":0,
         "costs":[],"effects":[{"kind":"noise","radius":1}]}
    ]}))
    .unwrap();
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["signal"]})).unwrap());
    s
}
fn run(e: &mut Engine, c: Command) -> Value {
    e.execute(c.into()).unwrap()
}

#[test]
fn long_offline_execution_keeps_only_pending_inputs_and_cannot_forge_a_complete_replay() {
    let mut e = Engine::new_with_mode(scene(), ExecutionMode::Continuous).unwrap();
    let binding = run(
        &mut e,
        Command::BindControl {
            actor: "player".into(),
        },
    );
    let control = binding["control"].as_str().unwrap().to_owned();
    for i in 0..5000_u64 {
        let accepted = run(
            &mut e,
            Command::SetControlInput {
                control: control.clone(),
                input: Input::default(),
            },
        );
        assert_eq!(accepted["sequence"], i + 1);
        let ability = run(
            &mut e,
            Command::ActivateAbility {
                actor: "player".into(),
                slot: 1,
                target: None,
            },
        );
        assert_eq!(ability["request_id"], i);
        let noise = run(
            &mut e,
            Command::EmitNoise {
                origin: NoiseOrigin::Position {
                    position: [0., 0., 0.],
                },
                radius: 1.,
            },
        );
        assert_eq!(noise["queued"]["id"], i);
        e.advance(1).unwrap();
    }
    assert!(
        e.inspect_execution()["diagnostics"]["recent_manual_noises"]
            .as_u64()
            .unwrap()
            <= 128
    );
    for _ in 0..9 {
        e.advance(3600).unwrap();
    }
    assert!(e.simulation.tick > 36000);
    let input = run(
        &mut e,
        Command::SetControlInput {
            control: control.clone(),
            input: Input::default(),
        },
    );
    assert_eq!(input["sequence"], 5001);
    let state = e.inspect_execution();
    assert_eq!(
        state["admitted"],
        json!({"controls":5002,"abilities":5000,"noises":5000,"population":0})
    );
    assert_eq!(
        state["retained"],
        json!({"ticks":0,"controls":0,"abilities":0,"noises":0,"population":0})
    );
    assert_eq!(e.simulation.abilities.state.emitted, 5000);
    assert!(e.recorded_replay().is_err());
    assert!(Engine::verify_replay(&e.replay()).is_err());
    let before = e.snapshot();
    assert!(e.set_execution_mode(ExecutionMode::Recorded).is_err());
    assert!(
        e.execute(
            Command::SetControlInput {
                control: control.clone(),
                input: Input {
                    x: f32::NAN,
                    ..Default::default()
                }
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    e.reset();
    assert_eq!(e.execution_mode(), ExecutionMode::Continuous);
    assert!(
        e.execute(
            Command::SetControlInput {
                control,
                input: Input::default()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.inspect_execution()["admitted"]["controls"], 0);
    e.set_execution_mode(ExecutionMode::Recorded).unwrap();
    e.advance(1).unwrap();
    assert_eq!(
        Engine::verify_replay(&e.recorded_replay().unwrap()).unwrap()["matched"],
        true
    );

    let mut e = Engine::new_with_mode(scene(), ExecutionMode::Continuous).unwrap();
    e.simulation.tick = MAX_RUNTIME_TICK - 1;
    let before = e.snapshot();
    assert!(e.advance(2).is_err());
    assert_eq!(e.snapshot(), before);
    e.advance(1).unwrap();
    assert!(e.advance(1).is_err());
    assert!(
        e.execute(
            Command::ActivateAbility {
                actor: "player".into(),
                slot: 1,
                target: None
            }
            .into()
        )
        .is_err()
    );
}

#[test]
fn noise_identity_namespaces_do_not_alias_after_the_old_counter_boundary() {
    let mut ids = std::collections::BTreeSet::new();
    for serial in [0, 1, (1 << 30) - 1, 1 << 30, MAX_SERIAL] {
        for domain in [Domain::Manual, Domain::Ability, Domain::Footstep] {
            assert!(ids.insert(event_id(domain, serial)));
        }
    }
    assert_eq!(event_id(Domain::Manual, 0), 0);
    assert_eq!(event_id(Domain::Ability, 0), 1 << 30);
    assert_eq!(event_id(Domain::Footstep, 0), 1 << 31);
}
