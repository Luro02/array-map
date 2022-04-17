use core::{array, iter, mem};

use crate::errors::{CapacityError, UnavailableMutError};
use crate::invariant;
use crate::raw::{ArrayTable, FixedSizeTable, IterMut, RawTable, RawTableIter, TableIndex};
use crate::utils::{ArrayExt, UnwrapExpectExt};

use super::{ArrayVec, FlatIter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndexTable<T, R, const N: usize>
where
    // TODO: maybe this should be: (u64, TableIndex<N>)?
    R: FixedSizeTable<TableIndex<N>, N>,
{
    /// Mapping from the entry hash to its index.
    indices: R,
    /// Entries in their order.
    entries: ArrayVec<T, N>,
}

pub type ArrayIndexTable<T, const N: usize> = IndexTable<T, ArrayTable<TableIndex<N>, N>, N>;

impl<T, R: FixedSizeTable<TableIndex<N>, N>, const N: usize> IndexTable<T, R, N> {
    /// Removes an entry from the table, preserving the insertion order by
    /// shifting all the following elements to the left.
    pub unsafe fn shift_remove(
        &mut self,
        ident: <Self as RawTable<T>>::Ident,
        hasher: impl Fn(&T) -> u64,
    ) -> T {
        let entry_index = self.indices.remove(ident, |value| unsafe {
            hasher(self.entries.get_unchecked(*value))
        });

        let result = self.entries.remove_unchecked(entry_index);

        // all indices that have been moved must be updated:

        if self.entries.is_empty() {
            return result;
        }

        for new_index in entry_index.index()..self.entries.len() {
            let old_index = TableIndex::new(new_index + 1);
            let new_index = TableIndex::new(new_index);
            let hash = hasher(self.entries.get_unchecked(new_index));

            let value = self
                .indices
                .get_mut(hash, |other| other == &old_index)
                .expect_unchecked("the entry should still be present");
            *value = new_index;
        }

        result
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
                if *idx == index {
                    hash
                } else {
                    hasher(self.entries.get_unchecked(*idx))
                }
            })
        };

        Some(entry)
    }

    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<&T> {
        // SAFETY: It is checked that the index is valid
        unsafe {
            if index < self.entries.len() {
                Some(self.entries.get_unchecked(TableIndex::new(index)))
            } else {
                None
            }
        }
    }

    #[must_use]
    pub fn get_index_mut(&mut self, index: usize) -> Option<&mut T> {
        // SAFETY: It is checked that the index is valid
        unsafe {
            if index < self.entries.len() {
                Some(self.entries.get_unchecked_mut(TableIndex::new(index)))
            } else {
                None
            }
        }
    }

    /// Swaps the position of two entries.
    ///
    /// # Safety
    ///
    /// Both indices must be valid, which implies that this map is not empty.
    pub unsafe fn swap(
        &mut self,
        a: <Self as RawTable<T>>::Ident,
        b: <Self as RawTable<T>>::Ident,
    ) {
        if a == b {
            return;
        }
        invariant!(!self.is_empty());

        // swap the indices in the indices table of the entries:

        // obtain the index of a
        let a_index = *self.indices.get_unchecked(a.clone());

        // replace the index of b with that of a
        let b_index = mem::replace(self.indices.get_unchecked_mut(b), a_index);

        // assign the index of b to a
        *self.indices.get_unchecked_mut(a) = b_index;

        // swap the entries in the entries vec:
        self.entries.swap(a_index, b_index);
    }

    #[must_use]
    pub fn ident_from_index(
        &self,
        index: TableIndex<N>,
        hasher: impl Fn(&T) -> u64,
    ) -> Option<<Self as RawTable<T>>::Ident> {
        let hash = hasher(self.get_index(index.index())?);

        self.indices.find(hash, |other| other == &index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexTableError<E> {
    CapacityError(CapacityError),
    Table(E),
}

impl<E> From<CapacityError> for IndexTableError<E> {
    fn from(value: CapacityError) -> Self {
        Self::CapacityError(value)
    }
}

// TODO: implement fmt::Display, Debug, Clone, ... for IndexTableError

impl<T, R: FixedSizeTable<TableIndex<N>, N>, const N: usize> RawTable<T> for IndexTable<T, R, N> {
    type DrainIter = FlatIter<T, N>;
    type Ident = R::Ident;
    type InsertError = IndexTableError<R::InsertError>;
    type RawIter = R::RawIter;

    fn find(&self, hash: u64, mut eq: impl FnMut(&T) -> bool) -> Option<Self::Ident> {
        self.indices.find(hash, |index| unsafe {
            let value = self.entries.get_unchecked(*index);
            eq(value)
        })
    }

    unsafe fn get_unchecked(&self, ident: Self::Ident) -> &T {
        let index = self.indices.get_unchecked(ident);
        self.entries.get_unchecked(*index)
    }

    unsafe fn get_unchecked_mut(&mut self, ident: Self::Ident) -> &mut T {
        let index = self.indices.get_unchecked(ident);
        self.entries.get_unchecked_mut(*index)
    }

    unsafe fn erase(&mut self, ident: Self::Ident) -> T {
        let index = self.indices.erase(ident);

        self.entries.remove_unchecked(index)
    }

    fn try_insert(
        &mut self,
        hash: u64,
        value: T,
        hasher: impl Fn(&T) -> u64,
    ) -> Result<Self::Ident, Self::InsertError> {
        let index = self.entries.push(value)?;
        let ident = self
            .indices
            .try_insert(hash, index, |ident| unsafe {
                hasher(self.entries.get_unchecked(*ident))
            })
            .map_err(Self::InsertError::Table)?;
        Ok(ident)
    }

    fn iter_idents(&self) -> Self::RawIter {
        self.indices.iter_idents()
    }

    /// # Note
    ///
    /// This does not preserve the order of the removed value.
    /// Use `shift_remove` if the order should be preserved!
    unsafe fn remove(&mut self, ident: Self::Ident, hasher: impl Fn(&T) -> u64) -> T {
        invariant!(!self.entries.is_empty());

        let last_index = TableIndex::new(self.entries.len() - 1);
        let last_hash = hasher(self.entries.get_unchecked(last_index));

        let last_index_ident = self
            .indices
            .find(last_hash, |other| other == &last_index)
            .expect_unchecked("the last index should be present");

        self.swap(ident, last_index_ident);

        // remove the ident from the indices table (now at the last_index!)

        // first get the ident of the table:
        let ident = self
            .indices
            .find(hasher(self.entries.get_unchecked(last_index)), |other| {
                other == &last_index
            })
            .expect_unchecked("ident must exist");

        self.indices.remove(ident, |ident| unsafe {
            hasher(self.entries.get_unchecked(*ident))
        });

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

impl<T, R: FixedSizeTable<TableIndex<N>, N>, const N: usize> IntoIterator for IndexTable<T, R, N> {
    // TODO: custom iterator
    type IntoIter = iter::Flatten<array::IntoIter<Option<T>, N>>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<T, R: FixedSizeTable<TableIndex<N>, N>, const N: usize> FixedSizeTable<T, N>
    for IndexTable<T, R, N>
{
}

impl<T, R: FixedSizeTable<TableIndex<N>, N>, const N: usize> RawTableIter<T>
    for IndexTable<T, R, N>
{
    type Iter<'a> = FlatIter<&'a T, N>
    where
        T: 'a,
        R: 'a;
    type IterMut<'a> = IterMut<'a, T>
    where
        T: 'a,
        R: 'a;

    /// Returns a mutable iterator over the table.
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        IterMut::new(self.entries.data.iter_mut())
    }

    /// Returns an immutable iterator over the table.
    fn iter(&self) -> Self::Iter<'_> {
        FlatIter::new(self.entries.data.each_ref().map(Option::as_ref))
    }
}

impl<T, R, const N: usize> Default for IndexTable<T, R, N>
where
    R: FixedSizeTable<TableIndex<N>, N> + Default,
{
    fn default() -> Self {
        Self {
            indices: R::default(),
            entries: ArrayVec::default(),
        }
    }
}

#[cfg(test)]
impl<T, R, const N: usize> From<(R, ArrayVec<T, N>)> for IndexTable<T, R, N>
where
    R: FixedSizeTable<TableIndex<N>, N>,
{
    fn from(value: (R, ArrayVec<T, N>)) -> Self {
        Self {
            indices: value.0,
            entries: value.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ext::IteratorExt;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_default() {
        let table: IndexTable<&str, ArrayTable<_, 5>, 5> = IndexTable::default();
        assert_eq!(table.is_empty(), true);
        assert_eq!(table.capacity(), 5);
    }

    #[test]
    fn test_into_iter() {
        const N: usize = 5;
        let mut table: IndexTable<u64, ArrayTable<_, N>, N> = IndexTable::default();

        for i in (0..N as u64).rev() {
            table.try_insert((N as u64) - i, i, |x| *x).unwrap();
        }

        let mut iter = table.into_iter();

        for i in (0..=4).rev() {
            assert_eq!(iter.next(), Some(i));
        }

        assert_eq!(iter.next(), None);
    }

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

    #[test]
    fn test_into_iter_order_shift_remove() {
        const N: usize = 5;
        let mut table: ArrayIndexTable<Value, N> = ArrayIndexTable::default();

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

        let mut table: ArrayIndexTable<Value, N> = ArrayIndexTable::default();

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
        let mut table: ArrayIndexTable<&str, N> = ArrayIndexTable::default();

        fn hasher(value: &&str) -> u64 {
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
            ArrayIndexTable {
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

    #[test]
    fn test_swap() {
        let mut table: ArrayIndexTable<Value, 7> = ArrayIndexTable::default();

        let idents = [Value::Foo, Value::Baz, Value::Bar, Value::Avc, Value::Bvc].map(|value| {
            table
                .try_insert(Value::hasher(&value), value, Value::hasher)
                .unwrap()
        });

        // Value::Foo and Value::Baz
        unsafe { table.swap(idents[0], idents[1]) };
        assert_eq!(
            [
                Some(&Value::Baz),
                Some(&Value::Foo),
                Some(&Value::Bar),
                Some(&Value::Avc),
                Some(&Value::Bvc),
                None,
                None,
            ],
            table.iter().try_collect::<[_; 7]>().unwrap()
        );
        // Value::Baz and Value::Bar
        unsafe { table.swap(idents[1], idents[2]) };
        assert_eq!(
            [
                Some(&Value::Bar),
                Some(&Value::Foo),
                Some(&Value::Baz),
                Some(&Value::Avc),
                Some(&Value::Bvc),
                None,
                None,
            ],
            table.iter().try_collect::<[_; 7]>().unwrap()
        );
        // Value::Avc and Value::Bvc
        unsafe { table.swap(idents[3], idents[4]) };
        assert_eq!(
            [
                Some(&Value::Bar),
                Some(&Value::Foo),
                Some(&Value::Baz),
                Some(&Value::Bvc),
                Some(&Value::Avc),
                None,
                None,
            ],
            table.iter().try_collect::<[_; 7]>().unwrap()
        );
    }
}
