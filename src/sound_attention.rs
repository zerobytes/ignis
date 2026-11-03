//! Stateless candidate filters plus one bounded active acoustic receipt per listener.
use crate::{
    model::{Result, invalid},
    perception::{Observation, Source},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub commitment_ticks: u32,
    pub switch_margin: f32,
    pub repeat_radius: f32,
    pub visual_priority_ticks: u32,
}
impl Config {
    pub fn validate(self) -> Result<()> {
        if self.commitment_ticks > 3600
            || self.visual_priority_ticks > 3600
            || !self.switch_margin.is_finite()
            || !(0. ..=1.).contains(&self.switch_margin)
            || !self.repeat_radius.is_finite()
            || !(0. ..=20.).contains(&self.repeat_radius)
        {
            return Err(invalid(
                "Sound attention requires ticks [0,3600], switch_margin [0,1], repeat_radius [0,20]m",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    #[default]
    AwaitingStep,
    Disabled,
    TerritoryReturn,
    OutsideTerritory,
    VisualPriority,
    HearingDisabled,
    NoStimuli,
    NoAudibleCue,
    Rejected,
    Acquired,
    ReplacedSound,
    ReplacedVisualMemory,
    ReplacedReport,
    LegacyStrongest,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub reason: Reason,
    pub audible_candidates: u16,
    pub selectable_candidates: u16,
    pub repeated_rejections: u16,
    pub commitment_rejections: u16,
    pub margin_rejections: u16,
    pub visual_memory_rejections: u16,
    pub strongest_audible_id: Option<u64>,
    pub strongest_audible_strength: Option<f32>,
    pub selected_noise_id: Option<u64>,
    pub active_noise_id: Option<u64>,
    pub active_strength: Option<f32>,
}
impl State {
    pub fn synchronize(&mut self, memory: Option<Observation>) {
        if !memory.is_some_and(|m| m.source == Source::Sound && m.noise_id == self.active_noise_id)
        {
            self.active_noise_id = None;
            self.active_strength = None;
        }
    }
    pub fn begin(&mut self, memory: Option<Observation>) {
        self.synchronize(memory);
        *self = Self {
            active_noise_id: self.active_noise_id,
            active_strength: self.active_strength,
            ..Default::default()
        };
    }
    /// No world or source-entity access: compare only the acoustic receipt and current evidence.
    pub fn consider(
        &mut self,
        config: Option<Config>,
        memory: Option<Observation>,
        now: u64,
        candidate: Observation,
        strength: f32,
    ) -> bool {
        self.audible_candidates += 1;
        if self
            .strongest_audible_strength
            .is_none_or(|s| strength >= s)
        {
            self.strongest_audible_id = candidate.noise_id;
            self.strongest_audible_strength = Some(strength);
        }
        if let (Some(config), Some(memory)) = (config, memory) {
            let age = now.saturating_sub(memory.tick);
            if memory.source.is_visual() {
                if age < u64::from(config.visual_priority_ticks) {
                    self.visual_memory_rejections += 1;
                    return false;
                }
            } else if memory.source == Source::Sound {
                if glam::Vec3::from(memory.position).distance_squared(candidate.position.into())
                    <= config.repeat_radius * config.repeat_radius
                {
                    self.repeated_rejections += 1;
                    return false;
                } else if age < u64::from(config.commitment_ticks) {
                    self.commitment_rejections += 1;
                    return false;
                } else if strength <= self.active_strength.unwrap_or(0.) + config.switch_margin {
                    self.margin_rejections += 1;
                    return false;
                }
            }
        }
        self.selectable_candidates += 1;
        true
    }
    pub fn accept(
        &mut self,
        config: Option<Config>,
        memory: Option<Observation>,
        observation: Observation,
        strength: f32,
    ) {
        self.reason = if config.is_none() {
            Reason::LegacyStrongest
        } else {
            match memory {
                None => Reason::Acquired,
                Some(m) if m.source == Source::Sound => Reason::ReplacedSound,
                Some(m) if m.source == Source::Report => Reason::ReplacedReport,
                _ => Reason::ReplacedVisualMemory,
            }
        };
        self.selected_noise_id = observation.noise_id;
        self.active_noise_id = observation.noise_id;
        self.active_strength = Some(strength);
    }
}
