use ignis::{
    engine::Command,
    model::RoomRecipe,
    project::{FILE, ObjectKind},
    service::Service,
};
use serde_json::{Value, json};
fn call(s: &mut Service, v: Value) -> ignis::model::Result<Value> {
    Ok(
        s.execute(serde_json::from_value::<Command>(v).unwrap().into())?
            .data,
    )
}
fn service(root: &std::path::Path) -> Service {
    Service::new(root, RoomRecipe::default().generate().unwrap()).unwrap()
}
#[test]
fn project_lifecycle_persists_scenes_objects_and_entry_independently() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("Meu jogo");
    let mut s = service(tmp.path());
    call(
        &mut s,
        json!({"op":"create_project","path":path,"name":"Meu jogo"}),
    )
    .unwrap();
    assert!(path.join(FILE).is_file());
    assert!(!s.scene_dirty());
    assert_eq!(s.engine.scene.entities.len(), 2);
    s.execute(
        Command::CreateSceneObject {
            kind: ObjectKind::Dynamic,
        }
        .into(),
    )
    .unwrap();
    let id = s.engine.selected_id().unwrap().to_owned();
    assert!(s.scene_dirty());
    s.execute(Command::DuplicateSceneObject { id: id.clone() }.into())
        .unwrap();
    assert_eq!(s.engine.scene.entities.len(), 4);
    s.execute(Command::Undo.into()).unwrap();
    assert_eq!(s.engine.scene.entities.len(), 3);
    let first = s.scene_path.clone().unwrap();
    call(&mut s, json!({"op":"save_scene","path":first})).unwrap();
    assert!(!s.scene_dirty());
    call(&mut s, json!({"op":"new_project_scene","name":"Floresta"})).unwrap();
    assert_eq!(s.engine.scene.entities.len(), 2);
    let second = s.scene_path.clone().unwrap();
    call(&mut s, json!({"op":"set_project_entry","path":second})).unwrap();
    call(&mut s, json!({"op":"open_project_scene","path":first})).unwrap();
    assert!(s.engine.scene.entities.iter().any(|e| e.id == id));
    let mut independent = service(tmp.path());
    call(&mut independent, json!({"op":"open_project","path":path})).unwrap();
    assert_eq!(independent.engine.scene.name, "Floresta");
    assert_eq!(independent.project.as_ref().unwrap().scenes.len(), 2);
    assert!(independent.execute(Command::Undo.into()).is_err());
}
#[test]
fn unsaved_switches_errors_and_existing_folders_preserve_everything() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("Game");
    let mut s = service(tmp.path());
    call(
        &mut s,
        json!({"op":"create_project","path":path,"name":"Game"}),
    )
    .unwrap();
    s.execute(
        Command::CreateSceneObject {
            kind: ObjectKind::Fixed,
        }
        .into(),
    )
    .unwrap();
    let before = s.engine.snapshot();
    let revision = s.engine.revision;
    for cmd in [
        json!({"op":"new_project_scene","name":"Other"}),
        json!({"op":"open_project","path":path}),
        json!({"op":"open_project_scene","path":"scenes/Principal.scene.json"}),
        json!({"op":"create_project","path":tmp.path().join("Other"),"name":"Other"}),
    ] {
        assert!(call(&mut s, cmd).is_err());
        assert_eq!(s.engine.snapshot(), before);
    }
    assert!(!tmp.path().join("Other").exists());
    call(
        &mut s,
        json!({"op":"new_project_scene","name":"Other","discard_changes":true}),
    )
    .unwrap();
    assert!(s.engine.revision > revision);
    assert!(!s.scene_dirty());
    assert!(call(&mut s, json!({"op":"new_project_scene","name":"Other"})).is_err());
    let manifest = std::fs::read(path.join(FILE)).unwrap();
    assert!(
        call(
            &mut s,
            json!({"op":"create_project","path":path,"name":"Replacement"})
        )
        .is_err()
    );
    assert_eq!(std::fs::read(path.join(FILE)).unwrap(), manifest);
}
#[test]
fn invalid_paths_manifests_and_failed_writes_leave_session_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let mut s = service(tmp.path());
    let path = tmp.path().join("Game");
    call(
        &mut s,
        json!({"op":"create_project","path":path,"name":"Game"}),
    )
    .unwrap();
    let before = s.engine.snapshot();
    for name in ["../escape", "", ".", "bad/name"] {
        assert!(call(&mut s, json!({"op":"new_project_scene","name":name})).is_err());
        assert_eq!(s.engine.snapshot(), before);
    }
    assert!(
        call(
            &mut s,
            json!({"op":"open_project_scene","path":"../escape"})
        )
        .is_err()
    );
    std::fs::remove_file(path.join(FILE)).unwrap();
    std::fs::create_dir(path.join(FILE)).unwrap();
    assert!(call(&mut s, json!({"op":"new_project_scene","name":"Failure"})).is_err());
    assert!(!path.join("scenes/Failure.scene.json").exists());
    assert_eq!(s.engine.snapshot(), before);
    let invalid = tmp.path().join("Invalid");
    std::fs::create_dir(&invalid).unwrap();
    std::fs::write(invalid.join(FILE), b"{\"version\":999}").unwrap();
    assert!(call(&mut s, json!({"op":"open_project","path":invalid})).is_err());
    assert_eq!(s.root, path.canonicalize().unwrap());
    assert_eq!(s.engine.snapshot(), before);
}
#[cfg(unix)]
#[test]
fn project_rejects_scene_symlink_escape_before_switch() {
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let mut s = service(tmp.path());
    let path = tmp.path().join("Game");
    call(
        &mut s,
        json!({"op":"create_project","path":path,"name":"Game"}),
    )
    .unwrap();
    let scene = path.join("scenes/Principal.scene.json");
    std::fs::copy(&scene, outside.path().join("scene.json")).unwrap();
    std::fs::remove_file(&scene).unwrap();
    std::os::unix::fs::symlink(outside.path().join("scene.json"), &scene).unwrap();
    let mut other = service(tmp.path());
    let before = other.engine.snapshot();
    assert!(call(&mut other, json!({"op":"open_project","path":path})).is_err());
    assert_eq!(other.engine.snapshot(), before);
}

#[test]
fn legacy_layout_migrates_without_leaking_project_into_scene() {
    use ignis::editor::{Action, Panel, State};
    let tmp = tempfile::tempdir().unwrap();
    let mut s = service(tmp.path());
    let scene = s.engine.snapshot();
    let mut layout = State::default();
    layout
        .apply(Action::Close {
            area: 3,
            panel: Panel::Project,
        })
        .unwrap();
    layout.version = 1;
    layout.validate().unwrap();
    std::fs::write(
        tmp.path().join("layout.json"),
        serde_json::to_vec(&layout).unwrap(),
    )
    .unwrap();
    s.execute(
        Command::LoadEditor {
            path: "layout.json".into(),
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.editor.version, 3);
    assert_eq!(s.engine.snapshot(), scene);
    layout = State::default();
    layout.version = 1;
    assert!(layout.validate().is_err());
}
