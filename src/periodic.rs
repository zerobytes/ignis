//! Bounded schedules installed by a successful ability delivery.
use crate::{
    abilities::Cast,
    model::{Result, invalid},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
pub const CAPACITY: usize = 64;
const RECEIPTS: usize = 256;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reapply {
    Refresh,
    Independent,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    pub interval_ticks: u32,
    pub pulses: u32,
    pub reapply: Reapply,
}
impl Schedule {
    pub fn validate(self, version: u32) -> Result<()> {
        if version < 33
            || !(1..=3600).contains(&self.interval_ticks)
            || !(1..=3600).contains(&self.pulses)
            || u64::from(self.interval_ticks) * u64::from(self.pulses) > 216_000
        {
            return Err(invalid(
                "Periodic effects require version33, interval/pulses1..3600 and duration<=216000 ticks",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Instance {
    pub actor: u16,
    pub definition: u8,
    pub cast: Cast,
    pub applied_tick: u64,
    pub next_tick: u64,
    pub remaining: u32,
    pub interval: u32,
}
impl crate::source_pool::SourceItem for Instance {
    const QUOTA: usize = CAPACITY;
    const REQUIRES_TARGET: bool = true;
    fn source(&self) -> u16 {
        self.actor
    }
    fn target(&self) -> Option<u16> {
        self.cast.target
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Applied,
    Dispelled,
    Refreshed,
    Finished,
    TargetDead,
    LifeChanged,
    InstanceChanged,
    EffectFailed,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Removal {
    pub actor_instance: crate::identity::Reference,
    pub actor: u16,
    pub execution_id: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub actor_instance: crate::identity::Reference,
    pub target_instance: crate::identity::Reference,
    pub removed_by: Option<Removal>,
    pub execution_id: u64,
    pub actor: u16,
    pub target: u16,
    pub tick: u64,
    pub outcome: Outcome,
    pub replaced_by: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub active: crate::source_pool::Pool<Instance>,
    pub receipts: VecDeque<Receipt>,
    pub dropped_receipts: u64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            active: Default::default(),
            receipts: VecDeque::with_capacity(RECEIPTS),
            dropped_receipts: 0,
        }
    }
}
impl State {
    pub fn matching(
        &self,
        actor: u16,
        definition: u8,
        target: u16,
        schedule: Schedule,
        life: crate::abilities::LifeStamp,
    ) -> Option<usize> {
        if schedule.reapply == Reapply::Independent {
            return None;
        }
        self.active
            .source_positions(actor)
            .iter()
            .copied()
            .find(|&index| {
                let e = &self.active[index];
                e.definition == definition
                    && e.cast.target == Some(target)
                    && e.cast.life.source_instance == life.source_instance
                    && e.cast.life.target_instance == life.target_instance
            })
    }
    pub fn available(
        &self,
        actor: u16,
        actors: usize,
        definition: u8,
        target: u16,
        schedule: Schedule,
        life: crate::abilities::LifeStamp,
    ) -> bool {
        self.active.available(actor, actors)
            || self
                .matching(actor, definition, target, schedule, life)
                .is_some()
    }
    pub fn record(&mut self, e: Instance, tick: u64, outcome: Outcome, replaced_by: Option<u64>) {
        if self.receipts.len() == RECEIPTS {
            self.receipts.pop_front();
            self.dropped_receipts += 1;
        }
        self.receipts.push_back(Receipt {
            actor_instance: e.cast.life.source_instance,
            target_instance: e.cast.life.target_instance.unwrap(),
            removed_by: None,
            execution_id: e.cast.id,
            actor: e.actor,
            target: e.cast.target.expect("periodic target"),
            tick,
            outcome,
            replaced_by,
        });
    }
    /// IDs are selected against the unchanged agenda used to build the plan.
    pub(crate) fn dispel(
        &mut self,
        selected: &crate::transaction::IdSet<512>,
        tick: u64,
        actor: u16,
        execution_id: u64,
        actor_instance: crate::identity::Reference,
    ) {
        if selected.is_empty() {
            return;
        }
        let mut retained = 0;
        for i in 0..self.active.len() {
            let effect = self.active[i];
            if selected.contains(effect.cast.id) {
                self.record(effect, tick, Outcome::Dispelled, None);
                self.receipts.back_mut().unwrap().removed_by = Some(Removal {
                    actor_instance,
                    actor,
                    execution_id,
                });
            } else {
                self.active.write_retained(retained, i, effect);
                retained += 1;
            }
        }
        self.active.finish_retained(retained);
    }
    /// Caller validated capacity and target components in the same atomic operation.
    pub fn install(
        &mut self,
        actor: u16,
        definition: u8,
        cast: Cast,
        schedule: Schedule,
        tick: u64,
    ) {
        let next = Instance {
            actor,
            definition,
            cast,
            applied_tick: tick,
            next_tick: tick + u64::from(schedule.interval_ticks),
            remaining: schedule.pulses,
            interval: schedule.interval_ticks,
        };
        if let Some(i) = self.matching(actor, definition, cast.target.unwrap(), schedule, cast.life)
        {
            self.record(self.active[i], tick, Outcome::Refreshed, Some(cast.id));
            self.active.replace(i, next);
        } else {
            self.active.push(next);
        }
        self.record(next, tick, Outcome::Applied, None);
    }
}
