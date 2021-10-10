use core::iter::FusedIterator;
use core::{array, fmt, iter};

use crate::ext::ToIter;

#[derive(Clone)]
pub struct FlatIter<T, const N: usize> {
    iter: array::IntoIter<Option<T>, N>,
}

impl<T, const N: usize> FlatIter<T, N> {
    pub(crate) fn new(value: [Option<T>; N]) -> Self {
        Self {
            iter: value.into_iter(),
        }
    }
}

impl<T, const N: usize> Iterator for FlatIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().flatten().next()
    }
}

impl<T, const N: usize> DoubleEndedIterator for FlatIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.by_ref().flatten().next_back()
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for FlatIter<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, const N: usize> FusedIterator for FlatIter<T, N> {}

impl<T, const N: usize> ToIter for FlatIter<T, N> {
    type Item = T;
    type Iter<'b>
    where
        Self::Item: 'b,
    = iter::Flatten<<array::IntoIter<Option<T>, N> as ToIter>::Iter<'b>>;

    fn iter(&self) -> Self::Iter<'_> {
        self.iter.iter().flatten()
    }
}
