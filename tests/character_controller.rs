use ignis::{
    character_controller::{Config, Step},
    engine::{Command, Engine},
    model::{BodyKind, Entity, RoomRecipe, Scene},
    physics::Input,
};
use serde_json::json;
fn box_(id: &str, p: [f32; 3], size: [f32; 3], body: &str) -> Entity {
    serde_json::from_value(json!({"id":id,"name":id,"body":body,"color":[0.2,0.6,0.5],"transform":{"position":p,"size":size}})).unwrap()
}
fn scene() -> Scene {
    let mut s = RoomRecipe {
        width: 30.,
        depth: 20.,
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position = [-1.5, 0.81, 0.];
    p.transform.size = [0.6, 1.6, 0.6];
    p.footsteps = None;
    p.character_controller = Some(Config::default());
    s
}
fn run(e: &mut Engine, c: Command) -> serde_json::Value {
    e.execute(c.into()).unwrap()
}
fn walk(e: &mut Engine, ticks: u32, x: f32) {
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
}
fn pos(e: &Engine) -> [f32; 3] {
    e.simulation.positions()["player"]
}
fn replay(e: &Engine) {
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
fn ramp(s: &mut Scene, degrees: f32) {
    let h = 4. * degrees.to_radians().tan();
    let mut r = box_("ramp", [2., h / 2., 0.], [4., h, 4.], "fixed");
    r.geometry = ignis::geometry::Geometry::Wedge {
        rise: ignis::geometry::Rise::PositiveX,
    };
    s.entities.push(r);
    s.entities
        .push(box_("landing", [7., h / 2., 0.], [6., h, 4.], "fixed"));
}
fn step(s: &mut Scene, h: f32) {
    s.entities
        .push(box_("step", [4., h / 2., 0.], [8., h, 4.], "fixed"));
}
#[test]
fn fast_sprint_sweeps_thin_wall_and_releasing_run_restores_walk() {
    let mut s = scene();
    let player = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    let mut locomotion = ignis::model::Locomotion::default();
    locomotion.run.speed = 600.;
    player.locomotion = Some(locomotion);
    s.entities
        .push(box_("thin_wall", [3., 2., 0.], [0.1, 4., 6.], "fixed"));
    let mut e = Engine::new(s).unwrap();
    e.advance(30).unwrap();
    run(
        &mut e,
        Command::SetInput {
            input: Input {
                x: 1.,
                z: 0.,
                mode: ignis::model::MovementMode::Run,
            },
        },
    );
    e.advance(3).unwrap();
    assert!(pos(&e)[0] > 2.5 && pos(&e)[0] < 2.65, "{:?}", pos(&e));
    assert!(e.simulation.character_controllers["player"].grounded);
    let before = pos(&e);
    walk(&mut e, 1, -1.);
    assert!((before[0] - pos(&e)[0] - 4. / 60.).abs() < 0.01);
    replay(&e);
}

#[test]
fn first_tick_ground_query_does_not_advance_time_or_fall_through_floor() {
    let mut e = Engine::new(scene()).unwrap();
    assert_eq!(e.simulation.tick, 0);
    e.advance(1).unwrap();
    let st = e.simulation.character_controllers["player"];
    assert!(st.grounded);
    assert!((pos(&e)[1] - 0.81).abs() < 0.003);
    e.advance(120).unwrap();
    assert!((pos(&e)[1] - 0.81).abs() < 0.003);
    replay(&e);
}
#[test]
fn climbs_allowed_ramp_and_rejects_steep_ramp() {
    for (angle, pass) in [(20., true), (40., false)] {
        let mut s = scene();
        ramp(&mut s, angle);
        let mut e = Engine::new(s).unwrap();
        walk(&mut e, 180, 1.);
        eprintln!("ramp{angle} {:?}", pos(&e));
        if pass {
            assert!(pos(&e)[0] > 4. && pos(&e)[1] > 2.);
        } else {
            assert!(pos(&e)[0] < 0.4);
        }
        replay(&e);
    }
}
#[test]
fn slope_override_and_descending_snap() {
    let mut s = scene();
    ramp(&mut s, 40.);
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.character_controller.as_mut().unwrap().max_slope_degrees = 45.;
    p.character_controller.as_mut().unwrap().slide_slope_degrees = 50.;
    let mut e = Engine::new(s).unwrap();
    walk(&mut e, 180, 1.);
    assert!(pos(&e)[0] > 4.);
    walk(&mut e, 180, -1.);
    assert!(pos(&e)[0] < 0. && pos(&e)[1] < 0.9);
    replay(&e);
}
#[test]
fn autostep_is_opt_in_and_respects_height_and_ceiling() {
    for (height, enabled, ceiling, pass) in [
        (0.2, false, false, false),
        (0.2, true, false, true),
        (0.5, true, false, false),
        (0.2, true, true, false),
    ] {
        let mut s = scene();
        step(&mut s, height);
        if enabled {
            s.entities
                .iter_mut()
                .find(|e| e.id == "player")
                .unwrap()
                .character_controller
                .as_mut()
                .unwrap()
                .step = Some(Step {
                max_height: 0.3,
                min_width: 0.4,
            });
        }
        if ceiling {
            s.entities
                .push(box_("ceiling", [2., 1.95, 0.], [6., 0.4, 4.], "fixed"));
        }
        let mut e = Engine::new(s).unwrap();
        walk(&mut e, 150, 1.);
        eprintln!("step {height} {enabled} {ceiling} {:?}", pos(&e));
        assert_eq!(pos(&e)[0] > 2., pass);
        replay(&e);
    }
}
#[test]
fn ledge_falls_with_bounded_velocity_instead_of_snapping_one_meter() {
    let mut s = scene();
    s.entities
        .push(box_("ledge", [-2., 0.5, 0.], [6., 1., 4.], "fixed"));
    s.entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [-1.5, 1.81, 0.];
    let mut e = Engine::new(s).unwrap();
    let mut air = 0;
    for _ in 0..130 {
        walk(&mut e, 1, 1.);
        let st = e.simulation.character_controllers["player"];
        air += usize::from(!st.grounded);
        assert!(st.vertical_velocity >= -50.);
    }
    assert!(air > 10);
    assert!(pos(&e)[0] > 2. && pos(&e)[1] < 0.9);
    replay(&e);
}
#[test]
fn dynamic_push_is_optional() {
    let mut outcomes = Vec::new();
    for push in [false, true] {
        let mut s = scene();
        s.entities
            .push(box_("crate", [1., 0.5, 0.], [1., 1., 1.], "dynamic"));
        s.entities
            .iter_mut()
            .find(|e| e.id == "player")
            .unwrap()
            .character_controller
            .as_mut()
            .unwrap()
            .push_dynamic_bodies = push;
        let mut e = Engine::new(s).unwrap();
        walk(&mut e, 150, 1.);
        outcomes.push(e.simulation.positions()["crate"][0]);
        replay(&e);
    }
    eprintln!("push {outcomes:?}");
    assert!(outcomes[0] < 1.1);
    assert!(outcomes[1] > 2.);
}
#[test]
fn config_validation_and_atomic_edit_history() {
    let mut e = Engine::new(scene()).unwrap();
    walk(&mut e, 15, 1.);
    let before = e.snapshot();
    let player = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "player")
        .unwrap()
        .clone();
    for key in [
        "gravity",
        "offset",
        "snap_distance",
        "max_slope_degrees",
        "slide_slope_degrees",
        "max_fall_speed",
    ] {
        let mut p = serde_json::to_value(&player).unwrap();
        p["character_controller"][key] = json!(-1);
        let c: Entity = serde_json::from_value(p).unwrap();
        assert!(
            e.execute(
                Command::Upsert {
                    entity: Box::new(c)
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(before, e.snapshot());
    }
    let mut p = player.clone();
    p.body = Some(BodyKind::Fixed);
    assert!(
        e.execute(
            Command::Upsert {
                entity: Box::new(p)
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(before, e.snapshot());
    let mut p = player;
    p.character_controller = None;
    run(
        &mut e,
        Command::Upsert {
            entity: Box::new(p),
        },
    );
    assert!(e.simulation.character_controllers.is_empty());
    run(&mut e, Command::Undo);
    assert!(e.simulation.character_controllers.contains_key("player"));
    run(&mut e, Command::Redo);
    assert!(e.simulation.character_controllers.is_empty());
}
#[test]
fn replay_compares_motor_state_and_prefixes() {
    let mut s = scene();
    ramp(&mut s, 20.);
    let mut e = Engine::new(s).unwrap();
    walk(&mut e, 40, 1.);
    let prefix = e.snapshot();
    walk(&mut e, 50, 1.);
    let r = e.replay();
    assert_eq!(Engine::replay_at(&r, Some(40)).unwrap().snapshot(), prefix);
    replay(&e);
    let mut bad = r;
    bad.expected_character_controllers
        .get_mut("player")
        .unwrap()
        .grounded ^= true;
    let result = Engine::verify_replay(&bad).unwrap();
    assert_eq!(result["matched"], false);
    assert_eq!(result["character_controllers_matched"], false);
}
fn npc() -> Entity {
    serde_json::from_value(json!({"id":"npc","name":"npc","body":"npc","color":[0.7,0.2,0.8],"transform":{"position":[0,0.8,3],"size":[0.6,1.6,0.6]},"character_controller":{},"vitality":{"initial":100,"maximum":100,"scale":1},"npc":{"surface":"floor","target":"player","speed":2,"stop_distance":0.5,"enabled":true,"awareness":{"mode":"perceptive","vision_range":20,"fov_degrees":360,"initial_facing_degrees":0,"memory_ticks":60,"search_ticks":60,"search_turn_degrees_per_second":0}}})).unwrap()
}
#[test]
fn npc_uses_same_ground_motor_with_planar_navigation() {
    let mut s = scene();
    s.entities.push(npc());
    let mut e = Engine::new(s).unwrap();
    e.advance(90).unwrap();
    assert!(e.simulation.positions()["npc"][2] < 1.);
    assert!(e.simulation.character_controllers["npc"].grounded);
    replay(&e);
}
#[test]
fn versions_local_ground_motor_and_rejects_air_and_older_templates() {
    let mut s = scene();
    let mut n = npc();
    n.npc.as_mut().unwrap().mobility = Some(serde_json::from_value(json!({"walk":true})).unwrap());
    n.npc.as_mut().unwrap().local_connections =
        Some(serde_json::from_value(json!({"walk":true,"fly":false,"max_distance":8})).unwrap());
    s.npc_templates.push(n);
    s.version = 84;
    assert!(s.validate().is_err());
    s.version = 85;
    assert!(s.validate().is_ok());
    s.npc_templates[0]
        .npc
        .as_mut()
        .unwrap()
        .mobility
        .as_mut()
        .unwrap()
        .fly_speed = Some(2.);
    assert!(s.validate().is_err());
    s.npc_templates.clear();
    s.version = 76;
    assert!(s.validate().is_err());
}
#[test]
fn death_and_resurrection_keep_identity_and_reenter_controller() {
    let mut s = scene();
    s.entities.push(npc());
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.abilities =
        Some(serde_json::from_value(json!({"resources":[],"grants":["kill","revive"]})).unwrap());
    s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[{"id":"kill","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"damage","recipient":"target","amount":100}]},{"id":"revive","selection":"actor","range":20,"cast_ticks":0,"cooldown_ticks":0,"costs":[],"effects":[{"kind":"resurrect","recipient":"target","amount":50}]}]})).unwrap();
    let mut e = Engine::new(s).unwrap();
    e.advance(2).unwrap();
    let instance = e.simulation.character_controllers["npc"].instance;
    run(
        &mut e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("npc".into()),
        },
    );
    e.advance(2).unwrap();
    assert!(!e.simulation.character_controllers["npc"].active);
    run(
        &mut e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 2,
            target: Some("npc".into()),
        },
    );
    e.advance(2).unwrap();
    assert!(e.simulation.character_controllers["npc"].active);
    assert_eq!(e.simulation.character_controllers["npc"].instance, instance);
    replay(&e);
}
#[test]
fn template_reuse_installs_fresh_state_and_despawn_removes_old() {
    let mut s = scene();
    let mut t = npc();
    t.id = "template".into();
    t.npc.as_mut().unwrap().enabled = false;
    s.npc_templates.push(t);
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        Command::SpawnActor {
            template: "template".into(),
            position: [0., 0.8, 3.],
            target: None,
        },
    );
    e.advance(2).unwrap();
    let (id, state) = e
        .simulation
        .character_controllers
        .iter()
        .find(|(id, _)| *id != "player")
        .map(|(id, s)| (id.clone(), *s))
        .unwrap();
    run(&mut e, Command::DespawnActor { actor: id.clone() });
    e.advance(1).unwrap();
    assert!(!e.simulation.character_controllers.contains_key(&id));
    run(
        &mut e,
        Command::SpawnActor {
            template: "template".into(),
            position: [1., 0.8, 3.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    let new = e
        .simulation
        .character_controllers
        .values()
        .find(|s| s.instance != e.simulation.character_controllers["player"].instance)
        .unwrap();
    assert_ne!(new.instance, state.instance);
    assert!(new.active);
    assert_eq!(new.vertical_velocity, 0.);
    replay(&e);
}

#[test]
fn cast_lock_stops_intention_but_not_gravity_and_interrupt_uses_actual_motion() {
    for mode in ["lock", "interrupt"] {
        let mut s = scene();
        let mut movement = json!({"mode":mode});
        if mode == "interrupt" {
            movement["speed_tolerance"] = json!(0.1);
        }
        s.ability_catalog=serde_json::from_value(json!({"resources":[],"abilities":[{"id":"cast","selection":"self_actor","range":0,"cast_ticks":20,"cooldown_ticks":30,"movement":movement,"costs":[],"effects":[{"kind":"noise","radius":5}]}]})).unwrap();
        let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
        p.abilities =
            Some(serde_json::from_value(json!({"resources":[],"grants":["cast"]})).unwrap());
        if mode == "lock" {
            p.transform.position[1] = 3.;
        }
        let mut e = Engine::new(s).unwrap();
        run(
            &mut e,
            Command::ActivateAbility {
                actor: "player".into(),
                slot: 1,
                target: None,
            },
        );
        e.advance(1).unwrap();
        walk(&mut e, 10, 1.);
        if mode == "lock" {
            assert!((pos(&e)[0] + 1.5).abs() < 0.001);
            assert!(pos(&e)[1] < 3.);
            assert_eq!(
                e.simulation.character_controllers["player"].requested_velocity,
                [0., 0.]
            );
        } else {
            assert!(pos(&e)[0] > -1.);
            assert_eq!(
                e.simulation
                    .abilities
                    .state
                    .receipts
                    .back()
                    .unwrap()
                    .outcome,
                ignis::abilities::Outcome::InterruptedByMovement
            );
        }
        replay(&e);
    }
}
#[test]
fn configured_fall_speed_and_gravity_and_dead_initial_body() {
    let mut s = scene();
    let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
    p.transform.position[1] = 20.;
    let c = p.character_controller.as_mut().unwrap();
    c.gravity = 60.;
    c.max_fall_speed = 2.;
    let mut e = Engine::new(s).unwrap();
    e.advance(60).unwrap();
    assert!((pos(&e)[1] - 18.016666).abs() < 0.03);
    assert_eq!(
        e.simulation.character_controllers["player"].vertical_velocity,
        -2.
    );
    replay(&e);
    let mut s = scene();
    let mut n = npc();
    n.vitality.as_mut().unwrap().initial = 0;
    n.transform.position[1] = 5.;
    s.entities.push(n);
    let mut e = Engine::new(s).unwrap();
    e.advance(30).unwrap();
    assert!(!e.simulation.character_controllers["npc"].active);
    assert!(e.simulation.positions()["npc"][1] < 4.);
    replay(&e);
}
#[test]
fn actor_queries_preserve_dynamic_pair_contacts_with_reserved_templates() {
    let mut s = scene();
    s.entities
        .push(box_("crate", [8., 1., 3.], [1., 1., 1.], "dynamic"));
    s.npc_templates.push(npc());
    let mut e = Engine::new(s).unwrap();
    e.advance(120).unwrap();
    let p = e.simulation.positions()["crate"];
    assert!((p[1] - 0.5).abs() < 0.02);
    assert!(
        e.simulation
            .contacts()
            .contains(&("crate".into(), "floor".into()))
    );
    replay(&e);
}

#[test]
fn wall_blocks_and_diagonal_intention_slides_along_it() {
    let mut s = scene();
    s.entities
        .push(box_("wall", [0.25, 1.5, 0.], [0.5, 3., 14.], "fixed"));
    let mut e = Engine::new(s).unwrap();
    run(
        &mut e,
        Command::SetInput {
            input: Input {
                x: 1.,
                z: 1.,
                ..Default::default()
            },
        },
    );
    e.advance(100).unwrap();
    assert!(pos(&e)[0] < -0.29);
    assert!(pos(&e)[2] > 3.);
    let st = e.simulation.character_controllers["player"];
    assert!(st.collisions > 0);
    assert!(st.corrected_translation[0].abs() < st.desired_translation[0]);
    replay(&e);
}
#[test]
fn autostep_requires_forward_clearance() {
    for (min_width, climbs) in [(0.05, true), (0.4, false)] {
        let mut s = scene();
        step(&mut s, 0.2);
        s.entities.push(box_(
            "wall_after_step",
            [0.55, 1.5, 0.],
            [0.5, 3., 4.],
            "fixed",
        ));
        s.entities
            .iter_mut()
            .find(|e| e.id == "player")
            .unwrap()
            .character_controller
            .as_mut()
            .unwrap()
            .step = Some(Step {
            max_height: 0.3,
            min_width,
        });
        let mut e = Engine::new(s).unwrap();
        walk(&mut e, 100, 1.);
        assert_eq!(pos(&e)[1] > 0.95, climbs, "{min_width} {:?}", pos(&e));
        replay(&e);
    }
}

#[test]
fn footstep_contact_uses_controller_offset_without_inventing_another_surface() {
    for raised in [false, true] {
        let mut s = scene();
        let p = s.entities.iter_mut().find(|e| e.id == "player").unwrap();
        p.footsteps = Some(
            serde_json::from_value(
                json!({"surface":"floor","stride":0.5,"radius":5,"min_speed":0.1}),
            )
            .unwrap(),
        );
        if raised {
            p.character_controller.as_mut().unwrap().step = Some(Step {
                max_height: 0.3,
                min_width: 0.4,
            });
            step(&mut s, 0.2);
        }
        let mut e = Engine::new(s).unwrap();
        walk(&mut e, 60, 1.);
        let earlier = e.simulation.footsteps.total_emitted;
        assert!(earlier > 0);
        walk(&mut e, 40, 1.);
        if raised {
            assert_eq!(e.simulation.footsteps.total_emitted, earlier);
        } else {
            assert!(e.simulation.footsteps.total_emitted > earlier);
        }
        replay(&e);
    }
}

// Critical: an idle diagonal cuboid must not sink into a flat floor and silently
// disable its physical footsteps before a multiplayer client acquires control.
#[test]
fn diagonal_idle_contact_preserves_floor_and_real_footsteps_after_control() {
    for xz in [-23.25, 9.75, 11.25] {
        for y in [0.8, 0.81] {
            let mut s = RoomRecipe {
                width: 63.,
                depth: 63.,
                obstacles: 0,
                ..Default::default()
            }
            .generate()
            .unwrap();
            let p = s
                .entities
                .iter_mut()
                .find(|e| e.body == Some(BodyKind::Player))
                .unwrap();
            p.transform.position = [xz, y, xz];
            p.character_controller = Some(Config::default());
            p.footsteps = Some(
                serde_json::from_value(
                    json!({"surface":"floor","stride":0.5,"radius":6,"min_speed":0.1}),
                )
                .unwrap(),
            );
            let half = p.transform.size[1] * 0.5;
            let mut e = Engine::new(s).unwrap();
            for tick in 0usize..840 {
                if tick >= 240 && (tick - 240).is_multiple_of(120) {
                    run(
                        &mut e,
                        Command::SetInput {
                            input: Input {
                                x: if (tick / 120).is_multiple_of(2) {
                                    0.25
                                } else {
                                    -0.25
                                },
                                ..Default::default()
                            },
                        },
                    );
                }
                e.advance(1).unwrap();
                let state = e.simulation.character_controllers["player"];
                assert!(
                    pos(&e)[1] >= half - 0.003,
                    "xz={xz}, y={y}, tick={tick}, position={:?}, motor={state:?}",
                    pos(&e)
                );
                assert!(
                    state.grounded,
                    "Lost flat-floor contact: {xz}/{y}/{tick}: {state:?}"
                );
            }
            assert!(
                e.simulation.footsteps.total_emitted >= 15,
                "Missing physical steps after idle: {xz}/{y}"
            );
            replay(&e);
        }
    }
}
