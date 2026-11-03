//! Native vegetation authoring; a validated bake and its source commit together.
use super::*;
use crate::{
    botany::{self as b, ManualPolicy as Policy, Operation as Op, Recipe, bake},
    engine::SceneEdit,
    meshes,
};
use glam::DVec3;

impl Service {
    fn botanical_recipe(&self, prefix: &str) -> Result<Recipe> {
        self.engine
            .scene
            .botany
            .iter()
            .find(|r| r.prefix == prefix)
            .cloned()
            .ok_or_else(|| invalid("Botanical recipe not found"))
    }
    /// Recover vertex edits against the previous semantic bake, never by nearest point.
    fn botanical_preserve(&self, mut next: Recipe, policy: Policy) -> Result<Recipe> {
        next.validate()?;
        if policy == Policy::Discard {
            next.manual.clear();
        }
        let Some(old) = self
            .engine
            .scene
            .botany
            .iter()
            .find(|r| r.prefix == next.prefix)
        else {
            if next.parts().iter().any(|id| {
                self.engine.scene.meshes.assets.iter().any(|a| &a.id == id)
                    || self
                        .engine
                        .scene
                        .meshes
                        .instances
                        .iter()
                        .any(|i| &i.id == id)
            }) {
                return Err(invalid(
                    "Prefix already owns meshes; choose a new botanical prefix",
                ));
            }
            return Ok(next);
        };
        if policy == Policy::Discard {
            return Ok(next);
        }
        let baseline = bake::compile(old)?;
        let mut recovered = old.manual.clone();
        for part in 0..2 {
            let id = &old.parts()[part];
            let current = self.engine.scene.meshes.asset(id)?;
            let reference = &baseline.document.assets[part];
            if current.rig != reference.rig
                || current.clips != reference.clips
                || current.material != reference.material
                || current.shading != reference.shading
                || current.geometry.triangles != reference.geometry.triangles
                || current.geometry.positions.len() != reference.geometry.positions.len()
                || current.geometry.material_coordinates != reference.geometry.material_coordinates
            {
                return Err(invalid(
                    "Manual topology/material/rig edit cannot be transported; use explicit discard or undo",
                ));
            }
            let instance = self
                .engine
                .scene
                .meshes
                .instances
                .iter()
                .find(|i| &i.id == id)
                .ok_or_else(|| invalid("Botanical instance missing"))?;
            if instance != &baseline.document.instances[part] {
                return Err(invalid(
                    "Manual instance transform/color cannot be transported; undo or explicitly discard",
                ));
            }
            if baseline.vertices[part].is_empty()
                && current.geometry.positions != reference.geometry.positions
            {
                return Err(invalid(
                    "Empty botanical placeholder has manual geometry; undo or explicitly discard",
                ));
            }
            for ((point, reference), binding) in current
                .geometry
                .positions
                .iter()
                .zip(&reference.geometry.positions)
                .zip(&baseline.vertices[part])
            {
                let delta = DVec3::from_array(point.map(f64::from))
                    - DVec3::from_array(reference.map(f64::from));
                if delta.length_squared() > 1e-18 {
                    if policy == Policy::Reject {
                        return Err(invalid("Manual botanical sculpt present"));
                    }
                    let d = baseline.poses[binding.organ].at(binding.u).local(delta);
                    let prior = recovered.entry(binding.key.clone()).or_insert([0.; 3]);
                    *prior = (DVec3::from_array(*prior) + d).to_array();
                }
            }
        }
        if policy == Policy::Reject && !recovered.is_empty() {
            return Err(invalid("Manual botanical offsets present"));
        }
        for (key, offset) in recovered {
            next.manual.insert(key, offset);
        }
        Ok(next)
    }
    fn botanical_merge(&self, document: meshes::Document) -> Result<meshes::Document> {
        let mut merged = self.engine.scene.meshes.clone();
        for asset in document.assets {
            merged.apply(meshes::Edit::PutAsset { asset })?;
        }
        for instance in document.instances {
            merged.apply(meshes::Edit::PutInstance { instance })?;
        }
        merged.validate()?;
        Ok(merged)
    }
    fn botanical_publish(&mut self, mut recipe: Recipe, policy: Policy) -> Result<Value> {
        if self.engine.playing || self.remote_client() {
            return Err(invalid(
                "Pause the local editor before publishing vegetation",
            ));
        }
        recipe.validate()?;
        recipe.version = b::VERSION;
        let recipe = self.botanical_preserve(recipe, policy)?;
        let compiled = bake::compile(&recipe)?;
        let (minimum, maximum) = compiled
            .document
            .assets
            .iter()
            .flat_map(|a| a.geometry.positions.iter())
            .map(|p| {
                DVec3::from_array(p.map(f64::from))
                    + DVec3::from_array(recipe.settings.position.map(f64::from))
            })
            .fold(
                (DVec3::splat(f64::INFINITY), DVec3::splat(f64::NEG_INFINITY)),
                |(lo, hi), p| (lo.min(p), hi.max(p)),
            );
        let report = json!({"development":b::development::state(&recipe)?,"bounds_min":minimum.to_array(),"bounds_max":maximum.to_array(),"organs":compiled.poses.len(),"vertices":compiled.document.assets.iter().map(|a|a.geometry.positions.len()).sum::<usize>(),"triangles":compiled.document.assets.iter().map(|a|a.geometry.triangles.len()).sum::<usize>(),"junctions":compiled.joints});
        let mut recipes = self.engine.scene.botany.clone();
        recipes.retain(|r| r.prefix != recipe.prefix);
        recipes.push(recipe.clone());
        let mut edits: Vec<_> = compiled
            .document
            .assets
            .into_iter()
            .map(|asset| SceneEdit::Mesh {
                edit: meshes::Edit::PutAsset { asset },
            })
            .collect();
        edits.extend(
            compiled
                .document
                .instances
                .into_iter()
                .map(|instance| SceneEdit::Mesh {
                    edit: meshes::Edit::PutInstance { instance },
                }),
        );
        edits.push(SceneEdit::SetBotany { recipes });
        let result = self.engine.execute(Command::EditScene { edits }.into())?;
        if self.botany.selected.as_ref() != Some(&recipe.prefix) {
            self.botany.organ = None;
        }
        self.botany.selected = Some(recipe.prefix);
        self.botany.revision = None;
        self.botany.preview = None;
        self.botany.overlay = None;
        self.botany.physics = None;
        Ok(
            json!({"revision":result["revision"],"compiled":report,"source_embedded":true,"manual_points":recipe.manual.len()}),
        )
    }
    fn botanical_basis(&self, prefix: &str) -> (Option<Recipe>, meshes::Document) {
        let recipe = self
            .engine
            .scene
            .botany
            .iter()
            .find(|r| r.prefix == prefix)
            .cloned();
        let mut document = meshes::Document::default();
        for id in [format!("{prefix}_wood"), format!("{prefix}_leaves")] {
            document.assets.extend(
                self.engine
                    .scene
                    .meshes
                    .assets
                    .iter()
                    .filter(|a| a.id == id)
                    .cloned(),
            );
            document.instances.extend(
                self.engine
                    .scene
                    .meshes
                    .instances
                    .iter()
                    .filter(|i| i.id == id)
                    .cloned(),
            );
        }
        (recipe, document)
    }
    pub(super) fn refresh_botany(&mut self) -> Result<()> {
        if self.botany.revision == Some(self.engine.revision) || self.botany.revision.is_none() {
            return Ok(());
        }
        let prefix = self
            .botany
            .preview
            .as_ref()
            .map(|r| r.prefix.as_str())
            .or_else(|| {
                self.botany
                    .physics
                    .as_ref()
                    .map(|p| p.recipe.prefix.as_str())
            });
        if let Some(prefix) = prefix
            && self.botany.basis.as_ref() == Some(&self.botanical_basis(prefix))
        {
            let parts = [format!("{prefix}_wood"), format!("{prefix}_leaves")];
            if let Some(old) = &self.botany.overlay {
                let mut document = meshes::Document::default();
                document
                    .assets
                    .extend(old.assets.iter().filter(|a| parts.contains(&a.id)).cloned());
                document.instances.extend(
                    old.instances
                        .iter()
                        .filter(|i| parts.contains(&i.id))
                        .cloned(),
                );
                self.botany.overlay = Some(self.botanical_merge(document)?);
                self.botany.revision = Some(self.engine.revision);
                return Ok(());
            }
        }
        self.botany.preview = None;
        self.botany.physics = None;
        self.botany.overlay = None;
        self.botany.revision = None;
        self.botany.basis = None;
        Ok(())
    }
    pub(super) fn advance_botany(&mut self, ticks: u32) -> Result<()> {
        self.refresh_botany()?;
        let prepared = if let Some(session) = &self.botany.physics
            && session.playing
            && self.botany.revision == Some(self.engine.revision)
        {
            let mut candidate = session.clone();
            candidate.step(
                ticks
                    .checked_mul(2)
                    .filter(|v| *v <= 3600)
                    .ok_or_else(|| invalid("Botanical playing step exceeds1800 engine ticks"))?,
            )?;
            let overlay = self.botanical_merge(candidate.document())?;
            Some((candidate, overlay))
        } else {
            None
        };
        self.engine.advance(ticks)?;
        if let Some((candidate, overlay)) = prepared {
            self.botany.physics = Some(candidate);
            self.botany.overlay = Some(overlay);
        }
        Ok(())
    }
    pub(super) fn botany_command(&mut self, operation: &Op) -> Result<Value> {
        match operation {
            Op::Inspect => Ok(
                json!({"open":self.botany.open,"selected":self.botany.selected,"organ":self.botany.organ,"preview":self.botany.preview.is_some()&&self.botany.revision==Some(self.engine.revision),"recipes":self.engine.scene.botany,"physics":self.botany.physics.as_ref().filter(|_|self.botany.revision==Some(self.engine.revision)).map(|p|p.inspect())}),
            ),
            Op::Open { open, prefix } => {
                if let Some(p) = prefix {
                    self.botanical_recipe(p)?;
                    if self.botany.selected.as_ref() != Some(p) {
                        self.botany.organ = None;
                    }
                    self.botany.selected = Some(p.clone());
                }
                self.botany.open = *open;
                if *open {
                    self.character_editor.open = false;
                }
                Ok(json!({"open":open}))
            }
            Op::Preview { recipe, policy } => {
                if self.engine.playing || self.remote_client() {
                    return Err(invalid("Pause the local editor before botanical preview"));
                }
                let recipe = self.botanical_preserve((**recipe).clone(), *policy)?;
                let compiled = bake::compile(&recipe)?;
                let overlay = self.botanical_merge(compiled.document)?;
                self.botany.basis = Some(self.botanical_basis(&recipe.prefix));
                self.botany.overlay = Some(overlay);
                self.botany.preview = Some(recipe);
                self.botany.physics = None;
                self.botany.revision = Some(self.engine.revision);
                Ok(json!({"preview":true,"revision":self.engine.revision,"world_changed":false}))
            }
            Op::CancelPreview => {
                self.botany.preview = None;
                self.botany.overlay = None;
                self.botany.physics = None;
                self.botany.revision = None;
                Ok(json!({"world_changed":false}))
            }
            Op::Publish { recipe, policy } => self.botanical_publish((**recipe).clone(), *policy),
            Op::SelectOrgan { prefix, organ } => {
                let recipe = self.botanical_recipe(prefix)?;
                if organ
                    .as_ref()
                    .is_some_and(|id| !recipe.source.organs.iter().any(|o| &o.id == id))
                {
                    return Err(invalid("Unknown botanical organ"));
                }
                self.botany.selected = Some(prefix.clone());
                self.botany.organ = organ.clone();
                Ok(json!({"organ":organ,"world_changed":false}))
            }
            Op::SetRoots {
                prefix,
                stem,
                settings,
                preview,
                policy,
            } => {
                let recipe = b::roots::configure_with_policy(
                    &self.botanical_recipe(prefix)?,
                    stem,
                    settings.clone(),
                    *policy,
                )?;
                if *preview {
                    self.botany_command(&Op::Preview {
                        recipe: Box::new(recipe),
                        policy: *policy,
                    })
                } else {
                    self.botanical_publish(recipe, *policy)
                }
            }
            Op::SetAge {
                prefix,
                age_years,
                vigor,
                preview,
                policy,
            } => {
                let mut recipe = self.botanical_recipe(prefix)?;
                recipe.settings.age_years = Some(*age_years);
                if let Some(vigor) = vigor {
                    recipe.settings.vigor = *vigor;
                }
                if *preview {
                    self.botany_command(&Op::Preview {
                        recipe: Box::new(recipe),
                        policy: *policy,
                    })
                } else {
                    self.botanical_publish(recipe, *policy)
                }
            }
            Op::EditOrgan {
                prefix,
                organ,
                edit,
            } => {
                let mut recipe = self.botanical_recipe(prefix)?;
                recipe.edits.insert(organ.clone(), edit.clone());
                self.botanical_publish(recipe, Policy::Preserve)
            }
            Op::Save { prefix, path } => {
                if !path.ends_with(".ignis-botany") {
                    return Err(invalid("Botanical source requires .ignis-botany suffix"));
                }
                let recipe =
                    self.botanical_preserve(self.botanical_recipe(prefix)?, Policy::Preserve)?;
                bake::compile(&recipe)?;
                let bytes = serde_json::to_vec(&recipe).map_err(io_error)?;
                if bytes.len() > b::MAX_SOURCE_BYTES {
                    return Err(invalid("Botanical source exceeds32MiB"));
                }
                let output = self.write(path, &bytes)?;
                Ok(json!({"path":output,"bytes":bytes.len(),"world_changed":false}))
            }
            Op::Load { path, publish } => {
                if !path.ends_with(".ignis-botany") {
                    return Err(invalid("Botanical source requires .ignis-botany suffix"));
                }
                let mut bytes = vec![];
                std::fs::File::open(self.resolve(path)?)
                    .map_err(io_error)?
                    .take((b::MAX_SOURCE_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(io_error)?;
                if bytes.len() > b::MAX_SOURCE_BYTES {
                    return Err(invalid("Botanical source exceeds32MiB"));
                }
                let recipe: Recipe = serde_json::from_slice(&bytes).map_err(io_error)?;
                recipe.validate()?;
                if *publish {
                    self.botanical_publish(recipe, Policy::Preserve)
                } else {
                    bake::compile(&recipe)?;
                    Ok(json!({"recipe":recipe,"world_changed":false}))
                }
            }
            Op::Physics { prefix, action } => {
                use b::dynamics::Action as A;
                if self.remote_client() || self.network_host.is_some() {
                    return Err(invalid("Botanical simulation is a local authoring session"));
                }
                let recipe = self.botanical_recipe(prefix)?;
                if recipe
                    .parts()
                    .iter()
                    .any(|id| self.engine.scene.collision_meshes.contains(id))
                {
                    return Err(invalid(
                        "Disable static botanical mesh collision before previewing deformation",
                    ));
                }
                if matches!(action, A::Reset) {
                    self.botany.physics = None;
                    self.botany.overlay = None;
                    self.botany.revision = None;
                    return Ok(json!({"reset":true}));
                }
                if matches!(action, A::Inspect) {
                    return self
                        .botany
                        .physics
                        .as_ref()
                        .filter(|s| {
                            s.recipe.prefix == *prefix
                                && self.botany.revision == Some(self.engine.revision)
                        })
                        .map(|s| s.inspect())
                        .ok_or_else(|| invalid("No current botanical physical session"));
                }
                let mut session = if let A::Start { settings } = action {
                    let recipe = self.botanical_preserve(recipe, Policy::Preserve)?;
                    let compiled = bake::compile(&recipe)?;
                    b::dynamics::Session::new(recipe, compiled, settings.clone())?
                } else {
                    self.botany
                        .physics
                        .as_ref()
                        .filter(|s| {
                            s.recipe.prefix == *prefix
                                && self.botany.revision == Some(self.engine.revision)
                        })
                        .cloned()
                        .ok_or_else(|| {
                            invalid(
                                "Start a botanical physical session at the current scene revision",
                            )
                        })?
                };
                match action {
                    A::Step { ticks } => session.step(*ticks)?,
                    A::Grow { age } => session.grow(*age)?,
                    A::Prune { organ } => session.prune(organ)?,
                    A::Play { playing } => session.playing = *playing,
                    _ => {}
                }
                let result = session.inspect();
                let overlay = self.botanical_merge(session.document())?;
                self.botany.basis = Some(self.botanical_basis(prefix));
                self.botany.physics = Some(session);
                self.botany.overlay = Some(overlay);
                self.botany.preview = None;
                self.botany.revision = Some(self.engine.revision);
                Ok(result)
            }
        }
    }
}
