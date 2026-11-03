//! Bounded identity storage. Generation checks separate recycled storage from logical individuals.
use crate::model::{Result, invalid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub slot: u16,
    pub generation: u64,
    pub instance: u64,
}
impl Reference {
    pub fn id(self) -> String {
        format!("{:016x}", self.instance)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Slot {
    generation: u64,
    occupant: Option<(u64, u16)>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    slots: Vec<Slot>,
    next_instance: u64,
}
/// A generation validates storage; an instance identifies its occupant permanently in this run.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    state: Snapshot,
}
impl Registry {
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 || capacity > crate::model::MAX_ENTITIES {
            return Err(invalid("Identity capacity requires 1..4096 slots"));
        }
        Ok(Self {
            state: Snapshot {
                slots: vec![
                    Slot {
                        generation: 1,
                        occupant: None
                    };
                    capacity
                ],
                next_instance: 1,
            },
        })
    }
    pub fn insert(&mut self, entity: u16) -> Result<Reference> {
        if self
            .state
            .slots
            .iter()
            .any(|s| s.occupant.is_some_and(|(_, e)| e == entity))
        {
            return Err(invalid("Entity already has an instance"));
        }
        let next = self
            .state
            .next_instance
            .checked_add(1)
            .ok_or_else(|| invalid("Instance identity exhausted"))?;
        let slot = self
            .state
            .slots
            .iter()
            .position(|s| s.occupant.is_none() && s.generation != u64::MAX)
            .ok_or_else(|| invalid("Identity capacity exhausted"))?;
        let s = &mut self.state.slots[slot];
        let reference = Reference {
            slot: slot as u16,
            generation: s.generation,
            instance: self.state.next_instance,
        };
        s.occupant = Some((reference.instance, entity));
        self.state.next_instance = next;
        Ok(reference)
    }
    pub fn resolve(&self, r: Reference) -> Option<u16> {
        let s = self.state.slots.get(r.slot as usize)?;
        let (id, entity) = s.occupant?;
        (s.generation == r.generation && id == r.instance).then_some(entity)
    }
    pub fn remove(&mut self, r: Reference) -> Result<u16> {
        let entity = self
            .resolve(r)
            .ok_or_else(|| invalid("Stale instance reference"))?;
        let s = &mut self.state.slots[r.slot as usize];
        // Exhausted slots retire permanently instead of wrapping into an old identity.
        s.generation = s
            .generation
            .checked_add(1)
            .ok_or_else(|| invalid("Generation exhausted"))?;
        s.occupant = None;
        Ok(entity)
    }
    pub fn reference(&self, entity: u16) -> Option<Reference> {
        self.state.slots.iter().enumerate().find_map(|(slot, s)| {
            let (instance, e) = s.occupant?;
            (e == entity).then_some(Reference {
                slot: slot as u16,
                generation: s.generation,
                instance,
            })
        })
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state.clone()
    }
}

/// Transport scope only; never stored in deterministic simulation/replay state.
/// Not an authentication secret. Process-start time + PID + monotonic nonce separate local runs.
pub fn fresh_scope() -> String {
    use std::sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    };
    static START: OnceLock<u128> = OnceLock::new();
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let start = START.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos()
    });
    let nonce = NEXT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .expect("execution scope exhausted");
    format!("{:x}-{:x}-{:x}", std::process::id(), start, nonce)
}

impl Registry {
    pub(crate) fn validate_checkpoint(&self, entities: usize) -> Result<()> {
        // Empty visual scenes reserve one unoccupied identity slot at construction.
        if self.state.slots.len() != entities.max(1)
            || entities > crate::model::MAX_ENTITIES
            || self.state.next_instance == 0
        {
            return Err(invalid("Checkpoint identity capacity"));
        }
        let mut instances = std::collections::BTreeSet::new();
        let mut occupied = std::collections::BTreeSet::new();
        for s in &self.state.slots {
            if s.generation == 0 {
                return Err(invalid("Checkpoint generation0"));
            }
            if let Some((instance, entity)) = s.occupant
                && (instance == 0
                    || instance >= self.state.next_instance
                    || entity as usize >= entities
                    || !instances.insert(instance)
                    || !occupied.insert(entity))
            {
                return Err(invalid("Checkpoint duplicate/future identity"));
            }
        }
        Ok(())
    }
}
impl Registry {
    pub(crate) fn historical(&self, r: Reference) -> bool {
        r.instance > 0
            && r.instance < self.state.next_instance
            && r.generation > 0
            && self.state.slots.get(r.slot as usize).is_some_and(|s| {
                r.generation < s.generation
                    || r.generation == s.generation
                        && s.occupant.is_some_and(|(i, _)| i == r.instance)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reused_slot_rejects_old_reference_and_preserves_other_occupants() {
        let mut r = Registry::new(2).unwrap();
        let a = r.insert(10).unwrap();
        let b = r.insert(20).unwrap();
        let before = r.snapshot();
        assert!(r.insert(30).is_err());
        assert_eq!(r.snapshot(), before);
        assert_eq!(r.remove(a).unwrap(), 10);
        let c = r.insert(30).unwrap();
        assert_eq!(a.slot, c.slot);
        assert_ne!(a.generation, c.generation);
        assert_ne!(a.instance, c.instance);
        assert_eq!(r.resolve(a), None);
        assert_eq!(r.resolve(b), Some(20));
        assert_eq!(r.resolve(c), Some(30));
        let before = r.snapshot();
        assert!(r.remove(a).is_err());
        assert_eq!(r.snapshot(), before);
    }
    #[test]
    fn identity_and_generation_exhaustion_never_wrap() {
        let mut r = Registry::new(1).unwrap();
        r.state.next_instance = u64::MAX;
        let before = r.snapshot();
        assert!(r.insert(1).is_err());
        assert_eq!(r.snapshot(), before);
        r.state.next_instance = 1;
        r.state.slots[0].generation = u64::MAX - 1;
        let a = r.insert(1).unwrap();
        r.remove(a).unwrap();
        assert!(r.insert(2).is_err());
        assert_eq!(r.resolve(a), None);
    }
    #[test]
    fn duplicate_and_forged_references_are_rejected_atomically() {
        assert!(Registry::new(0).is_err());
        assert!(Registry::new(crate::model::MAX_ENTITIES + 1).is_err());
        let mut r = Registry::new(2).unwrap();
        let a = r.insert(4).unwrap();
        let before = r.snapshot();
        assert!(r.insert(4).is_err());
        assert_eq!(r.snapshot(), before);
        assert_eq!(r.resolve(Reference { instance: 9, ..a }), None);
        assert_eq!(r.resolve(Reference { generation: 9, ..a }), None);
        assert_eq!(
            r.resolve(Reference {
                slot: u16::MAX,
                ..a
            }),
            None
        );
    }
    #[test]
    fn repeated_reuse_is_bounded_and_sequence_is_deterministic() {
        let mut a = Registry::new(1).unwrap();
        let mut b = Registry::new(1).unwrap();
        for _ in 0..10000 {
            let ar = a.insert(0).unwrap();
            let br = b.insert(0).unwrap();
            assert_eq!(ar, br);
            a.remove(ar).unwrap();
            b.remove(br).unwrap();
        }
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.state.slots.len(), 1);
        assert_ne!(fresh_scope(), fresh_scope());
    }
}
