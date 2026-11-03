//! A shared, read-only preparation boundary for viewport previews and MCP commits.
use super::*;
use crate::{
    engine::{Engine, SceneEdit},
    mesh_topology::Reference,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HandleView {
    pub enabled: bool,
    pub symmetry: bool,
    pub selected: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
}
impl Default for HandleView {
    fn default() -> Self {
        Self {
            enabled: false,
            symmetry: true,
            selected: None,
            group: None,
        }
    }
}
pub struct Prepared {
    pub recipe: Recipe,
    pub document: meshes::Document,
}
impl Prepared {
    pub fn preview(&self, engine: &Engine) -> Result<Scene> {
        let binding = Binding::from_document(self.recipe.clone(), &self.document)?;
        let mut edits: Vec<_> = self
            .document
            .assets
            .iter()
            .cloned()
            .map(|asset| SceneEdit::Mesh {
                edit: meshes::Edit::PutAsset { asset },
            })
            .chain(
                self.document
                    .instances
                    .iter()
                    .cloned()
                    .map(|instance| SceneEdit::Mesh {
                        edit: meshes::Edit::PutInstance { instance },
                    }),
            )
            .collect();
        edits.push(SceneEdit::SetCharacterBinding {
            binding: Box::new(binding),
        });
        engine.preview_edits(edits)
    }
}
impl State {
    pub(super) fn refresh_handles(&mut self, scene: &Scene, revision: u64) {
        if self.handles_revision == Some(revision) {
            return;
        }
        self.handles_revision = Some(revision);
        self.handles.clear();
        if self.ready
            && let Some(binding) = self.binding(scene)
            && self.has_source(binding)
        {
            let result = document(scene, binding).and_then(|d| {
                self.source.as_ref().unwrap().handle_poses(
                    &binding.recipe.configuration,
                    &binding.recipe.placement,
                    &d,
                )
            });
            match result {
                Ok(poses) => self.handles = poses,
                Err(error) => {
                    self.ready = false;
                    self.reason = Some(error.message);
                }
            }
        }
        if self
            .handle_view
            .group
            .as_ref()
            .is_some_and(|g| !self.handles.iter().any(|h| h.group.as_ref() == Some(g)))
        {
            self.handle_view.group = None;
        }
        if self
            .handle_view
            .selected
            .as_ref()
            .is_some_and(|name| !self.handles.iter().any(|h| h.name == *name))
        {
            self.handle_view.selected = None;
        }
    }
    pub fn prepare_handle(
        &self,
        engine: &Engine,
        prefix: &str,
        name: &str,
        offset: [f32; 3],
        symmetry: bool,
        reference: &Reference,
    ) -> Result<Prepared> {
        if reference != &engine.mesh_reference() {
            return Err(invalid(
                "Character handle reference is stale; inspect again before editing",
            ));
        }
        if engine.playing
            || !self.ready
            || self.checked_revision != Some(engine.revision)
            || self.selected.as_deref() != Some(prefix)
        {
            return Err(invalid(
                "Character handle requires the synchronized, selected base and paused editor",
            ));
        }
        let binding = self
            .binding(&engine.scene)
            .ok_or_else(|| invalid("Character recipe missing"))?;
        if !self.has_source(binding) {
            return Err(invalid("Character source is not loaded"));
        }
        let source = self.source.as_ref().unwrap();
        let mut recipe = binding.recipe.clone();
        recipe.configuration =
            source.handle_configuration(&recipe.configuration, name, offset, symmetry)?;
        let (document, _) = source.compile_cached(&recipe.configuration, &recipe.placement)?;
        Ok(Prepared { recipe, document })
    }
}
