use ignis::{
    engine::{Command, Engine},
    materials::{Material, Pattern},
    meshes::Edit,
    model::RoomRecipe,
};
use serde_json::json;
fn pattern() -> Pattern {
    serde_json::from_value(json!({"frequency":[2,3,4],"low":[0.4,0.5,0.6],"high":[1,1.2,1.1],"bump":0.002,"warp":0.3,"octaves":2,"seed":37,"contrast":1.2})).unwrap()
}
fn engine() -> Engine {
    let mut e = Engine::new(RoomRecipe::default().generate().unwrap()).unwrap();
    e.execute(
        Command::EditMesh {
            edit: Edit::Grid {
                id: "surface".into(),
                segments: 1,
                size: 2.,
            },
        }
        .into(),
    )
    .unwrap();
    e
}
#[test]
fn absent_secondary_preserves_serialization_and_equality() {
    let old: Material =
        serde_json::from_value(json!({"pattern":pattern(),"transmission":0.0})).unwrap();
    assert!(
        serde_json::to_value(&old)
            .unwrap()
            .get("secondary_pattern")
            .is_none()
    );
    let mut new = old.clone();
    new.secondary_pattern = Some(pattern());
    assert_ne!(old, new);
    new.secondary_pattern = None;
    assert_eq!(old, new);
    let mut v = serde_json::to_value(&new).unwrap();
    v["secondary_pattern"] = json!({"unknown":0});
    assert!(serde_json::from_value::<Material>(v).is_err());
}
#[test]
fn either_pattern_has_same_bounds_and_combined_bump_is_bounded() {
    for field in 0..9 {
        let mut p = pattern();
        match field {
            0 => p.frequency[0] = f32::NAN,
            1 => p.frequency[2] = 0.,
            2 => p.low[1] = -0.1,
            3 => p.high[0] = 2.1,
            4 => p.bump = 0.051,
            5 => p.warp = 2.1,
            6 => p.octaves = 0,
            7 => p.octaves = 5,
            _ => p.contrast = f32::INFINITY,
        }
        let a = Material {
            pattern: Some(p),
            ..Default::default()
        };
        let b = Material {
            secondary_pattern: Some(p),
            ..Default::default()
        };
        assert!(a.validate().is_err() && b.validate().is_err());
    }
    let mut p = pattern();
    p.bump = 0.025;
    let mut m = Material {
        pattern: Some(p),
        secondary_pattern: Some(p),
        ..Default::default()
    };
    m.validate().unwrap();
    m.secondary_pattern.as_mut().unwrap().bump = 0.026;
    assert!(m.validate().is_err());
    m.pattern = None;
    m.validate().unwrap();
}
#[test]
fn publication_preview_history_legacy_gate_and_replay_share_the_contract() {
    let mut e = engine();
    e.execute(Command::Step { ticks: 7 }.into()).unwrap();
    let before = e.snapshot();
    let old = e.scene.clone();
    let mut bad = pattern();
    bad.bump = 0.05;
    let edit = Edit::SetMaterial {
        asset: "surface".into(),
        material: Material {
            pattern: Some(bad),
            secondary_pattern: Some(bad),
            ..Default::default()
        },
    };
    for c in [
        Command::PreviewMeshEdit { edit: edit.clone() },
        Command::EditMesh { edit },
    ] {
        assert!(e.execute(c.into()).is_err());
        assert_eq!(e.snapshot(), before);
        assert_eq!(e.scene, old);
    }
    let edit = Edit::SetMaterial {
        asset: "surface".into(),
        material: Material {
            secondary_pattern: Some(pattern()),
            ..Default::default()
        },
    };
    let preview = e
        .execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(e.scene, old);
    e.execute(Command::EditMesh { edit }.into()).unwrap();
    assert_eq!(
        preview["meshes"],
        serde_json::to_value(&e.scene.meshes).unwrap()
    );
    let mut legacy = e.scene.clone();
    legacy.version = 72;
    assert!(legacy.validate().unwrap_err().message.contains("version73"));
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(Engine::replay_at(&replay, None).unwrap().scene, e.scene);
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, old);
}
