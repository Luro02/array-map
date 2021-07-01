use core::hash::BuildHasher;

use crate::{DefaultHashBuilder, OccupiedEntry};

#[derive(Debug)]
pub struct VacantEntry<'a, K: 'a, V: 'a, const N: usize, H: BuildHasher = DefaultHashBuilder> {
    key: K,
    entries: &'a mut [Option<(K, V)>; N],
    index: usize,
    hasher: &'a H,
    len: &'a mut usize,
}

impl<'a, K, V, const N: usize, H: BuildHasher> VacantEntry<'a, K, V, N, H> {
    #[must_use]
    pub(crate) fn new(
        key: K,
        entries: &'a mut [Option<(K, V)>; N],
        index: usize,
        hasher: &'a H,
        len: &'a mut usize,
    ) -> Self {
        assert!(index < N);
        assert!(entries[index].is_none());

        Self {
            key,
            entries,
            index,
            hasher,
            len,
        }
    }

    #[must_use]
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn insert(self, value: V) -> &'a mut V {
        debug_assert!(self.entries[self.index].is_none());
        self.entries[self.index] = Some((self.key, value));
        *self.len += 1;

        self.entries[self.index].as_mut().map(|(_, v)| v).unwrap()
    }

    #[must_use]
    pub fn into_key(self) -> K {
        self.key
    }

    #[must_use]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, N, H> {
        debug_assert!(self.entries[self.index].is_none());
        self.entries[self.index] = Some((self.key, value));
        *self.len += 1;

        OccupiedEntry::new(self.entries, self.index, self.hasher, self.len)
    }
}
