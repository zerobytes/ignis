//! Capacity is reserved before payment; detaching an origin never allocates.
use crate::{attributes, identity::Reference};
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use std::collections::{BTreeMap, BTreeSet};
pub const CAPACITY: usize = 32; // Per actor slot, including retained generations.
type Key = (u64, u16, u64);
fn key(r: Reference) -> Key {
    (r.instance, r.slot, r.generation)
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Slot {
    #[serde(default)]
    pub actor: u16,
    pub reference: Option<Reference>,
    pub detached: bool,
    pub entity: u16,
    pub faction: Option<u8>,
    pub life: u64,
    pub dead: bool,
    pub attributes: attributes::SourceSnapshot,
}
#[derive(Debug, Clone, Serialize)]
pub struct State {
    slots: Vec<Slot>,
    actor_capacity: usize,
    #[serde(skip)]
    by_reference: BTreeMap<Key, u16>,
    #[serde(skip)]
    free: BTreeSet<u16>,
    #[serde(skip)]
    by_actor: Vec<u8>,
    #[serde(skip)]
    used: Vec<bool>,
}
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.actor_capacity == other.actor_capacity && self.slots == other.slots
    }
}
fn legacy_actor_capacity() -> usize {
    1
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Data {
            slots: Vec<Slot>,
            #[serde(default = "legacy_actor_capacity")]
            actor_capacity: usize,
        }
        let data = Data::deserialize(d)?;
        if data.actor_capacity > crate::targeting::MAX_ACTORS
            || data.slots.len() > data.actor_capacity * CAPACITY
        {
            return Err(D::Error::custom("Origin actor domain/capacity exceeded"));
        }
        let mut state = Self::new(data.actor_capacity);
        state.slots = data.slots;
        state.used.resize(state.slots.len(), false);
        for (index, slot) in state.slots.iter_mut().enumerate() {
            if slot.actor as usize >= state.actor_capacity
                || slot.attributes.bases.len() > 65
                || slot.attributes.contributions.len() > attributes::MAX_OWNER_CAPACITY
            {
                return Err(D::Error::custom("Invalid origin source/snapshot capacity"));
            }
            // Deserialization must restore the same infallible-detach reservation.
            slot.attributes
                .bases
                .reserve(65 - slot.attributes.bases.len());
            slot.attributes
                .contributions
                .reserve(attributes::CAPACITY.saturating_sub(slot.attributes.contributions.len()));
            if let Some(reference) = slot.reference {
                if state
                    .by_reference
                    .insert(key(reference), index as u16)
                    .is_some()
                {
                    return Err(D::Error::custom("Duplicate origin reference"));
                }
                state.by_actor[slot.actor as usize] += 1;
                if state.by_actor[slot.actor as usize] as usize > CAPACITY {
                    return Err(D::Error::custom("Origin source quota exceeded"));
                }
            } else {
                state.free.insert(index as u16);
            }
        }
        Ok(state)
    }
}
impl Default for State {
    fn default() -> Self {
        Self::new(1)
    }
}
impl State {
    pub fn new(actors: usize) -> Self {
        assert!(actors <= crate::targeting::MAX_ACTORS);
        Self {
            slots: Vec::new(),
            actor_capacity: actors,
            by_reference: BTreeMap::new(),
            free: BTreeSet::new(),
            by_actor: vec![0; actors],
            used: Vec::new(),
        }
    }
    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }
    pub fn available(&self, actor: u16, reference: Reference) -> bool {
        if let Some(&index) = self.by_reference.get(&key(reference)) {
            return self.slots[index as usize].actor == actor;
        }
        self.by_actor
            .get(actor as usize)
            .is_some_and(|&n| (n as usize) < CAPACITY)
            && (!self.free.is_empty() || self.slots.len() < self.actor_capacity * CAPACITY)
    }
    pub fn reserve(&mut self, actor: u16, reference: Reference) -> Option<u16> {
        if !self.available(actor, reference) {
            return None;
        }
        if let Some(&index) = self.by_reference.get(&key(reference)) {
            return Some(index);
        }
        let index = if let Some(index) = self.free.pop_first() {
            index
        } else {
            let index = self.slots.len() as u16;
            self.slots.push(Slot::default());
            self.used.push(false);
            index
        };
        let slot = &mut self.slots[index as usize];
        slot.actor = actor;
        slot.reference = Some(reference);
        self.by_actor[actor as usize] += 1;
        self.by_reference.insert(key(reference), index);
        Some(index)
    }
    pub fn get(&self, lease: Option<u16>, reference: Reference) -> Option<&Slot> {
        self.slots
            .get(lease? as usize)
            .filter(|s| s.reference == Some(reference))
    }
    pub(crate) fn source_mut(&mut self, reference: Reference) -> Option<&mut Slot> {
        let index = *self.by_reference.get(&key(reference))? as usize;
        self.slots
            .get_mut(index)
            .filter(|s| s.reference == Some(reference))
    }
    /// Grow live source storage before an effect commit; detached data stays frozen.
    pub(crate) fn reserve_buffs(&mut self, reference: Reference, required: usize) {
        assert!(required <= attributes::MAX_OWNER_CAPACITY);
        if let Some(slot) = self.source_mut(reference)
            && !slot.detached
        {
            slot.attributes
                .contributions
                .reserve(required.saturating_sub(slot.attributes.contributions.len()));
        }
    }
    pub(crate) fn prepare_from_buffs(&mut self, buffs: &attributes::Contributions) -> bool {
        let mut counts = BTreeMap::new();
        for i in buffs.iter() {
            *counts.entry(key(i.owner_instance)).or_insert(0usize) += 1;
        }
        if counts.values().any(|&n| n > attributes::MAX_OWNER_CAPACITY) {
            return false;
        }
        for slot in &mut self.slots {
            if !slot.detached
                && let Some(reference) = slot.reference
            {
                let required = counts.get(&key(reference)).copied().unwrap_or(0);
                slot.attributes
                    .contributions
                    .reserve(required.saturating_sub(slot.attributes.contributions.len()));
            }
        }
        true
    }
    /// Only live deliveries with the complete matching reference retain a lease.
    pub fn collect(&mut self, leases: impl IntoIterator<Item = (u16, Reference)>) {
        self.used.fill(false);
        for (lease, reference) in leases {
            if self
                .slots
                .get(lease as usize)
                .is_some_and(|s| s.reference == Some(reference))
            {
                self.used[lease as usize] = true;
            }
        }
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if !self.used[index]
                && let Some(reference) = slot.reference.take()
            {
                self.by_reference.remove(&key(reference));
                self.by_actor[slot.actor as usize] -= 1;
                self.free.insert(index as u16);
                slot.detached = false;
                slot.attributes.bases.clear();
                slot.attributes.contributions.clear();
            }
        }
    }
    pub fn inspect(&self) -> serde_json::Value {
        serde_json::json!({"capacity":self.actor_capacity*CAPACITY,"per_actor_capacity":CAPACITY,"allocated":self.slots.len(),"reserved":self.slots.iter().enumerate().filter_map(|(i,s)|s.reference.map(|r|serde_json::json!({"lease":i,"actor_slot":s.actor,"source_instance":r.id(),"detached":s.detached,"bases":s.attributes.bases.len(),"contributions":s.attributes.contributions.len(),"life_index":s.detached.then_some(s.life)}))).collect::<Vec<_>>()})
    }
}
