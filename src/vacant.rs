use core::fmt;
use core::hash::{BuildHasher, Hash};

use crate::raw::{ArrayTable, RawTable};
use crate::{invariant, unreachable_unchecked, utils, OccupiedEntry};

/// A view into a vacant entry in an `ArrayMap`. It is part of the [`Entry`]
/// enum.
///
/// [`Entry`]: crate::Entry
pub struct VacantEntry<'a, K, V, B, const N: usize> {
    key: K,
    table: &'a mut ArrayTable<(K, V), N>,
    build_hasher: &'a B,
}

impl<'a, K, V, B: BuildHasher, const N: usize> VacantEntry<'a, K, V, B, N> {
    /// Constructs a new `VacantEntry`.
    ///
    /// # Safety
    ///
    /// There must be at least one vacant space in the table.
    #[must_use]
    pub(crate) unsafe fn new(
        table: &'a mut ArrayTable<(K, V), N>,
        key: K,
        build_hasher: &'a B,
    ) -> Self {
        invariant!(table.len() < table.capacity());
        Self {
            key,
            table,
            build_hasher,
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
}

impl<'a, K: Hash, V, B: BuildHasher, const N: usize> VacantEntry<'a, K, V, B, N> {
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
    pub fn insert(self, value: V) -> &'a mut V {
        self.insert_entry(value).into_mut()
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
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, B, N> {
        let ident = unsafe {
            let hash = utils::make_hash::<K, K, B>(self.build_hasher, self.key());
            let result = self.table.try_insert(
                hash,
                (self.key, value),
                utils::key_hasher(self.build_hasher),
            );

            match result {
                Ok(ident) => ident,
                // TODO: this relies on the assumption that insert will only error if there is not
                // enough space!
                Err(_) => unreachable_unchecked!("there must be free space for a vacant entry!"),
            }
        };

        // TODO: merge this with the other unsafe block
        unsafe { OccupiedEntry::new(self.table, ident, self.build_hasher) }
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
