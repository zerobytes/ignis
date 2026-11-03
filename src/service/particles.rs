use super::*;
use crate::particles::{Operation as Op, Source};
impl Service {
    pub(super) fn particles_command(&mut self, op: &Op) -> Result<Value> {
        if self.remote_client() {
            return Err(invalid(
                "Particle authoring/preview requires the local world; remote effects follow the replica clock",
            ));
        }
        match op {
            Op::Open { open } => {
                self.particle_panel_open = *open;
                if *open {
                    self.botany.open = false;
                    self.character_editor.open = false;
                }
                Ok(json!({"open":open}))
            }
            Op::SaveSource { id, path } => {
                if !path.ends_with(".ignis-vfx") {
                    return Err(invalid("Particle source requires .ignis-vfx suffix"));
                }
                let source = self
                    .engine
                    .scene
                    .particles
                    .sources
                    .iter()
                    .find(|s| s.id == *id)
                    .ok_or_else(|| invalid("Particle source not found"))?;
                source.validate()?;
                let bytes = document_bytes(source)?;
                Ok(json!({"path":self.write(path,&bytes)?,"bytes":bytes.len()}))
            }
            Op::LoadSource { path } => {
                if !path.ends_with(".ignis-vfx") {
                    return Err(invalid("Particle source requires .ignis-vfx suffix"));
                }
                let source: Source = self.read(path)?;
                source.validate()?;
                self.engine.particles_command(&Op::PutSource {
                    source: std::sync::Arc::new(source),
                })
            }
            _ => self.engine.particles_command(op),
        }
    }
}
