use core::fmt;
use core::hash::BuildHasher;

use crate::utils::{invariant, unwrap_unchecked};
use crate::OccupiedEntry;

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
        // SAFETY: invariants are guranteed by the constructor
        unsafe {
            debug_assert!(self.entries[self.index].is_none());
            *self.entries.get_unchecked_mut(self.index) = Some((self.key, value));
            *self.len += 1;

            &mut unwrap_unchecked(self.entries.get_unchecked_mut(self.index).as_mut()).1
        }
    }

    #[must_use]
    pub fn into_key(self) -> K {
        self.key
    }

    #[must_use]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, B, N> {
        // SAFETY: invariants are guranteed by the constructor
        unsafe {
            debug_assert!(self.entries[self.index].is_none());
            *self.entries.get_unchecked_mut(self.index) = Some((self.key, value));
            *self.len += 1;

            OccupiedEntry::new(self.entries, self.index, self.build_hasher, self.len)
        }
    }
}

impl<'a, K, V, B, const N: usize> fmt::Debug for VacantEntry<'a, K, V, B, N>
where
    K: fmt::Debug,
    V: fmt::Debug,
    B: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(stringify!(VacantEntry))
            .field(self.key())
            .finish()
    }
}
