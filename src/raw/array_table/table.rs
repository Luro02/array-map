use core::{array, fmt, iter, mem, slice};

use super::{DrainIter, IterMut};

use crate::errors::{CapacityError, UnavailableMutError};
use crate::raw::{RawTable, RawTableIter, TableIndex};
use crate::utils::{self, unwrap_unchecked, ArrayExt, IterCircular};
use crate::{invariant, unreachable_unchecked};

#[derive(Clone, Copy, PartialEq)]
pub struct ArrayTable<T, const N: usize> {
    data: [Option<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayTable<T, N> {
    #[cfg(test)]
    pub(crate) fn from_array(array: [Option<T>; N]) -> Self {
        let len = array.iter().filter_map(Option::as_ref).count();

        Self { data: array, len }
    }

    #[must_use]
    fn find_insert_slot(&self, hash: u64) -> Option<usize> {
        let index = utils::adjust_hash::<N>(hash);

        IterCircular::new(index, &self.data).find_map(|(index, entry)| {
            if entry.is_some() {
                None
            } else {
                Some(index)
            }
        })
    }

    fn restore_order(
        &mut self,
        index: TableIndex<N>,
        stop: TableIndex<N>,
        hasher: impl Fn(&T) -> u64,
    ) -> Option<TableIndex<N>> {
        let index_to_fill = index.index();
        let stop_index = stop.index();

        unsafe {
            invariant!(index_to_fill < self.data.len());

            if self.data.get_unchecked(index_to_fill).is_some() {
                return None;
            }
        }

        let mut last_entry_fill = None;
        // skip(1), so one does not check the vacant entry at index_to_fill
        for (index, entry) in IterCircular::new(index_to_fill, &self.data).skip(1) {
            if index == stop_index {
                break;
            }

            if let Some(entry) = entry {
                let hash = hasher(entry);
                let expected_index = utils::adjust_hash::<N>(hash);

                // skip entries that are correctly placed
                if expected_index == index {
                    continue;
                }

                let current_distance = {
                    if utils::likely(expected_index <= index) {
                        index - expected_index
                    } else {
                        // wrap around
                        self.capacity() - expected_index + index
                    }
                };

                let new_distance = {
                    if utils::likely(expected_index <= index_to_fill) {
                        index_to_fill - expected_index
                    } else {
                        self.capacity() - expected_index + index_to_fill
                    }
                };

                if current_distance > new_distance {
                    last_entry_fill = Some(index);
                }
            } else {
                // reached an empty entry, so one can stop search for possible fillers
                break;
            }
        }

        if let Some(last_entry_fill) = last_entry_fill {
            invariant!(last_entry_fill < self.data.len());
            invariant!(index_to_fill < self.data.len());

            self.data.swap(index_to_fill, last_entry_fill);
            Some(unsafe { TableIndex::new(last_entry_fill) })
        } else {
            None
        }
    }
}

impl<T, const N: usize> RawTable<T> for ArrayTable<T, N> {
    type DrainIter = DrainIter<T, N>;
    type Ident = TableIndex<N>;
    type InsertError = CapacityError;
    type RawIter = iter::Flatten<array::IntoIter<Option<Self::Ident>, N>>;

    unsafe fn into_mut(self: &mut Self, ident: Self::Ident) -> &mut T {
        let index = ident.index();
        invariant!(index < self.data.len());
        invariant!(self.data[index].is_some());

        unwrap_unchecked(self.data.get_unchecked_mut(index).as_mut())
    }

    fn find(&self, hash: u64, mut eq: impl FnMut(&T) -> bool) -> Option<Self::Ident> {
        let index = utils::adjust_hash::<N>(hash);

        for (index, entry) in IterCircular::new(index, &self.data) {
            if let Some(entry) = entry.as_ref() {
                if eq(entry) {
                    let table_index = unsafe {
                        invariant!(index < self.data.len());
                        TableIndex::new(index)
                    };
                    return Some(table_index);
                }
            } else {
                break;
            }
        }

        None
    }

    unsafe fn get_unchecked(&self, ident: Self::Ident) -> &T {
        let index = ident.index();
        invariant!(index < self.data.len());
        invariant!(self.data[index].is_some());

        unwrap_unchecked(self.data.get_unchecked(index).as_ref())
    }

    unsafe fn get_unchecked_mut(&mut self, ident: Self::Ident) -> &mut T {
        let index = ident.index();
        invariant!(index < self.data.len());
        invariant!(self.data[index].is_some());

        unwrap_unchecked(self.data.get_unchecked_mut(index).as_mut())
    }

    unsafe fn erase(&mut self, ident: Self::Ident) -> T {
        let index = ident.index();
        let entry = unwrap_unchecked(self.data.get_unchecked_mut(index).take());

        entry
    }

    // TODO: hasher needed? (one might just remove it?)
    fn try_insert(
        &mut self,
        hash: u64,
        value: T,
        _: impl Fn(&T) -> u64,
    ) -> Result<Self::Ident, Self::InsertError> {
        let index = self.find_insert_slot(hash).ok_or(CapacityError)?;

        unsafe {
            *self.data.get_unchecked_mut(index) = Some(value);
        }

        self.len += 1;

        unsafe { Ok(TableIndex::new(index)) }
    }

    fn iter(&self) -> Self::RawIter {
        let result = self.data.each_ref().enumerate().map(|(index, entry)| {
            // SAFETY: the entry is present, so the index points to an occupied entry and is
            //         less than N
            if entry.is_some() {
                unsafe { Some(TableIndex::new(index)) }
            } else {
                None
            }
        });

        array::IntoIter::new(result).flatten()
    }

    unsafe fn remove(&mut self, ident: Self::Ident, hasher: impl Fn(&T) -> u64) -> T {
        let old_entry = self.erase(ident);
        self.len -= 1;

        let start = TableIndex::new(utils::adjust_hash::<N>(hasher(&old_entry)));
        let mut ident = ident;
        while let Some(index) = self.restore_order(ident, start, |v| hasher(v)) {
            ident = index;
        }

        old_entry
    }

    fn drain(&mut self) -> Self::DrainIter {
        let data = mem::replace(&mut self.data, utils::none_array());
        self.len = 0;

        DrainIter::new(data)
    }

    fn capacity(&self) -> usize {
        N
    }

    fn len(&self) -> usize {
        self.len
    }

    fn get_each_mut<const M: usize>(
        &mut self,
        hashes: [u64; M],
        mut eq: impl FnMut(usize, &T) -> bool,
    ) -> [Result<&mut T, UnavailableMutError>; M] {
        // if an entry is already borrowed then an index will be present, which points
        // to the mutable reference in the resulting array
        let mut borrowed: [Option<usize>; N] = utils::none_array();
        // map each hash to it's index in the table (TableIndex):
        let table_indices = hashes
            .enumerate()
            .map(|(i, hash)| self.find(hash, |k| eq(i, k)));

        let mut entries: [Option<&mut Option<T>>; N] = self.data.each_mut().map(Some);

        table_indices.enumerate().map(|(result_index, table_index)| {
            let table_index = table_index.ok_or(UnavailableMutError::Absent)?;
            let index = table_index.index();

            invariant!(index < entries.len() && index < borrowed.len());
            unsafe {
                if let Some(Some(entry)) = entries.get_unchecked_mut(index).take() {
                    *borrowed.get_unchecked_mut(index) = Some(result_index);
                    Ok(entry)
                } else if let Some(idx) = borrowed.get_unchecked(index) {
                    Err(UnavailableMutError::Duplicate(*idx))
                } else {
                    unreachable_unchecked!("the entry should be present in entries or an entry in borrowed must be present")
                }
            }
        })
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayTable<T, N> {
    type IntoIter = iter::Flatten<slice::Iter<'a, Option<T>>>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter().flatten()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayTable<T, N> {
    type IntoIter = iter::Flatten<slice::IterMut<'a, Option<T>>>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut().flatten()
    }
}

impl<T, const N: usize> RawTableIter<T> for ArrayTable<T, N> {
    type Iter<'a>
    where
        T: 'a,
    = <&'a Self as IntoIterator>::IntoIter;
    type IterMut<'a>
    where
        T: 'a,
    = IterMut<'a, T>;

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        IterMut::new(self.data.iter_mut())
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.into_iter()
    }
}

impl<T, const N: usize> IntoIterator for ArrayTable<T, N> {
    type IntoIter = iter::Flatten<array::IntoIter<Option<T>, N>>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        array::IntoIter::new(self.data).flatten()
    }
}

impl<T, const N: usize> Default for ArrayTable<T, N> {
    fn default() -> Self {
        Self {
            data: utils::none_array(),
            len: 0,
        }
    }
}

impl<T, const N: usize> fmt::Debug for ArrayTable<T, N>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries((&self.data).iter().enumerate())
            .finish()
    }
}
