use core::hash::{BuildHasher, Hash};
use core::mem;

use crate::utils::{self, invariant, unwrap_unchecked, IterEntries, Slot};

/// A view into an occupied entry in an `ArrayMap`. It is part of the [`Entry`]
/// enum.
///
/// [`Entry`]: crate::Entry
#[derive(Debug)]
pub struct OccupiedEntry<'a, K: 'a, V: 'a, B, const N: usize> {
    entries: &'a mut [Option<(K, V)>; N],
    index: usize,
    build_hasher: &'a B,
    len: &'a mut usize,
}

impl<'a, K, V, B: BuildHasher, const N: usize> OccupiedEntry<'a, K, V, B, N> {
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
    pub(crate) unsafe fn new(
        entries: &'a mut [Option<(K, V)>; N],
        index: usize,
        build_hasher: &'a B,
        len: &'a mut usize,
    ) -> Self {
        invariant(entries.len() == N);
        invariant(index < N);
        invariant(entries[index].is_some());
        invariant(*len > 0 && *len <= N);

        Self {
            entries,
            index,
            build_hasher,
            len,
        }
    }

    #[must_use]
    fn entry(&self) -> (&K, &V) {
        // SAFETY: invariants are guranteed by the constructor
        unsafe {
            debug_assert!(self.index < self.entries.len());
            debug_assert!(self.entries[self.index].is_some());

            let (key, value) = unwrap_unchecked(self.entries.get_unchecked(self.index).as_ref());
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
        // SAFETY: invariants are guranteed by the constructor
        unsafe {
            debug_assert!(self.index < self.entries.len());
            debug_assert!(self.entries[self.index].is_some());

            &mut unwrap_unchecked(self.entries.get_unchecked_mut(self.index).as_mut()).1
        }
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
        // SAFETY: invariants are guranteed by the constructor
        unsafe {
            debug_assert!(self.index < self.entries.len());
            debug_assert!(self.entries[self.index].is_some());

            &mut unwrap_unchecked(self.entries.get_unchecked_mut(self.index).as_mut()).1
        }
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

impl<'a, K: Hash + Eq, V, B: BuildHasher, const N: usize> OccupiedEntry<'a, K, V, B, N> {
    fn find_with_hash(&self, key: &K) -> Option<usize> {
        let hash = utils::make_hash::<K, K, B>(self.build_hasher, key);

        IterEntries::new(hash, self.entries, utils::key_hasher(self.build_hasher)).rfind_map(
            |slot| {
                if let Slot::Collision { index, .. } = slot {
                    Some(index)
                } else {
                    None
                }
            },
        )
    }

    /// Removes the key value pair stored in the map for this entry and returns
    /// the value.
    ///
    /// # Example
    ///
    /// ```
    /// use array_map::ArrayMap;
    ///
    /// let mut map: ArrayMap<&str, &str, 11> = ArrayMap::new();
    /// map.insert("good", "job")?;
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
    /// map.insert("good", "job")?;
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
        debug_assert!(*self.len > 0);
        *self.len -= 1;

        let mut remove_index = self.index;
        if let Some(collision) = self.find_with_hash(self.key()) {
            self.entries.swap(collision, self.index);

            remove_index = collision;
        }

        // SAFETY: invariants are guarenteed by the constructor
        unsafe {
            debug_assert!(remove_index < self.entries.len());
            unwrap_unchecked(self.entries.get_unchecked_mut(remove_index).take())
        }
    }
}

#[cfg(test)]
mod tests {
    use core::hash::{BuildHasherDefault, Hasher};

    use super::*;
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
        let mut entries = [
            Some((0, "a")),
            Some((1, "b")),
            Some((2, "c")),
            Some((3, "d")),
            None,
            None,
        ];

        let build_hasher = BuildHasherDefault::<CollisionHasher>::default();
        let mut len = 4;
        let mut occupied = unsafe { OccupiedEntry::new(&mut entries, 0, &build_hasher, &mut len) };

        assert_eq!(occupied.key(), &0);
        assert_eq!(occupied.get(), &"a");
        assert_eq!(occupied.get_mut(), &mut "a");
    }
}
