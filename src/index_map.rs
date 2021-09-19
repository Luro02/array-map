use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash};

use crate::raw::{ArrayIndexTable, RawTable};
use crate::{utils, ArrayMapFacade, DefaultHashBuilder};

pub type IndexMap<K, V, const N: usize, B = DefaultHashBuilder> =
    ArrayMapFacade<K, V, ArrayIndexTable<(K, V), N>, B>;

impl<K, V, const N: usize, B: BuildHasher> IndexMap<K, V, N, B>
where
    K: Hash + Eq,
{
    /// Shortens the map, keeping the first `n` elements and removing the rest.
    ///
    /// If `n` is greater than `self.len` this does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::IndexMap;
    ///
    /// let mut map: IndexMap<&str, &str, 11> = IndexMap::new();
    ///
    /// map.insert("a", "b")?;
    /// map.insert("c", "d")?;
    /// map.insert("e", "f")?;
    ///
    /// map.truncate(1);
    /// assert_eq!(map.pop(), Some(("a", "b")));
    /// assert_eq!(map.pop(), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn truncate(&mut self, n: usize) {
        while self.len() > n && !self.is_empty() {
            self.pop();
        }
    }

    /// Removes the entry from the map and shifts all entries inserted after
    /// it to the left.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::IndexMap;
    ///
    /// let mut map: IndexMap<i32, &str, 7> = IndexMap::new();
    ///
    /// map.insert(1, "a")?;
    /// map.insert(2, "b")?;
    /// map.insert(3, "c")?;
    /// map.insert(4, "d")?;
    ///
    /// assert_eq!(map.shift_remove_entry(&1), Some((1, "a")));
    /// assert_eq!(map.shift_remove_entry(&1), None);
    /// assert_eq!(
    ///     Ok([(2, "b"), (3, "c"), (4, "d")]),
    ///     map.into_iter().try_collect()
    /// );
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    ///
    /// ### Complexity
    ///
    /// O(n)
    pub fn shift_remove_entry<Q: ?Sized>(&mut self, qkey: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = utils::make_hash::<K, Q, B>(self.build_hasher(), &qkey);

        unsafe {
            let ident = self.table.find(hash, |(k, _)| qkey.eq(k.borrow()))?;

            let entry = self
                .table
                .shift_remove(ident, utils::key_hasher(&self.build_hasher));
            Some(entry)
        }
    }

    /// Removes the last element from the map and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::IndexMap;
    ///
    /// let mut map: IndexMap<&str, &str, 11> = IndexMap::new();
    ///
    /// map.insert("a", "b")?;
    /// map.insert("c", "d")?;
    /// map.insert("e", "f")?;
    ///
    /// assert_eq!(map.pop(), Some(("e", "f")));
    /// assert_eq!(map.pop(), Some(("c", "d")));
    /// assert_eq!(map.pop(), Some(("a", "b")));
    /// assert_eq!(map.pop(), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn pop(&mut self) -> Option<(K, V)> {
        self.table.pop(utils::key_hasher(&self.build_hasher))
    }

    /// Returns the entry at the index. If the index is larger than or equal to
    /// the map's length, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::IndexMap;
    ///
    /// let mut map: IndexMap<usize, &str, 11> = IndexMap::new();
    /// map.insert(0, "a")?;
    /// map.insert(1, "b")?;
    /// map.insert(2, "c")?;
    /// map.insert(3, "d")?;
    ///
    /// assert_eq!(map.get_index_entry(0), Some((&0, &"a")));
    /// assert_eq!(map.get_index_entry(3), Some((&3, &"d")));
    /// assert_eq!(map.get_index_entry(4), None);
    ///
    /// map.pop();
    ///
    /// assert_eq!(map.get_index_entry(3), None);
    /// assert_eq!(map.get_index_entry(2), Some((&2, &"c")));
    ///
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn get_index_entry(&self, index: usize) -> Option<(&K, &V)> {
        let (key, value) = self.table.get_index(index)?;

        Some((key, value))
    }

    #[must_use]
    pub fn get_index_entry_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        let (key, value) = self.table.get_index_mut(index)?;

        Some((key, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ext::IteratorExt;
    use crate::index_map;
    use crate::raw::{ArrayTable, TableIndex};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pop() {
        let mut map: IndexMap<&str, &str, 7> = IndexMap::new();
        map.insert("a", "a").unwrap();
        map.insert("b", "b").unwrap();
        map.insert("c", "c").unwrap();
        map.insert("d", "d").unwrap();
        map.insert("e", "e").unwrap();
        map.insert("f", "f").unwrap();

        assert_eq!(map.pop(), Some(("f", "f")));
        assert_eq!(map.pop(), Some(("e", "e")));
        assert_eq!(map.pop(), Some(("d", "d")));
        assert_eq!(map.pop(), Some(("c", "c")));
        assert_eq!(map.pop(), Some(("b", "b")));
        assert_eq!(map.pop(), Some(("a", "a")));
        assert_eq!(map.pop(), None);
    }

    #[derive(PartialEq, Eq)]
    struct HasHash(u64, u64);

    impl core::hash::Hash for HasHash {
        fn hash<H>(&self, h: &mut H)
        where
            H: core::hash::Hasher,
        {
            h.write_u64(self.0);
        }
    }

    impl core::fmt::Debug for HasHash {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "HasHash({}, {})", self.0, self.1)
        }
    }

    #[derive(Default)]
    struct Hasher(u64);

    impl core::hash::Hasher for Hasher {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, _: &[u8]) {
            unimplemented!()
        }

        fn write_u64(&mut self, value: u64) {
            self.0 = value;
        }
    }

    #[test]
    fn test_fuzzer_failure_00() {
        let mut map: IndexMap<HasHash, usize, 5, _> = index_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(4, 0) => 1,
            HasHash(0, 0) => 0,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayIndexTable::from((
                // indices
                ArrayTable::from_array([
                    // HasHash(0, 0)
                    Some(unsafe { TableIndex::new(1) }),
                    None,
                    None,
                    None,
                    //
                    Some(unsafe { TableIndex::new(0) }),
                ]),
                // entries
                [(HasHash(4, 0), 1), (HasHash(0, 0), 0)]
                    .into_iter()
                    .try_collect()
                    .unwrap()
            ))
        );

        assert_eq!(map.get(&HasHash(4, 0)), Some(&1));
        assert_eq!(map.get(&HasHash(0, 0)), Some(&0));

        assert_eq!(map.remove(&HasHash(4, 0)), Some(1));

        assert_eq!(
            map.table,
            ArrayIndexTable::from((
                // indices
                ArrayTable::from_array([
                    // HasHash(0, 0)
                    Some(unsafe { TableIndex::new(0) }),
                    None,
                    None,
                    None,
                    //
                    None,
                ]),
                // entries
                [(HasHash(0, 0), 0)].into_iter().try_collect().unwrap()
            ))
        );

        assert_eq!(map.get(&HasHash(0, 0)), Some(&0));
    }

    #[test]
    fn test_fuzzer_failure_01() {
        let mut map: IndexMap<HasHash, usize, 5, _> = index_map! {
            @infer,
            @build_hasher => ::core::hash::BuildHasherDefault::<Hasher>::default(),
            HasHash(3, 0) => 0,
            HasHash(3, 0) => 1,
            HasHash(2, 0) => 2,
        }
        .unwrap();

        assert_eq!(
            map.table,
            ArrayIndexTable::from((
                // indices
                ArrayTable::from_array([
                    None,
                    None,
                    Some(unsafe { TableIndex::new(1) }),
                    Some(unsafe { TableIndex::new(0) }),
                    None,
                ]),
                // entries
                [(HasHash(3, 0), 1), (HasHash(2, 0), 2)]
                    .into_iter()
                    .try_collect()
                    .unwrap()
            ))
        );

        assert_eq!(map.get(&HasHash(3, 0)), Some(&1));
        assert_eq!(map.get(&HasHash(2, 0)), Some(&2));

        assert_eq!(
            map.shift_remove_entry(&HasHash(3, 0)),
            Some((HasHash(3, 0), 1))
        );

        assert_eq!(
            map.table,
            ArrayIndexTable::from((
                // indices
                ArrayTable::from_array([
                    None,
                    None,
                    Some(unsafe { TableIndex::new(0) }),
                    None,
                    //
                    None,
                ]),
                // entries
                [(HasHash(2, 0), 2)].into_iter().try_collect().unwrap()
            ))
        );

        assert_eq!(map.get(&HasHash(2, 0)), Some(&2));
    }
}
