//! Filesystem navigation and the editor's project library, independent of gameplay.
use crate::{
    model::{Result, invalid},
    project::{self, Manifest},
    service::io_error,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PAGE_SIZE: usize = 256;
pub const MAX_PROJECTS: usize = 32;
pub const MAX_FAVORITES: usize = 16;
const MAX_LIBRARY_BYTES: u64 = 262_144;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    #[default]
    Name,
    Kind,
    Modified,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Project,
    Folder,
    Manifest,
    Scene,
    File,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Cursor {
    pub name: String,
    pub kind: Kind,
    pub modified: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub path: String,
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default)]
    pub sort: Sort,
    #[serde(default)]
    pub descending: bool,
    #[serde(default)]
    pub after: Option<Cursor>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    pub modified: Option<u64>,
    pub bytes: Option<u64>,
}
impl Entry {
    pub fn cursor(&self) -> Cursor {
        Cursor {
            name: self.name.clone(),
            kind: self.kind,
            modified: self.modified,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub path: String,
    pub entries: Vec<Entry>,
    pub next: Option<Cursor>,
    pub project: Option<Project>,
    pub project_error: Option<String>,
}
fn absolute(path: &str) -> Result<PathBuf> {
    let p = Path::new(path);
    if path.len() > 2048
        || !p.is_absolute()
        || p.components()
            .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
    {
        return Err(invalid("Escolha um caminho absoluto, sem . ou .."));
    }
    Ok(p.to_owned())
}
fn directory(path: &str) -> Result<PathBuf> {
    let p = absolute(path)?.canonicalize().map_err(io_error)?;
    if !p.is_dir() {
        return Err(invalid("O local escolhido não é uma pasta"));
    }
    Ok(p)
}
fn text_path(p: &Path) -> Result<String> {
    p.to_str()
        .map(str::to_owned)
        .ok_or_else(|| invalid("O caminho precisa ser texto UTF-8"))
}
// Existing aliases identify the same record; missing paths remain removable by
// their stored spelling after a project or favorite has disappeared from disk.
fn record_path(path: &str) -> Result<String> {
    let path = absolute(path)?;
    text_path(&path.canonicalize().unwrap_or(path))
}
#[derive(Eq, PartialEq)]
struct OrderKey {
    value: (i128, String, String),
    descending: bool,
}
impl Ord for OrderKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.descending.cmp(&other.descending).then_with(|| {
            if self.descending {
                other.value.cmp(&self.value)
            } else {
                self.value.cmp(&other.value)
            }
        })
    }
}
impl PartialOrd for OrderKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
fn key(cursor: &Cursor, sort: Sort, descending: bool) -> OrderKey {
    let group = match cursor.kind {
        Kind::Project | Kind::Folder => 0,
        Kind::Manifest => 1,
        Kind::Scene => 2,
        Kind::File => 3,
    };
    let value = match sort {
        Sort::Name => group,
        Sort::Kind => match cursor.kind {
            Kind::Project => 0,
            Kind::Folder => 1,
            Kind::Manifest => 2,
            Kind::Scene => 3,
            Kind::File => 4,
        },
        Sort::Modified => -(cursor.modified.unwrap_or(0) as i128),
    };
    OrderKey {
        value: (value, cursor.name.to_lowercase(), cursor.name.clone()),
        descending,
    }
}
pub fn browse(query: &Query) -> Result<Page> {
    let folder = directory(&query.path)?;
    if query.search.len() > 256 || query.after.as_ref().is_some_and(|a| a.name.len() > 1024) {
        return Err(invalid("Busca ou cursor excede o limite"));
    }
    let search = query.search.to_lowercase();
    let after = query
        .after
        .as_ref()
        .map(|c| key(c, query.sort, query.descending));
    let mut entries = BTreeMap::new();
    for item in std::fs::read_dir(&folder).map_err(io_error)? {
        let item = item.map_err(io_error)?;
        let Some(name) = item.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if (!query.show_hidden && name.starts_with('.')) || !name.to_lowercase().contains(&search) {
            continue;
        }
        let path = item.path();
        let Ok(metadata) = path.metadata() else {
            continue;
        };
        let kind = if metadata.is_dir() {
            if path.join(project::FILE).is_file() {
                Kind::Project
            } else {
                Kind::Folder
            }
        } else if name == project::FILE {
            Kind::Manifest
        } else if name.ends_with(".scene.json") {
            Kind::Scene
        } else {
            Kind::File
        };
        let entry = Entry {
            path: text_path(&path)?,
            name,
            kind,
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
            bytes: metadata.is_file().then_some(metadata.len()),
        };
        let key = key(&entry.cursor(), query.sort, query.descending);
        if after.as_ref().is_some_and(|a| key <= *a) {
            continue;
        }
        entries.insert(key, entry);
        if entries.len() > PAGE_SIZE + 1 {
            entries.pop_last();
        }
    }
    let more = entries.len() > PAGE_SIZE;
    if more {
        entries.pop_last();
    }
    let entries: Vec<_> = entries.into_values().collect();
    let next = more.then(|| entries.last().unwrap().cursor());
    let (project, project_error) = if folder.join(project::FILE).is_file() {
        match inspect_project(&text_path(&folder)?) {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e.message)),
        }
    } else {
        (None, None)
    };
    Ok(Page {
        path: text_path(&folder)?,
        entries,
        next,
        project,
        project_error,
    })
}
pub fn create_folder(parent: &str, name: &str) -> Result<String> {
    project::validate_name(name)?;
    let path = directory(parent)?.join(name);
    std::fs::create_dir(&path).map_err(io_error)?;
    text_path(&path)
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub path: String,
    pub name: String,
    pub scenes: usize,
    pub last_opened: Option<u64>,
    pub pinned: bool,
}
pub fn inspect_project(path: &str) -> Result<Project> {
    let root = directory(path)?;
    let file = root.join(project::FILE).canonicalize().map_err(io_error)?;
    if !file.starts_with(&root) {
        return Err(invalid("O manifesto aponta para fora da pasta do projeto"));
    }
    let manifest: Manifest = read_json(&file)?;
    manifest.validate()?;
    Ok(Project {
        path: text_path(&root)?,
        name: manifest.name,
        scenes: manifest.scenes.len(),
        last_opened: None,
        pinned: false,
    })
}
fn read_json<T: serde::de::DeserializeOwned>(file: &Path) -> Result<T> {
    let mut bytes = Vec::new();
    std::fs::File::open(file)
        .map_err(io_error)?
        .take(MAX_LIBRARY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > MAX_LIBRARY_BYTES {
        return Err(invalid("Arquivo de projetos excede256KiB"));
    }
    serde_json::from_slice(&bytes).map_err(io_error)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Library {
    pub version: u32,
    pub projects: Vec<Project>,
    pub favorites: Vec<String>,
}
impl Default for Library {
    fn default() -> Self {
        Self {
            version: 1,
            projects: vec![],
            favorites: vec![],
        }
    }
}
impl Library {
    fn validate(&self) -> Result<()> {
        if self.version != 1
            || self.projects.len() > MAX_PROJECTS
            || self.favorites.len() > MAX_FAVORITES
            || self.projects.iter().filter(|p| p.pinned).count() > 16
        {
            return Err(invalid("Biblioteca de projetos incompatível ou cheia"));
        }
        for (i, p) in self.projects.iter().enumerate() {
            absolute(&p.path)?;
            project::validate_name(&p.name)?;
            if p.scenes > project::MAX_SCENES || self.projects[..i].iter().any(|q| p.path == q.path)
            {
                return Err(invalid("Projeto repetido ou inválido na biblioteca"));
            }
        }
        for (i, p) in self.favorites.iter().enumerate() {
            absolute(p)?;
            if self.favorites[..i].contains(p) {
                return Err(invalid("Favorito repetido"));
            }
        }
        Ok(())
    }
    fn remember(&mut self, mut project: Project) {
        if let Some(previous) = self.projects.iter().find(|p| p.path == project.path) {
            project.pinned = previous.pinned;
        }
        self.projects.retain(|p| p.path != project.path);
        self.projects.insert(0, project);
        if self.projects.len() > MAX_PROJECTS {
            let index = self
                .projects
                .iter()
                .rposition(|p| !p.pinned)
                .unwrap_or(MAX_PROJECTS);
            self.projects.remove(index);
        }
    }
}
#[derive(Default)]
pub struct State {
    pub data: Library,
    pub revision: u64,
    pub warning: Option<String>,
    file: Option<PathBuf>,
}
impl State {
    pub fn load(&mut self, path: &str) -> Result<()> {
        let file = absolute(path)?;
        let data = if file.exists() {
            read_json::<Library>(&file)?
        } else {
            Library::default()
        };
        data.validate()?;
        self.data = data;
        self.file = Some(file);
        self.warning = None;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
    fn save(&self, data: &Library) -> Result<()> {
        data.validate()?;
        if let Some(file) = &self.file {
            let parent = file.parent().ok_or_else(|| invalid("Arquivo sem pasta"))?;
            std::fs::create_dir_all(parent).map_err(io_error)?;
            let bytes = serde_json::to_vec_pretty(data).map_err(io_error)?;
            if bytes.len() as u64 > MAX_LIBRARY_BYTES {
                return Err(invalid("Biblioteca excede256KiB"));
            }
            let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(io_error)?;
            temp.write_all(&bytes).map_err(io_error)?;
            temp.as_file().sync_all().map_err(io_error)?;
            temp.persist(file).map_err(io_error)?;
        }
        Ok(())
    }
    fn publish(&mut self, data: Library) -> Result<()> {
        self.save(&data)?;
        self.data = data;
        if self.file.is_some() {
            self.warning = None;
        }
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
    pub fn remember_opened(&mut self, path: &Path, name: &str, scenes: usize) {
        let Ok(path) = text_path(path) else { return };
        let mut next = self.data.clone();
        next.remember(Project {
            path,
            name: name.into(),
            scenes,
            last_opened: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs()),
            pinned: false,
        });
        if let Err(e) = self.save(&next) {
            self.warning = Some(format!(
                "Projeto aberto; não foi possível salvar os recentes: {}",
                e.message
            ));
        } else if self.file.is_some() {
            self.warning = None;
        }
        self.data = next;
        self.revision = self.revision.wrapping_add(1);
    }
    pub fn register(&mut self, path: &str) -> Result<()> {
        let mut next = self.data.clone();
        let mut project = inspect_project(path)?;
        if let Some(old) = next.projects.iter().find(|p| p.path == project.path) {
            project.last_opened = old.last_opened;
        }
        next.remember(project);
        self.publish(next)
    }
    pub fn pin(&mut self, path: &str, pinned: bool) -> Result<()> {
        let path = if pinned {
            text_path(&directory(path)?)?
        } else {
            record_path(path)?
        };
        let mut next = self.data.clone();
        if pinned
            && next.projects.iter().filter(|p| p.pinned).count() >= 16
            && !next.projects.iter().any(|p| p.path == path && p.pinned)
        {
            return Err(invalid("Limite de16projetos fixados"));
        }
        let entry = next
            .projects
            .iter_mut()
            .find(|p| p.path == path)
            .ok_or_else(|| invalid("Adicione o projeto à biblioteca primeiro"))?;
        entry.pinned = pinned;
        self.publish(next)
    }
    pub fn forget(&mut self, path: &str) -> Result<()> {
        let path = record_path(path)?;
        let mut next = self.data.clone();
        next.projects.retain(|p| p.path != path);
        self.publish(next)
    }
    pub fn favorite(&mut self, path: &str, enabled: bool) -> Result<()> {
        let path = if enabled {
            text_path(&directory(path)?)?
        } else {
            record_path(path)?
        };
        let mut next = self.data.clone();
        next.favorites.retain(|p| *p != path);
        if enabled {
            if next.favorites.len() >= MAX_FAVORITES {
                return Err(invalid("Limite de16pastas favoritas"));
            }
            next.favorites.push(path);
        }
        self.publish(next)
    }
    pub fn discover(&mut self, paths: &[String]) -> Result<serde_json::Value> {
        if paths.len() > 8 {
            return Err(invalid("Escolha até8pastas para procurar projetos"));
        }
        let mut next = self.data.clone();
        let mut found = 0;
        let mut truncated = false;
        let mut errors = vec![];
        let mut candidates = 0;
        for path in paths {
            if let Ok(project) = inspect_project(path) {
                if !next.projects.iter().any(|p| p.path == project.path) {
                    next.remember(project);
                    found += 1;
                }
                continue;
            }
            let mut query = Query {
                path: path.clone(),
                search: String::new(),
                show_hidden: false,
                sort: Sort::Name,
                descending: false,
                after: None,
            };
            for _ in 0..4 {
                match browse(&query) {
                    Ok(page) => {
                        for entry in page.entries.iter().filter(|e| e.kind == Kind::Project) {
                            if candidates >= 64 {
                                truncated = true;
                                break;
                            }
                            candidates += 1;
                            match inspect_project(&entry.path) {
                                Ok(p) => {
                                    if !next.projects.iter().any(|q| q.path == p.path) {
                                        next.remember(p);
                                        found += 1;
                                    }
                                }
                                Err(e) => {
                                    if errors.len() < 32 {
                                        errors.push(e.message);
                                    }
                                }
                            }
                        }
                        query.after = page.next;
                        if query.after.is_none() || candidates >= 64 {
                            break;
                        }
                    }
                    Err(e) => {
                        errors.push(format!("{path}: {}", e.message));
                        query.after = None;
                        break;
                    }
                }
            }
            truncated |= query.after.is_some();
        }
        if found > 0 {
            self.publish(next)?;
        }
        Ok(
            serde_json::json!({"found":found,"truncated":truncated || found>MAX_PROJECTS,"errors":errors,"library":self.data}),
        )
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"library":self.data,"revision":self.revision,"warning":self.warning,"file":self.file})
    }
}
pub fn default_file() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    #[cfg(target_os = "macos")]
    let folder = home.join("Library/Application Support/Ignis");
    #[cfg(not(target_os = "macos"))]
    let folder = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
        .join("ignis");
    Some(folder.join("project-library.json"))
}
pub fn discovery_locations() -> Vec<String> {
    let mut paths = vec![project::default_parent()];
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        for p in [
            home.join("Projects"),
            home.join("Documents/Ignis"),
            home.join("Documents/Ignis Projects"),
        ] {
            if p.is_dir() && !paths.contains(&p) {
                paths.push(p);
            }
        }
    }
    paths
        .into_iter()
        .filter_map(|p| p.to_str().map(str::to_owned))
        .collect()
}
