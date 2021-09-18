use core::{array, iter};

use crate::errors::{CapacityError, UnavailableMutError};
use crate::ext::IteratorExt;
use crate::raw::{ArrayTable, RawTable, RawTableIter, TableIndex};
use crate::utils::{ArrayExt, UnwrapExpectExt};

use super::{ArrayVec, FlatIter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndexTable<T, const N: usize> {
    /// Mapping from the entry hash to its index.
    indices: ArrayTable<TableIndex<N>, N>,
    /// Entries in their order.
    entries: ArrayVec<T, N>,
}

impl<T, const N: usize> IndexTable<T, N> {
    /// Removes an entry from the table, preserving the insertion order by
    /// shifting all the following elements to the left.
    pub unsafe fn shift_remove(&mut self, ident: TableIndex<N>, hasher: impl Fn(&T) -> u64) -> T {
        let ident = self.indices.remove(ident, |ident| unsafe {
            hasher(self.entries.get_unchecked(*ident))
        });

        self.entries.remove_unchecked(ident)
    }

    /// Removes the last entry in the table if it is not empty.
    pub fn pop(&mut self, hasher: impl Fn(&T) -> u64) -> Option<T> {
        let entry = self.entries.pop()?;
        // NOTE: an entry has been removed => len = index that the last element had
        let index = unsafe { TableIndex::new(self.entries.len()) };
        let hash = hasher(&entry);

        unsafe {
            // this is the ident pointing to the position where the entry is in the indices
            // table
            let ident = self.indices.find(hash, |e| index.eq(e))?;
            self.indices.remove(ident, |idx| {
                // the table might hash the removed entry:
                if *idx == ident {
                    hash
                } else {
                    hasher(self.entries.get_unchecked(*idx))
                }
            })
        };

        Some(entry)
    }
}

impl<T, const N: usize> RawTable<T> for IndexTable<T, N> {
    type DrainIter = FlatIter<T, N>;
    type Ident = TableIndex<N>;
    type InsertError = CapacityError;
    type RawIter = iter::Flatten<array::IntoIter<Option<Self::Ident>, N>>;

    fn find(&self, hash: u64, mut eq: impl FnMut(&T) -> bool) -> Option<Self::Ident> {
        self.indices.find(hash, |index| unsafe {
            let value = self.entries.get_unchecked(*index);
            eq(value)
        })
    }

    unsafe fn get_unchecked(&self, ident: Self::Ident) -> &T {
        let index = self.indices.get_unchecked(ident);
        let value = self.entries.get_unchecked(*index);
        value
    }

    unsafe fn get_unchecked_mut(&mut self, ident: Self::Ident) -> &mut T {
        let index = self.indices.get_unchecked(ident);
        let value = self.entries.get_unchecked_mut(*index);
        value
    }

    unsafe fn erase(&mut self, ident: Self::Ident) -> T {
        let ident = self.indices.erase(ident);

        self.entries.swap_with_last(ident);
        self.entries
            .pop()
            .expect_unchecked("ident must be valid, table can not be empty")
    }

    fn try_insert(
        &mut self,
        hash: u64,
        value: T,
        hasher: impl Fn(&T) -> u64,
    ) -> Result<Self::Ident, Self::InsertError> {
        let index = self.entries.push(value)?;
        let ident = self.indices.try_insert(hash, index, |ident| unsafe {
            hasher(self.entries.get_unchecked(*ident))
        })?;
        Ok(ident)
    }

    fn iter_idents(&self) -> Self::RawIter {
        let values: [Option<TableIndex<N>>; N] =
            <ArrayTable<TableIndex<N>, N> as RawTableIter<TableIndex<N>>>::iter(&self.indices)
                .copied()
                .try_collect()
                // TODO: remove this panic and test that this works correctly
                .unwrap();

        values.into_iter().flatten()
    }

    /// # Note
    ///
    /// This does not preserve the order of the removed value.
    /// Use `shift_remove` if the order should be preserved!
    unsafe fn remove(&mut self, ident: Self::Ident, hasher: impl Fn(&T) -> u64) -> T {
        let ident = self.indices.remove(ident, |ident| unsafe {
            hasher(self.entries.get_unchecked(*ident))
        });

        self.entries.swap_with_last(ident);
        self.entries
            .pop()
            .expect_unchecked("table can not be empty")
    }

    fn drain(&mut self) -> Self::DrainIter {
        self.indices.clear();

        self.entries.drain()
    }

    fn capacity(&self) -> usize {
        N
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    // TODO: does this even work?
    fn get_each_mut<const M: usize>(
        &mut self,
        hashes: [u64; M],
        mut eq: impl FnMut(usize, &T) -> bool,
    ) -> [Result<&mut T, UnavailableMutError>; M] {
        let indices = hashes.enumerate().map(|(i, hash)| {
            self.indices
                .get(hash, |ident| unsafe {
                    let value = self.entries.get_unchecked(*ident);
                    eq(i, value)
                })
                .copied()
        });

        self.entries.get_each_mut_option(indices)
    }
}

impl<T, const N: usize> IntoIterator for IndexTable<T, N> {
    // TODO: custom iterator
    type IntoIter = iter::Flatten<array::IntoIter<Option<T>, N>>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<T, const N: usize> Default for IndexTable<T, N> {
    fn default() -> Self {
        Self {
            indices: ArrayTable::default(),
            entries: ArrayVec::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_default() {
        let table: IndexTable<&str, 5> = IndexTable::default();
        assert_eq!(table.is_empty(), true);
        assert_eq!(table.capacity(), 5);
    }

    #[test]
    fn test_into_iter() {
        const N: usize = 5;
        let mut table: IndexTable<u64, N> = IndexTable::default();

        for i in (0..N as u64).rev() {
            table.try_insert((N as u64) - i, i, |x| *x).unwrap();
        }

        let mut iter = table.into_iter();

        for i in (0..=4).rev() {
            assert_eq!(iter.next(), Some(i));
        }

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter_order_shift_remove() {
        const N: usize = 5;
        let mut table: IndexTable<Value, N> = IndexTable::default();

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Value {
            Foo,
            Bar,
            Avc,
            Bvc,
            Baz,
        }

        impl Value {
            #[must_use]
            fn hasher(&self) -> u64 {
                match self {
                    Self::Foo => 0,
                    Self::Bar => 1,
                    Self::Avc => 2,
                    Self::Bvc => 3,
                    Self::Baz => 4,
                }
            }
        }

        let insertion_order = [Value::Foo, Value::Baz, Value::Bar, Value::Avc, Value::Bvc];

        for value in insertion_order {
            table
                .try_insert(Value::hasher(&value), value, Value::hasher)
                .unwrap();
        }

        let mut iter = table.into_iter();

        assert_eq!(iter.next(), Some(Value::Foo));
        assert_eq!(iter.next(), Some(Value::Baz));
        assert_eq!(iter.next(), Some(Value::Bar));
        assert_eq!(iter.next(), Some(Value::Avc));
        assert_eq!(iter.next(), Some(Value::Bvc));
        assert_eq!(iter.next(), None);

        let mut table: IndexTable<Value, N> = IndexTable::default();

        for value in insertion_order {
            table
                .try_insert(Value::hasher(&value), value, Value::hasher)
                .unwrap();
        }

        // remove those values from the table:
        for value in [Value::Bar, Value::Foo] {
            let hash = Value::hasher(&value);
            let ident = table.find(hash, |v| v.eq(&value)).unwrap();
            assert_eq!(unsafe { table.shift_remove(ident, Value::hasher) }, value);
        }

        // check that the order is still correct:
        let mut iter = table.into_iter();

        assert_eq!(iter.next(), Some(Value::Baz));
        assert_eq!(iter.next(), Some(Value::Avc));
        assert_eq!(iter.next(), Some(Value::Bvc));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_strings_insert_get() {
        const N: usize = 5;
        let mut table: IndexTable<&'static str, N> = IndexTable::default();

        fn hasher(value: &&'static str) -> u64 {
            match *value {
                "foo" => 1,
                "bar" => 2,
                "avc" => 3,
                "baz" => 4,
                _ => 0,
            }
        }

        let values = ["avc", "foo", "baz", "bar"];

        let idents = values.enumerate().map(|(index, value)| {
            let ident = table.try_insert(hasher(&value), value, hasher).unwrap();
            (index, ident, value)
        });

        assert_eq!(
            table,
            IndexTable {
                indices: ArrayTable::from_array([
                    None,
                    // "foo"
                    Some(unsafe { TableIndex::new(1) }),
                    // "bar"
                    Some(unsafe { TableIndex::new(3) }),
                    // "avc"
                    Some(unsafe { TableIndex::new(0) }),
                    // "baz"
                    Some(unsafe { TableIndex::new(2) }),
                ]),
                entries: ["avc", "foo", "baz", "bar"]
                    .into_iter()
                    .try_collect()
                    .unwrap(),
            }
        );

        for (index, ident, value) in idents {
            assert_eq!(value, values[index]);
            let hash = hasher(&value);
            let is_eq = |other: &&str| other.eq(&value);

            assert_eq!(table.get(hash, is_eq), Some(&value));
            assert_eq!(table.find(hash, is_eq), Some(ident));
            assert_eq!(unsafe { table.get_unchecked(ident) }, &value);
        }
    }
}
