//! Behavioral home territory; it is not a collision boundary or a clipped navigation mesh.
use crate::{
    model::{NpcConfig, Result, invalid},
    navigation::NavGoal,
    perception::Observation,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub home: [f32; 2],
    pub radius: f32,
    pub arrival_distance: f32,
    pub rearm_ticks: u32,
}
impl Config {
    pub fn contains(self, p: [f32; 3]) -> bool {
        self.distance_squared(p) <= self.radius * self.radius
    }
    fn distance_squared(self, p: [f32; 3]) -> f32 {
        (p[0] - self.home[0]).powi(2) + (p[2] - self.home[1]).powi(2)
    }
    pub fn validate(self, npc: &NpcConfig) -> Result<()> {
        if !self.home.iter().all(|v| v.is_finite() && v.abs() <= 10000.)
            || !self.radius.is_finite()
            || !(0.5..=32.).contains(&self.radius)
            || !self.arrival_distance.is_finite()
            || !(0.1..=2.).contains(&self.arrival_distance)
            || self.arrival_distance + 0.02 >= self.radius
            || self.rearm_ticks > 3600
        {
            return Err(invalid(
                "Territory requires finite home within +/-10000m, radius [0.5,32]m, arrival [0.1,2]m below radius-0.02, rearm [0,3600] ticks",
            ));
        }
        if let Some(patrol) = &npc.patrol {
            patrol.validate()?;
            if patrol.resolve(0.).iter().any(|p| !self.contains(*p)) {
                return Err(invalid("Patrol points must be inside territory"));
            }
        }
        if npc.objectives.as_ref().is_some_and(|os|os.iter().any(|o|matches!(o.action,crate::objectives::Action::Guard{point,..} if !self.contains([point[0],0.,point[1]])))) {return Err(invalid("Guard points must be inside territory"))}
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Unrestricted,
    Active,
    Returning,
    CoolingDown,
    Disabled,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    OwnOutside,
    TargetLeft,
    GoalOutside,
    DisplacedFromHome,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub phase: Phase,
    pub last_trigger: Option<Trigger>,
    pub return_started_tick: Option<u64>,
    pub ready_at_tick: Option<u64>,
    pub returns: u32,
    pub changed_this_tick: bool,
    pub rejected_visual: u16,
    pub rejected_sound: u16,
    pub active_target_departed: bool,
}
impl State {
    pub fn blocked(&self) -> bool {
        matches!(self.phase, Phase::Returning | Phase::CoolingDown)
    }
    pub fn begin(&mut self, config: Option<Config>, own: [f32; 3], tick: u64) {
        let Some(c) = config else {
            *self = Self::default();
            return;
        };
        self.changed_this_tick = false;
        self.rejected_visual = 0;
        self.rejected_sound = 0;
        self.active_target_departed = false;
        let at_home = c.distance_squared(own) <= (c.arrival_distance + 0.02).powi(2);
        if matches!(self.phase, Phase::Unrestricted | Phase::Disabled) {
            self.phase = Phase::Active;
        }
        if self.phase == Phase::Returning && at_home {
            self.phase = Phase::CoolingDown;
            self.ready_at_tick = Some(tick + u64::from(c.rearm_ticks));
            self.changed_this_tick = true;
        }
        if self.phase == Phase::CoolingDown {
            if !at_home {
                self.start(Trigger::DisplacedFromHome, tick);
            } else if tick >= self.ready_at_tick.unwrap() {
                self.phase = Phase::Active;
                self.ready_at_tick = None;
                self.changed_this_tick = true;
            }
        }
        if self.phase == Phase::Active && !c.contains(own) {
            self.start(Trigger::OwnOutside, tick);
        }
    }
    pub fn start(&mut self, trigger: Trigger, tick: u64) {
        if !self.blocked() {
            self.returns = self.returns.saturating_add(1);
        }
        self.phase = Phase::Returning;
        self.last_trigger = Some(trigger);
        self.return_started_tick = Some(tick);
        self.ready_at_tick = None;
        self.changed_this_tick = true;
    }
    /// Called only after the sensor has produced an observation, never with hidden world positions.
    pub fn allow_visual(
        &mut self,
        config: Option<Config>,
        o: Observation,
        prior: Option<Observation>,
    ) -> bool {
        if config.is_none_or(|c| c.contains(o.position)) {
            return true;
        }
        self.rejected_visual += 1;
        if prior.is_some_and(|m| {
            m.source.is_visual() && m.actor_index.is_some() && m.instance == o.instance
        }) {
            self.active_target_departed = true;
        }
        false
    }
    pub fn return_goal(&self, config: Config, own: [f32; 3], half_height: f32) -> Option<NavGoal> {
        (self.phase == Phase::Returning).then_some(NavGoal {
            position: [config.home[0], own[1], config.home[1]],
            excluded_entity: None,
            target_half_height: half_height,
            stop_distance: config.arrival_distance,
            remaining_travel: None,
            replan_on_target_change: false,
        })
    }
}
