use ignis::{
    engine::{Command, Engine},
    materials::{Material, Reflectance},
    mesh_preservation::DisplacementPolicy,
    meshes::{Edit, ExtractionMode},
    model::RoomRecipe,
    plants,
};
use std::sync::Arc;

fn definition() -> plants::Definition {
    plants::Definition {
        trajectory: None,
        id: "surface".into(),
        profile: plants::Rectangle {
            id: "outline".into(),
            width: 2.,
            height: 3.,
        },
        depth: 0.3,
        sections: None,
        segments: [1, 2, 1],
        subdivision_levels: 0,
        sampling_tolerance: 0.001,
    }
}
fn put(definition: plants::Definition) -> Command {
    Command::EditPlant {
        edit: plants::Edit::Put {
            definition,
            detail_policy: DisplacementPolicy::Physical,
            animation_translation_policy: DisplacementPolicy::Physical,
        },
    }
}
fn engine() -> Engine {
    let mut e = Engine::new(
        RoomRecipe {
            obstacles: 0,
            ..Default::default()
        }
        .generate()
        .unwrap(),
    )
    .unwrap();
    e.execute(put(definition()).into()).unwrap();
    e
}
fn material(roughness: f32, f0: f32) -> Material {
    Material {
        reflectance: Some(Reflectance {
            roughness,
            f0,
            metallic: 0.,
        }),
        ..Default::default()
    }
}
fn edit(e: &mut Engine, edit: Edit) {
    e.execute(Command::EditMesh { edit }.into()).unwrap();
}

#[test]
fn invalid_response_is_atomic_and_does_not_add_undo_history() {
    let mut e = engine();
    let original = e.scene.clone();
    edit(
        &mut e,
        Edit::SetMaterial {
            asset: "surface".into(),
            material: material(0.5, 0.04),
        },
    );
    e.execute(Command::Step { ticks: 7 }.into()).unwrap();
    for (r, f0) in [
        (f32::NAN, 0.04),
        (f32::INFINITY, 0.04),
        (0.199, 0.04),
        (1.001, 0.04),
        (0.5, f32::NAN),
        (0.5, f32::INFINITY),
        (0.5, -0.001),
        (0.5, 0.161),
    ] {
        let before = e.snapshot();
        let scene = e.scene.clone();
        assert!(
            e.execute(
                Command::EditMesh {
                    edit: Edit::SetMaterial {
                        asset: "surface".into(),
                        material: material(r, f0)
                    }
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.snapshot(), before);
        assert_eq!(e.scene, scene);
    }
    for (r, f0) in [(0.2, 0.), (1., 0.16)] {
        material(r, f0).validate().unwrap();
    }
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
}

#[test]
fn metadata_preserves_source_reference_and_regeneration_is_reversible() {
    let mut e = engine();
    let original = e.scene.clone();
    let response = material(0.65, 0.04);
    let edit_material = Edit::SetMaterial {
        asset: "surface".into(),
        material: response.clone(),
    };
    let preview = e
        .execute(
            Command::PreviewMeshEdit {
                edit: edit_material.clone(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(e.scene, original);
    edit(&mut e, edit_material);
    assert_eq!(
        preview["meshes"],
        serde_json::to_value(&e.scene.meshes).unwrap()
    );
    let source = &e.scene.plants.entries[0].source;
    let old_source = &original.plants.entries[0].source;
    assert!(Arc::ptr_eq(&source.geometry, &old_source.geometry));
    assert_eq!(source.rig, old_source.rig);
    assert_eq!(source.clips, old_source.clips);
    assert_eq!(source.material, response);
    assert_eq!(
        e.scene.plants.entries[0].reference,
        original.plants.entries[0].reference
    );
    assert!(Arc::ptr_eq(
        &e.scene.meshes.assets[0].geometry,
        &original.meshes.assets[0].geometry
    ));
    let changed = e.scene.clone();
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, changed);
    let mut d = definition();
    d.segments = [2, 4, 2];
    e.execute(put(d).into()).unwrap();
    assert_eq!(e.scene.meshes.assets[0].material, response);
    assert_eq!(
        e.scene.plants.entries[0].reference,
        original.plants.entries[0].reference
    );
    e.execute(put(definition()).into()).unwrap();
    assert_eq!(e.scene, changed);
}

#[test]
fn extraction_subdivision_and_replay_keep_response() {
    let mut e = engine();
    let response = material(0.4, 0.08);
    edit(
        &mut e,
        Edit::SetMaterial {
            asset: "surface".into(),
            material: response.clone(),
        },
    );
    let islands = e
        .execute(
            Command::InspectMeshIslands {
                asset: "surface".into(),
            }
            .into(),
        )
        .unwrap();
    edit(
        &mut e,
        Edit::ExtractIslands {
            asset: "surface".into(),
            id: "copy".into(),
            reference: serde_json::from_value(islands["reference"].clone()).unwrap(),
            islands: vec![0],
            mode: ExtractionMode::Copy,
        },
    );
    assert_eq!(e.scene.meshes.assets[1].material, response);
    let topology = e
        .execute(
            Command::InspectMeshTopology {
                asset: "copy".into(),
            }
            .into(),
        )
        .unwrap();
    edit(
        &mut e,
        Edit::Subdivide {
            asset: "copy".into(),
            reference: serde_json::from_value(topology["reference"].clone()).unwrap(),
            components: None,
        },
    );
    assert_eq!(e.scene.meshes.assets[1].material, response);
    let replay = e.replay();
    assert_eq!(replay.version, ignis::engine::REPLAY_VERSION);
    assert_eq!(replay.scene.version, ignis::model::SCENE_VERSION);
    let roundtrip = serde_json::from_slice(&serde_json::to_vec(&replay).unwrap()).unwrap();
    assert_eq!(Engine::replay_at(&roundtrip, None).unwrap().scene, e.scene);
    let mut old = replay;
    old.version = 63;
    assert!(Engine::replay_at(&old, None).is_err());
}

#[test]
fn old_scene_rejects_response_in_asset_or_only_canonical_source_before_migration() {
    let mut e = engine();
    e.scene.version = 59;
    e.scene.meshes.assets[0].material = material(0.5, 0.);
    assert!(Engine::new(e.scene.clone()).is_err());
    e.scene.meshes.assets[0].material = Material::default();
    Arc::make_mut(&mut e.scene.plants.entries[0].source).material = material(0.5, 0.04);
    assert!(Engine::new(e.scene.clone()).is_err());
    Arc::make_mut(&mut e.scene.plants.entries[0].source).material = Material::default();
    assert_eq!(
        Engine::new(e.scene).unwrap().scene.version,
        ignis::model::SCENE_VERSION
    );
    assert!(
        !serde_json::to_string(&Material::default())
            .unwrap()
            .contains("reflectance")
    );
}
