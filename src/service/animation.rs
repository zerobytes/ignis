use super::*;
use crate::animation::{Document, MAX_BYTES, Operation};
impl Service {
    pub(super) fn animation_command(
        &mut self,
        op: &Operation,
        revision: Option<u64>,
    ) -> Result<Value> {
        self.animation_library.guard(revision)?;
        match op {
            Operation::Inspect => {
                return Ok(
                    json!({"animation_revision":self.animation_library.revision(),"document":self.animation_library.document().ok()}),
                );
            }
            Operation::Replace { document } => self.animation_library.replace(*document.clone())?,
            Operation::PutClip { clip } => {
                let mut d = self.animation_library.document()?.clone();
                if let Some(c) = d.clips.iter_mut().find(|c| c.id == clip.id) {
                    *c = clip.clone();
                } else {
                    d.clips.push(clip.clone());
                }
                self.animation_library.replace(d)?;
            }
            Operation::SetMotion { motion } => {
                let mut d = self.animation_library.document()?.clone();
                if let Some(m) = d.motions.iter_mut().find(|m| m.name == motion.name) {
                    *m = motion.clone();
                } else {
                    d.motions.push(motion.clone());
                }
                self.animation_library.replace(d)?;
            }
            Operation::ImportAsset { asset, motions } => {
                let a = self.engine.scene.meshes.asset(asset)?;
                let rig = a
                    .rig
                    .as_ref()
                    .ok_or_else(|| invalid("Motion import requires rig"))?;
                let floor_y = a
                    .geometry
                    .positions
                    .iter()
                    .map(|p| p[1])
                    .fold(f32::INFINITY, f32::min);
                self.animation_library.replace(Document {
                    version: 1,
                    floor_y,
                    bones: rig.bones.clone(),
                    clips: a.clips.clone(),
                    motions: motions.clone(),
                })?;
            }
            Operation::ExportClips { asset } => {
                let target = self.engine.scene.meshes.asset(asset)?;
                let rig = target
                    .rig
                    .as_ref()
                    .ok_or_else(|| invalid("Motion export requires rig"))?;
                let d = self.animation_library.document()?;
                let names = d
                    .motions
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect::<Vec<_>>();
                let clips = d.clips_for(rig, &names)?;
                let mut doc = self.engine.scene.meshes.clone();
                for clip in clips {
                    doc.apply(crate::meshes::Edit::PutClip {
                        asset: asset.clone(),
                        clip,
                    })?;
                }
                // Publish all native clip edits in one ordinary scene transaction.
                let edits = doc
                    .asset(asset)?
                    .clips
                    .iter()
                    .map(|clip| crate::engine::SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutClip {
                            asset: asset.clone(),
                            clip: clip.clone(),
                        },
                    })
                    .collect::<Vec<_>>();
                self.engine.apply_edits(edits.into_iter())?;
            }
            Operation::Sample {
                motion,
                phase,
                targets,
            } => {
                return self
                    .animation_library
                    .document()?
                    .sample(motion, *phase, targets);
            }
            Operation::CalibrateStride { motion } => {
                let mut d = self.animation_library.document()?.clone();
                let stride = d.calibrate(motion)?;
                d.motions
                    .iter_mut()
                    .find(|m| m.name == *motion)
                    .unwrap()
                    .stride = stride;
                self.animation_library.replace(d)?;
                return Ok(
                    json!({"animation_revision":self.animation_library.revision(),"stride":stride,"units":"reference metres per cycle"}),
                );
            }
            Operation::Undo => self.animation_library.history(false)?,
            Operation::BakeSupports { motion, samples } => {
                let candidate = self
                    .animation_library
                    .document()?
                    .bake_supports(motion, *samples)?;
                self.animation_library.replace(candidate)?;
            }
            Operation::Redo => self.animation_library.history(true)?,
            Operation::Save { path } => {
                let bytes = serde_json::to_vec_pretty(self.animation_library.document()?)
                    .map_err(io_error)?;
                if bytes.len() > MAX_BYTES {
                    return Err(invalid("Motion library exceeds2MiB"));
                }
                self.write(path, &bytes)?;
            }
            Operation::Load { path } => {
                let file = std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
                if !file.metadata().map_err(io_error)?.is_file() {
                    return Err(invalid("Motion library must be a regular file"));
                }
                let mut bytes = Vec::new();
                file.take((MAX_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(io_error)?;
                if bytes.len() > MAX_BYTES {
                    return Err(invalid("Motion library exceeds2MiB"));
                }
                let d = serde_json::from_slice(&bytes).map_err(io_error)?;
                self.animation_library.replace(d)?;
            }
        }
        Ok(
            json!({"animation_revision":self.animation_library.revision(),"scene_revision":self.engine.revision}),
        )
    }
}
