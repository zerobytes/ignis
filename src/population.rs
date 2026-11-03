//! Bounded requests and receipts for runtime population, separate from scene editing.
use crate::{
    identity::Reference,
    model::{Result, invalid},
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
pub const PER_TICK: usize = 64;
pub const MAX_INPUTS: usize = 4096;
pub const RECEIPTS: usize = 256;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Interact {
        actor: u16,
        instance: Reference,
        life: u64,
        request: crate::interactions::Request,
    },
    OrderNpc {
        actor: u16,
        instance: Reference,
        life: u64,
        order: crate::npc_cooperation::Frozen,
    },
    ControlGenerator {
        generator: u8,
        action: crate::generator_control::Action,
    },
    Despawn {
        actor: u16,
        instance: Reference,
    },
    Spawn {
        template: u8,
        position: [f32; 3],
        target: Option<Reference>,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub tick: u64,
    pub sequence: u64,
    pub operation: Operation,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    InteractionApplied,
    InteractionRejected,
    OrderApplied,
    ControlApplied,
    ControlUnchanged,
    ControlTerminal,
    Despawned,
    Spawned,
    InstanceUnavailable,
    Capacity,
    Occupied,
    UnsupportedPosition,
    PassageCapacity,
    IdentityExhausted,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub tick: u64,
    pub request_id: u64,
    pub operation: Operation,
    pub actor: Option<u16>,
    pub instance: Option<Reference>,
    pub outcome: Outcome,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub receipts: VecDeque<Receipt>,
    pub processed: u64,
    pub dropped_receipts: u64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            receipts: VecDeque::with_capacity(RECEIPTS),
            processed: 0,
            dropped_receipts: 0,
        }
    }
}
impl State {
    pub(crate) fn record(&mut self, e: Entry, outcome: Outcome, result: Option<(u16, Reference)>) {
        if self.receipts.len() == RECEIPTS {
            self.receipts.pop_front();
            self.dropped_receipts += 1;
        }
        self.receipts.push_back(Receipt {
            tick: e.tick,
            request_id: e.sequence,
            operation: e.operation,
            actor: result.map(|r| r.0),
            instance: result.map(|r| r.1),
            outcome,
        });
        self.processed += 1;
    }
}
pub fn validate_order(entries: &[Entry], ticks: usize) -> Result<()> {
    if entries.len() > MAX_INPUTS {
        return Err(invalid("Too many population inputs"));
    }
    let mut last = None;
    let mut count = 0;
    for (i, e) in entries.iter().enumerate() {
        if e.sequence != i as u64
            || e.tick > ticks as u64
            || e.tick >= crate::engine::MAX_TICKS as u64
            || last.is_some_and(|t| e.tick < t)
        {
            return Err(invalid("Invalid population input order, sequence or tick"));
        }
        count = if last == Some(e.tick) { count + 1 } else { 1 };
        if count > PER_TICK {
            return Err(invalid("Too many population inputs in tick"));
        }
        last = Some(e.tick);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_ring_and_replay_budgets() {
        let e = Entry {
            tick: 0,
            sequence: 0,
            operation: Operation::Despawn {
                actor: 0,
                instance: Reference {
                    slot: 0,
                    generation: 1,
                    instance: 1,
                },
            },
        };
        let mut s = State::default();
        for i in 0..1000 {
            s.record(
                Entry {
                    sequence: i,
                    tick: i,
                    ..e
                },
                Outcome::InstanceUnavailable,
                None,
            );
        }
        assert_eq!(s.receipts.len(), RECEIPTS);
        assert_eq!(s.dropped_receipts, 1000 - RECEIPTS as u64);
        assert_eq!(s.receipts.front().unwrap().request_id, 744);
        assert!(validate_order(&vec![e; MAX_INPUTS + 1], 0).is_err());
        let many: Vec<_> = (0..65).map(|i| Entry { sequence: i, ..e }).collect();
        assert!(validate_order(&many, 0).is_err());
        assert!(validate_order(&[Entry { tick: 1, ..e }], 0).is_err());
        assert!(validate_order(&[Entry { sequence: 1, ..e }], 0).is_err());
        assert!(validate_order(&[e], 0).is_ok());
    }
}
