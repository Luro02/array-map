use core::fmt;
use core::hash::BuildHasher;

use crate::utils::{MutateOnce, invariant, unwrap_unchecked};
use crate::OccupiedEntry;

/// A view into a vacant entry in an `ArrayMap`. It is part of the [`Entry`]
/// enum.
///
/// [`Entry`]: crate::Entry
pub struct VacantEntry<'a, K: 'a, V: 'a, B, const N: usize> {
    key: K,
    entries: &'a mut [Option<(K, V)>; N],
    index: usize,
    build_hasher: &'a B,
    len: MutateOnce<'a, usize>,
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
            len: MutateOnce::new(len),
        }
    }

    /// Returns a reference to the entry's key.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::{ArrayMap, Entry};
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    ///
    /// let vacant_entry = map.entry("good")?.remove_entry();
    ///
    /// assert_eq!(vacant_entry.key(), &"good");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Inserts the entry’s key and the given value into the map, and returns a
    /// mutable reference to the value.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::{ArrayMap, Entry};
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    ///
    /// let vacant_entry = map.entry("good")?.remove_entry();
    ///
    /// assert_eq!(vacant_entry.insert("morning"), &mut "morning");
    /// assert_eq!(map.get("good"), Some(&"morning"));
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn insert(mut self, value: V) -> &'a mut V {
        // SAFETY: invariants are guranteed by the constructor
        unsafe {
            debug_assert!(self.entries[self.index].is_none());
            *self.entries.get_unchecked_mut(self.index) = Some((self.key, value));
            self.len.mutate(|len| *len += 1);

            &mut unwrap_unchecked(self.entries.get_unchecked_mut(self.index).as_mut()).1
        }
    }

    /// Takes ownership of the key, leaving the entry vacant.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::{ArrayMap, Entry};
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    ///
    /// let vacant_entry = map.entry("good")?.remove_entry();
    ///
    /// let key = vacant_entry.into_key();
    /// assert_eq!(key, "good");
    /// assert_eq!(map.contains_key(&key), false);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn into_key(self) -> K {
        self.key
    }

    /// Inserts the value, returning an `OccupiedEntry`.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::{ArrayMap, Entry};
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    ///
    /// let vacant_entry = map.entry("good")?.remove_entry();
    ///
    /// let occupied_entry = vacant_entry.insert_entry("evening");
    /// assert_eq!(occupied_entry.key(), &"good");
    /// assert_eq!(occupied_entry.get(), &"evening");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn insert_entry(mut self, value: V) -> OccupiedEntry<'a, K, V, B, N> {
        // SAFETY: invariants are guranteed by the constructor
        unsafe {
            debug_assert!(self.entries[self.index].is_none());
            *self.entries.get_unchecked_mut(self.index) = Some((self.key, value));
            self.len.mutate(|len| *len += 1);

            OccupiedEntry::new(self.entries, self.index, self.build_hasher, self.len.into_mut())
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
