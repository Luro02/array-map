use core::hash::BuildHasher;

use crate::utils::invariant;
use crate::OccupiedEntry;

#[derive(Debug)]
pub struct VacantEntry<'a, K: 'a, V: 'a, B, const N: usize> {
    key: K,
    entries: &'a mut [Option<(K, V)>; N],
    index: usize,
    build_hasher: &'a B,
    len: &'a mut usize,
}

impl<'a, K, V, B: BuildHasher, const N: usize> VacantEntry<'a, K, V, B, N> {
    /// Constructs a new `VacantEntry`.
    ///
    /// # Safety
    ///
    /// The following invariants must hold:
    /// - `entries.len() == N` (should be guaranteed by the compiler)
    /// - `index < N` (index must not be out of bounds)
    /// - `entries[index].is_none()` (otherwise the entry would not be vacant)
    /// - `len <= N`
    #[must_use]
    pub(crate) unsafe fn new(
        key: K,
        entries: &'a mut [Option<(K, V)>; N],
        index: usize,
        build_hasher: &'a B,
        len: &'a mut usize,
    ) -> Self {
        // SAFETY: this assumption should be guranteed by the compiler (length is
        //         encoded in the type)
        invariant(entries.len() == N);
        // SAFETY: index must be valid
        invariant(index < N);
        // SAFETY: a `VacantEntry` should be vacant
        invariant(entries[index].is_none());
        // SAFETY: length should be smaller than N
        invariant(*len <= N);

        Self {
            key,
            entries,
            index,
            build_hasher,
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
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, B, N> {
        debug_assert!(self.entries[self.index].is_none());
        self.entries[self.index] = Some((self.key, value));
        *self.len += 1;

        unsafe { OccupiedEntry::new(self.entries, self.index, self.build_hasher, self.len) }
    }
}
