use ignis::{
    engine::{Command, Engine, Replay, Request},
    mcp::Session,
    model::{RoomRecipe, Scene},
    physics::Input,
    service::Service,
};
use serde_json::json;

fn scene() -> Scene {
    RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap()
}
fn engine() -> Engine {
    Engine::new(scene()).unwrap()
}
fn execute(engine: &mut Engine, command: Command) {
    engine.execute(command.into()).unwrap();
}

#[test]
fn generation_is_seeded_and_has_stable_ids() {
    let recipe = RoomRecipe::default();
    let a = recipe.generate().unwrap();
    let b = recipe.generate().unwrap();
    assert_eq!(a, b);
    let c = RoomRecipe { seed: 43, ..recipe }.generate().unwrap();
    assert_ne!(a, c);
    assert_eq!(
        a.entities.iter().map(|e| &e.id).collect::<Vec<_>>(),
        c.entities.iter().map(|e| &e.id).collect::<Vec<_>>()
    );
}

#[test]
fn invalid_edit_and_stale_revision_are_atomic() {
    let mut e = engine();
    let before = e.snapshot();
    let mut entity = e.scene.entities[0].clone();
    entity.transform.size[0] = -1.;
    assert!(
        e.execute(
            Command::Upsert {
                entity: entity.into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    assert_eq!(
        e.execute(Request {
            expected_revision: Some(999),
            command: Command::Remove {
                id: "wall_north".into()
            }
        })
        .unwrap_err()
        .code,
        "revision_conflict"
    );
    assert_eq!(e.snapshot(), before);
}

#[test]
fn physics_blocks_player_and_reports_contact_without_editing_scene() {
    let mut e = engine();
    let original = e.scene.clone();
    execute(
        &mut e,
        Command::SetInput {
            input: Input {
                x: 0.,
                z: -1.,
                ..Default::default()
            },
        },
    );
    execute(&mut e, Command::Step { ticks: 600 });
    let p = e.simulation.positions()["player"];
    assert!(p[2] > -6.49 && p[2] < -6.3, "position {p:?}");
    assert!((0.7..0.9).contains(&p[1]), "height {p:?}");
    assert!(
        e.simulation
            .events
            .iter()
            .any(|ev| ev.kind == "contact_started"
                && [ev.a.as_str(), ev.b.as_str()].contains(&"wall_north")
                && [ev.a.as_str(), ev.b.as_str()].contains(&"player"))
    );
    assert_eq!(e.scene, original);
}

#[test]
fn missing_wall_fails_and_undo_repairs_the_same_scenario() {
    let mut e = engine();
    execute(
        &mut e,
        Command::Remove {
            id: "wall_north".into(),
        },
    );
    execute(
        &mut e,
        Command::SetInput {
            input: Input {
                x: 0.,
                z: -1.,
                ..Default::default()
            },
        },
    );
    execute(&mut e, Command::Step { ticks: 300 });
    assert!(e.simulation.positions()["player"][2] < -7.);
    execute(&mut e, Command::Undo);
    execute(
        &mut e,
        Command::SetInput {
            input: Input {
                x: 0.,
                z: -1.,
                ..Default::default()
            },
        },
    );
    execute(&mut e, Command::Step { ticks: 300 });
    assert!(e.simulation.positions()["player"][2] > -6.49);
    execute(&mut e, Command::Redo);
    assert!(!e.scene.entities.iter().any(|e| e.id == "wall_north"));
}

#[test]
fn replay_survives_json_roundtrip_and_detects_tampering() {
    let mut e = engine();
    for input in [
        Input {
            x: 1.,
            z: 0.,
            ..Default::default()
        },
        Input {
            x: -0.4,
            z: -1.,
            ..Default::default()
        },
        Input::default(),
    ] {
        execute(&mut e, Command::SetInput { input });
        execute(&mut e, Command::Step { ticks: 100 });
    }
    let mut replay: Replay =
        serde_json::from_slice(&serde_json::to_vec(&e.replay()).unwrap()).unwrap();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    replay.expected_positions.get_mut("player").unwrap()[0] += 1.;
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], false);
}

#[test]
fn explicit_steps_require_pause_and_validate_entire_request() {
    let mut e = engine();
    execute(&mut e, Command::Play);
    assert!(e.execute(Command::Step { ticks: 1 }.into()).is_err());
    execute(&mut e, Command::Pause);
    assert!(e.execute(Command::Step { ticks: 4000 }.into()).is_err());
    assert_eq!(e.simulation.tick, 0);
    assert!(
        e.execute(
            Command::SetInput {
                input: Input {
                    x: f32::NAN,
                    z: 0.,
                    ..Default::default()
                }
            }
            .into()
        )
        .is_err()
    );
}

#[test]
fn persistence_saves_authoring_state_and_rejects_invalid_load() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Service::new(dir.path(), scene()).unwrap();
    s.execute(Command::Step { ticks: 60 }.into()).unwrap();
    s.execute(
        Command::SaveScene {
            path: "scene.json".into(),
        }
        .into(),
    )
    .unwrap();
    let saved: Scene =
        serde_json::from_slice(&std::fs::read(dir.path().join("scene.json")).unwrap()).unwrap();
    assert_eq!(saved, s.engine.scene);
    let before = s.engine.snapshot();
    std::fs::write(dir.path().join("bad.json"), r#"{"version":999}"#).unwrap();
    assert!(
        s.execute(
            Command::LoadScene {
                path: "bad.json".into()
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(s.engine.snapshot(), before);
    s.execute(
        Command::LoadScene {
            path: "scene.json".into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.engine.simulation.tick, 0);
    assert!(s.resolve("../escape.json").is_err());
    assert!(s.resolve("/tmp/escape.json").is_err());
}

#[cfg(unix)]
#[test]
fn project_paths_cannot_escape_via_symlinks() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(external.path(), root.path().join("outside")).unwrap();
    let mut s = Service::new(root.path(), scene()).unwrap();
    assert!(
        s.execute(
            Command::SaveScene {
                path: "outside/new/scene.json".into()
            }
            .into()
        )
        .is_err()
    );
    assert!(!external.path().join("new").exists());
}

#[test]
fn mcp_lifecycle_schema_and_structured_errors() {
    let root = tempfile::tempdir().unwrap();
    let mut s = Service::new(root.path(), scene()).unwrap();
    let mut session = Session::default();
    let call =
        |id, method, params| json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
    assert!(
        session
            .handle(call(1, "tools/list", json!({})), &mut s)
            .unwrap()
            .get("error")
            .is_some()
    );
    assert_eq!(session.handle(call(2,"initialize",json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}})),&mut s).unwrap()["result"]["protocolVersion"],"2025-11-25");
    assert!(
        session
            .handle(
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                &mut s
            )
            .is_none()
    );
    let list = session
        .handle(call(3, "tools/list", json!({})), &mut s)
        .unwrap();
    let schema = &list["result"]["tools"][0]["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert!(schema.to_string().contains("set_input"));
    assert!(schema.to_string().contains("capture"));
    let result = session
        .handle(
            call(
                4,
                "tools/call",
                json!({"name":"engine_command","arguments":{"command":{"op":"step","ticks":0}}}),
            ),
            &mut s,
        )
        .unwrap();
    assert_eq!(result["result"]["isError"], true);
    assert_eq!(s.engine.simulation.tick, 0);
}

#[test]
fn invalid_scene_contracts_are_rejected() {
    let mut s = scene();
    s.entities.push(s.entities[0].clone());
    assert!(s.validate().is_err());
    let mut s = scene();
    s.version = 999;
    assert_eq!(s.validate().unwrap_err().code, "unsupported_version");
    let mut s = scene();
    s.camera.target = s.camera.position;
    assert!(s.validate().is_err());
    let mut s = scene();
    s.light.direction = [0.; 3];
    assert!(s.validate().is_err());
}

#[test]
fn vision_guides_are_session_only_and_follow_real_npc_state() {
    let scene: ignis::model::Scene =
        serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
    let mut a = Engine::new(scene.clone()).unwrap();
    let mut b = Engine::new(scene).unwrap();
    assert!(a.render_frame().vision_guides.is_empty());
    let before = serde_json::to_value(a.replay()).unwrap();
    a.execute(
        Command::SetDebugView {
            npc_vision: Some(true),
            npc_hearing: None,
            noise_events: None,
            npc_routes: None,
        }
        .into(),
    )
    .unwrap();
    assert_eq!(a.view_revision, 1);
    a.execute(
        Command::SetDebugView {
            npc_vision: Some(true),
            npc_hearing: None,
            noise_events: None,
            npc_routes: None,
        }
        .into(),
    )
    .unwrap();
    assert_eq!(a.view_revision, 1);
    assert_eq!(serde_json::to_value(a.replay()).unwrap(), before);
    assert_eq!(a.revision, 0);
    let initial = a.render_frame().vision_guides[0];
    assert_eq!(initial.color, [0.1, 0.65, 1.]);
    assert_eq!(initial.fov_degrees, 120.);
    assert_eq!(initial.range, 6.);
    assert_eq!(initial.facing_degrees, 0.);
    for e in [&mut a, &mut b] {
        e.execute(
            Command::EmitNoise {
                origin: ignis::noise::NoiseOrigin::Entity {
                    id: "player".into(),
                },
                radius: 12.,
            }
            .into(),
        )
        .unwrap();
        e.advance(40).unwrap();
    }
    assert_eq!(
        serde_json::to_value(a.replay()).unwrap(),
        serde_json::to_value(b.replay()).unwrap()
    );
    let guide = a.render_frame().vision_guides[0];
    assert_eq!(
        guide.facing_degrees,
        a.simulation.npcs["hunter"].facing_degrees
    );
    assert_eq!(guide.position, a.simulation.position("hunter").unwrap());
    assert_eq!(guide.color, [1., 0.65, 0.05]);
    a.simulation
        .npcs
        .get_mut("hunter")
        .unwrap()
        .observed_this_tick = true;
    assert_eq!(a.render_frame().vision_guides[0].color, [0.1, 1., 0.2]);
    a.reset();
    assert!(a.debug_view.npc_vision);
    assert_eq!(a.render_frame().vision_guides[0], initial);
    let mut hunter = a
        .scene
        .entities
        .iter()
        .find(|e| e.id == "hunter")
        .unwrap()
        .clone();
    hunter.npc.as_mut().unwrap().awareness = Some(ignis::model::Awareness::Omniscient);
    a.execute(
        Command::Upsert {
            entity: hunter.clone().into(),
        }
        .into(),
    )
    .unwrap();
    let omni = a.render_frame().vision_guides[0];
    assert!(!omni.directional);
    assert_eq!(omni.range, 0.);
    assert_eq!(omni.color, [1., 0.15, 0.85]);
    hunter.npc.as_mut().unwrap().enabled = false;
    a.execute(
        Command::Upsert {
            entity: hunter.into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(a.render_frame().vision_guides[0].color, [0.4; 3]);
    a.execute(
        Command::Remove {
            id: "hunter".into(),
        }
        .into(),
    )
    .unwrap();
    assert!(a.render_frame().vision_guides.is_empty());
    a.execute(Command::Undo.into()).unwrap();
    assert_eq!(a.render_frame().vision_guides.len(), 1);
    a.execute(
        Command::SetDebugView {
            npc_vision: Some(false),
            npc_hearing: None,
            noise_events: None,
            npc_routes: None,
        }
        .into(),
    )
    .unwrap();
    assert!(a.render_frame().vision_guides.is_empty());
}

#[test]
fn acoustic_guides_patch_session_view_and_bound_frozen_noise_history() {
    let scene = serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
    let mut e = Engine::new(scene).unwrap();
    let patch = |e: &mut Engine, fields: serde_json::Value| {
        let c: Command = serde_json::from_value(fields).unwrap();
        e.execute(c.into()).unwrap();
    };
    let before = serde_json::to_value(e.replay()).unwrap();
    patch(
        &mut e,
        serde_json::json!({"op":"set_debug_view","npc_hearing":true,"noise_events":true}),
    );
    assert!(!e.debug_view.npc_vision);
    let revision = e.view_revision;
    patch(&mut e, serde_json::json!({"op":"set_debug_view"}));
    assert_eq!(e.view_revision, revision);
    assert_eq!(serde_json::to_value(e.replay()).unwrap(), before);
    let mut frame = e.render_frame();
    assert_eq!(frame.hearing_guides.len(), 1);
    assert_eq!(frame.hearing_guides[0].radius, 12.);
    for _ in 0..16 {
        e.execute(
            Command::EmitNoise {
                origin: ignis::noise::NoiseOrigin::Position {
                    position: [1., 0.8, 2.],
                },
                radius: 3.,
            }
            .into(),
        )
        .unwrap();
    }
    assert_eq!(e.view_revision, revision + 16);
    e.update_render_frame(&mut frame);
    assert_eq!(frame.noise_guides.len(), 16);
    assert_eq!(frame.noise_guides[0].center, [1., 0.8, 2.]);
    let capacity = frame.noise_guides.capacity();
    e.advance(1).unwrap();
    e.execute(
        Command::EmitNoise {
            origin: ignis::noise::NoiseOrigin::Position {
                position: [-1., 0.8, 2.],
            },
            radius: 4.,
        }
        .into(),
    )
    .unwrap();
    e.update_render_frame(&mut frame);
    assert_eq!(frame.noise_guides.len(), 16);
    assert_eq!(frame.noise_guides[0].center, [-1., 0.8, 2.]);
    assert_eq!(frame.noise_guides.capacity(), capacity);
    e.advance(89).unwrap();
    e.update_render_frame(&mut frame);
    assert_eq!(frame.noise_guides.len(), 1);
    e.advance(1).unwrap();
    e.update_render_frame(&mut frame);
    assert!(frame.noise_guides.is_empty());
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    e.reset();
    assert!(e.debug_view.npc_hearing && e.debug_view.noise_events);
    patch(
        &mut e,
        serde_json::json!({"op":"set_debug_view","npc_vision":true}),
    );
    assert!(e.debug_view.npc_hearing && e.debug_view.noise_events && e.debug_view.npc_vision);
    let mut hunter = e
        .scene
        .entities
        .iter()
        .find(|x| x.id == "hunter")
        .unwrap()
        .clone();
    hunter.npc.as_mut().unwrap().enabled = false;
    e.execute(
        Command::Upsert {
            entity: hunter.into(),
        }
        .into(),
    )
    .unwrap();
    assert!(e.render_frame().hearing_guides.is_empty());
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.render_frame().hearing_guides.len(), 1);
}

#[test]
fn route_guides_are_session_only_bounded_and_never_query_hidden_targets() {
    use ignis::{
        debug_routes::MAX_PATH_SEGMENTS,
        patrol::{Route, Traversal},
    };
    let mut scene: Scene =
        serde_json::from_str(include_str!("../examples/patrol.scene.json")).unwrap();
    let hunter = scene
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap();
    let patrol = hunter.npc.as_mut().unwrap().patrol.as_mut().unwrap();
    patrol.route = Route::Ring {
        center: [0., 3.],
        radius: 1.,
        points: 16,
        seed: 0,
    };
    patrol.traversal = Traversal::Loop;
    let hunter = hunter.clone();
    for i in 1..8 {
        let mut other = hunter.clone();
        other.id = format!("hunter_{i}");
        scene.entities.push(other);
    }
    let mut e = Engine::new(scene).unwrap();
    let before = serde_json::to_value(e.replay()).unwrap();
    let revision = e.revision;
    e.execute(
        Command::SetDebugView {
            npc_vision: None,
            npc_hearing: None,
            noise_events: None,
            npc_routes: Some(true),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(serde_json::to_value(e.replay()).unwrap(), before);
    assert_eq!(e.revision, revision);
    let mut frame = e.render_frame();
    assert_eq!(frame.route_guides.len(), 8 * (16 + 32));
    for nav in e.simulation.navigation.values_mut() {
        nav.path = vec![[1., 0.8, 2.]; MAX_PATH_SEGMENTS + 1];
        nav.target_position = Some([3., 0.8, 2.]);
        nav.status = "unreachable".into();
    }
    e.update_render_frame(&mut frame);
    assert_eq!(
        frame.route_guides.len(),
        8 * (16 + 32 + MAX_PATH_SEGMENTS + 2)
    );
    assert_eq!(frame.truncated_navigation_paths, 8);
    assert!(frame.route_guides.iter().any(|g| g.color == [1., 0.1, 0.1]));
    let capacity = frame.route_guides.capacity();
    let guides = frame.route_guides.clone();
    // Changing only the hidden target's authored transform cannot alter any route guide.
    e.scene
        .entities
        .iter_mut()
        .find(|e| e.id == "player")
        .unwrap()
        .transform
        .position = [5., 0.8, -4.];
    e.update_render_frame(&mut frame);
    assert_eq!(frame.route_guides, guides);
    assert_eq!(frame.route_guides.capacity(), capacity);
    e.execute(
        Command::SetDebugView {
            npc_vision: None,
            npc_hearing: None,
            noise_events: None,
            npc_routes: Some(false),
        }
        .into(),
    )
    .unwrap();
    e.update_render_frame(&mut frame);
    assert!(frame.route_guides.is_empty());
    assert_eq!(frame.truncated_navigation_paths, 0);
    assert_eq!(frame.route_guides.capacity(), capacity);
}
