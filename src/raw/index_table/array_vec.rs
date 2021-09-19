use core::{array, iter, mem};

use crate::errors::{CapacityError, UnavailableMutError};
use crate::ext::TryFromIterator;
use crate::raw::TableIndex;
use crate::utils::{ArrayExt, UnwrapExpectExt};
use crate::{invariant, utils};

use super::FlatIter;

/// A primitive implementation of an ArrayVec.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayVec<T, const N: usize> {
    pub(super) data: [Option<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    /// Returns the number of elements in the vector.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector is empty, `false` if it is not empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the maximum number of elements that can be stored in the vector.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Pushes an element to the end of the vector.
    pub fn push(&mut self, value: T) -> Result<TableIndex<N>, CapacityError> {
        if self.len() == self.capacity() {
            return Err(CapacityError);
        }

        unsafe {
            invariant!(self.len < self.capacity());
            *self.data.get_unchecked_mut(self.len) = Some(value);
        }
        self.len += 1;

        Ok(unsafe { TableIndex::new(self.len() - 1) })
    }

    /// Removes the last element from the vector and returns it.
    ///
    /// If the vector is empty, `None` is returned.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.len -= 1;
        // SAFETY: the data structure is based on the assumption that everything up to
        //         self.len() - 1 is filled with values
        //         and for the length the following must be true:
        //         self.len() <= self.capacity()
        //         with the previous statement self.len has been decremented
        //         => self.len() < self.capacity()
        let value = unsafe {
            invariant!(self.len < self.data.len());
            self.data
                .get_unchecked_mut(self.len)
                .take()
                .expect_unchecked("entries before vec.len must be present")
        };

        Some(value)
    }

    #[inline]
    pub unsafe fn swap(&mut self, a: TableIndex<N>, b: TableIndex<N>) {
        if self.is_empty() || a == b {
            return;
        }

        invariant!(a.index() < self.data.len());
        invariant!(b.index() < self.data.len());
        self.data.swap(a.index(), b.index())
    }

    /// # Safety
    ///
    /// This method assumes that the index is valid (`index < self.len()`).
    ///
    /// # Complexity
    ///
    /// This completes in `O(n)` time.
    pub unsafe fn remove_unchecked(&mut self, index: TableIndex<N>) -> T {
        invariant!(index.index() < self.len());
        let mut last_index = index;

        for index in index.index() + 1..self.len() {
            let index = TableIndex::new(index);
            self.swap(last_index, index);
            last_index = index;
        }

        self.pop().expect_unchecked("the vec must notb e empty")
    }

    /// Returns an immutable reference to the value at the given index.
    ///
    /// # Safety
    ///
    /// The index must be less than the value returned by [`ArrayVec::len`].
    pub unsafe fn get_unchecked(&self, index: TableIndex<N>) -> &T {
        let index = index.index();
        invariant!(index < self.data.len());
        self.data
            .get_unchecked(index)
            .as_ref()
            .expect_unchecked("entries before vec.len must be present")
    }

    /// Returns a mutable reference to the value at the given index.
    ///
    /// # Safety
    ///
    /// The index must be less than the value returned by [`ArrayVec::len`].
    #[must_use]
    pub unsafe fn get_unchecked_mut(&mut self, index: TableIndex<N>) -> &mut T {
        let index = index.index();
        invariant!(index < self.data.len());
        self.data
            .get_unchecked_mut(index)
            .as_mut()
            .expect_unchecked("entries before vec.len must be present")
    }

    pub fn get_each_mut_option<const M: usize>(
        &mut self,
        indices: [Option<TableIndex<N>>; M],
    ) -> [Result<&mut T, UnavailableMutError>; M] {
        let mut data = self.data.each_mut().map(Option::as_mut);
        let mut borrowed: [Option<usize>; N] = [(); N].map(|_| None);

        indices.enumerate().map(|(position, index)| {
            let index = index.ok_or_else(|| UnavailableMutError::Absent)?.index();

            unsafe {
                invariant!(index < data.len() && index < borrowed.len());

                if let Some(value) = data.get_unchecked_mut(index).take() {
                    *borrowed.get_unchecked_mut(index) = Some(position);
                    Ok(value)
                } else if let Some(position) = borrowed.get_unchecked(index) {
                    Err(UnavailableMutError::Duplicate(*position))
                } else {
                    Err(UnavailableMutError::Absent)
                }
            }
        })
    }

    #[must_use]
    pub fn drain(&mut self) -> FlatIter<T, N> {
        let data = mem::replace(&mut self.data, utils::none_array());
        self.len = 0;

        FlatIter::new(data)
    }
}

impl<T, const N: usize> TryFromIterator<T> for ArrayVec<T, N> {
    type Error = CapacityError;

    fn try_from_iter<I: IntoIterator<Item = T>>(iter: I) -> Result<Self, Self::Error> {
        let mut vec = Self::default();

        for item in iter {
            vec.push(item)?;
        }

        Ok(vec)
    }
}

impl<T, const N: usize> IntoIterator for ArrayVec<T, N> {
    type IntoIter = iter::Flatten<array::IntoIter<Option<T>, N>>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter().flatten()
    }
}

impl<T, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self {
            data: utils::none_array(),
            len: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ext::IteratorExt;

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_default() {
        struct DoesNotImplementDefault;

        let vec: ArrayVec<DoesNotImplementDefault, 5> = ArrayVec::default();
        assert_eq!(vec.is_empty(), true);
    }

    #[test]
    fn test_push() {
        const N: usize = 21;

        let values: [usize; N] = (0..N).try_collect().unwrap();

        let mut vec: ArrayVec<_, N> = ArrayVec::default();

        let inserted = values.map(|value| {
            let index = vec.push(value).unwrap();
            (index, value)
        });

        assert_eq!(vec.len(), N);
        assert_eq!(vec.capacity(), inserted.len());
        assert_eq!(vec.is_empty(), false);

        for (index, value) in inserted {
            let array_value = unsafe { vec.get_unchecked(index) };

            assert_eq!(array_value, &value);
        }
    }

    #[test]
    fn test_remove() {
        const N: usize = 5;

        let values: [usize; N] = (0..N).try_collect().unwrap();

        let mut vec: ArrayVec<_, N> = ArrayVec::default();

        for value in values {
            vec.push(value).unwrap();
        }

        assert_eq!(unsafe { vec.remove_unchecked(TableIndex::new(1)) }, 1);

        for value in 0..N {
            if value == 1 {
                // this value has been removed :(
                continue;
            }

            if value == 0 {
                assert_eq!(unsafe { vec.get_unchecked(TableIndex::new(0)) }, &0);
                continue;
            }

            let index = unsafe { TableIndex::new(value - 1) };
            assert_eq!(unsafe { vec.get_unchecked(index) }, &value);
        }
    }
}
