use ignis::{
    engine::{Command, DebugView, Engine, MAX_TICKS, Replay, Request},
    model::{MovementMode, Scene},
    noise::NoiseOrigin,
    physics::Input,
    service::Service,
};
use serde_json::{Value, json};

fn scene() -> Scene {
    serde_json::from_str(include_str!("../examples/patrol.scene.json")).unwrap()
}
fn noise(engine: &mut Engine, x: f32) {
    engine
        .execute(
            Command::EmitNoise {
                origin: NoiseOrigin::Position {
                    position: [x, 0.8, 4.],
                },
                radius: 12.,
            }
            .into(),
        )
        .unwrap();
}
fn same_simulation(a: &Value, b: &Value) {
    for key in [
        "tick",
        "scene",
        "runtime_positions",
        "navigation",
        "npcs",
        "contacts",
        "footsteps",
        "patrols",
        "generated_footsteps",
        "recorded_ticks",
        "recorded_noises",
        "pending_noises",
    ] {
        assert_eq!(a[key], b[key], "{key}");
    }
}
#[test]
fn prefix_preserves_pending_cues_and_regenerates_physics_and_footsteps() {
    let mut live = Engine::new(scene()).unwrap();
    noise(&mut live, 0.);
    let zero = live.snapshot();
    live.advance(30).unwrap();
    noise(&mut live, 1.);
    let thirty = live.snapshot();
    live.input = Input {
        x: 1.,
        z: 0.,
        mode: MovementMode::Run,
    };
    live.advance(1).unwrap();
    let thirty_one = live.snapshot();
    live.advance(19).unwrap();
    noise(&mut live, 2.);
    let fifty = live.snapshot();
    assert!(live.simulation.footsteps.total_emitted > 0);
    let replay = live.replay();
    for (tick, expected) in [(0, zero), (30, thirty), (31, thirty_one), (50, fifty)] {
        let prefix = Engine::replay_at(&replay, Some(tick)).unwrap();
        same_simulation(&prefix.snapshot(), &expected);
        assert_eq!(
            prefix.input,
            if tick > 30 {
                live.input
            } else {
                Input::default()
            }
        );
        assert!(!prefix.playing);
        assert_eq!(
            Engine::verify_replay(&prefix.replay()).unwrap()["matched"],
            true
        );
    }
    let end = Engine::replay_at(&replay, None).unwrap();
    assert_eq!(
        serde_json::to_value(end.replay()).unwrap(),
        serde_json::to_value(replay).unwrap()
    );
}
#[test]
fn whole_document_is_validated_even_when_requesting_zero() {
    let mut live = Engine::new(scene()).unwrap();
    live.advance(2).unwrap();
    noise(&mut live, 0.);
    let replay = live.replay();
    assert!(Engine::replay_at(&replay, Some(3)).is_err());
    assert!(Engine::replay_at(&replay, Some(u64::MAX)).is_err());
    let mut bad = replay.clone();
    bad.build_id.push('x');
    assert!(Engine::replay_at(&bad, Some(0)).is_err());
    let mut bad = replay.clone();
    bad.inputs[1].x = f32::NAN;
    assert!(Engine::replay_at(&bad, Some(0)).is_err());
    let mut bad = replay.clone();
    bad.noises[0].tick = 3;
    assert!(Engine::replay_at(&bad, Some(0)).is_err());
    let mut bad = replay.clone();
    bad.noises[0].emitter_index = Some(0);
    assert!(Engine::replay_at(&bad, Some(0)).is_err());
    let mut bad = replay.clone();
    bad.scene.entities[0].transform.size[0] = -1.;
    assert!(Engine::replay_at(&bad, Some(0)).is_err());
    let mut limit = replay;
    limit.inputs.resize(MAX_TICKS, Input::default());
    assert!(Engine::replay_at(&limit, Some(0)).is_ok());
    limit.inputs.push(Input::default());
    assert!(Engine::replay_at(&limit, Some(0)).is_err());
}
#[test]
fn service_distinguishes_prefix_from_verified_final_and_preserves_live_session() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::new(dir.path(), scene()).unwrap();
    service.engine.advance(30).unwrap();
    noise(&mut service.engine, 0.);
    service
        .execute(
            Command::SaveReplay {
                path: "sample.json".into(),
            }
            .into(),
        )
        .unwrap();
    service.engine.replace(scene()).unwrap();
    service.engine.advance(90).unwrap();
    service.engine.playing = true;
    service.engine.debug_view = DebugView {
        npc_vision: true,
        npc_hearing: true,
        noise_events: true,
        npc_routes: true,
    };
    let before = service.engine.snapshot();
    let before_replay = serde_json::to_value(service.engine.replay()).unwrap();
    let inspect = |service: &mut Service, tick| {
        service
            .execute(
                Command::InspectReplay {
                    path: "sample.json".into(),
                    tick,
                }
                .into(),
            )
            .unwrap()
            .data
    };
    let partial = inspect(&mut service, Some(0));
    assert_eq!(partial["replay"]["at_end"], false);
    assert!(partial["replay"]["final_verification"].is_null());
    let full = inspect(&mut service, None);
    assert_eq!(full["replay"]["final_verification"]["matched"], true);
    assert_eq!(full["snapshot"]["tick"], 30);
    assert_eq!(full["snapshot"]["pending_noises"], 1);
    assert_eq!(full["noises"]["noises"][0]["tick"], 30);
    assert_eq!(full["snapshot"]["debug_view"], before["debug_view"]);
    assert_eq!(full["snapshot"]["playing"], false);
    assert_eq!(full["snapshot"]["revision"], 0);
    assert_eq!(inspect(&mut service, Some(30)), full);
    let path = dir.path().join("sample.json");
    let mut tampered: Replay = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    tampered.expected_positions.get_mut("player").unwrap()[0] += 10.;
    std::fs::write(&path, serde_json::to_vec(&tampered).unwrap()).unwrap();
    assert_eq!(
        inspect(&mut service, None)["replay"]["final_verification"]["matched"],
        false
    );
    assert!(inspect(&mut service, Some(0))["replay"]["final_verification"].is_null());
    assert_eq!(service.engine.snapshot(), before);
    assert_eq!(
        serde_json::to_value(service.engine.replay()).unwrap(),
        before_replay
    );
}
#[test]
fn capture_rejects_aliases_dimensions_and_bad_tick_without_creating_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::new(dir.path(), scene()).unwrap();
    service
        .execute(
            Command::SaveReplay {
                path: "sample.json".into(),
            }
            .into(),
        )
        .unwrap();
    let bytes = std::fs::read(dir.path().join("sample.json")).unwrap();
    let before = service.engine.snapshot();
    let capture = |service: &mut Service, source: &str, dest: &str, tick, width| {
        service.execute(
            Command::CaptureReplay {
                replay_path: source.into(),
                path: dest.into(),
                tick,
                width,
                height: 128,
            }
            .into(),
        )
    };
    assert!(capture(&mut service, "sample.json", "sample.json", None, 128).is_err());
    assert!(capture(&mut service, "sample.json", "../escape.png", None, 128).is_err());
    assert!(capture(&mut service, "sample.json", "out.png", None, 63).is_err());
    assert!(capture(&mut service, "sample.json", "out.png", None, 2049).is_err());
    assert!(capture(&mut service, "sample.json", "out.png", Some(1), 128).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            dir.path().join("sample.json"),
            dir.path().join("source-link.json"),
        )
        .unwrap();
        std::os::unix::fs::symlink(dir.path(), dir.path().join("alias")).unwrap();
        assert!(capture(&mut service, "source-link.json", "sample.json", None, 128).is_err());
        assert!(capture(&mut service, "sample.json", "alias/sample.json", None, 128).is_err());
    }
    assert!(!dir.path().join("out.png").exists());
    assert_eq!(
        std::fs::read(dir.path().join("sample.json")).unwrap(),
        bytes
    );
    assert_eq!(service.engine.snapshot(), before);
    assert!(
        serde_json::from_value::<Request>(
            json!({"command":{"op":"inspect_replay","path":"sample.json","tick":-1}})
        )
        .is_err()
    );
}
#[test]
fn empty_replay_has_verified_final_at_zero_without_consuming_pending_noise() {
    let dir = tempfile::tempdir().unwrap();
    let mut service = Service::new(dir.path(), scene()).unwrap();
    noise(&mut service.engine, 0.);
    service
        .execute(
            Command::SaveReplay {
                path: "zero.json".into(),
            }
            .into(),
        )
        .unwrap();
    let r = service
        .execute(
            Command::InspectReplay {
                path: "zero.json".into(),
                tick: None,
            }
            .into(),
        )
        .unwrap()
        .data;
    assert_eq!(r["replay"]["at_end"], true);
    assert_eq!(r["replay"]["final_verification"]["matched"], true);
    assert_eq!(r["snapshot"]["pending_noises"], 1);
    assert!(r["snapshot"]["npcs"]["hunter"]["memory"].is_null());
}
