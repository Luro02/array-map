use core::fmt;
use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;

use crate::raw::RawTable;
use crate::{invariant, unreachable_unchecked, utils, OccupiedEntry};

/// A view into a vacant entry in an `ArrayMap`. It is part of the [`Entry`]
/// enum.
///
/// [`Entry`]: crate::Entry
pub struct VacantEntry<'a, K, V, R: RawTable<(K, V)>, B: BuildHasher> {
    key: K,
    table: &'a mut R,
    build_hasher: &'a B,
    _p: PhantomData<&'a (K, V)>,
}

impl<'a, K, V, R: RawTable<(K, V)>, B: BuildHasher> VacantEntry<'a, K, V, R, B> {
    /// Constructs a new `VacantEntry`.
    ///
    /// # Safety
    ///
    /// There must be at least one vacant space in the table.
    #[must_use]
    pub(crate) unsafe fn new(table: &'a mut R, key: K, build_hasher: &'a B) -> Self {
        invariant!(table.len() < table.capacity());
        Self {
            key,
            table,
            build_hasher,
            _p: PhantomData,
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

impl<'a, K: Hash, V, R: RawTable<(K, V)>, B: BuildHasher> VacantEntry<'a, K, V, R, B> {
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
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, R, B> {
        let ident = {
            let hash = utils::make_hash::<K, K, B>(self.build_hasher, self.key());
            let result = self.table.try_insert(
                hash,
                (self.key, value),
                utils::key_hasher(self.build_hasher),
            );

            match result {
                Ok(ident) => ident,
                Err(_) => unreachable_unchecked!("there must be free space for a vacant entry!"),
            }
        };

        unsafe { OccupiedEntry::new(self.table, ident, self.build_hasher) }
    }
}

impl<'a, K, V, R, B> fmt::Debug for VacantEntry<'a, K, V, R, B>
where
    K: fmt::Debug,
    V: fmt::Debug,
    B: BuildHasher,
    R: RawTable<(K, V)>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(stringify!(VacantEntry))
            .field(self.key())
            .finish()
    }
}
