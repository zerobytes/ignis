use ignis::{
    engine::{Command, Engine},
    model::Scene,
    network::replica::{Frame, Replica},
    npc_presentation::{Phase, pcm},
};
fn call(e: &mut Engine, c: Command) {
    e.execute(c.into()).unwrap();
}
#[test]
fn public_avatar_life_reuse_codec_and_audio_preserve_identity_and_replay() {
    let scene: Scene =
        serde_json::from_str(include_str!("../examples/npc-presentation.scene.json")).unwrap();
    let mut e = Engine::new(scene).unwrap();
    assert_eq!(e.simulation.presentation.actors["a"].variant, Some(0));
    assert_eq!(e.simulation.presentation.actors["b"].variant, Some(1));
    let authored = e.scene.meshes.clone();
    let mut frame = e.render_frame();
    assert_eq!(frame.meshes.drawn_avatars.len(), 3);
    assert_eq!(frame.meshes.vertices.len(), 54);
    let vertices = frame.meshes.vertices.clone();
    e.update_render_frame(&mut frame);
    assert_eq!(frame.meshes.vertices, vertices);
    call(
        &mut e,
        Command::OrderNpc {
            actor: "a".into(),
            order: serde_json::from_value(
                serde_json::json!({"kind":"move","position":[3,0.81,-2],"arrival_distance":0.2,"for_ticks":600,"formation":"offsets","spacing":1,"facing_degrees":0}),
            )
            .unwrap(),
        },
    );
    e.advance(20).unwrap();
    assert_eq!(e.simulation.presentation.actors["a"].phase, Phase::Walk);
    assert_eq!(e.simulation.presentation.actors["flyer"].phase, Phase::Air);
    assert_eq!(e.scene.meshes, authored);
    let first = Frame::capture(&e, 7, None).unwrap();
    let mut replica = Replica::new(7);
    replica.apply(&first.encode(None).unwrap()).unwrap();
    assert_eq!(replica.avatars(), first.avatars);
    // Expanded asset catalogs must not truncate an avatar's public asset identity.
    let mut wide = first.clone();
    wide.avatars[0].asset = 300;
    let mut wide_replica = Replica::new(7);
    wide_replica.apply(&wide.encode(None).unwrap()).unwrap();
    assert_eq!(wide_replica.avatars()[0].asset, 300);

    call(
        &mut e,
        Command::OrderNpc {
            actor: "a".into(),
            order: serde_json::from_value(serde_json::json!({"kind":"hold","for_ticks":600}))
                .unwrap(),
        },
    );
    e.advance(2).unwrap();
    call(
        &mut e,
        Command::ActivateAbility {
            actor: "a".into(),
            slot: 1,
            target: None,
        },
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.presentation.actors["a"].phase, Phase::Cast);
    let samples = pcm(
        &e.scene.avatar_audio,
        &e.simulation.presentation.voices,
        e.simulation.tick,
        2400,
        [-3., 0.81, -2.],
        30.,
    )
    .unwrap();
    assert!(samples.iter().any(|x| *x != 0));
    let wav = ignis::npc_presentation::wav(&samples);
    assert_eq!(&wav[..4], b"RIFF");
    assert_eq!(wav.len(), 4844);
    let old = e.simulation.presentation.actors["a"];
    call(
        &mut e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 1,
            target: Some("a".into()),
        },
    );
    e.advance(1).unwrap();
    assert_eq!(e.simulation.presentation.actors["a"].phase, Phase::Dead);
    call(
        &mut e,
        Command::ActivateAbility {
            actor: "player".into(),
            slot: 2,
            target: Some("a".into()),
        },
    );
    e.advance(1).unwrap();
    let new = e.simulation.presentation.actors["a"];
    assert_eq!(new.instance, old.instance);
    assert_eq!(new.life, old.life + 1);
    assert!(new.started > old.started);
    assert_eq!(new.previous_clip, None);
    let retired = e.simulation.presentation.actors["b"].instance;
    call(&mut e, Command::DespawnActor { actor: "b".into() });
    call(
        &mut e,
        Command::SpawnActor {
            template: "new_creature".into(),
            position: [-3., 0.81, 2.],
            target: None,
        },
    );
    e.advance(1).unwrap();
    assert!(!e.simulation.presentation.actors.contains_key("b"));
    assert_eq!(e.simulation.presentation.actors.len(), 3);
    assert!(
        e.simulation
            .presentation
            .actors
            .values()
            .all(|s| s.instance != retired)
    );
    let last = Frame::capture(&e, 7, Some(&first)).unwrap();
    let bytes = last.encode(Some(&first)).unwrap();
    let before = replica.clone();
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() = 255;
    assert!(replica.apply(&bad).is_err());
    assert_eq!(replica, before);
    let mut old_version = bytes.clone();
    old_version[4..6].copy_from_slice(&2u16.to_le_bytes());
    assert!(replica.apply(&old_version).is_err());
    assert_eq!(replica, before);
    replica.apply(&bytes).unwrap();
    assert_eq!(replica.avatars(), last.avatars);
    let mut invalid = last.clone();
    invalid.avatars[0].life += 1;
    assert!(invalid.encode(Some(&first)).is_err());
    assert_eq!(Engine::verify_replay(&e.replay()).unwrap()["matched"], true);
    let mut invalid_scene = e.scene.clone();
    invalid_scene.version = 94;
    assert!(invalid_scene.validate().is_err());
}
