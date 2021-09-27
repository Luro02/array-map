use core::hash::{BuildHasher, Hash};
use core::{fmt, mem};

use crate::raw::RawTable;
use crate::{utils, VacantEntry};

/// A view into an occupied entry in an `ArrayMap`. It is part of the [`Entry`]
/// enum.
///
/// [`Entry`]: crate::Entry
pub struct OccupiedEntry<'a, K, V, R: RawTable<(K, V)>, B: BuildHasher> {
    table: &'a mut R,
    ident: R::Ident,
    build_hasher: &'a B,
}

impl<'a, K: 'a, V, R: RawTable<(K, V)>, B: BuildHasher> OccupiedEntry<'a, K, V, R, B> {
    /// Constructs a new `OccupiedEntry`.
    ///
    /// # Safety
    ///
    /// The following invariants must hold:
    /// - `entries.len() == N` (should be guaranteed by the compiler)
    /// - `index < N` (index must not be out of bounds)
    /// - `entries[index].is_some()` (otherwise the entry would not be occupied)
    /// - `len > 0` and `len <= N`
    #[must_use]
    pub(crate) unsafe fn new(table: &'a mut R, ident: R::Ident, build_hasher: &'a B) -> Self {
        Self {
            table,
            ident,
            build_hasher,
        }
    }

    #[must_use]
    fn entry(&self) -> (&K, &V) {
        // SAFETY: self has exclusive access to the table, so self.ident is guranteed to
        //         be valid
        unsafe {
            let (key, value) = self.table.get_unchecked(self.ident.clone());
            (key, value)
        }
    }

    /// Returns a reference to the entry's key.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    ///
    /// let occupied_entry = map.entry("good")?.insert_entry("job");
    ///
    /// assert_eq!(occupied_entry.key(), &"good");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn key(&self) -> &K {
        self.entry().0
    }

    /// Returns a reference to the entry's value.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    ///
    /// let occupied_entry = map.entry("good")?.insert_entry("job");
    ///
    /// assert_eq!(occupied_entry.get(), &"job");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn get(&self) -> &V {
        self.entry().1
    }

    /// Returns a mutable reference to the entry's value.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    ///
    /// let mut occupied_entry = map.entry("good")?.insert_entry("job");
    ///
    /// assert_eq!(occupied_entry.get(), &"job");
    /// *occupied_entry.get_mut() = "friend";
    ///
    /// assert_eq!(occupied_entry.get(), &"friend");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn get_mut(&mut self) -> &mut V {
        // SAFETY: self has exclusive access to the table, so self.ident is guranteed to
        //         be valid
        unsafe { &mut self.table.get_unchecked_mut(self.ident.clone()).1 }
    }

    /// Replaces the existing value with the provided value and returns the old
    /// value.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    /// let mut occupied_entry = map.entry("good")?.insert_entry("job");
    ///
    /// assert_eq!(occupied_entry.get(), &"job");
    ///
    /// let old_value = occupied_entry.insert("friend");
    /// assert_eq!(old_value, "job");
    ///
    /// assert_eq!(occupied_entry.get(), &"friend");
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in
    /// the entry with a lifetime bound to the map itself.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    /// let mut occupied_entry = map.entry("good")?.insert_entry("job");
    ///
    /// let value: &mut &str = occupied_entry.into_mut();
    /// *value = "friend";
    ///
    /// assert_eq!(map.get("good"), Some(&"friend"));
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn into_mut(self) -> &'a mut V {
        // SAFETY: self has exclusive access to the table, so self.ident is guranteed to
        //         be valid
        let (_, value) = unsafe { R::get_unchecked_mut(self.table, self.ident) };
        value
    }
}

trait DoubleEndedIteratorExt: DoubleEndedIterator {
    fn rfind_map<B, F>(&mut self, f: F) -> Option<B>
    where
        F: FnMut(Self::Item) -> Option<B>,
    {
        self.filter_map(f).next_back()
    }
}

impl<D: DoubleEndedIterator> DoubleEndedIteratorExt for D {}

impl<'a, K: Hash + Eq, V, R: RawTable<(K, V)>, B: BuildHasher> OccupiedEntry<'a, K, V, R, B> {
    /// Removes the key value pair stored in the map for this entry and returns
    /// the value.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    /// map.try_insert("good", "job")?;
    ///
    /// assert_eq!(map.contains_key("good"), true);
    ///
    /// let mut occupied_entry = map.entry("good")?.insert_entry("job");
    /// assert_eq!(occupied_entry.remove(), "job");
    ///
    /// assert_eq!(map.contains_key("good"), false);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[allow(clippy::must_use_candidate)]
    pub fn remove(self) -> V {
        self.remove_entry().1
    }

    /// Removes the key value pair stored in the map for this entry and return
    /// the key value pair.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    /// map.try_insert("good", "job")?;
    ///
    /// assert_eq!(map.contains_key("good"), true);
    ///
    /// let mut occupied_entry = map.entry("good")?.insert_entry("job");
    /// assert_eq!(occupied_entry.remove_entry(), ("good", "job"));
    ///
    /// assert_eq!(map.contains_key("good"), false);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[allow(clippy::must_use_candidate)]
    pub fn remove_entry(self) -> (K, V) {
        let (vacant, value) = self.remove_entry_helper();

        (vacant.into_key(), value)
    }

    pub(crate) fn remove_entry_helper(self) -> (VacantEntry<'a, K, V, R, B>, V) {
        // SAFETY: invariants are guarenteed by the constructor
        let (key, value) = unsafe {
            self.table
                .remove(self.ident, utils::key_hasher(self.build_hasher))
        };

        let vacant_entry = unsafe { VacantEntry::new(self.table, key, self.build_hasher) };
        (vacant_entry, value)
    }
}

impl<'a, K, V, R: RawTable<(K, V)>, B> fmt::Debug for OccupiedEntry<'a, K, V, R, B>
where
    K: fmt::Debug,
    V: fmt::Debug,
    B: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(OccupiedEntry))
            .field("key", self.key())
            .field("value", self.get())
            .finish()
    }
}
#[cfg(test)]
mod tests {
    use core::hash::{BuildHasherDefault, Hasher};

    use super::*;
    use crate::raw::{ArrayTable, TableIndex};
    use pretty_assertions::assert_eq;

    // This hasher will always cause a collision
    #[derive(Default)]
    pub struct CollisionHasher;

    impl Hasher for CollisionHasher {
        fn write(&mut self, _: &[u8]) {}

        fn finish(&self) -> u64 {
            0
        }
    }

    #[test]
    fn test_occupied() {
        let mut table = ArrayTable::from_array([
            Some((0, "a")),
            Some((1, "b")),
            Some((2, "c")),
            Some((3, "d")),
            None,
            None,
        ]);

        let ident = unsafe { TableIndex::new(0) };

        let build_hasher = BuildHasherDefault::<CollisionHasher>::default();
        let mut occupied = unsafe { OccupiedEntry::new(&mut table, ident, &build_hasher) };

        assert_eq!(occupied.key(), &0);
        assert_eq!(occupied.get(), &"a");
        assert_eq!(occupied.get_mut(), &mut "a");
    }
}
