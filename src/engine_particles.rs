use super::*;
use crate::particles::{Frame, Operation as Op, Placement, Source, origin};
use std::sync::Arc;
impl Engine {
    pub(crate) fn particles_command(&mut self, operation: &Op) -> Result<Value> {
        match operation {
            Op::Inspect => {
                let mut frame = Frame::default();
                if !self.planet_active() {
                    frame.update(
                        &self.scene.particles,
                        self.particle_preview.unwrap_or(self.simulation.tick),
                    );
                }
                Ok(
                    json!({"document":self.scene.particles,"preview_tick":self.particle_preview,"planet_supported":false,"presentation":frame.inspect()}),
                )
            }
            Op::Preview { tick } => {
                if self.planet_active() {
                    return Err(invalid("Particle preview is unavailable in planetary mode"));
                }
                self.particle_preview = *tick;
                self.view_revision += 1;
                Ok(json!({"preview_tick":tick,"world_tick":self.simulation.tick}))
            }
            Op::Open { .. } | Op::SaveSource { .. } | Op::LoadSource { .. } => Err(invalid(
                "Particle panel/file operations require the editor service",
            )),
            _ => {
                if self.playing || self.planet_active() {
                    return Err(invalid(
                        "Pause the ordinary scene before authoring particles",
                    ));
                }
                let mut document = (*self.scene.particles).clone();
                match operation {
                    Op::Replace { document: d } => document = (**d).clone(),
                    Op::Preset { preset, id } => {
                        let mut source = Source::preset(*preset);
                        source.id = id.clone();
                        source.validate()?;
                        if document.sources.iter().any(|s| s.id == *id)
                            || document.instances.iter().any(|s| s.id == *id)
                        {
                            return Err(invalid("Preset requires a new source and instance ID"));
                        }
                        document.sources.push(Arc::new(source));
                        document.instances.push(Placement {
                            id: id.clone(),
                            source: id.clone(),
                            seed: 0,
                            position: origin(),
                            rotation: [0., 0., 0., 1.],
                            scale: 1.,
                            tint: [1.; 4],
                            start_tick: 0,
                            enabled: true,
                        });
                    }
                    Op::PutSource { source } => {
                        source.validate()?;
                        if let Some(s) = document.sources.iter_mut().find(|s| s.id == source.id) {
                            *s = source.clone();
                        } else {
                            document.sources.push(source.clone());
                        }
                    }
                    Op::RemoveSource { id } => {
                        if document.instances.iter().any(|p| p.source == *id) {
                            return Err(invalid("Remove particle placements before their source"));
                        }
                        let n = document.sources.len();
                        document.sources.retain(|s| s.id != *id);
                        if n == document.sources.len() {
                            return Err(invalid("Particle source not found"));
                        }
                    }
                    Op::PutInstance { instance } => {
                        if let Some(p) = document.instances.iter_mut().find(|p| p.id == instance.id)
                        {
                            *p = instance.clone();
                        } else {
                            document.instances.push(instance.clone());
                        }
                    }
                    Op::RemoveInstance { id } => {
                        let n = document.instances.len();
                        document.instances.retain(|p| p.id != *id);
                        if n == document.instances.len() {
                            return Err(invalid("Particle placement not found"));
                        }
                    }
                    _ => unreachable!(),
                }
                document.validate()?;
                self.apply_edits(std::iter::once(SceneEdit::SetParticles {
                    document: Arc::new(document),
                }))?;
                Ok(
                    json!({"sources":self.scene.particles.sources.len(),"instances":self.scene.particles.instances.len(),"revision":self.revision}),
                )
            }
        }
    }
}
