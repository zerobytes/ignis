//! Bounded priority selection over accepted evidence, independent of world/target access.
use crate::{
    model::{Result, invalid},
    navigation::NavGoal,
    perception::{Observation, Source},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Always,
    Evidence,
    VisualMemory,
    SoundMemory,
    VisibleTarget,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Routine,
    Tactical {
        configuration: crate::npc_tactics::Config,
    },
    Respond,
    Flee {
        distance: f32,
    },
    Patrol,
    Guard {
        point: [f32; 2],
        arrival_distance: f32,
    },
    #[serde(rename = "guard_3d")]
    Guard3d {
        point: [f32; 3],
        arrival_distance: f32,
    },
    UseAbility {
        slot: u8,
        retry_ticks: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        policy: Option<crate::ability_policy::Policy>,
    },
    Wait,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Routine,
    Tactical,
    Respond,
    Flee,
    Patrol,
    Guard,
    UseAbility,
    Wait,
}
impl Action {
    pub fn kind(self) -> Kind {
        match self {
            Self::Routine => Kind::Routine,
            Self::Tactical { .. } => Kind::Tactical,
            Self::Respond => Kind::Respond,
            Self::Flee { .. } => Kind::Flee,
            Self::Patrol => Kind::Patrol,
            Self::Guard { .. } | Self::Guard3d { .. } => Kind::Guard,
            Self::Wait => Kind::Wait,
            Self::UseAbility { .. } => Kind::UseAbility,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Objective {
    pub priority: u8,
    pub when: Condition,
    pub action: Action,
}
pub fn validate(objectives: &[Objective], has_patrol: bool) -> Result<()> {
    if !(1..=8).contains(&objectives.len()) {
        return Err(invalid("Objectives require 1..8 entries"));
    }
    for o in objectives {
        match o.action {
            Action::Tactical { configuration } => configuration.validate()?,
            Action::Patrol if !has_patrol => {
                return Err(invalid("Patrol objective requires a patrol route"));
            }
            Action::Flee { distance }
                if !distance.is_finite() || !(0.5..=32.).contains(&distance) =>
            {
                return Err(invalid("Flee distance must be [0.5,32]m"));
            }
            Action::Guard {
                point,
                arrival_distance,
            } if !point.iter().all(|v| v.is_finite() && v.abs() <= 10000.)
                || !arrival_distance.is_finite()
                || !(0.1..=2.).contains(&arrival_distance) =>
            {
                return Err(invalid(
                    "Guard requires finite XZ point within +/-10000m and arrival [0.1,2]m",
                ));
            }
            Action::Guard3d {
                point,
                arrival_distance,
            } if !point.iter().all(|v| v.is_finite() && v.abs() <= 10000.)
                || !arrival_distance.is_finite()
                || !(0.1..=2.).contains(&arrival_distance) =>
            {
                return Err(invalid(
                    "Guard3d requires finite XYZ center point and arrival [0.1,2]m",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    #[default]
    Legacy,
    Activated,
    HigherPriority,
    ConditionLost,
    Retained,
    NoEligible,
    Disabled,
    TerritoryReturn,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct State {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mind: Option<crate::npc_mind::Decision>,
    pub active: Option<u8>,
    pub ability_decisions: [crate::ability_policy::Decision; 8],
    pub action: Option<Kind>,
    pub eligible_mask: u8,
    pub reason: Reason,
    pub previous: Option<u8>,
    pub started_tick: Option<u64>,
    pub transitions: u32,
    pub activated_mask: u8,
    pub resumed: bool,
    pub changed_this_tick: bool,
    pub destination: Option<[f32; 3]>,
    pub evidence_tick: Option<u64>,
}
impl State {
    pub fn territory_suspend(&mut self) {
        self.changed_this_tick = self.active.is_some();
        if self.changed_this_tick {
            self.previous = self.active;
            self.transitions = self.transitions.saturating_add(1);
        }
        self.active = None;
        self.action = None;
        self.eligible_mask = 0;
        self.started_tick = None;
        self.destination = None;
        self.evidence_tick = None;
        self.resumed = false;
        self.reason = Reason::TerritoryReturn;
        self.ability_decisions = Default::default();
    }

    /// First authored entry wins a priority tie. No sorting, allocation or world query.
    pub fn select(
        &mut self,
        config: Option<&[Objective]>,
        memory: Option<Observation>,
        visible: bool,
        tick: u64,
    ) -> Option<Action> {
        self.select_with_abilities(config, memory, visible, tick, 0)
    }
    pub fn select_with_abilities(
        &mut self,
        config: Option<&[Objective]>,
        memory: Option<Observation>,
        visible: bool,
        tick: u64,
        ready: u16,
    ) -> Option<Action> {
        let mut eligible = 0;
        if let Some(config) = config {
            for (i, objective) in config.iter().enumerate() {
                let available = match objective.action {
                    Action::UseAbility { slot, .. } => {
                        (1..=16).contains(&slot) && ready & (1 << (slot - 1)) != 0
                    }
                    _ => true,
                };
                if available {
                    eligible |= 1 << i;
                }
            }
        }
        self.select_filtered(config, memory, visible, tick, eligible)
    }
    pub fn select_filtered(
        &mut self,
        config: Option<&[Objective]>,
        memory: Option<Observation>,
        visible: bool,
        tick: u64,
        eligible: u8,
    ) -> Option<Action> {
        self.select_minded(config, memory, visible, tick, eligible, None)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn select_minded(
        &mut self,
        config: Option<&[Objective]>,
        memory: Option<Observation>,
        visible: bool,
        tick: u64,
        eligible: u8,
        mind: Option<crate::npc_mind::Context>,
    ) -> Option<Action> {
        let Some(config) = config else {
            *self = Self::default();
            return None;
        };
        let mut chosen: Option<usize> = None;
        self.eligible_mask = 0;
        for (i, o) in config.iter().enumerate() {
            let condition = match o.when {
                Condition::Always => true,
                Condition::Evidence => memory.is_some(),
                Condition::VisualMemory => memory.is_some_and(|m| m.source.is_visual()),
                Condition::SoundMemory => memory.is_some_and(|m| m.source == Source::Sound),
                Condition::VisibleTarget => visible,
            };
            if eligible & (1 << i) == 0
                || !condition
                || (matches!(
                    o.action,
                    Action::Respond | Action::Flee { .. } | Action::Tactical { .. }
                ) && memory.is_none())
            {
                continue;
            }
            self.eligible_mask |= 1 << i;
            if chosen.is_none_or(|best| o.priority > config[best].priority) {
                chosen = Some(i)
            }
        }
        if let Some(context) = mind {
            chosen = crate::npc_mind::choose(
                &mut self.mind,
                context,
                config,
                self.eligible_mask,
                self.active,
                memory,
                visible,
                tick,
            );
        } else {
            self.mind = None;
        }
        let next = chosen.map(|i| i as u8);
        self.changed_this_tick = self.active != next;
        self.reason = if next.is_none() {
            Reason::NoEligible
        } else if self.changed_this_tick {
            if self.active.is_none() {
                Reason::Activated
            } else if self
                .active
                .is_some_and(|i| self.eligible_mask & (1 << i) != 0)
            {
                Reason::HigherPriority
            } else {
                Reason::ConditionLost
            }
        } else {
            Reason::Retained
        };
        if self.changed_this_tick {
            self.previous = self.active;
            self.active = next;
            self.started_tick = next.map(|_| tick);
            self.transitions = self.transitions.saturating_add(1);
            self.resumed = next.is_some_and(|i| self.activated_mask & (1 << i) != 0);
            if let Some(i) = next {
                self.activated_mask |= 1 << i;
            }
            self.destination = None;
            self.evidence_tick = None;
        }
        let action = chosen.map(|i| config[i].action);
        self.action = action.map(Action::kind);
        action
    }
    /// Only own position and a frozen accepted receipt are passed to movement actions.
    pub fn goal(
        &mut self,
        action: Option<Action>,
        memory: Option<Observation>,
        own: [f32; 3],
        height: f32,
    ) -> Option<NavGoal> {
        let (position, stop) = match action? {
            Action::Guard3d {
                point,
                arrival_distance,
            } => (point, arrival_distance),
            Action::Guard {
                point,
                arrival_distance,
            } => ([point[0], own[1], point[1]], arrival_distance),
            Action::Flee { distance } => {
                let memory = memory?;
                if self.destination.is_none() || self.evidence_tick != Some(memory.tick) {
                    let offset =
                        glam::Vec2::new(own[0] - memory.position[0], own[2] - memory.position[2]);
                    let direction = offset.try_normalize().unwrap_or(glam::Vec2::X);
                    self.destination = Some(if offset.length_squared() >= distance * distance {
                        own
                    } else {
                        [
                            memory.position[0] + direction.x * distance,
                            own[1],
                            memory.position[2] + direction.y * distance,
                        ]
                    });
                    self.evidence_tick = Some(memory.tick);
                }
                (self.destination.unwrap(), 0.1)
            }
            _ => return None,
        };
        self.destination = Some(position);
        Some(NavGoal {
            position,
            excluded_entity: None,
            target_half_height: height,
            stop_distance: stop,
            remaining_travel: None,
            replan_on_target_change: false,
        })
    }
}
