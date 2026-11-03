use ignis::{
    engine::{Command, Engine},
    materials::Material,
    meshes::Edit,
    model::RoomRecipe,
};
use serde_json::json;

fn material(metallic: f32, transmission: f32) -> Material {
    serde_json::from_value(json!({"reflectance":{"roughness":0.6,"f0":0.04,"metallic":metallic},"transmission":transmission})).unwrap()
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
fn default_metallic_omits_the_field_and_changes_equality_only_when_enabled() {
    let old: Material =
        serde_json::from_value(json!({"reflectance":{"roughness":0.6,"f0":0.04}})).unwrap();
    let zero = material(0., 0.);
    assert_eq!(old, zero);
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        json!({"reflectance":{"roughness":0.6f32,"f0":0.04f32},"transmission":0.0})
    );
    assert_ne!(old, material(1., 0.));
    for m in [0., 0.5, 1.] {
        material(m, 0.).validate().unwrap();
    }
    material(0., 1.).validate().unwrap();
    for m in [-0.001, 1.001, f32::NAN, f32::INFINITY] {
        let mut invalid = old.clone();
        invalid.reflectance.as_mut().unwrap().metallic = m;
        assert!(invalid.validate().is_err());
    }
    assert!(material(0.01, 0.01).validate().is_err());
}
#[test]
fn invalid_metal_preserves_preview_history_and_tick_and_publication_is_reversible() {
    let mut e = engine();
    let old = e.scene.clone();
    e.execute(Command::Step { ticks: 9 }.into()).unwrap();
    let before = e.snapshot();
    for m in [material(-0.01, 0.), material(1.01, 0.), material(0.5, 0.1)] {
        let edit = Edit::SetMaterial {
            asset: "surface".into(),
            material: m,
        };
        for command in [
            Command::PreviewMeshEdit { edit: edit.clone() },
            Command::EditMesh { edit },
        ] {
            assert!(e.execute(command.into()).is_err());
            assert_eq!(e.snapshot(), before);
            assert_eq!(e.scene, old);
        }
    }
    let edit = Edit::SetMaterial {
        asset: "surface".into(),
        material: material(1., 0.),
    };
    let preview = e
        .execute(Command::PreviewMeshEdit { edit: edit.clone() }.into())
        .unwrap();
    assert_eq!(e.scene, old);
    e.execute(Command::EditMesh { edit }.into()).unwrap();
    let published = e.scene.clone();
    assert_eq!(
        preview["meshes"],
        serde_json::to_value(&published.meshes).unwrap()
    );
    assert_eq!(
        published.meshes.assets[0].geometry,
        old.meshes.assets[0].geometry
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, old);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, published);
    let mut legacy = e.scene.clone();
    legacy.version = 73;
    assert!(legacy.validate().unwrap_err().message.contains("version74"));
    legacy.meshes.assets[0]
        .material
        .reflectance
        .as_mut()
        .unwrap()
        .metallic = 0.;
    legacy.validate().unwrap();
    assert_eq!(e.replay().version, ignis::engine::REPLAY_VERSION);
    assert_eq!(
        Engine::replay_at(&e.replay(), None).unwrap().scene,
        published
    );
}
