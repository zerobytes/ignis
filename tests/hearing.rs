use ignis::{
    engine::{Command, Engine},
    model::{Awareness, Scene},
    noise::{Noise, NoiseOrigin},
    perception::{self, Phase, Source},
};
use std::collections::BTreeMap;

fn scene() -> Scene {
    let mut s: serde_json::Value =
        serde_json::from_str(include_str!("../examples/search.scene.json")).unwrap();
    s["version"] = 6.into();
    for e in s["entities"].as_array_mut().unwrap() {
        match e["id"].as_str().unwrap() {
            "player" => e["transform"]["position"] = serde_json::json!([0, 0.85, -2]),
            "hunter" => {
                e["transform"]["position"] = serde_json::json!([0, 0.85, 2]);
                e["npc"]["awareness"]["hearing"] = serde_json::json!({"range":20});
                e["npc"]["awareness"]["vision_range"] = 20.into();
                e["npc"]["awareness"]["fov_degrees"] = 360.into();
            }
            _ => (),
        }
    }
    serde_json::from_value(s).unwrap()
}
fn emit(e: &mut Engine, p: [f32; 3], radius: f32) {
    e.execute(
        Command::EmitNoise {
            origin: NoiseOrigin::Position { position: p },
            radius,
        }
        .into(),
    )
    .unwrap();
}
fn hear(s: Scene, noises: &[Noise]) -> ignis::perception::NpcState {
    let mut e = Engine::new(s.clone()).unwrap();
    let positions = e.simulation.positions();
    perception::update_with_noises(
        &s,
        &positions,
        0,
        &e.simulation.navigation,
        &mut e.simulation.npcs,
        &mut BTreeMap::new(),
        noises,
    );
    e.simulation.npcs.remove("hunter").unwrap()
}
fn noise(id: u64, x: f32, radius: f32) -> Noise {
    Noise {
        id,
        tick: 0,
        position: [x, 0.85, -2.],
        radius,
        emitter_index: None,
        emitter_instance: None,
    }
}

#[test]
fn hearing_is_opt_in_and_respects_range_transmission_and_visual_priority() {
    let s = scene();
    assert_eq!(hear(s.clone(), &[]).phase, Phase::Unaware);
    let heard = hear(s.clone(), &[noise(0, 0., 12.)]);
    assert_eq!(heard.phase, Phase::Investigating);
    assert!(heard.heard_this_tick && !heard.observed_this_tick);
    assert_eq!(heard.memory.unwrap().source, Source::Sound);
    assert_eq!(hear(s.clone(), &[noise(0, 0., 3.)]).phase, Phase::Unaware);
    let mut disabled = s.clone();
    if let Some(Awareness::Perceptive { hearing, .. }) = &mut disabled
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        *hearing = None;
    }
    assert_eq!(hear(disabled, &[noise(0, 0., 12.)]).phase, Phase::Unaware);
    for (transmission, radius, expected) in [
        (0., 100., false),
        (0.25, 12., false),
        (0.5, 12., true),
        (1., 12., true),
    ] {
        let mut s = s.clone();
        let barrier = s.entities.iter_mut().find(|e| e.id == "barrier").unwrap();
        barrier.sound_transmission = Some(transmission);
        // Acoustic behavior also works for a visual-only object.
        barrier.body = None;
        barrier.blocks_vision = Some(true);
        assert_eq!(hear(s, &[noise(0, 0., radius)]).heard_this_tick, expected);
    }
    let mut limited = s.clone();
    if let Some(Awareness::Perceptive {
        hearing: Some(h), ..
    }) = &mut limited
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        h.range = 3.;
    }
    assert!(!hear(limited, &[noise(0, 0., 100.)]).heard_this_tick);
    let mut visible = s;
    visible
        .entities
        .iter_mut()
        .find(|e| e.id == "barrier")
        .unwrap()
        .blocks_vision = Some(false);
    let state = hear(visible, &[noise(0, 4., 100.)]);
    assert!(state.observed_this_tick && !state.heard_this_tick);
    assert_eq!(state.phase, Phase::Pursuing);
    assert_eq!(state.memory.unwrap().source, Source::Vision);
}

#[test]
fn strongest_noise_wins_and_equal_strength_uses_command_order() {
    let weak = noise(0, 0., 6.);
    let strong = noise(1, 2., 12.);
    let heard = hear(scene(), &[strong, weak]);
    assert_eq!(heard.memory.unwrap().noise_id, Some(1));
    let a = noise(0, -2., 12.);
    let b = noise(1, 2., 12.);
    assert_eq!(hear(scene(), &[a, b]).memory.unwrap().noise_id, Some(1));
    // Two independent walls compose; not coupled to blocks_vision.
    let mut s = scene();
    let wall = s.entities.iter_mut().find(|e| e.id == "barrier").unwrap();
    wall.sound_transmission = Some(0.5);
    let mut second = wall.clone();
    second.id = "second".into();
    second.transform.position[2] = -1.;
    s.entities.push(second);
    assert!(!hear(s, &[noise(0, 0., 12.)]).heard_this_tick);
}

#[test]
fn hidden_movement_never_moves_a_sound_memory_or_changes_its_goal() {
    let s = scene();
    let e = Engine::new(s.clone()).unwrap();
    let mut pa = e.simulation.positions();
    let mut pb = pa.clone();
    let mut a = e.simulation.npcs.clone();
    let mut b = a.clone();
    let mut ga = BTreeMap::new();
    let mut gb = BTreeMap::new();
    let cue = [noise(0, 0., 12.)];
    for tick in 0..60 {
        pa.insert("player".into(), [-1., 0.85, -3.]);
        pb.insert("player".into(), [1. + tick as f32 * 0.01, 0.85, -4.]);
        let noises = if tick == 0 { &cue[..] } else { &[] };
        perception::update_with_noises(
            &s,
            &pa,
            tick,
            &e.simulation.navigation,
            &mut a,
            &mut ga,
            noises,
        );
        perception::update_with_noises(
            &s,
            &pb,
            tick,
            &e.simulation.navigation,
            &mut b,
            &mut gb,
            noises,
        );
        assert_eq!(a, b);
        assert_eq!(
            ga["hunter"].as_ref().unwrap().position,
            gb["hunter"].as_ref().unwrap().position
        );
        assert_eq!(
            a["hunter"].memory.as_ref().unwrap().position,
            cue[0].position
        );
        assert_eq!(a["hunter"].memory.as_ref().unwrap().tick, 0);
    }
}

#[test]
fn noise_command_freezes_origin_replays_pending_and_consumed_and_resets() {
    let mut e = Engine::new(scene()).unwrap();
    e.execute(
        Command::EmitNoise {
            origin: NoiseOrigin::Entity {
                id: "player".into(),
            },
            radius: 12.,
        }
        .into(),
    )
    .unwrap();
    let before = e.replay();
    assert_eq!(before.noises[0].position, [0., 0.85, -2.]);
    assert!(
        Engine::verify_replay(&before).unwrap()["matched"]
            .as_bool()
            .unwrap()
    );
    e.advance(1).unwrap();
    let remembered = e.simulation.npcs["hunter"].memory;
    e.execute(
        Command::SetInput {
            input: ignis::physics::Input {
                x: 1.,
                z: 0.,
                ..Default::default()
            },
        }
        .into(),
    )
    .unwrap();
    e.advance(15).unwrap();
    assert_eq!(e.simulation.npcs["hunter"].memory, remembered);
    emit(&mut e, [3., 0.8, -3.], 15.);
    let replay = e.replay();
    let replay = serde_json::from_str(&serde_json::to_string(&replay).unwrap()).unwrap();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    e.advance(1).unwrap();
    assert_eq!(
        e.simulation.npcs["hunter"]
            .memory
            .as_ref()
            .unwrap()
            .noise_id,
        Some(1)
    );
    let replay = e.replay();
    assert_eq!(Engine::verify_replay(&replay).unwrap()["matched"], true);
    let mut tampered = replay.clone();
    tampered.noises[1].position[0] = -3.;
    assert_eq!(Engine::verify_replay(&tampered).unwrap()["matched"], false);
    let mut invalid = replay;
    invalid.noises[1].tick = 0;
    invalid.noises[0].tick = 1;
    assert!(Engine::verify_replay(&invalid).is_err());
    e.reset();
    assert!(e.replay().noises.is_empty());
    assert!(e.simulation.npcs["hunter"].memory.is_none());
}

#[test]
fn noise_limits_and_invalid_config_are_atomic_and_legacy_stays_silent() {
    let mut e = Engine::new(scene()).unwrap();
    for radius in [0., 101., f32::NAN] {
        let before = e.snapshot();
        assert!(
            e.execute(
                Command::EmitNoise {
                    origin: NoiseOrigin::Position {
                        position: [0., 1., 0.]
                    },
                    radius
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
    }
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::EmitNoise {
                origin: NoiseOrigin::Entity {
                    id: "missing".into()
                },
                radius: 12.
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    for _ in 0..16 {
        emit(&mut e, [0., 1., 0.], 12.);
    }
    let before = e.snapshot();
    assert!(
        e.execute(
            Command::EmitNoise {
                origin: NoiseOrigin::Position {
                    position: [0., 1., 0.]
                },
                radius: 12.
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    assert!(e.advance(0).is_err());
    assert_eq!(e.snapshot(), before);
    e.advance(1).unwrap();
    emit(&mut e, [0., 1., 0.], 12.);
    let mut bad = e
        .scene
        .entities
        .iter()
        .find(|e| e.id == "hunter")
        .unwrap()
        .clone();
    if let Some(Awareness::Perceptive {
        hearing: Some(h), ..
    }) = &mut bad.npc.as_mut().unwrap().awareness
    {
        h.range = 101.;
    }
    let before = e.snapshot();
    assert!(
        e.execute(Command::Upsert { entity: bad.into() }.into())
            .is_err()
    );
    assert_eq!(e.snapshot(), before);
    let mut old = scene();
    old.version = 5;
    assert!(Engine::new(old.clone()).is_err());
    if let Some(Awareness::Perceptive { hearing, .. }) = &mut old
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        *hearing = None;
    }
    let legacy = Engine::new(old).unwrap();
    assert_eq!(legacy.scene.version, ignis::model::SCENE_VERSION);
    let mut invalid = legacy.scene.clone();
    invalid.entities[0].sound_transmission = Some(-0.1);
    assert!(invalid.validate().is_err());
    // Exercise the live total budget too, not only malicious replay validation.
    e.reset();
    for _ in 0..256 {
        for _ in 0..16 {
            emit(&mut e, [0., 1., 0.], 12.);
        }
        e.advance(1).unwrap();
    }
    let full = e.snapshot();
    assert!(
        e.execute(
            Command::EmitNoise {
                origin: NoiseOrigin::Position {
                    position: [0., 1., 0.]
                },
                radius: 12.
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.snapshot(), full);
    assert_eq!(e.replay().noises.len(), 4096);
    let mut replay = e.replay();
    replay.noises = vec![noise(0, 0., 12.); 4097];
    assert!(Engine::verify_replay(&replay).is_err());
}

#[test]
fn sound_investigation_expires_and_real_sight_replaces_the_cue() {
    let mut s = scene();
    if let Some(Awareness::Perceptive { memory_ticks, .. }) = &mut s
        .entities
        .iter_mut()
        .find(|e| e.id == "hunter")
        .unwrap()
        .npc
        .as_mut()
        .unwrap()
        .awareness
    {
        *memory_ticks = Some(3);
    }
    let mut e = Engine::new(s.clone()).unwrap();
    emit(&mut e, [0., 0.85, -2.], 12.);
    e.advance(4).unwrap();
    assert_eq!(e.simulation.npcs["hunter"].reason, "memory_expired");
    let mut e = Engine::new(s.clone()).unwrap();
    let mut positions = e.simulation.positions();
    let mut goals = BTreeMap::new();
    perception::update_with_noises(
        &s,
        &positions,
        0,
        &e.simulation.navigation,
        &mut e.simulation.npcs,
        &mut goals,
        &[noise(0, 0., 12.)],
    );
    positions.insert("player".into(), [2., 0.85, 2.]);
    perception::update_with_noises(
        &s,
        &positions,
        1,
        &e.simulation.navigation,
        &mut e.simulation.npcs,
        &mut goals,
        &[],
    );
    let state = &e.simulation.npcs["hunter"];
    assert_eq!(state.phase, Phase::Pursuing);
    assert_eq!(state.memory.as_ref().unwrap().source, Source::Vision);
    assert_eq!(state.memory.as_ref().unwrap().noise_id, None);
}

#[test]
fn sound_behind_npc_recovers_wall_contact_before_visual_acquisition() {
    let mut s: Scene =
        serde_json::from_str(include_str!("../examples/hearing.scene.json")).unwrap();
    for entity in &mut s.entities {
        match entity.id.as_str() {
            "hunter" => {
                entity.transform.position = [3.3489742, 0.79893064, 0.43535045];
                if let Some(Awareness::Perceptive {
                    initial_facing_degrees,
                    ..
                }) = &mut entity.npc.as_mut().unwrap().awareness
                {
                    *initial_facing_degrees = 81.55771;
                }
            }
            "player" => entity.transform.position = [2.2461648, 0.7989311, 0.6489972],
            _ => (),
        }
    }
    let mut e = Engine::new(s).unwrap();
    e.advance(1).unwrap();
    assert_eq!(e.simulation.npcs["hunter"].phase, Phase::Unaware);
    emit(&mut e, [2.2461648, 0.7989311, 0.6489972], 12.);
    e.advance(1).unwrap();
    assert!(e.simulation.npcs["hunter"].heard_this_tick);
    assert_eq!(e.simulation.navigation["hunter"].status, "recovering");
    let mut acquired = false;
    for _ in 0..120 {
        e.advance(1).unwrap();
        let state = &e.simulation.npcs["hunter"];
        let p = e.simulation.positions()["hunter"];
        assert!(p[0] >= 3.345 || p[2].abs() >= 0.645, "crossed wall: {p:?}");
        assert_ne!(state.phase, Phase::Lost);
        assert_eq!(state.skipped_search_points, 0);
        acquired |= state.observed_this_tick;
    }
    assert!(acquired, "{:?}", e.snapshot());
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
}
