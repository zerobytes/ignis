//! Bounded point stimuli. No device audio or association with a hidden target.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_NOISES: usize = 4096;
pub const MAX_SERIAL: u64 = (1u64 << 62) - 1;
#[derive(Clone, Copy)]
pub enum Domain {
    Manual = 0,
    Ability = 1,
    Footstep = 2,
}
/// Interleave namespaces without reusing IDs when a counter passes its old30bit lane.
/// Initial IDs remain compatible with the previous manual/ability/footstep ranges.
pub fn event_id(domain: Domain, serial: u64) -> u64 {
    assert!(serial <= MAX_SERIAL, "validated noise identity budget");
    ((serial >> 30) << 32) | ((domain as u64) << 30) | (serial & ((1 << 30) - 1))
}
pub const MAX_NOISES_PER_TICK: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NoiseOrigin {
    Position { position: [f32; 3] },
    Entity { id: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Noise {
    pub emitter_instance: Option<crate::identity::Reference>,
    pub id: u64,
    /// Pre-physics tick at which this impulse is consumed.
    pub tick: u64,
    pub position: [f32; 3],
    pub radius: f32,
    /// Automatic emitter slot in the normalized scene; only used to ignore self-noise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emitter_index: Option<u16>,
}
impl Noise {
    pub fn validate(&self) -> Result<()> {
        if self
            .position
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 10000.)
            || !self.radius.is_finite()
            || !(0.1..=100.).contains(&self.radius)
        {
            return Err(invalid(
                "Noise requires finite position within ±10000 and radius [0.1,100]m",
            ));
        }
        Ok(())
    }
}
