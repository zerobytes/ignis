//! Runtime producer control. Existing individuals and corpse retirement keep running.
use crate::{generators::Generator, population::Outcome, waves::Phase};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Activate,
    Pause,
    Cancel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Paused,
    Cancelled,
}

impl Generator {
    pub fn production_enabled(&self) -> bool {
        self.status == Status::Active && self.activation.is_none_or(|a| a.open)
    }
    pub(crate) fn sync_production_clock(&mut self, tick: u64, max_births: u32) {
        if self.status == Status::Cancelled || self.finished(max_births) {
            return;
        }
        if self.production_enabled() {
            if let Some(since) = self.paused_since.take() {
                let elapsed = tick - since;
                self.next_attempt_tick += elapsed;
                if let Some(w) = &mut self.wave
                    && w.phase == Phase::Delay
                {
                    w.due_tick += elapsed;
                }
            }
        } else {
            self.paused_since.get_or_insert(tick);
        }
    }
    pub fn finished(&self, max_births: u32) -> bool {
        self.wave
            .map_or(self.births >= max_births, |w| w.phase == Phase::Completed)
    }

    pub(crate) fn control(&mut self, action: Action, tick: u64, max_births: u32) -> Outcome {
        if self.status == Status::Cancelled {
            return if action == Action::Cancel {
                Outcome::ControlUnchanged
            } else {
                Outcome::ControlTerminal
            };
        }
        if self.finished(max_births) {
            return Outcome::ControlTerminal;
        }
        match action {
            Action::Activate if self.status == Status::Paused => {
                self.status = Status::Active;
            }
            Action::Pause if self.status == Status::Active => {
                self.status = Status::Paused;
            }
            Action::Cancel => {
                self.status = Status::Cancelled;
                self.paused_since = None;
                self.cancelled_tick = Some(tick);
            }
            _ => return Outcome::ControlUnchanged,
        }
        Outcome::ControlApplied
    }
}
