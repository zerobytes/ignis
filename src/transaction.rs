//! Bounded write sets. Storage depends on an operation's footprint, never world size.

pub(crate) struct Writes<K, V, const N: usize> {
    entries: [Option<(K, V)>; N],
    len: usize,
}

impl<K: Copy + Eq, V: Copy, const N: usize> Default for Writes<K, V, N> {
    fn default() -> Self {
        Self {
            entries: [None; N],
            len: 0,
        }
    }
}

impl<K: Copy + Eq, V: Copy, const N: usize> Writes<K, V, N> {
    /// Read lazily, once per key. Subsequent operators observe earlier planned writes.
    pub(crate) fn get_or_insert(&mut self, key: K, read: impl FnOnce() -> V) -> &mut V {
        let index = self.entries[..self.len]
            .iter()
            .position(|entry| entry.as_ref().is_some_and(|(k, _)| *k == key))
            .unwrap_or_else(|| {
                assert!(
                    self.len < N,
                    "operation exceeded its validated write footprint"
                );
                let index = self.len;
                self.entries[index] = Some((key, read()));
                self.len += 1;
                index
            });
        &mut self.entries[index].as_mut().unwrap().1
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.entries[..self.len].iter().flatten()
    }

    pub(crate) fn sort_by_key<T: Ord>(&mut self, key: impl Fn(K) -> T) {
        self.entries[..self.len].sort_unstable_by_key(|e| key(e.as_ref().unwrap().0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_writes_read_once_and_leave_other_thousand_actor_fields_untouched() {
        for population in [1, 9, 1000, 4096] {
            let mut ledger = vec![[100_i64; 32]; population];
            let mut changes = Writes::<(usize, usize), i64, 16>::default();
            let mut reads = 0;
            let key = (population - 1, 31);
            *changes.get_or_insert(key, || {
                reads += 1;
                ledger[key.0][key.1]
            }) -= 17;
            *changes.get_or_insert(key, || panic!("must reuse planned value")) += 3;
            assert_eq!(reads, 1);
            assert_eq!(changes.iter().count(), 1);
            // Dropping a failed plan has no visible writes.
            assert!(ledger.iter().flatten().all(|v| *v == 100));
            for &((actor, resource), value) in changes.iter() {
                ledger[actor][resource] = value;
            }
            assert_eq!(ledger[key.0][key.1], 86);
            assert_eq!(ledger.iter().flatten().filter(|v| **v != 100).count(), 1);
        }
    }
}

#[cfg(test)]
mod capacity_tests {
    use super::*;
    #[test]
    fn full_cost_and_effect_footprint_reuses_keys_and_sorts_only_touched_records() {
        let mut writes = Writes::<(usize, usize), i64, 16>::default();
        for resource in 24..32 {
            *writes.get_or_insert((999, resource), || 100) -= 1;
        }
        for resource in 0..8 {
            *writes.get_or_insert((0, resource), || 100) += 1;
        }
        assert_eq!(writes.iter().count(), 16);
        *writes.get_or_insert((999, 31), || panic!("existing")) += 1;
        writes.sort_by_key(|key| key);
        assert_eq!(writes.iter().next(), Some(&((0, 0), 101)));
        assert_eq!(writes.iter().last(), Some(&((999, 31), 100)));
    }
}

/// Sorted local membership, bounded by one operation's removal footprint.
#[derive(Default)]
pub(crate) struct IdSet<const N: usize> {
    sorted: Vec<u64>,
}
impl<const N: usize> IdSet<N> {
    pub(crate) fn contains(&self, id: u64) -> bool {
        self.sorted.binary_search(&id).is_ok()
    }
    pub(crate) fn insert(&mut self, id: u64) {
        if let Err(index) = self.sorted.binary_search(&id) {
            assert!(
                self.sorted.len() < N,
                "validated local identifier footprint"
            );
            self.sorted.insert(index, id);
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.sorted.is_empty()
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.sorted.iter().copied()
    }
}
