use ignis::{
    engine::{Command, Engine, SceneEdit},
    environment::Environment,
    model::RoomRecipe,
};
fn probe() -> Environment {
    Environment {
        sky: [0.8, 0.9, 1.],
        horizon: [0.4, 0.4, 0.4],
        ground: [0.1, 0.1, 0.1],
        axis: [0., 1., 0.],
        intensity: 0.5,
    }
}
fn engine() -> Engine {
    Engine::new(RoomRecipe::default().generate().unwrap()).unwrap()
}
#[test]
fn light_defaults_omit_new_controls_and_old_formats_cannot_smuggle_them() {
    let e = engine();
    let old = e.scene.light.clone();
    let value = serde_json::to_value(&old).unwrap();
    assert!(value.get("environment").is_none() && value.get("direct_intensity").is_none());
    assert_eq!(
        serde_json::from_value::<ignis::model::Light>(value).unwrap(),
        old
    );
    let mut s = e.scene.clone();
    s.version = 75;
    s.validate().unwrap();
    let mut disabled = probe();
    disabled.intensity = 0.;
    s.light.environment = Some(disabled);
    assert!(s.validate().unwrap_err().message.contains("version76"));
    s.light.environment = None;
    s.light.direct_intensity = 0.;
    assert!(s.validate().unwrap_err().message.contains("version76"));
}
#[test]
fn environment_light_preview_atomic_history_and_replay_share_the_contract() {
    let mut e = engine();
    let old = e.scene.clone();
    let mut light = old.light.clone();
    light.environment = Some(probe());
    light.direct_intensity = 0.;
    let edits = vec![SceneEdit::SetLight {
        light: light.clone(),
    }];
    let before = e.snapshot();
    let preview = e
        .execute(
            Command::PreviewSceneEdits {
                edits: edits.clone(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(e.snapshot(), before);
    e.execute(Command::EditScene { edits }.into()).unwrap();
    let published = e.scene.clone();
    assert_eq!(preview["scene"], serde_json::to_value(&published).unwrap());
    e.execute(Command::Step { ticks: 7 }.into()).unwrap();
    let before = e.snapshot();
    for case in 0..7 {
        let mut bad = light.clone();
        match case {
            0 => bad.direct_intensity = f32::NAN,
            1 => bad.direct_intensity = 4.01,
            2 => bad.direct_intensity = -0.01,
            3 => bad.environment.as_mut().unwrap().intensity = f32::INFINITY,
            4 => bad.environment.as_mut().unwrap().axis = [0.; 3],
            5 => bad.environment.as_mut().unwrap().sky[0] = 1.01,
            _ => bad.environment.as_mut().unwrap().ground[0] = -0.01,
        };
        let edits = vec![SceneEdit::SetLight { light: bad }];
        for command in [
            Command::PreviewSceneEdits {
                edits: edits.clone(),
            },
            Command::EditScene { edits },
        ] {
            assert!(e.execute(command.into()).is_err());
            assert_eq!(e.snapshot(), before);
            assert_eq!(e.scene, published);
        }
    }
    assert_eq!(e.replay().version, ignis::engine::REPLAY_VERSION);
    assert_eq!(
        Engine::replay_at(&e.replay(), None).unwrap().scene,
        published
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, old);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, published);
}
