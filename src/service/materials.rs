//! File/bake work runs in command execution, never in editor drawing or rendering.
use super::*;
use crate::materials::{
    library::{self, Execution, Operation as Op},
    recipe::Procedural,
};
use std::sync::Arc;
impl Service {
    pub(super) fn material_command(&mut self, op: &Op) -> Result<Value> {
        let apply = |service: &mut Self,
                     asset: &str,
                     material: crate::materials::Material|
         -> Result<Value> {
            if service.engine.playing || service.remote_client() {
                return Err(invalid("Pause the local editor before applying a material"));
            }
            let result = service.engine.execute(
                Command::EditMesh {
                    edit: crate::meshes::Edit::SetMaterial {
                        asset: asset.to_owned(),
                        material,
                    },
                }
                .into(),
            )?;
            let mut report = library::inspect(&service.engine.scene.meshes.asset(asset)?.material);
            report["edit"] = result;
            report["world_changed"] = json!(true);
            Ok(report)
        };
        match op {
            Op::Rock {
                asset,
                family,
                seed,
                unique_instances,
                execution,
            } => {
                self.engine.scene.meshes.asset(asset)?;
                let mut r = family.recipe();
                if let Some(seed) = seed {
                    r.seed = *seed;
                }
                r.unique_instances = *unique_instances;
                apply(self, asset, library::material(r, *execution)?)
            }
            Op::Marble {
                asset,
                family,
                seed,
                unique_instances,
                execution,
            } => {
                self.engine.scene.meshes.asset(asset)?;
                let mut r = family.recipe();
                if let Some(seed) = seed {
                    r.seed = *seed;
                }
                r.unique_instances = *unique_instances;
                apply(self, asset, library::material(r, *execution)?)
            }
            Op::BakeDisplacement {
                instance,
                id,
                reference,
            } => {
                if self.engine.playing || self.remote_client() || self.engine.planet_active() {
                    return Err(invalid(
                        "Pause the local editor without planet runtime before baking geometry",
                    ));
                }
                if *reference != self.engine.mesh_reference() {
                    return Err(invalid(
                        "Mesh source changed; inspect before baking displacement",
                    ));
                }
                if self.engine.scene.meshes.assets.iter().any(|a| a.id == *id) {
                    return Err(invalid("Displacement output requires a new asset ID"));
                }
                let mut target = self
                    .engine
                    .scene
                    .meshes
                    .instances
                    .iter()
                    .find(|i| i.id == *instance)
                    .cloned()
                    .ok_or_else(|| invalid("Mesh instance not found"))?;
                let source = self.engine.scene.meshes.asset(&target.asset)?;
                let asset = library::displace(source, id, instance, target.scale)?;
                target.asset = id.clone();
                self.engine.execute(
                    Command::EditScene {
                        edits: vec![
                            crate::engine::SceneEdit::Mesh {
                                edit: crate::meshes::Edit::PutAsset { asset },
                            },
                            crate::engine::SceneEdit::Mesh {
                                edit: crate::meshes::Edit::PutInstance { instance: target },
                            },
                        ],
                    }
                    .into(),
                )
            }

            Op::Inspect { asset } => Ok(library::inspect(
                &self.engine.scene.meshes.asset(asset)?.material,
            )),
            Op::Generate {
                asset,
                recipe,
                execution,
            } => {
                self.engine.scene.meshes.asset(asset)?;
                apply(
                    self,
                    asset,
                    library::material((**recipe).clone(), *execution)?,
                )
            }
            Op::Preset {
                asset,
                preset,
                seed,
                unique_instances,
                execution,
            } => {
                self.engine.scene.meshes.asset(asset)?;
                let mut r = preset.recipe();
                if let Some(seed) = seed {
                    r.seed = *seed;
                }
                r.unique_instances = *unique_instances;
                apply(self, asset, library::material(r, *execution)?)
            }
            Op::SetExecution { asset, execution } => {
                let mut m = self.engine.scene.meshes.asset(asset)?.material.clone();
                let p = m
                    .procedural
                    .as_ref()
                    .ok_or_else(|| invalid("Material has no procedural recipe"))?;
                m.procedural = Some(Arc::new(match execution {
                    Execution::Runtime => Procedural::runtime(p.recipe.clone()),
                    Execution::Fixed { resolution } => p.bake(*resolution)?,
                }));
                apply(self, asset, m)
            }
            Op::Save { asset, path } => {
                if !path.ends_with(".ignis-material") {
                    return Err(invalid("Material path requires .ignis-material suffix"));
                }
                let m = &self.engine.scene.meshes.asset(asset)?.material;
                let bytes = library::encode(m)?;
                let mut report = library::inspect(m);
                report["path"] = json!(self.write(path, &bytes)?);
                report["file_bytes"] = json!(bytes.len());
                Ok(report)
            }
            Op::Load { path, asset } => {
                if !path.ends_with(".ignis-material") {
                    return Err(invalid("Material path requires .ignis-material suffix"));
                }
                let file = std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
                let mut bytes = Vec::new();
                file.take((crate::materials::recipe::MAX_FILE_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(io_error)?;
                let m = library::decode(&bytes)?;
                if let Some(asset) = asset {
                    apply(self, asset, m)
                } else {
                    Ok(library::inspect(&m))
                }
            }
        }
    }
}
