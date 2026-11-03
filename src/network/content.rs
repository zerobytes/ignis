//! Explicit public visual package. Never serialize the administrative Scene here.
use crate::{
    engine::Engine,
    meshes::Document,
    model::{Camera, Light, Result, invalid},
};
use serde::{Deserialize, Serialize};
pub const VERSION: u16 = 14;
pub const MAX_BYTES: usize = 8 * 1024 * 1024;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Content {
    pub version: u16,
    pub world: u64,
    pub camera: Camera,
    pub light: Light,
    pub meshes: Document,
    pub particles: std::sync::Arc<crate::particles::Document>,
    pub distant_meshes: std::sync::Arc<Vec<crate::distant::Pack>>,
    pub consumed: Vec<String>,
}
impl Content {
    pub fn capture(engine: &Engine, world: u64) -> Self {
        Self {
            version: VERSION,
            world,
            camera: engine.scene.camera.clone(),
            light: engine.scene.light.clone(),
            meshes: engine.scene.meshes.clone(),
            particles: engine.scene.particles.clone(),
            distant_meshes: engine.scene.distant_meshes.clone(),
            consumed: engine
                .scene
                .plants
                .connections
                .iter()
                .map(|c| c.child_instance.clone())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
        }
    }
    pub fn validate(&self, world: u64) -> Result<()> {
        if self.version != VERSION || self.world != world || world == 0 {
            return Err(invalid("Visual content version/world mismatch"));
        }
        self.camera.validate()?;
        self.light.validate()?;
        self.meshes.validate()?;
        self.particles.validate()?;
        crate::distant::validate(&self.distant_meshes, &self.meshes)?;
        let mut ids = std::collections::BTreeSet::new();
        for id in &self.consumed {
            if !ids.insert(id) || !self.meshes.instances.iter().any(|i| i.id == *id) {
                return Err(invalid("Invalid visual mesh mask"));
            }
        }
        Ok(())
    }
    /// Backend-only serialization: never call from the world's mutation section.
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate(self.world)?;
        let bytes = serde_json::to_vec(self).map_err(|e| invalid(e.to_string()))?;
        if bytes.len() > MAX_BYTES {
            return Err(invalid("Visual content exceeds8MiB"));
        }
        Ok(bytes)
    }
    pub fn decode(world: u64, bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(invalid("Visual content exceeds8MiB"));
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
        value.validate(world)?;
        Ok(value)
    }
}
