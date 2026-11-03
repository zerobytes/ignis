//! Authoring recipes persist with their baked meshes; loaded source data is session-only.
use crate::{
    meshes::{
        self,
        character::{Configuration, Placement, Source},
    },
    model::{Result, Scene, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io::Write,
    path::{Component, Path},
    sync::Arc,
};
mod handles;
pub use handles::{HandleView, Prepared};

pub const RECIPE_VERSION: u16 = 2;
pub const MAX_RECIPE_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub version: u16,
    pub source_path: String,
    pub source_sha256: String,
    pub configuration: Configuration,
    pub placement: Placement,
}
impl Recipe {
    pub fn validate(&self) -> Result<()> {
        if !(1..=RECIPE_VERSION).contains(&self.version)
            || self.source_path.len() > 1024
            || !self.source_path.ends_with(".ignis-character")
            || !Path::new(&self.source_path)
                .components()
                .all(|p| matches!(p, Component::Normal(_)))
            || !digest_valid(&self.source_sha256)
            || self.configuration.parameters.len()
                > if self.version == 1 {
                    256
                } else {
                    crate::meshes::character::MAX_PARAMETERS
                }
            || self
                .configuration
                .parameters
                .iter()
                .any(|(name, v)| !identifier(name) || !v.is_finite())
        {
            return Err(invalid(
                "Invalid character recipe version, source path/hash or parameters",
            ));
        }
        self.placement.validate()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub recipe: Recipe,
    pub parts: Vec<String>,
    /// Fingerprint of this recipe's complete baked assets/instances, not the source file.
    pub bake_sha256: String,
}
impl Binding {
    pub fn validate(&self) -> Result<()> {
        self.recipe.validate()?;
        let mut ids = BTreeSet::new();
        if self.parts.is_empty()
            || self.parts.len() > 32
            || !digest_valid(&self.bake_sha256)
            || self.parts.iter().any(|v| !identifier(v) || !ids.insert(v))
        {
            return Err(invalid(
                "Invalid character binding parts or bake fingerprint",
            ));
        }
        Ok(())
    }
    pub fn from_document(recipe: Recipe, document: &meshes::Document) -> Result<Self> {
        let binding = Self {
            recipe,
            parts: document.assets.iter().map(|a| a.id.clone()).collect(),
            bake_sha256: fingerprint(document)?,
        };
        binding.validate()?;
        Ok(binding)
    }
}
pub fn validate(scene: &Scene) -> Result<()> {
    if scene.version < 106 && !scene.characters.is_empty() {
        return Err(invalid("Character authoring recipes require scene106"));
    }
    let mut prefixes = BTreeSet::new();
    let mut parts = BTreeSet::new();
    for binding in &scene.characters {
        binding.validate()?;
        if binding.recipe.version >= 2 && scene.version < 109 {
            return Err(invalid("Character recipe2 requires scene109"));
        }
        if !prefixes.insert(&binding.recipe.placement.prefix) {
            return Err(invalid("Duplicate character recipe prefix"));
        }
        if binding.parts.iter().any(|p| !parts.insert(p)) {
            return Err(invalid("Character recipes cannot own overlapping parts"));
        }
    }
    Ok(())
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn digest_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}
pub fn sha256(bytes: &[u8]) -> String {
    hex(ring::digest::digest(&ring::digest::SHA256, bytes).as_ref())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
struct DigestWriter(ring::digest::Context);
impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub fn fingerprint(document: &meshes::Document) -> Result<String> {
    let mut writer = DigestWriter(ring::digest::Context::new(&ring::digest::SHA256));
    serde_json::to_writer(&mut writer, document).map_err(|e| invalid(e.to_string()))?;
    Ok(hex(writer.0.finish().as_ref()))
}
pub fn document(scene: &Scene, binding: &Binding) -> Result<meshes::Document> {
    let mut result = meshes::Document::default();
    for id in &binding.parts {
        result.assets.push(scene.meshes.asset(id)?.clone());
        result.instances.push(
            scene
                .meshes
                .instances
                .iter()
                .find(|i| i.id == *id)
                .ok_or_else(|| invalid("Character part is absent from the scene"))?
                .clone(),
        );
    }
    Ok(result)
}
pub fn current_bake_matches(scene: &Scene, binding: &Binding) -> bool {
    document(scene, binding)
        .and_then(|d| fingerprint(&d))
        .is_ok_and(|hash| hash == binding.bake_sha256)
}

#[derive(Default)]
pub struct State {
    pub open: bool,
    pub selected: Option<String>,
    pub source: Option<Arc<Source>>,
    pub ready: bool,
    pub reason: Option<String>,
    pub handle_view: HandleView,
    pub handles: Vec<crate::meshes::character::HandlePose>,
    handles_revision: Option<u64>,
    path: String,
    sha: String,
    checked_revision: Option<u64>,
    checked_binding: Option<Binding>,
    verified_bake: Option<String>,
}
impl State {
    pub fn binding<'a>(&self, scene: &'a Scene) -> Option<&'a Binding> {
        let selected = self.selected.as_ref()?;
        scene
            .characters
            .iter()
            .find(|b| &b.recipe.placement.prefix == selected)
    }
    pub fn has_source(&self, binding: &Binding) -> bool {
        self.source.is_some()
            && self.path == binding.recipe.source_path
            && self.sha == binding.recipe.source_sha256
    }
    pub fn install(&mut self, path: String, sha: String, source: Arc<Source>) {
        self.source = Some(source);
        self.path = path;
        self.sha = sha;
        self.checked_revision = None;
        self.checked_binding = None;
        self.verified_bake = None;
        self.handles_revision = None;
    }
    pub fn choose(&mut self, prefix: Option<String>, scene: &Scene) {
        let prefix = prefix
            .or_else(|| {
                self.selected.clone().filter(|p| {
                    scene
                        .characters
                        .iter()
                        .any(|b| b.recipe.placement.prefix == *p)
                })
            })
            .or_else(|| {
                scene
                    .characters
                    .first()
                    .map(|b| b.recipe.placement.prefix.clone())
            });
        if self.selected != prefix {
            self.selected = prefix;
            self.checked_revision = None;
            self.checked_binding = None;
            self.verified_bake = None;
            self.handles_revision = None;
        }
    }
    /// Called after commands, never from a vertex loop or on an unchanged frame.
    pub fn refresh(&mut self, scene: &Scene, revision: u64) {
        self.refresh_integrity(scene, revision);
        self.refresh_handles(scene, revision);
    }
    fn refresh_integrity(&mut self, scene: &Scene, revision: u64) {
        if self.checked_revision == Some(revision) {
            return;
        }
        self.checked_revision = Some(revision);
        self.ready = false;
        let Some(binding) = self.binding(scene).cloned() else {
            self.reason = None;
            self.checked_binding = None;
            return;
        };
        if !self.has_source(&binding) {
            self.reason = Some("Carregue a fonte para editar os parâmetros.".into());
            return;
        }
        if self.checked_binding.as_ref() != Some(&binding) {
            self.verified_bake = None;
            let result = self
                .source
                .as_ref()
                .unwrap()
                .compile_cached(&binding.recipe.configuration, &binding.recipe.placement)
                .and_then(|(d, _)| {
                    if d.assets
                        .iter()
                        .map(|a| a.id.as_str())
                        .ne(binding.parts.iter().map(String::as_str))
                    {
                        return Err(invalid(
                            "Character recipe part identities differ from the source",
                        ));
                    }
                    fingerprint(&d)
                });
            match result {
                Ok(hash) if hash == binding.bake_sha256 => self.verified_bake = Some(hash),
                _ => {
                    self.reason = Some("A configuração salva não corresponde à base gerada.".into())
                }
            }
            self.checked_binding = Some(binding.clone());
        }
        if self.verified_bake.as_deref() != Some(binding.bake_sha256.as_str()) {
            return;
        }
        if !current_bake_matches(scene, &binding) {
            self.reason=Some("Esta base recebeu edições manuais. Desfaça as alterações ou regenere a partir dos parâmetros.".into());
            return;
        }
        self.ready = true;
        self.reason = None;
    }
    pub fn accepted(&mut self, binding: &Binding, revision: u64) {
        self.selected = Some(binding.recipe.placement.prefix.clone());
        self.checked_binding = Some(binding.clone());
        self.verified_bake = Some(binding.bake_sha256.clone());
        self.checked_revision = Some(revision);
        self.ready = true;
        self.reason = None;
        self.handles_revision = None;
    }
    pub fn inspect(&self, scene: &Scene) -> serde_json::Value {
        let binding = self.binding(scene);
        serde_json::json!({"open":self.open,"selected":self.selected,"ready":self.ready,"reason":self.reason,
            "binding":binding,"source_loaded":binding.is_some_and(|b|self.has_source(b)),"handles":self.handles,"handle_view":self.handle_view,
            "parameters":self.source.as_ref().filter(|_|binding.is_some_and(|b|self.has_source(b))).map(|s|&s.parameters),
            "sources":scene.characters.iter().map(|b|&b.recipe.placement.prefix).collect::<Vec<_>>(),"editor_only":true})
    }
}
