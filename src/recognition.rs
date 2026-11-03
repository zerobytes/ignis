//! Bounded visual evidence, independent from the active pursuit memory.
use crate::{
    model::{Result, invalid},
    perception::{Observation, Source},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub acquire_ticks: u32,
    pub decay_ticks: u32,
}
impl Config {
    pub fn validate(self) -> Result<()> {
        if !(1..=3600).contains(&self.acquire_ticks) || !(1..=3600).contains(&self.decay_ticks) {
            return Err(invalid(
                "Recognition acquire_ticks and decay_ticks require 1..3600",
            ));
        }
        Ok(())
    }
    pub fn threshold(self) -> u32 {
        self.acquire_ticks * self.decay_ticks
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub evidence: u32,
    pub confirmed: bool,
    pub visible: bool,
    pub last_seen: Option<Observation>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Knowledge {
    pub threshold: u32,
    pub tracks: Vec<Track>,
}
impl Knowledge {
    pub fn begin(&mut self) {
        for track in &mut self.tracks {
            track.visible = false;
        }
    }
    /// Called once for each sensor-visible eligible actor in the shared pre-physics snapshot.
    pub fn observe(&mut self, config: Option<Config>, observation: Observation) -> bool {
        let Some(config) = config else {
            return true;
        };
        self.threshold = config.threshold();
        let slot = observation.actor_index.unwrap() as usize;
        if self.tracks.len() <= slot {
            self.tracks.resize(slot + 1, Track::default());
        }
        let track = &mut self.tracks[slot];
        if track
            .last_seen
            .is_some_and(|old| old.life != observation.life || old.instance != observation.instance)
        {
            *track = Track::default();
        }
        track.visible = true;
        track.last_seen = Some(observation);
        track.evidence = if observation.source == Source::Omniscient {
            self.threshold
        } else {
            (track.evidence + config.decay_ticks).min(self.threshold)
        };
        track.confirmed |= track.evidence == self.threshold;
        track.confirmed
    }
    pub fn finish(&mut self, config: Option<Config>) {
        let Some(config) = config else {
            return;
        };
        for track in &mut self.tracks {
            if !track.visible {
                track.evidence = track.evidence.saturating_sub(config.acquire_ticks);
                if track.evidence == 0 {
                    *track = Track::default();
                }
            }
        }
    }
    pub fn suspicious(&self) -> bool {
        self.tracks.iter().any(|t| t.evidence > 0 && !t.confirmed)
    }
}
