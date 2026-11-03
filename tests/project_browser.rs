use ignis::{
    engine::Command,
    model::RoomRecipe,
    project::{FILE, Manifest},
    project_browser::{self as browser, Kind, Query, Sort},
    service::Service,
};
use std::path::Path;
fn service(path: &Path) -> Service {
    Service::new(path, RoomRecipe::default().generate().unwrap()).unwrap()
}
fn query(path: &Path) -> Query {
    Query {
        path: path.to_string_lossy().into_owned(),
        search: String::new(),
        show_hidden: false,
        sort: Sort::Name,
        descending: false,
        after: None,
    }
}
fn manifest(parent: &Path, name: &str) -> std::path::PathBuf {
    let path = parent.join(name);
    std::fs::create_dir(&path).unwrap();
    std::fs::write(
        path.join(FILE),
        serde_json::to_vec(&Manifest {
            version: 1,
            name: name.into(),
            entry_scene: "scenes/Principal.scene.json".into(),
            scenes: vec!["scenes/Principal.scene.json".into()],
        })
        .unwrap(),
    )
    .unwrap();
    path
}
#[test]
fn browser_keeps_every_sorted_page_and_recognizes_projects_and_files() {
    let temp = tempfile::tempdir().unwrap();
    for i in (0..270).rev() {
        std::fs::create_dir(temp.path().join(format!("Pasta {i:03}"))).unwrap();
    }
    std::fs::write(temp.path().join("arquivo.txt"), b"hello").unwrap();
    std::fs::write(temp.path().join("Mapa.scene.json"), b"{}").unwrap();
    std::fs::create_dir(temp.path().join(".oculta")).unwrap();
    manifest(temp.path(), "Meu projeto");
    let mut q = query(temp.path());
    let first = browser::browse(&q).unwrap();
    assert_eq!(first.entries.len(), 256);
    assert!(first.next.is_some());
    q.after = first.next;
    let second = browser::browse(&q).unwrap();
    assert_eq!(second.entries.len(), 17);
    assert!(second.next.is_none());
    let all: Vec<_> = first.entries.iter().chain(&second.entries).collect();
    assert_eq!(all.iter().filter(|e| e.kind == Kind::Project).count(), 1);
    assert_eq!(all.iter().filter(|e| e.kind == Kind::Scene).count(), 1);
    assert_eq!(
        all.iter().find(|e| e.name == "arquivo.txt").unwrap().bytes,
        Some(5)
    );
    let mut names: Vec<_> = all.iter().map(|e| e.path.clone()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 273);
    q.after = None;
    q.descending = true;
    let reverse = browser::browse(&q).unwrap();
    assert_eq!(reverse.entries[0].name, "arquivo.txt");
    q.search = "pAsTa 26".into();
    let filtered = browser::browse(&q).unwrap();
    assert_eq!(filtered.entries.len(), 10);
    q.search = "oculta".into();
    assert!(browser::browse(&q).unwrap().entries.is_empty());
    q.show_hidden = true;
    assert_eq!(browser::browse(&q).unwrap().entries.len(), 1);
}
#[test]
fn folder_creation_never_overwrites_or_changes_the_scene() {
    let temp = tempfile::tempdir().unwrap();
    let mut s = service(temp.path());
    let before = s.engine.snapshot();
    let path = s
        .execute(
            Command::CreateProjectFolder {
                parent: temp.path().to_string_lossy().into_owned(),
                name: "Meus jogos".into(),
            }
            .into(),
        )
        .unwrap()
        .data["created_folder"]
        .as_str()
        .unwrap()
        .to_owned();
    std::fs::write(Path::new(&path).join("keep.txt"), b"preserve").unwrap();
    for name in ["Meus jogos", "../outside", "", "a/b", ".."] {
        assert!(
            s.execute(
                Command::CreateProjectFolder {
                    parent: temp.path().to_string_lossy().into_owned(),
                    name: name.into()
                }
                .into()
            )
            .is_err()
        );
    }
    assert_eq!(s.engine.snapshot(), before);
    assert_eq!(
        std::fs::read(Path::new(&path).join("keep.txt")).unwrap(),
        b"preserve"
    );
    assert!(browser::create_folder("relative", "game").is_err());
    assert!(browser::browse(&query(&temp.path().join("missing"))).is_err());
}
#[test]
fn library_survives_new_service_pins_favorites_and_metadata_only_removal() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("preferences/projects.json");
    let root = temp.path().join("Jogo");
    let mut s = service(temp.path());
    s.execute(
        Command::UseProjectLibrary {
            path: file.to_string_lossy().into_owned(),
        }
        .into(),
    )
    .unwrap();
    s.execute(
        Command::CreateProject {
            path: root.to_string_lossy().into_owned(),
            name: "Jogo".into(),
            discard_changes: false,
        }
        .into(),
    )
    .unwrap();
    let before = s.engine.snapshot();
    let saved = std::fs::read(root.join(FILE)).unwrap();
    s.execute(
        Command::PinProject {
            path: root.to_string_lossy().into_owned(),
            pinned: true,
        }
        .into(),
    )
    .unwrap();
    s.execute(
        Command::SetProjectFavorite {
            path: temp.path().to_string_lossy().into_owned(),
            favorite: true,
        }
        .into(),
    )
    .unwrap();
    assert_eq!(s.engine.snapshot(), before);
    let mut next = service(temp.path());
    next.execute(
        Command::UseProjectLibrary {
            path: file.to_string_lossy().into_owned(),
        }
        .into(),
    )
    .unwrap();
    let data = next
        .execute(Command::InspectProjectLibrary.into())
        .unwrap()
        .data;
    assert_eq!(data["library"]["projects"][0]["name"], "Jogo");
    assert_eq!(data["library"]["projects"][0]["pinned"], true);
    assert!(data["library"]["projects"][0]["last_opened"].is_u64());
    assert_eq!(data["library"]["favorites"].as_array().unwrap().len(), 1);
    next.execute(
        Command::ForgetProject {
            path: root.to_string_lossy().into_owned(),
        }
        .into(),
    )
    .unwrap();
    assert!(next.project_library.data.projects.is_empty());
    assert_eq!(std::fs::read(root.join(FILE)).unwrap(), saved);
    assert!(next.engine.scene.entities.iter().any(|e| e.id == "player"));
}
#[test]
fn invalid_library_load_is_atomic_and_cannot_erase_good_preferences() {
    let temp = tempfile::tempdir().unwrap();
    let project = manifest(temp.path(), "Good");
    let good = temp.path().join("good.json");
    let bad = temp.path().join("bad.json");
    let mut state = browser::State::default();
    state.load(good.to_str().unwrap()).unwrap();
    state.register(project.to_str().unwrap()).unwrap();
    let bytes = std::fs::read(&good).unwrap();
    let before = state.inspect();
    for data in [
        "{}",
        r#"{"version":99,"projects":[],"favorites":[]}"#,
        "not json",
    ] {
        std::fs::write(&bad, data).unwrap();
        assert!(state.load(bad.to_str().unwrap()).is_err());
        assert_eq!(state.inspect(), before);
        assert_eq!(std::fs::read(&good).unwrap(), bytes);
    }
    std::fs::write(&bad, vec![b' '; 262_145]).unwrap();
    assert!(state.load(bad.to_str().unwrap()).is_err());
    assert_eq!(state.inspect(), before);
}
#[test]
fn discovery_is_shallow_and_library_limits_preserve_pins() {
    let temp = tempfile::tempdir().unwrap();
    let direct = manifest(temp.path(), "Direto");
    let deep = temp.path().join("deep");
    std::fs::create_dir(&deep).unwrap();
    manifest(&deep, "Nested");
    let mut state = browser::State::default();
    let data = state
        .discover(&[temp.path().to_string_lossy().into_owned()])
        .unwrap();
    assert_eq!(data["found"], 1);
    assert_eq!(
        state.data.projects[0].path,
        direct.canonicalize().unwrap().to_str().unwrap()
    );
    state
        .pin(direct.canonicalize().unwrap().to_str().unwrap(), true)
        .unwrap();
    for i in 0..60 {
        state.remember_opened(
            &temp.path().join(format!("Other{i}")),
            &format!("Other{i}"),
            1,
        );
    }
    assert_eq!(state.data.projects.len(), 32);
    assert!(
        state
            .data
            .projects
            .iter()
            .any(|p| p.pinned && p.path == direct.canonicalize().unwrap().to_str().unwrap())
    );
    let before = state.inspect();
    assert!(state.discover(&vec![String::from("/"); 9]).is_err());
    assert_eq!(state.inspect(), before);
}
#[test]
fn recent_write_failure_keeps_the_successfully_created_project() {
    let temp = tempfile::tempdir().unwrap();
    let blocker = temp.path().join("not-a-folder");
    std::fs::write(&blocker, b"keep").unwrap();
    let mut s = service(temp.path());
    s.execute(
        Command::UseProjectLibrary {
            path: blocker.join("library.json").to_string_lossy().into_owned(),
        }
        .into(),
    )
    .unwrap();
    let root = temp.path().join("New");
    s.execute(
        Command::CreateProject {
            path: root.to_string_lossy().into_owned(),
            name: "New".into(),
            discard_changes: false,
        }
        .into(),
    )
    .unwrap();
    assert!(root.join(FILE).is_file());
    assert_eq!(s.project.as_ref().unwrap().name, "New");
    assert!(s.project_library.warning.is_some());
    assert_eq!(std::fs::read(blocker).unwrap(), b"keep");
}
#[cfg(unix)]
#[test]
fn aliases_deduplicate_projects_and_manifest_escapes_are_rejected() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let project = manifest(temp.path(), "Original");
    let alias = temp.path().join("Alias");
    symlink(&project, &alias).unwrap();
    let mut state = browser::State::default();
    state.register(project.to_str().unwrap()).unwrap();
    state.register(alias.to_str().unwrap()).unwrap();
    assert_eq!(state.data.projects.len(), 1);
    state.pin(project.to_str().unwrap(), true).unwrap();
    state.pin(alias.to_str().unwrap(), false).unwrap();
    assert!(!state.data.projects[0].pinned);
    state.favorite(project.to_str().unwrap(), true).unwrap();
    state.favorite(alias.to_str().unwrap(), false).unwrap();
    assert!(state.data.favorites.is_empty());
    let broken = temp.path().join("Escape");
    std::fs::create_dir(&broken).unwrap();
    symlink(project.join(FILE), broken.join(FILE)).unwrap();
    assert!(state.register(broken.to_str().unwrap()).is_err());
    assert_eq!(state.data.projects.len(), 1);
    state.forget(alias.to_str().unwrap()).unwrap();
    assert!(state.data.projects.is_empty());
    assert!(project.join(FILE).is_file());
    state.register(project.to_str().unwrap()).unwrap();
    state.pin(project.to_str().unwrap(), true).unwrap();
    state.favorite(project.to_str().unwrap(), true).unwrap();
    let stored = state.data.projects[0].path.clone();
    let moved = temp.path().join("Moved");
    std::fs::rename(&project, &moved).unwrap();
    state.pin(&stored, false).unwrap();
    state.favorite(&stored, false).unwrap();
    state.forget(&stored).unwrap();
    assert!(state.data.projects.is_empty() && state.data.favorites.is_empty());
    assert!(moved.join(FILE).is_file());
}
