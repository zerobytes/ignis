//! Protect authored-state atomicity and format boundaries across the new Light field.
use ignis::{
    atmosphere::Atmosphere,
    engine::{Command, Engine, SceneEdit},
    model::RoomRecipe,
    network::content::Content,
    portable_save::PortableSave,
};

#[test]
fn atmosphere_cannot_bypass_formats_or_partially_mutate_and_survives_histories() {
    let mut engine = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    let original = engine.scene.clone();
    assert!(
        serde_json::to_value(&original.light)
            .unwrap()
            .get("atmosphere")
            .is_none()
    );
    let mut light = original.light.clone();
    light.atmosphere = Some(Atmosphere::default());
    let edits = vec![SceneEdit::SetLight {
        light: light.clone(),
    }];
    let before = engine.snapshot();
    engine
        .execute(
            Command::PreviewSceneEdits {
                edits: edits.clone(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(engine.snapshot(), before);
    engine.execute(Command::EditScene { edits }.into()).unwrap();
    let published = engine.scene.clone();
    engine.execute(Command::Step { ticks: 2 }.into()).unwrap();
    let before = engine.snapshot();
    for case in 0..5 {
        let mut invalid = light.clone();
        let a = invalid.atmosphere.as_mut().unwrap();
        match case {
            0 => a.density = f32::NAN,
            1 => a.density = -0.01,
            2 => a.color[0] = f32::INFINITY,
            3 => a.sun_color[1] = 1.01,
            _ => a.sun_strength = -0.01,
        }
        assert!(
            engine
                .execute(Command::SetLight { light: invalid }.into())
                .is_err()
        );
        assert_eq!(engine.scene, published);
        assert_eq!(engine.snapshot(), before);
    }
    let mut legacy = published.clone();
    legacy.version = 99;
    legacy.light.atmosphere.as_mut().unwrap().density = 0.;
    assert!(
        legacy
            .validate()
            .unwrap_err()
            .message
            .contains("version100")
    );
    legacy.light.atmosphere = None;
    legacy.validate().unwrap();
    assert_eq!(
        Engine::replay_at(&engine.replay(), None).unwrap().scene,
        published
    );
    let bytes = PortableSave::capture(&engine).unwrap().encode().unwrap();
    assert_eq!(
        Engine::from_portable_save(&bytes, None)
            .unwrap()
            .scene
            .light,
        light
    );
    let content = Content::capture(&engine, 42);
    let bytes = content.encode().unwrap();
    assert_eq!(Content::decode(42, &bytes).unwrap().light, light);
    let mut old_content = content;
    old_content.version = 1;
    assert!(old_content.validate(42).is_err());
    engine.execute(Command::Undo.into()).unwrap();
    assert_eq!(engine.scene, original);
    engine.execute(Command::Redo.into()).unwrap();
    assert_eq!(engine.scene, published);
}

#[test]
fn compiled_payload_is_atomic_source_bound_and_portable() {
    use ignis::meshes::Edit;
    use std::sync::Arc;
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::Grid {
                id: "ridge".into(),
                segments: 32,
                size: 16.,
            },
        }
        .into(),
    )
    .unwrap();
    let before = e.scene.clone();
    let mut bad = before.light.clone();
    bad.ambient = f32::NAN;
    assert!(
        e.execute(
            Command::EditScene {
                edits: vec![
                    SceneEdit::CompileDistantMeshes,
                    SceneEdit::SetLight { light: bad }
                ]
            }
            .into()
        )
        .is_err()
    );
    assert_eq!(e.scene, before);
    e.execute(
        Command::EditScene {
            edits: vec![SceneEdit::CompileDistantMeshes],
        }
        .into(),
    )
    .unwrap();
    assert_eq!(e.scene.distant_meshes.len(), 1);
    assert_eq!(&e.scene.distant_meshes[0].data[..4], b"ATM3");
    let compiled = e.scene.clone();
    let bytes = PortableSave::capture(&e).unwrap().encode().unwrap();
    assert_eq!(
        Engine::from_portable_save(&bytes, None)
            .unwrap()
            .scene
            .distant_meshes,
        compiled.distant_meshes
    );
    let packet = Content::capture(&e, 42).encode().unwrap();
    assert_eq!(
        Content::decode(42, &packet).unwrap().distant_meshes,
        compiled.distant_meshes
    );
    let mut poisoned = compiled.clone();
    let p = &mut Arc::make_mut(&mut poisoned.distant_meshes)[0];
    let bytes = Arc::make_mut(&mut p.data);
    bytes[40] ^= 1; // Same source hash, forged certificate.
    assert!(poisoned.validate().is_err());
    let mut stale = compiled.clone();
    Arc::make_mut(&mut stale.meshes.assets[0].geometry).positions[0][1] += 0.1;
    stale.validate().unwrap(); // Safe fallback, not a failed authoring edit.
    assert!(Engine::new(stale).unwrap().scene.distant_meshes.is_empty());
    let mut old = compiled.clone();
    old.version = 100;
    assert!(old.validate().is_err());
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, before);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, compiled);
}

// New appearance fields must not leak into old formats or partially mutate state.
#[test]
fn layers_are_atomic_versioned_and_portable() {
    use ignis::atmosphere::Layers;
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    let mut light = e.scene.light.clone();
    light.atmosphere = Some(Atmosphere {
        layers: Some(Layers::default()),
        ..Default::default()
    });
    e.execute(
        Command::SetLight {
            light: light.clone(),
        }
        .into(),
    )
    .unwrap();
    let accepted = e.scene.clone();
    for k in 0..4 {
        let mut bad = light.clone();
        let l = bad.atmosphere.as_mut().unwrap().layers.as_mut().unwrap();
        match k {
            0 => l.detail_end = l.detail_start,
            1 => l.influence = f32::NAN,
            2 => l.sky_color[0] = 1.1,
            _ => l.mountain_color[0] = -0.1,
        }
        assert!(e.execute(Command::SetLight { light: bad }.into()).is_err());
        assert_eq!(e.scene, accepted);
    }
    let mut old = accepted.clone();
    old.version = 100;
    assert!(old.validate().is_err());
    let bytes = PortableSave::capture(&e).unwrap().encode().unwrap();
    assert_eq!(
        Engine::from_portable_save(&bytes, None)
            .unwrap()
            .scene
            .light,
        light
    );
    let content = Content::capture(&e, 42).encode().unwrap();
    assert_eq!(Content::decode(42, &content).unwrap().light, light);
    e.execute(Command::Undo.into()).unwrap();
    assert!(e.scene.light.atmosphere.is_none());
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene.light, light);
    assert_eq!(
        Engine::replay_at(&e.replay(), None).unwrap().scene.light,
        light
    );
}

// A new authored effect must reject invalid/old-format writes atomically and
// retain its exact controls through undo, replay, network content and saves.
#[test]
fn edge_softness_is_versioned_atomic_and_persistent() {
    use ignis::atmosphere::EdgeSoftness;
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    let original = e.scene.clone();
    let mut light = original.light.clone();
    light.atmosphere = Some(Atmosphere {
        edge_softness: Some(EdgeSoftness::default()),
        ..Default::default()
    });
    e.execute(
        Command::SetLight {
            light: light.clone(),
        }
        .into(),
    )
    .unwrap();
    let accepted = e.scene.clone();
    let before = e.snapshot();
    for k in 0..6 {
        let mut bad = light.clone();
        let edge = bad
            .atmosphere
            .as_mut()
            .unwrap()
            .edge_softness
            .as_mut()
            .unwrap();
        match k {
            0 => edge.strength = f32::NAN,
            1 => edge.radius_px = 13,
            2 => edge.start = -1.,
            3 => edge.end = edge.start,
            4 => edge.end = f32::INFINITY,
            _ => edge.strength = 1.01,
        }
        assert!(e.execute(Command::SetLight { light: bad }.into()).is_err());
        assert_eq!(e.scene, accepted);
        assert_eq!(e.snapshot(), before);
    }
    let mut legacy = accepted.clone();
    legacy.version = 101;
    assert!(legacy.validate().is_err());
    legacy.light.atmosphere.as_mut().unwrap().edge_softness = None;
    legacy.validate().unwrap();
    let bytes = PortableSave::capture(&e).unwrap().encode().unwrap();
    assert_eq!(
        Engine::from_portable_save(&bytes, None)
            .unwrap()
            .scene
            .light,
        light
    );
    let content = Content::capture(&e, 42).encode().unwrap();
    assert_eq!(Content::decode(42, &content).unwrap().light, light);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, accepted);
    assert_eq!(
        Engine::replay_at(&e.replay(), None).unwrap().scene.light,
        light
    );
}
