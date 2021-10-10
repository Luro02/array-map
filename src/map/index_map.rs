use core::borrow::Borrow;
use core::fmt;
use core::hash::{BuildHasher, Hash};
use core::ops::{Bound, Index, RangeBounds};

use crate::map::iter::DrainRange;
use crate::map::{ArrayMapFacade, DefaultHashBuilder};
use crate::raw::{ArrayIndexTable, RawTable, TableIndex};
use crate::utils::{self, UnwrapExpectExt};

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
    /// map.try_insert("a", "b")?;
    /// map.try_insert("c", "d")?;
    /// map.try_insert("e", "f")?;
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
    /// map.try_insert(1, "a")?;
    /// map.try_insert(2, "b")?;
    /// map.try_insert(3, "c")?;
    /// map.try_insert(4, "d")?;
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
        let hash = utils::make_hash::<K, Q, B>(self.build_hasher(), qkey);

        unsafe {
            let ident = self.table.find(hash, |(k, _)| qkey.eq(k.borrow()))?;

            let entry = self
                .table
                .shift_remove(ident, utils::key_hasher(&self.build_hasher));
            Some(entry)
        }
    }

    /// Removes the entry at the index and shifts down all entries after it.
    ///
    /// This preserves the order of the entries.
    ///
    /// `None` is returned if the index is larger than the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{index_map, IndexMap};
    ///
    /// let mut map: IndexMap<&str, &str, 11> = index_map! {
    ///     @infer,
    ///     "apple" => "apfel",
    ///     "tree" => "baum",
    ///     "cake" => "kuchen",
    ///     "food" => "essen",
    /// }?;
    ///
    /// assert_eq!(map.shift_remove_index(0), Some(("apple", "apfel")));
    ///
    /// assert_eq!(
    ///     map.iter().try_collect::<[Option<_>; 3]>(),
    ///     Ok([
    ///         Some((&"tree", &"baum")),
    ///         Some((&"cake", &"kuchen")),
    ///         Some((&"food", &"essen")),
    ///     ])
    /// );
    ///
    /// assert_eq!(map.shift_remove_index(5), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    ///
    /// # Complexity
    ///
    /// O(n)
    pub fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        // SAFETY: it is checked that the index is less than `N`
        //         (implied by `self.len()`)
        let index = unsafe {
            if index >= self.len() {
                return None;
            }

            TableIndex::new(index)
        };

        // SAFETY: ident_from_index should always return a valid ident
        unsafe {
            let ident = self
                .table
                .ident_from_index(index, utils::key_hasher(&self.build_hasher))?;
            Some(
                self.table
                    .shift_remove(ident, utils::key_hasher(&self.build_hasher)),
            )
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
    /// map.try_insert("a", "b")?;
    /// map.try_insert("c", "d")?;
    /// map.try_insert("e", "f")?;
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
    /// map.try_insert(0, "a")?;
    /// map.try_insert(1, "b")?;
    /// map.try_insert(2, "c")?;
    /// map.try_insert(3, "d")?;
    ///
    /// assert_eq!(map.get_entry_at(0), Some((&0, &"a")));
    /// assert_eq!(map.get_entry_at(3), Some((&3, &"d")));
    /// assert_eq!(map.get_entry_at(4), None);
    ///
    /// map.pop();
    ///
    /// assert_eq!(map.get_entry_at(3), None);
    /// assert_eq!(map.get_entry_at(2), Some((&2, &"c")));
    ///
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn get_entry_at(&self, index: usize) -> Option<(&K, &V)> {
        let (key, value) = self.table.get_index(index)?;

        Some((key, value))
    }

    /// Returns the entry at the index. If the index is larger than or equal to
    /// the map's length, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{index_map, IndexMap};
    ///
    /// let mut map: IndexMap<&str, &str, 11> = index_map! {
    ///     @infer,
    ///     "apple" => "apfel",
    ///     "tree" => "baum",
    ///     "cake" => "kuchen",
    ///     "food" => "essen",
    /// }?;
    ///
    /// if let Some(entry) = map.get_entry_at_mut(1) {
    ///     assert_eq!(entry.0, &"tree");
    ///     assert_eq!(entry.1, &mut "baum");
    ///     *entry.1 = "bäumchen";
    /// }
    ///
    /// assert_eq!(map.get_entry_at_mut(1), Some((&"tree", &mut "bäumchen")));
    ///
    /// assert_eq!(map.get_entry_at_mut(5), None);
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    #[must_use]
    pub fn get_entry_at_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        let (key, value) = self.table.get_index_mut(index)?;

        Some((key, value))
    }

    /// Swaps the position of the two entries `a` and `b`.
    ///
    /// # Panics
    ///
    /// If either `a` or `b` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{index_map, IndexMap};
    ///
    /// let mut map: IndexMap<&str, &str, 11> = index_map! {
    ///     @infer,
    ///     "apple" => "apfel",
    ///     "tree" => "baum",
    ///     "cake" => "kuchen",
    ///     "food" => "essen",
    /// }?;
    ///
    /// assert_eq!(map.get_entry_at(1), Some((&"tree", &"baum")));
    /// assert_eq!(map.get_entry_at(2), Some((&"cake", &"kuchen")));
    ///
    /// map.swap_indices(1, 2);
    ///
    /// assert_eq!(map.get_entry_at(1), Some((&"cake", &"kuchen")));
    /// assert_eq!(map.get_entry_at(2), Some((&"tree", &"baum")));
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn swap_indices(&mut self, a: usize, b: usize) {
        self.try_swap_indices(a, b)
            .expect("failed to swap indices, because either a or b is not in bounds");
    }

    /// Swaps the position of the two entries `a` and `b`.
    ///
    /// # Errors
    ///
    /// If either `a` or `b` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::{index_map, IndexMap};
    ///
    /// let mut map: IndexMap<&str, &str, 11> = index_map! {
    ///     @infer,
    ///     "apple" => "apfel",
    ///     "tree" => "baum",
    ///     "cake" => "kuchen",
    ///     "food" => "essen",
    /// }?;
    ///
    /// assert_eq!(map.get_entry_at(1), Some((&"tree", &"baum")));
    /// assert_eq!(map.get_entry_at(2), Some((&"cake", &"kuchen")));
    ///
    /// assert_eq!(map.try_swap_indices(1, 2), Ok(()));
    ///
    /// assert_eq!(map.get_entry_at(1), Some((&"cake", &"kuchen")));
    /// assert_eq!(map.get_entry_at(2), Some((&"tree", &"baum")));
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    pub fn try_swap_indices(&mut self, a: usize, b: usize) -> Result<(), IndexOutOfBoundsError> {
        if a >= self.table.len() {
            return Err(IndexOutOfBoundsError(a));
        }

        if b >= self.table.len() {
            return Err(IndexOutOfBoundsError(b));
        }

        let a = unsafe { TableIndex::new(a) };
        let b = unsafe { TableIndex::new(b) };

        // SAFETY: it has been verified that a is in bounds
        let a = unsafe {
            self.table
                .ident_from_index(a, utils::key_hasher(&self.build_hasher))
                .expect_unchecked("failed to get ident for index a")
        };
        // SAFETY: it has been verified that b is in bounds
        let b = unsafe {
            self.table
                .ident_from_index(b, utils::key_hasher(&self.build_hasher))
                .expect_unchecked("failed to get ident for index b")
        };

        // SAFETY: remove has not been called, so the idents are still valid
        unsafe {
            self.table.swap(a, b);
        }
        Ok(())
    }

    /// Removes the entry at the index or returns `None` if the index is out of
    /// bounds by swapping it's position with the last element and then removing
    /// it.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{index_map, IndexMap};
    ///
    /// let mut map: IndexMap<&str, &str, 11> = index_map! {
    ///     @infer,
    ///     "apple" => "apfel",
    ///     "tree" => "baum",
    ///     "cake" => "kuchen",
    ///     "food" => "essen",
    /// }?;
    ///
    /// assert_eq!(map.swap_remove_index(1), Some(("tree", "baum")));
    ///
    /// assert_eq!(
    ///     map.iter().try_collect::<[Option<_>; 3]>(),
    ///     Ok([
    ///         Some((&"apple", &"apfel")),
    ///         Some((&"food", &"essen")),
    ///         Some((&"cake", &"kuchen")),
    ///     ])
    /// );
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    ///
    /// # Complexity
    ///
    /// O(1)
    pub fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        if self.len() <= index || self.is_empty() {
            return None;
        }

        self.try_swap_indices(index, self.len() - 1).ok()?;

        self.pop()
    }

    /// Clears the map in the given index range, returning all entries in that
    /// range as an iterator.
    ///
    /// This shifts down all the entries following the drained range to fill the
    /// gap.
    ///
    /// # Panics
    ///
    /// If the starting point is greater than the end point of if the end point
    /// is greater than the length of the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use array_map::ext::IteratorExt;
    /// use array_map::{index_map, IndexMap};
    ///
    /// let mut map: IndexMap<&str, &str, 11> = index_map! {
    ///     @infer,
    ///     "apple" => "apfel",
    ///     "tree" => "baum",
    ///     "cake" => "kuchen",
    ///     "food" => "essen",
    /// }?;
    ///
    /// assert_eq!(
    ///     map.drain_range(1..3).try_collect::<[Option<_>; 4]>(),
    ///     Ok([
    ///         //
    ///         Some(("tree", "baum")),
    ///         Some(("cake", "kuchen")),
    ///         None,
    ///         None
    ///     ])
    /// );
    ///
    /// assert_eq!(
    ///     map.iter().try_collect::<[Option<_>; 4]>(),
    ///     Ok([
    ///         Some((&"apple", &"apfel")),
    ///         Some((&"food", &"essen")),
    ///         None,
    ///         None
    ///     ])
    /// );
    /// # Ok::<_, array_map::CapacityError>(())
    /// ```
    ///
    /// # Complexity
    ///
    /// O(n^2)
    pub fn drain_range<R: RangeBounds<usize>>(&mut self, range: R) -> DrainRange<'_, K, V, B, N> {
        let start = {
            match range.start_bound() {
                Bound::Included(index) => *index,
                Bound::Excluded(index) => *index + 1,
                Bound::Unbounded => 0,
            }
        };

        let end = {
            match range.end_bound() {
                Bound::Included(index) => *index + 1,
                Bound::Excluded(index) => *index,
                Bound::Unbounded => self.len(),
            }
        };

        if start > end {
            panic!("start of range is greater than the end");
        }

        if end > self.len() {
            panic!("end of range is out of bounds");
        }

        DrainRange::new(self, start..end)
    }
}

impl<K, V, const N: usize, B: BuildHasher> Index<usize> for IndexMap<K, V, N, B>
where
    K: Hash + Eq,
{
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        self.get_entry_at(index)
            .map(|entry| entry.1)
            .expect("index is out of bounds")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexOutOfBoundsError(usize);

impl fmt::Display for IndexOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "index `{}` is out of bounds", self.0)
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
        map.try_insert("a", "a").unwrap();
        map.try_insert("b", "b").unwrap();
        map.try_insert("c", "c").unwrap();
        map.try_insert("d", "d").unwrap();
        map.try_insert("e", "e").unwrap();
        map.try_insert("f", "f").unwrap();

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

    #[test]
    fn test_drain_range() {
        let mut map: IndexMap<&str, &str, 11> = index_map! {
            @infer,
            "apple" => "apfel",
            "tree" => "baum",
            "cake" => "kuchen",
            "food" => "essen",
        }
        .unwrap();

        assert_eq!(
            map.iter().try_collect::<[Option<_>; 4]>(),
            Ok([
                Some((&"apple", &"apfel")),
                Some((&"tree", &"baum")),
                Some((&"cake", &"kuchen")),
                Some((&"food", &"essen")),
            ])
        );

        assert_eq!(map.get_entry_at(0), Some((&"apple", &"apfel")));
        assert_eq!(map.get_entry_at(1), Some((&"tree", &"baum")));
        assert_eq!(map.get_entry_at(2), Some((&"cake", &"kuchen")));
        assert_eq!(map.get_entry_at(3), Some((&"food", &"essen")));
        assert_eq!(map.get_entry_at(4), None);

        assert_eq!(
            map.drain_range(1..3).try_collect::<[Option<_>; 4]>(),
            Ok([
                //
                Some(("tree", "baum")),
                Some(("cake", "kuchen")),
                None,
                None
            ])
        );

        assert_eq!(
            map.iter().try_collect::<[Option<_>; 4]>(),
            Ok([
                Some((&"apple", &"apfel")),
                Some((&"food", &"essen")),
                None,
                None
            ])
        );
    }

    #[test]
    fn test_drain_range_reverse_iter() {
        let mut map: IndexMap<&str, &str, 11> = index_map! {
            @infer,
            "apple" => "apfel",
            "tree" => "baum", // 1
            "kitchen" => "küche",
            "house" => "haus",
            "chair" => "stuhl",
            "dog" => "hund",
            "cat" => "katze",
            "mouse" => "maus", // 7
            "cake" => "kuchen",
            "food" => "essen",
        }
        .unwrap();

        assert_eq!(
            map.drain_range(1..=7).rev().try_collect::<[Option<_>; 8]>(),
            Ok([
                //
                Some(("mouse", "maus")),
                Some(("cat", "katze")),
                Some(("dog", "hund")),
                Some(("chair", "stuhl")),
                Some(("house", "haus")),
                Some(("kitchen", "küche")),
                Some(("tree", "baum")),
                None,
            ])
        );

        assert_eq!(
            map.iter().try_collect::<[Option<_>; 4]>(),
            Ok([
                Some((&"apple", &"apfel")),
                Some((&"cake", &"kuchen")),
                Some((&"food", &"essen")),
                None
            ])
        );
    }

    #[test]
    fn test_drain_range_empty() {
        let mut map: IndexMap<&str, &str, 11> = index_map! {
            @infer,
            "apple" => "apfel",
            "cake" => "kuchen",
            "food" => "essen",
        }
        .unwrap();

        assert_eq!(
            map.drain_range(1..1).try_collect::<[Option<_>; 4]>(),
            Ok([None, None, None, None])
        );
        assert_eq!(
            map.drain_range(0..0).try_collect::<[Option<_>; 4]>(),
            Ok([None, None, None, None])
        );
        assert_eq!(
            map.drain_range(1..1).rev().try_collect::<[Option<_>; 4]>(),
            Ok([None, None, None, None])
        );
        assert_eq!(
            map.drain_range(0..0).rev().try_collect::<[Option<_>; 4]>(),
            Ok([None, None, None, None])
        );
    }
}
