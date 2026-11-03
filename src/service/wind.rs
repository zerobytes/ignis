use super::*;
use crate::wind::Operation as Op;
impl Service {
    pub(super) fn wind_command(&mut self, operation: &Op) -> Result<Value> {
        if self.remote_client() || self.network_host.is_some() {
            return Err(invalid(
                "Wind authoring and inspection require the offline local world",
            ));
        }
        match operation {
            Op::Open { open } => {
                self.wind_panel_open = *open;
                if *open {
                    self.botany.open = false;
                    self.character_editor.open = false;
                    self.particle_panel_open = false;
                }
                Ok(json!({"open":open}))
            }
            Op::Save { path } => {
                if !path.ends_with(".ignis-wind") {
                    return Err(invalid("Wind source requires .ignis-wind suffix"));
                }
                let bytes = document_bytes(&self.engine.scene.wind)?;
                Ok(json!({"path":self.write(path,&bytes)?,"bytes":bytes.len()}))
            }
            Op::Load { path } => {
                if !path.ends_with(".ignis-wind") {
                    return Err(invalid("Wind source requires .ignis-wind suffix"));
                }
                let document = self.read::<crate::wind::Document>(path)?;
                self.engine.wind_command(&Op::Replace {
                    document: std::sync::Arc::new(document),
                })
            }
            _ => self.engine.wind_command(operation),
        }
    }
}
