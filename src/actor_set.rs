//! Bounded actor membership, distinct from per-actor ability/grant bitmasks.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ActorSet([u64; 16]);
const _: () = assert!(crate::targeting::MAX_ACTORS == 16 * 64);
impl ActorSet {
    pub const EMPTY: Self = Self([0; 16]);
    pub fn contains(self, actor: usize) -> bool {
        actor < crate::targeting::MAX_ACTORS && self.0[actor / 64] & (1u64 << (actor % 64)) != 0
    }
    pub fn insert(&mut self, actor: usize) {
        assert!(
            actor < crate::targeting::MAX_ACTORS,
            "validated actor set capacity"
        );
        self.0[actor / 64] |= 1u64 << (actor % 64);
    }
    pub fn remove(&mut self, actor: usize) {
        if actor < crate::targeting::MAX_ACTORS {
            self.0[actor / 64] &= !(1u64 << (actor % 64));
        }
    }
    pub fn is_empty(self) -> bool {
        self.0.iter().all(|word| *word == 0)
    }
}
impl std::ops::BitOr for ActorSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self {
        for (a, b) in self.0.iter_mut().zip(rhs.0) {
            *a |= b;
        }
        self
    }
}

impl From<u16> for ActorSet {
    fn from(value: u16) -> Self {
        let mut result = Self::EMPTY;
        result.0[0] = u64::from(value);
        result
    }
}
