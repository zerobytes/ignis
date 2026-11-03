//! Fixed world volumes. Membership is sampled at pulses, not tracked as knowledge.
use crate::{
    abilities::Cast,
    area::Area,
    model::{Result, invalid},
    periodic::Reapply,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const CAPACITY: usize = 16;
pub const RECEIPTS: usize = 128;
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    #[default]
    Fixed,
    Source,
}
impl Anchor {
    pub fn is_fixed(&self) -> bool {
        *self == Self::Fixed
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    #[serde(default, skip_serializing_if = "Anchor::is_fixed")]
    pub anchor: Anchor,
    pub duration_ticks: u32,
    pub interval_ticks: u32,
    pub apply_on_spawn: bool,
    pub reapply: Reapply,
    pub end_on_source_death: bool,
}
impl Schedule {
    pub fn validate(self, version: u32) -> Result<()> {
        if self.anchor == Anchor::Source && version < 43 {
            return Err(invalid("Source-anchored area requires version43"));
        }
        if version < 42
            || !(1..=216_000).contains(&self.duration_ticks)
            || !(1..=3600).contains(&self.interval_ticks)
            || (!self.apply_on_spawn && self.interval_ticks >= self.duration_ticks)
        {
            return Err(invalid(
                "Persistent area requires version42, duration1..216000, interval1..3600 and at least one pulse before expiration",
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
    pub center: [f32; 3],
    pub area: Area,
    pub created_tick: u64,
    pub expires_at: u64,
    pub next_tick: u64,
    pub pulses: u32,
}
impl crate::source_pool::SourceItem for Instance {
    const QUOTA: usize = CAPACITY;
    fn source(&self) -> u16 {
        self.actor
    }
}
impl Instance {
    pub fn schedule(self) -> Schedule {
        self.area.persistence.expect("persistent instance")
    }
    pub fn end(self, tick: u64, dead: bool) -> Option<Outcome> {
        if tick >= self.expires_at {
            Some(Outcome::Expired)
        } else if dead && self.schedule().end_on_source_death {
            Some(Outcome::SourceDead)
        } else {
            None
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Created,
    Refreshed,
    Expired,
    SourceDead,
    SourceRemoved,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub actor_instance: crate::identity::Reference,
    pub actor: u16,
    pub execution_id: u64,
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
    /// Presentation/inspection use post-physics centers; pulse receipts keep their own snapshot.
    pub fn sync_centers(&mut self, mut position: impl FnMut(u16) -> [f32; 3]) {
        for index in 0..self.active.len() {
            let mut i = self.active[index];
            if i.schedule().anchor == Anchor::Source {
                i.center = position(i.actor);
                self.active.replace(index, i);
            }
        }
    }
    pub fn matching(
        &self,
        actor: u16,
        definition: u8,
        schedule: Schedule,
        source: crate::identity::Reference,
    ) -> Option<usize> {
        if schedule.reapply == Reapply::Independent {
            return None;
        }
        self.active
            .source_positions(actor)
            .iter()
            .copied()
            .find(|&index| {
                let i = &self.active[index];
                i.definition == definition && i.cast.life.source_instance == source
            })
    }
    pub fn record(&mut self, i: Instance, tick: u64, outcome: Outcome, replaced_by: Option<u64>) {
        if self.receipts.len() == RECEIPTS {
            self.receipts.pop_front();
            self.dropped_receipts += 1;
        }
        self.receipts.push_back(Receipt {
            actor_instance: i.cast.life.source_instance,
            actor: i.actor,
            execution_id: i.cast.id,
            tick,
            outcome,
            replaced_by,
        });
    }
    pub fn install(&mut self, i: Instance) {
        if let Some(index) = self.matching(
            i.actor,
            i.definition,
            i.schedule(),
            i.cast.life.source_instance,
        ) {
            self.record(
                self.active[index],
                i.created_tick,
                Outcome::Refreshed,
                Some(i.cast.id),
            );
            self.active.replace(index, i);
        } else {
            self.active.push(i);
        }
        self.record(i, i.created_tick, Outcome::Created, None);
    }
    pub fn inspect(
        &self,
        scene: &crate::model::Scene,
        actors: &crate::targeting::Catalog,
    ) -> serde_json::Value {
        use serde_json::json;
        let active:Vec<_> = self.active.iter().map(|i|json!({
            "actor":actors.label(scene,i.cast.life.source_instance),"actor_instance":i.cast.life.source_instance.id(),"ability":scene.ability_catalog.abilities[i.definition as usize].id,
            "anchor_actor":(i.schedule().anchor == Anchor::Source).then(|| actors.label(scene,i.cast.life.source_instance)).flatten(),"slot":i.cast.slot+1,"execution_id":i.cast.id,"center":i.center,"area":i.area,
            "created_tick":i.created_tick,"expires_at":i.expires_at,"next_tick":i.next_tick,"pulses":i.pulses
        })).collect();
        let receipts:Vec<_> = self.receipts.iter().map(|r|json!({"actor":actors.label(scene,r.actor_instance),"actor_instance":r.actor_instance.id(),"execution_id":r.execution_id,"tick":r.tick,"outcome":r.outcome,"replaced_by":r.replaced_by})).collect();
        json!({"active":active,"receipts":receipts,"capacity":CAPACITY*actors.actors.len(),"per_actor_capacity":CAPACITY,"receipt_capacity":RECEIPTS,"dropped_receipts":self.dropped_receipts})
    }
}
