use ignis::{
    abilities::CastMovement,
    controls::{Operation, PER_TICK},
    engine::{Command, Engine},
    model::{BodyKind, Entity, MovementMode, RoomRecipe, SCENE_VERSION, Scene},
    physics::Input,
};
use serde_json::{Value, json};

fn scene(players: usize) -> Scene {
    let mut scene = RoomRecipe {
        width: 40.,
        depth: 36.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let original = scene
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Player))
        .unwrap()
        .clone();
    scene.entities.retain(|e| e.body != Some(BodyKind::Player));
    for i in 0..players {
        let mut p = original.clone();
        p.id = if i == 0 {
            "player".into()
        } else {
            format!("player{i}")
        };
        p.transform.position = [-7. + i as f32 * 2., 0.8, -6. + i as f32 * 2.];
        p.footsteps = Some(
            serde_json::from_value(
                json!({"surface":"floor","stride":0.4,"radius":8,"min_speed":0.01}),
            )
            .unwrap(),
        );
        p.vitality =
            Some(serde_json::from_value(json!({"initial":100,"maximum":100,"scale":1})).unwrap());
        scene.entities.push(p);
    }
    scene
}
fn run(e: &mut Engine, command: Command) -> Value {
    e.execute(command.into()).unwrap()
}
fn bind(e: &mut Engine, actor: &str) -> String {
    run(
        e,
        Command::BindControl {
            actor: actor.into(),
        },
    )["control"]
        .as_str()
        .unwrap()
        .into()
}
fn drive(e: &mut Engine, control: &str, x: f32, z: f32, mode: MovementMode) {
    run(
        e,
        Command::SetControlInput {
            control: control.into(),
            input: Input { x, z, mode },
        },
    );
}
fn controls(e: &mut Engine) -> Value {
    run(e, Command::InspectControls)
}
fn check_replay(e: &Engine) {
    let replay = serde_json::from_str(&serde_json::to_string(&e.replay()).unwrap()).unwrap();
    let result = Engine::verify_replay(&replay).unwrap();
    assert_eq!(result["controls_matched"], true);
    assert_eq!(result["matched"], true, "{result}");
}
fn ability_scene(mode: CastMovement) -> Scene {
    let mut s = scene(2);
    s.ability_catalog = serde_json::from_value(json!({"resources":[{"id":"energy","scale":1}],"abilities":[
        {"id":"cast","selection":"self_actor","range":0,"cast_ticks":12,"cooldown_ticks":30,"costs":[{"resource":"energy","units":7}],"effects":[{"kind":"noise","radius":1}]},
        {"id":"kill","selection":"actor","range":30,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},
        {"id":"return","selection":"actor","range":30,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":50}]}
    ]})).unwrap();
    s.ability_catalog.abilities[0].movement = mode;
    for p in s
        .entities
        .iter_mut()
        .filter(|e| e.body == Some(BodyKind::Player))
    {
        p.abilities = Some(serde_json::from_value(json!({"resources":[{"resource":"energy","initial":100,"capacity":100}],"grants":["cast","kill","return"]})).unwrap());
        p.blocks_vision = Some(false);
    }
    s
}
fn activate(e: &mut Engine, actor: &str, slot: u8, target: Option<&str>) {
    run(
        e,
        Command::ActivateAbility {
            actor: actor.into(),
            slot,
            target: target.map(Into::into),
        },
    );
}
fn actor_slot(e: &Engine, id: &str) -> usize {
    e.simulation
        .actors
        .actors
        .iter()
        .position(|&i| e.simulation.definitions().entities[i as usize].id == id)
        .unwrap()
}

#[test]
fn independent_profiles_inputs_and_stationary_unbound_players() {
    let mut s = scene(3);
    let second = s.entities.iter_mut().find(|e| e.id == "player1").unwrap();
    let mut profile = second.locomotion.unwrap_or_default();
    profile.sneak.speed = 0.75;
    profile.sneak.noise_multiplier = 0.1;
    second.locomotion = Some(profile);
    let mut e = Engine::new(s).unwrap();
    let a = bind(&mut e, "player");
    let b = bind(&mut e, "player1");
    drive(&mut e, &a, 1., 0., MovementMode::Run);
    drive(&mut e, &b, 1., 0., MovementMode::Sneak);
    let before = e.simulation.positions();
    e.advance(60).unwrap();
    let after = e.simulation.positions();
    assert!((after["player"][0] - before["player"][0] - 6.).abs() < 0.05);
    assert!((after["player1"][0] - before["player1"][0] - 0.75).abs() < 0.05);
    assert!((after["player2"][0] - before["player2"][0]).abs() < 0.01);
    for (id, radius) in [("player", 14.4), ("player1", 0.8)] {
        let index = e.scene.entities.iter().position(|p| p.id == id).unwrap() as u16;
        assert!(
            e.simulation
                .footsteps
                .recent
                .iter()
                .any(|n| n.emitter_index == Some(index) && (n.radius - radius).abs() < 0.001)
        );
    }
    assert_eq!(e.simulation.footsteps.states["player2"].emitted, 0);
    check_replay(&e);
}

#[test]
fn single_player_legacy_and_explicit_control_have_identical_physics_and_steps() {
    let mut legacy = Engine::new(scene(1)).unwrap();
    let mut explicit = Engine::new(scene(1)).unwrap();
    let control = bind(&mut explicit, "player");
    for mode in [MovementMode::Walk, MovementMode::Sneak, MovementMode::Run] {
        let input = Input {
            x: 1.,
            z: 0.25,
            mode,
        };
        run(&mut legacy, Command::SetInput { input });
        drive(&mut explicit, &control, input.x, input.z, mode);
        legacy.advance(20).unwrap();
        explicit.advance(20).unwrap();
        assert_eq!(
            legacy.simulation.positions(),
            explicit.simulation.positions()
        );
        assert_eq!(legacy.simulation.footsteps, explicit.simulation.footsteps);
    }
    check_replay(&legacy);
    check_replay(&explicit);
}

#[test]
fn exclusivity_release_and_stale_handles_are_atomic() {
    let mut e = Engine::new(scene(2)).unwrap();
    let a = bind(&mut e, "player");
    let b = bind(&mut e, "player1");
    let before = e.snapshot();
    let log = serde_json::to_vec(&e.replay()).unwrap();
    for command in [
        Command::BindControl {
            actor: "player".into(),
        },
        Command::BindControl {
            actor: "floor".into(),
        },
        Command::SetInput {
            input: Input::default(),
        },
        Command::SetControlInput {
            control: "bad".into(),
            input: Input::default(),
        },
        Command::SetControlInput {
            control: a.clone(),
            input: Input {
                x: f32::NAN,
                ..Default::default()
            },
        },
        Command::SetControlInput {
            control: a.clone(),
            input: Input {
                z: 2.,
                ..Default::default()
            },
        },
    ] {
        assert!(e.execute(command.into()).is_err());
        assert_eq!(e.snapshot(), before);
        assert_eq!(serde_json::to_vec(&e.replay()).unwrap(), log);
    }
    drive(&mut e, &a, 1., 0., MovementMode::Walk);
    drive(&mut e, &b, 0., 1., MovementMode::Walk);
    run(&mut e, Command::ReleaseControl { control: a.clone() });
    let fresh = bind(&mut e, "player");
    assert_ne!(a, fresh);
    let before = e.snapshot();
    assert!(
        e.execute(Command::ReleaseControl { control: a.clone() }.into())
            .is_err()
    );
    assert!(
        e.execute(
            Command::SetControlInput {
                control: a,
                input: Input::default()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let p = e.simulation.positions();
    e.advance(20).unwrap();
    assert!((e.simulation.positions()["player"][0] - p["player"][0]).abs() < 0.01);
    assert!(e.simulation.positions()["player1"][2] > p["player1"][2] + 1.);
    check_replay(&e);
}

#[test]
fn bind_clears_legacy_input_and_prefix_preserves_final_tick_control_changes() {
    let mut e = Engine::new(scene(1)).unwrap();
    run(
        &mut e,
        Command::SetInput {
            input: Input {
                x: 1.,
                ..Default::default()
            },
        },
    );
    e.advance(2).unwrap();
    let a = bind(&mut e, "player");
    drive(&mut e, &a, 0., 1., MovementMode::Sneak);
    assert_eq!(e.input, Input::default());
    let replay = e.replay();
    let mut prefix = Engine::replay_at(&replay, Some(2)).unwrap();
    assert_eq!(prefix.input, Input::default());
    assert_eq!(prefix.simulation.positions(), e.simulation.positions());
    let one = controls(&mut prefix);
    let two = controls(&mut e);
    assert_eq!(one["players"][0]["input"], two["players"][0]["input"]);
    e.advance(5).unwrap();
    prefix.advance(5).unwrap();
    assert_eq!(prefix.simulation.positions(), e.simulation.positions());
    run(&mut e, Command::ReleaseControl { control: a });
    check_replay(&e);
}

#[test]
fn reset_edit_undo_and_failed_edit_do_not_reuse_control_handles() {
    let mut e = Engine::new(scene(2)).unwrap();
    let a = bind(&mut e, "player");
    let before = e.snapshot();
    let mut invalid = e.scene.entities[0].clone();
    invalid.transform.size[0] = -1.;
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(invalid)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    for command in [
        Command::Reset,
        Command::Upsert {
            entity: Box::new(e.scene.entities[0].clone()),
        },
        Command::Undo,
        Command::Redo,
    ] {
        let old = controls(&mut e)["players"][0]["control"]
            .as_str()
            .unwrap_or(&a)
            .to_owned();
        run(&mut e, command);
        let fresh = bind(&mut e, "player");
        assert_ne!(old, fresh);
        assert!(
            e.execute(
                Command::SetControlInput {
                    control: old,
                    input: Input::default()
                }
                .into()
            )
            .is_err()
        );
    }
    check_replay(&e);
}

#[test]
fn movement_lock_is_per_player_and_other_players_do_not_interrupt_a_cast() {
    let mut e = Engine::new(ability_scene(CastMovement::Lock {})).unwrap();
    let a = bind(&mut e, "player");
    let b = bind(&mut e, "player1");
    drive(&mut e, &a, 1., 0., MovementMode::Walk);
    drive(&mut e, &b, 1., 0., MovementMode::Walk);
    activate(&mut e, "player", 1, None);
    let before = e.simulation.positions();
    e.advance(6).unwrap();
    assert!((e.simulation.positions()["player"][0] - before["player"][0]).abs() < 0.01);
    assert!(e.simulation.positions()["player1"][0] > before["player1"][0] + 0.3);
    e.advance(10).unwrap();
    assert!(e.simulation.positions()["player"][0] > before["player"][0]);
    check_replay(&e);
    let mut e = Engine::new(ability_scene(CastMovement::Interrupt {
        speed_tolerance: 0.2,
    }))
    .unwrap();
    let a = bind(&mut e, "player");
    let b = bind(&mut e, "player1");
    activate(&mut e, "player", 1, None);
    drive(&mut e, &b, 1., 0., MovementMode::Walk);
    e.advance(5).unwrap();
    assert!(e.simulation.abilities.state.casts[actor_slot(&e, "player")].is_some());
    drive(&mut e, &a, 1., 0., MovementMode::Walk);
    e.advance(1).unwrap();
    let slot = actor_slot(&e, "player");
    assert!(e.simulation.abilities.state.casts[slot].is_none());
    assert_eq!(e.simulation.abilities.state.balances[slot][0], 100);
    check_replay(&e);
}

#[test]
fn death_clears_only_the_victims_input_and_revival_does_not_restore_it() {
    for enabled in [false, true] {
        let mut s = ability_scene(CastMovement::Free {});
        if enabled {
            for p in s
                .entities
                .iter_mut()
                .filter(|e| e.body == Some(BodyKind::Player))
            {
                p.character_controller = Some(Default::default());
            }
        }
        let mut e = Engine::new(s).unwrap();
        let a = bind(&mut e, "player");
        let b = bind(&mut e, "player1");
        drive(&mut e, &a, 1., 0., MovementMode::Run);
        drive(&mut e, &b, 0., 1., MovementMode::Walk);
        activate(&mut e, "player1", 2, Some("player"));
        e.advance(1).unwrap();
        assert!(e.simulation.vitality.dead(actor_slot(&e, "player")));
        if enabled {
            assert!(!e.simulation.character_controllers["player"].active);
        }
        let state = controls(&mut e);
        assert_eq!(state["players"][0]["input"]["x"], 0.);
        assert_eq!(state["players"][1]["input"]["z"], 1.);
        let before = e.snapshot();
        assert!(
            e.execute(
                Command::SetControlInput {
                    control: a.clone(),
                    input: Input::default()
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(before, e.snapshot());
        activate(&mut e, "player1", 3, Some("player"));
        e.advance(2).unwrap();
        if enabled {
            assert!(e.simulation.character_controllers["player"].active);
            assert_eq!(
                e.simulation.character_controllers["player"].requested_velocity,
                [0.; 2]
            );
        }
        let p = e.simulation.positions()["player"];
        e.advance(5).unwrap();
        assert!((e.simulation.positions()["player"][0] - p[0]).abs() < 0.01);
        drive(&mut e, &a, 1., 0., MovementMode::Walk);
        e.advance(3).unwrap();
        assert!(e.simulation.positions()["player"][0] > p[0]);
        check_replay(&e);
    }
}

#[test]
fn simultaneous_players_collide_with_each_other_and_walls() {
    let mut s = scene(2);
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [-1.5, 0.8, 0.];
    s.entities
        .iter_mut()
        .find(|e| e.id == "player1")
        .unwrap()
        .transform
        .position = [1.5, 0.8, 0.];
    let mut e = Engine::new(s).unwrap();
    let a = bind(&mut e, "player");
    let b = bind(&mut e, "player1");
    drive(&mut e, &a, 1., 0., MovementMode::Run);
    drive(&mut e, &b, -1., 0., MovementMode::Run);
    e.advance(60).unwrap();
    let p = e.simulation.positions();
    assert!(p["player"][0] < p["player1"][0]);
    assert!(p["player1"][0] - p["player"][0] > 0.5);
    drive(&mut e, &a, 0., 1., MovementMode::Run);
    drive(&mut e, &b, 0., -1., MovementMode::Run);
    e.advance(240).unwrap();
    let p = e.simulation.positions();
    assert!(p["player"][2] < 18.);
    assert!(p["player1"][2] > -18.);
    check_replay(&e);
}

#[test]
fn death_and_return_in_one_tick_do_not_apply_old_input_to_the_new_life() {
    let mut s = ability_scene(CastMovement::Free {});
    let mut third = s
        .entities
        .iter()
        .find(|e| e.id == "player1")
        .unwrap()
        .clone();
    third.id = "player2".into();
    third.transform.position = [4., 0.8, 0.];
    s.entities.push(third);
    let mut e = Engine::new(s).unwrap();
    let control = bind(&mut e, "player");
    drive(&mut e, &control, 1., 0., MovementMode::Run);
    activate(&mut e, "player1", 2, Some("player"));
    activate(&mut e, "player2", 3, Some("player"));
    let before = e.simulation.positions()["player"];
    e.advance(1).unwrap();
    let slot = actor_slot(&e, "player");
    assert_eq!(e.simulation.vitality.state.lives[slot], 1);
    assert!(!e.simulation.vitality.dead(slot));
    assert!((e.simulation.positions()["player"][0] - before[0]).abs() < 0.001);
    assert_eq!(controls(&mut e)["players"][0]["input"]["x"], 0.);
    check_replay(&e);
}

#[test]
fn actor_capacity_and_historical_versions_are_checked_before_catalog_creation() {
    let mut s = scene(2);
    s.version = 75;
    assert!(Engine::new(s).is_err());
    let mut s = scene(9);
    assert_eq!(s.version, SCENE_VERSION);
    assert!(Engine::new(s.clone()).is_ok());
    let mut extra = s
        .entities
        .iter()
        .find(|e| e.body == Some(BodyKind::Player))
        .unwrap()
        .clone();
    extra.id = "extra".into();
    s.entities.push(extra);
    assert!(Engine::new(s.clone()).is_ok());
    s.version = 86;
    assert!(Engine::new(s).is_err());
    let mut s = scene(3);
    let template:Entity=serde_json::from_value(json!({"id":"guard","name":"guard","color":[0.4,0.2,0.7],"body":"npc","transform":{"position":[0,0.8,0],"size":[0.6,1.6,0.6]},"npc":{"surface":"floor","speed":1,"stop_distance":0.5,"enabled":false}})).unwrap();
    s.npc_templates.push(template);
    let mut e = Engine::new(s).unwrap();
    assert_eq!(e.simulation.actors.actors.len(), 11);
    assert_eq!(controls(&mut e)["limits"]["npc_capacity"], 8);
    for i in 0..8 {
        run(
            &mut e,
            Command::SpawnActor {
                template: "guard".into(),
                position: [-6. + i as f32 * 2., 0.8, 6.],
                target: None,
            },
        );
    }
    e.advance(1).unwrap();
    assert!(
        e.simulation
            .population
            .receipts
            .iter()
            .all(|r| r.outcome == ignis::population::Outcome::Spawned)
    );
    run(
        &mut e,
        Command::SpawnActor {
            template: "guard".into(),
            position: [8., 0.8, 6.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.population.receipts.back().unwrap().outcome,
        ignis::population::Outcome::Capacity
    );
    check_replay(&e);
}

#[test]
fn input_budget_reserves_release_and_invalid_events_preserve_replay() {
    let mut e = Engine::new(scene(1)).unwrap();
    let a = bind(&mut e, "player");
    for _ in 0..PER_TICK - 2 {
        drive(&mut e, &a, 1., 0., MovementMode::Walk);
    }
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::SetControlInput {
                control: a.clone(),
                input: Input::default()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    run(&mut e, Command::ReleaseControl { control: a });
    assert_eq!(e.replay().control_events.len(), PER_TICK);
    check_replay(&e);
    let mut bad = e.replay();
    bad.control_events[0].sequence = 10;
    assert!(Engine::verify_replay(&bad).is_err());
    let mut bad = e.replay();
    bad.control_events[1].tick = 1;
    assert!(Engine::verify_replay(&bad).is_err());
    let mut bad = e.replay();
    if let Operation::Bind { ref mut life, .. } = bad.control_events[0].operation {
        *life = 3;
    }
    assert!(Engine::verify_replay(&bad).is_err());
    let mut bad = e.replay();
    bad.expected_controls.next_control += 1;
    assert_eq!(Engine::verify_replay(&bad).unwrap()["matched"], false);
}

#[test]
fn total_event_budget_also_reserves_release_without_resetting_the_world() {
    let mut e = Engine::new(scene(1)).unwrap();
    let a = bind(&mut e, "player");
    while e.replay().control_events.len() < ignis::controls::MAX_EVENTS - 1 {
        if e.execute(
            Command::SetControlInput {
                control: a.clone(),
                input: Input {
                    x: 0.001,
                    ..Default::default()
                },
            }
            .into(),
        )
        .is_err()
        {
            e.advance(1).unwrap();
        }
    }
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::SetControlInput {
                control: a.clone(),
                input: Input::default()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    run(&mut e, Command::ReleaseControl { control: a });
    assert_eq!(e.replay().control_events.len(), ignis::controls::MAX_EVENTS);
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::BindControl {
                actor: "player".into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    e.advance(2).unwrap();
    check_replay(&e);
}

#[test]
fn control_serial_exhaustion_and_stale_instance_fail_without_mutation() {
    let e = Engine::new(scene(1)).unwrap();
    let actors = &e.simulation.actors;
    let slot = actor_slot(&e, "player");
    let actor = actors.reference(slot as u16);
    let mut controls = ignis::controls::Runtime::new(actors.actors.len());
    controls.state.next_control = u64::MAX;
    let before = controls.state.clone();
    assert!(
        controls
            .apply(
                Operation::Bind {
                    control: u64::MAX,
                    actor,
                    life: 0
                },
                actors,
                e.simulation.definitions(),
                &e.simulation.vitality
            )
            .is_err()
    );
    assert_eq!(before, controls.state);
    controls.state.next_control = 1;
    let before = controls.state.clone();
    let mut stale = actor;
    stale.generation += 1;
    assert!(
        controls
            .apply(
                Operation::Bind {
                    control: 1,
                    actor: stale,
                    life: 0
                },
                actors,
                e.simulation.definitions(),
                &e.simulation.vitality
            )
            .is_err()
    );
    assert_eq!(before, controls.state);
    controls
        .apply(
            Operation::Bind {
                control: 1,
                actor,
                life: 0,
            },
            actors,
            e.simulation.definitions(),
            &e.simulation.vitality,
        )
        .unwrap();
    let pointer = controls.inputs(actors, &e.simulation.vitality).as_ptr();
    controls
        .apply(
            Operation::Input {
                control: 1,
                input: Input {
                    x: 1.,
                    ..Default::default()
                },
                life: 0,
            },
            actors,
            e.simulation.definitions(),
            &e.simulation.vitality,
        )
        .unwrap();
    assert_eq!(
        pointer,
        controls.inputs(actors, &e.simulation.vitality).as_ptr()
    );
}

#[test]
fn independent_terrestrial_motors_and_dynamic_player_share_controls() {
    let mut s = scene(3);
    for id in ["player", "player1"] {
        s.entities
            .iter_mut()
            .find(|p| p.id == id)
            .unwrap()
            .character_controller = Some(Default::default());
    }
    let mut e = Engine::new(s).unwrap();
    let a = bind(&mut e, "player");
    let b = bind(&mut e, "player1");
    let c = bind(&mut e, "player2");
    drive(&mut e, &a, 1., 0., MovementMode::Run);
    drive(&mut e, &b, 0., -1., MovementMode::Sneak);
    drive(&mut e, &c, -1., 0., MovementMode::Walk);
    let start = e.simulation.positions();
    e.advance(30).unwrap();
    let end = e.simulation.positions();
    assert!(end["player"][0] > start["player"][0] + 0.5);
    assert!(end["player1"][2] < start["player1"][2] - 0.1);
    assert!(end["player2"][0] < start["player2"][0] - 0.5);
    let motors = &e.simulation.character_controllers;
    assert!(motors["player"].requested_velocity[0] > 0.);
    assert_eq!(motors["player"].requested_velocity[1], 0.);
    assert_eq!(motors["player1"].requested_velocity[0], 0.);
    assert!(motors["player1"].requested_velocity[1] < 0.);
    assert!(!motors.contains_key("player2"));
    check_replay(&e);
    let mut replay = e.replay();
    assert_eq!(
        Engine::verify_replay(&replay).unwrap()["character_controllers_matched"],
        true
    );
    replay
        .expected_character_controllers
        .get_mut("player")
        .unwrap()
        .requested_velocity[0] += 1.;
    let tampered = Engine::verify_replay(&replay).unwrap();
    assert_eq!(tampered["character_controllers_matched"], false);
    assert_eq!(tampered["controls_matched"], true);
    assert_eq!(tampered["meshes_matched"], true);
    assert_eq!(tampered["matched"], false);
    run(&mut e, Command::ReleaseControl { control: a });
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.character_controllers["player"].requested_velocity,
        [0.; 2]
    );
    assert!(e.simulation.character_controllers["player1"].requested_velocity[1] < 0.);
    check_replay(&e);
}
