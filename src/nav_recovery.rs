//! Optional bounded retry policy; observes only the NPC's own physical movement.
use crate::model::{Result, invalid};
use glam::Vec2;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub retry_ticks: u32,
    pub max_retry_ticks: u32,
    pub stalled_ticks: u32,
    pub min_progress: f32,
}
impl Config {
    pub fn validate(self, speed: f32) -> Result<()> {
        if !(1..=3600).contains(&self.retry_ticks)
            || !(self.retry_ticks..=3600).contains(&self.max_retry_ticks)
            || !(12..=3600).contains(&self.stalled_ticks)
            || !self.min_progress.is_finite()
            || !(0.01..=1.).contains(&self.min_progress)
            || self.min_progress > speed * self.stalled_ticks as f32 / 120.
        {
            return Err(invalid(
                "Navigation recovery requires retry_ticks [1,3600], max_retry_ticks [retry_ticks,3600], stalled_ticks [12,3600], min_progress [0.01,1]m and at most half the nominal travel over stalled_ticks",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Failure {
    NoClearPath,
    NpcOffSurface,
    TargetOffSurface,
    NoProgress,
}
impl Failure {
    pub fn reason(self) -> &'static str {
        match self {
            Self::NoClearPath => "no_clear_path",
            Self::NpcOffSurface => "npc_off_surface",
            Self::TargetOffSurface => "target_off_surface",
            Self::NoProgress => "no_physical_progress",
        }
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub consecutive_failures: u32,
    pub failures: u32,
    pub path_attempts: u32,
    pub retry_at_tick: Option<u64>,
    pub last_failure: Option<Failure>,
    pub last_failure_tick: Option<u64>,
    pub progress_anchor: Option<[f32; 2]>,
    pub progress_since_tick: Option<u64>,
    pub movement_requested: bool,
}
impl State {
    pub fn failed(&mut self, c: Config, tick: u64, failure: Failure) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.failures = self.failures.saturating_add(1);
        self.last_failure = Some(failure);
        self.last_failure_tick = Some(tick);
        let shift = (self.consecutive_failures - 1).min(12);
        let delay = (u64::from(c.retry_ticks) << shift).min(u64::from(c.max_retry_ticks));
        self.retry_at_tick = Some(tick.saturating_add(delay));
        self.progress_anchor = None;
        self.progress_since_tick = None;
        self.movement_requested = false;
    }
    pub fn waiting(&mut self, tick: u64) -> bool {
        if self.retry_at_tick.is_some_and(|at| tick < at) {
            return true;
        }
        self.retry_at_tick = None;
        false
    }
    /// Returns true only after consecutive requested motion without enough displacement.
    pub fn stalled(&mut self, c: Config, tick: u64, p: Vec2) -> bool {
        if !self.movement_requested || self.progress_anchor.is_none() {
            self.progress_anchor = Some(p.to_array());
            self.progress_since_tick = Some(tick);
        } else if p.distance_squared(Vec2::from_array(self.progress_anchor.unwrap()))
            >= c.min_progress * c.min_progress
        {
            self.progress_anchor = Some(p.to_array());
            self.progress_since_tick = Some(tick);
            self.consecutive_failures = 0;
        } else if tick.saturating_sub(self.progress_since_tick.unwrap())
            >= u64::from(c.stalled_ticks)
        {
            return true;
        }
        self.movement_requested = false;
        false
    }
    pub fn arrived(&mut self) {
        self.consecutive_failures = 0;
        self.retry_at_tick = None;
        self.progress_anchor = None;
        self.progress_since_tick = None;
        self.movement_requested = false;
    }
}
