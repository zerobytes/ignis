//! Bounded candidate sampling. Physical admission remains the spawn transaction's job.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
pub(crate) fn draw(rng: &mut u32) -> u32 {
    *rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
    *rng
}
impl Region {
    pub fn validate(&self, version: u32) -> Result<()> {
        if version < 64
            || (0..3).any(|axis| {
                !self.min[axis].is_finite()
                    || !self.max[axis].is_finite()
                    || self.min[axis].abs() > 10000.
                    || self.max[axis].abs() > 10000.
                    || self.min[axis] > self.max[axis]
            })
            || self.min == self.max
        {
            return Err(invalid(
                "Spawn region requires scene64, finite ordered bounds within +/-10000 and at least one nonzero extent",
            ));
        }
        Ok(())
    }
    pub fn sample(&self, rng: &mut u32) -> [f32; 3] {
        std::array::from_fn(|axis| {
            let fraction = (draw(rng) >> 8) as f32 * (1. / 16777216.);
            self.min[axis] + (self.max[axis] - self.min[axis]) * fraction
        })
    }
}
