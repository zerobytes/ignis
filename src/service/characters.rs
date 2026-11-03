//! File IO and editor mutations share the same command boundary as native widgets.
use super::*;
use crate::{
    character_editor::{self, Binding, Recipe},
    engine::SceneEdit,
    meshes::{self, character::Source},
};
use std::sync::Arc;

impl Service {
    fn read_character(&self, path: &str) -> Result<(Arc<Source>, String)> {
        if !path.ends_with(".ignis-character") {
            return Err(invalid("Character source requires .ignis-character suffix"));
        }
        let file = std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
        let mut bytes = Vec::new();
        file.take((meshes::character::MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        let source = meshes::character::decode(&bytes)?;
        Ok((Arc::new(source), character_editor::sha256(&bytes)))
    }
    fn character_binding(&self, prefix: &str) -> Result<Binding> {
        self.engine
            .scene
            .characters
            .iter()
            .find(|b| b.recipe.placement.prefix == prefix)
            .cloned()
            .ok_or_else(|| invalid("Character recipe not found"))
    }
    fn load_character_source(&mut self, prefix: &str) -> Result<()> {
        let binding = self.character_binding(prefix)?;
        let (source, hash) = self.read_character(&binding.recipe.source_path)?;
        if hash != binding.recipe.source_sha256 {
            return Err(invalid(
                "Character source hash changed; restore the saved source before editing this recipe",
            ));
        }
        self.character_editor
            .install(binding.recipe.source_path, hash, source);
        self.character_editor
            .choose(Some(prefix.into()), &self.engine.scene);
        self.character_editor
            .refresh(&self.engine.scene, self.engine.revision);
        Ok(())
    }
    fn character_editable(&mut self, prefix: &str) -> Result<Binding> {
        let binding = self.character_binding(prefix)?;
        self.character_editor
            .choose(Some(prefix.into()), &self.engine.scene);
        self.character_editor
            .refresh(&self.engine.scene, self.engine.revision);
        if !self.character_editor.ready {
            return Err(invalid(
                self.character_editor
                    .reason
                    .as_deref()
                    .unwrap_or("Load the character source before editing"),
            ));
        }
        Ok(binding)
    }
    fn install_character(
        &mut self,
        recipe: Recipe,
        document: meshes::Document,
        gesture: Option<&str>,
        connect_only: bool,
    ) -> Result<Value> {
        if self.engine.playing {
            return Err(invalid("Pause before editing a character"));
        }
        let binding = Binding::from_document(recipe, &document)?;
        if connect_only && !character_editor::current_bake_matches(&self.engine.scene, &binding) {
            return Err(invalid(
                "The scene differs from this source/configuration. Connect the matching recipe, or publish a new base with a different prefix",
            ));
        }
        let mut edits = Vec::new();
        if !connect_only {
            edits.extend(document.assets.into_iter().map(|asset| SceneEdit::Mesh {
                edit: meshes::Edit::PutAsset { asset },
            }));
            edits.extend(
                document
                    .instances
                    .into_iter()
                    .map(|instance| SceneEdit::Mesh {
                        edit: meshes::Edit::PutInstance { instance },
                    }),
            );
        }
        edits.push(SceneEdit::SetCharacterBinding {
            binding: Box::new(binding.clone()),
        });
        let command = match gesture {
            Some(gesture) => Command::EditSceneLive {
                edits,
                gesture: format!("character/{}/{gesture}", binding.recipe.placement.prefix),
            },
            None => Command::EditScene { edits },
        };
        let result = self.engine.execute(command.into())?;
        self.character_editor
            .accepted(&binding, self.engine.revision);
        Ok(json!({"revision":result["revision"],"binding":binding,"editor_only":true}))
    }
    pub(super) fn character_command(&mut self, command: &Command) -> Result<Value> {
        match command {
            Command::InspectCharacterSource { path } => self.read_character(path)?.0.inspect(),
            Command::UpgradeCharacterSource {
                prefix,
                path,
                reference,
            } => {
                if *reference != self.engine.mesh_reference() {
                    return Err(invalid("Stale character source upgrade reference"));
                }
                let old = self.character_editable(prefix)?;
                let (source, hash) = self.read_character(path)?;
                let configuration = source.canonical_configuration(&old.recipe.configuration)?;
                let (document, mut report) =
                    source.compile_cached(&configuration, &old.recipe.placement)?;
                let old_parts: std::collections::BTreeSet<_> = old.parts.iter().collect();
                let new_parts: std::collections::BTreeSet<_> =
                    document.assets.iter().map(|a| &a.id).collect();
                if old_parts != new_parts {
                    return Err(invalid(
                        "Source upgrade must preserve the owned part identities",
                    ));
                }
                let recipe = Recipe {
                    version: character_editor::RECIPE_VERSION,
                    source_path: path.clone(),
                    source_sha256: hash.clone(),
                    configuration,
                    placement: old.recipe.placement,
                };
                self.install_character(recipe, document, None, false)?;
                self.character_editor
                    .install(path.clone(), hash.clone(), source);
                self.character_editor
                    .refresh(&self.engine.scene, self.engine.revision);
                report["source_sha256"] = json!(hash);
                report["revision"] = json!(self.engine.revision);
                Ok(report)
            }
            Command::PublishCharacter {
                path,
                configuration,
                placement,
            }
            | Command::ConnectCharacterSource {
                path,
                configuration,
                placement,
            } => {
                let connect_only = matches!(command, Command::ConnectCharacterSource { .. });
                if self.engine.playing {
                    return Err(invalid("Pause before publishing a character"));
                }
                if !connect_only {
                    if self
                        .engine
                        .scene
                        .characters
                        .iter()
                        .any(|b| b.recipe.placement.prefix == placement.prefix)
                    {
                        self.character_editable(&placement.prefix)?;
                    } else if self
                        .engine
                        .scene
                        .meshes
                        .assets
                        .iter()
                        .any(|a| a.id.starts_with(&format!("{}_", placement.prefix)))
                        || self
                            .engine
                            .scene
                            .meshes
                            .instances
                            .iter()
                            .any(|i| i.id.starts_with(&format!("{}_", placement.prefix)))
                    {
                        return Err(invalid(
                            "This prefix already owns scene meshes; connect its matching recipe or choose another prefix",
                        ));
                    }
                }
                let (source, hash) = self.read_character(path)?;
                if !connect_only
                    && let Some(existing) = self
                        .engine
                        .scene
                        .characters
                        .iter()
                        .find(|b| b.recipe.placement.prefix == placement.prefix)
                    && (existing.recipe.source_sha256 != hash
                        || existing.recipe.source_path != *path
                        || existing.recipe.placement != *placement)
                {
                    return Err(invalid(
                        "Publishing an existing character requires the same source and placement; choose a new prefix for another base",
                    ));
                }
                let configuration = source.canonical_configuration(configuration)?;
                let (document, mut report) = source.compile_cached(&configuration, placement)?;
                let recipe = Recipe {
                    version: character_editor::RECIPE_VERSION,
                    source_path: path.clone(),
                    source_sha256: hash.clone(),
                    configuration,
                    placement: placement.clone(),
                };
                self.install_character(recipe, document, None, connect_only)?;
                self.character_editor
                    .install(path.clone(), hash.clone(), source);
                self.character_editor
                    .refresh(&self.engine.scene, self.engine.revision);
                report["source_sha256"] = json!(hash);
                report["revision"] = json!(self.engine.revision);
                report["binding"] = json!(self.character_binding(&placement.prefix)?);
                Ok(report)
            }
            Command::OpenCharacterPanel { open, prefix } => {
                if let Some(p) = prefix {
                    self.character_binding(p)?;
                }
                self.character_editor.open = *open;
                self.character_editor
                    .choose(prefix.clone(), &self.engine.scene);
                if *open
                    && let Some(binding) =
                        self.character_editor.binding(&self.engine.scene).cloned()
                    && !self.character_editor.has_source(&binding)
                    && let Err(error) = self.load_character_source(&binding.recipe.placement.prefix)
                {
                    self.character_editor.reason = Some(error.message);
                }
                Ok(self.character_editor.inspect(&self.engine.scene))
            }
            Command::InspectCharacterEditor => {
                let mut result = self.character_editor.inspect(&self.engine.scene);
                result["reference"] = json!(self.engine.mesh_reference());
                Ok(result)
            }
            Command::SetCharacterHandleView { view } => {
                if view.group.as_ref().is_some_and(|g| {
                    !self
                        .character_editor
                        .handles
                        .iter()
                        .any(|h| h.group.as_ref() == Some(g))
                }) {
                    return Err(invalid("Unknown character handle group"));
                }
                if view.enabled
                    && (!self.character_editor.ready || self.character_editor.handles.is_empty())
                {
                    return Err(invalid(
                        "Load a synchronized source with facial handles before enabling direct editing",
                    ));
                }
                if let Some(name) = &view.selected {
                    self.character_editor
                        .source
                        .as_ref()
                        .ok_or_else(|| invalid("Character source not loaded"))?
                        .handle(name)?;
                }
                self.character_editor.handle_view = view.clone();
                Ok(self.character_editor.inspect(&self.engine.scene))
            }
            Command::SetCharacterHandle {
                prefix,
                handle,
                offset,
                symmetry,
                reference,
                gesture,
            } => {
                self.character_editable(prefix)?;
                let prepared = self.character_editor.prepare_handle(
                    &self.engine,
                    prefix,
                    handle,
                    *offset,
                    *symmetry,
                    reference,
                )?;
                let result = self.install_character(
                    prepared.recipe,
                    prepared.document,
                    gesture.as_deref(),
                    false,
                )?;
                self.character_editor.handle_view.selected = Some(handle.clone());
                Ok(result)
            }
            Command::PreviewCharacterHandle {
                prefix,
                handle,
                offset,
                symmetry,
                reference,
            } => {
                self.character_editable(prefix)?;
                let prepared = self.character_editor.prepare_handle(
                    &self.engine,
                    prefix,
                    handle,
                    *offset,
                    *symmetry,
                    reference,
                )?;
                // Run the exact native preview validation without publishing it.
                prepared.preview(&self.engine)?;
                let binding = Binding::from_document(prepared.recipe.clone(), &prepared.document)?;
                let handles = self
                    .character_editor
                    .source
                    .as_ref()
                    .unwrap()
                    .handle_poses(
                        &prepared.recipe.configuration,
                        &prepared.recipe.placement,
                        &prepared.document,
                    )?;
                Ok(
                    json!({"world_changed":false,"revision":self.engine.revision,"reference":reference,"binding":binding,"handles":handles}),
                )
            }
            Command::LoadCharacterEditorSource { prefix } => {
                self.load_character_source(prefix)?;
                Ok(self.character_editor.inspect(&self.engine.scene))
            }
            Command::SetCharacterParameters {
                prefix,
                configuration,
                gesture,
            } => {
                let mut binding = self.character_editable(prefix)?;
                binding
                    .recipe
                    .configuration
                    .parameters
                    .extend(configuration.parameters.clone());
                let source = self.character_editor.source.as_ref().unwrap();
                binding.recipe.configuration =
                    source.canonical_configuration(&binding.recipe.configuration)?;
                let (document, _) = source
                    .compile_cached(&binding.recipe.configuration, &binding.recipe.placement)?;
                self.install_character(binding.recipe, document, gesture.as_deref(), false)
            }
            Command::RegenerateCharacter { prefix } => {
                let binding = self.character_binding(prefix)?;
                if !self.character_editor.has_source(&binding) {
                    return Err(invalid(
                        "Load the saved character source before regenerating",
                    ));
                }
                let (document, _) = self
                    .character_editor
                    .source
                    .as_ref()
                    .unwrap()
                    .compile_cached(&binding.recipe.configuration, &binding.recipe.placement)?;
                // Existing owned part identities must remain fixed; no unrelated meshes can be replaced.
                if document
                    .assets
                    .iter()
                    .map(|a| &a.id)
                    .ne(binding.parts.iter())
                {
                    return Err(invalid("Character source part identities changed"));
                }
                self.install_character(binding.recipe, document, None, false)
            }
            Command::SaveCharacterConfiguration { prefix, path } => {
                if !path.ends_with(".ignis-character-config") {
                    return Err(invalid(
                        "Configuration requires .ignis-character-config suffix",
                    ));
                }
                let binding = self.character_binding(prefix)?;
                let bytes = serde_json::to_vec_pretty(&binding.recipe).map_err(io_error)?;
                if bytes.len() > character_editor::MAX_RECIPE_BYTES {
                    return Err(invalid("Character configuration exceeds64KiB"));
                }
                let dest = self.write(path, &bytes)?;
                Ok(json!({"path":dest,"bytes":bytes.len(),"world_changed":false}))
            }
            Command::LoadCharacterConfiguration { path } => {
                if !path.ends_with(".ignis-character-config") {
                    return Err(invalid(
                        "Configuration requires .ignis-character-config suffix",
                    ));
                }
                let file = std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
                let mut bytes = Vec::new();
                file.take((character_editor::MAX_RECIPE_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(io_error)?;
                if bytes.len() > character_editor::MAX_RECIPE_BYTES {
                    return Err(invalid("Character configuration exceeds64KiB"));
                }
                let mut recipe: Recipe = serde_json::from_slice(&bytes).map_err(io_error)?;
                recipe.validate()?;
                let existing = self.character_editable(&recipe.placement.prefix)?;
                if existing.recipe.source_sha256 != recipe.source_sha256
                    || existing.recipe.source_path != recipe.source_path
                    || existing.recipe.placement != recipe.placement
                {
                    return Err(invalid(
                        "Import requires the same character source and placement; configurations cannot replace another base",
                    ));
                }
                let source = self.character_editor.source.as_ref().unwrap();
                recipe.configuration = source.canonical_configuration(&recipe.configuration)?;
                let (document, _) =
                    source.compile_cached(&recipe.configuration, &recipe.placement)?;
                self.install_character(recipe, document, None, false)
            }
            _ => Err(invalid("Unknown character editor command")),
        }
    }
}
