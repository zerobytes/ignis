//! Project/session operations. Scene and replay formats remain independent.
use crate::{
    engine::{Command, Engine},
    model::{BodyKind, Entity, Result, RoomRecipe, Scene, invalid},
    service::{Service, document_bytes, io_error},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Component, Path, PathBuf},
};

pub const FILE: &str = "ignis.project.json";
pub const MAX_SCENES: usize = 256;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub name: String,
    pub entry_scene: String,
    pub scenes: Vec<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Npc,
    Visual,
    Fixed,
    Dynamic,
}
pub fn validate_name(name: &str) -> Result<()> {
    if name.trim() != name
        || name.is_empty()
        || name.len() > 80
        || name == "."
        || name == ".."
        || name
            .chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
    {
        return Err(invalid(
            "Use a name of1..80 characters without path separators or reserved characters",
        ));
    }
    Ok(())
}
fn scene_path(name: &str) -> Result<String> {
    validate_name(name)?;
    Ok(format!("scenes/{}.scene.json", name.replace(' ', "_")))
}
fn valid_scene_path(path: &str) -> bool {
    path.len() <= 512
        && !path.contains("//")
        && path.starts_with("scenes/")
        && path.ends_with(".scene.json")
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
impl Manifest {
    pub fn validate(&self) -> Result<()> {
        validate_name(&self.name)?;
        if self.version != 1
            || self.scenes.is_empty()
            || self.scenes.len() > MAX_SCENES
            || !self.scenes.contains(&self.entry_scene)
            || self
                .scenes
                .iter()
                .enumerate()
                .any(|(i, p)| !valid_scene_path(p) || self.scenes[..i].contains(p))
        {
            return Err(invalid(
                "Invalid project version, scene list or entry scene",
            ));
        }
        Ok(())
    }
}
pub fn initial_scene(name: &str) -> Result<Scene> {
    validate_name(name)?;
    let mut scene = RoomRecipe {
        obstacles: 0,
        ..Default::default()
    }
    .generate()?;
    scene.name = name.into();
    scene.recipe = None;
    scene
        .entities
        .retain(|e| e.id == "floor" || e.id == "player");
    scene
        .entities
        .iter_mut()
        .filter(|e| e.id == "player")
        .for_each(|e| e.transform.position = [0., 0.85, 0.]);
    scene.validate()?;
    Ok(scene)
}
pub fn default_parent() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"));
    if home.join("Documents").is_dir() {
        home.join("Documents")
    } else {
        home
    }
}
fn absolute(path: &str) -> Result<PathBuf> {
    let p = Path::new(path);
    if !p.is_absolute()
        || path.len() > 2048
        || p.components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(invalid("Choose an absolute project folder without . or .."));
    }
    Ok(p.into())
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| invalid("Missing parent"))?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(io_error)?;
    temp.write_all(bytes).map_err(io_error)?;
    temp.as_file().sync_all().map_err(io_error)?;
    temp.persist_noclobber(path).map_err(io_error)?;
    Ok(())
}
impl Service {
    pub fn scene_dirty(&self) -> bool {
        self.saved_scene_revision != self.engine.revision
    }
    pub fn project_status(&self) -> Value {
        json!({"project":self.project,"root":self.root,"scene_path":self.scene_path,"modified":self.scene_dirty(),"default_parent":default_parent(),"limits":{"scenes":MAX_SCENES}})
    }
    fn check_discard(&self, discard: bool) -> Result<()> {
        if self.engine.revision == u64::MAX {
            return Err(invalid("Revision exhausted"));
        }
        if self.scene_dirty() && !discard {
            return Err(invalid(
                "A cena tem alterações não salvas. Salve ou confirme descartar antes de continuar.",
            ));
        }
        Ok(())
    }
    fn install_project_scene(&mut self, mut engine: Engine, path: String) -> Result<()> {
        engine.set_execution_mode(self.engine.execution_mode())?;
        engine.revision = self
            .engine
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Revision exhausted"))?;
        self.engine = engine;
        self.scene_path = Some(path);
        self.saved_scene_revision = self.engine.revision;
        for w in &mut self.editor.workspaces {
            w.camera = None;
        }
        Ok(())
    }
    pub(crate) fn create_project(
        &mut self,
        path: &str,
        name: &str,
        discard: bool,
    ) -> Result<Value> {
        self.check_discard(discard)?;
        validate_name(name)?;
        let root = absolute(path)?;
        let parent = root
            .parent()
            .ok_or_else(|| invalid("Choose a new child folder"))?;
        let scene = initial_scene("Principal")?;
        let engine = Engine::new(scene.clone())?;
        let manifest = Manifest {
            version: 1,
            name: name.into(),
            entry_scene: scene_path("Principal")?,
            scenes: vec![scene_path("Principal")?],
        };
        let scene_bytes = document_bytes(&scene)?;
        let manifest_bytes = document_bytes(&manifest)?;
        // Reserve the destination atomically, never replace an existing folder.
        std::fs::create_dir_all(parent).map_err(io_error)?;
        std::fs::create_dir(&root).map_err(io_error)?;
        let result = (|| {
            std::fs::create_dir(root.join("scenes")).map_err(io_error)?;
            write_new(&root.join(&manifest.entry_scene), &scene_bytes)?;
            write_new(&root.join(FILE), &manifest_bytes)?;
            root.canonicalize().map_err(io_error)
        })();
        let canonical = match result {
            Ok(root) => root,
            Err(e) => {
                let _ = std::fs::remove_file(root.join(&manifest.entry_scene));
                let _ = std::fs::remove_file(root.join(FILE));
                let _ = std::fs::remove_dir(root.join("scenes"));
                let _ = std::fs::remove_dir(&root);
                return Err(e);
            }
        };
        self.install_project_scene(engine, manifest.entry_scene.clone())?;
        self.root = canonical;
        self.project = Some(manifest);
        self.project_library.remember_opened(
            &self.root,
            &self.project.as_ref().unwrap().name,
            self.project.as_ref().unwrap().scenes.len(),
        );
        Ok(self.project_status())
    }
    pub(crate) fn open_project(&mut self, path: &str, discard: bool) -> Result<Value> {
        self.check_discard(discard)?;
        let root = absolute(path)?;
        // Temporary service parses candidates and checks paths before switching.
        let mut next = Service::new(&root, initial_scene("Principal")?)?;
        let manifest: Manifest = next.read(FILE)?;
        manifest.validate()?;
        for p in &manifest.scenes {
            if !next.resolve(p)?.is_file() {
                return Err(invalid(format!("Scene missing: {p}")));
            }
        }
        let scene = next.read_scene(&manifest.entry_scene)?;
        let engine = Engine::new(scene)?;
        self.install_project_scene(engine, manifest.entry_scene.clone())?;
        self.root = std::mem::take(&mut next.root);
        self.project = Some(manifest);
        self.project_library.remember_opened(
            &self.root,
            &self.project.as_ref().unwrap().name,
            self.project.as_ref().unwrap().scenes.len(),
        );
        Ok(self.project_status())
    }
    pub(crate) fn new_project_scene(&mut self, name: &str, discard: bool) -> Result<Value> {
        self.check_discard(discard)?;
        let mut manifest = self
            .project
            .clone()
            .ok_or_else(|| invalid("Crie ou abra um projeto primeiro"))?;
        let path = scene_path(name)?;
        if manifest.scenes.len() >= MAX_SCENES || manifest.scenes.contains(&path) {
            return Err(invalid("Scene already exists or scene limit reached"));
        }
        let scene = initial_scene(name)?;
        let engine = Engine::new(scene.clone())?;
        manifest.scenes.push(path.clone());
        manifest.validate()?;
        let dest = self.resolve(&path)?;
        write_new(&dest, &document_bytes(&scene)?)?;
        if let Err(e) = self.write(FILE, &document_bytes(&manifest)?) {
            let _ = std::fs::remove_file(&dest);
            return Err(e);
        }
        self.install_project_scene(engine, path)?;
        self.project = Some(manifest);
        self.project_library.remember_opened(
            &self.root,
            &self.project.as_ref().unwrap().name,
            self.project.as_ref().unwrap().scenes.len(),
        );
        Ok(self.project_status())
    }
    pub(crate) fn open_project_scene(&mut self, path: &str, discard: bool) -> Result<Value> {
        self.check_discard(discard)?;
        if let Some(m) = &self.project
            && !m.scenes.iter().any(|p| p == path)
        {
            return Err(invalid("Scene is not registered in this project"));
        }
        let scene = self.read_scene(path)?;
        let engine = Engine::new(scene)?;
        self.install_project_scene(engine, path.into())?;
        Ok(self.project_status())
    }
    pub(crate) fn set_project_entry(&mut self, path: &str) -> Result<Value> {
        let mut manifest = self
            .project
            .clone()
            .ok_or_else(|| invalid("No project open"))?;
        if !manifest.scenes.iter().any(|p| p == path) {
            return Err(invalid("Unknown project scene"));
        }
        manifest.entry_scene = path.into();
        manifest.validate()?;
        self.write(FILE, &document_bytes(&manifest)?)?;
        self.project = Some(manifest);
        Ok(self.project_status())
    }
    pub(crate) fn create_scene_object(
        &mut self,
        kind: ObjectKind,
        duplicate: Option<&str>,
    ) -> Result<Value> {
        let id = (0..=crate::model::MAX_ENTITIES)
            .map(|i| format!("object_{i}"))
            .find(|id| !self.engine.scene.entities.iter().any(|e| e.id == *id))
            .ok_or_else(|| invalid("Object limit reached"))?;
        let mut entity: Entity = if let Some(source) = duplicate {
            self.engine
                .scene
                .entities
                .iter()
                .find(|e| e.id == source)
                .cloned()
                .ok_or_else(|| invalid("Select an authored object"))?
        } else {
            serde_json::from_value(json!({"id":id,"name":"Objeto","transform":{"position":[0.,1.,0.],"size":[1.,1.,1.]},"color":[0.65,0.65,0.65],"body":null})).map_err(io_error)?
        };
        if duplicate.is_some() {
            entity.name = format!("{} cópia", entity.name);
            entity.transform.position[0] += 1.;
        } else {
            entity.body = match kind {
                ObjectKind::Npc => Some(BodyKind::Npc),
                ObjectKind::Visual => None,
                ObjectKind::Fixed => Some(BodyKind::Fixed),
                ObjectKind::Dynamic => Some(BodyKind::Dynamic),
            };
            entity.name = match kind {
                ObjectKind::Npc => "NPC",
                ObjectKind::Visual => "Cubo visual",
                ObjectKind::Fixed => "Bloco sólido",
                ObjectKind::Dynamic => "Caixa física",
            }
            .into();
        }
        if duplicate.is_none() {
            let n = id.trim_start_matches("object_").parse::<u32>().unwrap_or(0);
            entity.transform.position = [2. + (n % 4) as f32 * 1.5, 1., -2. - (n / 4) as f32 * 1.5];
        }
        if duplicate.is_none() && kind == ObjectKind::Npc {
            let surface = self
                .engine
                .scene
                .entities
                .iter()
                .find(|e| e.body == Some(BodyKind::Fixed) && e.geometry.is_box())
                .ok_or_else(|| invalid("Crie um piso sólido antes de adicionar um NPC"))?;
            entity.transform.size = [0.7, 1.6, 0.7];
            entity.transform.position[1] =
                surface.transform.position[1] + surface.transform.size[1] * 0.5 + 0.8;
            entity.npc = Some(
                serde_json::from_value(
                    json!({"surface":surface.id,"speed":2.,"stop_distance":1.,"enabled":true}),
                )
                .map_err(io_error)?,
            );
        }
        entity.id = id.clone();
        self.engine.execute(
            Command::Upsert {
                entity: Box::new(entity),
            }
            .into(),
        )?;
        let result = self
            .engine
            .execute(Command::SelectEntity { id: Some(id) }.into())?;
        self.selected_mesh = None;
        self.session_revision += 1;
        Ok(result)
    }
}
