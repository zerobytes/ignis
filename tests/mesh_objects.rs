//! Critical integration: explicit visual identity must survive transactions and
//! persistence without changing transforms, geometry, physical definitions or ownership.
use ignis::{
    engine::{Command, Engine, SceneEdit},
    mesh_objects::Object,
    meshes::{self, Edit},
    model::{RoomRecipe, Scene},
    network::content::Content,
    portable_save::PortableSave,
    service::Service,
};
use serde_json::json;
use std::sync::Arc;
fn scene() -> Scene {
    let mut s = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    // Match the normal authored-edit ordering before comparing physical slots.
    // An edit still resets execution scopes through the existing Scene contract.
    s.entities.sort_by(|a, b| a.id.cmp(&b.id));
    s.meshes = serde_json::from_value(json!({
        "assets":[{"id":"part","geometry":{"positions":[[-1,0,0],[1,0,0],[0,2,0]],"triangles":[[0,1,2]]}}],
        "instances": (["stem","crown","other"].map(|id| json!({"id":id,"asset":"part","position":[0,1,0],"rotation":[0,0,0,1],"scale":1,"color":[0.2,0.6,0.1]})))
    })).unwrap();
    s
}
fn object(id: &str, members: &[&str]) -> Object {
    Object {
        id: id.into(),
        members: members.iter().map(|m| (*m).into()).collect(),
    }
}
fn edit(e: &mut Engine, edit: Edit) {
    e.execute(Command::EditMesh { edit }.into()).unwrap();
}
#[test]
fn membership_is_atomic_independent_and_survives_undo_replay_save_and_public_content() {
    let tmp = tempfile::tempdir().unwrap();
    let mut service = Service::new(tmp.path(), scene()).unwrap();
    let e = &mut service.engine;
    let original = e.scene.clone();
    let positions = e.simulation.positions();
    let identities = e.simulation.actors.instance_snapshot();
    edit(
        e,
        Edit::PutObject {
            object: object("plant", &["stem", "crown"]),
        },
    );
    let grouped = e.scene.clone();
    for bad in [
        object("missing", &["absent"]),
        object("duplicate", &["stem"]),
        object("empty", &[]),
        object("plant", &["stem", "stem"]),
    ] {
        let revision = e.revision;
        assert!(
            e.execute(
                Command::EditMesh {
                    edit: Edit::PutObject { object: bad }
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(e.scene, grouped);
        assert_eq!(e.revision, revision);
    }
    assert_eq!(e.simulation.positions(), positions);
    assert_eq!(e.simulation.actors.instance_snapshot(), identities);
    assert_eq!(e.scene.meshes.instances, original.meshes.instances);
    assert_eq!(e.scene.meshes.assets, original.meshes.assets);
    assert_eq!(e.scene.collision_meshes, original.collision_meshes);
    assert_eq!(e.scene.characters, original.characters);
    assert_eq!(
        e.scene.meshes.pose("crown", 0).unwrap(),
        original.meshes.pose("crown", 0).unwrap()
    );
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(e.scene, original);
    e.execute(Command::Redo.into()).unwrap();
    assert_eq!(e.scene, grouped);
    edit(
        e,
        Edit::AssignObject {
            instance: "crown".into(),
            object: Some("plant".into()),
        },
    );
    assert_eq!(e.scene, grouped); // no reorder of authored membership
    edit(
        e,
        Edit::AssignObject {
            instance: "crown".into(),
            object: Some("second".into()),
        },
    );
    assert_eq!(e.scene.meshes.objects.len(), 2);
    edit(
        e,
        Edit::AssignObject {
            instance: "stem".into(),
            object: None,
        },
    );
    assert_eq!(
        e.scene.meshes.objects.as_ref(),
        &vec![object("second", &["crown"])]
    );
    edit(e, Edit::RemoveInstance { id: "crown".into() });
    assert!(e.scene.meshes.objects.is_empty());
    e.execute(Command::Undo.into()).unwrap();
    assert_eq!(
        e.scene.meshes.objects.as_ref(),
        &vec![object("second", &["crown"])]
    );
    // One batch can create a piece and the corresponding complete object.
    let mut i = e.scene.meshes.instances[0].clone();
    i.id = "new_piece".into();
    e.execute(
        Command::EditScene {
            edits: vec![
                SceneEdit::Mesh {
                    edit: Edit::PutInstance { instance: i },
                },
                SceneEdit::Mesh {
                    edit: Edit::PutObject {
                        object: object("new_object", &["stem", "new_piece"]),
                    },
                },
            ],
        }
        .into(),
    )
    .unwrap();
    let expected = e.scene.clone();
    let replay = e.replay();
    assert_eq!(Engine::replay_at(&replay, None).unwrap().scene, expected);
    let portable = PortableSave::capture(e).unwrap().encode().unwrap();
    assert_eq!(
        Engine::from_portable_save(&portable, None).unwrap().scene,
        expected
    );
    let public = Content::capture(e, 42);
    assert_eq!(
        Content::decode(42, &public.encode().unwrap())
            .unwrap()
            .meshes,
        expected.meshes
    );
    let mut old = public.clone();
    old.version -= 1;
    assert!(old.encode().is_err());
    let mut corrupt = public;
    Arc::make_mut(&mut corrupt.meshes.objects)[0]
        .members
        .push("absent".into());
    assert!(corrupt.encode().is_err());
    service
        .execute(
            Command::SaveScene {
                path: "objects.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    let mut independent = Service::new(tmp.path(), scene()).unwrap();
    independent
        .execute(
            Command::LoadScene {
                path: "objects.scene.json".into(),
            }
            .into(),
        )
        .unwrap();
    assert_eq!(independent.engine.scene, expected);
}
#[test]
fn legacy_empty_documents_keep_serialized_fingerprint_and_explicit_version_boundary() {
    let mut s = scene();
    let legacy = serde_json::to_value(&s.meshes).unwrap();
    assert!(legacy.get("objects").is_none());
    assert_eq!(
        serde_json::from_value::<meshes::Document>(legacy.clone()).unwrap(),
        s.meshes
    );
    s.version = 106;
    s.validate().unwrap();
    s.meshes.objects = Arc::new(vec![object("plant", &["stem", "crown"])]);
    assert!(s.validate().is_err());
    s.version = ignis::model::SCENE_VERSION;
    s.validate().unwrap();
    let mut frame = meshes::Frame::default();
    frame.update(&s.meshes, 0);
    let generation = frame.static_generation;
    frame.update(&s.meshes, 9);
    assert_eq!(frame.static_generation, generation);
    let mut transformed = s.meshes.clone();
    transformed.instances[0].position[0] += 12.;
    frame.update(&transformed, 9);
    assert_ne!(frame.static_generation, generation);
}
