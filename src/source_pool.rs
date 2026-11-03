//! Dense delivery order with an encapsulated, derived index by actor slot.
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use std::ops::Deref;

pub trait SourceItem {
    const QUOTA: usize;
    const REQUIRES_TARGET: bool = false;
    fn source(&self) -> u16;
    fn target(&self) -> Option<u16> {
        None
    }
}
#[derive(Debug, Clone)]
pub struct Pool<T> {
    entries: Vec<T>,
    by_source: Vec<Vec<usize>>,
    by_target: Vec<Vec<usize>>,
    compacting: bool,
}
impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            by_source: Vec::new(),
            by_target: Vec::new(),
            compacting: false,
        }
    }
}
impl<T: PartialEq> PartialEq for Pool<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}
impl<T> Deref for Pool<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.entries
    }
}
impl<'a, T> IntoIterator for &'a Pool<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}
impl<T: Serialize> Serialize for Pool<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.entries.serialize(serializer)
    }
}
impl<'de, T: Deserialize<'de> + SourceItem> Deserialize<'de> for Pool<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entries = Vec::<T>::deserialize(deserializer)?;
        if entries.len() > T::QUOTA * crate::targeting::MAX_ACTORS
            || entries.iter().any(|e| {
                e.source() as usize >= crate::targeting::MAX_ACTORS
                    || (T::REQUIRES_TARGET && e.target().is_none())
                    || e.target()
                        .is_some_and(|t| t as usize >= crate::targeting::MAX_ACTORS)
            })
        {
            return Err(D::Error::custom("Delivery source domain/capacity exceeded"));
        }
        let mut pool = Self {
            entries,
            ..Default::default()
        };
        pool.reindex();
        if pool.by_source.iter().any(|bucket| bucket.len() > T::QUOTA) {
            return Err(D::Error::custom("Delivery source quota exceeded"));
        }
        Ok(pool)
    }
}
impl<T: SourceItem> Pool<T> {
    pub fn source_positions(&self, actor: u16) -> &[usize] {
        assert!(
            !self.compacting,
            "source query during unfinished compaction"
        );
        self.by_source
            .get(actor as usize)
            .map_or(&[], Vec::as_slice)
    }
    pub fn available(&self, actor: u16, actors: usize) -> bool {
        (actor as usize) < actors
            && self.entries.len() < actors * T::QUOTA
            && self.source_positions(actor).len() < T::QUOTA
    }
    pub fn target_positions(&self, actor: u16) -> &[usize] {
        assert!(
            !self.compacting,
            "target query during unfinished compaction"
        );
        self.by_target
            .get(actor as usize)
            .map_or(&[], Vec::as_slice)
    }
    pub fn push(&mut self, value: T) {
        assert!(!self.compacting);
        let source = value.source() as usize;
        assert!(source < crate::targeting::MAX_ACTORS);
        let target = value.target();
        assert!(!T::REQUIRES_TARGET || target.is_some());
        assert!(target.is_none_or(|target| (target as usize) < crate::targeting::MAX_ACTORS));
        assert!(
            self.by_source.get(source).map_or(0, Vec::len) < T::QUOTA,
            "validated source quota"
        );
        self.by_source
            .resize_with(self.by_source.len().max(source + 1), Vec::new);
        self.by_source[source].push(self.entries.len());
        if let Some(target) = target {
            let target = target as usize;
            self.by_target
                .resize_with(self.by_target.len().max(target + 1), Vec::new);
            self.by_target[target].push(self.entries.len());
        }
        self.entries.push(value);
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        for bucket in &mut self.by_source {
            bucket.clear();
        }
        for bucket in &mut self.by_target {
            bucket.clear();
        }
        self.compacting = false;
    }
    pub(crate) fn replace(&mut self, index: usize, value: T) {
        assert!(!self.compacting);
        assert_eq!(self.entries[index].source(), value.source());
        assert_eq!(self.entries[index].target(), value.target());
        self.entries[index] = value;
    }
    pub(crate) fn write_retained(&mut self, destination: usize, source: usize, value: T) {
        assert!(destination <= source);
        assert_eq!(self.entries[source].source(), value.source());
        assert_eq!(self.entries[source].target(), value.target());
        self.entries[destination] = value;
        self.compacting |= destination != source;
    }
    pub(crate) fn finish_retained(&mut self, len: usize) {
        let changed = self.compacting || len != self.entries.len();
        self.entries.truncate(len);
        if changed {
            self.reindex();
        }
    }
    fn reindex(&mut self) {
        for bucket in &mut self.by_source {
            bucket.clear();
        }
        for bucket in &mut self.by_target {
            bucket.clear();
        }
        for (index, value) in self.entries.iter().enumerate() {
            let source = value.source() as usize;
            self.by_source
                .resize_with(self.by_source.len().max(source + 1), Vec::new);
            self.by_source[source].push(index);
            if let Some(target) = value.target() {
                let target = target as usize;
                self.by_target
                    .resize_with(self.by_target.len().max(target + 1), Vec::new);
                self.by_target[target].push(index);
            }
        }
        self.compacting = false;
    }
}
