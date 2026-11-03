//! Sustained ability timing; effects and cancellation stay in the shared executor.
use crate::model::{Result, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CostMode {
    OnStart,
    PerPulse,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    pub interval_ticks: u32,
    pub pulses: u32,
    pub cost_mode: CostMode,
}
impl Channel {
    pub fn validate(self, version: u32) -> Result<()> {
        if version < 36
            || !(1..=3600).contains(&self.interval_ticks)
            || !(1..=3600).contains(&self.pulses)
            || u64::from(self.interval_ticks) * u64::from(self.pulses) > 216_000
        {
            return Err(invalid(
                "Channel requires version36, interval/pulses1..3600 and duration<=216000 ticks",
            ));
        }
        Ok(())
    }
    pub fn start(self, tick: u64) -> Progress {
        Progress {
            started: tick,
            finishes_at: tick + u64::from(self.interval_ticks) * u64::from(self.pulses),
            next_tick: tick + u64::from(self.interval_ticks),
            remaining: self.pulses,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progress {
    pub started: u64,
    pub finishes_at: u64,
    pub next_tick: u64,
    pub remaining: u32,
}
