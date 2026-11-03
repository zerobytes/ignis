//! Spatial permission for population production; never an NPC perception source.
use crate::model::{Result, Scene, invalid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Once,
    WhileOccupied,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub volume: String,
    #[serde(default)]
    pub mode: Mode,
}
impl Config {
    pub fn validate(&self, scene: &Scene) -> Result<()> {
        if scene.version < 63
            || !scene
                .entities
                .iter()
                .any(|e| e.id == self.volume && e.trigger.is_some())
        {
            return Err(invalid(
                "Generator activation requires scene63 and an existing trigger volume",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub volume: u8,
    pub mode: Mode,
    pub occupied: bool,
    pub open: bool,
    pub sampled_tick: Option<u64>,
    pub last_change_tick: Option<u64>,
    pub changes: u32,
}
impl State {
    pub fn new(config: &Config, scene: &Scene) -> Self {
        // Same lexicographic order as trigger::Runtime; resolve once without a temporary ID list.
        let volume = scene
            .entities
            .iter()
            .filter(|e| e.trigger.is_some() && e.id < config.volume)
            .count() as u8;
        Self {
            volume,
            mode: config.mode,
            occupied: false,
            open: false,
            sampled_tick: None,
            last_change_tick: None,
            changes: 0,
        }
    }
    pub fn sample(&mut self, occupied: bool, tick: u64) {
        self.occupied = occupied;
        self.sampled_tick = Some(tick);
        let open = occupied || (self.mode == Mode::Once && self.open);
        if open != self.open {
            self.open = open;
            self.last_change_tick = Some(tick);
            self.changes += 1;
        }
    }
}
